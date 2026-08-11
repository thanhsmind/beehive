// OpenCode guard-belt fixtures + the three-belt parity test (plan.md E3,
// cell oc-7).
//
// PROVENANCE / WHY FROM ZERO. `docs/06-runtime-integration.md:143` names a
// two-belt parity test that DOES NOT EXIST in this tree — it died with the
// Node runtime at the R6 cutover (commit 5c62cad0, `packages/bee/hooks/
// test_hook_contracts.mjs`'s Node-runtime half). There is nothing to port
// forward for the OpenCode belt specifically: OpenCode is a NEW third
// runtime (opencode-support D1), so this file is authored from zero, inside
// the cargo suite `commands.test` already runs (never a standalone script).
//
// WHAT THIS FILE PROVES, in two parts:
//
//   1. Fixture tests over the REAL `.opencode/plugins/bee-guard.ts`, run
//      under a real `node` subprocess (Node 24+ strips this file's erasable
//      TypeScript syntax natively — no build step, no ts-node). Each fixture
//      swaps in a STUB `.bee/bin/bee` binary that can deny (exit 2), allow
//      (exit 0), crash (any other nonzero exit), or be absent, and asserts
//      the plugin's two documented failure policies hold
//      (bee-guard.ts:14-32): BLOCKING surfaces throw on deny/crash/missing
//      (fail CLOSED); ADVISORY surfaces never throw, on any stub behavior
//      (fail OPEN, matching `docs/knowledge/patterns/20260714-a-fail-open-
//      host-swallows-fail-closed-throws.md`'s warning about NOT letting the
//      two mix). Node absence is a named, non-fatal skip — see
//      `node_or_skip`.
//
//   2. A three-belt parity test that DERIVES the guard-rule inventory from
//      the catalog of record — the two checked-in, generated hook manifests
//      (`packages/bee/hooks/claude-hooks.json`, `packages/bee/hooks/
//      hooks.json`; both are `hook_manifests.rs`'s CATALOG rendered to disk,
//      kept honest by `hook_manifests_match_disk` in that module) — and
//      never a hand-authored list, per
//      `docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-
//      truth-it-never-compares-two-hand-lists.md`. For every rule the
//      catalog marks BLOCKING (found wired under a `PreToolUse` event: today
//      `write-guard` and `model-guard`), the test asserts FOUR independent
//      signals exist, failing by name (rule + belt) if any is missing:
//        - HELPER level  — `bee hook <rule>` itself denies (the shared FIRST
//          belt every runtime's translation layer calls into — plan.md's
//          Approach section: "helpers stay the FIRST belt on every
//          runtime").
//        - CLAUDE belt   — `hook_contracts.rs`'s own source (embedded at
//          compile time) contains a deny fixture using one of the
//          claude-shaped tool names the catalog says reaches this rule.
//        - CODEX belt    — the rule is wired under `PreToolUse` in the
//          file-shipped codex projection itself (no separate Codex
//          translation layer exists to fixture-test beyond that wiring —
//          Codex's PreToolUse command execs `bee hook <rule>` directly, byte
//          for byte the same mechanism the helper-level check already
//          exercises; see bee-guard.ts's own header comment on the
//          matcher-only Codex difference).
//        - OPENCODE belt — `bee-guard.ts`'s `mapToolCall` actually routes at
//          least one real OpenCode tool to this rule (derived by parsing the
//          plugin's own switch statement, never hand-listed) — the actual
//          deny/allow/crash/missing PROOF for every such row lives in part
//          1 above; this is the routing-exists cross-check that keeps part 1
//          from silently going vacuous if a row's `hook:` literal changes.
//
// The design and any environment-skip behavior are recorded in
// `docs/history/opencode-support/discovery.md`.

use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// ─── repo layout ────────────────────────────────────────────────────────────

/// `CARGO_MANIFEST_DIR` is `packages/bee-rs/crates/bee`; four `parent()`s
/// (crates, bee-rs, packages, repo root) reach the checkout root, the same
/// one `.opencode/plugins/bee-guard.ts` and the checked-in hook manifests
/// live under.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR too shallow: {}", env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn bee_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("bee")
}

