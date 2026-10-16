use super::feedback::{emit_success, js_trim, now_iso, random_uuid_v4};
use super::supervisor::{
    control_root_path, normalise_point_key, one_line, read_interventions, read_observations,
    supervisor_dir, MAX_POINT_KEY_CHARS,
};
use crate::fsutil::{append_jsonl, warn_corrupt_jsonl_line};
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

fn events_path(control: &Path) -> PathBuf {
    supervisor_dir(control).join("events.jsonl")
}

pub(crate) const EVENT_SIGNALS: [&str; 4] =
    ["self-correction", "boundary-approach", "test-on-unstable-contract", "budget-80"];

pub(crate) const MAX_EXCERPT_CHARS: usize = 300;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct Event {
    pub(crate) id: String,
    pub(crate) ts: String,
    pub(crate) signal: String,
    pub(crate) job_id: String,
    pub(crate) cell_id: Option<String>,
    pub(crate) target_session: Option<String>,
    pub(crate) point_key: String,
    pub(crate) excerpt: String,
    pub(crate) source: String,
}

impl Event {
    #[allow(dead_code)]
    pub(crate) fn new(
        signal: &str,
        job_id: &str,
        cell_id: Option<&str>,
        target_session: Option<&str>,
        point_key: &str,
        excerpt: &str,
        source: &str,
    ) -> Self {
        Self {
            id: random_uuid_v4(),
            ts: now_iso(),
            signal: signal.to_string(),
            job_id: job_id.to_string(),
            cell_id: cell_id.map(str::to_string),
            target_session: target_session.map(str::to_string),
            point_key: point_key.to_string(),
            excerpt: one_line(excerpt),
            source: source.to_string(),
        }
    }

    pub(crate) fn to_value(&self) -> Value {
        json!({
            "id": self.id,
            "ts": self.ts,
            "signal": self.signal,
            "job_id": self.job_id,
            "cell_id": self.cell_id,
            "target_session": self.target_session,
            "point_key": self.point_key,
            "excerpt": self.excerpt,
            "source": self.source,
        })
    }

    fn from_value(v: &Value) -> Option<Self> {
        let m = v.as_object()?;
        let signal = m.get("signal")?.as_str()?.to_string();
        if !EVENT_SIGNALS.contains(&signal.as_str()) {
            return None;
        }
        let id = m.get("id")?.as_str()?.to_string();
        if id.is_empty() {
            return None;
        }
        let ts = m.get("ts")?.as_str()?.to_string();
        let job_id = m.get("job_id")?.as_str()?.to_string();
        if job_id.is_empty() {
            return None;
        }
        let point_key = m.get("point_key")?.as_str()?.to_string();
        if point_key.is_empty() {
            return None;
        }
        let excerpt = m.get("excerpt")?.as_str()?.to_string();
        let source = m.get("source")?.as_str()?.to_string();
        let opt = |name: &str| {
            m.get(name).and_then(Value::as_str).map(str::to_string).filter(|s| !s.is_empty())
        };
        Some(Self {
            id,
            ts,
            signal,
            job_id,
            cell_id: opt("cell_id"),
            target_session: opt("target_session"),
            point_key,
            excerpt,
            source,
        })
    }

    pub(crate) fn line(&self) -> String {
        let cell = self.cell_id.as_deref().unwrap_or("-");
        format!(
            "- {} [{}] {} {} {} — {}",
            self.ts, self.signal, self.job_id, cell, self.point_key, self.excerpt
        )
    }
}

#[allow(dead_code)]
pub(crate) fn append_event(control: &Path, event: &Event) -> Result<(), String> {
    let cmd = "detector";
    if !EVENT_SIGNALS.contains(&event.signal.as_str()) {
        return Err(format!(
            "bee {cmd}: signal must be one of {}, got {:?}.",
            EVENT_SIGNALS.join(", "),
            event.signal
        ));
    }
    if normalise_point_key(cmd, Some(&event.point_key)).ok().as_deref()
        != Some(event.point_key.as_str())
    {
        return Err(format!(
            "bee {cmd}: point_key must be a slug of a-z, 0-9 and inner hyphens \
             ({MAX_POINT_KEY_CHARS} max), got {:?}.",
            event.point_key
        ));
    }
    let excerpt = one_line(&event.excerpt);
    if excerpt.is_empty() {
        return Err(format!("bee {cmd}: excerpt must not be empty."));
    }
    if excerpt.chars().count() > MAX_EXCERPT_CHARS {
        return Err(format!(
            "bee {cmd}: excerpt is {} characters; keep it to one sentence \
             ({MAX_EXCERPT_CHARS} max).",
            excerpt.chars().count()
        ));
    }
    let mut row = event.to_value();
    row["excerpt"] = Value::String(excerpt);
    if append_jsonl(&events_path(control), &row).is_err() {
        return Err(format!("bee {cmd}: could not append to the detector event store."));
    }
    Ok(())
}

