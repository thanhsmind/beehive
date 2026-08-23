// herding/tmux.rs — the tmux half of `bee herding run`'s transport seam
// (tmux-herding-transport D2/D3/D4/D5).
//
// `RealTmux` implements the same `PaneTransport` trait `RealHerdr` does
// (run.rs), so the run verb, the mailbox, the split lock and every safety
// boundary stay exactly as they are; only the process bee shells out to
// changes. Selection happens at ONE construction site and is driven by the
// `herding.transport` config key (D1) — never by sniffing `$TMUX`.
//
// Three properties separate this transport from the herdr one, and each is
// load-bearing:
//
//   1. tmux has NO agent API. There is no `agent list`, no `agent_status`,
//      no lifecycle sequence number. The only signal is the pane's own
//      screen text. So status here is a CLASSIFIER over a bounded
//      `capture-pane` read: content stability plus two marker lists held as
//      config data with upstream defaults (D4). It is advisory only —
//      `result-N.json` and `ack-N.json` stay the sole truth for done and
//      delivered, and `classify` deliberately has no `Done` variant so this
//      file can never be mistaken for that truth.
//
//   2. tmux has no "send this prompt" verb either. Text reaches a pane by
//      being TYPED into whatever shell or TUI is there. That makes every
//      send a two-call gesture — `send-keys -l <text>` then a separate
//      `send-keys Enter` — because a single call cannot express "these
//      bytes literally, then a submit". It also makes `agent_start` a
//      shell line, so every token is single-quoted before it is typed.
//
//   3. A pane showing a trust / permission / auth dialog is `blocked`
//      (D3): the wait ends, the pane stays open, and bee types NOTHING.
//      A key sent into a dialog answers it on the human's behalf, which is
//      exactly the failure D3 exists to prevent. `agent_wait` returns
//      `blocked` the moment a blocked marker shows, and `agent_prompt`
//      preflights the pane and refuses rather than typing into a dialog.
//
// Panes always live in the CALLER's current tmux window (D2) — this file
// never runs `new-session`, `attach-session` or `switch-client`. The first
// two would put the worker somewhere the human is not looking; the last two
// need a TTY a tool shell does not have.
//
// Nothing here is wired into production yet: the run verb's construction
// site is switched by tht-4. Until then the module is compiled and fully
// tested, but unreferenced — hence the allow below, which tht-4 removes.
#![allow(dead_code)]

use std::collections::HashMap;
use std::ffi::OsString;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use serde_json::Value;

// The ONE classifier (tmux-herding-cockpit D4). It used to live in this
// file; it now lives one crate down, in `fleet`, where this crate's own
// `TmuxBackend` reaches it too. Nothing about the semantics changed — the
// import is the change.
use fleet::screen::ScreenSettings;
// Re-exported at this module's own path, `pub(crate)`, so every sibling
// that already says `use super::tmux::{classify, Screen, …}` keeps
// reading exactly as it did — the classifier moved crates, the local
// vocabulary did not.
pub(crate) use fleet::screen::{classify, Screen};

use super::run::{Liveness, PaneGeom, PaneTransport};

// ═══════════════════════════════════════════════════════════════════════════
// settings and the classifier — ONE copy, held down in `fleet` (D4)
// ═══════════════════════════════════════════════════════════════════════════
//
// tmux-herding-cockpit D4: ONE screen classifier (markers plus stability
// knobs) serves both crates, and it lives in `fleet` because `bee` depends
// on `fleet` and never the other way round. `fleet::screen` therefore owns
// the marker lists, the two tail windows, `Screen` and `classify`; this
// file owns only the half that is bee's — reading the knobs out of
// `.bee/config.json`, which `fleet` must never do (herding-orchestration
// D2, the crate boundary).
//
// There is no second classifier body here, and adding one would be the
// drift D4 exists to prevent.

/// bee's wrapper around `fleet::screen::ScreenSettings`: the SAME data,
/// carried under the local name every call site in this crate already
/// uses, with the one method that is bee's alone — reading the knobs out
/// of `.bee/config.json`.
///
/// A newtype rather than a bare `type` alias for exactly that method:
/// `from_config` is an inherent function on a bee-owned type, so `fleet`
/// never grows a bee config key and no caller needs a trait in scope to
/// spell `TmuxSettings::from_config`. `Deref`/`DerefMut` to the wrapped
/// value keep every field read (`settings.scrollback`) and every
/// `classify(&screen, &settings)` call reading as they did before the
/// classifier moved crates.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct TmuxSettings(ScreenSettings);

