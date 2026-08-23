//! The tmux implementation of `WorkerBackend` — a peer of
//! `backend::herdr::HerdrBackend`, selected by bee's `herding.transport`
//! key at the one wave construction site (tmux-herding-cockpit D1/D4).
//! Shells out to the real `tmux` CLI with `std::process::Command`,
//! argv-based, stdin always null. No shell string is ever built for
//! `tmux` itself.
//!
//! **Three properties separate this backend from the herdr one**, and each
//! shapes a method below (the same three
//! `crates/bee/src/herding/tmux.rs` records for the run-verb transport):
//!
//!   1. tmux has NO agent API. There is no `agent list`, no
//!      `agent_status`. The only signal is the pane's own screen text, so
//!      `status` is `crate::screen::classify` over a bounded
//!      `capture-pane` read. It is advisory (D4) — the choreography's own
//!      baseline + marker comparison stays the completion proof, exactly
//!      as it is on herdr, and `Screen` has no `Done` state to tempt a
//!      caller.
//!
//!   2. tmux has no "send this prompt" verb either. Text reaches a pane by
//!      being TYPED into whatever shell or TUI is there, which makes every
//!      send a TWO-call gesture: `send-keys -t <pane> -l <text>` for the
//!      literal bytes, then a separate `send-keys -t <pane> Enter` to
//!      submit. `-l` means "these bytes literally", so the newline cannot
//!      ride along in the same call — tmux would type the characters
//!      `E`,`n`,`t`,`e`,`r`. Never `-p`/`--print`.
//!
//!   3. A pane showing a trust / permission / auth dialog is BLOCKED
//!      (tmux-herding-transport D3): `send` preflights the screen and
//!      refuses with an `Err` rather than typing anything. A key sent into
//!      a dialog answers it on the human's behalf, which is exactly the
//!      failure D3 exists to prevent.
//!
//! **Worker names.** `WorkerSpec::name`, addressed through this backend,
//! is a tmux pane id (`%N`) or a pane TITLE. `canonical_id` resolves the
//! second to the first through `display-message`; anything it cannot
//! resolve comes back unchanged — the same safe no-collapse default
//! `HerdrBackend` takes (herding-orchestration D15).
//!
//! **Scope boundary — what `start` does NOT do.** Like the herdr backend,
//! this one never SPLITS a pane: `WorkerSpec` carries only a name and a
//! task, not the worktree cwd a split needs. The caller hands over a pane
//! that already exists. `start` types the agent's command line into it;
//! it never runs `new-session`, `attach-session` or `switch-client` —
//! the first would put the worker where the human is not looking, and the
//! last two need a TTY a tool shell does not have.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::{WorkerBackend, WorkerStatus};
use crate::screen::{classify, Screen, ScreenSettings};
use crate::wave::WorkerSpec;

/// The tmux worker backend.
///
/// Every field is caller-resolved at construction, the same
/// crate-boundary rule `HerdrBackend` follows (D2): `fleet` never reads
/// bee's configuration, so `agent_kind`, `agent_args` and `settings` all
/// arrive already decided by the bee-side `bee herding wave` verb.
#[derive(Debug)]
pub struct TmuxBackend {
    /// Directories searched AHEAD of the inherited `PATH` for the `tmux`
    /// executable. Empty in production (`new`), so the real installed
    /// `tmux` resolves however the host's own `PATH` says it should; this
    /// field exists for the PATH-prepended stub binary the tests use —
    /// the same seam `HerdrBackend` carries, and the reason `child_path`
    /// PREPENDS rather than replaces (a stub `tmux` written in `sh` still
    /// needs coreutils off the inherited `PATH`).
    path_prepend: Vec<PathBuf>,
    /// Where `send`/`read_output` spill a reply too long for the pane's
    /// own scrollback window. The OS temp directory in production; tests
    /// point this at a scratch directory they control.
    spill_dir: PathBuf,
    /// The agent executable typed as token 0 of the `exec` line `start`
    /// sends. Caller-resolved from `herding.agent_command` token 0
    /// (herding-orchestration D14) — never derived here, never validated
    /// against an allow-list of this crate's own.
    agent_kind: String,
    /// The remaining `herding.agent_command` tokens, typed after
    /// `agent_kind` on the same `exec` line, each shell-quoted.
    agent_args: Vec<String>,
    /// The screen-reading knobs `status` and `send`'s D3 preflight
    /// classify with. Caller-resolved from `herding.tmux.*` on the bee
    /// side (`TmuxSettings::from_config`).
    settings: ScreenSettings,
}

