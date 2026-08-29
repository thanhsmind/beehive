// Pi guard-belt fixtures (plan.md cell pis-3).
//
// PROVENANCE. The Pi belt is `.pi/extensions/bee-guard.ts` — the FOURTH
// harness belt, authored by pis-1 as the OpenCode belt re-targeted at Pi's
// event names (pi-support D1/D2/D3/D8). This file is its fixture suite, and
// it is deliberately the sibling of
// `packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs`: same node
// subprocess, same stub-`.bee/bin/bee` technique, same named-skip posture.
// The BELT PARITY test itself is NOT duplicated here — it stays in
// `opencode_plugin_contracts.rs`
// (`three_belt_parity_every_blocking_rule_hits_helper_claude_codex_and_opencode`,
// name kept because two docs cite it), where pis-3 widened its derived row
// set to include this belt.
//
// WHAT THIS FILE PROVES. Everything here runs the REAL
// `.pi/extensions/bee-guard.ts` under a real `node` subprocess against a STUB
// `pi` object and a STUB `.bee/bin/bee` binary, so every claim is about the
// shipped file rather than about a Rust re-implementation of it (pattern
// 20260812: a guard and its tests are one model, so green proves only that
// the model agrees with itself — the stub-under-node harness is what keeps
// the two models apart here).
//
//   1. BLOCKING path, fail CLOSED (D3). Every routed tool row is driven
//      through seven stub behaviors: deny (exit 2), allow (exit 0), crash
//      (exit 17), `.bee` present with the binary MISSING, an exit-0
//      `updatedInput` repair, an exit-0 `permissionDecision: "ask"`, and
//      exit-0 stdout that will not parse. Six of the seven must block; only
//      the plain allow may pass — and on that one the payload the stub bee
//      actually received on stdin is compared against an INDEPENDENT,
//      hand-authored field-shape table, so a field-name mistranslation in
//      `mapToolCall` cannot fail open and stay green (the OpenCode suite's
//      own F3 lesson, docs/knowledge/patterns/20260710-a-boundary-that-lists-
//      field-names-will-leak.md).
//   2. FAIL-SAFE unknown-tool routing. Tool names OUTSIDE the derived
//      `PI_BUILTIN_TOOLS` export — a sibling extension's `pi.registerTool`
//      tool, a future Pi built-in, a shapeless call with no `input` at all —
//      route to write-guard as write-capable calls, never a TypeScript-side
//      allow.
//   3. PASSIVITY, per call, not per load (advisor condition 2). A repo with
//      no `.bee` DIRECTORY feels nothing (no block, nothing on stderr); a
//      `.bee` directory that APPEARS mid-session starts guarding on the very
//      next call, inside the same node process, with no `/reload` — the
//      guard-that-tests-one-state hole (pattern 20260713) closed by testing
//      the transition itself.
//   4. The LINKED-WORKTREE edge: the extension loaded from a linked worktree
//      whose store and binary live only at the MAIN worktree root still finds
//      both through `git rev-parse --git-common-dir`.
//   5. D8 preamble idempotence: `session_start` + `/reload`'s second
//      `session_start` run `bee hook session-init` ONCE and inject the cached
//      preamble ONCE, while `bee hook prompt-context` still runs every turn;
//      a genuinely new session resets both.
//   6. ADVISORY surfaces never throw, under ANY stub behavior, on any of the
//      four advisory events — the fail-OPEN half of D3, kept strictly apart
//      from the fail-CLOSED half above (pattern 20260714).
//
// Node absence — or a `node` too old to strip TypeScript natively — is a
// NAMED skip, never a silent one: see `node_or_skip` and `ALLOW_SKIP_ENV`.

use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

// ─── repo layout ────────────────────────────────────────────────────────────

/// `CARGO_MANIFEST_DIR` is `packages/bee-rs/crates/bee`; four `parent()`s
/// (crates, bee-rs, packages, repo root) reach the checkout root, where
/// `.pi/extensions/bee-guard.ts` lives. Same walk as
/// `opencode_plugin_contracts.rs::repo_root`.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR too shallow: {}", env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn pi_extension_path() -> PathBuf {
    repo_root().join(".pi/extensions/bee-guard.ts")
}

/// The real Pi belt source, embedded at compile time. Every derivation below
/// reads THIS, never a hand-copied list — per
/// `docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-it-
/// never-compares-two-hand-lists.md`.
const PI_PLUGIN_SOURCE: &str = include_str!("../../../../../.pi/extensions/bee-guard.ts");

// ─── derivations from the belt's own source ────────────────────────────────

/// Pi 0.84.3's built-in tool registry as the belt itself enumerates it —
/// parsed out of the `PI_BUILTIN_TOOLS` export rather than restated here, so
/// a built-in added to or removed from that export changes this test's
/// coverage requirement on its own.
fn pi_builtin_tools() -> Vec<String> {
    const MARKER: &str = "const PI_BUILTIN_TOOLS = [";
    let start = PI_PLUGIN_SOURCE.find(MARKER).unwrap_or_else(|| {
        panic!(".pi/extensions/bee-guard.ts: `{MARKER}` not found — has the enumerated built-in tool list been renamed or reshaped?")
    });
    let rest = &PI_PLUGIN_SOURCE[start + MARKER.len()..];
    let end = rest
        .find(']')
        .expect(".pi/extensions/bee-guard.ts: unterminated PI_BUILTIN_TOOLS array literal");
    let body = &rest[..end];

    let mut tools = Vec::new();
    let mut rest = body;
    while let Some(open) = rest.find('"') {
        let after = &rest[open + 1..];
        let close = after
            .find('"')
            .expect(".pi/extensions/bee-guard.ts: unterminated string literal in PI_BUILTIN_TOOLS");
        tools.push(after[..close].to_string());
        rest = &after[close + 1..];
    }
    assert!(
        tools.len() >= 8,
        ".pi/extensions/bee-guard.ts: PI_BUILTIN_TOOLS derivation found only {} tool name(s) ({tools:?}) — \
         expected at least the eight Pi 0.84.3 built-ins; the parser or the export changed shape",
        tools.len()
    );
    tools
}

