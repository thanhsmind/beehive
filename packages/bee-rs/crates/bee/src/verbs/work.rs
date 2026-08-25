// bee work — the agent's half of the prompt work record (prompt-work-record
// D1/D2/D5).
//
// The activity hook opens the record: the user's own words, status "open", and
// nothing yet about what finishing would mean. This group is the other half —
// the agent says what "done" is for that ask, and moves the record off `open`
// as the work advances. An acceptance is what D2 promotes from a line on a
// session row to a card on the board, so `work set --acceptance` IS the
// promotion; there is no separate one.
//
// Verbs served natively (exact argv shapes only):
//   work show [--session S] [--json]
//   work set  [--acceptance A] [--status open|active|done|dropped]
//             [--session S] [--json]
//
// Sink parity with the hook (D2), decided the same way and in the same order:
// an explicit --session wins; else a herded pane addresses its job mailbox and
// never a session file; else CLAUDE_CODE_SESSION_ID.
//
// D5 reaches the acceptance too, but as a REFUSAL rather than a redaction. The
// prompt is the user's text and bee stores what it is given; the acceptance is
// the agent's own sentence, and a credential inside one is a bug to report,
// not to swallow.
//
// hm-3 (docs/history/human-mailbox/CONTEXT.md D4/D9/D11): this group also
// carries the END of a run — see `file_letter_at_run_end` at the foot of the
// file, and the store itself in `verbs/mailbox.rs`.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::activity::well_formed_id;
use crate::lock::{acquire_store_lock_once, AcquireOnce};
use crate::verbs::cells::resolve_session_flag_env;
use crate::verbs::decisions::scanners::SECRET_PATTERNS;
use crate::verbs::knowledge::{g_prelude, pre_json_scan, GPre};
use crate::verbs::mailbox;
use crate::verbs::reservations::{js_trim, keys_known, now_iso, parse_flags, FlagV};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

/// The whole status vocabulary, and the order a record travels it. `open` is
/// the hook's; the other three are the agent's.
pub(crate) const WORK_STATUSES: [&str; 4] = ["open", "active", "done", "dropped"];

/// Which record this call addresses. The two arms are the hook's two sinks.
enum Sink {
    Session { file: PathBuf, ctrl: PathBuf, id: String },
    Mailbox { file: PathBuf, job: String },
}

impl Sink {
    fn file(&self) -> &Path {
        match self {
            Sink::Session { file, .. } => file,
            Sink::Mailbox { file, .. } => file,
        }
    }

    fn label(&self) -> String {
        match self {
            Sink::Session { id, .. } => format!("session {id}"),
            Sink::Mailbox { job, .. } => format!("job {job}"),
        }
    }
}

fn session_file(ctrl: &Path, id: &str) -> PathBuf {
    ctrl.join(".bee").join("sessions").join(format!("{id}.json"))
}

fn mailbox_file(ctrl: &Path, job: &str) -> PathBuf {
    ctrl.join(".bee").join("mailbox").join(job).join("activity.json")
}

fn env_id(key: &str) -> Option<String> {
    std::env::var(key).ok().map(|v| v.trim().to_string()).filter(|v| well_formed_id(v))
}

fn resolve_sink(root: &Path, explicit: Option<&str>) -> Result<Sink, String> {
    let ctrl = crate::hooks::session_init::control_root_for(root);
    if let Some(raw) = explicit {
        let id = js_trim(raw).to_string();
        if !well_formed_id(&id) {
            return Err(format!(
                "bee work: --session {id:?} is not a plain id (no path separators, never empty). FIX: pass the session id bee recorded, e.g. the one `bee state session list` prints."
            ));
        }
        let file = session_file(&ctrl, &id);
        return Ok(Sink::Session { file, ctrl, id });
    }
    if crate::hooks::herding_worker_marker_set() {
        if let Some(job) = env_id("BEE_HERDING_JOB_ID") {
            let file = mailbox_file(&ctrl, &job);
            return Ok(Sink::Mailbox { file, job });
        }
    }
    match env_id("CLAUDE_CODE_SESSION_ID") {
        Some(id) => {
            let file = session_file(&ctrl, &id);
            Ok(Sink::Session { file, ctrl, id })
        }
        None => Err(
            "bee work: no session to address — nothing named one and CLAUDE_CODE_SESSION_ID is unset or unusable. FIX: pass --session <id>."
                .to_string(),
        ),
    }
}