impl TmuxBackend {
    /// The production backend: `tmux` resolves off the host's own `PATH`,
    /// long replies spill to the OS temp directory.
    ///
    /// All three parameters are CALLER-RESOLVED, never derived here —
    /// see the struct's own field docs and D2.
    pub fn new(
        agent_kind: impl Into<String>,
        agent_args: Vec<String>,
        settings: ScreenSettings,
    ) -> Self {
        Self {
            path_prepend: Vec::new(),
            spill_dir: std::env::temp_dir(),
            agent_kind: agent_kind.into(),
            agent_args,
            settings,
        }
    }

    /// A backend wired for testing, mirroring
    /// `HerdrBackend::with_test_seams`: `dirs` is searched ahead of the
    /// inherited `PATH` (the PATH-prepended stub binary seam) and long
    /// replies spill into `spill_dir` instead of the shared OS temp
    /// directory, so parallel tests never collide. This crate's whole
    /// test suite runs with no real `tmux` installed.
    pub fn with_test_seams(
        dirs: Vec<PathBuf>,
        spill_dir: PathBuf,
        agent_kind: impl Into<String>,
        agent_args: Vec<String>,
        settings: ScreenSettings,
    ) -> Self {
        Self {
            path_prepend: dirs,
            spill_dir,
            agent_kind: agent_kind.into(),
            agent_args,
            settings,
        }
    }

    /// The child `PATH` a spawned `tmux` command should see: every
    /// `path_prepend` directory, in order, AHEAD of the inherited `PATH`
    /// — prepend, never replace. Replacing would break a stub written as
    /// a `sh` script, which still has to find `cat`, `printf` and friends
    /// on the inherited path. `None` when there is nothing to prepend, so
    /// production spawns leave `PATH` exactly as the process inherited
    /// it. (Copied from `super::herdr::HerdrBackend::child_path`, which
    /// is private to that module; the two are deliberately identical.)
    fn child_path(&self) -> Option<OsString> {
        if self.path_prepend.is_empty() {
            return None;
        }
        let sep = if cfg!(windows) { ';' } else { ':' };
        let mut out = OsString::new();
        for (index, dir) in self.path_prepend.iter().enumerate() {
            if index > 0 {
                out.push(sep.to_string());
            }
            out.push(dir);
        }
        if let Some(inherited) = std::env::var_os("PATH") {
            out.push(sep.to_string());
            out.push(inherited);
        }
        Some(out)
    }

