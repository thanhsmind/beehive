// verbs/supervisor — the supervisor observation store (Phase 1 of
// docs/history/slp-supervisor-heartbeat/plan.md; decisions da7cb49b and
// c80debd7).
//
// The supervisor role of `bee herding control-loop` wakes COLD on an
// interval, reads bee's existing state surfaces, and must leave a durable
// trace of what it saw — including the legitimate outcome "I looked and
// chose silence" (SLP §4.7). A cold tick keeps nothing in context, so the
// record IS the memory: an append-only JSONL store at the CONTROL root
// (shared across worktrees, exactly the `verbs/triggers` topology, which
// re-roots its store through `rsv::control_root_for`):
//
//   .bee/supervisor/observations.jsonl
//
// Row shape, one JSON object per line:
//   {ts, kind, signal, note, target_session, tick}
//
//   ts              write time, ISO-8601 with milliseconds
//   kind            `observation` | `silence` (closed set)
//   signal          `struggling-loop` | `big-decision` | `danger-op` |
//                   `none` (closed set — the day-1 signals of da7cb49b)
//   note            one or two sentences, whitespace-collapsed to one line
//   target_session  the OBSERVED session's id, or null when the note is
//                   about the herd rather than one session
//   tick            the control loop's tick index, or null when the verb
//                   is run outside a loop
//
// Verbs:
//   supervisor record --kind observation|silence [--signal <s>] --note <text>
//                     [--target-session <id>] [--tick <n>] [--json]
//   supervisor list   [--json]
//
// CLI-ONLY STATE. Nothing else in the tree writes this file; `record` is
// the one door, and it VALIDATES BEFORE IT WRITES — a refused row leaves
// the store byte-identical (`record_into` below is the seam both the verb
// and the tests go through, so "refuses typed and writes nothing" is one
// assertion, not two code paths that have to agree).
//
// Fail-open reads: one unparseable line warns (fsutil::warn_corrupt_jsonl_line)
// and is skipped with a count, never a crash — a store this role appends to
// unattended must stay readable after a partial write.
//
// This module OBSERVES. It never dispatches, merges, approves, or writes any
// other bee record (787a9eb0).

use super::feedback::{emit_error, emit_success, js_trim, now_iso, parse_shape, ParsedArgs};
use crate::fsutil::{append_jsonl, warn_corrupt_jsonl_line};
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::reservations as rsv;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{json, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

// ─── store paths ────────────────────────────────────────────────────────

fn supervisor_dir(control: &Path) -> PathBuf {
    control.join(".bee").join("supervisor")
}

fn observations_path(control: &Path) -> PathBuf {
    supervisor_dir(control).join("observations.jsonl")
}

/// The control root for a (possibly linked-worktree) repo root — the same
/// cycle-free git walk `verbs/triggers` and `deferred_queue` already reuse.
/// A supervisor tick and a worktree session must see ONE store.
fn control_root_path(root: &Path) -> PathBuf {
    let root_s = root.to_string_lossy().into_owned();
    PathBuf::from(rsv::control_root_for(&root_s).unwrap_or(root_s))
}

// ─── record ─────────────────────────────────────────────────────────────

pub(crate) const KNOWN_KINDS: [&str; 2] = ["observation", "silence"];
pub(crate) const KNOWN_SIGNALS: [&str; 4] = ["struggling-loop", "big-decision", "danger-op", "none"];

/// The bound on one note. Two sentences fit in a fraction of it; the cap
/// exists because these rows are rendered back into a ≤10-line WakeReport
/// (9f5cd250), and an unbounded note there is a broken report.
const MAX_NOTE_CHARS: usize = 500;

#[derive(Debug)]
pub(crate) struct Observation {
    pub(crate) ts: String,
    pub(crate) kind: String,
    pub(crate) signal: String,
    pub(crate) note: String,
    pub(crate) target_session: Option<String>,
    pub(crate) tick: Option<u64>,
}

impl Observation {
    fn to_value(&self) -> Value {
        json!({
            "ts": self.ts,
            "kind": self.kind,
            "signal": self.signal,
            "note": self.note,
            "target_session": self.target_session,
            "tick": self.tick,
        })
    }

    /// `None` for anything JSON-shaped but missing a required field or
    /// carrying a kind/signal outside the closed set — read exactly like a
    /// parse failure by every caller (skipped with a warning, never a crash).
    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        let ts = m.get("ts")?.as_str()?.to_string();
        let kind = m.get("kind")?.as_str()?.to_string();
        if !KNOWN_KINDS.contains(&kind.as_str()) {
            return None;
        }
        let signal = m.get("signal")?.as_str()?.to_string();
        if !KNOWN_SIGNALS.contains(&signal.as_str()) {
            return None;
        }
        let note = m.get("note")?.as_str()?.to_string();
        let target_session = m.get("target_session").and_then(Value::as_str).map(str::to_string);
        let tick = m.get("tick").and_then(Value::as_u64);
        Some(Self { ts, kind, signal, note, target_session, tick })
    }

    fn line(&self) -> String {
        let target = self.target_session.as_deref().unwrap_or("-");
        let tick = self.tick.map(|t| t.to_string()).unwrap_or_else(|| "-".to_string());
        format!("- {} [{}/{}] tick {} {} — {}", self.ts, self.kind, self.signal, tick, target, self.note)
    }
}