/// Every `(Pi tool, bee hook)` pair `mapToolCall`'s switch actually routes,
/// parsed from the belt's own source. Unlike the OpenCode sibling's parser
/// this one is FALL-THROUGH aware: `case "bash": case "powershell":` share
/// one body, and dropping the label without a body of its own would silently
/// under-report the routed set.
/// `mapToolCall`'s own source, sliced at the function's closing brace. The
/// bound is load-bearing, not tidiness: unbounded, the slice runs on into
/// `sessionSource`'s own `switch (reason)` and its `case "new"` /
/// `case "reload"` labels get parsed as routed TOOL names.
fn map_tool_call_body() -> &'static str {
    let fn_start = PI_PLUGIN_SOURCE
        .find("function mapToolCall")
        .expect(".pi/extensions/bee-guard.ts: mapToolCall not found — has the routing function been renamed?");
    let body = &PI_PLUGIN_SOURCE[fn_start..];
    // The function's own closing brace is the first `}` at column 0 after it;
    // every brace inside the body is indented.
    let end = body
        .find("\n}\n")
        .expect(".pi/extensions/bee-guard.ts: could not find the end of mapToolCall");
    &body[..end]
}

fn pi_tool_hook_pairs() -> Vec<(String, String)> {
    let body = map_tool_call_body();
    let switch_start = body
        .find("switch (tool)")
        .expect(".pi/extensions/bee-guard.ts: mapToolCall no longer switches on `tool` — routing derivation needs an update");
    let switch_body = &body[switch_start..];

    let mut pairs = Vec::new();
    let mut pending: Vec<String> = Vec::new();
    for seg in switch_body.split("case \"").skip(1) {
        let tool_end = seg
            .find('"')
            .expect(".pi/extensions/bee-guard.ts: unterminated `case \"...\"` tool literal");
        pending.push(seg[..tool_end].to_string());
        if let Some(h) = seg.find("hook: \"") {
            let rest = &seg[h + "hook: \"".len()..];
            let hend = rest
                .find('"')
                .expect(".pi/extensions/bee-guard.ts: unterminated `hook: \"...\"` literal");
            let hook = rest[..hend].to_string();
            for tool in pending.drain(..) {
                pairs.push((tool, hook.clone()));
            }
        }
    }
    assert!(
        pending.is_empty(),
        ".pi/extensions/bee-guard.ts: case label(s) {pending:?} fall through to no `hook:` literal — \
         a routed tool with no destination is exactly the silent allow this belt exists to close"
    );
    pairs
}

/// The `default:` arm's own source text — the FAIL-SAFE route for every tool
/// name outside `PI_BUILTIN_TOOLS`. Sliced out separately because the
/// switch-case parser above stops at the last `case`, and this arm is the one
/// place a "return null" regression would reopen the TypeScript-side allow
/// that `apply_patch` once slipped through on the OpenCode belt (oc-3).
fn pi_default_arm_source() -> &'static str {
    let body = map_tool_call_body();
    let arm_start = body
        .find("default: {")
        .expect(".pi/extensions/bee-guard.ts: mapToolCall has no `default: {` arm — the fail-safe unknown-tool route is gone");
    &body[arm_start..]
}

/// Every ADVISORY hook name the belt wires, parsed from its own
/// `runAdvisoryHook(directory, "<name>", ...)` call sites — same derivation
/// the OpenCode suite uses for its belt.
fn pi_advisory_hooks() -> BTreeSet<String> {
    const MARKER: &str = "runAdvisoryHook(directory, \"";
    let mut set = BTreeSet::new();
    let mut idx = 0usize;
    while let Some(pos) = PI_PLUGIN_SOURCE[idx..].find(MARKER) {
        let start = idx + pos + MARKER.len();
        let rest = &PI_PLUGIN_SOURCE[start..];
        let end = rest
            .find('"')
            .expect(".pi/extensions/bee-guard.ts: unterminated runAdvisoryHook name literal");
        set.insert(rest[..end].to_string());
        idx = start + end;
    }
    assert!(
        !set.is_empty(),
        ".pi/extensions/bee-guard.ts: found zero runAdvisoryHook call sites — advisory derivation broke"
    );
    set
}

// ─── node availability (named, non-fatal skip) ─────────────────────────────
//
// Identical in intent to `opencode_plugin_contracts.rs`'s probe: what matters
// is not a version number but the REAL capability the harness needs — running
// a `.ts` file directly, exactly the way Pi itself loads an extension.

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
             .pi/extensions/bee-guard.ts exactly the way Pi itself loads it — no build step) — stderr: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

/// The one opt-out surface for the environment capability this suite needs to
/// prove REAL Pi-belt coverage. Unset — the default — an absent or
/// TS-incapable `node` is a FAIL, never a silent skip: a green suite that
/// exercised zero enforcement is strictly worse than a red one, because a
/// green suite stops getting looked at (the OpenCode suite's own F1 finding).
const ALLOW_SKIP_ENV: &str = "BEE_PI_SUITE_ALLOW_SKIP";

fn env_allows_skip() -> bool {
    std::env::var_os(ALLOW_SKIP_ENV).is_some()
}

macro_rules! node_or_skip {
    ($test_name:expr) => {
        if let Err(reason) = node_typescript_probe() {
            let allow = env_allows_skip();
            eprintln!("{} (env-limited: {reason}) — {}", if allow { "SKIP" } else { "FAIL" }, $test_name);
            if allow {
                return;
            }
            panic!(
                "{}: a `node` capable of stripping TypeScript natively is required to prove real \
                 Pi enforcement coverage ({reason}) — refusing to report this test green with zero \
                 enforcement actually exercised. Set {ALLOW_SKIP_ENV}=1 to explicitly accept a \
                 degraded, unproven run in an environment that deliberately has no such node.",
                $test_name
            );
        }
    };
}

// ─── the node harness: drives the REAL extension against a stub `pi` ───────
//
// Pi hands an extension's default export an `ExtensionAPI` object and the
// extension registers handlers with `pi.on(<event>, handler)` — there is no
// returned hooks object (and, unlike OpenCode, no `directory` argument: the
// working directory arrives via `ctx.cwd`, which is why every call below
// carries one explicitly). The harness therefore builds a stub `pi`, collects
// the registrations, and then invokes named events IN ORDER inside ONE node
// process — the only way to observe process-lifetime state such as D8's
// preamble cache or the per-call passivity re-check.
const HARNESS_JS: &str = r#"
import { pathToFileURL } from "node:url";
import fs from "node:fs";