fn read_record(file: &Path) -> Map<String, Value> {
    match read_json(file) {
        ReadJson::Parsed(Value::Object(m)) => m,
        _ => Map::new(),
    }
}

fn work_of(record: &Map<String, Value>) -> Option<Map<String, Value>> {
    record.get("work").and_then(Value::as_object).cloned()
}

/// D5 over the agent's own sentence.
fn secret_shaped(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    SECRET_PATTERNS.iter().any(|(matcher, _)| matcher(&chars))
}

fn describe(work: &Map<String, Value>) -> String {
    let title = work.get("title").and_then(Value::as_str).unwrap_or("(untitled)");
    let status = work.get("status").and_then(Value::as_str).unwrap_or("open");
    let turns = work.get("turns").and_then(Value::as_u64).unwrap_or(0);
    let mut line = format!("work {title:?} — {status}, {turns} turn(s)");
    match work.get("acceptance").and_then(Value::as_str) {
        Some(acceptance) => line.push_str(&format!("\nacceptance: {acceptance}")),
        None => line.push_str("\nacceptance: none yet — `bee work set --acceptance` promotes it"),
    }
    line
}

// ── the two shapes ──────────────────────────────────────────────────────────

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "work" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    if verb != "show" && verb != "set" {
        return None;
    }
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    let known: &[&str] = match verb {
        "show" => &["session"],
        _ => &["session", "acceptance", "status"],
    };
    if !keys_known(&flags, known) {
        return None;
    }
    let str_flag = |name: &str| match flags.get(name) {
        Some(FlagV::S(s)) => Some(s.clone()),
        _ => None,
    };

    let cmd: &'static str = if verb == "show" { "work show" } else { "work set" };
    let ctx = match g_prelude(cmd, json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };

    let session_flag = str_flag("session");
    let sink = match resolve_sink(&ctx.root, session_flag.as_deref()) {
        Ok(sink) => sink,
        Err(message) => return Some(ctx.fail(&message)),
    };

    if verb == "show" {
        // intent show's precedent: no record is an empty result, never an
        // error — a repo that has never opened one behaves as it always did.
        let work = work_of(&read_record(sink.file()));
        let mut result = Map::new();
        result.insert("target".into(), Value::String(sink.label()));
        result.insert(
            "work".into(),
            work.clone().map(Value::Object).unwrap_or(Value::Null),
        );
        let text = match &work {
            Some(work) => describe(work),
            None => format!("no work record for {} — a prompt opens one", sink.label()),
        };
        return Some(ctx.emit(&Value::Object(result), &text, 0));
    }

    Some(run_set(&ctx, &sink, str_flag("acceptance"), str_flag("status"), session_flag))
}