impl std::ops::Deref for TmuxSettings {
    type Target = ScreenSettings;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for TmuxSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TmuxSettings {
    /// The wrapped `fleet` value, for a caller that must hand the shared
    /// type across the crate boundary — `fleet::backend::tmux::TmuxBackend::new`
    /// takes a `ScreenSettings`, because `fleet` may not name this
    /// wrapper any more than it may name bee's config keys.
    pub(crate) fn into_screen_settings(self) -> ScreenSettings {
        self.0
    }

    /// Reads `herding.tmux.{busy_markers,blocked_markers,scrollback,
    /// quiet_cycles,interval_ms}` out of an already-parsed
    /// `.bee/config.json`, defaulting every key that is absent or the
    /// wrong JSON type.
    ///
    /// A list override REPLACES the default list, it does not extend it.
    /// That is the deliberate choice: a repo correcting a rotted marker
    /// needs the stale one GONE, and an extend-only seam cannot express
    /// that. The cost is that an override must restate the markers it still
    /// wants — an explicit empty array is therefore a legal override
    /// meaning "never classify on this list".
    ///
    /// Fail-open, like every other reader of this file: a malformed value
    /// leaves the default in place rather than refusing the run. The one
    /// key that IS a typed refusal is `herding.transport` itself
    /// (`super::transport_kind`) — picking the wrong transport is not
    /// recoverable, reading a rotted marker list is.
    pub(crate) fn from_config(cfg: &Value) -> Self {
        let mut settings = Self::default();
        let Some(tmux) = cfg.get("herding").and_then(|h| h.get("tmux")) else {
            return settings;
        };
        if let Some(list) = string_list(tmux.get("busy_markers")) {
            settings.busy_markers = list;
        }
        if let Some(list) = string_list(tmux.get("blocked_markers")) {
            settings.blocked_markers = list;
        }
        if let Some(n) = tmux.get("scrollback").and_then(Value::as_u64) {
            settings.scrollback = n.clamp(1, u64::from(u32::MAX)) as u32;
        }
        if let Some(n) = tmux.get("quiet_cycles").and_then(Value::as_u64) {
            // At least one read has to happen before a screen can be called
            // stable; `0` would make every first read "settled".
            settings.quiet_cycles = n.clamp(1, u64::from(u32::MAX)) as u32;
        }
        if let Some(n) = tmux.get("interval_ms").and_then(Value::as_u64) {
            settings.interval_ms = n;
        }
        settings
    }

    /// The `-S` argument shared by every `capture-pane` read here.
    fn capture_start(&self) -> String {
        format!("-{}", self.scrollback)
    }
}

/// A JSON array of strings, or `None` for anything else (absent, a scalar,
/// an object). A non-string element is dropped rather than failing the
/// whole override.
fn string_list(v: Option<&Value>) -> Option<Vec<String>> {
    let arr = v?.as_array()?;
    Some(arr.iter().filter_map(Value::as_str).map(str::to_string).collect())
}

// ═══════════════════════════════════════════════════════════════════════════
// shell quoting — every token typed into a pane goes through here
// ═══════════════════════════════════════════════════════════════════════════

/// Wraps one token in POSIX single quotes so a shell reads it as exactly
/// one literal word.
///
/// This is not decoration. `agent_start` does not spawn a process — it
/// TYPES a command line into whatever shell the pane is running, so a task
/// slug carrying `$HOME`, a backtick or a space becomes shell syntax unless
/// it is quoted here. Single quotes suppress every expansion a POSIX shell
/// performs; the only character they cannot carry is a single quote itself,
/// which is spelled by closing the quote, escaping one, and reopening:
/// `'` → `'\''`.
fn shell_quote(token: &str) -> String {
    format!("'{}'", token.replace('\'', "'\\''"))
}

// ═══════════════════════════════════════════════════════════════════════════
// RealTmux — the PaneTransport implementation
// ═══════════════════════════════════════════════════════════════════════════

/// The production tmux transport.
///
/// `panes` is the job→pane map this transport needs and herdr does not.
/// herdr tracks agents by name server-side, so `agent_status(job_id)` is a
/// lookup it can answer alone. tmux knows nothing about jobs, so bee has to
/// remember which pane a job was started into: `agent_start` records the
/// pairing and the three job-addressed methods (`agent_status`,
/// `agent_prompt`, `agent_wait`) read it back. A job with no recorded pane
/// is unverifiable, never a guess — `None` or an `Err`, matching the
/// fail-safe posture `RealHerdr` takes for an absent agent.
pub(crate) struct RealTmux {
    settings: TmuxSettings,
    panes: Mutex<HashMap<String, String>>,
    /// Test-only `PATH` for the spawned `tmux`. Production leaves this
    /// `None` and inherits the process `PATH`; the stub-binary tests set it
    /// so a fake `tmux` is found WITHOUT mutating the test process's own
    /// environment, which `cargo test`'s parallel threads share.
    path_override: Option<OsString>,
}

impl RealTmux {
    pub(crate) fn new(settings: TmuxSettings) -> Self {
        Self { settings, panes: Mutex::new(HashMap::new()), path_override: None }
    }

    /// `new`, plus the stub `PATH` the in-module tests inject. Mirrors
    /// `fleet::backend::herdr::HerdrBackend::with_test_seams`.
    ///
    /// `pub(crate)`, not private: `crates/bee` is a binary crate with
    /// in-module tests only, so a SIBLING module's tests (the pane verbs
    /// built on this transport) can reach a stub `tmux` only through a
    /// seam this module exports. Still `#[cfg(test)]` — it exists in no
    /// shipped build.
    #[cfg(test)]
    pub(crate) fn with_test_path(settings: TmuxSettings, path: OsString) -> Self {
        Self { settings, panes: Mutex::new(HashMap::new()), path_override: Some(path) }
    }

    /// Seeds the job→pane map without typing anything into the pane, so a
    /// test can exercise a job-addressed method in isolation from
    /// `agent_start`'s sends.
    #[cfg(test)]
    fn remember_pane(&self, job_id: &str, pane_id: &str) {
        self.record_pane(job_id, pane_id);
    }

