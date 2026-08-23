//! The screen classifier — the ONE reading of "what is this terminal pane
//! doing", shared by every caller that has nothing but pane text to go on.
//!
//! **Why it lives here.** tmux-herding-cockpit D4: waves and occupancy pick
//! the tmux backend from the same `herding.transport` key the run verb
//! does, and ONE classifier (markers plus stability knobs) serves both
//! crates. `bee` depends on `fleet`, never the other way round, so the
//! shared half moves DOWN into `fleet`; two copies would drift the moment
//! one crate's marker list was corrected and the other's was not.
//! `bee`'s `herding/tmux.rs` (`RealTmux`) reuses exactly this module, and
//! so does this crate's own `backend::tmux::TmuxBackend`.
//!
//! **What it is not.** There is no `Done` state and there must never be
//! one. A pane can only show what the agent PAINTED; the truth about a
//! finished job is the worker's own result file (herding-executor D3,
//! restated by tmux-herding-transport D4). Screen status is advisory —
//! a screen that looks finished is `Idle`, nothing more.
//!
//! **Crate-boundary note (D2).** This module holds the marker lists and
//! the poll knobs as plain data with upstream defaults. It does NOT read
//! `.bee/config.json` — parsing bee's config is bee's own job, and
//! `fleet` naming a bee config key would be the defect the crate boundary
//! exists to prevent. `bee`'s `TmuxSettings::from_config` builds a
//! `ScreenSettings` and hands it in already resolved, the same
//! caller-resolves-it rule `backend::herdr::HerdrBackend::new` follows for
//! its agent kind.

/// How many trailing non-empty lines each marker list is scanned over.
///
/// The two windows differ on purpose, and the difference comes from
/// upstream (`wait_for_idle.py`, tmux-herding-transport D5:
/// luongnv89/skills @ `ab46724e`, `skills/tmux-agent-comms/`). A busy
/// marker is TUI chrome painted on the very last row ("esc to interrupt"),
/// so a 2-line window keeps a stale mention scrolled up in the transcript
/// from reading as "still working". A dialog is a multi-row box, so its
/// marker can sit a dozen rows above the cursor and needs the wider
/// window.
const BUSY_TAIL_LINES: usize = 2;
const BLOCKED_TAIL_LINES: usize = 12;

/// The screen-reading knobs, defaulted from upstream `wait_for_idle.py`
/// (tmux-herding-transport D5) and overridable per repo — on the bee side,
/// under `herding.tmux.*`.
///
/// D4's reason for holding these as data rather than code: marker strings
/// are another tool's UI chrome. They rot with every CLI release, and a
/// repo pinned to an older agent build must be able to correct them
/// without a bee release.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenSettings {
    /// Strings whose presence in the last `BUSY_TAIL_LINES` non-empty
    /// lines means the agent is mid-turn.
    pub busy_markers: Vec<String>,
    /// Strings whose presence in the last `BLOCKED_TAIL_LINES` non-empty
    /// lines means a human must answer a dialog
    /// (tmux-herding-transport D3).
    pub blocked_markers: Vec<String>,
    /// Lines of scrollback each screen read pulls (tmux: `-S -<n>`).
    pub scrollback: u32,
    /// How many consecutive identical reads count as a settled screen.
    pub quiet_cycles: u32,
    /// Delay between polls, in milliseconds.
    pub interval_ms: u64,
}

impl Default for ScreenSettings {
    fn default() -> Self {
        Self {
            busy_markers: [
                "esc to interrupt",
                "esc to cancel",
                "ctrl+c to interrupt",
                "press esc to",
            ]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
            blocked_markers: [
                "do you trust",
                "trust the files",
                "paste your api key",
                "press enter to submit",
            ]
            .iter()
            .map(|s| (*s).to_string())
            .collect(),
            scrollback: 40,
            quiet_cycles: 3,
            interval_ms: 2000,
        }
    }
}

/// What a captured pane screen says about the agent in it.
///
/// There is no `Done` variant and there must never be one — see the module
/// docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Idle,
    Working,
    Blocked,
}

impl Screen {
    /// The wire spelling callers hand back on the herdr-shaped status
    /// vocabulary (`"idle"` / `"working"` / `"blocked"`), so a transport
    /// swap never changes what a caller branches on.
    pub fn as_str(self) -> &'static str {
        match self {
            Screen::Idle => "idle",
            Screen::Working => "working",
            Screen::Blocked => "blocked",
        }
    }
}

/// Classifies a captured pane body. Blocked BEATS working, always: a
/// dialog box frequently renders while the agent's own "esc to interrupt"
/// footer is still on screen, and reading that as `working` would let a
/// caller keep waiting on a pane that will never move until a human
/// answers it (tmux-herding-transport D3). Matching is case-insensitive
/// substring containment — TUI chrome changes case between releases far
/// more often than it changes wording.
pub fn classify(screen: &str, settings: &ScreenSettings) -> Screen {
    if any_marker(&tail_lines(screen, BLOCKED_TAIL_LINES), &settings.blocked_markers) {
        return Screen::Blocked;
    }
    if any_marker(&tail_lines(screen, BUSY_TAIL_LINES), &settings.busy_markers) {
        return Screen::Working;
    }
    Screen::Idle
}