/// One line of free text, safe to append as one JSONL row and to render
/// back into a report: every whitespace run (newlines and tabs included)
/// collapses to a single space, and C0/C7F control characters are dropped.
/// A note that carries its own line breaks would otherwise re-shape any
/// surface that later prints it.
fn one_line(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pending_space = false;
    for c in text.chars() {
        if c.is_whitespace() {
            pending_space = !out.is_empty();
            continue;
        }
        if (c as u32) < 0x20 || c as u32 == 0x7f {
            continue;
        }
        if pending_space {
            out.push(' ');
            pending_space = false;
        }
        out.push(c);
    }
    out
}

fn closed_set_error(cmd: &str, flag: &str, got: &str, allowed: &[&str]) -> String {
    format!("bee {cmd}: --{flag} must be one of {}, got {got:?}.", allowed.join(", "))
}

/// Validate, then append. THE one write door for this store: every refusal
/// below returns before a single byte is written, so a refused `record`
/// leaves the store exactly as it found it (including still absent).
pub(crate) fn record_into(
    control: &Path,
    cmd: &str,
    kind: Option<&str>,
    signal: Option<&str>,
    note: Option<&str>,
    target_session: Option<&str>,
    tick: Option<&str>,
) -> Result<Observation, String> {
    let Some(kind) = kind else {
        return Err(format!("bee {cmd}: --kind is required ({}).", KNOWN_KINDS.join("|")));
    };
    if !KNOWN_KINDS.contains(&kind) {
        return Err(closed_set_error(cmd, "kind", kind, &KNOWN_KINDS));
    }
    // Absent `--signal` is the ordinary case for a `silence` row and for an
    // observation that carries no day-1 signal; it is spelled `none` in the
    // record so every row answers the question.
    let signal = signal.unwrap_or("none");
    if !KNOWN_SIGNALS.contains(&signal) {
        return Err(closed_set_error(cmd, "signal", signal, &KNOWN_SIGNALS));
    }
    let Some(note) = note else {
        return Err(format!(
            "bee {cmd}: --note is required — a record with no note tells the next cold tick nothing."
        ));
    };
    let note = one_line(note);
    if note.is_empty() {
        return Err(format!("bee {cmd}: --note must not be empty."));
    }
    if note.chars().count() > MAX_NOTE_CHARS {
        return Err(format!(
            "bee {cmd}: --note is {} characters; keep it to one or two sentences ({MAX_NOTE_CHARS} max).",
            note.chars().count()
        ));
    }
    let tick = match tick {
        None => None,
        Some(raw) => match raw.parse::<u64>() {
            Ok(n) => Some(n),
            Err(_) => {
                return Err(format!("bee {cmd}: --tick must be a non-negative whole number, got {raw:?}."))
            }
        },
    };
    let rec = Observation {
        ts: now_iso(),
        kind: kind.to_string(),
        signal: signal.to_string(),
        note,
        target_session: target_session.map(|s| one_line(s)).filter(|s| !s.is_empty()),
        tick,
    };
    // Creates .bee/supervisor/ on the first write (append_jsonl ensures the
    // parent directory) — the store never needs a separate init step.
    if append_jsonl(&observations_path(control), &rec.to_value()).is_err() {
        return Err(format!("bee {cmd}: could not append to the observation store."));
    }
    Ok(rec)
}

