//! Drives `HerdrBackend` against a PATH-prepended stub `herdr` binary — the
//! shape `packages/bee-rs/crates/bee/src/shell.rs:74-89` established for a
//! Windows Git-Bash resolver, reused here so this crate's whole test suite
//! runs green with NO real herdr installed (D7). Every test builds its own
//! throwaway stub directory; nothing here talks to a live herdr server.
//!
//! Unix-only (`#[cfg(unix)]`, gating the whole file): the stub is a POSIX
//! shell script, and this crate has no Windows-native stub-binary
//! equivalent yet — a named, recorded gap, not a silent one. The pure
//! status-mapping logic these tests exercise indirectly is ALSO covered by
//! `src/backend/herdr.rs`'s own `#[cfg(test)]` unit tests
//! (`map_status`, `find_agent_entry`, `is_pane_id_shaped`), which have no
//! process dependency and run on every platform including Windows.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use fleet::backend::herdr::HerdrBackend;
use fleet::backend::{WorkerBackend, WorkerStatus};
use fleet::wave::WorkerSpec;

static STUB_SEQ: AtomicU64 = AtomicU64::new(0);

/// A throwaway PATH-prepended stub `herdr`. `dir` holds the executable
/// script itself (what gets prepended to the child's `PATH`); `state`
/// holds the response queues `queue_response` writes into; `log` records
/// every invocation's argv, one line per call, for tests that need to
/// assert on exactly what was passed (for example: never `--wait`).
struct Stub {
    dir: PathBuf,
    state: PathBuf,
    log: PathBuf,
    spill: PathBuf,
}

impl Stub {
    /// Builds a fresh stub in its own scratch directory — unique per call
    /// (a process-wide counter folded into the dir name) so parallel
    /// `cargo test` threads never share state.
    fn new() -> Self {
        let seq = STUB_SEQ.fetch_add(1, Ordering::SeqCst);
        let root = std::env::temp_dir().join(format!(
            "fleet-herdr-stub-{}-{seq}",
            std::process::id()
        ));
        let dir = root.join("bin");
        let state = root.join("state");
        let spill = root.join("spill");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::create_dir_all(&spill).unwrap();
        let log = root.join("invocations.log");

        let script_path = dir.join("herdr");
        // `Q` is keyed on `$1-$2` (a hyphen join of the first two argv
        // words, e.g. `agent-list`) so it never has to be kept in sync
        // with a hardcoded per-verb list — `queue_response` below joins
        // its own `verb` argument the same way.
        //
        // The log line is NOT a plain `"$*"` + newline: a real argument
        // here (the wrapped task text `send` builds) contains embedded
        // newlines of its own, which would otherwise fragment one
        // invocation into several log "lines". Each arg is instead
        // written with a trailing ASCII Unit Separator (`\037`), and each
        // whole invocation with a trailing ASCII Record Separator
        // (`\036`) — both control characters no real argument here uses —
        // so `Stub::invocations` can split unambiguously regardless of
        // what an argument's own text contains.
        let script = format!(
            "#!/bin/sh\n\
             for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> {log}\n\
             printf '\\036' >> {log}\n\
             if [ -z \"$1\" ] || [ -z \"$2\" ]; then\n\
             printf '%s' '{{\"type\":\"error\",\"message\":\"stub: unhandled verb\"}}' 1>&2\n\
             exit 1\n\
             fi\n\
             Q={state}/\"$1-$2\"\n\
             COUNTER_FILE=\"$Q.counter\"\n\
             MAX_FILE=\"$Q.max\"\n\
             COUNT=0\n\
             [ -f \"$COUNTER_FILE\" ] && COUNT=$(cat \"$COUNTER_FILE\")\n\
             MAX=0\n\
             [ -f \"$MAX_FILE\" ] && MAX=$(cat \"$MAX_FILE\")\n\
             IDX=$COUNT\n\
             if [ \"$IDX\" -gt \"$MAX\" ]; then IDX=$MAX; fi\n\
             NEXT=$((COUNT + 1))\n\
             echo \"$NEXT\" > \"$COUNTER_FILE\"\n\
             RESP_FILE=\"$Q.$IDX\"\n\
             EXIT_FILE=\"$RESP_FILE.exit\"\n\
             if [ -f \"$RESP_FILE\" ]; then cat \"$RESP_FILE\"; fi\n\
             if [ -f \"$EXIT_FILE\" ]; then exit \"$(cat \"$EXIT_FILE\")\"; fi\n\
             exit 0\n",
            log = shell_quote(&log),
            state = shell_quote(&state),
        );
        fs::write(&script_path, script).unwrap();
        let mut perms = fs::metadata(&script_path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms).unwrap();

        Self { dir, state, log, spill }
    }