fn run_set(
    ctx: &crate::verbs::knowledge::GCtx,
    sink: &Sink,
    acceptance: Option<String>,
    status: Option<String>,
    session_flag: Option<String>,
) -> ExitCode {
    let acceptance = acceptance.map(|a| js_trim(&a).to_string());
    let status = status.map(|s| js_trim(&s).to_string());
    if acceptance.is_none() && status.is_none() {
        return ctx.fail(
            "bee work set: nothing to set — pass --acceptance \"<what done means>\", --status <open|active|done|dropped>, or both.",
        );
    }
    if let Some(acceptance) = &acceptance {
        if acceptance.is_empty() {
            return ctx.fail(
                "bee work set: --acceptance is empty — say what finishing this ask would look like, or leave the flag off.",
            );
        }
        if secret_shaped(acceptance) {
            return ctx.fail(
                "bee work set: --acceptance matches a secret pattern and was NOT stored. An acceptance is your own sentence — describe the outcome without the credential.",
            );
        }
    }
    if let Some(status) = &status {
        if !WORK_STATUSES.contains(&&**status) {
            return ctx.fail(&format!(
                "bee work set: --status {status:?} is not a work status. Legal: {}.",
                WORK_STATUSES.join(", ")
            ));
        }
    }

    let apply = |record: &mut Map<String, Value>| -> Result<Map<String, Value>, String> {
        let Some(mut work) = work_of(record) else {
            return Err(format!(
                "bee work set: no work record for {} — a prompt opens one, so there is nothing to upgrade yet.",
                sink.label()
            ));
        };
        if let Some(acceptance) = &acceptance {
            work.insert("acceptance".into(), Value::String(acceptance.clone()));
        }
        if let Some(status) = &status {
            work.insert("status".into(), Value::String(status.clone()));
        }
        work.insert("updated_at".into(), Value::String(now_iso()));
        record.insert("work".into(), Value::Object(work.clone()));
        Ok(work)
    };

    let written = match sink {
        // The hook writes this file under the sessions lock and re-reads
        // inside it; a second writer that skipped the lock would lose a
        // concurrent heartbeat.
        Sink::Session { file, ctrl, .. } => match acquire_store_lock_once(ctrl, "sessions") {
            AcquireOnce::Busy { .. } => {
                return ctx.fail(
                    "bee work set: the sessions store is locked by another writer and nothing was written. FIX: run it again.",
                )
            }
            AcquireOnce::Acquired(mut guard) => {
                let outcome = (|| -> Result<Map<String, Value>, String> {
                    let mut record = read_record(file);
                    let work = apply(&mut record)?;
                    write_json_atomic(file, &Value::Object(record))
                        .map_err(|e| format!("bee work set: {} could not be written: {e}", file.display()))?;
                    Ok(work)
                })();
                guard.release();
                match outcome {
                    Ok(work) => work,
                    Err(message) => return ctx.fail(&message),
                }
            }
        },
        // The mailbox record has one writer — the agent inside this one pane,
        // which is also this process. The atomic rename is the whole story.
        Sink::Mailbox { file, .. } => {
            let mut record = read_record(file);
            match apply(&mut record) {
                Err(message) => return ctx.fail(&message),
                Ok(work) => {
                    if let Err(e) = write_json_atomic(file, &Value::Object(record)) {
                        return ctx
                            .fail(&format!("bee work set: {} could not be written: {e}", file.display()));
                    }
                    work
                }
            }
        }
    };

    // The record is on disk and the ask is over — see `file_letter_at_run_end`.
    file_letter_at_run_end(&ctx.root, session_flag.as_deref(), status.as_deref());

    let mut result = Map::new();
    result.insert("target".into(), Value::String(sink.label()));
    result.insert("work".into(), Value::Object(written.clone()));
    ctx.emit(&Value::Object(result), &describe(&written), 0)
}

// ── the human mailbox: the end of a run (hm-3) ─────────────────────────────
//
// D4 has two layers: every clean stop appends its raw entry the moment it
// happens (wired at the cap by hm-2, in `cells/handlers_close.rs`), and the
// letter is composed from those entries WHEN THE RUN ENDS. This is that second
// half. The composing itself, the five sections and the authorship ban all
// live in `verbs/mailbox.rs`; nothing about a letter's content is decided here.

/// The statuses that END a run's ask. `open` and `active` are mid-run and file
/// nothing; `dropped` counts as much as `done` — a run that abandoned its ask
/// still did work worth reading about, and the entries it left are the same
/// entries either way.
fn run_ended(status: &str) -> bool {
    status == "done" || status == "dropped"
}