const [, , extensionPath] = process.argv;

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = "";
    process.stdin.setEncoding("utf8");
    process.stdin.on("data", (chunk) => { data += chunk; });
    process.stdin.on("end", () => resolve(data));
    process.stdin.on("error", reject);
  });
}

const spec = JSON.parse(await readStdin());

const handlers = new Map();
const pi = {
  on(event, handler) {
    if (!handlers.has(event)) handlers.set(event, []);
    handlers.get(event).push(handler);
  },
};

const mod = await import(pathToFileURL(extensionPath).href);
await mod.default(pi);

const results = [];
for (const call of spec.calls) {
  // `installFrom` copies a prepared tree over the working directory BEFORE
  // the call runs — how the "a .bee store appears mid-session" fixture
  // changes the world without restarting this process.
  if (call.installFrom) fs.cpSync(call.installFrom, call.cwd, { recursive: true });

  const list = handlers.get(call.event) ?? [];
  if (list.length === 0) {
    results.push({ threw: false, message: null, result: null, event_after: null, registered: false });
    continue;
  }
  const event = call.event_arg ?? {};
  const ctx = {
    cwd: call.cwd,
    sessionManager: { getSessionId: () => call.session_id ?? undefined },
  };
  let entry;
  try {
    let out = null;
    for (const fn of list) {
      const r = await fn(event, ctx);
      if (r !== undefined && r !== null) out = r;
    }
    entry = { threw: false, message: null, result: out, registered: true };
  } catch (err) {
    entry = { threw: true, message: String(err && err.message ? err.message : err), result: null, registered: true };
  }
  entry.event_after = event;
  results.push(entry);
}
console.log(JSON.stringify({ results }));
"#;

#[derive(Debug)]
struct CallResult {
    threw: bool,
    message: Option<String>,
    result: Option<Value>,
    event_after: Option<Value>,
    registered: bool,
}

impl CallResult {
    /// `Some(reason)` when the belt returned Pi's documented block object.
    fn block_reason(&self) -> Option<String> {
        let r = self.result.as_ref()?;
        if r.get("block").and_then(Value::as_bool) != Some(true) {
            return None;
        }
        Some(r.get("reason").and_then(Value::as_str).unwrap_or("").to_string())
    }

    fn blocked(&self) -> bool {
        self.block_reason().is_some()
    }
}

struct HarnessRun {
    results: Vec<CallResult>,
    stderr: String,
}

fn write_harness(dir: &Path) -> PathBuf {
    let path = dir.join("harness.mjs");
    std::fs::write(&path, HARNESS_JS).expect("failed to write the node harness");
    path
}

/// One node process, one ordered list of calls. Every call is
/// `{event, event_arg, cwd, session_id, installFrom?}`.
fn run_harness(harness: &Path, calls: Vec<Value>) -> HarnessRun {
    let extension = pi_extension_path();
    let spec = json!({ "calls": calls });
    let mut child = Command::new("node")
        .arg(harness)
        .arg(&extension)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|e| panic!("failed to launch `node {}`: {e}", harness.display()));
    child
        .stdin
        .take()
        .unwrap()
        .write_all(spec.to_string().as_bytes())
        .expect("failed to write the call spec to the harness's stdin");
    let out: Output = child.wait_with_output().expect("node harness never exited");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "node harness exited non-zero: stdout={} stderr={stderr}",
        String::from_utf8_lossy(&out.stdout)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last_line = stdout.lines().last().unwrap_or("").trim();
    let v: Value = serde_json::from_str(last_line).unwrap_or_else(|e| {
        panic!("node harness did not print JSON on its last stdout line: {e}; stdout={stdout} stderr={stderr}")
    });
    let results = v["results"]
        .as_array()
        .expect("node harness printed no `results` array")
        .iter()
        .map(|r| CallResult {
            threw: r["threw"].as_bool().unwrap_or(false),
            message: r["message"].as_str().map(str::to_string),
            result: r.get("result").cloned().filter(|x| !x.is_null()),
            event_after: r.get("event_after").cloned().filter(|x| !x.is_null()),
            registered: r["registered"].as_bool().unwrap_or(false),
        })
        .collect();
    HarnessRun { results, stderr }
}

fn tool_call(cwd: &Path, session_id: &str, tool: &str, input: &Value) -> Value {
    let mut event = json!({ "toolName": tool });
    if !input.is_null() {
        event["input"] = input.clone();
    }
    json!({
        "event": "tool_call",
        "event_arg": event,
        "cwd": cwd.to_string_lossy(),
        "session_id": session_id,
    })
}

fn advisory_call(event: &str, cwd: &Path, session_id: &str, event_arg: Value) -> Value {
    json!({
        "event": event,
        "event_arg": event_arg,
        "cwd": cwd.to_string_lossy(),
        "session_id": session_id,
    })
}

// ─── stub bee binaries ─────────────────────────────────────────────────────

enum StubBehavior {
    /// bee's documented DENY verdict: exit 2 with the reason on stderr.
    Deny(String),
    /// Plain allow: exit 0, empty stdout.
    Allow,
    /// Any other nonzero exit — a crash, not a verdict.
    Crash,
    /// No `.bee` directory at all: not a bee repo. The belt must be PASSIVE.
    NoStore,
    /// `.bee` directory present, `.bee/bin/bee` absent: a bee-managed repo the
    /// guard cannot decide for. The blocking path must BLOCK (D3), and this is
    /// deliberately the OPPOSITE case from `NoStore` above.
    StorePresentNoBinary,
    /// exit-0 verdict carrying `hookSpecificOutput.updatedInput`.
    Repair,
    /// exit-0 verdict carrying `permissionDecision: "ask"` — bee's own "ask,
    /// never allow" (write_guard/main.rs:389-394).
    Ask(String),
    /// exit-0 stdout that is non-empty and will not parse.
    UnparseableVerdict,
    /// exit 0, emitting a per-hook marker on stdout so the ADVISORY tests can
    /// tell the session-init preamble apart from the prompt-context delta.
    AdvisoryMarks,
}

const PREAMBLE_MARK: &str = "PREAMBLE-MARK";
const DELTA_MARK: &str = "DELTA-MARK";

