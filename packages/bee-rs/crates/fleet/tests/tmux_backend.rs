//! Drives `TmuxBackend` against a PATH-prepended stub `tmux` binary —
//! the same shape `tests/herdr_backend.rs` uses for `herdr`, so this
//! crate's whole test suite runs green with NO real tmux installed and no
//! tmux server running. Every test builds its own throwaway stub
//! directory.
//!
//! Unix-only (`#![cfg(unix)]`, gating the whole file): the stub is a POSIX
//! shell script, and this crate has no Windows-native stub-binary
//! equivalent yet — the same named, recorded gap `herdr_backend.rs`
//! carries. The pure halves (pane-id shape, quoting, the `Screen` →
//! `WorkerStatus` map, the spill helpers) are covered by
//! `src/backend/tmux.rs`'s own `#[cfg(test)] mod tests`, which have no
//! process dependency and run everywhere.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use fleet::backend::tmux::TmuxBackend;
use fleet::backend::{WorkerBackend, WorkerStatus};
use fleet::screen::ScreenSettings;
use fleet::wave::WorkerSpec;

static STUB_SEQ: AtomicU64 = AtomicU64::new(0);

/// A settled shell prompt: no chrome, no dialog.
const IDLE_SCREEN: &str = "$ ls\nCargo.toml  src  tests\n$\n";
/// An agent mid-turn — the busy footer is the last painted row.
const WORKING_SCREEN: &str = "> summarize the plan\n\n* Thinking... (12s . esc to interrupt)\n";
/// A trust dialog: the human must answer it, so bee types nothing (D3).
const BLOCKED_SCREEN: &str =
    "* Thinking...\n\n+---------------------------+\n| Do you trust the files?   |\n+---------------------------+\n";