    /// Queues `body` as the `index`-th response `herdr <verb-group> <verb>
    /// ...` returns (0-based; calls beyond the highest queued index keep
    /// returning the LAST one, the fake backend's own "steady value"
    /// idea, reproduced here in shell). `exit_code` simulates a non-zero
    /// exit when `Some`. `verb` is a space-separated pair like `"agent
    /// list"` — hyphenated here to match the stub script's own `$1-$2`
    /// key.
    fn queue_response(&self, verb: &str, index: u32, body: &str, exit_code: Option<i32>) {
        let key = verb.replace(' ', "-");
        let base = self.state.join(key);
        let resp_path = PathBuf::from(format!("{}.{index}", base.display()));
        fs::write(&resp_path, body).unwrap();
        if let Some(code) = exit_code {
            fs::write(format!("{}.exit", resp_path.display()), code.to_string()).unwrap();
        }
        let max_path = PathBuf::from(format!("{}.max", base.display()));
        let current_max: u32 = fs::read_to_string(&max_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        if index >= current_max {
            fs::write(&max_path, index.to_string()).unwrap();
        }
    }

    fn backend(&self) -> HerdrBackend {
        self.backend_with_kind("claude")
    }

    /// Same as `backend`, but with an explicit agent kind — used by the
    /// test proving the kind reaching `agent start --kind` is
    /// `HerdrBackend`'s own construction parameter (D14), never a
    /// hardcoded literal.
    fn backend_with_kind(&self, kind: &str) -> HerdrBackend {
        HerdrBackend::with_test_seams(vec![self.dir.clone()], self.spill.clone(), kind, Vec::new())
    }

    /// Every invocation, reconstructed as its args space-joined (matching
    /// the shape a plain `"$*"` log would have shown), in call order. Uses
    /// the Unit/Record Separator control characters `Stub::new`'s script
    /// wrote — never a plain newline split, which a multi-line argument
    /// (like `send`'s wrapped task text) would fragment.
    fn invocations(&self) -> Vec<String> {
        let raw = fs::read_to_string(&self.log).unwrap_or_default();
        raw.split('\u{1e}')
            .filter(|record| !record.is_empty())
            .map(|record| {
                record
                    .split('\u{1f}')
                    .filter(|arg| !arg.is_empty())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }
}

/// Wraps `path` in single quotes for embedding in the generated `sh`
/// script — every path used here is a scratch temp-dir path this test
/// itself created, never external input, so a plain quote-and-escape is
/// enough.
fn shell_quote(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

const OK_AGENT_LIST_EMPTY: &str = r#"{"type":"ok","result":{"agents":[]}}"#;

fn agent_list_with(name: &str, pane_id: &str, agent_status: &str) -> String {
    format!(
        r#"{{"type":"ok","result":{{"agents":[{{"name":"{name}","pane_id":"{pane_id}","agent_status":"{agent_status}"}}]}}}}"#
    )
}

// ── canonical_id ────────────────────────────────────────────────────────

#[test]
fn canonical_id_resolves_a_friendly_name_to_its_pane_id_via_agent_list() {
    let stub = Stub::new();
    stub.queue_response(
        "agent list",
        0,
        &agent_list_with("reviewer-1", "w4:pB", "idle"),
        None,
    );
    let backend = stub.backend();

    assert_eq!(backend.canonical_id("reviewer-1"), "w4:pB");
}

#[test]
fn canonical_id_is_a_no_op_for_an_already_pane_id_shaped_identifier() {
    // No `agent list` call is even needed to resolve an identifier that is
    // already herdr's own canonical addressing form — proven here by
    // never queuing a response at all: an unconfigured stub call would
    // return an empty body (unparseable), so a passing result proves no
    // call happened.
    let stub = Stub::new();
    let backend = stub.backend();
    assert_eq!(backend.canonical_id("w4:pB"), "w4:pB");
    assert!(
        stub.invocations().is_empty(),
        "resolving a pane-id-shaped identifier must not call herdr at all"
    );
}

#[test]
fn canonical_id_falls_back_to_identity_when_herdr_has_never_seen_the_name() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, OK_AGENT_LIST_EMPTY, None);
    let backend = stub.backend();

    assert_eq!(
        backend.canonical_id("nobody-herdr-has-heard-of"),
        "nobody-herdr-has-heard-of",
        "an identifier absent from `agent list` must resolve to itself, never collapse to a \
         guess (herding-orchestration D15)"
    );
}

#[test]
fn canonical_id_falls_back_to_identity_when_the_lookup_itself_fails() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, r#"{"type":"error"}"#, Some(1));
    let backend = stub.backend();

    assert_eq!(
        backend.canonical_id("reviewer-1"),
        "reviewer-1",
        "a failed canonical-identity lookup must fall back to the identifier unchanged, the \
         same safe no-collapse default as an unknown name"
    );
}

// ── status: the fail-closed law (D7, Ordering Invariant 4) ────────────────

#[test]
fn status_maps_every_herdr_state_through_a_live_agent_list_call() {
    for (raw, expected) in [
        ("idle", WorkerStatus::Ready),
        ("working", WorkerStatus::Working),
        ("blocked", WorkerStatus::Blocked),
        ("done", WorkerStatus::Finished),
        ("unknown", WorkerStatus::Unverifiable),
    ] {
        let stub = Stub::new();
        stub.queue_response("agent list", 0, &agent_list_with("w1", "w4:pB", raw), None);
        let backend = stub.backend();
        assert_eq!(backend.status("w1"), expected, "herdr status {raw:?}");
    }
}

#[test]
fn status_is_unverifiable_on_a_non_zero_exit() {
    let stub = Stub::new();
    stub.queue_response(
        "agent list",
        0,
        r#"{"type":"error","message":"server unavailable"}"#,
        Some(1),
    );
    let backend = stub.backend();
    assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
}

#[test]
fn status_is_unverifiable_on_an_unparseable_body() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, "not json at all {{{", None);
    let backend = stub.backend();
    assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
}

