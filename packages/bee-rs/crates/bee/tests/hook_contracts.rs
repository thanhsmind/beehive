// Black-box hook contracts over the SHIPPED binary.
//
// R5 port of the wrapper table in `packages/bee/hooks/test_hook_contracts.mjs`
// (the adversarial-input matrix and the Codex event-output rows). That suite
// drove each of the nine production wrappers as a child process and asserted
// three target contracts:
//
//   1. malformed / adversarial stdin NEVER crashes a hook — the only
//      acceptable exit is 0 (D2: "unsupported paths fail open with visible
//      limits");
//   2. Codex advisory events (PreCompact / SubagentStop / Stop) produce either
//      silence or a parseable JSON object carrying a string `systemMessage`,
//      and NEVER `decision: "block"` — close/nudge stay advisory and can never
//      continue a child or loop a turn;
//   3. an apply_patch PreToolUse payload runs the same gate / direct-edit /
//      reservation decisions as Edit/Write/Bash (deny on `.bee/state.json`),
//      while the model guard simply ignores the unfamiliar shape.
//
// These stay black-box rather than becoming in-module unit tests on purpose:
// the .mjs suite's value was that it went through the real process boundary —
// stdin framing, the argv shape, the exit code the host actually observes.
// A unit test on `run_native` would not catch a panic in the stdin reader or a
// bad exit-code conversion.
//
// PROVING THE NATIVE PATH RAN: every row is executed with the crate's own
// `BEE_HOOK_NO_DELEGATE` tripwire armed (hooks/mod.rs). Without it a hook that
// delegated to Node would answer correctly and the row would pass while
// testing nothing; with it, a delegation exits 42 and names itself. Rows that
// delegate print a loud SKIP naming the hook instead of passing quietly, and
// each matrix asserts that enough rows stayed native for the test to mean
// something — a suite where everything delegated must never report green.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

/// Every hook `bee hook <name>` serves, in hooks/mod.rs dispatch order.
const HOOKS: &[&str] = &[
    "tools-logger",
    "activity",
    "codex-subagent-audit",
    "chain-nudge",
    "state-sync",
    "prompt-context",
    "session-init",
    "session-close",
    "model-guard",
    "write-guard",
];

const TRIPWIRE_EXIT: i32 = 42;

fn bee_bin() -> PathBuf {
    assert_cmd::cargo::cargo_bin("bee")
}

// R6 CUTOVER — `copy_vendored_lib` is GONE, and its job with it.
//
// It used to copy this repo's `.bee/bin/lib/*.mjs` into every fixture, because
// each hook began `if (!existsSync(storeRoot/.bee/bin/lib/state.mjs)) return 0;`
// ("bee is not installed here, decide nothing"), and the write guard
// additionally byte-compared the whole 26-file lib closure before trusting its
// own decision. A fixture without the lib made every hook exit 0 silently,
// which would have turned this entire matrix into a vacuous pass — hence the
// copy, and hence its `panic!` when the source directory was missing.
//
// Both probes were re-keyed onto `.bee/onboarding.json` BEFORE the deletion
// (see hooks/write_guard.rs's byte-gate note): the install marker every onboard
// writes and nothing else does. `fixture()` writes that marker, so the hooks
// reach their real decision tables exactly as before. The lib copy would now
// populate a directory nothing reads, and its panic on the deleted source would
// be the only part of it still doing anything.
//
// NON-VACUITY is still proven, by the same evidence it always was: the DENY
// cases below (`a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr`,
// `a_model_guard_deny_reaches_the_host_as_exit_two_on_stderr`,
// `apply_patch_targeting_state_json_is_denied_by_the_write_guard`) assert
// exit 2 and specific stderr. A fixture the hooks decline to act on cannot
// produce those, so a silently-disabled matrix goes RED here, not green.

/// A fixture repo the hooks will accept as a bee root: `.bee/` with a state
/// file in an execution-approved phase, so the write guard reaches its real
/// decision table rather than short-circuiting on a missing root.
struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
}

fn fixture() -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(dir.path()).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("logs")).unwrap();
    std::fs::create_dir_all(root.join(".bee").join("cells")).unwrap();
    // THE install marker — what every hook's activation probe reads since the
    // R6 cutover. Without it the hooks decline every payload and this whole
    // matrix goes vacuously green (see the note above).
    std::fs::write(root.join(".bee").join("onboarding.json"), "{}\n").unwrap();
    std::fs::write(
        root.join(".bee").join("state.json"),
        serde_json::to_string_pretty(&serde_json::json!({
            "phase": "swarming",
            "mode": "standard",
            "feature": "demo",
            "approved_gates": {
                "context": true, "shape": true, "execution": true, "review": false
            }
        }))
        .unwrap()
            + "\n",
    )
    .unwrap();
    Fixture { _dir: dir, root }
}

