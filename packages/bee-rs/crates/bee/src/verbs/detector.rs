use super::feedback::{emit_success, js_trim, now_iso, random_uuid_v4};
use super::supervisor::{
    control_root_path, normalise_point_key, one_line, read_interventions, read_observations,
    supervisor_dir, MAX_POINT_KEY_CHARS,
};
use crate::fsutil::{append_jsonl, read_json, warn_corrupt_jsonl_line, ReadJson};
use crate::herding::mailbox;
use crate::registry::check_manifest_drift;
use crate::roots::{resolve_store_root_worktree, RootsWt};
use crate::verbs::worktree::run_git;
use crate::verbs::{emit_no_root_error, emit_unsupported_root};
use serde_json::{json, Map, Value};
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
        "tick" => run_tick(parse_tick_args(rest)?, t0),
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

pub(crate) const BUDGET_THRESHOLD: f64 = 0.8;

const TOKENS_PER_JOB_KEY: &str = "tokens_per_job";

fn detector_state_dir(control: &Path) -> PathBuf {
    supervisor_dir(control).join("detector")
}

fn state_path(control: &Path, job_id: &str) -> PathBuf {
    detector_state_dir(control).join(format!("{job_id}.json"))
}

pub(crate) fn head_sha(cwd: &Path) -> Option<String> {
    let out = run_git(cwd, &["rev-parse", "HEAD"]);
    if out.status != Some(0) {
        return None;
    }
    let sha = js_trim(out.stdout.as_deref().unwrap_or("")).to_string();
    (!sha.is_empty()).then_some(sha)
}

fn is_checkout(cwd: &Path) -> bool {
    if !cwd.is_dir() {
        return false;
    }
    let out = run_git(cwd, &["rev-parse", "--is-inside-work-tree"]);
    out.status == Some(0) && js_trim(out.stdout.as_deref().unwrap_or("")) == "true"
}

pub(crate) fn excluded_path(path: &str) -> bool {
    path == ".bee"
        || path.starts_with(".bee/")
        || path == "docs/decisions"
        || path.starts_with("docs/decisions/")
}

pub(crate) fn looks_like_test_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized.split('/').collect();
    if segments.iter().any(|s| *s == "tests" || *s == "spec") {
        return true;
    }
    let name = segments.last().copied().unwrap_or("");
    name.contains("_test.") || name.contains(".test.") || name.contains("_tests.")
}

pub(crate) fn path_key(path: &str) -> String {
    use sha2::{Digest, Sha256};
    let hex = format!("{:x}", Sha256::digest(path.as_bytes()));
    hex.chars().take(8).collect()
}

fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut pending_hyphen = false;
    for c in text.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(c);
        } else {
            pending_hyphen = true;
        }
    }
    out
}

fn collect_paths(raw: &str, out: &mut Vec<String>) {
    for line in raw.lines() {
        let path = js_trim(line).replace('\\', "/");
        if path.is_empty() || excluded_path(&path) || out.iter().any(|p| *p == path) {
            continue;
        }
        out.push(path);
    }
}

pub(crate) fn changed_paths(cwd: &Path, base_sha: &str) -> Option<Vec<String>> {
    let diff = run_git(cwd, &["diff", "--name-only", base_sha]);
    if diff.status != Some(0) {
        return None;
    }
    let untracked = run_git(cwd, &["ls-files", "--others", "--exclude-standard"]);
    if untracked.status != Some(0) {
        return None;
    }
    let mut out = Vec::new();
    collect_paths(diff.stdout.as_deref().unwrap_or(""), &mut out);
    collect_paths(untracked.stdout.as_deref().unwrap_or(""), &mut out);
    out.sort();
    Some(out)
}

#[derive(Debug, Default)]
pub(crate) struct JobState {
    pub(crate) base_sha: Option<String>,
    pub(crate) changed_paths: Vec<String>,
}

fn read_state(control: &Path, job_id: &str) -> JobState {
    match read_json(&state_path(control, job_id)) {
        ReadJson::Parsed(Value::Object(m)) => JobState {
            base_sha: m
                .get("base_sha")
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.is_empty()),
            changed_paths: m
                .get("changed_paths")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default(),
        },
        _ => JobState::default(),
    }
}