pub(crate) struct ReadStore {
    pub(crate) rows: Vec<Observation>,
    /// 1-based line numbers that did not parse into a row.
    pub(crate) unreadable: Vec<usize>,
}

/// Read the store oldest-first. A missing store reads as empty (the loop's
/// first tick has not run yet), never an error.
pub(crate) fn read_observations(control: &Path) -> ReadStore {
    let path = observations_path(control);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return ReadStore { rows: Vec::new(), unreadable: Vec::new() };
    };
    let mut rows = Vec::new();
    let mut unreadable = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line_no = i + 1;
        if js_trim(raw).is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(raw).ok().as_ref().and_then(Observation::from_value) {
            Some(rec) => rows.push(rec),
            None => {
                warn_corrupt_jsonl_line(&path, line_no);
                unreadable.push(line_no);
            }
        }
    }
    ReadStore { rows, unreadable }
}

// ─── argv plumbing ──────────────────────────────────────────────────────

struct Ctx {
    root: PathBuf,
    control: PathBuf,
    drift: crate::registry::Drift,
}

fn preamble(cmd: &str, pre_json: bool, t0: Instant) -> Result<Option<Ctx>, ExitCode> {
    let Ok(cwd) = std::env::current_dir() else { return Ok(None) };
    let root = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r.root,
        RootsWt::Unsupported(why) => return Err(emit_unsupported_root(&cwd, cmd, pre_json, t0, &why)),
        RootsWt::None => return Err(emit_no_root_error(&cwd, cmd, pre_json, t0)),
    };
    let control = control_root_path(&root);
    let drift = check_manifest_drift(&root);
    Ok(Some(Ctx { root, control, drift }))
}

fn flag<'a>(parsed: &'a ParsedArgs, name: &str) -> Option<&'a str> {
    parsed.flags.get(name).map(|s| js_trim(s)).filter(|s| !s.is_empty())
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "supervisor" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "record" => {
            run_record(parse_shape(rest, &["kind", "signal", "note", "target-session", "tick"])?, t0)
        }
        "list" => run_list(parse_shape(rest, &[])?, t0),
        _ => None,
    }
}

// ─── record ──────────────────────────────────────────────────────────────

fn run_record(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "supervisor record";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let rec = match record_into(
        &ctx.control,
        cmd,
        flag(&parsed, "kind"),
        flag(&parsed, "signal"),
        flag(&parsed, "note"),
        flag(&parsed, "target-session"),
        flag(&parsed, "tick"),
    ) {
        Ok(rec) => rec,
        Err(msg) => return Some(emit_error(&ctx.root, cmd, parsed.json, &msg, t0)),
    };
    let text = format!("Recorded {} ({}) in the supervisor observation store.", rec.kind, rec.signal);
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &rec.to_value(), &text, t0))
}

// ─── list ────────────────────────────────────────────────────────────────