/// Writes (or, for the two absent cases, deliberately does NOT write) a stub
/// `.bee/bin/bee` under `root`. `root` must be OUTSIDE this checkout (a fresh
/// `tempfile::tempdir()` already is) so the belt's git-common-dir fallback can
/// never resolve to this repo's real binary and mask the scenario under test.
///
/// Every stub CAPTURES the exact stdin bee received (`last_stdin.json`) and
/// APPENDS its argv (`calls.log`) next to itself, via `$(dirname "$0")`. The
/// capture is what makes a field-name mistranslation visible; the log is what
/// makes "the guard never ran at all" visible, which is the whole content of
/// the passivity claim.
#[cfg(unix)]
fn write_stub_bee(root: &Path, behavior: &StubBehavior) {
    use std::os::unix::fs::PermissionsExt;

    match behavior {
        StubBehavior::NoStore => return,
        StubBehavior::StorePresentNoBinary => {
            std::fs::create_dir_all(root.join(".bee")).expect("failed to create the .bee directory");
            return;
        }
        _ => {}
    }

    let bin_dir = root.join(".bee").join("bin");
    std::fs::create_dir_all(&bin_dir).expect("failed to create .bee/bin");
    let prelude = "#!/bin/sh\nd=\"$(dirname \"$0\")\"\ncat > \"$d/last_stdin.json\"\nprintf '%s\\n' \"$*\" >> \"$d/calls.log\"\n";
    let body = match behavior {
        StubBehavior::Deny(reason) => format!("echo \"{reason}\" >&2\nexit 2\n"),
        StubBehavior::Allow => "exit 0\n".to_string(),
        StubBehavior::Crash => "echo \"stub crash\" >&2\nexit 17\n".to_string(),
        StubBehavior::Repair => {
            let stdout = json!({"hookSpecificOutput": {"updatedInput": {"repairedField": "repaired-value"}}}).to_string();
            format!("printf '%s' '{stdout}'\nexit 0\n")
        }
        StubBehavior::Ask(reason) => {
            let stdout =
                json!({"hookSpecificOutput": {"permissionDecision": "ask", "permissionDecisionReason": reason}})
                    .to_string();
            format!("printf '%s' '{stdout}'\nexit 0\n")
        }
        StubBehavior::UnparseableVerdict => "printf '%s' 'not-json{{{'\nexit 0\n".to_string(),
        StubBehavior::AdvisoryMarks => format!(
            "case \"$2\" in\n  session-init) printf '%s' '{PREAMBLE_MARK}' ;;\n  prompt-context) printf '%s' '{DELTA_MARK}' ;;\nesac\nexit 0\n"
        ),
        StubBehavior::NoStore | StubBehavior::StorePresentNoBinary => unreachable!(),
    };
    let path = bin_dir.join("bee");
    std::fs::write(&path, format!("{prelude}{body}")).expect("failed to write the stub bee binary");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
        .expect("failed to make the stub bee binary executable");
}

/// The payload the stub bee actually received on stdin — the ground truth the
/// field-shape assertions compare against, never the belt's own view of it.
fn read_captured_stdin(root: &Path) -> Value {
    let path = root.join(".bee").join("bin").join("last_stdin.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("stub bee never captured stdin at {}: {e}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("captured stdin at {} was not valid JSON: {e}\ntext={text}", path.display()))
}

/// Every argv line the stub bee was invoked with, in order (`"hook write-guard"`,
/// `"hook prompt-context"`, …). An EMPTY vec means the stub was never run —
/// the positive content of the passivity claim.
fn stub_invocations(root: &Path) -> Vec<String> {
    let path = root.join(".bee").join("bin").join("calls.log");
    match std::fs::read_to_string(&path) {
        Ok(text) => text.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect(),
        Err(_) => Vec::new(),
    }
}

fn count_invocations(root: &Path, hook: &str) -> usize {
    stub_invocations(root).iter().filter(|line| line.as_str() == format!("hook {hook}")).count()
}

// ─── the independent field-shape table ─────────────────────────────────────

/// One blocking-path fixture row: the Pi tool call as a live Pi session sends
/// it, and the EXACT bee-shaped `tool_name`/`tool_input` the belt's
/// translation must produce.
///
/// Hand-authored on purpose. Deriving the expected translation from the file
/// under test would prove nothing: a field THIS table gets wrong is a bug in
/// this test, a field `mapToolCall` gets wrong is the defect this test exists
/// to catch, and only a fixed, independent expectation tells the two apart.
/// The Pi-side argument names come from the binary's own typebox schemas as
/// recorded in `PI_BUILTIN_TOOLS`' doc comment; the bee-side names come from
/// `src/hooks/write_guard/main.rs`.
struct PiCallFixture {
    /// Row label, used in failure messages.
    name: &'static str,
    tool: &'static str,
    /// `Value::Null` means the `input` key is ABSENT from the event entirely.
    input: Value,
    expected_tool_name: &'static str,
    expected_tool_input: Value,
}