pub(crate) fn read_events(control: &Path) -> Vec<Event> {
    let path = events_path(control);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut rows = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        if js_trim(raw).is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(raw).ok().as_ref().and_then(Event::from_value) {
            Some(rec) => rows.push(rec),
            None => warn_corrupt_jsonl_line(&path, i + 1),
        }
    }
    rows
}

#[allow(dead_code)]
pub(crate) fn events_for_job(control: &Path, job_id: &str) -> Vec<Event> {
    read_events(control).into_iter().filter(|e| e.job_id == job_id).collect()
}

pub(crate) fn judged_event_ids(control: &Path) -> HashSet<String> {
    let mut out = HashSet::new();
    for row in read_observations(control).rows {
        out.extend(row.events);
    }
    for row in read_interventions(control).rows {
        out.extend(row.events);
    }
    out
}

struct EventsArgs {
    unjudged: bool,
    json: bool,
    pre_json: bool,
}

fn parse_events_args(tokens: &[OsString]) -> Option<EventsArgs> {
    let mut out = EventsArgs { unjudged: false, json: false, pre_json: false };
    for t in tokens {
        match t.to_str()? {
            "--json" => {
                out.json = true;
                out.pre_json = true;
            }
            "--unjudged" => out.unjudged = true,
            _ => return None,
        }
    }
    Some(out)
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "detector" {
        return None;
    }
    let verb = args.get(1)?.to_str()?;
    let rest = &args[2..];
    match verb {
        "events" => run_events(parse_events_args(rest)?, t0),
        _ => None,
    }
}

fn run_events(parsed: EventsArgs, t0: Instant) -> Option<ExitCode> {
    let cmd = "detector events";
    let Ok(cwd) = std::env::current_dir() else { return None };
    let root = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r.root,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, parsed.pre_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, parsed.pre_json, t0)),
    };
    let control = control_root_path(&root);
    let drift = check_manifest_drift(&root);
    let rows = select_events(&control, parsed.unjudged);
    let text = events_text(&rows);
    let result = json!({ "events": rows.iter().map(Event::to_value).collect::<Vec<_>>() });
    Some(emit_success(&root, cmd, parsed.json, &drift, &result, &text, t0))
}

fn select_events(control: &Path, unjudged: bool) -> Vec<Event> {
    let rows = read_events(control);
    if !unjudged {
        return rows;
    }
    let judged = judged_event_ids(control);
    rows.into_iter().filter(|e| !judged.contains(&e.id)).collect()
}