    /// Runs one `tmux` invocation and returns its trimmed stdout.
    ///
    /// Every tmux verb in this file goes through here, so there is exactly
    /// one place that decides what "trouble" means: a `tmux` that will not
    /// spawn (not installed, not on `PATH`) and a non-zero exit are both
    /// `Err`, and the message names the full argv. Naming the argv is what
    /// makes a live failure diagnosable — tmux's own stderr is often a bare
    /// "can't find pane", which says nothing about which call produced it.
    ///
    /// stdout is trimmed because every consumer here wants it trimmed: an
    /// id read is a single line with a newline tmux always appends, and a
    /// screen capture ends in the blank rows padding the bottom of the
    /// pane, which only widen the classifier's tail window for nothing.
    fn call(&self, args: &[&str]) -> Result<String, String> {
        let mut cmd = Command::new("tmux");
        cmd.args(args).stdin(Stdio::null());
        if let Some(path) = &self.path_override {
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

    /// One bounded screen read of `pane_id`.
    fn capture(&self, pane_id: &str) -> Result<String, String> {
        let start = self.settings.capture_start();
        self.call(&["capture-pane", "-p", "-t", pane_id, "-S", &start])
    }

    fn record_pane(&self, job_id: &str, pane_id: &str) {
        let mut map = self.panes.lock().unwrap_or_else(|e| e.into_inner());
        map.insert(job_id.to_string(), pane_id.to_string());
    }

    /// The pane a job was started into, or `None` if this process never
    /// started it. A poisoned lock is recovered rather than panicking: a
    /// map of ids has no invariant a panic could have broken, and taking
    /// down the run verb over it would lose the mailbox result.
    fn pane_of(&self, job_id: &str) -> Option<String> {
        let map = self.panes.lock().unwrap_or_else(|e| e.into_inner());
        map.get(job_id).cloned()
    }

    /// Types one literal line into a pane and submits it, as the two
    /// separate `send-keys` calls tmux requires (D5's upstream discipline).
    ///
    /// `-l` means "these bytes literally", which is what makes a line
    /// containing `Enter`, `C-c` or any other key name safe to type — but
    /// it also means the newline cannot ride along in the same call, since
    /// tmux would type the characters `E`,`n`,`t`,`e`,`r`. The submit is
    /// therefore always a second invocation with the key NAME `Enter`.
    fn send_line(&self, pane_id: &str, line: &str) -> Result<(), String> {
        self.call(&["send-keys", "-t", pane_id, "-l", line])?;
        self.call(&["send-keys", "-t", pane_id, "Enter"])?;
        Ok(())
    }

    /// Geometry for one pane, picked out of its window's layout listing.
    fn geom_of(&self, pane_id: &str) -> Option<PaneGeom> {
        self.pane_layout(pane_id)?.into_iter().find(|g| g.pane_id == pane_id)
    }
}

/// Parses `list-panes -F '#{pane_id} #{pane_width} #{pane_height}'` output.
/// Pure, so the parse is pinned by a test against captured tmux text rather
/// than only ever seen through a fake — the failure
/// `docs/knowledge/patterns/20260821-a-faked-seam-hides-the-parse.md`
/// records. A malformed row is dropped (mirroring `extract_pane_layout` in
/// run.rs); a body with no usable row at all is `None`, because for tmux an
/// empty listing means the read failed, not that the window is empty.
fn parse_pane_geoms(stdout: &str) -> Option<Vec<PaneGeom>> {
    let geoms: Vec<PaneGeom> = stdout
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let pane_id = parts.next()?.to_string();
            let width = parts.next()?.parse::<u64>().ok()?;
            let height = parts.next()?.parse::<u64>().ok()?;
            Some(PaneGeom { pane_id, width, height })
        })
        .collect();
    if geoms.is_empty() {
        None
    } else {
        Some(geoms)
    }
}

/// Parses `list-panes -F '#{pane_id} #{pane_pid} #{pane_current_command}
/// #{pane_dead}'` and reduces the row for `pane_id` to a `Liveness`.
/// Pure, for the same reason `parse_pane_geoms` is.
///
/// Fails OPEN to `Unknown` — the row missing, the last field unparseable,
/// or the pane simply absent from the listing all read as "cannot tell".
/// `pane_dead=1` is tmux's own word for a pane whose process exited while
/// `remain-on-exit` held the pane open, so it maps to `Absent`.
fn parse_liveness(stdout: &str, pane_id: &str) -> Liveness {
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 || parts[0] != pane_id {
            continue;
        }
        if parts[3] == "1" {
            return Liveness::Absent;
        }
        return match parts[1].parse::<u32>() {
            Ok(pid) => Liveness::Alive { pid },
            Err(_) => Liveness::Unknown,
        };
    }
    Liveness::Unknown
}