/// The real plugin source, embedded at compile time — the same file
/// `docs/history/opencode-support/discovery.md` documents and this test
/// exercises live via `node`. Never copied or re-derived by hand.
const PLUGIN_SOURCE: &str = include_str!("../../../../../.opencode/plugins/bee-guard.ts");

/// `hook_contracts.rs`'s own source, embedded so the parity test can look
/// for the CLAUDE belt's deny fixtures by what they actually assert, rather
/// than trusting a hand-maintained pointer to a test name.
const HOOK_CONTRACTS_SOURCE: &str = include_str!("hook_contracts.rs");

/// `discovery.md`'s own text, embedded so the parity test can confirm any
/// ADVISORY hook the OpenCode plugin does not wire is a NAMED gap (already
/// written up by oc-6), never a silent omission.
const DISCOVERY_DOC: &str = include_str!("../../../../../docs/history/opencode-support/discovery.md");

// ─── catalog of record: derive the guard-rule inventory ────────────────────
//
// The catalog of record is the two checked-in, GENERATED hook manifests
// `packages/bee/hooks/claude-hooks.json` (Runtime::Claude) and
// `packages/bee/hooks/hooks.json` (Runtime::Codex) — both rendered from
// `hook_manifests.rs`'s `CATALOG` and drift-checked byte-for-byte against it
// by `hook_manifests_match_disk`. Reading them here (rather than reaching
// into the `devtools` module's private types from this black-box test) is
// itself the derivation: it can never drift from the catalog without that
// existing drift test going red first.

#[derive(Debug, Clone)]
struct CatalogRow {
    hook: String,
    /// True iff this hook is wired under a `PreToolUse` event in EITHER
    /// projection — `PreToolUse` is the only event a `tool.execute.before`-
    /// style belt can block on; every other event this catalog uses
    /// (`SessionStart`, `UserPromptSubmit`, `PostToolUse`, `SubagentStart`,
    /// `SubagentStop`, `PreCompact`, `Stop`) is advisory-only by
    /// construction (`hook_manifests.rs`'s own `PostToolUse` comment: "this
    /// hook can never deny or block").
    blocking: bool,
    /// The `|`-split matcher tokens (e.g. `["Edit","Write",...]`) the CLAUDE
    /// projection uses to reach this hook under `PreToolUse`, empty if this
    /// hook is not claude-blocking.
    claude_pretooluse_matchers: Vec<String>,
    /// Same, for the CODEX projection.
    codex_pretooluse_matchers: Vec<String>,
}

impl CatalogRow {
    fn new(hook: &str) -> Self {
        CatalogRow {
            hook: hook.to_string(),
            blocking: false,
            claude_pretooluse_matchers: Vec::new(),
            codex_pretooluse_matchers: Vec::new(),
        }
    }
}

/// Pulls the hook NAME out of a rendered command string. Every rendered
/// command execs `<bin> hook <name>` at least once (the project-then-
/// main-worktree fallback chain repeats the same name twice) — mirrors the
/// same extraction `hook_manifests.rs`'s own
/// `projections_differ_only_where_approved` test performs on these exact
/// strings.
fn extract_hook_name(command: &str) -> Option<String> {
    let idx = command.find(" hook ")?;
    let rest = &command[idx + " hook ".len()..];
    let end = rest.find(|c: char| c == ';' || c.is_whitespace())?;
    Some(rest[..end].to_string())
}

/// `event name -> [(matcher, hook name)]` for one rendered projection JSON.
fn events_map(v: &Value) -> BTreeMap<String, Vec<(Option<String>, String)>> {
    let hooks_obj = v["hooks"].as_object().expect("projection JSON has no top-level \"hooks\" object");
    let mut map = BTreeMap::new();
    for (event, groups) in hooks_obj {
        let mut rows = Vec::new();
        for g in groups.as_array().unwrap_or_else(|| panic!("{event}: groups must be an array")) {
            let matcher = g.get("matcher").and_then(|m| m.as_str()).map(str::to_string);
            for h in g["hooks"].as_array().unwrap_or_else(|| panic!("{event}: group hooks must be an array")) {
                let command = h["command"].as_str().unwrap_or_else(|| panic!("{event}: hook command must be a string"));
                let name = extract_hook_name(command)
                    .unwrap_or_else(|| panic!("{event}: could not extract a hook name from command: {command}"));
                rows.push((matcher.clone(), name));
            }
        }
        map.insert(event.clone(), rows);
    }
    map
}