fn events_text(rows: &[Event]) -> String {
    if rows.is_empty() {
        "No detector events.".to_string()
    } else {
        rows.iter().map(Event::line).collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plant(control: &Path, signal: &str, job: &str, point: &str, excerpt: &str) -> Event {
        let e = Event::new(signal, job, Some("cell-1"), Some("sess-1"), point, excerpt, "diff");
        append_event(control, &e).expect("a valid event appends");
        e
    }

    #[test]
    fn a_valid_event_appends_exactly_one_row_and_creates_the_store() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        assert!(!events_path(control).exists(), "fixture starts with no store");

        let e = Event::new(
            "budget-80",
            "job-1",
            Some("det-1"),
            Some("sess-1"),
            "budget-80-ceiling-job-1",
            "The job is at 80% of its ceiling.",
            "job.json",
        );
        append_event(control, &e).unwrap();

        assert!(supervisor_dir(control).is_dir(), "the store directory is created on first write");
        let rows = read_events(control);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].signal, "budget-80");
        assert_eq!(rows[0].job_id, "job-1");
        assert_eq!(rows[0].cell_id.as_deref(), Some("det-1"));
        assert_eq!(rows[0].point_key, "budget-80-ceiling-job-1");
        assert_eq!(rows[0].id, e.id);
        assert!(rows[0].ts.ends_with('Z'), "ts is RFC 3339 UTC: {}", rows[0].ts);
        let raw = std::fs::read_to_string(events_path(control)).unwrap();
        assert_eq!(raw.lines().count(), 1, "one event is one line");
    }

    #[test]
    fn every_refusal_names_its_field_and_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let mk = |signal: &str, point: &str, excerpt: &str| Event {
            id: random_uuid_v4(),
            ts: now_iso(),
            signal: signal.to_string(),
            job_id: "job-1".to_string(),
            cell_id: None,
            target_session: None,
            point_key: point.to_string(),
            excerpt: excerpt.to_string(),
            source: "diff".to_string(),
        };

        let err = append_event(control, &mk("vibes", "p", "x")).unwrap_err();
        assert!(err.contains("signal must be one of"), "{err}");
        assert!(err.contains("self-correction"), "the refusal names the set: {err}");
        let err = append_event(control, &mk("budget-80", "Not A Slug", "x")).unwrap_err();
        assert!(err.contains("point_key must be a slug"), "{err}");
        let err = append_event(control, &mk("budget-80", "p", "   ")).unwrap_err();
        assert!(err.contains("excerpt must not be empty"), "{err}");

        let long = "s".repeat(MAX_EXCERPT_CHARS + 1);
        let err = append_event(control, &mk("budget-80", "p", &long)).unwrap_err();
        assert!(err.contains("one sentence"), "{err}");

        assert!(!events_path(control).exists(), "no refusal ever wrote a row");
    }

    #[test]
    fn an_excerpt_is_collapsed_to_one_line() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        plant(control, "self-correction", "job-1", "p", "  two\nlines\tand\r\nspaces  ");
        let rows = read_events(control);
        assert_eq!(rows[0].excerpt, "two lines and spaces");
        assert_eq!(std::fs::read_to_string(events_path(control)).unwrap().lines().count(), 1);
    }

    #[test]
    fn a_malformed_line_is_skipped_and_the_rows_around_it_survive() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        plant(control, "budget-80", "job-1", "p-one", "first");
        let path = events_path(control);
        let mut text = std::fs::read_to_string(&path).unwrap();
        text.push_str("{\"ts\":\"2026-10-15T00:00:00.000Z\",\"sig\n");
        text.push_str("{\"ts\":\"2026-10-15T00:00:01.000Z\",\"signal\":\"vibes\",\"id\":\"a\",\"job_id\":\"j\",\"point_key\":\"p\",\"excerpt\":\"e\",\"source\":\"s\"}\n");
        std::fs::write(&path, text).unwrap();
        plant(control, "self-correction", "job-2", "p-two", "second");

        let rows = read_events(control);
        assert_eq!(rows.len(), 2, "the two good rows still read back");
        assert_eq!(rows[0].point_key, "p-one");
        assert_eq!(rows[1].point_key, "p-two");
    }

    #[test]
    fn events_for_job_keeps_only_that_jobs_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        plant(control, "budget-80", "job-1", "p-one", "first");
        plant(control, "boundary-approach", "job-2", "p-two", "second");
        plant(control, "self-correction", "job-1", "p-three", "third");

        let rows = events_for_job(control, "job-1");
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].point_key, "p-one");
        assert_eq!(rows[1].point_key, "p-three");
    }

    #[test]
    fn an_empty_or_missing_store_reads_as_empty() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        assert!(read_events(control).is_empty(), "a missing store reads as empty");
        assert_eq!(events_text(&select_events(control, false)), "No detector events.");
        assert_eq!(events_text(&select_events(control, true)), "No detector events.");
    }

    #[test]
    fn select_events_names_both_rows_and_unjudged_drops_what_a_row_judged() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let a = plant(control, "budget-80", "job-1", "p-one", "first");
        let b = plant(control, "self-correction", "job-1", "p-two", "second");

        let all = select_events(control, false);
        assert_eq!(all.len(), 2, "both rows, oldest first");
        assert_eq!(all[0].id, a.id);

        crate::verbs::supervisor::record_into(
            control,
            "supervisor record",
            Some("silence"),
            Some("none"),
            Some("Reviewed the budget event and chose silence."),
            None,
            None,
            Some(&a.id),
        )
        .unwrap();
        let unjudged = select_events(control, true);
        assert_eq!(unjudged.len(), 1, "the judged row leaves the unjudged list");
        assert_eq!(unjudged[0].id, b.id, "the other row stays");

        crate::verbs::supervisor::record_intervention_into(
            control,
            "supervisor record",
            "intervention",
            Some("self-correction"),
            Some("sess-1"),
            Some("self-correction-job-1-abcd1234"),
            Some("What would tell you the restore was deliberate?"),
            None,
            Some(&b.id),
        )
        .unwrap();
        assert!(select_events(control, true).is_empty(), "both rows are judged now");
        assert_eq!(select_events(control, false).len(), 2, "the rows still read back");
    }

    #[test]
    fn judged_event_ids_unions_both_record_stores() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        let a = plant(control, "budget-80", "job-1", "p-one", "first");
        let b = plant(control, "self-correction", "job-1", "p-two", "second");
        assert!(judged_event_ids(control).is_empty());

        crate::verbs::supervisor::record_into(
            control,
            "supervisor record",
            Some("observation"),
            Some("budget-80"),
            Some("Seen."),
            None,
            None,
            Some(&a.id),
        )
        .unwrap();
        let judged = judged_event_ids(control);
        assert!(judged.contains(&a.id));
        assert!(!judged.contains(&b.id));
    }

    #[test]
    fn a_row_line_carries_the_ts_signal_job_cell_point_and_excerpt() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path();
        plant(control, "budget-80", "job-1", "budget-80-ceiling-job-1", "The job is at 80%.");
        let line = &read_events(control)[0].line();
        assert!(line.starts_with("- "), "{line}");
        assert!(line.contains("[budget-80]"), "{line}");
        assert!(line.contains("job-1"), "{line}");
        assert!(line.contains("cell-1"), "{line}");
        assert!(line.contains("budget-80-ceiling-job-1"), "{line}");
        assert!(line.contains("The job is at 80%."), "{line}");
    }
}