/// Run `bee hook <name>` with the given stdin bytes, tripwire armed.
fn run_hook(hook: &str, stdin: &[u8], cwd: &Path) -> Output {
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
    child
        .wait_with_output()
        .unwrap_or_else(|e| panic!("`bee hook {hook}` never exited: {e}"))
}

fn code(out: &Output) -> i32 {
    out.status.code().unwrap_or(-1)
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// The tripwire's own verdict, separated from the contract under test. A
/// delegation is not a failure — some branches are documented, deliberate
/// delegates until Node is deleted (plans/rust-port.md, "Cross-cutting
/// delegate classes") — but it IS a row that proves nothing about the Rust
/// path, so it is announced by name rather than passing quietly. Callers
/// count these; a suite where everything delegated must not report green.
fn ran_native(hook: &str, out: &Output, row: &str) -> bool {
    if code(out) == TRIPWIRE_EXIT {
        eprintln!(
            "SKIP (unported: `bee hook {hook}` still delegates to Node on this input \
             — BEE_HOOK_NO_DELEGATE tripwire) — row {row}"
        );
        return false;
    }
    true
}

/// A hook must never fail closed over infrastructure: a panic (or any nonzero
/// exit) on malformed input is exactly the defect this matrix exists to catch.
/// Returns true when the row actually exercised the native path.
fn assert_no_crash(hook: &str, row: &str, out: &Output) -> bool {
    if !ran_native(hook, out, row) {
        return false;
    }
    assert_eq!(
        code(out),
        0,
        "row {row}: `bee hook {hook}` must fail OPEN (exit 0) on malformed or \
         adversarial input; got exit {} stderr={}",
        code(out),
        stderr(out)
    );
    let err = stderr(out);
    assert!(
        !err.contains("panicked at") && !err.contains("RUST_BACKTRACE"),
        "row {row}: `bee hook {hook}` panicked instead of failing open: {err}"
    );
    true
}

// ── 1. the universal adversarial-input matrix ─────────────────────────────
// Provenance: test_hook_contracts.mjs `universalRows()`.

#[test]
fn every_hook_fails_open_on_adversarial_stdin() {
    let fx = fixture();
    let object_cwd = serde_json::json!({ "cwd": { "not": "a string" } }).to_string();
    let rows: Vec<(&str, Vec<u8>)> = vec![
        ("empty-stdin", Vec::new()),
        ("junk-bytes", b"not json at all {{{".to_vec()),
        ("top-level-null", b"null".to_vec()),
        ("json-array", b"[]".to_vec()),
        ("object-cwd", object_cwd.into_bytes()),
        (
            "missing-cwd",
            serde_json::json!({ "tool_name": "Read" }).to_string().into_bytes(),
        ),
        // Not JSON-shaped at all, but valid UTF-8 with embedded NULs — the
        // stdin reader must survive bytes it can never parse.
        ("nul-bytes", b"\0\0\0{\"cwd\":\0}".to_vec()),
    ];
    let mut native = 0usize;
    let total = HOOKS.len() * rows.len();
    for hook in HOOKS {
        for (row, input) in &rows {
            let out = run_hook(hook, input, &fx.root);
            if assert_no_crash(hook, row, &out) {
                native += 1;
            }
        }
    }
    // Not a coverage target — a vacuity guard. If delegation ever swallowed
    // the whole matrix this test would report green having asserted nothing.
    assert!(
        native * 2 > total,
        "only {native}/{total} adversarial rows reached the native path — this \
         matrix is no longer testing the Rust hooks"
    );
}

#[test]
fn every_hook_survives_a_two_megabyte_payload() {
    // Provenance: the `huge-payload-2mb` row. A hook that buffers stdin badly
    // — or hands the whole blob to a parser with a small limit — dies here.
    let fx = fixture();
    let payload = serde_json::json!({
        "cwd": fx.root.to_string_lossy(),
        "blob": "a".repeat(2 * 1024 * 1024),
    })
    .to_string()
    .into_bytes();
    let mut native = 0usize;
    for hook in HOOKS {
        let out = run_hook(hook, &payload, &fx.root);
        if assert_no_crash(hook, "huge-payload-2mb", &out) {
            native += 1;
        }
    }
    assert!(native > 0, "every hook delegated the 2MB row — nothing was asserted");
}

#[test]
fn an_unknown_hook_name_is_a_named_refusal_not_a_crash() {
    let fx = fixture();
    let out = run_hook("not-a-real-hook", b"{}", &fx.root);
    assert_ne!(code(&out), 0, "an unknown hook name must not report success");
    assert!(
        stderr(&out).contains("not-a-real-hook"),
        "the refusal must name the hook it did not recognize; got: {}",
        stderr(&out)
    );
}

// ── 2. Codex advisory event output ────────────────────────────────────────
// Provenance: test_hook_contracts.mjs `eventRows()` +
// `expectAdvisoryJsonOrSilent`. The event→wrapper pairing mirrors the real
// matchers in hooks/hooks.json, not a cross product: PreCompact fires only
// session-close; SubagentStop fires state-sync + chain-nudge; Stop fires
// state-sync + session-close.

fn assert_advisory_or_silent(hook: &str, event: &str, out: &Output) {
    let row = format!("codex-{}-advisory", event.to_lowercase());
    if !ran_native(hook, out, &row) {
        return;
    }
    assert_eq!(
        code(out),
        0,
        "{row}/{hook}: an advisory event must exit 0 (advisory-only, never \
         blocking); got {} stderr={}",
        code(out),
        stderr(out)
    );
    let text = stdout(out);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return; // silence is a legitimate advisory outcome
    }
    let parsed: serde_json::Value = serde_json::from_str(trimmed).unwrap_or_else(|e| {
        panic!(
            "{row}/{hook}: stdout is non-empty but is not parseable JSON — Codex \
             advisory output must be a JSON systemMessage, not plain prose ({e}). \
             stdout: {trimmed}"
        )
    });
    assert!(
        parsed.is_object(),
        "{row}/{hook}: advisory stdout must be a JSON OBJECT; got {trimmed}"
    );
    assert!(
        parsed.get("systemMessage").and_then(|v| v.as_str()).is_some(),
        "{row}/{hook}: advisory JSON lacks a string `systemMessage`; got {trimmed}"
    );
    assert_ne!(
        parsed.get("decision").and_then(|v| v.as_str()),
        Some("block"),
        "{row}/{hook}: an advisory used decision:\"block\" — close/nudge must stay \
         advisory and can never block a turn. stdout: {trimmed}"
    );
}

