// doctor — the runtime health verdict, and the Codex attestation it needs.
//
// WHY THIS EXISTS AT ALL. `bee doctor` was declared in the command registry
// and never ported; the R6 Node deletion removed the only implementation. Its
// own unavailable-marker said what that costs:
//
//     a Codex install cannot be attested until doctor is ported, so treat
//     that runtime as degraded
//
// So bee shipped calling one of its two supported runtimes degraded, by its
// own words, with no way to lift the verdict.
//
// WHAT WAS PORTED, AND WHAT WAS NOT. There is no surviving doctor source and
// no spec file — the contract is the registry description, which is precise
// about the verdict ladder and the attestation legs. Three of its four
// mechanical rows map onto artifacts that still exist and are checked here.
// The fourth, "capability-baseline byte match", named an artifact that no
// longer exists anywhere in the tree. Rather than invent a baseline and grade
// against it — a row that always passes is worse than no row — it is replaced
// by the byte match that IS meaningful today: the host's rendered hook
// manifest against what this binary renders for that runtime. Same question
// (does the wiring on disk match the wiring this bee believes in), an artifact
// that exists, and it fails when it should.
//
// THE VERDICT LADDER, verbatim from the contract:
//
//   blocked   any mechanical row is not ok
//   degraded  mechanical rows all ok, but Codex's trust rows are structurally
//             unknown and no valid attestation covers them
//   ready     mechanical rows all ok AND, on Codex, a currently-valid
//             attestation. Claude has no trust-unknown rows, so mechanical
//             green alone reaches ready there.
//
// Never "ready" from file presence alone, and `doctor` itself performs ZERO
// writes — including the dispatcher's manifest-hash cache, which is why it
// probes the store directly rather than going through the status builders.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::jsjson;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const ATTEST_REL: &str = ".bee/doctor-attest.json";

#[derive(Clone, Copy, PartialEq)]
enum Runtime {
    Claude,
    Codex,
}

impl Runtime {
    fn name(self) -> &'static str {
        match self {
            Runtime::Claude => "claude",
            Runtime::Codex => "codex",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(Runtime::Claude),
            "codex" => Some(Runtime::Codex),
            _ => None,
        }
    }
    /// The wiring file the host loads for this runtime.
    fn hooks_rel(self) -> &'static str {
        match self {
            Runtime::Claude => ".claude/settings.json",
            Runtime::Codex => ".codex/hooks.json",
        }
    }
    fn skills_rel(self) -> &'static str {
        match self {
            Runtime::Claude => ".claude/skills",
            Runtime::Codex => ".agents/skills",
        }
    }
}

struct Row {
    key: &'static str,
    ok: Option<bool>, // None == structurally unknown (a trust row)
    detail: String,
}

impl Row {
    fn value(&self) -> Value {
        json!({
            "row": self.key,
            "status": match self.ok {
                Some(true) => "ok",
                Some(false) => "not_ok",
                None => "unknown",
            },
            "detail": self.detail,
        })
    }
}

fn repo_root() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    let cwd = dunce::canonicalize(&cwd).unwrap_or(cwd);
    let mut dir = Some(cwd.as_path());
    while let Some(d) = dir {
        if d.join(".bee").is_dir() {
            return Some(d.to_path_buf());
        }
        dir = d.parent();
    }
    None
}

fn host_binary(root: &Path) -> Option<PathBuf> {
    for name in ["bee", "bee.exe"] {
        let p = root.join(".bee").join("bin").join(name);
        if p.is_file() {
            return Some(p);
        }
    }
    None
}