fn run_list(parsed: ParsedArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "supervisor list";
    let ctx = match preamble(cmd, parsed.pre_json, t0) {
        Err(code) => return Some(code),
        Ok(c) => c?,
    };
    let store = read_observations(&ctx.control);
    let text = if store.rows.is_empty() {
        "No supervisor observations.".to_string()
    } else {
        store.rows.iter().map(Observation::line).collect::<Vec<_>>().join("\n")
    };
    let result = json!({
        "observations": store.rows.iter().map(Observation::to_value).collect::<Vec<_>>(),
        "unreadable_lines": store.unreadable,
    });
    Some(emit_success(&ctx.root, cmd, parsed.json, &ctx.drift, &result, &text, t0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(root: &Path, rel: &str, content: &str) {
        let p = root.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }

    fn git(cwd: &Path, args: &[&str]) {
        let out = std::process::Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .expect("git must be on PATH for the worktree fixture");
        assert!(out.status.success(), "git {:?} failed: {}", args, String::from_utf8_lossy(&out.stderr));
    }

    /// A real main checkout plus one real linked worktree — the same fixture
    /// verbs/triggers uses, re-derived locally since that helper is private.
    fn worktree_fixture(tmp: &Path) -> (PathBuf, PathBuf) {
        let tmp = dunce::canonicalize(tmp).unwrap_or_else(|_| tmp.to_path_buf());
        let main = tmp.join("main");
        std::fs::create_dir_all(&main).unwrap();
        write(&main, ".bee/onboarding.json", "{}");
        write(&main, "f.txt", "x");
        git(&main, &["init", "-q", "-b", "main", "."]);
        git(&main, &["config", "user.email", "a@b.c"]);
        git(&main, &["config", "user.name", "t"]);
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "init"]);
        let wt = tmp.join("wt");
        git(&main, &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "wt/one"]);
        write(&wt, ".bee/onboarding.json", "{}");
        (main, wt)
    }

    fn n(p: &Path) -> String {
        dunce::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()).to_string_lossy().into_owned()
    }

    fn lines(control: &Path) -> Vec<String> {
        std::fs::read_to_string(observations_path(control))
            .map(|t| t.lines().map(str::to_string).collect())
            .unwrap_or_default()
    }

    #[test]
    fn a_valid_record_appends_exactly_one_row_and_creates_the_store() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        assert!(!supervisor_dir(control).exists(), "fixture starts with no store");

        let rec = record_into(
            control,
            "supervisor record",
            Some("silence"),
            Some("none"),
            Some("Looked at every live session; nothing needs a question."),
            None,
            None,
        )
        .expect("a valid record is accepted");

        assert!(supervisor_dir(control).is_dir(), "the store directory is created on first write");
        let rows = lines(control);
        assert_eq!(rows.len(), 1, "exactly one row: {rows:?}");
        let parsed: Value = serde_json::from_str(&rows[0]).unwrap();
        assert_eq!(parsed["kind"], "silence");
        assert_eq!(parsed["signal"], "none");
        assert_eq!(parsed["note"], "Looked at every live session; nothing needs a question.");
        assert_eq!(parsed["target_session"], Value::Null);
        assert_eq!(parsed["tick"], Value::Null);
        assert!(parsed["ts"].as_str().unwrap().ends_with('Z'), "ts is ISO-8601: {}", parsed["ts"]);
        assert_eq!(rec.kind, "silence");

        // A second record appends rather than replaces — the store is a log.
        record_into(
            control,
            "supervisor record",
            Some("observation"),
            Some("struggling-loop"),
            Some("Same test has failed three times with the same error."),
            Some("sess-42"),
            Some("7"),
        )
        .unwrap();
        let rows = lines(control);
        assert_eq!(rows.len(), 2);
        let second: Value = serde_json::from_str(&rows[1]).unwrap();
        assert_eq!(second["target_session"], "sess-42");
        assert_eq!(second["tick"], 7);
    }

    #[test]
    fn a_bad_kind_is_a_typed_refusal_that_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let err = record_into(
            control,
            "supervisor record",
            Some("dispatch"),
            Some("none"),
            Some("x"),
            None,
            None,
        )
        .unwrap_err();
        assert!(err.contains("--kind must be one of observation, silence"), "{err}");
        assert!(err.contains("\"dispatch\""), "the refusal names what it got: {err}");
        assert!(!observations_path(control).exists(), "a refused record writes nothing at all");
        assert!(!supervisor_dir(control).exists(), "not even the store directory");
    }

    #[test]
    fn every_other_refusal_also_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let cmd = "supervisor record";
        let cases: Vec<(Option<&str>, Option<&str>, Option<&str>, Option<&str>, &str)> = vec![
            (None, Some("none"), Some("x"), None, "--kind is required"),
            (Some("observation"), Some("vibes"), Some("x"), None, "--signal must be one of"),
            (Some("observation"), Some("none"), None, None, "--note is required"),
            (Some("observation"), Some("none"), Some("   "), None, "--note must not be empty"),
            (Some("observation"), Some("none"), Some("x"), Some("later"), "--tick must be a non-negative"),
        ];
        for (kind, signal, note, tick, needle) in cases {
            let err = record_into(control, cmd, kind, signal, note, None, tick).unwrap_err();
            assert!(err.contains(needle), "expected {needle:?} in {err:?}");
        }
        let long = "s".repeat(MAX_NOTE_CHARS + 1);
        let err = record_into(control, cmd, Some("observation"), None, Some(&long), None, None).unwrap_err();
        assert!(err.contains("one or two sentences"), "{err}");
        assert!(!observations_path(control).exists(), "no refusal ever wrote a row");
    }

    #[test]
    fn list_reads_the_rows_back_in_order() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        assert!(read_observations(control).rows.is_empty(), "a missing store reads as empty");

        record_into(control, "c", Some("observation"), Some("big-decision"), Some("first"), None, Some("1"))
            .unwrap();
        record_into(control, "c", Some("silence"), None, Some("second"), Some("sess-9"), Some("2")).unwrap();

        let store = read_observations(control);
        assert!(store.unreadable.is_empty());
        assert_eq!(store.rows.len(), 2);
        assert_eq!(store.rows[0].note, "first");
        assert_eq!(store.rows[0].signal, "big-decision");
        assert_eq!(store.rows[1].note, "second");
        assert_eq!(store.rows[1].signal, "none", "an absent --signal is recorded as none");
        assert_eq!(store.rows[1].target_session.as_deref(), Some("sess-9"));
        assert!(store.rows[1].line().contains("tick 2"));
    }

    #[test]
    fn one_bad_line_is_skipped_with_a_count_never_a_crash() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        record_into(control, "c", Some("observation"), Some("danger-op"), Some("good"), None, None).unwrap();
        // A half-written line (the crash a cold unattended appender can leave)
        // and a shape-valid row with a kind outside the closed set.
        let path = observations_path(control);
        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("{\"ts\":\"2026-08-27T00:00:00.000Z\",\"kind\":\"obs\n");
        text.push_str("{\"ts\":\"2026-08-27T00:00:01.000Z\",\"kind\":\"merge\",\"signal\":\"none\",\"note\":\"x\"}\n");
        std::fs::write(&path, text).unwrap();

        let store = read_observations(control);
        assert_eq!(store.rows.len(), 1, "the one good row still reads back");
        assert_eq!(store.unreadable, vec![2, 3]);
    }

    #[test]
    fn a_note_is_collapsed_to_one_line() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        record_into(
            control,
            "c",
            Some("observation"),
            None,
            Some("  two\nlines\tand\r\nspaces  "),
            None,
            None,
        )
        .unwrap();
        let store = read_observations(control);
        assert_eq!(store.rows[0].note, "two lines and spaces");
        // One row means one line in the file, always.
        assert_eq!(lines(control).len(), 1);
    }

    #[test]
    fn the_store_resolves_to_the_control_root_from_a_linked_worktree() {
        let tmp = tempfile::tempdir().unwrap();
        let (main, wt) = worktree_fixture(tmp.path());
        assert_eq!(n(&control_root_path(&main)), n(&main));
        assert_eq!(n(&control_root_path(&wt)), n(&main));
        assert_eq!(
            n(&observations_path(&control_root_path(&wt))),
            n(&main.join(".bee").join("supervisor").join("observations.jsonl"))
        );
    }
}