/// A throwaway PATH-prepended stub `tmux`. `dir` holds the executable
/// script itself (what gets prepended to the child's `PATH`); `state`
/// holds the response queues `queue_response` writes into; `log` records
/// every invocation's argv, one record per call, for tests that assert on
/// exactly what was passed (for example: `-l` then a separate `Enter`,
/// and never `-p`/`--print` on a send).
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
        let root =
            std::env::temp_dir().join(format!("fleet-tmux-stub-{}-{seq}", std::process::id()));
        let dir = root.join("bin");
        let state = root.join("state");
        let spill = root.join("spill");
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir_all(&state).unwrap();
        fs::create_dir_all(&spill).unwrap();
        let log = root.join("invocations.log");

        let script_path = dir.join("tmux");
        // `Q` is keyed on `$1` alone — tmux verbs are single words
        // (`capture-pane`, `send-keys`, `display-message`) unlike herdr's
        // two-word pairs — so the stub never has to be kept in sync with
        // a hardcoded per-verb list.
        //
        // The log record is NOT a plain `"$*"` + newline: a real argument
        // here (the wrapped task text `send` builds) contains embedded
        // newlines of its own, which would fragment one invocation into
        // several log "lines". Each arg is written with a trailing ASCII
        // Unit Separator (`\037`) and each whole invocation with a
        // trailing Record Separator (`\036`) — control characters no real
        // argument here uses — so `Stub::invocations` splits
        // unambiguously.
        //
        // The script needs `cat` and `printf` off the INHERITED `PATH`:
        // that is why `TmuxBackend::child_path` prepends rather than
        // replaces, and this stub is what would break first if it ever
        // stopped doing so.
        let script = format!(
            "#!/bin/sh\n\
             for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> {log}\n\
             printf '\\036' >> {log}\n\
             if [ -z \"$1\" ]; then\n\
             printf '%s' 'stub: no verb' 1>&2\n\
             exit 1\n\
             fi\n\
             Q={state}/\"$1\"\n\
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

    /// Queues `body` as the `index`-th stdout `tmux <verb> …` returns
    /// (0-based; calls beyond the highest queued index keep returning the
    /// LAST one). `exit_code` simulates a non-zero exit when `Some`.
    fn queue_response(&self, verb: &str, index: u32, body: &str, exit_code: Option<i32>) {
        let base = self.state.join(verb);
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

    fn backend(&self) -> TmuxBackend {
        self.backend_with(("claude", Vec::new()), ScreenSettings::default())
    }

    fn backend_with(
        &self,
        (kind, args): (&str, Vec<String>),
        settings: ScreenSettings,
    ) -> TmuxBackend {
        TmuxBackend::with_test_seams(
            vec![self.dir.clone()],
            self.spill.clone(),
            kind,
            args,
            settings,
        )
    }

    /// Every invocation as a `Vec` of its raw args, in call order. Uses
    /// the Unit/Record Separator characters `Stub::new`'s script wrote —
    /// never a plain newline split, which a multi-line argument (like
    /// `send`'s wrapped task text) would fragment.
    fn invocations(&self) -> Vec<Vec<String>> {
        let raw = fs::read_to_string(&self.log).unwrap_or_default();
        raw.split('\u{1e}')
            .filter(|record| !record.is_empty())
            .map(|record| {
                record.split('\u{1f}').filter(|arg| !arg.is_empty()).map(str::to_string).collect()
            })
            .collect()
    }
}

/// Wraps `path` in single quotes for embedding in the generated `sh`
/// script — every path here is a scratch temp-dir path this test itself
/// created, never external input.
fn shell_quote(path: &std::path::Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

// ── canonical_id ────────────────────────────────────────────────────────

#[test]
fn canonical_id_passes_a_pane_id_through_without_calling_tmux_at_all() {
    // Proven by never queuing a response: the stub logs every call it
    // receives, so an empty log is proof no lookup happened.
    let stub = Stub::new();
    let backend = stub.backend();

    assert_eq!(backend.canonical_id("%3"), "%3");
    assert!(
        stub.invocations().is_empty(),
        "resolving a pane-id-shaped identifier must not call tmux at all"
    );
}

#[test]
fn canonical_id_resolves_a_pane_title_through_display_message() {
    let stub = Stub::new();
    stub.queue_response("display-message", 0, "%7\n", None);
    let backend = stub.backend();

    assert_eq!(backend.canonical_id("dispatch"), "%7");

    let calls = stub.invocations();
    assert_eq!(calls.len(), 1, "got {calls:?}");
    assert_eq!(
        calls[0],
        vec!["display-message", "-p", "-t", "dispatch", "#{pane_id}"],
        "the format string is a bare argv token — quoting it would make the quotes part of \
         the format"
    );
}

#[test]
fn canonical_id_falls_back_to_the_name_unchanged_when_the_lookup_fails() {
    let stub = Stub::new();
    stub.queue_response("display-message", 0, "can't find pane", Some(1));
    let backend = stub.backend();

    assert_eq!(
        backend.canonical_id("ghost-pane"),
        "ghost-pane",
        "a failed lookup must fall back to the identifier unchanged — the safe no-collapse \
         default (herding-orchestration D15), never a guess"
    );
}

#[test]
fn canonical_id_falls_back_to_the_name_when_display_message_prints_nothing() {
    let stub = Stub::new();
    stub.queue_response("display-message", 0, "\n", None);
    let backend = stub.backend();

    assert_eq!(backend.canonical_id("ghost-pane"), "ghost-pane");
}

// ── start: the two-call typing gesture ──────────────────────────────────

#[test]
fn start_types_the_exec_line_literally_then_a_separate_enter() {
    let stub = Stub::new();
    let backend = stub.backend_with(
        ("codex", vec!["--flag".to_string(), "a b".to_string()]),
        ScreenSettings::default(),
    );

    backend.start(&WorkerSpec::new("%3", "the task")).unwrap();

    let calls = stub.invocations();
    assert_eq!(calls.len(), 2, "a send is TWO tmux calls, never one; got {calls:?}");
    assert_eq!(
        calls[0],
        vec!["send-keys", "-t", "%3", "-l", "exec 'codex' '--flag' 'a b'"],
        "the constructed kind and args must reach the typed line, each shell-quoted, and the \
         line must be typed with -l (literal bytes)"
    );
    assert_eq!(
        calls[1],
        vec!["send-keys", "-t", "%3", "Enter"],
        "the submit is always a second invocation with the key NAME Enter — -l cannot carry it"
    );
    assert!(
        !calls.iter().flatten().any(|arg| arg == "-p" || arg == "--print"),
        "a send must never pass -p/--print; got {calls:?}"
    );
    assert!(
        !calls.iter().flatten().any(|arg| arg == "new-session" || arg == "attach-session"),
        "a worker pane is never a new session and is never attached to; got {calls:?}"
    );
    assert!(
        !calls[0].contains(&"the task".to_string()),
        "the task is never an argv token on start — it travels through send \
         (herding-run-prompt-delivery D1); got {:?}",
        calls[0]
    );
}

// ── status: the fail-closed law (D7, Ordering Invariant 4) ──────────────

#[test]
fn status_maps_each_classified_screen_onto_the_trait_vocabulary() {
    for (screen, expected) in [
        (IDLE_SCREEN, WorkerStatus::Ready),
        (WORKING_SCREEN, WorkerStatus::Working),
        (BLOCKED_SCREEN, WorkerStatus::Blocked),
    ] {
        let stub = Stub::new();
        stub.queue_response("capture-pane", 0, screen, None);
        let backend = stub.backend();

        assert_eq!(backend.status("%3"), expected, "screen {screen:?}");

        let calls = stub.invocations();
        assert_eq!(calls.len(), 1, "status is exactly one bounded read; got {calls:?}");
        assert_eq!(calls[0], vec!["capture-pane", "-p", "-t", "%3", "-S", "-40"]);
    }
}

#[test]
fn status_reads_the_scrollback_depth_off_its_constructed_settings() {
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, IDLE_SCREEN, None);
    let settings = ScreenSettings { scrollback: 200, ..ScreenSettings::default() };
    let backend = stub.backend_with(("claude", Vec::new()), settings);

    assert_eq!(backend.status("%3"), WorkerStatus::Ready);
    assert_eq!(
        stub.invocations()[0],
        vec!["capture-pane", "-p", "-t", "%3", "-S", "-200"],
        "the capture depth must come from the caller-resolved settings, not a constant"
    );
}

#[test]
fn status_is_unverifiable_when_the_capture_itself_fails() {
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, "can't find pane %9", Some(1));
    let backend = stub.backend();

    assert_eq!(
        backend.status("%9"),
        WorkerStatus::Unverifiable,
        "a failed read is never an Err and never a guess — fail-closed status is a property \
         of the return type (D7, Ordering Invariant 4)"
    );
}