fn read_projection(rel: &str) -> Value {
    let path = repo_root().join(rel);
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{}: not valid JSON: {e}", path.display()))
}

/// The derived catalog: hook name -> row. Built once per call from the two
/// checked-in projections — no hand-authored hook list anywhere in this
/// function.
fn derive_catalog() -> BTreeMap<String, CatalogRow> {
    let claude = events_map(&read_projection("packages/bee/hooks/claude-hooks.json"));
    let codex = events_map(&read_projection("packages/bee/hooks/hooks.json"));
    let mut rows: BTreeMap<String, CatalogRow> = BTreeMap::new();

    for (event, list) in &claude {
        for (matcher, name) in list {
            let row = rows.entry(name.clone()).or_insert_with(|| CatalogRow::new(name));
            if event == "PreToolUse" {
                row.blocking = true;
                if let Some(m) = matcher {
                    row.claude_pretooluse_matchers = m.split('|').map(str::to_string).collect();
                }
            }
        }
    }
    for (event, list) in &codex {
        for (matcher, name) in list {
            let row = rows.entry(name.clone()).or_insert_with(|| CatalogRow::new(name));
            if event == "PreToolUse" {
                row.blocking = true;
                if let Some(m) = matcher {
                    row.codex_pretooluse_matchers = m.split('|').map(str::to_string).collect();
                }
            }
        }
    }
    rows
}

// ─── the opencode plugin's own routing, derived from its source ───────────

/// Parses `bee-guard.ts`'s `mapToolCall` switch statement for every
/// `(OpenCode tool, bee hook)` pair it actually routes — the same mechanism
/// section (1)'s fixtures exercise, reused here so the parity test's
/// "opencode belt" claim can never drift from what the fixtures cover.
fn opencode_tool_hook_pairs() -> Vec<(String, String)> {
    let fn_start = PLUGIN_SOURCE
        .find("function mapToolCall")
        .expect("bee-guard.ts: mapToolCall not found — has the routing function been renamed?");
    let body = &PLUGIN_SOURCE[fn_start..];
    let switch_start = body
        .find("switch (tool)")
        .expect("bee-guard.ts: mapToolCall no longer switches on `tool` — routing derivation needs an update");
    let switch_body = &body[switch_start..];

    let mut pairs = Vec::new();
    for seg in switch_body.split("case \"").skip(1) {
        let tool_end = seg.find('"').expect("bee-guard.ts: unterminated `case \"...\"` tool literal");
        let tool = seg[..tool_end].to_string();
        if let Some(h) = seg.find("hook: \"") {
            let rest = &seg[h + "hook: \"".len()..];
            let hend = rest.find('"').expect("bee-guard.ts: unterminated `hook: \"...\"` literal");
            pairs.push((tool, rest[..hend].to_string()));
        }
        // A `default: return null` arm carries no `hook: "..."` literal, so
        // it is silently skipped rather than mis-parsed as a row.
    }
    assert!(
        pairs.len() >= 9,
        "bee-guard.ts: mapToolCall derivation found only {} routed tool cases — \
         expected at least 9 (write/edit/bash/apply_patch/read/grep/glob/question/task); \
         either the switch statement changed shape or this parser broke",
        pairs.len()
    );
    pairs
}

/// Parses every `runAdvisoryHook(directory, "<name>", ...)` call site —
/// the ADVISORY hooks the plugin actually wires (session-init,
/// prompt-context, state-sync, session-close, tools-logger today).
/// `codex-subagent-audit` and `chain-nudge` deliberately do not appear here;
/// see `advisory_gaps_the_plugin_does_not_wire_are_named_not_silent` below.
fn opencode_advisory_hooks() -> BTreeSet<String> {
    const MARKER: &str = "runAdvisoryHook(directory, \"";
    let mut set = BTreeSet::new();
    let mut idx = 0usize;
    while let Some(pos) = PLUGIN_SOURCE[idx..].find(MARKER) {
        let start = idx + pos + MARKER.len();
        let rest = &PLUGIN_SOURCE[start..];
        let end = rest.find('"').expect("bee-guard.ts: unterminated runAdvisoryHook name literal");
        set.insert(rest[..end].to_string());
        idx = start + end;
    }
    assert!(!set.is_empty(), "bee-guard.ts: found zero runAdvisoryHook call sites — advisory derivation broke");
    set
}