fn write_state(control: &Path, job_id: &str, base_sha: &str, changed: &[String]) {
    let dir = detector_state_dir(control);
    if std::fs::create_dir_all(&dir).is_err() {
        return;
    }
    let row = json!({ "base_sha": base_sha, "changed_paths": changed, "at": now_iso() });
    let _ = std::fs::write(state_path(control, job_id), format!("{row}\n"));
}

pub(crate) struct ContractReads {
    pub(crate) known_ids: Vec<String>,
    pub(crate) active: Vec<(String, Value)>,
    pub(crate) active_ids: Vec<String>,
    pub(crate) open_trigger_keys: Vec<String>,
}

fn load_contract_reads(control: &Path) -> Option<ContractReads> {
    let known_ids: Vec<String> = crate::verbs::decisions::decision_target_candidates(control)
        .into_iter()
        .map(|(id, _)| id)
        .collect();
    let active = crate::verbs::decisions::active_decide_or_supersede_candidates(control).ok()?;
    let active_ids: Vec<String> = active.iter().map(|(id, _)| id.clone()).collect();
    let open_trigger_keys = crate::verbs::decisions::open_trigger_decision_keys(control);
    Some(ContractReads { known_ids, active, active_ids, open_trigger_keys })
}

pub(crate) fn contract_names(event: &Value) -> Vec<String> {
    let Some(Value::Array(tags)) = event.get("tags") else {
        return Vec::new();
    };
    tags.iter()
        .filter_map(|t| t.as_str())
        .filter_map(|t| js_trim(t).split_once(':'))
        .filter(|(ns, name)| ns.eq_ignore_ascii_case("contract") && !name.is_empty())
        .map(|(_, name)| name.to_string())
        .collect()
}

pub(crate) struct JobSnapshot {
    pub(crate) job_id: String,
    pub(crate) cell_id: Option<String>,
    pub(crate) target_session: Option<String>,
    pub(crate) job: Map<String, Value>,
    pub(crate) cell: Option<Map<String, Value>>,
    pub(crate) changed: Vec<String>,
}

impl JobSnapshot {
    fn event(&self, signal: &str, point_key: &str, excerpt: &str, source: &str) -> Event {
        Event::new(
            signal,
            &self.job_id,
            self.cell_id.as_deref(),
            self.target_session.as_deref(),
            point_key,
            excerpt,
            source,
        )
    }
}

fn parse_ms(value: Option<&Value>) -> Option<i64> {
    value
        .and_then(Value::as_str)
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis())
}

pub(crate) fn rule_budget(snap: &JobSnapshot, tokens_per_job: Option<f64>, now_ms: i64) -> Vec<Event> {
    let mut out = Vec::new();
    let job = &snap.job;
    let started = parse_ms(job.get("continued_at")).or_else(|| parse_ms(job.get("created_at")));
    let ceiling = job.get("ceiling_secs").and_then(Value::as_f64).unwrap_or(0.0);
    if let Some(started) = started {
        let elapsed = (now_ms - started).max(0) as f64 / 1000.0;
        if ceiling > 0.0 && elapsed >= BUDGET_THRESHOLD * ceiling {
            out.push(snap.event(
                "budget-80",
                &format!("budget-80-ceiling-{}", snap.job_id),
                &format!(
                    "The job has used {:.0}s of its {:.0}s ceiling ({:.0}%).",
                    elapsed,
                    ceiling,
                    elapsed / ceiling * 100.0
                ),
                "job.json",
            ));
        }
    }
    if let Some(cell) = &snap.cell {
        let budgets = crate::verbs::cells::resolve_cell_budgets(cell);
        if let Ok(attempts) = crate::verbs::cells::attempts_since_budget_reset(cell) {
            let used = attempts.len() as f64 + 1.0;
            if budgets.max_claims > 0.0 && used >= BUDGET_THRESHOLD * budgets.max_claims {
                out.push(snap.event(
                    "budget-80",
                    &format!("budget-80-claims-{}", snap.job_id),
                    &format!(
                        "Claim {:.0} of {:.0} on cell {}.",
                        used,
                        budgets.max_claims,
                        snap.cell_id.as_deref().unwrap_or("-")
                    ),
                    "cell",
                ));
            }
        }
    }
    if let Some(budget) = tokens_per_job.filter(|b| *b > 0.0) {
        let total = job
            .get("usage")
            .and_then(Value::as_object)
            .and_then(|u| u.get("total_tokens"))
            .and_then(Value::as_f64);
        if let Some(total) = total {
            if total >= BUDGET_THRESHOLD * budget {
                out.push(snap.event(
                    "budget-80",
                    &format!("budget-80-tokens-{}", snap.job_id),
                    &format!(
                        "The last finished round used {:.0} of the {:.0} tokens per job.",
                        total, budget
                    ),
                    "config",
                ));
            }
        }
    }
    out
}