impl PaneTransport for RealTmux {
    /// The one override of the trait's `"herdr"` default: this transport is
    /// the tmux arm of `herding.transport` (tmux-herding-transport D1).
    fn name(&self) -> &'static str {
        "tmux"
    }

    /// `$TMUX_PANE` when the caller's own shell exports it (tmux sets it in
    /// every pane), else `display-message -p '#{pane_id}'`. The env read
    /// comes first because it is free and exact; the tmux call is the
    /// fallback for a shell that lost the variable — and unlike the env
    /// read it resolves to tmux's ACTIVE pane, which is only the caller's
    /// own pane when the caller is the focused one.
    ///
    /// This is not transport auto-detection (D1 forbids that): the
    /// transport was already chosen from config before this runs. It only
    /// answers "which pane am I".
    fn pane_current(&self) -> Result<String, String> {
        if let Ok(pane) = std::env::var("TMUX_PANE") {
            if !pane.trim().is_empty() {
                return Ok(pane.trim().to_string());
            }
        }
        let pane = self.call(&["display-message", "-p", "#{pane_id}"])?;
        if pane.is_empty() {
            return Err("tmux display-message -p '#{pane_id}' returned nothing".to_string());
        }
        Ok(pane)
    }

    /// Geometry for every pane in the window holding `pane_id` — which is
    /// exactly what `list-panes -t <pane>` returns, since a pane target
    /// resolves to its window. That whole-window listing is what the caller
    /// wants: the split-parent choice, the direction and the ratio are all
    /// pure functions over the full list (herding-split-serialize D2).
    ///
    /// The format string is passed as a bare argv token. The surrounding
    /// single quotes seen in tmux documentation are SHELL syntax; there is
    /// no shell here, so quoting them would make them part of the format.
    ///
    /// `None` on any trouble — fails open, the caller falls back to its own
    /// pane rather than losing the run over a geometry read.
    fn pane_layout(&self, pane_id: &str) -> Option<Vec<PaneGeom>> {
        let out = self
            .call(&["list-panes", "-t", pane_id, "-F", "#{pane_id} #{pane_width} #{pane_height}"])
            .ok()?;
        parse_pane_geoms(&out)
    }

    /// Splits a new pane out of `pane_id` inside the caller's own window
    /// (D2) and returns its id.
    ///
    /// The size arithmetic is the one thing worth reading twice. The
    /// trait's `ratio` is the share the PARENT KEEPS, while tmux's `-l`
    /// wants the size of the CHILD, so the two are complements: the child
    /// gets `round(parent_cells * (1 - ratio))` cells of whichever
    /// dimension the split divides — width for a `-h` (side-by-side) split,
    /// height for a `-v` (stacked) one. Passing the ratio straight through
    /// would hand the worker the main pane's share and squeeze the human.
    ///
    /// Flags: `-d` keeps focus where the human left it, `-P -F
    /// '#{pane_id}'` prints the new id, `-c` sets the child's start
    /// directory. A parent whose geometry cannot be read falls open to
    /// tmux's default even split rather than failing the spawn.
    fn pane_split(
        &self,
        pane_id: &str,
        direction: &str,
        ratio: f64,
        cwd: &Path,
    ) -> Result<String, String> {
        let flag = match direction {
            "right" => "-h",
            "down" => "-v",
            other => {
                return Err(format!(
                    "tmux pane_split: unknown direction {other:?} — the only directions bee \
                     splits are \"right\" and \"down\""
                ))
            }
        };
        let cwd_str = cwd.display().to_string();
        let mut argv: Vec<String> =
            vec!["split-window".into(), "-t".into(), pane_id.into(), flag.into()];
        if let Some(geom) = self.geom_of(pane_id) {
            let parent_cells = if flag == "-h" { geom.width } else { geom.height };
            let child = (parent_cells as f64 * (1.0 - ratio)).round().max(1.0) as u64;
            argv.push("-l".into());
            argv.push(child.to_string());
        }
        argv.extend([
            "-c".to_string(),
            cwd_str,
            "-d".to_string(),
            "-P".to_string(),
            "-F".to_string(),
            "#{pane_id}".to_string(),
        ]);
        let borrowed: Vec<&str> = argv.iter().map(String::as_str).collect();
        let new_pane = self.call(&borrowed)?;
        if new_pane.is_empty() {
            return Err(format!("tmux {} printed no pane id", argv.join(" ")));
        }
        Ok(new_pane)
    }

    /// The new-window fallback for a window with no pane roomy enough to
    /// split into a usable child: `new-window -d` gives the worker a fresh
    /// window's root pane at full width.
    ///
    /// `workspace` is IGNORED. tmux has no workspace concept — its
    /// hierarchy is server → session → window → pane, with no layer
    /// matching herdr's workspace — and the caller's session is already the
    /// only place D2 lets a worker land, so there is nothing to select.
    /// The parameter stays in the signature because the trait is shared
    /// with the herdr transport, where it is meaningful.
    ///
    /// `-d` mirrors `pane_split`'s own no-focus rule: a worker never steals
    /// the human's focus, not even to a new window.
    fn tab_create(&self, _workspace: &str, cwd: &Path, label: &str) -> Result<String, String> {
        let cwd_str = cwd.display().to_string();
        let pane = self.call(&[
            "new-window",
            "-d",
            "-c",
            &cwd_str,
            "-n",
            label,
            "-P",
            "-F",
            "#{pane_id}",
        ])?;
        if pane.is_empty() {
            return Err("tmux new-window printed no pane id".to_string());
        }
        Ok(pane)
    }

    /// Types one shell line into the pane and submits it — the per-agent
    /// `export K='v' …` line, sent after the split and before
    /// `agent_start` so the agent's own process inherits the vars.
    fn pane_run(&self, pane_id: &str, command: &str) -> Result<(), String> {
        self.send_line(pane_id, command)
    }

    /// Starts the agent by TYPING its command line into the pane's shell,
    /// then records job→pane so the job-addressed methods can find it.
    ///
    /// The line is `exec '<kind>' '<arg>'…`. Two details carry weight:
    ///
    /// `exec` replaces the pane's shell with the agent process instead of
    /// leaving the agent as its child. That is what makes `#{pane_pid}` in
    /// `process_info` the AGENT's pid, and what makes the pane close when
    /// the agent exits rather than dropping back to a prompt that looks
    /// idle.
    ///
    /// Every token is single-quoted by `shell_quote`, kind included. This
    /// is a shell line, not an argv — an unquoted task fragment containing
    /// `$`, a space or a backtick would be expanded or split by the pane's
    /// shell before the agent ever saw it. As on the herdr transport, the
    /// brief is never an argv token here: it travels through `agent_prompt`
    /// after the start succeeds (herding-run-prompt-delivery D1).
    fn agent_start(
        &self,
        job_id: &str,
        kind: &str,
        pane_id: &str,
        args: &[String],
    ) -> Result<(), String> {
        self.record_pane(job_id, pane_id);
        let mut line = String::from("exec ");
        line.push_str(&shell_quote(kind));
        for arg in args {
            line.push(' ');
            line.push_str(&shell_quote(arg));
        }
        self.send_line(pane_id, &line)
    }

    /// One bounded screen read, classified (D4). `None` on any trouble —
    /// an unknown job, a `tmux` that will not run, a failed capture. Never
    /// a "safe" guess: an unverifiable status must not count as a
    /// heartbeat, the same rule `RealHerdr::agent_status` follows.
    fn agent_status(&self, job_id: &str) -> Option<String> {
        let pane = self.pane_of(job_id)?;
        let screen = self.capture(&pane).ok()?;
        Some(classify(&screen, &self.settings).as_str().to_string())
    }

    fn pane_close(&self, pane_id: &str) -> Result<(), String> {
        self.call(&["kill-pane", "-t", pane_id]).map(|_| ())
    }

    /// Sends a prompt to an already-running agent and waits for the pane to
    /// show it landed.
    ///
    /// D3 preflight first: if the baseline screen classifies `Blocked`,
    /// this returns without typing ANYTHING. Sending into a dialog would
    /// answer it on the human's behalf, and the prompt text's first
    /// character is as likely to be "y" as anything else.
    ///
    /// Then text and `Enter` as two calls, and a poll for evidence. For
    /// `until == "working"` — the only value run.rs passes — evidence is a
    /// busy marker OR any change from the baseline screen. The second
    /// clause matters more than the first: text appearing on screen proves
    /// the keys were TYPED, never that they were SUBMITTED (upstream's own
    /// finding, D5), so the baseline comparison is what distinguishes a
    /// submitted prompt from one sitting unsent in the composer. Any other
    /// `until` waits for `classify` to reach that state.
    ///
    /// A timeout is reported with the literal `agent_prompt_stalled`, the
    /// string `is_agent_prompt_stalled` in run.rs greps for, so the bounded
    /// resend path (herding-prompt-stall D6) behaves identically on both
    /// transports.
    fn agent_prompt(
        &self,
        job_id: &str,
        prompt: &str,
        until: &str,
        timeout_ms: u64,
    ) -> Result<(), String> {
        let pane = self
            .pane_of(job_id)
            .ok_or_else(|| format!("tmux agent_prompt: no pane recorded for job {job_id}"))?;
        let baseline = self.capture(&pane).unwrap_or_default();
        if classify(&baseline, &self.settings) == Screen::Blocked {
            return Err(format!(
                "tmux agent_prompt: pane {pane} is showing a dialog (blocked) — nothing was \
                 typed; a human answers it"
            ));
        }

        self.send_line(&pane, prompt)?;

        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let interval = Duration::from_millis(self.settings.interval_ms);
        let want = match until {
            "idle" => Some(Screen::Idle),
            "working" => Some(Screen::Working),
            "blocked" => Some(Screen::Blocked),
            _ => None,
        };
        loop {
            if let Ok(screen) = self.capture(&pane) {
                let state = classify(&screen, &self.settings);
                let landed = match want {
                    Some(Screen::Working) | None => {
                        state == Screen::Working || screen != baseline
                    }
                    Some(wanted) => state == wanted,
                };
                if landed {
                    return Ok(());
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(format!(
                    "agent_prompt_stalled: pane {pane} showed no observed change within \
                     {timeout_ms} ms of the prompt"
                ));
            }
            std::thread::sleep(interval.min(deadline - now));
        }
    }

    /// Polls the pane until its screen settles, and reports what settled.
    ///
    /// Stability is upstream's rule (D5): `quiet_cycles` consecutive
    /// IDENTICAL reads, `interval_ms` apart. A settled screen with no busy
    /// marker is `idle`; a settled screen still showing a busy marker is
    /// not — the agent is thinking with a static frame, so the poll keeps
    /// going.
    ///
    /// A blocked marker short-circuits the whole loop the moment it appears
    /// (D3): the wait ends as `blocked` without waiting for stability and
    /// without sending a single key, because a dialog will never settle on
    /// its own and no key of bee's may answer it.
    ///
    /// `None` at timeout, on an unknown job, or on a read that never
    /// succeeds — the same unverifiable-is-not-a-heartbeat rule
    /// `agent_status` follows.
    fn agent_wait(&self, job_id: &str, timeout_ms: u64) -> Option<String> {
        let pane = self.pane_of(job_id)?;
        let deadline = Instant::now() + Duration::from_millis(timeout_ms);
        let interval = Duration::from_millis(self.settings.interval_ms);
        let mut last: Option<String> = None;
        let mut unchanged: u32 = 0;
        loop {
            match self.capture(&pane) {
                Ok(screen) => {
                    if classify(&screen, &self.settings) == Screen::Blocked {
                        return Some(Screen::Blocked.as_str().to_string());
                    }
                    if last.as_deref() == Some(screen.as_str()) {
                        unchanged = unchanged.saturating_add(1);
                    } else {
                        unchanged = 1;
                        last = Some(screen.clone());
                    }
                    if unchanged >= self.settings.quiet_cycles
                        && classify(&screen, &self.settings) == Screen::Idle
                    {
                        return Some(Screen::Idle.as_str().to_string());
                    }
                }
                // A failed read is not a settled screen: reset, keep
                // polling, and let the deadline decide.
                Err(_) => {
                    unchanged = 0;
                    last = None;
                }
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            std::thread::sleep(interval.min(deadline - now));
        }
    }

    /// Membership of `pane_id` in the server-wide pane listing (`-a`).
    ///
    /// Fails CLOSED, unlike `pane_layout`: a `tmux` that will not spawn, a
    /// non-zero exit and an unreadable body all read as "not alive". This
    /// is the `--continue` refusal gate — bee must not type a brief into a
    /// pane it cannot confirm still exists.
    fn pane_alive(&self, pane_id: &str) -> bool {
        let Ok(out) = self.call(&["list-panes", "-a", "-F", "#{pane_id}"]) else {
            return false;
        };
        out.lines().any(|line| line.trim() == pane_id)
    }

    fn pane_read(&self, pane_id: &str) -> Result<String, String> {
        self.capture(pane_id)
    }

    /// Liveness of the process in the pane, from tmux's own pane fields.
    ///
    /// `#{pane_id}` leads the format even though only one pane is wanted:
    /// `list-panes -t <pane>` resolves its target to the pane's WINDOW and
    /// lists every pane in it, so without the id in the row there is no way
    /// to tell which line belongs to the target — and under D2 that window
    /// always holds the human's pane plus every other worker.
    ///
    /// `#{pane_current_command}` is captured but not judged. Naming which
    /// commands count as "an agent" would be a guess about another tool's
    /// process name; `agent_start`'s `exec` already makes the pane's own
    /// pid the agent's, so a live pid is a live agent. Fails open to
    /// `Unknown`.
    fn process_info(&self, pane_id: &str) -> Liveness {
        let Ok(out) = self.call(&[
            "list-panes",
            "-t",
            pane_id,
            "-F",
            "#{pane_id} #{pane_pid} #{pane_current_command} #{pane_dead}",
        ]) else {
            return Liveness::Unknown;
        };
        parse_liveness(&out, pane_id)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// tests
// ═══════════════════════════════════════════════════════════════════════════
//
// `crates/bee` is a BINARY crate with no lib target, so nothing under
// `crates/bee/tests/` could reach these items — every test here is
// in-module by necessity, not by preference.
//
// No test needs a real tmux. The pure halves (classifier, config override,
// quoting, both parsers) run everywhere including Windows; the round-trip
// tests drive a stub `tmux` shell script and are `#[cfg(unix)]` for that
// reason, the same recorded gap `crates/fleet/tests/herdr_backend.rs`
// carries.
#[cfg(test)]
mod tests {
    use super::*;

    // ── screen fixtures ─────────────────────────────────────────────────
    //
    // The classifier's OWN tests moved down to `crates/fleet/src/screen.rs`
    // with the classifier (D4) — there is one body now, so there is one
    // set of tests for it. These three captured screens stay here because
    // the stub-tmux round trips below feed them to a fake `capture-pane`,
    // and because the config-override test needs a real dialog to prove a
    // rotted marker list stops recognising.

    /// A settled shell prompt: no chrome, no dialog.
    const IDLE_SCREEN: &str = "\
$ ls
Cargo.toml  src  tests
$
";

    /// Claude Code mid-turn — the busy footer is the last painted row.
    const WORKING_SCREEN: &str = "\
> summarize the plan

* Thinking… (12s · esc to interrupt)
";

    /// The trust dialog, with the busy footer still on screen above it.
    const BLOCKED_SCREEN: &str = "\
* Thinking… (3s · esc to interrupt)

╭──────────────────────────────────────────╮
│ Do you trust the files in this folder?   │
│                                          │
│ /home/dev/project                        │
│                                          │
│ ❯ 1. Yes, proceed                        │
│   2. No, exit                            │
╰──────────────────────────────────────────╯
";

    // ── settings from config ────────────────────────────────────────────

    #[test]
    fn tmux_settings_default_carries_the_upstream_marker_lists() {
        let d = TmuxSettings::default();
        assert!(d.busy_markers.iter().any(|m| m == "esc to interrupt"));
        assert!(d.busy_markers.iter().any(|m| m == "press esc to"));
        assert!(d.blocked_markers.iter().any(|m| m == "do you trust"));
        assert!(d.blocked_markers.iter().any(|m| m == "press enter to submit"));
        assert_eq!(d.scrollback, 40);
        assert_eq!(d.quiet_cycles, 3);
        assert_eq!(d.interval_ms, 2000);
    }

    #[test]
    fn tmux_from_config_replaces_one_list_and_keeps_every_other_default() {
        let cfg: Value = serde_json::json!({
            "herding": { "tmux": { "busy_markers": ["working on it"] } }
        });
        let s = TmuxSettings::from_config(&cfg);
        // REPLACES, never extends — the stale default must be gone.
        assert_eq!(s.busy_markers, vec!["working on it".to_string()]);
        assert!(!s.busy_markers.iter().any(|m| m == "esc to interrupt"));
        // Every other key keeps its default.
        assert_eq!(s.blocked_markers, TmuxSettings::default().blocked_markers);
        assert_eq!(s.scrollback, 40);
        assert_eq!(s.quiet_cycles, 3);
        assert_eq!(s.interval_ms, 2000);
    }

    #[test]
    fn tmux_from_config_reads_the_scalar_knobs_and_floors_quiet_cycles_at_one() {
        let cfg: Value = serde_json::json!({
            "herding": {
                "tmux": { "scrollback": 200, "quiet_cycles": 0, "interval_ms": 500 }
            }
        });
        let s = TmuxSettings::from_config(&cfg);
        assert_eq!(s.scrollback, 200);
        assert_eq!(s.quiet_cycles, 1, "zero quiet cycles would settle on the first read");
        assert_eq!(s.interval_ms, 500);
    }

    #[test]
    fn tmux_from_config_with_no_herding_tmux_block_is_the_default() {
        let cfg: Value = serde_json::json!({"herding": {"transport": "tmux"}});
        assert_eq!(TmuxSettings::from_config(&cfg), TmuxSettings::default());
    }

    #[test]
    fn tmux_overridden_markers_drive_the_classifier() {
        let cfg: Value = serde_json::json!({
            "herding": { "tmux": { "blocked_markers": ["approve this action"] } }
        });
        let s = TmuxSettings::from_config(&cfg);
        // The upstream default is GONE, so the very same trust dialog now
        // reads as a settled screen — which is exactly the damage a wrong
        // marker list does, and why D4 keeps the list correctable.
        assert_eq!(classify(BLOCKED_SCREEN, &s), Screen::Idle);
        // The repo's own marker blocks in its place.
        assert_eq!(classify("Approve this action? (y/n)\n", &s), Screen::Blocked);
    }

    // ── shell quoting ───────────────────────────────────────────────────

    #[test]
    fn tmux_shell_quote_neutralizes_a_single_quote_and_a_dollar_sign() {
        // The line is TYPED into a shell, so `$HOME` must survive as five
        // literal characters and the apostrophe must not end the quoting.
        assert_eq!(shell_quote("it's $HOME"), r#"'it'\''s $HOME'"#);
    }

    #[test]
    fn tmux_shell_quote_keeps_a_plain_token_one_word() {
        assert_eq!(shell_quote("--task"), "'--task'");
        assert_eq!(shell_quote("two words"), "'two words'");
    }

    // ── pure parsers, pinned to captured tmux output ────────────────────

    #[test]
    fn tmux_parse_pane_geoms_reads_a_multi_pane_window_listing() {
        // `tmux list-panes -t %0 -F '#{pane_id} #{pane_width} #{pane_height}'`
        let out = "%0 120 43\n%3 59 43\n%4 59 21\n";
        let geoms = parse_pane_geoms(out).expect("a well-formed listing parses");
        assert_eq!(geoms.len(), 3);
        assert_eq!(geoms[0], PaneGeom { pane_id: "%0".into(), width: 120, height: 43 });
        assert_eq!(geoms[2], PaneGeom { pane_id: "%4".into(), width: 59, height: 21 });
    }

    #[test]
    fn tmux_parse_pane_geoms_drops_a_bad_row_and_refuses_an_empty_body() {
        assert_eq!(parse_pane_geoms("%0 wide 43\n%3 59 43\n").unwrap().len(), 1);
        assert!(parse_pane_geoms("").is_none());
    }

    #[test]
    fn tmux_parse_liveness_maps_the_captured_rows() {
        let out = "%0 4110 bash 0\n%7 4242 claude 0\n%9 5150 bash 1\n";
        assert_eq!(parse_liveness(out, "%7"), Liveness::Alive { pid: 4242 });
        assert_eq!(parse_liveness(out, "%9"), Liveness::Absent, "pane_dead=1 is Absent");
        assert_eq!(parse_liveness(out, "%42"), Liveness::Unknown, "an absent pane cannot be judged");
        assert_eq!(parse_liveness("", "%7"), Liveness::Unknown);
    }

    // ── stub-tmux round trips ───────────────────────────────────────────

    #[cfg(unix)]
    mod stub {
        use super::*;
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        use std::path::PathBuf;
        use std::sync::atomic::{AtomicU64, Ordering};

        static STUB_SEQ: AtomicU64 = AtomicU64::new(0);

        /// A throwaway `tmux` shell script on a private PATH.
        ///
        /// It logs every invocation's argv and prints one canned body per
        /// tmux verb (keyed on `$1`, which is all this file's calls need to
        /// be told apart). Argument boundaries are written with ASCII Unit
        /// Separators and invocation boundaries with Record Separators —
        /// never plain newlines, because a captured screen body and a
        /// prompt are both multi-line arguments that would otherwise
        /// fragment one invocation into several log lines. Same shape as
        /// `crates/fleet/tests/herdr_backend.rs`.
        pub(super) struct Stub {
            dir: PathBuf,
            state: PathBuf,
            log: PathBuf,
        }

        impl Stub {
            pub(super) fn new() -> Self {
                let seq = STUB_SEQ.fetch_add(1, Ordering::SeqCst);
                let root = std::env::temp_dir()
                    .join(format!("bee-tmux-stub-{}-{seq}", std::process::id()));
                let dir = root.join("bin");
                let state = root.join("state");
                fs::create_dir_all(&dir).unwrap();
                fs::create_dir_all(&state).unwrap();
                let log = root.join("invocations.log");
                let _ = fs::remove_file(&log);

                let script = format!(
                    "#!/bin/sh\n\
                     for a in \"$@\"; do printf '%s\\037' \"$a\"; done >> {log}\n\
                     printf '\\036' >> {log}\n\
                     Q={state}/\"$1\"\n\
                     if [ -f \"$Q.out\" ]; then cat \"$Q.out\"; fi\n\
                     if [ -f \"$Q.exit\" ]; then exit \"$(cat \"$Q.exit\")\"; fi\n\
                     exit 0\n",
                    log = sq(&log),
                    state = sq(&state),
                );
                let script_path = dir.join("tmux");
                fs::write(&script_path, script).unwrap();
                let mut perms = fs::metadata(&script_path).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&script_path, perms).unwrap();

                Self { dir, state, log }
            }

            /// What `tmux <verb> …` prints, and optionally the code it
            /// exits with.
            pub(super) fn reply(&self, verb: &str, body: &str, exit_code: Option<i32>) {
                fs::write(self.state.join(format!("{verb}.out")), body).unwrap();
                if let Some(code) = exit_code {
                    fs::write(self.state.join(format!("{verb}.exit")), code.to_string()).unwrap();
                }
            }

            /// A transport whose spawned `tmux` resolves to this stub. The
            /// PATH is injected into the CHILD only — never into the test
            /// process's own environment, which every parallel `cargo test`
            /// thread shares.
            pub(super) fn tmux(&self) -> RealTmux {
                RealTmux::with_test_path(TmuxSettings::default(), self.child_path())
            }

            /// The stub directory PREPENDED to the inherited `PATH`, the
            /// same shape `HerdrBackend::child_path` builds. Prepending
            /// (rather than replacing) is required, not cosmetic: the stub
            /// is a shell script that runs `cat`, so an empty tail would
            /// leave the script unable to find the very coreutils it needs
            /// and every canned reply would come back as empty stdout. The
            /// stub still wins over a real `tmux` because it comes first.
            pub(super) fn child_path(&self) -> OsString {
                let mut out = self.dir.clone().into_os_string();
                if let Some(inherited) = std::env::var_os("PATH") {
                    out.push(":");
                    out.push(inherited);
                }
                out
            }

            /// Every invocation, args space-joined, in call order.
            pub(super) fn invocations(&self) -> Vec<String> {
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

        /// Single-quotes a scratch path for embedding in the generated
        /// `sh` script. Every path here is one this test just created.
        fn sq(path: &Path) -> String {
            format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
        }
    }

    #[cfg(unix)]
    use stub::Stub;

    #[cfg(unix)]
    #[test]
    fn tmux_pane_split_sizes_the_child_as_the_complement_of_the_parent_s_ratio() {
        let stub = Stub::new();
        // A 120-cell-wide parent, alone in its window.
        stub.reply("list-panes", "%1 120 40\n", None);
        stub.reply("split-window", "%7\n", None);
        let tmux = stub.tmux();

        // ratio 0.75 = the parent KEEPS three quarters, so the child gets
        // 120 * 0.25 = 30 cells. Handing tmux 0.75 would invert the split.
        let pane = tmux
            .pane_split("%1", "right", 0.75, Path::new("/tmp/wt"))
            .expect("the stub split succeeds");
        assert_eq!(pane, "%7");

        let calls = stub.invocations();
        assert_eq!(calls.len(), 2, "one geometry read, one split: {calls:?}");
        assert_eq!(calls[0], "list-panes -t %1 -F #{pane_id} #{pane_width} #{pane_height}");
        assert_eq!(
            calls[1],
            "split-window -t %1 -h -l 30 -c /tmp/wt -d -P -F #{pane_id}",
            "-h for right, -l is the CHILD's cells, -d keeps the human's focus"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tmux_pane_split_down_measures_height_and_never_new_sessions() {
        let stub = Stub::new();
        stub.reply("list-panes", "%1 120 40\n", None);
        stub.reply("split-window", "%8\n", None);
        let tmux = stub.tmux();
        tmux.pane_split("%1", "down", 0.5, Path::new("/tmp/wt")).unwrap();

        let calls = stub.invocations();
        assert_eq!(calls[1], "split-window -t %1 -v -l 20 -c /tmp/wt -d -P -F #{pane_id}");
        // D2: panes go in the caller's window. Nothing detaches a session.
        assert!(
            !calls.iter().any(|c| c.contains("new-session")
                || c.contains("attach-session")
                || c.contains("switch-client")),
            "D2 forbids a detached session per worker: {calls:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tmux_pane_split_refuses_an_unknown_direction() {
        let stub = Stub::new();
        let err = stub.tmux().pane_split("%1", "sideways", 0.5, Path::new("/tmp")).unwrap_err();
        assert!(err.contains("sideways"), "the refusal names the direction: {err}");
        assert!(stub.invocations().is_empty(), "a refused direction spawns no tmux");
    }

    #[cfg(unix)]
    #[test]
    fn tmux_pane_run_sends_the_text_and_enter_as_two_separate_invocations() {
        let stub = Stub::new();
        let tmux = stub.tmux();
        tmux.pane_run("%7", "export BEE_AGENT_NAME='w-1'").unwrap();

        let calls = stub.invocations();
        assert_eq!(calls.len(), 2, "text and Enter are never one call: {calls:?}");
        assert_eq!(calls[0], "send-keys -t %7 -l export BEE_AGENT_NAME='w-1'");
        assert_eq!(calls[1], "send-keys -t %7 Enter");
    }

    #[cfg(unix)]
    #[test]
    fn tmux_agent_start_types_one_shell_quoted_exec_line_and_records_the_pane() {
        let stub = Stub::new();
        let tmux = stub.tmux();
        tmux.agent_start(
            "job-9",
            "claude",
            "%7",
            &["--model".to_string(), "it's $HOME".to_string()],
        )
        .unwrap();

        let calls = stub.invocations();
        assert_eq!(calls.len(), 2);
        assert_eq!(
            calls[0],
            r#"send-keys -t %7 -l exec 'claude' '--model' 'it'\''s $HOME'"#,
            "every token is single-quoted, apostrophe and $ included"
        );
        assert_eq!(calls[1], "send-keys -t %7 Enter");

        // The pairing is what lets the job-addressed methods find the pane.
        stub.reply("capture-pane", IDLE_SCREEN, None);
        assert_eq!(tmux.agent_status("job-9").as_deref(), Some("idle"));
        assert_eq!(tmux.agent_status("job-nope"), None, "an unknown job is unverifiable");
    }

    #[cfg(unix)]
    #[test]
    fn tmux_agent_wait_reports_blocked_without_sending_a_single_key() {
        // The must-hold behavior of D3, end to end: a dialog on screen ends
        // the wait immediately and bee types nothing.
        let stub = Stub::new();
        stub.reply("capture-pane", BLOCKED_SCREEN, None);
        let tmux = stub.tmux();
        tmux.remember_pane("job-9", "%7");

        assert_eq!(tmux.agent_wait("job-9", 60_000).as_deref(), Some("blocked"));

        let calls = stub.invocations();
        assert_eq!(calls.len(), 1, "it returns on the first read, no polling: {calls:?}");
        assert!(
            !calls.iter().any(|c| c.contains("send-keys")),
            "D3: no key is ever typed into a blocked pane: {calls:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tmux_agent_prompt_refuses_a_blocked_pane_before_typing() {
        let stub = Stub::new();
        stub.reply("capture-pane", BLOCKED_SCREEN, None);
        let tmux = stub.tmux();
        tmux.remember_pane("job-9", "%7");

        let err = tmux.agent_prompt("job-9", "round 2 brief", "working", 5_000).unwrap_err();
        assert!(err.contains("blocked"), "the refusal says why: {err}");
        assert!(
            !stub.invocations().iter().any(|c| c.contains("send-keys")),
            "D3 preflight: the prompt is never typed into a dialog"
        );
    }

    #[cfg(unix)]
    #[test]
    fn tmux_agent_prompt_succeeds_when_the_screen_moves_and_stalls_by_name_when_it_does_not() {
        // The stub returns the SAME body for every capture, so the baseline
        // and the poll read match: no observed change, and the timeout must
        // carry the literal `is_agent_prompt_stalled` greps for.
        let stub = Stub::new();
        stub.reply("capture-pane", IDLE_SCREEN, None);
        let tmux = stub.tmux();
        tmux.remember_pane("job-9", "%7");

        let err = tmux.agent_prompt("job-9", "hello", "working", 0).unwrap_err();
        assert!(
            err.contains("agent_prompt_stalled"),
            "run.rs's is_agent_prompt_stalled matches on this literal: {err}"
        );
        let calls = stub.invocations();
        assert!(calls.iter().any(|c| c == "send-keys -t %7 -l hello"));
        assert!(calls.iter().any(|c| c == "send-keys -t %7 Enter"));
    }

    #[cfg(unix)]
    #[test]
    fn tmux_agent_prompt_lands_when_the_screen_differs_from_the_baseline() {
        let stub = Stub::new();
        stub.reply("capture-pane", WORKING_SCREEN, None);
        let tmux = stub.tmux();
        tmux.remember_pane("job-9", "%7");
        // Baseline and poll both read WORKING_SCREEN, whose busy marker is
        // the `until == "working"` evidence.
        tmux.agent_prompt("job-9", "go", "working", 5_000).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn tmux_pane_alive_is_true_on_membership_and_false_when_tmux_exits_nonzero() {
        let stub = Stub::new();
        stub.reply("list-panes", "%0\n%7\n", None);
        let tmux = stub.tmux();
        assert!(tmux.pane_alive("%7"));
        assert!(!tmux.pane_alive("%9"), "a pane not in the listing is not alive");
        assert_eq!(stub.invocations()[0], "list-panes -a -F #{pane_id}");

        // Fails CLOSED: a non-zero tmux is "not alive", never "assume yes".
        let broken = Stub::new();
        broken.reply("list-panes", "", Some(1));
        assert!(!broken.tmux().pane_alive("%7"));
    }

    #[cfg(unix)]
    #[test]
    fn tmux_process_info_reads_the_target_row_out_of_the_whole_window_listing() {
        let stub = Stub::new();
        // `list-panes -t %9` lists the WINDOW, so the human's pane is in
        // the body too — the row must be picked by id, not by position.
        stub.reply("list-panes", "%0 4110 bash 0\n%9 5150 bash 1\n", None);
        let tmux = stub.tmux();

        assert_eq!(tmux.process_info("%9"), Liveness::Absent, "pane_dead=1 is a dead pane");
        assert_eq!(
            stub.invocations()[0],
            "list-panes -t %9 -F #{pane_id} #{pane_pid} #{pane_current_command} #{pane_dead}"
        );

        let live = Stub::new();
        live.reply("list-panes", "%0 4110 bash 0\n%7 4242 claude 0\n", None);
        assert_eq!(live.tmux().process_info("%7"), Liveness::Alive { pid: 4242 });
    }

    #[cfg(unix)]
    #[test]
    fn tmux_process_info_and_pane_layout_are_unknown_when_tmux_is_missing() {
        // An empty PATH: no `tmux` to spawn at all.
        let tmux = RealTmux::with_test_path(TmuxSettings::default(), OsString::from(""));
        assert_eq!(tmux.process_info("%7"), Liveness::Unknown, "fails open");
        assert!(tmux.pane_layout("%7").is_none(), "fails open");
        assert!(!tmux.pane_alive("%7"), "fails closed");
        assert!(tmux.pane_read("%7").is_err());
    }

    #[cfg(unix)]
    #[test]
    fn tmux_tab_create_ignores_the_workspace_and_never_takes_focus() {
        let stub = Stub::new();
        stub.reply("new-window", "%12\n", None);
        let tmux = stub.tmux();
        let pane = tmux.tab_create("w4", Path::new("/tmp/wt"), "bee-worker").unwrap();
        assert_eq!(pane, "%12");

        let calls = stub.invocations();
        assert_eq!(calls[0], "new-window -d -c /tmp/wt -n bee-worker -P -F #{pane_id}");
        assert!(!calls[0].contains("w4"), "tmux has no workspace layer to select");
    }

    #[cfg(unix)]
    #[test]
    fn tmux_pane_read_and_close_use_the_bounded_capture_and_kill_pane() {
        let stub = Stub::new();
        stub.reply("capture-pane", IDLE_SCREEN, None);
        let tmux = stub.tmux();
        let body = tmux.pane_read("%7").unwrap();
        assert!(body.contains("Cargo.toml"));
        tmux.pane_close("%7").unwrap();

        let calls = stub.invocations();
        assert_eq!(calls[0], "capture-pane -p -t %7 -S -40", "scrollback is bounded (-S -40)");
        assert_eq!(calls[1], "kill-pane -t %7");
    }
}