fn pi_call_fixtures() -> Vec<PiCallFixture> {
    vec![
        PiCallFixture {
            name: "bash",
            tool: "bash",
            input: json!({"command": "ls -la /tmp", "timeout": 5000}),
            expected_tool_name: "Bash",
            expected_tool_input: json!({"command": "ls -la /tmp"}),
        },
        PiCallFixture {
            name: "powershell (shares the shell schema)",
            tool: "powershell",
            input: json!({"command": "Get-ChildItem"}),
            expected_tool_name: "Bash",
            expected_tool_input: json!({"command": "Get-ChildItem"}),
        },
        PiCallFixture {
            name: "write",
            tool: "write",
            input: json!({"path": "/tmp/pi-fixture/target.txt", "content": "hello"}),
            expected_tool_name: "Write",
            expected_tool_input: json!({"file_path": "/tmp/pi-fixture/target.txt", "content": "hello"}),
        },
        PiCallFixture {
            // Pi's `edit` carries an ARRAY of replacements — Claude's MultiEdit
            // shape, not its single-edit Edit shape.
            name: "edit (array of replacements -> MultiEdit)",
            tool: "edit",
            input: json!({
                "path": "/tmp/pi-fixture/target.txt",
                "edits": [{"oldText": "a", "newText": "b"}, {"oldText": "c", "newText": "d"}],
            }),
            expected_tool_name: "MultiEdit",
            expected_tool_input: json!({
                "file_path": "/tmp/pi-fixture/target.txt",
                "edits": [{"old_string": "a", "new_string": "b"}, {"old_string": "c", "new_string": "d"}],
            }),
        },
        PiCallFixture {
            name: "read (bounded)",
            tool: "read",
            input: json!({"path": "/tmp/pi-fixture/target.txt", "offset": 5, "limit": 100}),
            expected_tool_name: "Read",
            expected_tool_input: json!({"file_path": "/tmp/pi-fixture/target.txt", "offset": 5, "limit": 100}),
        },
        PiCallFixture {
            // The presence/absence signal is load-bearing: bee's unbounded-read
            // denial (write_guard/main.rs:121-124) fires only when NEITHER
            // "offset" NOR "limit" is a key. An omitted Pi argument must reach
            // bee as truly ABSENT, never as a present null.
            name: "read (unbounded — offset/limit keys must be absent, not null)",
            tool: "read",
            input: json!({"path": "/tmp/pi-fixture/target.txt"}),
            expected_tool_name: "Read",
            expected_tool_input: json!({"file_path": "/tmp/pi-fixture/target.txt"}),
        },
        PiCallFixture {
            name: "grep (glob -> include)",
            tool: "grep",
            input: json!({"pattern": "needle", "path": "/tmp/pi-fixture", "glob": "*.rs", "ignoreCase": true}),
            expected_tool_name: "Grep",
            expected_tool_input: json!({"path": "/tmp/pi-fixture", "pattern": "needle", "include": "*.rs"}),
        },
        PiCallFixture {
            name: "find",
            tool: "find",
            input: json!({"pattern": "*.rs", "path": "/tmp/pi-fixture", "limit": 20}),
            expected_tool_name: "Glob",
            expected_tool_input: json!({"path": "/tmp/pi-fixture", "pattern": "*.rs"}),
        },
        PiCallFixture {
            name: "ls",
            tool: "ls",
            input: json!({"path": "/tmp/pi-fixture", "limit": 20}),
            expected_tool_name: "Glob",
            expected_tool_input: json!({"path": "/tmp/pi-fixture"}),
        },
        // ── the FAIL-SAFE rows: names outside PI_BUILTIN_TOOLS ──────────────
        PiCallFixture {
            name: "UNMAPPED tool carrying a command string -> Bash",
            tool: "sibling_extension_shell",
            input: json!({"command": "rm -rf /tmp/pi-fixture", "cwd": "/tmp"}),
            expected_tool_name: "Bash",
            expected_tool_input: json!({"command": "rm -rf /tmp/pi-fixture"}),
        },
        PiCallFixture {
            // Everything else routes as a write-capable Write, with the first
            // path-shaped field lifted into `file_path` and the raw arguments
            // riding along untouched so no field is hidden from bee.
            name: "UNMAPPED write-capable tool -> Write with a lifted file_path",
            tool: "sibling_extension_writer",
            input: json!({"destination": "/tmp/pi-fixture/out.txt", "body": "hi"}),
            expected_tool_name: "Write",
            expected_tool_input: json!({
                "destination": "/tmp/pi-fixture/out.txt",
                "body": "hi",
                "file_path": "/tmp/pi-fixture/out.txt",
            }),
        },
        PiCallFixture {
            name: "UNMAPPED tool with no recognisable target at all",
            tool: "mystery_tool",
            input: json!({"opaque": 1}),
            expected_tool_name: "Write",
            expected_tool_input: json!({"opaque": 1, "file_path": ""}),
        },
        PiCallFixture {
            // Input malformation (plan.md test matrix): a shapeless tool_call
            // with no `input` key and an empty tool name still reaches bee.
            name: "shapeless tool_call — no input key, empty tool name",
            tool: "",
            input: Value::Null,
            expected_tool_name: "Write",
            expected_tool_input: json!({"file_path": ""}),
        },
    ]
}

fn expected_payload(session_id: &str, cwd: &Path, tool_name: &str, tool_input: &Value) -> Value {
    json!({
        "hook_event_name": "PreToolUse",
        "session_id": session_id,
        "cwd": cwd.to_string_lossy(),
        "tool_name": tool_name,
        "tool_input": tool_input,
    })
}

// ═════════════════════════════════════════════════════════════════════════
// PART 1 — derivations that need no node at all.
// ═════════════════════════════════════════════════════════════════════════

#[test]
fn every_enumerated_builtin_routes_to_the_write_guard_and_has_a_field_shape_fixture() {
    let builtins = pi_builtin_tools();
    let pairs = pi_tool_hook_pairs();
    let routed: BTreeSet<&str> = pairs.iter().map(|(t, _)| t.as_str()).collect();
    let fixtured: BTreeSet<&str> = pi_call_fixtures().iter().map(|f| f.tool).collect();

    let mut gaps: Vec<String> = Vec::new();
    for tool in &builtins {
        if !routed.contains(tool.as_str()) {
            gaps.push(format!(
                "{tool}: enumerated in PI_BUILTIN_TOOLS but mapToolCall's switch routes no case for it"
            ));
        }
        if !fixtured.contains(tool.as_str()) {
            gaps.push(format!(
                "{tool}: enumerated in PI_BUILTIN_TOOLS but this suite has no field-shape fixture row for it — \
                 add one to `pi_call_fixtures` before trusting payload coverage for this tool"
            ));
        }
    }
    for (tool, hook) in &pairs {
        if hook != "write-guard" {
            gaps.push(format!(
                "{tool} -> {hook}: the Pi belt's only BLOCKING destination is write-guard \
                 (model-guard is a NAMED EXCLUSION — Pi has no subagent surface, store 7f9c8518)"
            ));
        }
    }
    assert!(
        gaps.is_empty(),
        "Pi built-in tool coverage gap(s):\n{}\n(derived built-ins: {builtins:?}; derived routes: {pairs:?})",
        gaps.join("\n")
    );
}

