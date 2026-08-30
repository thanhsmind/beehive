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
//
// letter-digest (docs/history/letter-digest/CONTEXT.md D3/D4): and the moment a
// finished DAY or WEEK gets folded — see `compose_digests_for_finished_periods`
// at the foot of the file, and the composer in `verbs/mailbox_digest.rs`. Three
// hooks, one path, for one reason: `work set` is where this process first
// learns a session has taken up work.

use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::activity::well_formed_id;
use crate::lock::{acquire_store_lock_once, AcquireOnce};
use crate::verbs::cells::resolve_session_flag_env;
use crate::verbs::decisions::scanners::SECRET_PATTERNS;
use crate::verbs::knowledge::{g_prelude, pre_json_scan, GPre};
use crate::verbs::mailbox;
use crate::verbs::mailbox_digest;
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

    // D12 first: some EARLIER run may have died without ever reaching its own
    // end, and this session is the next one — see `file_letters_for_silent_runs`.
    file_letters_for_silent_runs(&ctx.root, session_flag.as_deref());
    // Then this run's own end. The record is on disk and the ask is over — see
    // `file_letter_at_run_end`.
    file_letter_at_run_end(&ctx.root, session_flag.as_deref(), status.as_deref());
    // Then the periods that ENDED while nobody was looking — see
    // `compose_digests_for_finished_periods`. BESIDE the two calls above, never
    // inside either: those two stop early when the mailbox is not armed, and a
    // digest is owed to an unarmed checkout just the same (D2 files a letter at
    // every close, attended or not).
    compose_digests_for_finished_periods(&ctx.root);

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

// ── the human mailbox: a run that went silent (hm-9, D12) ──────────────────
//
// D12: a run that dies WITHOUT reaching its own end gets its letter from THE
// NEXT SESSION THAT STARTS. No background scheduler is added — a scheduler
// shares the failure mode it would exist to cover, so the recovery rides a
// path a later session already walks. All of the detection, the never-sweep-a-
// live-run rule and the letter's own shape live in `verbs/mailbox.rs`; this is
// only the hook.
//
// WHY HERE. CONTEXT.md's Integration Points names this file — "session start
// and the herding surface … where D12's next-session filing hooks in" — and
// `work set` is the moment in it: the agent's first move on a new ask is to
// say what "done" means, so it is the earliest point at which this file learns
// a session has taken up work. It is also where the run's own END already
// lives, which keeps both edges of a run's span in one place.
//
// WHY IT IS CHEAP ENOUGH TO SIT ON THIS PATH. `work set` already takes the
// sessions lock and writes a JSON file. The recovery adds one config read when
// the mailbox is not armed — the ordinary case, and it stops there — and three
// directory listings when it is. It opens no entry file to decide anything;
// `mailbox::ENTRY_READS` holds that promise down in a test. A `work set` that
// runs more than once in a session repeats a no-op: the first pass gives every
// silent run its ONE letter (D11), and a run that has a letter is never a
// candidate again.
//
// FAIL-OPEN, like the run end beside it: `mailbox::record_silent_runs` warns
// and returns. Recovering some other run's letter must never turn the caller's
// own ask into a refusal.

/// D12's pass, hooked to the moment this session takes up work.
///
/// WHICH RUN is this one — the same `resolve_session_flag_env` chain the cap
/// and the run end use, so "not me" means exactly what it means everywhere
/// else. WHICH ROOT is the control root, for the same reason
/// `file_letter_at_run_end` uses it: that is where the entries a cap appended
/// actually are.
fn file_letters_for_silent_runs(root: &Path, session_flag: Option<&str>) {
    let control = crate::hooks::session_init::control_root_for(root);
    let run = mailbox::run_id(resolve_session_flag_env(session_flag).as_deref());
    mailbox::record_silent_runs(&control, &run);
}