pub(crate) fn rule_contract(snap: &JobSnapshot, reads: &ContractReads) -> Vec<Event> {
    let mut out = Vec::new();
    let Some(cell) = &snap.cell else { return out };
    let Some(test_path) = snap.changed.iter().find(|p| looks_like_test_path(p)) else {
        return out;
    };
    let entries: Vec<String> = match cell.get("decisions") {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect(),
        _ => Vec::new(),
    };
    for entry in entries {
        let Some(id) = crate::verbs::decisions::resolve_store_citation(&reads.known_ids, &entry)
        else {
            continue;
        };
        let status = crate::verbs::decisions::contract_status_over(
            &reads.active_ids,
            &reads.open_trigger_keys,
            &id,
        );
        if !matches!(status, crate::verbs::decisions::ContractStatus::Unsettled) {
            continue;
        }
        let Some((_, event)) = reads.active.iter().find(|(k, _)| *k == id) else { continue };
        for name in contract_names(event) {
            let short = crate::textutil::truncate_chars_head(&id, 8);
            out.push(snap.event(
                "test-on-unstable-contract",
                &format!("unstable-contract-{}-{}", slug(&name), snap.job_id),
                &format!(
                    "{test_path} changed while contract {name} (decision {short}) is still unsettled."
                ),
                "cell",
            ));
        }
    }
    out
}

pub(crate) fn rule_boundary(snap: &JobSnapshot) -> Vec<Event> {
    let mut out = Vec::new();
    let Some(cell) = &snap.cell else { return out };
    let Some(Value::Array(files)) = cell.get("files") else { return out };
    if files.is_empty() {
        return out;
    }
    for path in &snap.changed {
        if crate::verbs::cells::declared_covers(files, path) {
            continue;
        }
        out.push(snap.event(
            "boundary-approach",
            &format!("boundary-approach-{}-{}", snap.job_id, path_key(path)),
            &format!("{path} changed outside the cell's declared files."),
            "diff",
        ));
    }
    out
}

pub(crate) fn rule_self_correction(snap: &JobSnapshot, previous: &[String]) -> Vec<Event> {
    previous
        .iter()
        .filter(|p| !snap.changed.contains(p))
        .map(|path| {
            snap.event(
                "self-correction",
                &format!("self-correction-{}-{}", snap.job_id, path_key(path)),
                &format!("{path} left the job's changed set: it was changed earlier and now matches the base again."),
                "diff",
            )
        })
        .collect()
}

pub(crate) struct TickOutcome {
    pub(crate) fired: Vec<Event>,
    pub(crate) scanned: usize,
}

fn warn_job(job_id: &str, what: &str) {
    eprintln!("detector: job {job_id}: {what}; skipped");
}

fn tokens_per_job(control: &Path) -> Option<f64> {
    match read_json(&control.join(".bee").join("config.json")) {
        ReadJson::Parsed(cfg) => cfg
            .get("herding")
            .and_then(|h| h.get("budget"))
            .and_then(|b| b.get(TOKENS_PER_JOB_KEY))
            .and_then(Value::as_f64)
            .filter(|v| *v > 0.0),
        _ => None,
    }
}

fn mailbox_job_ids(bee_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(bee_dir.join("mailbox")) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = entries
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().to_str().map(str::to_string))
        .collect();
    ids.sort();
    ids
}

