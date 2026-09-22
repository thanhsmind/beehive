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
const PI_EXTENSION_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");

#[derive(Clone, Copy, PartialEq, Debug)]
enum Runtime {
    Claude,
    Codex,
    Pi,
}

impl Runtime {
    fn name(self) -> &'static str {
        match self {
            Runtime::Claude => "claude",
            Runtime::Codex => "codex",
            Runtime::Pi => "pi",
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s {
            "claude" => Some(Runtime::Claude),
            "codex" => Some(Runtime::Codex),
            "pi" => Some(Runtime::Pi),
            _ => None,
        }
    }
    /// The wiring file the host loads for this runtime.
    fn hooks_rel(self) -> &'static str {
        match self {
            Runtime::Claude => ".claude/settings.json",
            Runtime::Codex => ".codex/hooks.json",
            Runtime::Pi => ".pi/extensions/bee-guard.ts",
        }
    }
    fn skills_rel(self) -> &'static str {
        match self {
            Runtime::Claude => ".claude/skills",
            Runtime::Codex => ".agents/skills",
            Runtime::Pi => ".agents/skills",
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
    mechanical_rows_with_env(root, runtime, &|k| std::env::var(k).ok())
}

fn mechanical_rows_with_env(
    root: &Path,
    runtime: Runtime,
    env: &dyn Fn(&str) -> Option<String>,
) -> Vec<Row> {
    let mut rows = Vec::new();

    let hooks_path = root.join(runtime.hooks_rel());
    let (hooks_ok, hooks_detail, hooks_bytes) = match std::fs::read(&hooks_path) {
        Ok(b) => (
            Some(true),
            format!("{} present ({} bytes)", runtime.hooks_rel(), b.len()),
            Some(b),
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
            Some(false),
            format!("{} is missing — the runtime loads no bee hooks", runtime.hooks_rel()),
            None,
        ),
        Err(e) => (
            None,
            format!("{} cannot be read ({e})", runtime.hooks_rel()),
            None,
        ),
    };
    rows.push(Row {
        key: "hooks_file",
        ok: hooks_ok,
        detail: hooks_detail,
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
    let (skills_ok, skills_detail) = match std::fs::read_dir(&skills_dir) {
        Ok(entries) => {
            let skill_count = entries.filter_map(|x| x.ok()).filter(|x| x.path().is_dir()).count();
            if skill_count > 0 {
                (
                    Some(true),
                    format!("{} skill(s) under {}", skill_count, runtime.skills_rel()),
                )
            } else {
                (
                    Some(false),
                    format!("no skills under {} — the agent has no bee craft to load", runtime.skills_rel()),
                )
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (
            Some(false),
            format!("no skills under {} — the agent has no bee craft to load", runtime.skills_rel()),
        ),
        Err(e) => (
            None,
            format!("{} cannot be enumerated ({e})", runtime.skills_rel()),
        ),
    };
    rows.push(Row {
        key: "skills_installed",
        ok: skills_ok,
        detail: skills_detail,
    });

    // The byte match. See the header: this stands in for the retired
    // capability baseline, against an artifact that still exists.
    //
    // THE THREE RUNTIMES ARE NOT THE SAME SHAPE, and treating them alike is a
    // false FAIL. `.codex/hooks.json` IS the rendered artifact, so whole-file
    // equality is the right question. `.claude/settings.json` is the user's
    // settings file that onboarding MERGES a `hooks` key into — it also
    // carries permissions and anything else the host put there, none of which
    // bee renders. Comparing the whole file there fails every correctly
    // installed repo. Only the `hooks` subtree is bee's to answer for.
    // `.pi/extensions/bee-guard.ts` is a TypeScript extension whose canonical
    // text is embedded at compile time — exact byte equality against the
    // compiled extension is the right question.
    match runtime {
        Runtime::Codex => {
            let rendered = crate::devtools::render_projection_text_for(runtime.name());
            rows.push(match (hooks_bytes.as_ref(), rendered) {
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
            });

            if let Some(row) = binary_freshness_row(root) {
                rows.push(row);
            }
        }
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
            rows.push(Row {
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
            });

            if let Some(row) = binary_freshness_row(root) {
                rows.push(row);
            }
        }
        Runtime::Pi => {
            let extension_match = match hooks_bytes.as_ref() {
                Some(on_disk) => {
                    let same = on_disk.as_slice() == PI_EXTENSION_SOURCE.as_bytes();
                    Row {
                        key: "wiring_matches_binary",
                        ok: Some(same),
                        detail: if same {
                            ".pi/extensions/bee-guard.ts is byte-identical to what this bee embeds".to_string()
                        } else {
                            ".pi/extensions/bee-guard.ts differs from what this bee embeds — re-run the installer to refresh it".to_string()
                        },
                    }
                }
                None => match hooks_ok {
                    Some(false) => Row {
                        key: "wiring_matches_binary",
                        ok: Some(false),
                        detail: "no .pi/extensions/bee-guard.ts to compare".to_string(),
                    },
                    _ => Row {
                        key: "wiring_matches_binary",
                        ok: None,
                        detail: ".pi/extensions/bee-guard.ts cannot be read to compare".to_string(),
                    },
                },
            };
            rows.push(extension_match);

            rows.push(pi_binary_freshness_row(root));
            rows.push(pi_herding_transport_row_with_env(root, env));
        }
    }

    rows
}

/// The configured `hat-*` slots on this runtime that carry no description —
/// lane-model-diversity D3 (store `23de5362`).
///
/// D3 requires every configured hat slot to state what the hat is FOR, so the
/// model table reads self-documenting: five hats fanning out at once are worth
/// nothing to the operator who cannot tell which seat is which. D3 named `bee
/// config validate` as the enforcement point; that command was never ported off
/// Node, and the dispatch door may not enforce it either, because `description`
/// is DISPLAY-ONLY law (`hooks::model_guard::role_slot_description`) —
/// `normalize_models` drops the field before any resolver sees it, and nothing
/// that resolves, guards, or dispatches may read it. So the venue is `bee
/// doctor`, as an ADVISORY: it reports, it never votes on the verdict ladder,
/// and no dispatch behavior depends on the field. D3's intent is served and the
/// display-only law stands unsuperseded.
///
/// Reads the RAW config for that same reason — the normalized map has no
/// `description` to find. Three cases and their answers:
///
///   * a string-shaped slot (`"hat-risks": "opus"`) has nowhere to PUT a
///     description, so it is flagged: it resolves fine, and it still reads as
///     an unlabelled seat.
///   * an object slot with no `description`, or an empty/whitespace one, is
///     flagged — the same emptiness test the door itself applies.
///   * a `null` slot is a seat switched OFF, not a configured one. It has no
///     purpose to state and falls through to the advisor at dispatch, so it is
///     passed over rather than nagged about.
///
/// Any key spelled `hat-…` counts, not only the five in `SEAT_ROLES`: an
/// operator who invents a sixth hat is asking the same question of their own
/// config, and a rule keyed to a closed list would answer it silently.
fn hat_slots_missing_a_description(root: &Path, runtime: Runtime) -> Vec<String> {
    let config = crate::state::read_config_raw(root);
    let Some(table) = config
        .get("team")
        .and_then(|m| m.get(runtime.name()))
        .and_then(Value::as_object)
    else {
        return Vec::new();
    };
    let mut missing = Vec::new();
    for (slot, value) in table {
        if !slot.to_ascii_lowercase().starts_with(crate::verbs::drivers::HAT_ROLE_PREFIX) {
            continue;
        }
        if value.is_null() {
            continue;
        }
        let described = value
            .as_object()
            .and_then(|o| o.get("description"))
            .and_then(Value::as_str)
            .is_some_and(|text| !text.trim().is_empty());
        if !described {
            missing.push(slot.clone());
        }
    }
    missing
}

/// The advisory as doctor publishes it — `None` when every configured hat says
/// what it is for, so a clean config's payload and lines are byte-identical to
/// before this row existed.
///
/// Status `advisory` is a fourth word beside ok/not_ok/unknown, and it is
/// deliberately NOT a `Row`: every `Row` votes in `mechanical_ok`, so a hat
/// missing one sentence would have BLOCKED the runtime. A missing description
/// costs an operator clarity, never a working dispatch.
fn hat_description_advisory(root: &Path, runtime: Runtime) -> Option<(Value, String)> {
    let missing = hat_slots_missing_a_description(root, runtime);
    if missing.is_empty() {
        return None;
    }
    let detail = format!(
        "{} hat slot(s) in team.{} carry no description: {} — each hat's purpose belongs in its own slot (\"{}\": {{\"model\": \"…\", \"description\": \"what this hat looks for\"}}), so the config reads self-documenting. Advisory only: it does not change the verdict.",
        missing.len(),
        runtime.name(),
        missing.join(", "),
        missing[0],
    );
    Some((
        json!({"row": "hat_slot_descriptions", "status": "advisory", "detail": detail.clone()}),
        detail,
    ))
}

/// team-config-rename D2: a top-level check for the legacy models config key.
/// Emits one advisory when .bee/config.json still uses models (folded on read).
/// When both keys are present, notes that team is used and models is ignored.
pub(crate) fn team_config_advisory(root: &Path) -> Option<(Value, String)> {
    let config_path = root.join(".bee").join("config.json");
    let ReadJson::Parsed(Value::Object(map)) = read_json(&config_path) else {
        return None;
    };
    let had_team = map.contains_key("team");
    let mut copy = map;
    if !crate::verbs::drivers::fold_team_key(&mut copy) {
        return None;
    }
    let detail = if had_team {
        ".bee/config.json still uses the models key — team is used and models is ignored.".to_string()
    } else {
        ".bee/config.json still uses the models key — rename it to team (models is read as an alias for now).".to_string()
    };
    Some((
        json!({"row": "legacy_models_key", "status": "advisory", "detail": detail.clone()}),
        detail,
    ))
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
/// own `rs-info` bee_version disagrees with the source release version in
/// `.claude-plugin/plugin.json`, (b) the binary's `rs-info` omits bee_version
/// (the binary predates this check and is stale), or (c) any source input —
/// `packages/bee-rs/crates/**/*.rs`, `packages/bee-rs/**/Cargo.toml`,
/// `.claude-plugin/plugin.json`, `packages/bee/prompts/*.md` — is newer by
/// mtime than the installed binary. Read-only throughout: it only stats and
/// reads files and spawns the installed binary to ask its own version, never
/// builds, copies, or writes anything.
///
/// When the probe itself cannot run — the binary refuses to exec, or answers
/// with something unreadable — no version was read, so the row reports unknown
/// rather than fresh. The mtime leg still runs first: real evidence of drift
/// beats "unknown".
fn binary_freshness_row(root: &Path) -> Option<Row> {
    binary_freshness_row_impl(root, false)
}

fn pi_binary_freshness_row(root: &Path) -> Row {
    binary_freshness_row_impl(root, true).expect("pi binary freshness is always present")
}

fn binary_freshness_row_impl(root: &Path, is_pi: bool) -> Option<Row> {
    const KEY: &str = "binary_freshness";
    const REMEDY: &str = "FIX: cargo build --release --manifest-path packages/bee-rs/Cargo.toml \
        -p bee --bin bee, then copy target/release/bee to .bee/bin/bee.";
    const HOST_REMEDY: &str = "FIX: re-run the bee installer in this repo: curl -fsSL \
        https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.sh | bash -s -- -y";
    const PLUGIN_MANIFEST: &str = ".claude-plugin/plugin.json";
    const ONBOARDING_RECORD: &str = ".bee/onboarding.json";

    let workspace_cargo = root.join("packages/bee-rs/Cargo.toml");
    let is_source_checkout = workspace_cargo.is_file();
    if !is_source_checkout && !is_pi {
        return None;
    }

    // A host has no plugin manifest; the version its installer wrote to
    // `.bee/onboarding.json` is what it expects (host-packaging-gaps D1).
    let (expected, source, remedy) = if is_source_checkout {
        (read_source_release_version(root), PLUGIN_MANIFEST, REMEDY)
    } else {
        match read_onboarded_version(root) {
            Some(v) => (Some(v), ONBOARDING_RECORD, HOST_REMEDY),
            None => (read_source_release_version(root), PLUGIN_MANIFEST, HOST_REMEDY),
        }
    };
    let Some(source_version) = expected else {
        return Some(Row {
            key: KEY,
            ok: None,
            detail: if is_source_checkout {
                ".claude-plugin/plugin.json is missing or unreadable — cannot determine source release version".to_string()
            } else {
                ".bee/onboarding.json has no bee_version and .claude-plugin/plugin.json is missing or unreadable — cannot determine the expected release version".to_string()
            },
        });
    };

    // Missing binary is `hook_handler`'s verdict to give; repeating not_ok
    // here under a second name would just be noise, so this reports unknown.
    let Some(bin) = host_binary(root) else {
        return Some(Row {
            key: KEY,
            ok: None,
            detail: "no installed binary to check for freshness (see hook_handler)".to_string(),
        });
    };

    let mut probe_failed: Option<String> = None;
    match installed_binary_bee_version(&bin) {
        ProbedBeeVersion::Missing => {
            return Some(Row {
                key: KEY,
                ok: Some(false),
                detail: format!(
                    "installed binary is too old to report its release version (rs-info carries no bee_version field). {remedy}"
                ),
            });
        }
        ProbedBeeVersion::Present(installed_version) => {
            if installed_version != source_version {
                return Some(Row {
                    key: KEY,
                    ok: Some(false),
                    detail: format!(
                        "installed binary reports release version {installed_version}, source \
                         ({source}) is {source_version}. {remedy}"
                    ),
                });
            }
        }
        // A probe that could not run read no version at all. That is not
        // evidence of staleness, so this is never not_ok on its own — but it
        // is not evidence of freshness either. The mtime scan below still
        // runs and still wins; only when it finds nothing does this flag turn
        // the row into an honest unknown.
        ProbedBeeVersion::Failed(reason) => probe_failed = Some(reason),
    }

    if is_source_checkout {
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
                        "{} was modified {} (binary is {}). {remedy}",
                        rel.display(),
                        fmt_system_time(mtime),
                        fmt_system_time(bin_mtime)
                    ),
                });
            }
        }
    }

    if let Some(reason) = probe_failed {
        return Some(Row {
            key: KEY,
            ok: None,
            detail: if is_source_checkout {
                format!(
                    "could not read the installed binary's release version (bee rs-info: {reason}), \
                     and no source input is newer than it — freshness is unknown. {remedy}"
                )
            } else {
                format!(
                    "could not read the installed binary's release version (bee rs-info: {reason}) \
                     — freshness is unknown. {remedy}"
                )
            },
        });
    }

    Some(Row {
        key: KEY,
        ok: Some(true),
        detail: if is_source_checkout {
            format!(
                "installed binary matches source (version {source_version}), no source input newer \
                 than the binary"
            )
        } else {
            format!("installed binary matches release version {source_version}")
        },
    })
}