#[test]
fn the_unknown_tool_route_is_fail_safe_never_a_typescript_side_allow() {
    let arm = pi_default_arm_source();
    assert!(
        !arm.contains("return null") && !arm.contains("return undefined"),
        ".pi/extensions/bee-guard.ts: mapToolCall's `default:` arm returns a null/undefined route — \
         a tool name the belt does not recognise would then be allowed on the TypeScript side, \
         which is the one bypass this belt exists to close (oc-3's apply_patch defect, ported). \
         Arm source:\n{arm}"
    );
    assert!(
        arm.matches("hook: \"write-guard\"").count() >= 2,
        ".pi/extensions/bee-guard.ts: mapToolCall's `default:` arm no longer routes BOTH unknown \
         shapes (command-carrying -> Bash, everything else -> Write) to write-guard. Arm source:\n{arm}"
    );
    // Non-vacuity: the live proof that an unmapped name really reaches bee
    // lives in the fixture rows above; this is the source-level cross-check
    // that keeps it from going quietly inert.
    let builtins = pi_builtin_tools();
    let fixtures = pi_call_fixtures();
    let unmapped: Vec<&str> =
        fixtures.iter().map(|f| f.tool).filter(|t| !builtins.iter().any(|b| b == t)).collect();
    assert!(
        unmapped.len() >= 3,
        "expected at least three UNMAPPED-tool fixture rows to exercise the fail-safe route, found {unmapped:?}"
    );
}

#[test]
fn the_belt_wires_every_advisory_surface_the_event_map_promises() {
    let wired = pi_advisory_hooks();
    for expected in ["session-init", "prompt-context", "state-sync", "session-close"] {
        assert!(
            wired.contains(expected),
            "expected .pi/extensions/bee-guard.ts to wire \"{expected}\" via runAdvisoryHook (D2's event map), \
             but the derived set was {wired:?}"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════
// PART 2 — live fixtures: the real extension, under node, against stubs.
// ═════════════════════════════════════════════════════════════════════════

#[cfg(unix)]
#[test]
fn every_routed_tool_blocks_on_deny_crash_missing_binary_ask_repair_and_unparseable() {
    node_or_skip!("every_routed_tool_blocks_on_deny_crash_missing_binary_ask_repair_and_unparseable");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let mut failures: Vec<String> = Vec::new();

    for (idx, fx) in pi_call_fixtures().into_iter().enumerate() {
        // Row-indexed rather than tool-named: one row deliberately carries an
        // EMPTY tool name (the shapeless-input row), and a reason ending in a
        // trailing space would be trimmed away before the assertion sees it.
        let session_id = format!("sess-row-{idx}");

        // (a) DENY — bee's exit-2 verdict must become Pi's block object,
        // carrying bee's own reason verbatim (this is also how a
        // `@@BEE_PRIVACY@@` marker reaches the human).
        {
            let dir = tempfile::tempdir().expect("tempdir");
            let reason = format!("stub-deny for row {idx}");
            write_stub_bee(dir.path(), &StubBehavior::Deny(reason.clone()));
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains(&reason)) {
                failures.push(format!(
                    "{}: DENY must block carrying the stub's reason (threw={}, result={:?})",
                    fx.name, r.threw, r.result
                ));
            }
        }

        // (b) ALLOW — no block, Pi's own `event.input` untouched, AND the
        // payload bee actually received matches the independent field-shape
        // table.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::Allow);
            let call = tool_call(dir.path(), &session_id, fx.tool, &fx.input);
            let run = run_harness(&harness, vec![call.clone()]);
            let r = &run.results[0];
            if r.threw || r.blocked() {
                failures.push(format!(
                    "{}: ALLOW must neither throw nor block (threw={}, result={:?})",
                    fx.name, r.threw, r.result
                ));
            }
            if r.event_after.as_ref() != Some(&call["event_arg"]) {
                failures.push(format!(
                    "{}: ALLOW must leave Pi's own event.input untouched (before={}, after={:?})",
                    fx.name, call["event_arg"], r.event_after
                ));
            }
            let captured = read_captured_stdin(dir.path());
            let expected = expected_payload(&session_id, dir.path(), fx.expected_tool_name, &fx.expected_tool_input);
            if captured != expected {
                failures.push(format!(
                    "{}: the payload bee received does not match the translated shape — expected {expected}, got {captured}",
                    fx.name
                ));
            }
        }

        // (c) CRASH — any other nonzero exit is not a verdict; fail closed.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::Crash);
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains("did not return a verdict")) {
                failures.push(format!(
                    "{}: CRASH must block with a \"did not return a verdict\" reason (result={:?})",
                    fx.name, r.result
                ));
            }
        }

        // (d) `.bee` PRESENT, binary MISSING — a bee-managed repo the guard
        // cannot decide for. Blocks (D3), and never silently allows.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::StorePresentNoBinary);
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains("could not find the bee binary")) {
                failures.push(format!(
                    "{}: a .bee store with NO binary must block with \"could not find the bee binary\" (result={:?})",
                    fx.name, r.result
                ));
            }
        }

        // (e) REPAIR — an exit-0 `updatedInput` verdict. Every route in this
        // table is FIELD-TRANSLATED (no Pi built-in maps pass-through today),
        // so the repair cannot be applied in Pi's own field space and running
        // the call unrepaired would be the silent bypass this belt closes:
        // undecidable, therefore blocked.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::Repair);
            let call = tool_call(dir.path(), &session_id, fx.tool, &fx.input);
            let run = run_harness(&harness, vec![call.clone()]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains("field space")) {
                failures.push(format!(
                    "{}: a repair verdict on a field-TRANSLATED tool must block rather than run the \
                     call unrepaired (result={:?})",
                    fx.name, r.result
                ));
            }
            if r.event_after.as_ref() != Some(&call["event_arg"]) {
                failures.push(format!(
                    "{}: a refused repair must not have been written into Pi's own event.input \
                     (before={}, after={:?})",
                    fx.name, call["event_arg"], r.event_after
                ));
            }
        }

        // (f) ASK — bee's own "ask, never allow" verdict. Pi's tool_call return
        // is two-valued, so treating "ask" as an allow would silently drop
        // write-guard's dominant enforcement path.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            let reason = format!("stub-ask for row {idx}");
            write_stub_bee(dir.path(), &StubBehavior::Ask(reason.clone()));
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains(&reason)) {
                failures.push(format!(
                    "{}: an \"ask\" verdict must block carrying the stub's reason (result={:?})",
                    fx.name, r.result
                ));
            }
        }

        // (g) UNPARSEABLE exit-0 stdout — undecidable stays fail-closed.
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::UnparseableVerdict);
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if !r.block_reason().is_some_and(|m| m.contains("could not parse")) {
                failures.push(format!(
                    "{}: an unparseable exit-0 verdict must block with a \"could not parse\" reason (result={:?})",
                    fx.name, r.result
                ));
            }
        }

        // (h) NO `.bee` DIRECTORY — passive, per call: no block, and the guard
        // is not even consulted. The contrast with (d) is the whole point:
        // "bee-less repo" and "undecidable bee repo" are different states, and
        // a guard that tests only one of them is a law with a hole (pattern
        // 20260713).
        {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), &StubBehavior::NoStore);
            let run = run_harness(&harness, vec![tool_call(dir.path(), &session_id, fx.tool, &fx.input)]);
            let r = &run.results[0];
            if r.threw || r.blocked() {
                failures.push(format!(
                    "{}: a repo with no .bee directory must feel nothing (threw={}, result={:?})",
                    fx.name, r.threw, r.result
                ));
            }
            if run.stderr.lines().any(|l| l.contains("bee ")) {
                failures.push(format!(
                    "{}: a repo with no .bee directory must stay SILENT, but the belt logged: {}",
                    fx.name, run.stderr.trim()
                ));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "one or more Pi blocking-path rows failed their fixtures:\n{}",
        failures.join("\n")
    );
}