/// The last `n` NON-EMPTY lines, oldest first. Blank lines are skipped
/// rather than counted: a TUI pads the bottom of a pane with empty rows,
/// so counting them would push real chrome out of a 2-line window.
fn tail_lines(screen: &str, n: usize) -> Vec<&str> {
    let non_empty: Vec<&str> = screen.lines().filter(|l| !l.trim().is_empty()).collect();
    let start = non_empty.len().saturating_sub(n);
    non_empty[start..].to_vec()
}

/// True when any line contains any marker, case-insensitively. An empty
/// marker string is ignored — it would otherwise match every line and pin
/// the classifier to one answer forever.
fn any_marker(lines: &[&str], markers: &[String]) -> bool {
    if markers.is_empty() {
        return false;
    }
    let lowered: Vec<String> = markers
        .iter()
        .filter(|m| !m.trim().is_empty())
        .map(|m| m.to_lowercase())
        .collect();
    lines.iter().any(|line| {
        let low = line.to_lowercase();
        lowered.iter().any(|m| low.contains(m))
    })
}

// ═══════════════════════════════════════════════════════════════════════
// tests
// ═══════════════════════════════════════════════════════════════════════
//
// These are the classifier tests MOVED down from
// `crates/bee/src/herding/tmux.rs` when D4 collapsed the two copies into
// this one. They are pure — no process, no filesystem — so they run on
// every platform, Windows included. The config-parsing tests stayed
// behind in `bee`, where the config parsing itself stayed.
#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn classify_reads_a_settled_shell_prompt_as_idle() {
        assert_eq!(classify(IDLE_SCREEN, &ScreenSettings::default()), Screen::Idle);
    }

    #[test]
    fn classify_reads_a_busy_footer_on_the_last_line_as_working() {
        assert_eq!(classify(WORKING_SCREEN, &ScreenSettings::default()), Screen::Working);
    }

    #[test]
    fn classify_reads_a_trust_dialog_as_blocked_even_with_a_busy_marker_present() {
        // D3, and the whole reason blocked is checked first: the dialog box
        // renders while the agent's own busy footer is still on screen. The
        // marker sits nine rows above the bottom, inside the 12-line
        // blocked window and outside the 2-line busy one.
        assert_eq!(classify(BLOCKED_SCREEN, &ScreenSettings::default()), Screen::Blocked);
    }

    #[test]
    fn classify_ignores_a_busy_marker_scrolled_out_of_the_last_two_lines() {
        // A transcript mentioning the chrome is not a working agent.
        let screen = "* Thinking… (2s · esc to interrupt)\ndone\n$\n";
        assert_eq!(classify(screen, &ScreenSettings::default()), Screen::Idle);
    }

    #[test]
    fn classify_matches_markers_case_insensitively() {
        let screen = "Press ESC To Interrupt\n";
        assert_eq!(classify(screen, &ScreenSettings::default()), Screen::Working);
    }

    #[test]
    fn classify_has_no_done_state() {
        // D4: the screen never reports done — the worker's result file
        // does. A pane literally printing "done" is idle, nothing more.
        let screen = "task complete. done.\n$\n";
        assert_eq!(classify(screen, &ScreenSettings::default()), Screen::Idle);
    }

    #[test]
    fn default_settings_carry_the_upstream_marker_lists() {
        let d = ScreenSettings::default();
        assert!(d.busy_markers.iter().any(|m| m == "esc to interrupt"));
        assert!(d.busy_markers.iter().any(|m| m == "press esc to"));
        assert!(d.blocked_markers.iter().any(|m| m == "do you trust"));
        assert!(d.blocked_markers.iter().any(|m| m == "press enter to submit"));
        assert_eq!(d.scrollback, 40);
        assert_eq!(d.quiet_cycles, 3);
        assert_eq!(d.interval_ms, 2000);
    }

    #[test]
    fn overridden_markers_drive_the_classifier() {
        // The upstream default is GONE, so the very same trust dialog now
        // reads as a settled screen — which is exactly the damage a wrong
        // marker list does, and why D4 keeps the list correctable.
        let settings = ScreenSettings {
            blocked_markers: vec!["approve this action".to_string()],
            ..ScreenSettings::default()
        };
        assert_eq!(classify(BLOCKED_SCREEN, &settings), Screen::Idle);
        assert_eq!(classify("Approve this action? (y/n)\n", &settings), Screen::Blocked);
    }

    #[test]
    fn an_empty_marker_list_never_matches() {
        let settings = ScreenSettings {
            busy_markers: Vec::new(),
            blocked_markers: Vec::new(),
            ..ScreenSettings::default()
        };
        assert_eq!(classify(BLOCKED_SCREEN, &settings), Screen::Idle);
        assert_eq!(classify(WORKING_SCREEN, &settings), Screen::Idle);
    }
}