#[test]
fn advisory_events_never_block_a_turn() {
    let fx = fixture();
    let pairs = [
        ("session-close", "PreCompact"),
        ("state-sync", "SubagentStop"),
        ("chain-nudge", "SubagentStop"),
        ("state-sync", "Stop"),
        ("session-close", "Stop"),
    ];
    let mut delegated: Vec<&str> = Vec::new();
    for (hook, event) in pairs {
        let input = serde_json::json!({
            "hook_event_name": event,
            "agent_name": "kevin",
            "cwd": fx.root.to_string_lossy(),
        })
        .to_string();
        let out = run_hook(hook, input.as_bytes(), &fx.root);
        if code(&out) == TRIPWIRE_EXIT {
            // session-close's PreCompact branch is a documented delegate class
            // (plans/rust-port.md, "Cross-cutting delegate classes"). Record it
            // rather than asserting a contract the Rust path never reached.
            delegated.push(event);
            eprintln!(
                "SKIP (unported: `{hook}` still delegates the {event} branch to Node — \
                 plans/rust-port.md cross-cutting delegate classes) — advisory shape for {hook}/{event}"
            );
            continue;
        }
        assert_advisory_or_silent(hook, event, &out);
    }
    // Guard against the whole test going vacuous if delegation ever widens.
    assert!(
        delegated.len() < pairs.len(),
        "every advisory row delegated — this test asserted nothing"
    );
}

#[test]
fn chain_nudge_advisory_names_the_returning_worker() {
    // The advisory's VALUE, not its prose: the agent_name that came in on the
    // payload must reach the rendered systemMessage
    // (expertise/tests/patterns/asserting-on-generated-output.md).
    let fx = fixture();
    let input = serde_json::json!({
        "hook_event_name": "SubagentStop",
        "agent_name": "kevin",
        "cwd": fx.root.to_string_lossy(),
    })
    .to_string();
    let out = run_hook("chain-nudge", input.as_bytes(), &fx.root);
    assert_eq!(code(&out), 0, "stderr: {}", stderr(&out));
    let v: serde_json::Value = serde_json::from_str(stdout(&out).trim())
        .expect("chain-nudge advisory must be parseable JSON");
    let msg = v["systemMessage"].as_str().expect("string systemMessage");
    assert!(msg.contains("kevin"), "the advisory must name the worker: {msg}");
    assert!(!msg.contains("{{"), "no placeholder syntax may survive: {msg}");
}

// ── 3. apply_patch runs the same decisions as Edit/Write/Bash ─────────────
// Provenance: `codex-pretooluse-applypatch-deny` /
// `codex-pretooluse-applypatch-ignored`.

fn apply_patch_payload(root: &Path) -> String {
    serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "apply_patch",
        "tool_input": {
            "input": "*** Begin Patch\n*** Update File: .bee/state.json\n@@\n-old\n+new\n*** End Patch"
        },
        "cwd": root.to_string_lossy(),
    })
    .to_string()
}