fn sha256_of(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

/// The four mechanical rows, plus a fifth that exists only in a bee SOURCE
/// checkout (`binary_freshness_row`, appended below — see its own doc). Every
/// one of the first four reads an artifact that exists; a row with nothing to
/// read reports not_ok, never ok-by-absence. The fifth is the one deliberate
/// exception: a missing installed binary is not_ok on `hook_handler` already,
/// so `binary_freshness` reports unknown there rather than repeating the
/// same verdict under a second name.
fn mechanical_rows(root: &Path, runtime: Runtime) -> Vec<Row> {
    let mut rows = Vec::new();

    let hooks_path = root.join(runtime.hooks_rel());
    let hooks_bytes = std::fs::read(&hooks_path).ok();
    rows.push(Row {
        key: "hooks_file",
        ok: Some(hooks_bytes.is_some()),
        detail: match &hooks_bytes {
            Some(b) => format!("{} present ({} bytes)", runtime.hooks_rel(), b.len()),
            None => format!("{} is missing — the runtime loads no bee hooks", runtime.hooks_rel()),
        },
    });

    // Hook-handler resolvability: the path every wired command names must
    // exist and be the thing that answers `hook`.
    let bin = host_binary(root);
    rows.push(Row {
        key: "hook_handler",
        ok: Some(bin.is_some()),
        detail: match &bin {
            Some(p) => format!("{} resolves", p.display()),
            None => ".bee/bin/bee[.exe] is missing — every wired hook command points at nothing"
                .to_string(),
        },
    });

    let skills_dir = root.join(runtime.skills_rel());
    let skill_count = std::fs::read_dir(&skills_dir)
        .map(|e| e.filter_map(|x| x.ok()).filter(|x| x.path().is_dir()).count())
        .unwrap_or(0);
    rows.push(Row {
        key: "skills_installed",
        ok: Some(skill_count > 0),
        detail: if skill_count > 0 {
            format!("{} skill(s) under {}", skill_count, runtime.skills_rel())
        } else {
            format!("no skills under {} — the agent has no bee craft to load", runtime.skills_rel())
        },
    });

    // The byte match. See the header: this stands in for the retired
    // capability baseline, against an artifact that still exists.
    //
    // THE TWO RUNTIMES ARE NOT THE SAME SHAPE, and treating them alike is a
    // false FAIL. `.codex/hooks.json` IS the rendered artifact, so whole-file
    // equality is the right question. `.claude/settings.json` is the user's
    // settings file that onboarding MERGES a `hooks` key into — it also
    // carries permissions and anything else the host put there, none of which
    // bee renders. Comparing the whole file there fails every correctly
    // installed repo. Only the `hooks` subtree is bee's to answer for.
    let rendered = crate::devtools::render_projection_text_for(runtime.name());
    rows.push(match runtime {
        // `.codex/hooks.json` IS the rendered artifact — onboarding copies it,
        // so whole-file equality is exactly the question, and a host running a
        // different bee is caught.
        Runtime::Codex => match (hooks_bytes.as_ref(), rendered) {
            (Some(on_disk), Some(expected)) => {
                let same = sha256_of(on_disk) == sha256_of(expected.as_bytes());
                Row {
                    key: "wiring_matches_binary",
                    ok: Some(same),
                    detail: if same {
                        ".codex/hooks.json is byte-identical to what this bee renders".to_string()
                    } else {
                        ".codex/hooks.json differs from what this bee renders — re-run the installer to refresh it".to_string()
                    },
                }
            }
            _ => Row {
                key: "wiring_matches_binary",
                ok: Some(false),
                detail: "no .codex/hooks.json to compare".to_string(),
            },
        },
        // `.claude/settings.json` is the USER's file, with a hooks key merged
        // in; it also carries permissions and whatever else the host put
        // there, and the per-repo renderer feature-detects bee vs bee.exe. So
        // byte equality against a shipped projection is the wrong question and
        // fails every correct install. What must hold is that the wiring is
        // bee's and points at the vendored binary — which is precisely what
        // broke when the installer copied the binary in after onboarding.
        Runtime::Claude => {
            let parsed: Option<Value> =
                hooks_bytes.as_ref().and_then(|b| serde_json::from_slice(b).ok());
            let mut total = 0usize;
            let mut wrong: Vec<String> = Vec::new();
            if let Some(Value::Object(hooks)) = parsed.as_ref().and_then(|v| v.get("hooks")) {
                for groups in hooks.values() {
                    for g in groups.as_array().into_iter().flatten() {
                        for h in g.get("hooks").and_then(Value::as_array).into_iter().flatten() {
                            let cmd = h.get("command").and_then(Value::as_str).unwrap_or("");
                            total += 1;
                            if !cmd.contains(".bee/bin/bee") {
                                wrong.push(cmd.to_string());
                            }
                        }
                    }
                }
            }
            Row {
                key: "wiring_points_at_the_binary",
                ok: Some(total > 0 && wrong.is_empty()),
                detail: if total == 0 {
                    ".claude/settings.json wires no bee hooks at all".to_string()
                } else if wrong.is_empty() {
                    format!("{total} hook command(s), all invoking .bee/bin/bee")
                } else {
                    format!(
                        "{} of {total} hook command(s) do not invoke .bee/bin/bee: {}",
                        wrong.len(),
                        wrong.join("; ")
                    )
                },
            }
        }
    });

    if let Some(row) = binary_freshness_row(root) {
        rows.push(row);
    }

    rows
}

/// Source that ships without reinstalling the binary the hooks call is
/// inert — a pattern this repo has paid for more than once (four features
/// shipped to main in one session with `.bee/bin/bee` never rebuilt). This
/// row gives that pattern a machine owner.
///
/// It exists ONLY in a bee SOURCE checkout, detected the same neighbourhood
/// `devtools::SOURCE_CHECKOUT_DEV_VERBS` gates on: `packages/bee-rs/Cargo.toml`
/// present under the repo root. A host project carries no such tree, so the
/// row is absent there entirely — never a false alarm from a distributed
/// binary that never had source to lag.
///
/// In a source checkout it is not_ok when either (a) the installed binary's
/// own `rs-info` version disagrees with the source workspace version, or (b)
/// any source input — `packages/bee-rs/crates/**/*.rs`,
/// `packages/bee-rs/**/Cargo.toml`, `packages/bee/prompts/*.md` — is newer by
/// mtime than the installed binary. Read-only throughout: it only stats and
/// reads files and spawns the installed binary to ask its own version, never
/// builds, copies, or writes anything.
fn binary_freshness_row(root: &Path) -> Option<Row> {
    const KEY: &str = "binary_freshness";
    const REMEDY: &str = "FIX: cargo build --release --manifest-path packages/bee-rs/Cargo.toml \
        -p bee --bin bee, then copy target/release/bee to .bee/bin/bee.";

    let workspace_cargo = root.join("packages/bee-rs/Cargo.toml");
    let source_version = std::fs::read_to_string(&workspace_cargo)
        .ok()
        .and_then(|text| parse_workspace_version(&text))?;

    // Missing binary is `hook_handler`'s verdict to give; repeating not_ok
    // here under a second name would just be noise, so this reports unknown.
    let Some(bin) = host_binary(root) else {
        return Some(Row {
            key: KEY,
            ok: None,
            detail: "no installed binary to check for freshness (see hook_handler)".to_string(),
        });
    };

    if let Some(installed_version) = installed_binary_version(&bin) {
        if installed_version != source_version {
            return Some(Row {
                key: KEY,
                ok: Some(false),
                detail: format!(
                    "installed binary reports version {installed_version}, source \
                     (packages/bee-rs/Cargo.toml) is {source_version}. {REMEDY}"
                ),
            });
        }
    }

    if let Ok(bin_mtime) = std::fs::metadata(&bin).and_then(|m| m.modified()) {
        let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
        for path in source_inputs(root) {
            let Ok(mtime) = std::fs::metadata(&path).and_then(|m| m.modified()) else { continue };
            if mtime > bin_mtime && newest.as_ref().is_none_or(|(_, t)| mtime > *t) {
                newest = Some((path, mtime));
            }
        }
        if let Some((path, mtime)) = newest {
            let rel = path.strip_prefix(root).unwrap_or(&path);
            return Some(Row {
                key: KEY,
                ok: Some(false),
                detail: format!(
                    "{} was modified {} (binary is {}). {REMEDY}",
                    rel.display(),
                    fmt_system_time(mtime),
                    fmt_system_time(bin_mtime)
                ),
            });
        }
    }

    Some(Row {
        key: KEY,
        ok: Some(true),
        detail: format!(
            "installed binary matches source (version {source_version}), no source input newer \
             than the binary"
        ),
    })
}

/// `version = "…"` under `[workspace.package]` in `packages/bee-rs/Cargo.toml`
/// — a deliberately narrow line scanner, the same idiom `version.rs` uses for
/// the plugin manifest, over a file this repo controls the exact shape of.
fn parse_workspace_version(cargo_toml_text: &str) -> Option<String> {
    let mut in_section = false;
    for line in cargo_toml_text.lines() {
        let trimmed = line.trim();
        if let Some(name) = trimmed.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            in_section = name == "workspace.package";
            continue;
        }
        if !in_section {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("version") else { continue };
        let Some(rest) = rest.trim_start().strip_prefix('=') else { continue };
        return Some(rest.trim().trim_matches('"').to_string());
    }
    None
}

/// The installed binary's own answer to what version it was built from —
/// `bee rs-info`'s `version` field, which is `env!("CARGO_PKG_VERSION")` at
/// that binary's build time. A probe, never a mutation: this only spawns and
/// reads stdout.
fn installed_binary_version(bin: &Path) -> Option<String> {
    let out = std::process::Command::new(bin).arg("rs-info").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let value: Value = serde_json::from_slice(&out.stdout).ok()?;
    value.get("version").and_then(Value::as_str).map(str::to_string)
}

/// The freshness inputs: every `.rs` file and `Cargo.toml` under
/// `packages/bee-rs/crates`, the workspace `packages/bee-rs/Cargo.toml`
/// itself, and every `.md` prompt directly under `packages/bee/prompts`. The
/// walk stays inside `crates/` rather than all of `packages/bee-rs` on
/// purpose — the sibling `target/` build directory lives at the workspace
/// root, not under `crates/`, and a doctor row must never wander into it.
fn source_inputs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk_files(&root.join("packages/bee-rs/crates"), &mut out);
    out.retain(|p| {
        p.extension().is_some_and(|e| e == "rs") || p.file_name().is_some_and(|n| n == "Cargo.toml")
    });

    let workspace_cargo = root.join("packages/bee-rs/Cargo.toml");
    if workspace_cargo.is_file() {
        out.push(workspace_cargo);
    }

    if let Ok(entries) = std::fs::read_dir(root.join("packages/bee/prompts")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out
}

fn walk_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn fmt_system_time(t: std::time::SystemTime) -> String {
    chrono::DateTime::<chrono::Utc>::from(t).format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

/// Codex's trust rows. The contract calls them structurally unknown: Codex
/// exposes no surface that reports whether its hook discovery, trust prompt,
/// project trust or pending review actually let bee's hooks fire. Nothing here
/// can probe them, so they are reported as unknown and answered — if at all —
/// by an attestation a human recorded after checking the /hooks TUI.
const CODEX_TRUST_ROWS: [(&str, &str); 4] = [
    ("hook_discovery", "whether Codex discovered .codex/hooks.json"),
    ("hook_trust", "whether the hooks were trusted in the /hooks TUI"),
    ("project_trust", "whether this project is trusted"),
    ("pending_review", "whether any hook is still awaiting review"),
];

struct Attestation {
    valid: bool,
    reason: &'static str,
}

fn codex_version() -> Option<String> {
    let out = std::process::Command::new("codex").arg("--version").output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!s.is_empty()).then_some(s)
}

/// Repo identity for the attestation's third leg. The canonical root path is
/// enough: an attestation recorded for one checkout must not silently cover a
/// different one.
fn repo_identity(root: &Path) -> String {
    sha256_of(root.to_string_lossy().as_bytes())
}

fn read_attestation(root: &Path, runtime: Runtime) -> Attestation {
    if runtime != Runtime::Codex {
        return Attestation { valid: false, reason: "not_applicable" };
    }
    let ReadJson::Parsed(rec) = read_json(&root.join(ATTEST_REL)) else {
        return Attestation { valid: false, reason: "no_attestation" };
    };
    let hooks = std::fs::read(root.join(runtime.hooks_rel())).unwrap_or_default();
    if rec.get("hooks_sha256").and_then(Value::as_str) != Some(sha256_of(&hooks).as_str()) {
        return Attestation { valid: false, reason: "hash_changed" };
    }
    let live = codex_version();
    match (rec.get("codex_version").and_then(Value::as_str), live.as_deref()) {
        (Some(a), Some(b)) if a == b => {}
        (_, None) => return Attestation { valid: false, reason: "unprobed_version" },
        _ => return Attestation { valid: false, reason: "version_changed" },
    }
    if rec.get("repo_identity").and_then(Value::as_str) != Some(repo_identity(root).as_str()) {
        return Attestation { valid: false, reason: "identity_changed" };
    }
    Attestation { valid: true, reason: "valid" }
}

fn emit(payload: &Value, as_json: bool, lines: &[String]) {
    if as_json {
        print!("{}\n", jsjson::stringify_pretty(payload));
    } else {
        for l in lines {
            println!("{l}");
        }
    }
}

fn run_doctor(runtime: Runtime, as_json: bool) -> ExitCode {
    let Some(root) = repo_root() else {
        let msg = "bee doctor: no bee repo here (looked upward for a .bee/ directory). FIX: run it inside an onboarded project.";
        if as_json {
            print!("{}\n", jsjson::stringify(&json!({"error": msg, "kind": "no_repo"})));
        } else {
            eprintln!("{msg}");
        }
        return ExitCode::from(1);
    };

    let rows = mechanical_rows(&root, runtime);
    let mechanical_ok = rows.iter().all(|r| r.ok == Some(true));

    let mut all: Vec<Value> = rows.iter().map(Row::value).collect();
    let attest = read_attestation(&root, runtime);
    if runtime == Runtime::Codex {
        for (key, what) in CODEX_TRUST_ROWS {
            let detail = if attest.valid {
                format!("{what} — covered by a valid attestation")
            } else {
                format!("{what} — Codex exposes no surface to probe this ({})", attest.reason)
            };
            all.push(json!({
                "row": key,
                "status": if attest.valid { "ok" } else { "unknown" },
                "detail": detail,
            }));
        }
    }

    // Never ready from presence alone: the ladder is evaluated, not assumed.
    let status = if !mechanical_ok {
        "blocked"
    } else if runtime == Runtime::Codex && !attest.valid {
        "degraded"
    } else {
        "ready"
    };

    let payload = json!({
        "runtime": runtime.name(),
        "repo_root": root.to_string_lossy(),
        "overall_status": status,
        "rows": all,
        "attestation": if runtime == Runtime::Codex {
            json!({"valid": attest.valid, "reason": attest.reason, "record": ATTEST_REL})
        } else {
            Value::Null
        },
    });

    let mut lines = vec![format!("bee doctor ({}): {}", runtime.name(), status.to_uppercase())];
    for r in &rows {
        let mark = match r.ok {
            Some(true) => "ok  ",
            Some(false) => "FAIL",
            None => "?   ",
        };
        lines.push(format!("  {mark} {:<22} {}", r.key, r.detail));
    }
    if runtime == Runtime::Codex && !attest.valid {
        lines.push(format!(
            "  ?    codex trust rows        structurally unknown ({}) — review them in Codex's /hooks TUI, then: bee doctor attest --runtime codex",
            attest.reason
        ));
    }
    lines.push(match status {
        "blocked" => "next: fix the FAIL row(s) above — nothing else can be trusted until they are ok".to_string(),
        "degraded" => "next: the wiring is correct; what is unproven is whether Codex is letting it fire".to_string(),
        _ => "next: nothing — this runtime is ready".to_string(),
    });
    emit(&payload, as_json, &lines);
    if status == "blocked" { ExitCode::from(1) } else { ExitCode::SUCCESS }
}

fn run_attest(runtime: Runtime, session: Option<&str>, as_json: bool) -> ExitCode {
    if runtime != Runtime::Codex {
        let msg = "bee doctor attest: --runtime codex only. Claude has no trust-unknown rows, so mechanical green already reaches ready there — there is nothing to attest.";
        if as_json {
            print!("{}\n", jsjson::stringify(&json!({"error": msg, "kind": "not_applicable"})));
        } else {
            eprintln!("{msg}");
        }
        return ExitCode::from(1);
    }
    let Some(root) = repo_root() else {
        eprintln!("bee doctor attest: no bee repo here.");
        return ExitCode::from(1);
    };
    let hooks_path = root.join(runtime.hooks_rel());
    let Ok(hooks) = std::fs::read(&hooks_path) else {
        let msg = format!(
            "bee doctor attest: {} is missing — there is no wiring to attest.",
            runtime.hooks_rel()
        );
        eprintln!("{msg}");
        return ExitCode::from(1);
    };
    // A version we cannot read is not a leg we can pin. Refusing beats
    // recording an attestation that is inert the moment it is written.
    let Some(version) = codex_version() else {
        let msg = "bee doctor attest: `codex --version` did not answer, so the version leg cannot be pinned and the attestation would be inert on sight. FIX: run this where the codex CLI works.";
        if as_json {
            print!("{}\n", jsjson::stringify(&json!({"error": msg, "kind": "unprobed_version"})));
        } else {
            eprintln!("{msg}");
        }
        return ExitCode::from(1);
    };

    let mut rec = Map::new();
    rec.insert("schema".into(), json!("doctor-attest/1"));
    rec.insert("runtime".into(), json!("codex"));
    rec.insert("hooks_sha256".into(), json!(sha256_of(&hooks)));
    rec.insert("codex_version".into(), json!(version));
    rec.insert("repo_identity".into(), json!(repo_identity(&root)));
    rec.insert("recorded_at".into(), json!(chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()));
    if let Some(s) = session {
        rec.insert("session".into(), json!(s));
    }
    let value = Value::Object(rec);
    if let Err(e) = write_json_atomic(&root.join(ATTEST_REL), &value) {
        eprintln!("bee doctor attest: could not write {ATTEST_REL}: {e}");
        return ExitCode::from(1);
    }
    let lines = vec![
        format!("Attested codex trust state for this repo ({ATTEST_REL})."),
        "  It covers exactly this hooks.json content, this codex --version, and this checkout;"
            .to_string(),
        "  any one of the three drifting makes it inert and doctor reports degraded again."
            .to_string(),
        "next: bee doctor --runtime codex".to_string(),
    ];
    emit(&value, as_json, &lines);
    ExitCode::SUCCESS
}

pub fn try_native(args: &[OsString]) -> Option<ExitCode> {
    let strs: Vec<&str> = args.iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if strs.first().copied() != Some("doctor") {
        return None;
    }
    let rest = &strs[1..];
    let attest = rest.first().copied() == Some("attest");
    let flags = if attest { &rest[1..] } else { rest };

    let mut runtime: Option<Runtime> = None;
    let mut session: Option<&str> = None;
    let mut as_json = false;
    let mut i = 0usize;
    while i < flags.len() {
        match flags[i] {
            "--json" => as_json = true,
            "--runtime" => {
                runtime = Runtime::parse(flags.get(i + 1).copied()?);
                runtime?;
                i += 1;
            }
            "--session" => {
                session = flags.get(i + 1).copied();
                session?;
                i += 1;
            }
            _ => return None, // an unproven shape refuses through the catalog
        }
        i += 1;
    }
    // `--runtime` is the one required flag; the catalog says so too, and a
    // missing one must reach its refusal rather than default to a guess.
    let runtime = runtime?;
    Some(if attest { run_attest(runtime, session, as_json) } else { run_doctor(runtime, as_json) })
}

#[cfg(test)]
mod tests;