#[test]
fn status_of_a_settings_override_that_blanks_the_markers_is_still_never_an_err() {
    // An empty marker list classifies everything as Idle. The point here
    // is the shape, not the value: no configuration of the settings can
    // make `status` return an error, because its signature has none.
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, BLOCKED_SCREEN, None);
    let settings = ScreenSettings {
        busy_markers: Vec::new(),
        blocked_markers: Vec::new(),
        ..ScreenSettings::default()
    };
    let backend = stub.backend_with(("claude", Vec::new()), settings);

    assert_eq!(backend.status("%3"), WorkerStatus::Ready);
}

// ── send: D3, never type into a dialog ──────────────────────────────────

#[test]
fn send_refuses_a_blocked_screen_and_types_nothing() {
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, BLOCKED_SCREEN, None);
    let backend = stub.backend();

    let err = backend.send("%3", "please reply with PONG").unwrap_err();
    assert!(
        err.to_string().contains("blocked"),
        "the refusal must name why it refused; got {err}"
    );

    let calls = stub.invocations();
    assert_eq!(
        calls.len(),
        1,
        "the preflight capture is the ONLY call a blocked send makes — a key typed into a \
         dialog answers it on the human's behalf (tmux-herding-transport D3); got {calls:?}"
    );
    assert_eq!(calls[0][0], "capture-pane");
    assert!(
        !calls.iter().any(|c| c[0] == "send-keys"),
        "nothing may be typed into a blocked pane; got {calls:?}"
    );
}

#[test]
fn send_types_the_task_with_its_spill_instruction_then_a_separate_enter() {
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, IDLE_SCREEN, None);
    let backend = stub.backend();

    backend.send("%3", "please reply with PONG").unwrap();

    let calls = stub.invocations();
    assert_eq!(calls.len(), 3, "preflight capture plus the two-call send; got {calls:?}");
    assert_eq!(calls[0][0], "capture-pane");
    assert_eq!(calls[1][0..4], ["send-keys", "-t", "%3", "-l"]);
    let typed = &calls[1][4];
    assert!(typed.starts_with("please reply with PONG"), "{typed}");
    assert!(
        typed.contains("bee-fleet-tmux-_3.reply"),
        "the task must carry the spill-file fallback instruction read_output recovers from; \
         got {typed}"
    );
    assert_eq!(calls[2], vec!["send-keys", "-t", "%3", "Enter"]);
}

// ── read_output: the bounded-window recovery ────────────────────────────

#[test]
fn read_output_recovers_a_reply_spilled_to_a_file_when_longer_than_the_capture_window() {
    let stub = Stub::new();
    let backend = stub.backend();

    // `send`'s own spill path for "%3" — read_output must agree with it
    // exactly, since nothing else tells the two methods about each other.
    let spill_path = stub.spill.join("bee-fleet-tmux-_3.reply");
    let long_reply = "X".repeat(50_000); // far longer than any capture-pane window
    fs::write(&spill_path, &long_reply).unwrap();

    // The screen tmux hands back shows only the bare spill path on the
    // agent's own reply row, prefixed by the TUI's reply marker.
    let screen = format!("> please reply\n\n* {}\n", spill_path.display());
    stub.queue_response("capture-pane", 0, &screen, None);

    assert_eq!(
        backend.read_output("%3").unwrap(),
        long_reply,
        "a reply too long for the capture window must still be recovered in full via the \
         spill file, not truncated to the bare path the screen showed"
    );
}

#[test]
fn read_output_returns_the_captured_screen_unchanged_when_nothing_was_spilled() {
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, "> hi\n\n* DONE.", None);
    let backend = stub.backend();

    assert_eq!(backend.read_output("%3").unwrap(), "> hi\n\n* DONE.");
}

#[test]
fn read_output_is_an_err_when_the_capture_fails() {
    // Unlike `status`, `read_output` DOES return an Err — the trait says
    // so, and the choreography's baseline capture needs to know it never
    // got one rather than treating an empty string as the pane's content.
    let stub = Stub::new();
    stub.queue_response("capture-pane", 0, "can't find pane", Some(1));
    let backend = stub.backend();

    assert!(backend.read_output("%9").is_err());
}