#[allow(dead_code)]
fn pi_herding_transport_row(root: &Path) -> Row {
    pi_herding_transport_row_with_env(root, &|k| std::env::var(k).ok())
}

fn pi_herding_transport_row_with_env(root: &Path, env: &dyn Fn(&str) -> Option<String>) -> Row {
    const KEY: &str = "herding_transport";
    if every_pi_slot_runs_no_pane(root) {
        return Row {
            key: KEY,
            ok: Some(true),
            detail: "every team.pi role runs a Pi process with --no-pane; no pane multiplexer is needed"
                .to_string(),
        };
    }
    match crate::herding::transport_kind_at(root) {
        Ok(kind) => {
            let (ready, reason, _) =
                crate::verbs::drivers::herding_transport_probe_for(kind, env);
            Row {
                key: KEY,
                ok: Some(ready),
                detail: reason,
            }
        }
        Err(reason) => Row {
            key: KEY,
            ok: None,
            detail: reason,
        },
    }
}

/// True when `team.pi` has at least one slot and every slot is a herding slot
/// whose agent is a Pi process — the same test dispatch uses to add
/// `--no-pane` (host-packaging-gaps D5).
fn every_pi_slot_runs_no_pane(root: &Path) -> bool {
    let mut map = crate::state::read_config_raw(root);
    crate::verbs::drivers::fold_team_key(&mut map);
    let cfg = Value::Object(map);
    let Some(slots) = cfg.pointer("/team/pi").and_then(Value::as_object) else {
        return false;
    };
    !slots.is_empty()
        && slots.values().all(|slot| {
            slot.get("kind").and_then(Value::as_str) == Some("herding")
                && slot
                    .get("agent")
                    .and_then(Value::as_str)
                    .is_some_and(|name| crate::verbs::drivers::is_pi_agent(&cfg, name))
        })
}