#[cfg(unix)]
#[test]
fn a_bee_store_appearing_mid_session_starts_guarding_without_a_reload() {
    node_or_skip!("a_bee_store_appearing_mid_session_starts_guarding_without_a_reload");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());

    // The working directory starts with NO `.bee` at all…
    let work = tempfile::tempdir().expect("tempdir");
    write_stub_bee(work.path(), &StubBehavior::NoStore);
    // …and a prepared store waits elsewhere, to be copied in between calls —
    // this is `bee onboard --apply` running in another pane, mid-session.
    let staging = tempfile::tempdir().expect("tempdir");
    let reason = "stub-deny after onboarding".to_string();
    write_stub_bee(staging.path(), &StubBehavior::Deny(reason.clone()));

    let call = json!({"path": "/tmp/pi-fixture/target.txt", "content": "x"});
    let before = tool_call(work.path(), "sess-mid", "write", &call);
    let mut after = tool_call(work.path(), "sess-mid", "write", &call);
    after["installFrom"] = json!(staging.path().to_string_lossy());

    let run = run_harness(&harness, vec![before, after]);

    assert!(
        !run.results[0].blocked() && !run.results[0].threw,
        "before onboarding the belt must be passive, got {:?}",
        run.results[0]
    );
    let reason_after = run.results[1].block_reason();
    assert!(
        reason_after.as_deref().is_some_and(|m| m.contains(&reason)),
        "the SAME node process must start guarding on the very next call once a .bee store appears \
         — passivity is re-checked per call, never cached at load time (advisor condition 2). \
         Got {:?}",
        run.results[1]
    );
    assert_eq!(
        count_invocations(work.path(), "write-guard"),
        1,
        "exactly one write-guard call is expected: none before the store appeared, one after"
    );
}