// ─── node availability (named, non-fatal skip) ─────────────────────────────
//
// "Node is absent" is not the only way this belt is unrunnable — a `node`
// that IS on PATH but too old to strip TypeScript natively (verified live:
// an ambient PATH with system node v18 ahead of nvm's v24 makes every
// harness spawn die with `ERR_UNKNOWN_FILE_EXTENSION` on the real
// `bee-guard.ts`, not a clean "not found") is functionally the same gap.
// This probes the REAL capability the harness needs — running a one-line
// `.ts` file directly, exactly how it loads the real plugin — rather than
// trusting a version number, so any node that cannot do that degrades to a
// named skip instead of a panic.

fn node_typescript_probe() -> Result<(), String> {
    let version_out = Command::new("node").arg("--version").output();
    let version = match version_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Err("`node` not found on PATH".to_string()),
    };
    let dir = tempfile::tempdir().map_err(|e| format!("could not create a tempdir for the node/TS probe: {e}"))?;
    let probe = dir.path().join("probe.ts");
    std::fs::write(&probe, "const x: number = 1\nconsole.log(x)\n").map_err(|e| e.to_string())?;
    let out = Command::new("node")
        .arg(&probe)
        .output()
        .map_err(|e| format!("failed to spawn `node {}`: {e}", probe.display()))?;
    if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "1" {
        Ok(())
    } else {
        Err(format!(
            "`node` ({version}) cannot run a minimal .ts file directly (needed to load \
             bee-guard.ts exactly the way OpenCode itself loads it — no build step) — stderr: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// Every test that drives the real plugin under `node` opens with this: an
/// absent or TS-incapable `node` is a named, visible SKIP (matching
/// `hook_contracts.rs`'s own `ran_native` skip convention), never a silent
/// pass and never a panic.
macro_rules! node_or_skip {
    ($test_name:expr) => {
        if let Err(reason) = node_typescript_probe() {
            eprintln!("SKIP (env-limited: {reason}) — {}", $test_name);
            return;
        }
    };
}

// ─── the node harness: drives the real plugin's exported hooks ────────────
//
// A tiny, generic driver (never checked in — written fresh into a tempdir
// per test run, exactly the way `hook_contracts.rs`'s `fixture()` writes its
// `.bee/onboarding.json` marker) that dynamically imports the REAL
// `bee-guard.ts` file by path, calls its default export to get the `Hooks`
// object, invokes exactly one named surface with a JSON payload read from
// stdin, and reports whether the call threw. Node 24's native TypeScript
// type-stripping runs the `.ts` file directly — no separate build step, no
// ts-node dependency.
const HARNESS_JS: &str = r#"
import { pathToFileURL } from "node:url";

const [, , pluginPath, directory, surface] = process.argv;

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => { data += chunk; });
    process.stdin.on("end", () => resolve(data));
    process.stdin.on("error", reject);
  });
}

const payload = JSON.parse(await readStdin());
const mod = await import(pathToFileURL(pluginPath).href);
const hooks = await mod.default({ directory, worktree: directory, client: {}, $: {}, project: {}, app: {} });

const fn = hooks[surface];
if (typeof fn !== "function") {
  console.log(JSON.stringify({ threw: false, message: null, output: null, note: `surface ${surface} not registered` }));
  process.exit(0);
}

let result;
try {
  if (surface === "chat.message") {
    const input = { sessionID: payload.sessionID ?? "sess1", messageID: payload.messageID ?? "msg1" };
    const output = { parts: [], message: { id: payload.messageID ?? "msg1" } };
    await fn(input, output);
    result = { threw: false, message: null, output };
  } else if (surface === "event") {
    const input = { event: payload.event };
    await fn(input);
    result = { threw: false, message: null, output: null };
  } else {
    const input = { tool: payload.tool, sessionID: payload.sessionID ?? "sess1", callID: payload.callID ?? "call1", args: payload.args };
    const output = { args: payload.args ?? {} };
    await fn(input, output);
    result = { threw: false, message: null, output };
  }
} catch (err) {
  result = { threw: true, message: String(err && err.message ? err.message : err), output: null };
}
console.log(JSON.stringify(result));
"#;