/// D4/D9/D11: at the end of an armed run, compose the run's stored entries
/// into ONE letter and file it.
///
/// WHICH MOMENT. A run is a SESSION's span, not a herding job (`mailbox::
/// run_id`) — one unattended night dispatches many jobs, and a letter per job
/// would shatter the night D11 exists to keep whole. The moment this group can
/// see the span end is the work record reaching a terminal status, so that is
/// the hook. A run that ends more than once re-composes its one letter in
/// place; `mailbox::file_run_letter` owns that rule, not this call site.
///
/// WHICH RUN. The session id, through `resolve_session_flag_env` — the SAME
/// chain (`--session`, then `BEE_SESSION_ID`, then `CLAUDE_CODE_SESSION_ID`)
/// the cap used to name the run it appended under. A second, nearly-identical
/// resolution here would silently compose an empty letter the day the two
/// drifted. Deliberately NOT `Sink`: a herded pane addresses its job mailbox,
/// and the job is not the run.
///
/// WHICH ROOT. The control root — the main checkout for a linked worktree.
/// `cells finish` resolves its store root to `StoreRoots::main_root()` before
/// the cap appends, so the entries this letter is composed from are there and
/// not under a worktree. From the main checkout the two are the same path.
///
/// FAIL-OPEN: `mailbox::record_run_end` warns and returns. Setting a work
/// record's status is the caller's actual ask; a mailbox that cannot be
/// written must not turn it into a refusal.
fn file_letter_at_run_end(root: &Path, session_flag: Option<&str>, status: Option<&str>) {
    if !status.is_some_and(run_ended) {
        return;
    }
    let control = crate::hooks::session_init::control_root_for(root);
    let run = mailbox::run_id(resolve_session_flag_env(session_flag).as_deref());
    mailbox::record_run_end(&control, &run);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::mailbox::{
        append_entry, list_letter_files, read_letter, Entry, KIND_CAP, STATUS_UNREAD,
    };

    fn arm(root: &Path) {
        std::fs::create_dir_all(root.join(".bee").join("tmp")).unwrap();
        std::fs::write(
            root.join(".bee").join("config.json"),
            r#"{"herding": {"agent_command": "claude-sonnet"}}"#,
        )
        .unwrap();
        std::fs::write(root.join(".bee").join("tmp").join("bee-herding.enable"), "").unwrap();
    }

    fn entry(at: &str, what: &str) -> Entry {
        Entry {
            at: at.to_string(),
            kind: KIND_CAP.to_string(),
            what: what.to_string(),
            files: vec!["x.rs".to_string()],
            commit: Some("deadbee".to_string()),
            proof: Some("cargo test — green — one module".to_string()),
            departure: None,
            needs_you: vec![],
        }
    }

    #[test]
    fn a_terminal_status_is_what_ends_a_run_for_the_mailbox() {
        assert!(run_ended("done"));
        assert!(run_ended("dropped"));
        assert!(!run_ended("open"));
        assert!(!run_ended("active"));
    }

    #[test]
    fn an_armed_run_that_ends_files_its_mailbox_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        append_entry(root, "sess-9", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start")).unwrap();
        append_entry(root, "sess-9", &entry("2026-08-25T02:00:00.000Z", "Wrote down what changed")).unwrap();

        file_letter_at_run_end(root, Some("sess-9"), Some("done"));

        let letters = list_letter_files(root);
        assert_eq!(letters.len(), 1, "one run, one letter");
        let letter = read_letter(&letters[0]).unwrap();
        assert_eq!(letter.run, "sess-9");
        assert_eq!(letter.status, STATUS_UNREAD);
        assert_eq!(letter.items.len(), 2);
        assert_eq!(letter.subject, "Fixed the slow start");
    }

    #[test]
    fn a_run_still_going_writes_no_mailbox_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        append_entry(root, "sess-9", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start")).unwrap();
        for status in [None, Some("open"), Some("active")] {
            file_letter_at_run_end(root, Some("sess-9"), status);
        }
        assert!(list_letter_files(root).is_empty());
    }

    #[test]
    fn an_attended_run_that_ends_writes_no_mailbox_letter() {
        // D9: the owner never armed the loop, so this session is attended.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(
            root.join(".bee").join("config.json"),
            r#"{"herding": {"agent_command": "claude-sonnet"}}"#,
        )
        .unwrap();
        append_entry(root, "sess-9", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start")).unwrap();
        file_letter_at_run_end(root, Some("sess-9"), Some("done"));
        assert!(list_letter_files(root).is_empty());
    }

    #[test]
    fn two_runs_in_one_night_each_end_with_their_own_mailbox_letter() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        append_entry(root, "sess-a", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start")).unwrap();
        append_entry(root, "sess-b", &entry("2026-08-25T05:00:00.000Z", "Wrote down what changed")).unwrap();
        file_letter_at_run_end(root, Some("sess-a"), Some("done"));
        file_letter_at_run_end(root, Some("sess-b"), Some("dropped"));
        assert_eq!(list_letter_files(root).len(), 2, "one letter per run, never one per night (D11)");
    }
}