/// The `bee_version` the installer recorded in `<root>/.bee/onboarding.json`;
/// null or absent reads as missing.
fn read_onboarded_version(root: &Path) -> Option<String> {
    let text = std::fs::read_to_string(root.join(".bee/onboarding.json")).ok()?;
    let parsed: Value = serde_json::from_str(&text).ok()?;
    parsed.get("bee_version").and_then(Value::as_str).map(str::to_string)
}

/// The release version from `<root>/.claude-plugin/plugin.json`.
fn read_source_release_version(root: &Path) -> Option<String> {
    let manifest_path = root.join(".claude-plugin/plugin.json");
    let text = std::fs::read_to_string(manifest_path).ok()?;
    let parsed: Value = serde_json::from_str(&text).ok()?;
    parsed.get("version").and_then(Value::as_str).map(str::to_string)
}

enum ProbedBeeVersion {
    Present(String),
    Missing,
    /// Carries WHY the probe produced no version. The reason is not decoration:
    /// this arm ends in an `ok: None` row, and a row that says only "unknown"
    /// leaves the next reader with nothing to act on.
    Failed(String),
}

/// How long the spawn keeps retrying `ETXTBSY`, and the gap between tries.
/// A binary written moments ago is not runnable while ANY process still holds
/// a write descriptor to it, and a multi-threaded caller (cargo's test harness
/// is one) can fork a child that inherits exactly that descriptor. The window
/// closes as soon as the writer's fd does, so a handful of short retries turns
/// a spurious "freshness unknown" into the real verdict; nothing else retries,
/// because nothing else is transient.
const PROBE_ETXTBSY_ATTEMPTS: u32 = 10;
const PROBE_ETXTBSY_DELAY_MS: u64 = 20;