struct HarnessResult {
    threw: bool,
    message: Option<String>,
    output: Option<Value>,
}

fn write_harness(dir: &Path) -> PathBuf {
    let path = dir.join("harness.mjs");
    std::fs::write(&path, HARNESS_JS).expect("failed to write the node harness");
    path
}

fn run_harness(harness: &Path, plugin: &Path, directory: &Path, surface: &str, payload: &Value) -> HarnessResult {
    let mut child = Command::new("node")
        .arg(harness)
        .arg(plugin)
        .arg(directory)
        .arg(surface)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to launch `node {}`: {e}", harness.display()));
    child
        .stdin
        .take()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .expect("failed to write payload to the harness's stdin");
    let out: Output = child.wait_with_output().expect("node harness never exited");
    assert!(
        out.status.success(),
        "node harness (surface={surface}) exited non-zero: stdout={} stderr={}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last_line = stdout.lines().last().unwrap_or("").trim();
    let v: Value = serde_json::from_str(last_line).unwrap_or_else(|e| {
        panic!(
            "node harness (surface={surface}) did not print JSON on its last stdout line: {e}; \
             stdout={stdout} stderr={}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    HarnessResult {
        threw: v["threw"].as_bool().unwrap_or(false),
        message: v["message"].as_str().map(str::to_string),
        output: v.get("output").cloned().filter(|o| !o.is_null()),
    }
}

// ─── stub bee binaries: deny / allow / crash / absent ──────────────────────

enum StubBehavior {
    Deny(String),
    Allow,
    Crash,
    Missing,
}

/// Writes (or, for `Missing`, deliberately does NOT write) a stub
/// `.bee/bin/bee` under `root`. `root` must be OUTSIDE this checkout (a
/// fresh `tempfile::tempdir()` already is, since it defaults to the system
/// temp root) so the plugin's git-common-dir fallback can never accidentally
/// resolve to this repo's real binary and mask the scenario under test.
#[cfg(unix)]
fn write_stub_bee(root: &Path, behavior: &StubBehavior) {
    use std::os::unix::fs::PermissionsExt;
    let StubBehavior::Missing = behavior else {
        let bin_dir = root.join(".bee").join("bin");
        std::fs::create_dir_all(&bin_dir).expect("failed to create .bee/bin");
        let script = match behavior {
            StubBehavior::Deny(reason) => format!("#!/bin/sh\ncat >/dev/null\necho \"{reason}\" >&2\nexit 2\n"),
            StubBehavior::Allow => "#!/bin/sh\ncat >/dev/null\nexit 0\n".to_string(),
            StubBehavior::Crash => "#!/bin/sh\ncat >/dev/null\necho \"stub crash\" >&2\nexit 17\n".to_string(),
            StubBehavior::Missing => unreachable!(),
        };
        let path = bin_dir.join("bee");
        std::fs::write(&path, script).expect("failed to write the stub bee binary");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("failed to make the stub bee binary executable");
        return;
    };
}

// ═════════════════════════════════════════════════════════════════════════
// PART 1 — fixture tests: deny / allow / crash / missing, per BLOCKING
// mapped row; never-throws, per ADVISORY surface.
// ═════════════════════════════════════════════════════════════════════════

#[cfg(unix)]
#[test]
fn every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary() {
    node_or_skip!("every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let plugin = repo_root().join(".opencode/plugins/bee-guard.ts");

    let pairs = opencode_tool_hook_pairs();
    let mut failures: Vec<String> = Vec::new();

    for (tool, hook) in &pairs {
        let args = json!({ "probe": tool, "value": "x" });

        // (a) deny — a distinct, per-row reason must reach the thrown Error.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            let reason = format!("stub-deny for {hook} via {tool}");
            write_stub_bee(fx.path(), &StubBehavior::Deny(reason.clone()));
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &json!({"tool": tool, "args": args}));
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains(&reason))) {
                failures.push(format!(
                    "{tool} -> {hook}: DENY did not throw the stub's reason (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (b) allow — no throw, and output.args passes through unchanged.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Allow);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &json!({"tool": tool, "args": args}));
            let args_after = r.output.as_ref().and_then(|o| o.get("args")).cloned();
            if r.threw || args_after.as_ref() != Some(&args) {
                failures.push(format!(
                    "{tool} -> {hook}: ALLOW must not throw and must pass args through unchanged \
                     (threw={}, args_after={:?}, args_expected={args})",
                    r.threw, args_after
                ));
            }
        }

        // (c) crash — a nonzero, non-2 exit must still throw (fail closed).
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Crash);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &json!({"tool": tool, "args": args}));
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains("did not return a verdict"))) {
                failures.push(format!(
                    "{tool} -> {hook}: CRASH must throw a \"did not return a verdict\" Error \
                     (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }

        // (d) missing binary — no .bee/bin/bee anywhere reachable must throw.
        {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), &StubBehavior::Missing);
            let r = run_harness(&harness, &plugin, fx.path(), "tool.execute.before", &json!({"tool": tool, "args": args}));
            if !(r.threw && r.message.as_deref().is_some_and(|m| m.contains("could not find the bee binary"))) {
                failures.push(format!(
                    "{tool} -> {hook}: MISSING BINARY must throw a \"could not find the bee binary\" Error \
                     (threw={}, message={:?})",
                    r.threw, r.message
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "one or more BLOCKING mapped rows failed their deny/allow/crash/missing fixtures:\n{}",
        failures.join("\n")
    );
}

#[cfg(unix)]
#[test]
fn advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior() {
    node_or_skip!("advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let plugin = repo_root().join(".opencode/plugins/bee-guard.ts");

    // Every surface the plugin wires an ADVISORY hook onto (bee-guard.ts):
    // chat.message covers session-init + prompt-context; the three `event`
    // shapes cover state-sync (file.edited, session.idle) and session-close
    // (session.idle, session.deleted); tool.execute.after covers
    // tools-logger.
    let surfaces: Vec<(&str, Value)> = vec![
        ("chat.message", json!({"sessionID": "s1", "messageID": "m1"})),
        ("event", json!({"event": {"type": "file.edited", "properties": {"file": "x.txt"}}})),
        ("event", json!({"event": {"type": "session.idle", "properties": {"sessionID": "s1"}}})),
        ("event", json!({"event": {"type": "session.deleted", "properties": {"info": {"id": "s1"}}}})),
        ("tool.execute.after", json!({"tool": "write", "args": {"filePath": "/tmp/x", "content": "hi"}})),
    ];

    let scenarios = [
        StubBehavior::Deny("stub-deny-advisory".to_string()),
        StubBehavior::Allow,
        StubBehavior::Crash,
        StubBehavior::Missing,
    ];

    let mut failures: Vec<String> = Vec::new();
    for (surface, payload) in &surfaces {
        for behavior in &scenarios {
            let fx = tempfile::tempdir().expect("tempdir");
            write_stub_bee(fx.path(), behavior);
            let r = run_harness(&harness, &plugin, fx.path(), surface, payload);
            if r.threw {
                failures.push(format!(
                    "surface {surface} threw ({:?}) even though every hook it reaches is ADVISORY \
                     (must swallow and log, never throw)",
                    r.message
                ));
            }
        }
    }

    assert!(failures.is_empty(), "advisory surfaces must never throw:\n{}", failures.join("\n"));

    // Non-vacuity: confirm the surfaces above really do reach the plugin's
    // ADVISORY hooks (derived from its own source), not an empty no-op.
    let wired = opencode_advisory_hooks();
    for expected in ["session-init", "prompt-context", "state-sync", "session-close", "tools-logger"] {
        assert!(
            wired.contains(expected),
            "expected bee-guard.ts to wire \"{expected}\" via runAdvisoryHook, but the derived set was {wired:?}"
        );
    }
}

#[cfg(not(unix))]
#[test]
fn opencode_plugin_fixtures_skip_on_non_unix() {
    eprintln!(
        "SKIP (env-limited: the stub bee binaries in this suite are unix shebang scripts) \
         — opencode plugin fixture tests"
    );
}

// ═════════════════════════════════════════════════════════════════════════
// PART 2 — the three-belt parity test.
// ═════════════════════════════════════════════════════════════════════════

/// A minimal fixture repo the NATIVE hooks accept as a bee root — same shape
/// as `hook_contracts.rs`'s own `fixture()` (the `.bee/onboarding.json`
/// install marker every hook's activation probe reads since the R6
/// cutover), reproduced here so the HELPER-level check below is
/// self-contained and never depends on another test binary's internals.
struct HelperFixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn helper_fixture() -> HelperFixture {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dunce::canonicalize(dir.path()).expect("canonicalize tempdir");
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    std::fs::write(
        root.join(".bee").join("state.json"),
        serde_json::to_string_pretty(&json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": { "context": true, "shape": true, "execution": true, "review": false }
        }))
        .unwrap()
            + "\n",
    )
    .unwrap();
    HelperFixture { _dir: dir, root }
}

fn run_helper_hook(hook: &str, stdin: &[u8], cwd: &Path) -> Output {
    let mut child = Command::new(bee_bin())
        .args(["hook", hook])
        .current_dir(cwd)
        .env("BEE_HOOK_NO_DELEGATE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to launch `bee hook {hook}`: {e}"));
    child.stdin.take().unwrap().write_all(stdin).ok();
    child.wait_with_output().unwrap_or_else(|e| panic!("`bee hook {hook}` never exited: {e}"))
}

/// The known-denying payload SHAPE per BLOCKING rule — the same domain
/// knowledge `hook_contracts.rs`'s own two "decision reaches the host"
/// tests already encode (a direct edit to `.bee/state.json` for
/// write-guard; a bare Agent/Task dispatch with no tier marker for
/// model-guard). Only the TOOL NAME is taken from the derived catalog row
/// rather than hand-typed.
fn helper_deny_payload(hook: &str, claude_tool_name: &str, root: &Path) -> Value {
    match hook {
        "write-guard" => json!({
            "tool_name": claude_tool_name,
            "tool_input": { "file_path": ".bee/state.json" },
            "cwd": root.to_string_lossy(),
        }),
        "model-guard" => json!({
            "tool_name": claude_tool_name,
            "tool_input": { "prompt": "implement the widget", "description": "some description" },
            "cwd": root.to_string_lossy(),
        }),
        other => panic!("no known deny-payload shape for BLOCKING rule \"{other}\" — add one before trusting this parity check"),
    }
}

/// For each `#[test] fn ... { ... }` block in `hook_contracts.rs`'s embedded
/// source, true iff some block both names one of `tool_names` as a
/// `"tool_name"` literal AND asserts a `code(&out), 2` / `code(out), 2` deny.
fn claude_belt_deny_fixture_exists(tool_names: &[String]) -> bool {
    let idxs: Vec<usize> = HOOK_CONTRACTS_SOURCE.match_indices("#[test]").map(|(i, _)| i).collect();
    for (n, &start) in idxs.iter().enumerate() {
        let end = idxs.get(n + 1).copied().unwrap_or(HOOK_CONTRACTS_SOURCE.len());
        let block = &HOOK_CONTRACTS_SOURCE[start..end];
        let names_a_tool =
            tool_names.iter().any(|t| block.contains(&format!("\"tool_name\": \"{t}\"")));
        let asserts_deny = block.contains("code(&out), 2") || block.contains("code(out), 2");
        if names_a_tool && asserts_deny {
            return true;
        }
    }
    false
}

#[test]
fn three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode() {
    let catalog = derive_catalog();
    let blocking: Vec<&CatalogRow> = catalog.values().filter(|r| r.blocking).collect();
    assert!(
        blocking.len() >= 2,
        "derived catalog found only {} BLOCKING rule(s) — expected at least write-guard and \
         model-guard; the derivation likely broke: {:?}",
        blocking.len(),
        catalog.keys().collect::<Vec<_>>()
    );

    let opencode_pairs = opencode_tool_hook_pairs();
    let opencode_hooks: BTreeSet<&str> = opencode_pairs.iter().map(|(_, h)| h.as_str()).collect();

    let mut gaps: Vec<String> = Vec::new();

    for row in &blocking {
        let hook = row.hook.as_str();

        // HELPER level: `bee hook <hook>` itself denies the known-denying
        // shape for this rule.
        {
            let claude_tool = row
                .claude_pretooluse_matchers
                .first()
                .unwrap_or_else(|| panic!("BLOCKING rule \"{hook}\" has no claude PreToolUse matcher to derive a tool name from"));
            let fx = helper_fixture();
            let payload = helper_deny_payload(hook, claude_tool, &fx.root);
            let out = run_helper_hook(hook, payload.to_string().as_bytes(), &fx.root);
            let code = out.status.code().unwrap_or(-1);
            if code == 42 {
                gaps.push(format!(
                    "{hook} / helper belt: `bee hook {hook}` still delegates to Node under \
                     BEE_HOOK_NO_DELEGATE — the native decision path was never reached"
                ));
            } else if code != 2 {
                gaps.push(format!(
                    "{hook} / helper belt: expected exit 2 (deny) for the known-denying payload, \
                     got exit {code} stderr={}",
                    String::from_utf8_lossy(&out.stderr)
                ));
            }
        }

        // CLAUDE belt: the rule is wired under PreToolUse in claude's
        // file-shipped projection (already true by construction of
        // `blocking`, but checked per-projection here since `blocking` is a
        // union) AND hook_contracts.rs carries a deny fixture using one of
        // claude's own matcher tokens.
        if row.claude_pretooluse_matchers.is_empty() {
            gaps.push(format!("{hook} / claude belt: not wired under PreToolUse in claude-hooks.json"));
        } else if !claude_belt_deny_fixture_exists(&row.claude_pretooluse_matchers) {
            gaps.push(format!(
                "{hook} / claude belt: hook_contracts.rs has no deny fixture using any of {:?}",
                row.claude_pretooluse_matchers
            ));
        }

        // CODEX belt: the rule is wired under PreToolUse in codex's
        // file-shipped projection. Codex has no separate translation layer
        // to fixture-test beyond that wiring — its PreToolUse command execs
        // `bee hook <rule>` directly, the exact call the HELPER check above
        // already proved denies.
        if row.codex_pretooluse_matchers.is_empty() {
            gaps.push(format!("{hook} / codex belt: not wired under PreToolUse in packages/bee/hooks/hooks.json"));
        }

        // OPENCODE belt: bee-guard.ts's mapToolCall actually routes some
        // real tool to this hook — the deny/allow/crash/missing PROOF lives
        // in `every_blocking_mapped_row_denies_allows_crashes_and_reports_a_missing_binary`.
        if !opencode_hooks.contains(hook) {
            gaps.push(format!(
                "{hook} / opencode belt: bee-guard.ts's mapToolCall routes no tool to this hook \
                 (derived pairs: {opencode_pairs:?})"
            ));
        }
    }

    assert!(
        gaps.is_empty(),
        "three-belt parity gap(s), naming the rule and the belt that missed it:\n{}",
        gaps.join("\n")
    );
}

/// The parity test above only requires BLOCKING rules to hit all three
/// belts. ADVISORY rules the OpenCode plugin does not wire
/// (`codex-subagent-audit`, `chain-nudge`) are allowed to be missing — but
/// ONLY if that gap is already a NAMED, documented exclusion, never a
/// silent one. This is the coverage-gate half of the contract: any future
/// ADVISORY hook the catalog gains that the plugin does not wire, and that
/// discovery.md does not name, fails here rather than shipping silently.
#[test]
fn advisory_gaps_the_plugin_does_not_wire_are_named_not_silent() {
    let catalog = derive_catalog();
    let advisory_rules: Vec<&str> = catalog.values().filter(|r| !r.blocking).map(|r| r.hook.as_str()).collect();
    assert!(
        advisory_rules.len() >= 2,
        "derived catalog found too few ADVISORY rules to be a meaningful check: {advisory_rules:?}"
    );

    let wired = opencode_advisory_hooks();
    let mut unnamed_gaps: Vec<String> = Vec::new();
    for rule in advisory_rules {
        if wired.contains(rule) {
            continue; // covered live by `advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior`
        }
        // Not wired — must be named in discovery.md, not silently dropped.
        let documented = DISCOVERY_DOC.contains(rule)
            && (DISCOVERY_DOC.contains("NAMED EXCLUSION") || DISCOVERY_DOC.contains("Deferred"));
        if !documented {
            unnamed_gaps.push(format!(
                "{rule}: not wired by bee-guard.ts's runAdvisoryHook AND not documented as a named \
                 gap in docs/history/opencode-support/discovery.md"
            ));
        }
    }

    assert!(
        unnamed_gaps.is_empty(),
        "silent ADVISORY coverage gap(s) — every hook the opencode belt does not wire must be a \
         NAMED exclusion in discovery.md:\n{}",
        unnamed_gaps.join("\n")
    );
}