    /// Runs one `tmux` invocation and returns its trimmed stdout.
    ///
    /// The ONLY function in this module that spawns a process, so there
    /// is exactly one place that decides what "trouble" means: a `tmux`
    /// that will not spawn (not installed, not on `PATH`) and a non-zero
    /// exit are both `Err`, and the message names the full argv. Naming
    /// the argv is what makes a live failure diagnosable — tmux's own
    /// stderr is often a bare "can't find pane", which says nothing about
    /// which call produced it. stdin is null so a tmux that unexpectedly
    /// prompts can never block this call.
    fn call(&self, args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new("tmux");
        cmd.args(args).stdin(Stdio::null());
        if let Some(path) = self.child_path() {
            cmd.env("PATH", path);
        }
        let out = cmd
            .output()
            .map_err(|e| format!("could not spawn tmux {}: {e}", args.join(" ")))?;
        if !out.status.success() {
            let code = out.status.code().map_or_else(|| "unknown".to_string(), |c| c.to_string());
            return Err(format!(
                "tmux {} exited {code}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    }

    /// One bounded screen read of `pane`, `settings.scrollback` lines
    /// deep.
    fn capture(&self, pane: &str) -> Result<String, String> {
        let start = format!("-{}", self.settings.scrollback);
        self.call(&["capture-pane", "-p", "-t", pane, "-S", &start])
    }

    /// Types one literal line into a pane and submits it, as the two
    /// separate `send-keys` calls tmux requires (module docs, property 2).
    fn send_line(&self, pane: &str, line: &str) -> Result<(), String> {
        self.call(&["send-keys", "-t", pane, "-l", line])?;
        self.call(&["send-keys", "-t", pane, "Enter"])?;
        Ok(())
    }

    /// Where a reply too long for the pane's scrollback window is spilled
    /// for `worker`. Deterministic per worker so `send`'s spill
    /// instruction and `read_output`'s recovery check agree on the same
    /// path with no side channel between them — the same contract
    /// `HerdrBackend::spill_path` holds, with `%` folded away too because
    /// a tmux pane id starts with one.
    fn spill_path(&self, worker: &str) -> PathBuf {
        let safe: String = worker
            .chars()
            .map(|c| if c == '%' || c == ':' || c == '/' || c == '\\' { '_' } else { c })
            .collect();
        self.spill_dir.join(format!("bee-fleet-tmux-{safe}.reply"))
    }

    /// The `exec` line `start` types, built from the caller-resolved
    /// kind and args. Pure, so the quoting is provable without a process
    /// on every platform including Windows.
    fn start_line(&self) -> String {
        let mut line = String::from("exec ");
        line.push_str(&shell_quote(&self.agent_kind));
        for arg in &self.agent_args {
            line.push(' ');
            line.push_str(&shell_quote(arg));
        }
        line
    }
}

/// True for tmux's own canonical pane-id form: `%` followed by at least
/// one digit and nothing else. This is the shape `canonical_id` short
/// circuits on, the tmux twin of `HerdrBackend`'s `is_pane_id_shaped`.
fn is_pane_id_shaped(id: &str) -> bool {
    let Some(rest) = id.strip_prefix('%') else {
        return false;
    };
    !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
}

/// Wraps one token in POSIX single quotes so a shell reads it as exactly
/// one literal word.
///
/// This is not decoration. `start` does not spawn the agent — it TYPES a
/// command line into whatever shell the pane is running, so an argument
/// carrying `$HOME`, a backtick or a space becomes shell syntax unless it
/// is quoted here. Single quotes suppress every expansion a POSIX shell
/// performs; the only character they cannot carry is a single quote
/// itself, which is spelled by closing the quote, escaping one, and
/// reopening: `'` → `'\''`.
fn shell_quote(token: &str) -> String {
    format!("'{}'", token.replace('\'', "'\\''"))
}

/// The literal text `send` types into the pane: the task, plus the
/// spill-file fallback instruction `read_output` knows how to recover.
///
/// COPIED from `super::herdr::wrap_task_with_spill_instruction`, whose
/// full reasoning lives there — it is private to that module and this
/// cell's file scope does not include it, so the text is duplicated with
/// this pointer rather than widened across a file this worker does not
/// own. Keep the two in step: `read_output` here and there both recognize
/// the same "reply with EXACTLY that file path" hand-back.
fn wrap_task_with_spill_instruction(task: &str, spill: &Path) -> String {
    format!(
        "{task}\n\n(If this reply would be longer than a typical terminal screen, instead \
         write your FULL reply to the file {spill} and reply here with EXACTLY that file \
         path on its own line and nothing else.)",
        spill = spill.display()
    )
}

/// Recovers a reply spilled to a file when `transcript`'s last non-blank
/// line hands back `spill`'s own path.
///
/// COPIED from `super::herdr::recover_transcript` for the same
/// file-scope reason as `wrap_task_with_spill_instruction`; the failure
/// it mitigates is even sharper here, because a `capture-pane` read is
/// bounded by `settings.scrollback` lines and cannot see a row that has
/// already left the pane's history. A SUBSTRING check, not equality: a
/// TUI prefixes a reply line with its own marker (`"● <text>"`), so the
/// agent's bare-path reply arrives as `"● <path>"`, never the path alone.
///
/// Reads the real filesystem but spawns no process.
fn recover_transcript(transcript: String, spill: &Path) -> String {
    let spill_str = spill.to_string_lossy();
    let last_line = transcript
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("")
        .trim();
    if last_line.contains(spill_str.as_ref()) {
        if let Ok(spilled) = std::fs::read_to_string(spill) {
            return spilled;
        }
    }
    transcript
}

/// Maps one classified screen onto the trait's status vocabulary. Pure,
/// and deliberately total: `Screen` has three states and `WorkerStatus`
/// has five, so the two states this can NEVER produce are named here
/// rather than left implicit — `Finished` (D4: a screen can never prove
/// completion) and `Unverifiable`, which only a failed READ produces,
/// never a successful one.
fn map_screen(screen: Screen) -> WorkerStatus {
    match screen {
        Screen::Idle => WorkerStatus::Ready,
        Screen::Working => WorkerStatus::Working,
        Screen::Blocked => WorkerStatus::Blocked,
    }
}

impl WorkerBackend for TmuxBackend {
    /// A `%N`-shaped identifier is already tmux's canonical addressing
    /// form and short circuits with NO tmux call at all. Anything else is
    /// treated as a pane TITLE and resolved through
    /// `display-message -p -t <name> '#{pane_id}'`. A lookup that fails,
    /// or one that prints nothing, returns `name` unchanged — the safe
    /// no-collapse default the trait requires (herding-orchestration
    /// D15), never a guess.
    ///
    /// The format string is a bare argv token. The single quotes seen in
    /// tmux documentation are SHELL syntax; there is no shell here, so
    /// quoting them would make them part of the format.
    fn canonical_id(&self, name: &str) -> String {
        if is_pane_id_shaped(name) {
            return name.to_string();
        }
        match self.call(&["display-message", "-p", "-t", name, "#{pane_id}"]) {
            Ok(id) if !id.trim().is_empty() => id.trim().to_string(),
            _ => name.to_string(),
        }
    }

    /// Types the agent's `exec` line into the worker's pane.
    ///
    /// `exec` replaces the pane's shell with the agent process instead of
    /// leaving the agent as its child. That is what makes the pane's own
    /// pid the AGENT's, and what makes the pane close when the agent
    /// exits rather than dropping back to a prompt that reads as idle.
    ///
    /// The task is NOT an argv token here — it travels through `send`
    /// after the start, exactly as it does on the herdr backend
    /// (herding-run-prompt-delivery D1).
    fn start(&self, worker: &WorkerSpec) -> anyhow::Result<()> {
        let pane = self.canonical_id(&worker.name);
        let line = self.start_line();
        self.send_line(&pane, &line)
            .map_err(|e| anyhow::anyhow!("tmux start {}: {e}", worker.name))
    }

    /// One bounded screen read, classified (D4). NEVER an `Err`: an
    /// unresolvable pane, a `tmux` that will not spawn and a failed
    /// capture all arrive as `Unverifiable`, which is fail-closed status
    /// as a property of the return type rather than of the caller's
    /// discipline (D7, Ordering Invariant 4).
    fn status(&self, worker: &str) -> WorkerStatus {
        let pane = self.canonical_id(worker);
        match self.capture(&pane) {
            Ok(screen) => map_screen(classify(&screen, &self.settings)),
            Err(_) => WorkerStatus::Unverifiable,
        }
    }

    /// Types `task` into the worker's pane and submits it.
    ///
    /// D3 preflight FIRST: if the pane's current screen classifies
    /// `Blocked`, this refuses with an `Err` and types NOTHING. Sending
    /// into a dialog would answer it on the human's behalf, and the task
    /// text's first character is as likely to be "y" as anything else.
    /// A capture that FAILS is not a blocked screen — it reads as an
    /// empty screen and the send proceeds, the same fail-open posture
    /// `crates/bee/src/herding/tmux.rs`'s `agent_prompt` takes; a pane
    /// that truly cannot be written to fails on the `send-keys` call
    /// itself, one line later.
    fn send(&self, worker: &str, task: &str) -> anyhow::Result<()> {
        let pane = self.canonical_id(worker);
        let baseline = self.capture(&pane).unwrap_or_default();
        if classify(&baseline, &self.settings) == Screen::Blocked {
            return Err(anyhow::anyhow!(
                "tmux send {worker}: pane {pane} is showing a dialog (blocked) — nothing was \
                 typed; a human answers it"
            ));
        }
        let spill = self.spill_path(worker);
        let wrapped = wrap_task_with_spill_instruction(task, &spill);
        self.send_line(&pane, &wrapped)
            .map_err(|e| anyhow::anyhow!("tmux send {worker} failed: {e}"))
    }

    /// The pane's current screen, with a spilled reply recovered when the
    /// transcript's last line hands back the spill path.
    fn read_output(&self, worker: &str) -> anyhow::Result<String> {
        let pane = self.canonical_id(worker);
        let transcript = self
            .capture(&pane)
            .map_err(|e| anyhow::anyhow!("tmux read_output {worker} failed: {e}"))?;
        let spill = self.spill_path(worker);
        Ok(recover_transcript(transcript, &spill))
    }
}

// ═══════════════════════════════════════════════════════════════════════
// pure unit tests — no process, so they run on every platform including
// Windows, where the stub-script integration tests in
// `tests/tmux_backend.rs` are skipped (the same recorded gap
// `tests/herdr_backend.rs` carries).
// ═══════════════════════════════════════════════════════════════════════
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pane_id_shape_accepts_only_percent_plus_digits() {
        assert!(is_pane_id_shaped("%0"));
        assert!(is_pane_id_shaped("%42"));
        assert!(!is_pane_id_shaped("%"));
        assert!(!is_pane_id_shaped("%3a"));
        assert!(!is_pane_id_shaped("dispatch"));
        assert!(!is_pane_id_shaped("w4:pB"), "herdr's own pane shape is not tmux's");
    }

    #[test]
    fn shell_quote_neutralizes_a_single_quote_and_a_dollar_sign() {
        // The line is TYPED into a shell, so `$HOME` must survive as five
        // literal characters and the apostrophe must not end the quoting.
        assert_eq!(shell_quote("it's $HOME"), r#"'it'\''s $HOME'"#);
        assert_eq!(shell_quote("two words"), "'two words'");
    }

    #[test]
    fn start_line_quotes_every_token_it_was_constructed_with() {
        let backend = TmuxBackend::new(
            "claude",
            vec!["--dangerously".to_string(), "a b".to_string()],
            ScreenSettings::default(),
        );
        assert_eq!(backend.start_line(), "exec 'claude' '--dangerously' 'a b'");
    }

    #[test]
    fn map_screen_never_reports_finished_or_unverifiable() {
        // D4: a screen cannot prove completion, so no successful read may
        // ever produce `Finished`; `Unverifiable` belongs to a FAILED read
        // alone. Mutating `map_screen` to hand back either fails here.
        for screen in [Screen::Idle, Screen::Working, Screen::Blocked] {
            let status = map_screen(screen);
            assert_ne!(status, WorkerStatus::Finished, "{screen:?}");
            assert_ne!(status, WorkerStatus::Unverifiable, "{screen:?}");
        }
        assert_eq!(map_screen(Screen::Idle), WorkerStatus::Ready);
        assert_eq!(map_screen(Screen::Working), WorkerStatus::Working);
        assert_eq!(map_screen(Screen::Blocked), WorkerStatus::Blocked);
    }

    #[test]
    fn spill_path_folds_the_pane_id_percent_into_a_filename_safe_character() {
        let backend = TmuxBackend::with_test_seams(
            Vec::new(),
            PathBuf::from("/tmp/scratch"),
            "claude",
            Vec::new(),
            ScreenSettings::default(),
        );
        assert_eq!(
            backend.spill_path("%3"),
            PathBuf::from("/tmp/scratch").join("bee-fleet-tmux-_3.reply")
        );
    }

    #[test]
    fn wrap_task_with_spill_instruction_names_the_spill_path() {
        let spill = PathBuf::from("/tmp/scratch/bee-fleet-tmux-_3.reply");
        let wrapped = wrap_task_with_spill_instruction("do the thing", &spill);
        assert!(wrapped.starts_with("do the thing"), "{wrapped}");
        assert!(wrapped.contains("/tmp/scratch/bee-fleet-tmux-_3.reply"), "{wrapped}");
    }

    #[test]
    fn recover_transcript_returns_the_transcript_unchanged_when_nothing_was_spilled() {
        let transcript = "❯ hi\n\n● DONE.\n".to_string();
        let spill = PathBuf::from("/tmp/scratch/never-written.reply");
        assert_eq!(recover_transcript(transcript.clone(), &spill), transcript);
    }
}