/// The installed binary's answer to what release version it was built from —
/// `bee rs-info`'s `bee_version` field. A probe, never a mutation: this only
/// spawns and reads stdout.
fn installed_binary_bee_version(bin: &Path) -> ProbedBeeVersion {
    let mut spawned = Err(std::io::ErrorKind::Other.into());
    for attempt in 0..PROBE_ETXTBSY_ATTEMPTS {
        spawned = std::process::Command::new(bin).arg("rs-info").output();
        // `ExecutableFileBusy` is the ONE retryable spawn failure: it means
        // the file exists and is ours, just not runnable yet. A missing file,
        // a permission denial or anything else is a real answer already.
        match &spawned {
            Err(e) if e.kind() == std::io::ErrorKind::ExecutableFileBusy => {}
            _ => break,
        }
        if attempt + 1 < PROBE_ETXTBSY_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(PROBE_ETXTBSY_DELAY_MS));
        }
    }
    let out = match spawned {
        Ok(out) => out,
        Err(e) => return ProbedBeeVersion::Failed(format!("could not spawn it ({e})")),
    };
    if !out.status.success() {
        return ProbedBeeVersion::Failed(format!("it exited {}", out.status));
    }
    let Ok(value) = serde_json::from_slice::<Value>(&out.stdout) else {
        return ProbedBeeVersion::Failed("its rs-info output is not JSON".to_string());
    };
    match value.get("bee_version").and_then(Value::as_str) {
        Some(ver) => ProbedBeeVersion::Present(ver.to_string()),
        None => ProbedBeeVersion::Missing,
    }
}