#[test]
fn apply_patch_targeting_state_json_is_denied_by_the_write_guard() {
    let fx = fixture();
    let out = run_hook("write-guard", apply_patch_payload(&fx.root).as_bytes(), &fx.root);
    assert!(
        ran_native("write-guard", &out, "codex-pretooluse-applypatch-deny"),
        "the write guard must serve apply_patch natively — a delegation here means the \n         apply_patch decision path is untested"
    );
    assert_eq!(
        code(&out),
        2,
        "an apply_patch write to .bee/state.json must be denied exactly like an \
         Edit/Write/Bash write is; got exit {} stderr={}",
        code(&out),
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(!err.trim().is_empty(), "a deny must carry a corrective reason");
    assert!(
        err.contains(".bee/state.json") || err.contains("state.json"),
        "the deny must name the offending target: {err}"
    );

    // Control (proving-the-code-under-test-ran): the same envelope with a safe
    // target passes, so the exit-2 above is the direct-edit rule firing rather
    // than apply_patch being rejected wholesale.
    let safe = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "apply_patch",
        "tool_input": { "input": "*** Begin Patch\n*** Add File: src/new.rs\n+fn main() {}\n*** End Patch" },
        "cwd": fx.root.to_string_lossy(),
    })
    .to_string();
    let ok = run_hook("write-guard", safe.as_bytes(), &fx.root);
    assert_eq!(code(&ok), 0, "a safe apply_patch target must pass: {}", stderr(&ok));
}

#[test]
fn apply_patch_is_ignored_by_the_model_guard() {
    // approach.md keeps the model-tier guard Claude-only; a PreToolUse
    // apply_patch is simply not a dispatch tool, and the only requirement is
    // that the unfamiliar shape does not crash the wrapper.
    let fx = fixture();
    let out = run_hook("model-guard", apply_patch_payload(&fx.root).as_bytes(), &fx.root);
    assert_no_crash("model-guard", "codex-pretooluse-applypatch-ignored", &out);
    assert_eq!(
        stderr(&out),
        "",
        "the model guard must stay silent on a non-dispatch tool"
    );
}

// ── 4. the guards' decisions survive the process boundary ─────────────────
// The in-module unit tests in hooks/write_guard.rs and hooks/model_guard.rs
// cover the decision tables exhaustively against `run_native`. These two rows
// exist only to prove the DECISION reaches the host: the exit code and the
// stderr channel a real hook consumer reads.

#[test]
fn a_write_guard_deny_reaches_the_host_as_exit_two_on_stderr() {
    let fx = fixture();
    let input = serde_json::json!({
        "tool_name": "Edit",
        "tool_input": { "file_path": ".bee/state.json" },
        "cwd": fx.root.to_string_lossy(),
    })
    .to_string();
    let out = run_hook("write-guard", input.as_bytes(), &fx.root);
    assert!(ran_native("write-guard", &out, "direct-edit-deny"));
    assert_eq!(code(&out), 2);
    assert!(stdout(&out).is_empty(), "a deny writes its reason to stderr, not stdout");
    assert!(stderr(&out).contains("FIX"), "a deny carries corrective guidance");
}

#[test]
fn a_model_guard_deny_reaches_the_host_as_exit_two_on_stderr() {
    let fx = fixture();
    let input = serde_json::json!({
        "tool_name": "Agent",
        "tool_input": { "prompt": "implement the widget", "description": "some description" },
        "cwd": fx.root.to_string_lossy(),
    })
    .to_string();
    let out = run_hook("model-guard", input.as_bytes(), &fx.root);
    assert!(ran_native("model-guard", &out, "bare-dispatch-deny"));
    assert_eq!(code(&out), 2, "stderr: {}", stderr(&out));
    assert!(stderr(&out).contains("bee-tier"), "{}", stderr(&out));
    assert!(stderr(&out).contains("FIX"), "{}", stderr(&out));
}

// ── 5. a hook never fails closed over missing infrastructure ─────────────
// Provenance: hooks/mod.rs — "A hook must NEVER fail closed over
// infrastructure: exit 0 silently." Run from a directory with no bee root at
// all; every hook must still exit 0 and stay silent on stdout.

#[test]
fn no_bee_root_is_silent_success_for_every_hook() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dunce::canonicalize(dir.path()).unwrap();
    for hook in HOOKS {
        let input = serde_json::json!({ "cwd": outside.to_string_lossy() }).to_string();
        let out = run_hook(hook, input.as_bytes(), &outside);
        if code(&out) == TRIPWIRE_EXIT {
            continue; // known delegate, covered by the matrix above
        }
        assert_eq!(
            code(&out),
            0,
            "`bee hook {hook}` must exit 0 outside a bee repo; got {} stderr={}",
            code(&out),
            stderr(&out)
        );
    }
}