// ── the digest of a finished period (letter-digest, D3/D4) ─────────────────
//
// D3: "the daily and weekly digest is composed by the next session that starts
// after the period ended and finds the digest missing — the same
// recover-on-next-session pattern as dead-run letters. No scheduler, no cron."
// This is that hook. Everything about WHICH periods are due, what a digest
// says, and the weekly fold's lesson mining lives in `verbs/mailbox_digest.rs`;
// this call site decides only WHEN, WHERE and HOW LOUDLY.
//
// WHY BESIDE THE TWO CALLS ABOVE, AND NOT INSIDE ONE OF THEM. Both mailbox
// hooks stop early when the loop is not armed (D9: only an unattended run files
// a run-end letter). A digest must not inherit that gate. D2 files a letter at
// every close, attended sessions included, so an unarmed checkout accumulates
// letters exactly like an armed one — and a checkout whose owner never arms
// anything would then be the one that never gets a digest of the letters it
// does have. Nesting the call inside either hook would hide that gate behind a
// function whose name says nothing about arming.
//
// WHY IT IS CHEAP ENOUGH TO SIT ON THIS PATH. The ordinary answer is "no period
// is due", reached from ONE directory listing of names — the same bounded read
// D12's recovery makes, with nothing opened to decide. A period whose digest is
// already on disk costs one `exists` call. Only the first session after a day
// or a week actually ends reads letters, and it reads them once, ever: the
// digest file it writes is the marker that stops the next session repeating it.
//
// FAIL-OPEN, like both hooks above: the composer warns on its own and returns
// what it wrote. Setting a work record's status is the caller's actual ask, and
// a digest that cannot be composed — or a lesson that cannot be logged — must
// never turn that ask into a refusal.