#[test]
fn status_is_unverifiable_when_the_agent_status_field_is_missing() {
    let stub = Stub::new();
    stub.queue_response(
        "agent list",
        0,
        r#"{"type":"ok","result":{"agents":[{"name":"w1","pane_id":"w4:pB"}]}}"#,
        None,
    );
    let backend = stub.backend();
    assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
}

#[test]
fn status_is_unverifiable_when_agent_status_is_an_off_enum_value() {
    let stub = Stub::new();
    stub.queue_response(
        "agent list",
        0,
        &agent_list_with("w1", "w4:pB", "confused"),
        None,
    );
    let backend = stub.backend();
    assert_eq!(backend.status("w1"), WorkerStatus::Unverifiable);
}

#[test]
fn status_is_unverifiable_when_the_target_is_absent_from_the_list() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, OK_AGENT_LIST_EMPTY, None);
    let backend = stub.backend();
    assert_eq!(backend.status("ghost"), WorkerStatus::Unverifiable);
}

// ── Hazard: a status read must not depend on the pane having been
// focused (spawn-proof.md Step 4 / Takeaway 3) ─────────────────────────

#[test]
fn status_does_not_depend_on_the_pane_having_been_focused() {
    // Reproduces the live round trip: a `done` reading with `focused:
    // false` on the target pane, no focus command issued by this call at
    // all — `agent list` is the ONLY herdr invocation `status` ever makes
    // (asserted below), so there is no focus command for it to depend on.
    let stub = Stub::new();
    stub.queue_response(
        "agent list",
        0,
        r#"{"type":"ok","result":{"agents":[
            {"name":"w1","pane_id":"w4:pB","agent_status":"done","focused":false}
        ]}}"#,
        None,
    );
    let backend = stub.backend();

    assert_eq!(
        backend.status("w1"),
        WorkerStatus::Finished,
        "a `done` reading must map to Finished regardless of the pane's own `focused` field"
    );
    let invocations = stub.invocations();
    assert_eq!(invocations.len(), 1, "status must make exactly one herdr call; got {invocations:?}");
    assert!(
        invocations[0].starts_with("agent list"),
        "the one call status makes must be a plain `agent list` read, never a focus command; \
         got {:?}",
        invocations[0]
    );
}

// ── Hazard: a prompt sent to an already-working agent must not report
// the previous turn's completion as this send's ─────────────────────────