/// The freshness inputs: every `.rs` file and `Cargo.toml` under
/// `packages/bee-rs/crates`, the workspace `packages/bee-rs/Cargo.toml`
/// itself, `.claude-plugin/plugin.json`, and every `.md` prompt directly under
/// `packages/bee/prompts`. The walk stays inside `crates/` rather than all of
/// `packages/bee-rs` on purpose — the sibling `target/` build directory lives
/// at the workspace root, not under `crates/`, and a doctor row must never
/// wander into it.
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

    let plugin_manifest = root.join(".claude-plugin/plugin.json");
    if plugin_manifest.is_file() {
        out.push(plugin_manifest);
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

    // D3's advisory rides ALONGSIDE the ladder, never inside it: it is
    // appended after every row that votes, and `status` below is computed from
    // `mechanical_ok` and the attestation exactly as it was.
    let advisory = hat_description_advisory(&root, runtime);
    if let Some((row, _)) = &advisory {
        all.push(row.clone());
    }
    let team_advisory = team_config_advisory(&root);
    if let Some((row, _)) = &team_advisory {
        all.push(row.clone());
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
    if let Some((_, detail)) = &advisory {
        lines.push(format!("  note {:<22} {}", "hat_slot_descriptions", detail));
    }
    if let Some((_, detail)) = &team_advisory {
        lines.push(format!("  note {:<22} {}", "legacy_models_key", detail));
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
        let msg = match runtime {
            Runtime::Claude => "bee doctor attest: --runtime codex only. Claude has no trust-unknown rows, so mechanical green already reaches ready there — there is nothing to attest.",
            Runtime::Pi => "bee doctor attest: --runtime codex only. Pi has no trust-unknown rows, so mechanical green already reaches ready there — there is nothing to attest.",
            Runtime::Codex => unreachable!(),
        };
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