#[cfg(unix)]
#[test]
fn a_linked_worktree_finds_the_store_and_binary_at_the_main_worktree_root() {
    node_or_skip!("a_linked_worktree_finds_the_store_and_binary_at_the_main_worktree_root");
    let Some(git) = git_or_skip("a_linked_worktree_finds_the_store_and_binary_at_the_main_worktree_root") else {
        return;
    };

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());

    let scratch_dir = tempfile::tempdir().expect("tempdir");
    let scratch = dunce::canonicalize(scratch_dir.path()).expect("canonicalize tempdir");
    let main_root = scratch.join("main");
    // `git worktree add` creates this directory itself and refuses a path
    // that already exists.
    let worktree = scratch.join("feature-wt");
    std::fs::create_dir_all(&main_root).unwrap();

    // A REAL linked worktree, made by git itself rather than by hand-writing
    // the `.git` file and the `worktrees/<name>/` pair: the belt resolves the
    // main root through `git rev-parse --git-common-dir`, so the fixture has
    // to be a layout git actually accepts, not one that merely looks right.
    let run_git = |args: &[&str], what: &str| {
        let out = Command::new(&git)
            .args(args)
            .output()
            .unwrap_or_else(|e| panic!("failed to run `git {}`: {e}", args.join(" ")));
        assert!(
            out.status.success(),
            "{what} failed (`git {}`): stdout={} stderr={}",
            args.join(" "),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let main_str = main_root.to_str().unwrap();
    run_git(&["-C", main_str, "init", "-q"], "git init of the fixture main worktree");
    run_git(
        &[
            "-C",
            main_str,
            "-c",
            "user.email=fixture@example.invalid",
            "-c",
            "user.name=fixture",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "init",
        ],
        "the empty root commit `git worktree add` needs a HEAD for",
    );
    run_git(
        &["-C", main_str, "worktree", "add", "-q", "--detach", worktree.to_str().unwrap()],
        "git worktree add of the fixture linked worktree",
    );

    // The store and the binary exist ONLY at the main worktree root — exactly
    // the shape a `bee worktree new` checkout has.
    let reason = "stub-deny from the main worktree root".to_string();
    write_stub_bee(&main_root, &StubBehavior::Deny(reason.clone()));
    assert!(!worktree.join(".bee").exists(), "the fixture worktree must have no .bee of its own");

    let input = json!({"path": "/tmp/pi-fixture/target.txt", "content": "x"});
    let run = run_harness(&harness, vec![tool_call(&worktree, "sess-wt", "write", &input)]);
    let r = &run.results[0];
    assert!(
        r.block_reason().is_some_and(|m| m.contains(&reason)),
        "a linked worktree must resolve the store AND the binary through `git rev-parse \
         --git-common-dir` at the main worktree root, got {r:?} (stderr={})",
        run.stderr.trim()
    );
    assert_eq!(
        count_invocations(&main_root, "write-guard"),
        1,
        "the main-root stub bee is the one that must have been consulted"
    );
}

#[cfg(unix)]
#[test]
fn the_session_preamble_is_injected_once_and_a_reload_never_re_runs_session_init() {
    node_or_skip!("the_session_preamble_is_injected_once_and_a_reload_never_re_runs_session_init");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());
    let dir = tempfile::tempdir().expect("tempdir");
    write_stub_bee(dir.path(), &StubBehavior::AdvisoryMarks);

    let turn = |prompt: &str| {
        advisory_call(
            "before_agent_start",
            dir.path(),
            "sess-d8",
            json!({"prompt": prompt, "systemPrompt": "BASE"}),
        )
    };
    let start = |reason: &str| advisory_call("session_start", dir.path(), "sess-d8", json!({"reason": reason}));

    let run = run_harness(
        &harness,
        vec![
            start("new"),      // 0 — a genuinely fresh session
            turn("first"),     // 1 — preamble + delta
            start("reload"),   // 2 — /reload: must NOT re-run session-init
            turn("second"),    // 3 — delta only
            start("new"),      // 4 — a genuinely NEW session resets the pair
            turn("third"),     // 5 — preamble again
        ],
    );

    let system_prompt = |i: usize| -> String {
        run.results[i]
            .result
            .as_ref()
            .and_then(|r| r.get("systemPrompt"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string()
    };

    let first = system_prompt(1);
    assert!(
        first.contains(PREAMBLE_MARK) && first.contains(DELTA_MARK) && first.starts_with("BASE"),
        "the FIRST turn must append the cached session preamble AND the per-turn delta onto Pi's \
         own systemPrompt, got {first:?}"
    );

    let second = system_prompt(3);
    assert!(
        !second.contains(PREAMBLE_MARK),
        "after a /reload the preamble must NOT be injected a second time (D8), got {second:?}"
    );
    assert!(
        second.contains(DELTA_MARK),
        "every turn still carries `bee hook prompt-context`'s own per-turn delta (D8), got {second:?}"
    );

    let third = system_prompt(5);
    assert!(
        third.contains(PREAMBLE_MARK),
        "a genuinely NEW session_start resets the cache and injects the preamble again, got {third:?}"
    );

    assert_eq!(
        count_invocations(dir.path(), "session-init"),
        2,
        "session-init runs once per REAL session boundary — twice here (two `new` starts), never \
         a third time for the /reload in between: invocations={:?}",
        stub_invocations(dir.path())
    );
    assert_eq!(
        count_invocations(dir.path(), "prompt-context"),
        3,
        "prompt-context is the per-turn delta and runs on EVERY turn: invocations={:?}",
        stub_invocations(dir.path())
    );
    assert!(
        run.results.iter().all(|r| !r.threw),
        "no advisory surface may throw: {:?}",
        run.results
    );
}

#[cfg(unix)]
#[test]
fn advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior() {
    node_or_skip!("advisory_surfaces_never_throw_regardless_of_the_bee_binarys_behavior");

    let harness_dir = tempfile::tempdir().expect("tempdir for the harness script");
    let harness = write_harness(harness_dir.path());

    // Every ADVISORY event the belt wires (D2's event map), including
    // deliberately shapeless payloads: a `tool_result` with no toolName and no
    // input, and a `session_start` with no reason at all.
    let events: Vec<(&str, Value)> = vec![
        ("session_start", json!({"reason": "new"})),
        ("session_start", json!({})),
        ("before_agent_start", json!({"prompt": "hello", "systemPrompt": "BASE"})),
        ("before_agent_start", json!({})),
        ("tool_result", json!({"toolName": "write", "input": {"path": "/tmp/x", "content": "hi"}})),
        ("tool_result", json!({})),
        ("agent_settled", json!({})),
    ];

    let behaviors = [
        StubBehavior::Deny("stub-deny-advisory".to_string()),
        StubBehavior::Allow,
        StubBehavior::Crash,
        StubBehavior::StorePresentNoBinary,
        StubBehavior::NoStore,
        StubBehavior::UnparseableVerdict,
        StubBehavior::Ask("stub-ask-advisory".to_string()),
        StubBehavior::Repair,
    ];

    let mut failures: Vec<String> = Vec::new();
    for (event, payload) in &events {
        for behavior in &behaviors {
            let dir = tempfile::tempdir().expect("tempdir");
            write_stub_bee(dir.path(), behavior);
            let run = run_harness(&harness, vec![advisory_call(event, dir.path(), "sess-adv", payload.clone())]);
            let r = &run.results[0];
            if !r.registered {
                failures.push(format!("{event}: the belt registers no handler for this event at all"));
                continue;
            }
            if r.threw {
                failures.push(format!(
                    "{event} threw ({:?}) — every hook it reaches is ADVISORY and must swallow and \
                     log, never throw (D3 fail OPEN; pattern 20260714)",
                    r.message
                ));
            }
            if r.blocked() {
                failures.push(format!("{event} returned a BLOCK object — advisory surfaces never block"));
            }
        }
    }

    assert!(failures.is_empty(), "Pi advisory surfaces must never throw or block:\n{}", failures.join("\n"));
}

#[cfg(not(unix))]
#[test]
fn pi_plugin_fixtures_skip_on_non_unix() {
    eprintln!(
        "SKIP (env-limited: the stub bee binaries in this suite are unix shebang scripts) \
         — pi extension fixture tests"
    );
}

// ─── git availability (named, non-fatal skip) ──────────────────────────────

/// The linked-worktree fixture is the one row that needs a real `git`, since
/// the belt resolves the main worktree root through `git rev-parse
/// --git-common-dir`. Absent git is a NAMED skip on the same terms as absent
/// node: silent by default is what turns a missing capability into a green
/// nothing.
fn git_or_skip(test_name: &str) -> Option<PathBuf> {
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => Some(PathBuf::from("git")),
        _ => {
            let allow = env_allows_skip();
            eprintln!(
                "{} (env-limited: `git` not found on PATH, needed for the linked-worktree fixture) — {test_name}",
                if allow { "SKIP" } else { "FAIL" }
            );
            if allow {
                return None;
            }
            panic!(
                "{test_name}: `git` is required to build the linked-worktree fixture the Pi belt's \
                 main-worktree fallback is about — refusing to report this test green without it. \
                 Set {ALLOW_SKIP_ENV}=1 to accept a degraded run."
            );
        }
    }
}