/// D3's pass, hooked to the moment this session takes up work.
///
/// WHICH ROOT is the control root, for the same reason `file_letter_at_run_end`
/// uses it: the letters a cap filed and the `.bee/usage/<feature>.json` records
/// a close wrote both live under the MAIN checkout, so a linked worktree that
/// resolved its own root would fold an empty period and file a digest saying
/// so.
///
/// WHICH MOMENT is now — a real clock read, because "has this period ended" is
/// a question about the wall the human is looking at. The composer takes the
/// stamp as an argument so a test can pin it; only this call site reads a
/// clock.
fn compose_digests_for_finished_periods(root: &Path) {
    let control = crate::hooks::session_init::control_root_for(root);
    mailbox_digest::compose_and_mine(&control, &now_iso());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::verbs::mailbox::{
        append_entry, list_letter_files, read_letter, Entry, KIND_CAP, STATUS_UNREAD,
        UNFINISHED_SUBJECT_MARK,
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

    // ── D12: the next session files the silent run's letter (hm-9) ─────────

    fn session_record(root: &Path, id: &str, status: &str, last_heartbeat: &str) {
        let dir = root.join(".bee").join("sessions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.json")),
            serde_json::json!({ "id": id, "status": status, "last_heartbeat": last_heartbeat })
                .to_string(),
        )
        .unwrap();
    }

    #[test]
    fn the_next_session_files_the_letter_of_a_run_that_went_silent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        // A run that died: entries on disk, no letter, and its session record
        // says the session is over.
        append_entry(root, "sess-dead", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start"))
            .unwrap();
        append_entry(root, "sess-dead", &entry("2026-08-25T02:30:00.000Z", "Wrote down what changed"))
            .unwrap();
        session_record(root, "sess-dead", "dead", "2026-08-25T02:31:00.000Z");
        // This session, alive and taking up work.
        session_record(root, "sess-now", "active", &now_iso());

        file_letters_for_silent_runs(root, Some("sess-now"));

        let letters = list_letter_files(root);
        assert_eq!(letters.len(), 1, "the silent run gets exactly one letter (D11, D12)");
        let letter = read_letter(&letters[0]).unwrap();
        assert_eq!(letter.run, "sess-dead");
        assert_eq!(letter.status, STATUS_UNREAD);
        assert!(
            letter.subject.starts_with(UNFINISHED_SUBJECT_MARK),
            "the subject {:?} does not mark the run unfinished",
            letter.subject
        );
        assert_eq!(letter.items.len(), 2, "the entries up to the last one");
        assert!(
            letter.body.contains("2026-08-25T02:30:00.000Z"),
            "the body never names the moment the run went silent: {}",
            letter.body
        );
    }

    // ── letter-digest D3: the hook that folds a finished period ────────────

    /// One letter on disk, filed at `stamp`, through the store's own writer.
    fn letter_at(root: &Path, run: &str, stamp: &str, subject: &str) {
        let letter = crate::verbs::mailbox::Letter {
            subject: subject.to_string(),
            run: run.to_string(),
            project: "beehive".to_string(),
            filed_at: stamp.to_string(),
            status: STATUS_UNREAD.to_string(),
            items: Vec::new(),
            needs_you: Vec::new(),
            body: "## Done\n\n- did the work\n".to_string(),
        };
        crate::verbs::mailbox::write_letter(root, &letter).unwrap();
    }

    #[test]
    fn a_session_taking_up_work_folds_a_period_that_ended_long_ago() {
        // Deliberately NOT armed: D2 files a letter at every close, attended
        // sessions included, so an unarmed checkout has letters to fold and
        // must get its digest like any other. This is the whole reason the
        // hook sits BESIDE the two mailbox calls rather than inside one.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        letter_at(root, "run-old", "2020-01-02T03:15:00.000Z", "The first run finished.");

        compose_digests_for_finished_periods(root);

        let dir = root.join(".bee").join("human-mailbox");
        assert!(dir.join("digest-2020-01-02.md").exists(), "no daily digest was left behind");
        assert!(dir.join("digest-2020-W01.md").exists(), "no weekly digest was left behind");
        assert!(
            list_letter_files(root).len() == 1,
            "a digest was counted as a letter: {:?}",
            list_letter_files(root)
        );
    }

    #[test]
    fn folding_a_period_is_a_no_op_when_nothing_has_finished() {
        // An empty mailbox is the ordinary case on this path, and it must cost
        // nothing and write nothing.
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        compose_digests_for_finished_periods(root);
        assert!(!root.join(".bee").join("human-mailbox").exists() || list_letter_files(root).is_empty());
        // Running it twice over a folded period leaves the same one file.
        letter_at(root, "run-old", "2020-01-02T03:15:00.000Z", "The first run finished.");
        compose_digests_for_finished_periods(root);
        let before = std::fs::read_to_string(
            root.join(".bee").join("human-mailbox").join("digest-2020-01-02.md"),
        )
        .unwrap();
        compose_digests_for_finished_periods(root);
        let after = std::fs::read_to_string(
            root.join(".bee").join("human-mailbox").join("digest-2020-01-02.md"),
        )
        .unwrap();
        assert_eq!(before, after, "the second pass rewrote the digest");
    }

    #[test]
    fn a_session_start_never_files_a_letter_for_a_run_still_working() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        append_entry(root, "sess-live", &entry("2026-08-25T01:00:00.000Z", "Still going")).unwrap();
        session_record(root, "sess-live", "active", &now_iso());
        session_record(root, "sess-now", "active", &now_iso());

        file_letters_for_silent_runs(root, Some("sess-now"));
        assert!(list_letter_files(root).is_empty(), "a run still working was reported as dead");
    }

    #[test]
    fn a_second_session_start_does_not_file_the_same_letter_again() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        arm(root);
        append_entry(root, "sess-dead", &entry("2026-08-25T01:00:00.000Z", "Fixed the slow start"))
            .unwrap();
        session_record(root, "sess-dead", "closed", "2026-08-25T01:05:00.000Z");

        file_letters_for_silent_runs(root, Some("sess-now"));
        file_letters_for_silent_runs(root, Some("sess-later"));
        assert_eq!(list_letter_files(root).len(), 1, "one run, one letter (D11)");
    }
}