#[test]
fn send_never_passes_wait_so_a_busy_agents_prior_turn_is_never_misreported_as_this_sends() {
    let stub = Stub::new();
    stub.queue_response("agent prompt", 0, r#"{"type":"ok","result":{}}"#, None);
    let backend = stub.backend();

    backend.send("w1", "please reply with PONG").unwrap();

    let invocations = stub.invocations();
    assert_eq!(invocations.len(), 1, "got {invocations:?}");
    let call = &invocations[0];
    assert!(call.starts_with("agent prompt w1"), "got {call:?}");
    assert!(
        !call.contains("--wait"),
        "send must never pass --wait: on a herdr 0.8.0 agent already mid-turn from something \
         outside this wave's control, --wait can settle on that turn's own completion and \
         report it as this send's (docs/history/research/herdr-orchestrator-distill.md, \
         \"agent prompt --wait does not track turns\") — completion is confirmed by the \
         choreography's own baseline+marker poll instead; got {call:?}"
    );
    assert!(
        call.contains("please reply with PONG"),
        "the task text must still reach herdr; got {call:?}"
    );
}

// ── Hazard: a response longer than the read window is still recovered
// (the alternate-screen read failure) ──────────────────────────────────

#[test]
fn read_output_recovers_a_reply_spilled_to_a_file_when_longer_than_the_read_window() {
    let stub = Stub::new();
    let backend = stub.backend();

    // `send`'s own spill path for "w1" — read_output must agree with it
    // exactly, since nothing else tells the two methods about each other.
    let spill_path = stub.spill.join("bee-fleet-herdr-w1.reply");
    let long_reply = "X".repeat(50_000); // far longer than any realistic terminal scrollback window
    fs::write(&spill_path, &long_reply).unwrap();

    // The transcript herdr hands back is just the bare spill path on its
    // own line — exactly the hand-back herdr's own documented fallback
    // describes (the agent wrote its full reply to a file and replied
    // with the path).
    let read_body = format!(
        r#"{{"type":"ok","result":{{"text":"❯ please reply\n\n● {}\n"}}}}"#,
        spill_path.display()
    );
    stub.queue_response("agent read", 0, &read_body, None);

    let recovered = backend.read_output("w1").unwrap();
    assert_eq!(
        recovered, long_reply,
        "a reply too long for the read window must still be recovered in full via the spill \
         file, not truncated to the bare path herdr's own transcript read returned"
    );
}

#[test]
fn read_output_returns_the_transcript_unchanged_when_nothing_was_spilled() {
    let stub = Stub::new();
    stub.queue_response(
        "agent read",
        0,
        r#"{"type":"ok","result":{"text":"❯ hi\n\n● DONE.\n"}}"#,
        None,
    );
    let backend = stub.backend();

    assert_eq!(backend.read_output("w1").unwrap(), "❯ hi\n\n● DONE.\n");
}

// ── start ──────────────────────────────────────────────────────────────

#[test]
fn start_is_idempotent_when_the_target_is_already_running() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, &agent_list_with("w1", "w4:pB", "idle"), None);
    let backend = stub.backend();

    backend.start(&WorkerSpec::new("w1", "task")).unwrap();

    let invocations = stub.invocations();
    assert_eq!(
        invocations.len(),
        1,
        "an already-running target must cost exactly one `agent list` check and no \
         `agent start` call; got {invocations:?}"
    );
    assert!(invocations[0].starts_with("agent list"), "got {:?}", invocations[0]);
}

#[test]
fn start_refuses_a_never_seen_non_pane_shaped_name_rather_than_fabricate_a_spawn() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, OK_AGENT_LIST_EMPTY, None);
    let backend = stub.backend();

    let result = backend.start(&WorkerSpec::new("brand-new-friendly-name", "task"));
    assert!(
        result.is_err(),
        "a name herdr has never seen, with no pane id to start an agent into, must refuse \
         rather than silently no-op or invent a spawn"
    );

    let invocations = stub.invocations();
    assert_eq!(
        invocations.len(),
        1,
        "must not attempt `agent start` when it cannot supply a pane; got {invocations:?}"
    );
    assert!(
        !invocations.iter().any(|line| line.starts_with("agent start")),
        "got {invocations:?}"
    );
}

#[test]
fn start_spawns_a_fresh_agent_using_the_constructed_kind_never_a_hardcoded_literal() {
    let stub = Stub::new();
    stub.queue_response("agent list", 0, OK_AGENT_LIST_EMPTY, None); // not yet running
    stub.queue_response(
        "agent start",
        0,
        r#"{"type":"ok","result":{"agent":{"pane_id":"w4:pB","name":"w4-pB","agent_status":"idle"}}}"#,
        None,
    );
    // A kind other than the module's old hardcoded "claude" literal —
    // proves the value reaching `--kind` is HerdrBackend's own
    // construction parameter (D14), not a compiled-in constant.
    let backend = stub.backend_with_kind("codex");

    backend.start(&WorkerSpec::new("w4:pB", "task")).unwrap();

    let invocations = stub.invocations();
    assert_eq!(invocations.len(), 2, "got {invocations:?}");
    assert!(invocations[0].starts_with("agent list"), "got {:?}", invocations[0]);
    assert!(
        invocations[1].starts_with("agent start w4-pB --kind codex --pane w4:pB"),
        "the constructed kind must reach agent start's argv, never a hardcoded literal; got {:?}",
        invocations[1]
    );
}