pub(crate) fn tick_core(control: &Path) -> TickOutcome {
    let bee_dir = control.join(".bee");
    let now_ms = chrono::Utc::now().timestamp_millis();
    let tokens = tokens_per_job(control);
    let reads = load_contract_reads(control);
    let mut seen: HashSet<(String, String, String)> = read_events(control)
        .into_iter()
        .map(|e| (e.job_id, e.signal, e.point_key))
        .collect();
    let mut fired = Vec::new();
    let mut scanned = 0usize;
    for job_id in mailbox_job_ids(&bee_dir) {
        let job = match read_json(&mailbox::job_path(&bee_dir, &job_id)) {
            ReadJson::Parsed(Value::Object(m)) => m,
            ReadJson::Missing => continue,
            _ => {
                warn_job(&job_id, "job.json is unreadable");
                continue;
            }
        };
        let round = job
            .get("round")
            .and_then(Value::as_u64)
            .and_then(|r| u32::try_from(r).ok())
            .unwrap_or(1);
        if mailbox::result_path(&bee_dir, &job_id, round).exists() {
            continue;
        }
        let cwd = PathBuf::from(js_trim(job.get("cwd").and_then(Value::as_str).unwrap_or("")));
        if cwd.as_os_str().is_empty() || !is_checkout(&cwd) {
            warn_job(&job_id, "its cwd is missing or not a checkout");
            continue;
        }
        scanned += 1;
        let cell_id = job
            .get("cell_id")
            .and_then(Value::as_str)
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let cell = cell_id
            .as_deref()
            .and_then(|id| crate::verbs::cells::read_cell(control, id).ok().flatten())
            .and_then(|v| v.as_object().cloned());
        let state = read_state(control, &job_id);
        let base = job
            .get("base_sha")
            .and_then(Value::as_str)
            .map(js_trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .or(state.base_sha.clone())
            .or_else(|| head_sha(&cwd));
        let Some(base) = base else {
            warn_job(&job_id, "its base commit cannot be read");
            continue;
        };
        let Some(changed) = changed_paths(&cwd, &base) else {
            warn_job(&job_id, &format!("the diff against base {base} failed"));
            continue;
        };
        let snap = JobSnapshot {
            job_id: job_id.clone(),
            cell_id,
            target_session: job
                .get("leader_session")
                .and_then(Value::as_str)
                .map(str::to_string)
                .filter(|s| !s.is_empty()),
            job,
            cell,
            changed,
        };
        let mut candidates = rule_budget(&snap, tokens, now_ms);
        if let Some(reads) = &reads {
            candidates.extend(rule_contract(&snap, reads));
        }
        candidates.extend(rule_boundary(&snap));
        candidates.extend(rule_self_correction(&snap, &state.changed_paths));
        for event in candidates {
            let key = (event.job_id.clone(), event.signal.clone(), event.point_key.clone());
            if !seen.insert(key) {
                continue;
            }
            match append_event(control, &event) {
                Ok(()) => fired.push(event),
                Err(why) => eprintln!("detector: job {job_id}: {why}"),
            }
        }
        write_state(control, &job_id, &base, &snap.changed);
    }
    TickOutcome { fired, scanned }
}

pub(crate) fn tick_quiet(control: &Path) -> usize {
    tick_core(control).fired.len()
}

fn parse_tick_args(tokens: &[OsString]) -> Option<bool> {
    let mut json = false;
    for t in tokens {
        match t.to_str()? {
            "--json" => json = true,
            _ => return None,
        }
    }
    Some(json)
}

fn run_tick(use_json: bool, t0: Instant) -> Option<ExitCode> {
    let cmd = "detector tick";
    let Ok(cwd) = std::env::current_dir() else { return None };
    let root = match resolve_store_root_worktree(&cwd) {
        RootsWt::Go(r) => r.root,
        RootsWt::Unsupported(why) => {
            return Some(emit_unsupported_root(&cwd, cmd, use_json, t0, &why))
        }
        RootsWt::None => return Some(emit_no_root_error(&cwd, cmd, use_json, t0)),
    };
    let control = control_root_path(&root);
    let drift = check_manifest_drift(&root);
    let outcome = tick_core(&control);
    let mut lines: Vec<String> = outcome.fired.iter().map(Event::line).collect();
    lines.push(format!(
        "detector: scanned {} jobs, fired {}",
        outcome.scanned,
        outcome.fired.len()
    ));
    let result = json!({
        "fired": outcome.fired.iter().map(Event::to_value).collect::<Vec<_>>(),
        "scanned": outcome.scanned,
    });
    Some(emit_success(&root, cmd, use_json, &drift, &result, &lines.join("\n"), t0))
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

    fn git_ok(cwd: &Path, args: &[&str]) {
        let out = run_git(cwd, args);
        assert_eq!(out.status, Some(0), "{args:?}: {:?}", out.stderr);
    }

    fn commit_all(cwd: &Path, msg: &str) {
        git_ok(cwd, &["add", "-A"]);
        git_ok(
            cwd,
            &[
                "-c",
                "user.email=bee@example.com",
                "-c",
                "user.name=bee",
                "-c",
                "commit.gpgSign=false",
                "commit",
                "-q",
                "-m",
                msg,
            ],
        );
    }

    fn checkout_fixture(tmp: &Path, name: &str) -> PathBuf {
        let cwd = tmp.join(name);
        std::fs::create_dir_all(cwd.join("src/b")).unwrap();
        git_ok(&cwd, &["init", "-q"]);
        std::fs::write(cwd.join("src/a.rs"), "fn a() {}\n").unwrap();
        std::fs::write(cwd.join("src/b/c.rs"), "fn c() {}\n").unwrap();
        commit_all(&cwd, "base");
        cwd
    }

    fn write_job(control: &Path, job_id: &str, cwd: &Path, extra: Value) {
        let dir = control.join(".bee").join("mailbox").join(job_id);
        std::fs::create_dir_all(&dir).unwrap();
        let mut job = json!({
            "job_id": job_id,
            "cwd": cwd.display().to_string(),
            "round": 1,
            "ceiling_secs": 1800,
            "created_at": now_iso(),
            "cell_id": "cell-1",
            "leader_session": "sess-1",
        });
        if let (Some(base), Some(add)) = (job.as_object_mut(), extra.as_object()) {
            for (k, v) in add {
                base.insert(k.clone(), v.clone());
            }
        }
        std::fs::write(dir.join("job.json"), serde_json::to_string(&job).unwrap()).unwrap();
    }

    fn write_cell(control: &Path, files: Value, extra: Value) {
        let dir = control.join(".bee").join("cells");
        std::fs::create_dir_all(&dir).unwrap();
        let mut cell = json!({
            "id": "cell-1",
            "feature": "f",
            "title": "t",
            "status": "claimed",
            "files": files,
            "decisions": [],
            "trace": {"attempts": []},
        });
        if let (Some(base), Some(add)) = (cell.as_object_mut(), extra.as_object()) {
            for (k, v) in add {
                base.insert(k.clone(), v.clone());
            }
        }
        std::fs::write(dir.join("cell-1.json"), serde_json::to_string(&cell).unwrap()).unwrap();
    }

    fn snapshot(job: Value, cell: Option<Value>, changed: &[&str]) -> JobSnapshot {
        JobSnapshot {
            job_id: "job-1".to_string(),
            cell_id: Some("cell-1".to_string()),
            target_session: Some("sess-1".to_string()),
            job: job.as_object().cloned().unwrap(),
            cell: cell.and_then(|c| c.as_object().cloned()),
            changed: changed.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn ago(secs: i64) -> String {
        (chrono::Utc::now() - chrono::Duration::seconds(secs)).to_rfc3339()
    }

    #[test]
    fn budget_rule_fires_the_ceiling_once_it_reaches_eighty_percent() {
        let now = chrono::Utc::now().timestamp_millis();
        let hot = snapshot(json!({"ceiling_secs": 100, "created_at": ago(80)}), None, &[]);
        let events = rule_budget(&hot, None, now);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].signal, "budget-80");
        assert_eq!(events[0].point_key, "budget-80-ceiling-job-1");
        assert!(events[0].excerpt.contains("ceiling"), "{}", events[0].excerpt);

        let cold = snapshot(json!({"ceiling_secs": 100, "created_at": ago(10)}), None, &[]);
        assert!(rule_budget(&cold, None, now).is_empty());
    }

    #[test]
    fn budget_rule_measures_a_continued_job_from_continued_at() {
        let now = chrono::Utc::now().timestamp_millis();
        let snap = snapshot(
            json!({"ceiling_secs": 100, "created_at": ago(5000), "continued_at": ago(10)}),
            None,
            &[],
        );
        assert!(rule_budget(&snap, None, now).is_empty(), "the round clock restarted");
    }

    #[test]
    fn budget_rule_counts_claims_and_tokens_against_their_budgets() {
        let now = chrono::Utc::now().timestamp_millis();
        let cell = json!({
            "budgets": {"max_claims": 5},
            "trace": {"attempts": [{"claim_session": "s", "at": "2026-01-01T00:00:00Z"},
                                  {"claim_session": "s", "at": "2026-01-02T00:00:00Z"},
                                  {"claim_session": "s", "at": "2026-01-03T00:00:00Z"}]},
        });
        let snap = snapshot(
            json!({"ceiling_secs": 100, "created_at": ago(1), "usage": {"total_tokens": 900}}),
            Some(cell),
            &[],
        );
        let keys: Vec<String> =
            rule_budget(&snap, Some(1000.0), now).into_iter().map(|e| e.point_key).collect();
        assert_eq!(keys, vec!["budget-80-claims-job-1", "budget-80-tokens-job-1"]);

        let quiet = snapshot(
            json!({"ceiling_secs": 100, "created_at": ago(1), "usage": {"total_tokens": 900}}),
            None,
            &[],
        );
        assert!(rule_budget(&quiet, None, now).is_empty(), "no token budget, no token rule");
    }

    #[test]
    fn contract_rule_fires_only_for_an_unsettled_citation_with_a_test_path() {
        let decision = json!({"id": "deadbeefcafe", "tags": ["contract:events-row", "x"]});
        let reads = ContractReads {
            known_ids: vec!["deadbeefcafe".to_string()],
            active: vec![("deadbeefcafe".to_string(), decision)],
            active_ids: vec!["deadbeefcafe".to_string()],
            open_trigger_keys: vec!["deadbeef".to_string()],
        };
        let cell = json!({"decisions": ["deadbeefcafe"]});
        let snap = snapshot(json!({}), Some(cell.clone()), &["tests/foo_test.rs", "src/a.rs"]);
        let events = rule_contract(&snap, &reads);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].signal, "test-on-unstable-contract");
        assert_eq!(events[0].point_key, "unstable-contract-events-row-job-1");
        assert!(events[0].excerpt.contains("tests/foo_test.rs"), "{}", events[0].excerpt);

        let no_test = snapshot(json!({}), Some(cell.clone()), &["src/a.rs"]);
        assert!(rule_contract(&no_test, &reads).is_empty());

        let settled = ContractReads { open_trigger_keys: Vec::new(), ..reads };
        assert!(rule_contract(&snap, &settled).is_empty());
    }

    #[test]
    fn boundary_rule_uses_declared_covers_and_skips_an_empty_files_list() {
        let cell = json!({"files": ["src/a.rs", "src/b/*"]});
        let snap = snapshot(
            json!({}),
            Some(cell),
            &["src/a.rs", "src/b/c.rs", "src/b/d/e.rs", "src/other.rs"],
        );
        let keys: Vec<String> = rule_boundary(&snap).into_iter().map(|e| e.point_key).collect();
        assert_eq!(
            keys,
            vec![
                format!("boundary-approach-job-1-{}", path_key("src/b/d/e.rs")),
                format!("boundary-approach-job-1-{}", path_key("src/other.rs")),
            ]
        );
        let open = snapshot(json!({}), Some(json!({"files": []})), &["src/other.rs"]);
        assert!(rule_boundary(&open).is_empty());
    }

    #[test]
    fn self_correction_rule_fires_for_a_path_that_left_the_changed_set() {
        let snap = snapshot(json!({}), None, &["src/b/c.rs"]);
        let previous = vec!["src/a.rs".to_string(), "src/b/c.rs".to_string()];
        let events = rule_self_correction(&snap, &previous);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].signal, "self-correction");
        assert_eq!(events[0].point_key, format!("self-correction-job-1-{}", path_key("src/a.rs")));
    }

    #[test]
    fn helpers_exclude_bee_paths_and_spot_test_paths() {
        assert!(excluded_path(".bee/decisions.jsonl"));
        assert!(excluded_path("docs/decisions/taxonomy.json"));
        assert!(!excluded_path("src/bee.rs"));
        assert!(looks_like_test_path("tests/foo.rs"));
        assert!(looks_like_test_path("spec/x.js"));
        assert!(looks_like_test_path("src/foo_test.rs"));
        assert!(looks_like_test_path("src/foo.test.ts"));
        assert!(looks_like_test_path("src/foo_tests.rs"));
        assert!(!looks_like_test_path("src/testing.rs"));
        assert_eq!(path_key("src/a.rs").len(), 8);
        assert_eq!(slug("Events Row/v2"), "events-row-v2");
    }

    #[test]
    fn tick_fires_once_per_point_and_tracks_the_changed_set_across_ticks() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path().join("control");
        std::fs::create_dir_all(&control).unwrap();
        let cwd = checkout_fixture(tmp.path(), "wt");
        let base = head_sha(&cwd).unwrap();
        write_job(&control, "job-1", &cwd, json!({"base_sha": base, "created_at": ago(1700)}));
        write_cell(&control, json!(["src/a.rs", "src/b/*"]), json!({}));

        std::fs::write(cwd.join("src/a.rs"), "fn a() { changed }\n").unwrap();
        std::fs::write(cwd.join("src/other.rs"), "fn o() {}\n").unwrap();
        std::fs::create_dir_all(cwd.join(".bee")).unwrap();
        std::fs::write(cwd.join(".bee/decisions.jsonl"), "{}\n").unwrap();

        let first = tick_core(&control);
        assert_eq!(first.scanned, 1);
        let mut signals: Vec<(String, String)> =
            first.fired.iter().map(|e| (e.signal.clone(), e.point_key.clone())).collect();
        signals.sort();
        assert_eq!(
            signals,
            vec![
                (
                    "boundary-approach".to_string(),
                    format!("boundary-approach-job-1-{}", path_key("src/other.rs"))
                ),
                ("budget-80".to_string(), "budget-80-ceiling-job-1".to_string()),
            ],
            "the untracked undeclared file fires, the declared edit and the .bee path do not"
        );
        assert_eq!(read_events(&control).len(), 2);
        assert_eq!(read_state(&control, "job-1").changed_paths, vec!["src/a.rs", "src/other.rs"]);

        let second = tick_core(&control);
        assert!(second.fired.is_empty(), "every point fires once ever");
        assert_eq!(read_events(&control).len(), 2);

        commit_all(&cwd, "mid-round commit");
        let third = tick_core(&control);
        assert!(third.fired.is_empty(), "a commit keeps the diff against base_sha intact");

        std::fs::write(cwd.join("src/a.rs"), "fn a() {}\n").unwrap();
        let fourth = tick_core(&control);
        assert_eq!(fourth.fired.len(), 1);
        assert_eq!(fourth.fired[0].signal, "self-correction");
        assert_eq!(
            fourth.fired[0].point_key,
            format!("self-correction-job-1-{}", path_key("src/a.rs"))
        );
    }

    #[test]
    fn tick_skips_a_finished_job_an_unreadable_job_and_a_cwd_that_is_no_checkout() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path().join("control");
        std::fs::create_dir_all(&control).unwrap();
        let cwd = checkout_fixture(tmp.path(), "wt");
        let base = head_sha(&cwd).unwrap();
        write_job(&control, "job-done", &cwd, json!({"base_sha": base, "created_at": ago(1700)}));
        std::fs::write(
            control.join(".bee/mailbox/job-done/result-1.json"),
            r#"{"status":"done"}"#,
        )
        .unwrap();
        let plain = tmp.path().join("plain");
        std::fs::create_dir_all(&plain).unwrap();
        write_job(&control, "job-plain", &plain, json!({"created_at": ago(1700)}));
        let broken = control.join(".bee/mailbox/job-broken");
        std::fs::create_dir_all(&broken).unwrap();
        std::fs::write(broken.join("job.json"), "{not json").unwrap();

        let outcome = tick_core(&control);
        assert_eq!(outcome.scanned, 0, "nothing live and readable was scanned");
        assert!(outcome.fired.is_empty());
        assert!(!events_path(&control).exists(), "no event store was created");
    }

    #[test]
    fn tick_captures_the_base_for_an_old_job_without_base_sha() {
        let tmp = tempfile::tempdir().unwrap();
        let control = tmp.path().join("control");
        std::fs::create_dir_all(&control).unwrap();
        let cwd = checkout_fixture(tmp.path(), "wt");
        write_job(&control, "job-old", &cwd, json!({"created_at": ago(1)}));
        write_cell(&control, json!(["src/a.rs"]), json!({}));
        std::fs::write(cwd.join("src/new.rs"), "fn n() {}\n").unwrap();

        let outcome = tick_core(&control);
        assert_eq!(outcome.scanned, 1);
        assert_eq!(outcome.fired.len(), 1, "only the untracked path can fire on the first tick");
        assert_eq!(outcome.fired[0].signal, "boundary-approach");
        let state = read_state(&control, "job-old");
        assert_eq!(state.base_sha, head_sha(&cwd));
        assert_eq!(state.changed_paths, vec!["src/new.rs"]);
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
