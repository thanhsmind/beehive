use crate::jsjson;
use crate::roots::{resolve_store_root_worktree, RootsWt, StoreRoots};
use crate::verbs::knowledge::{cwd_store_root, emit_no_root_error_at};
use crate::verbs::{emit_no_root_error, emit_unsupported_root, record_timing};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

const CMD: &str = "report outcomes";

struct Opts {
    feature: Option<String>,
    root: Option<PathBuf>,
    json: bool,
    by_feature: bool,
    since: Option<DateTime<Utc>>,
}

impl Opts {
    fn start(&self, cwd: &Path) -> PathBuf {
        match &self.root {
            Some(r) => cwd.join(r),
            None => cwd.to_path_buf(),
        }
    }
}

fn parse(tokens: &[OsString]) -> Option<Opts> {
    let toks: Vec<&str> = tokens.iter().map(|t| t.to_str()).collect::<Option<_>>()?;
    let mut feature = None;
    let mut root = None;
    let mut json = false;
    let mut by_feature = false;
    let mut since = None;
    let mut i = 0;
    while i < toks.len() {
        let tok = toks[i];
        if tok == "--json" {
            json = true;
            i += 1;
            continue;
        }
        if tok == "--by-feature" {
            by_feature = true;
            i += 1;
            continue;
        }
        if tok == "--since" {
            i += 1;
            since = Some(parse_since(toks.get(i)?, Utc::now())?);
            i += 1;
            continue;
        }
        if let Some(val) = tok.strip_prefix("--since=") {
            since = Some(parse_since(val, Utc::now())?);
            i += 1;
            continue;
        }
        if tok == "--feature" {
            i += 1;
            feature = Some(toks.get(i)?.to_string());
            i += 1;
            continue;
        }
        if let Some(val) = tok.strip_prefix("--feature=") {
            feature = Some(val.to_string());
            i += 1;
            continue;
        }
        if tok == "--root" {
            i += 1;
            root = Some(PathBuf::from(toks.get(i)?));
            i += 1;
            continue;
        }
        if let Some(val) = tok.strip_prefix("--root=") {
            root = Some(PathBuf::from(val));
            i += 1;
            continue;
        }
        return None;
    }
    Some(Opts { feature, root, json, by_feature, since })
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "report" || args.get(1)?.to_str()? != "outcomes" {
        return None;
    }
    let opts = parse(&args[2..])?;
    Some(run(&opts, t0))
}

fn run(opts: &Opts, t0: Instant) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(c) => c,
        Err(e) => return fail(Path::new("."), opts.json, "bad_cwd", format!("bee {CMD}: {e}"), t0),
    };
    let roots = match resolve_store_root_worktree(&opts.start(&cwd)) {
        RootsWt::Go(r) => r,
        RootsWt::Unsupported(why) => {
            return emit_unsupported_root(&cwd, CMD, opts.json, t0, &why)
        }
        RootsWt::None => {
            return match &opts.root {
                Some(target) => emit_no_root_error_at(&cwd, target, CMD, opts.json, t0),
                None => emit_no_root_error(&cwd, CMD, opts.json, t0),
            }
        }
    };
    let report = collect_outcomes(&ledger_root(&roots), opts.feature.as_deref());
    if opts.by_feature {
        let card = scorecard(&report, &read_inputs(&ledger_root(&roots)), opts.since, opts.feature.as_deref());
        if opts.json {
            println!("{}", jsjson::stringify_pretty(&card.to_json()));
        } else {
            println!("{}", card.to_text());
        }
    } else if opts.json {
        println!("{}", jsjson::stringify_pretty(&render_json(&report)));
    } else {
        println!("{}", render_text(&report));
    }
    let timing_root = match &opts.root {
        Some(_) => cwd_store_root(&cwd),
        None => Some(roots.root.clone()),
    };
    if let Some(root) = timing_root {
        record_timing(&root, CMD, t0, true);
    }
    ExitCode::SUCCESS
}

fn ledger_root(roots: &StoreRoots) -> PathBuf {
    roots.main_root()
}

fn fail(root: &Path, use_json: bool, kind: &'static str, msg: String, t0: Instant) -> ExitCode {
    if use_json {
        println!("{}", jsjson::stringify(&json!({"error": msg, "kind": kind})));
    } else {
        eprintln!("{msg}");
    }
    record_timing(root, CMD, t0, false);
    ExitCode::FAILURE
}

#[derive(Default, Debug)]
struct ReactionMetrics {
    confirm: usize,
    partial: usize,
    challenge: usize,
    blocked: usize,
    null: usize,
    absent: usize,
}

impl ReactionMetrics {
    fn cells_with_record(&self) -> usize {
        self.confirm + self.partial + self.challenge + self.blocked + self.null
    }

    fn changed(&self) -> usize {
        self.partial + self.challenge + self.blocked
    }
}

#[derive(Default, Debug)]
struct DissentMetrics {
    cells_with_record: usize,
    accept: usize,
    reject: usize,
    escalate: usize,
    unanswered: usize,
}

impl DissentMetrics {
    fn total_entries(&self) -> usize {
        self.accept + self.reject + self.escalate + self.unanswered
    }

    fn changed(&self) -> usize {
        self.accept + self.escalate
    }
}

#[derive(Default, Debug)]
struct JudgeMetrics {
    cells_with_record: usize,
    pass: usize,
    needs_revision: usize,
    other: usize,
}

impl JudgeMetrics {
    fn total_entries(&self) -> usize {
        self.pass + self.needs_revision + self.other
    }

    fn changed(&self) -> usize {
        self.needs_revision
    }
}

#[derive(Default, Debug)]
struct LeaderCheckMetrics {
    cells_with_record: usize,
    verdicts: BTreeMap<String, usize>,
}

impl LeaderCheckMetrics {
    fn total_entries(&self) -> usize {
        self.verdicts.values().sum()
    }

    fn changed(&self) -> usize {
        self.verdicts
            .iter()
            .filter(|(k, _)| k.as_str() != "ok")
            .map(|(_, v)| *v)
            .sum()
    }
}

#[derive(Default, Debug)]
struct ReworkMetrics {
    cells_with_record: usize,
}

impl ReworkMetrics {
    fn changed(&self) -> usize {
        self.cells_with_record
    }
}

#[derive(Default, Debug)]
struct RoleRerouteMetrics {
    cells_with_record: usize,
    entries: usize,
}

impl RoleRerouteMetrics {
    fn changed(&self) -> usize {
        self.entries
    }
}

#[derive(Default, Debug)]
struct PlanDepartureMetrics {
    cells_with_record: usize,
    hit_unforeseen_obstacle: usize,
    found_better_route: usize,
    plan_wrong_fact: usize,
    something_else_fixed_first: usize,
    followed_plan: usize,
    other: usize,
}

impl PlanDepartureMetrics {
    fn total_entries(&self) -> usize {
        self.hit_unforeseen_obstacle
            + self.found_better_route
            + self.plan_wrong_fact
            + self.something_else_fixed_first
            + self.followed_plan
            + self.other
    }
}

enum DepartureKind {
    HitUnforeseenObstacle,
    FoundBetterRoute,
    PlanWrongFact,
    SomethingElseFixedFirst,
    FollowedPlan,
    Other,
}

fn departure_text(entry: &Value) -> Option<String> {
    match entry {
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Object(m) => {
            if let Some(s) = m.get("kind").and_then(Value::as_str) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            if let Some(s) = m.get("text").and_then(Value::as_str) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            if let Some(s) = m.get("description").and_then(Value::as_str) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            if let Some(s) = m.get("what").and_then(Value::as_str) {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            let strings: Vec<&str> = m
                .values()
                .filter_map(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .collect();
            if !strings.is_empty() {
                Some(strings.join(" "))
            } else {
                let s = entry.to_string();
                if s.trim().is_empty() {
                    None
                } else {
                    Some(s)
                }
            }
        }
        _ => None,
    }
}

fn classify_departure(entry: &Value) -> Option<DepartureKind> {
    let text = departure_text(entry)?;
    let lower = text.to_lowercase();
    if lower.contains("hit an unforeseen obstacle") {
        Some(DepartureKind::HitUnforeseenObstacle)
    } else if lower.contains("found a better route") {
        Some(DepartureKind::FoundBetterRoute)
    } else if lower.contains("the plan was wrong about a fact") {
        Some(DepartureKind::PlanWrongFact)
    } else if lower.contains("something else had to be fixed first") {
        Some(DepartureKind::SomethingElseFixedFirst)
    } else if lower.contains("followed the plan") {
        Some(DepartureKind::FollowedPlan)
    } else {
        Some(DepartureKind::Other)
    }
}

#[derive(Default, Debug)]
struct OutcomesReport {
    feature: Option<String>,
    cells_scanned: usize,
    unreadable: Vec<String>,
    reaction: ReactionMetrics,
    dissent: DissentMetrics,
    judge: JudgeMetrics,
    leader_check: LeaderCheckMetrics,
    rework: ReworkMetrics,
    role_reroute: RoleRerouteMetrics,
    plan_departure: PlanDepartureMetrics,
    tallies: BTreeMap<String, CellTally>,
    fixes: BTreeMap<String, usize>,
}

fn find_cell_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let cells_dir = root.join(".bee").join("cells");
    if let Ok(entries) = std::fs::read_dir(&cells_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.ends_with(".json") && !name.starts_with('.') {
                        files.push(path);
                    }
        }
    }
    let archive_dir = cells_dir.join("archive");
    if let Ok(feat_entries) = std::fs::read_dir(&archive_dir) {
        for feat_entry in feat_entries.flatten() {
            let feat_path = feat_entry.path();
            if feat_path.is_dir()
                && let Ok(cell_entries) = std::fs::read_dir(&feat_path) {
                    for cell_entry in cell_entries.flatten() {
                        let path = cell_entry.path();
                        if path.is_file()
                            && let Some(name) = path.file_name().and_then(|n| n.to_str())
                                && name.ends_with(".json") && !name.starts_with('.') {
                                    files.push(path);
                                }
                    }
                }
        }
    }
    files.sort();
    files
}

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(_) => true,
        Value::Object(_) => true,
    }
}

fn collect_outcomes(root: &Path, feature_filter: Option<&str>) -> OutcomesReport {
    let mut report = OutcomesReport {
        feature: feature_filter.map(str::to_string),
        cells_scanned: 0,
        unreadable: Vec::new(),
        reaction: ReactionMetrics::default(),
        dissent: DissentMetrics::default(),
        judge: JudgeMetrics::default(),
        leader_check: LeaderCheckMetrics::default(),
        rework: ReworkMetrics::default(),
        role_reroute: RoleRerouteMetrics::default(),
        plan_departure: PlanDepartureMetrics::default(),
        tallies: BTreeMap::new(),
        fixes: BTreeMap::new(),
    };
    let files = find_cell_files(root);
    for file in files {
        let raw = match std::fs::read_to_string(&file) {
            Ok(s) => s,
            Err(_) => {
                report.unreadable.push(file.display().to_string());
                continue;
            }
        };
        let val: Value = match serde_json::from_str(&raw) {
            Ok(v) => v,
            Err(_) => {
                report.unreadable.push(file.display().to_string());
                continue;
            }
        };
        let Some(obj) = val.as_object() else {
            report.unreadable.push(file.display().to_string());
            continue;
        };
        if let Some(target) = obj.get("fixes_feature").and_then(Value::as_str) {
            *report.fixes.entry(target.to_string()).or_insert(0) += 1;
        }
        if let Some(feat) = feature_filter {
            let cell_feat = obj.get("feature").and_then(Value::as_str);
            if cell_feat != Some(feat) {
                continue;
            }
        }
        report.cells_scanned += 1;

        let trace_obj = obj.get("trace").and_then(Value::as_object);

        match trace_obj.and_then(|t| t.get("reaction")) {
            None => report.reaction.absent += 1,
            Some(Value::Null) => report.reaction.null += 1,
            Some(Value::String(s)) => {
                match s.trim().to_lowercase().as_str() {
                    "confirm" => report.reaction.confirm += 1,
                    "partial" => report.reaction.partial += 1,
                    "challenge" => report.reaction.challenge += 1,
                    "blocked" => report.reaction.blocked += 1,
                    _ => report.reaction.null += 1,
                }
            }
            Some(_) => report.reaction.null += 1,
        }

        if let Some(dissent_arr) = trace_obj.and_then(|t| t.get("dissent")).and_then(Value::as_array)
            && !dissent_arr.is_empty() {
                report.dissent.cells_with_record += 1;
                for entry in dissent_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str);
                    match verdict.map(|v| v.trim().to_lowercase()).as_deref() {
                        Some("accept") => report.dissent.accept += 1,
                        Some("reject") => report.dissent.reject += 1,
                        Some("escalate") => report.dissent.escalate += 1,
                        _ => report.dissent.unanswered += 1,
                    }
                }
            }

        if let Some(judge_arr) = trace_obj.and_then(|t| t.get("semantic_judge")).and_then(Value::as_array)
            && !judge_arr.is_empty() {
                report.judge.cells_with_record += 1;
                for entry in judge_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str);
                    match verdict {
                        Some("PASS") => report.judge.pass += 1,
                        Some("NEEDS_REVISION") => report.judge.needs_revision += 1,
                        _ => report.judge.other += 1,
                    }
                }
            }

        if let Some(leader_arr) = trace_obj.and_then(|t| t.get("leader_check")).and_then(Value::as_array)
            && !leader_arr.is_empty() {
                report.leader_check.cells_with_record += 1;
                for entry in leader_arr {
                    let verdict = entry.as_object().and_then(|e| e.get("verdict")).and_then(Value::as_str).unwrap_or("unknown");
                    *report.leader_check.verdicts.entry(verdict.to_string()).or_insert(0) += 1;
                }
            }

        let reworked = ["reopened_for_rework", "reopened_at"]
            .iter()
            .any(|k| trace_obj.and_then(|t| t.get(*k)).is_some_and(is_truthy));
        if reworked {
            report.rework.cells_with_record += 1;
        }

        if let Some(reroute_arr) = obj.get("role_reroutes").and_then(Value::as_array)
            && !reroute_arr.is_empty() {
                report.role_reroute.cells_with_record += 1;
                report.role_reroute.entries += reroute_arr.len();
            }

        if let Some(dev_arr) = trace_obj.and_then(|t| t.get("deviations")).and_then(Value::as_array) {
            let mut cell_has_departure = false;
            for entry in dev_arr {
                if let Some(kind) = classify_departure(entry) {
                    cell_has_departure = true;
                    match kind {
                        DepartureKind::HitUnforeseenObstacle => {
                            report.plan_departure.hit_unforeseen_obstacle += 1;
                        }
                        DepartureKind::FoundBetterRoute => {
                            report.plan_departure.found_better_route += 1;
                        }
                        DepartureKind::PlanWrongFact => {
                            report.plan_departure.plan_wrong_fact += 1;
                        }
                        DepartureKind::SomethingElseFixedFirst => {
                            report.plan_departure.something_else_fixed_first += 1;
                        }
                        DepartureKind::FollowedPlan => {
                            report.plan_departure.followed_plan += 1;
                        }
                        DepartureKind::Other => {
                            report.plan_departure.other += 1;
                        }
                    }
                }
            }
            if cell_has_departure {
                report.plan_departure.cells_with_record += 1;
            }
        }

        if let Some(feat) = obj.get("feature").and_then(Value::as_str) {
            report.tallies.entry(feat.to_string()).or_default().add(obj, reworked);
        }
    }
    report
}

type Stamp = DateTime<Utc>;

#[derive(Default, Debug)]
struct CellTally {
    cells: usize,
    reworked: usize,
    judge_records: usize,
    judge_revisions: usize,
    with_reaction: usize,
    newest_capped: Option<Stamp>,
}

impl CellTally {
    fn add(&mut self, cell: &Map<String, Value>, reworked: bool) {
        self.cells += 1;
        if reworked {
            self.reworked += 1;
        }
        let trace = cell.get("trace").and_then(Value::as_object);
        if let Some(judges) = trace.and_then(|t| t.get("semantic_judge")).and_then(Value::as_array) {
            for entry in judges {
                self.judge_records += 1;
                if entry.get("verdict").and_then(Value::as_str) == Some("NEEDS_REVISION") {
                    self.judge_revisions += 1;
                }
            }
        }
        if trace
            .and_then(|t| t.get("reaction"))
            .and_then(Value::as_str)
            .is_some_and(|r| !r.trim().is_empty())
        {
            self.with_reaction += 1;
        }
        let capped = trace
            .and_then(|t| t.get("capped_at"))
            .and_then(Value::as_str)
            .and_then(parse_stamp);
        self.newest_capped = self.newest_capped.max(capped);
    }
}

struct GateRow {
    feature: String,
    bypass: bool,
    reopen: bool,
}

struct JobUsage {
    feature: String,
    cost_usd: Option<f64>,
    total_tokens: Option<u64>,
}

#[derive(Default)]
struct ScoreInputs {
    lanes: BTreeMap<String, Stamp>,
    closes: BTreeMap<String, Stamp>,
    gate_rows: Vec<GateRow>,
    jobs: Vec<JobUsage>,
}

fn parse_stamp(s: &str) -> Option<Stamp> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}

fn parse_since(s: &str, now: Stamp) -> Option<Stamp> {
    if let Some(stamp) = parse_stamp(s) {
        return Some(stamp);
    }
    if let Ok(day) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return day.and_hms_opt(0, 0, 0).map(|n| n.and_utc());
    }
    let unit = s.chars().last()?;
    let count: u32 = s[..s.len() - unit.len_utf8()].parse().ok()?;
    let per_unit: i64 = match unit {
        'w' => 604_800,
        'd' => 86_400,
        'h' => 3_600,
        'm' => 60,
        _ => return None,
    };
    now.checked_sub_signed(Duration::seconds(i64::from(count) * per_unit))
}

fn json_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
        .collect();
    files.sort();
    files
}

fn read_object(path: &Path) -> Option<Map<String, Value>> {
    match serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()? {
        Value::Object(m) => Some(m),
        _ => None,
    }
}

fn read_inputs(root: &Path) -> ScoreInputs {
    let bee = root.join(".bee");
    let mut inputs = ScoreInputs::default();
    for path in json_files(&bee.join("lanes")) {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if let Some(lane) = read_object(&path) {
            let name = lane.get("feature").and_then(Value::as_str).unwrap_or(&stem).to_string();
            if let Some(at) = lane.get("created_at").and_then(Value::as_str).and_then(parse_stamp) {
                inputs.lanes.insert(name, at);
            }
        }
    }
    for path in json_files(&bee.join("usage")) {
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_string();
        if let Some(at) = read_object(&path)
            .and_then(|r| r.get("closed_at").and_then(Value::as_str).and_then(parse_stamp))
        {
            inputs.closes.insert(stem, at);
        }
    }
    if let Ok(raw) = std::fs::read_to_string(bee.join("audit.jsonl")) {
        for line in raw.lines() {
            let Ok(Value::Object(row)) = serde_json::from_str::<Value>(line) else { continue };
            let kind = row.get("kind").and_then(Value::as_str);
            let feature = row.get("feature").and_then(Value::as_str);
            if let (Some(kind @ ("gate" | "gate-bypass")), Some(feature)) = (kind, feature) {
                let note = row.get("note").and_then(Value::as_str).unwrap_or_default();
                inputs.gate_rows.push(GateRow {
                    feature: feature.to_string(),
                    bypass: kind == "gate-bypass",
                    reopen: note.contains(" set to false"),
                });
            }
        }
    }
    if let Ok(entries) = std::fs::read_dir(bee.join("mailbox")) {
        let mut jobs: Vec<PathBuf> = entries.flatten().map(|e| e.path().join("job.json")).collect();
        jobs.sort();
        for path in jobs {
            let Some(job) = read_object(&path) else { continue };
            let Some(feature) = job.get("feature").and_then(Value::as_str) else { continue };
            let usage = job.get("usage").and_then(Value::as_object);
            inputs.jobs.push(JobUsage {
                feature: feature.to_string(),
                cost_usd: usage.and_then(|u| u.get("cost_usd")).and_then(Value::as_f64),
                total_tokens: usage.and_then(|u| u.get("total_tokens")).and_then(Value::as_u64),
            });
        }
    }
    inputs
}

fn pct(num: usize, den: usize) -> Option<f64> {
    (den > 0).then(|| (num as f64 / den as f64 * 1000.0).round() / 10.0)
}

fn cycle_secs(created: Option<Stamp>, closed: Option<Stamp>, capped: Option<Stamp>) -> Option<i64> {
    let end = closed.or(capped)?;
    Some((end - created?).num_seconds().max(0))
}

fn gate_reopens(rows: &[GateRow], feature: &str) -> Option<usize> {
    let mine: Vec<&GateRow> = rows.iter().filter(|r| r.feature == feature).collect();
    let audited = mine.iter().any(|r| !r.bypass);
    (!mine.is_empty()).then(|| mine.iter().filter(|r| r.reopen && (!audited || !r.bypass)).count())
}

fn fix_first(fixes: &BTreeMap<String, usize>, feature: &str) -> Option<usize> {
    (!fixes.is_empty()).then(|| fixes.get(feature).copied().unwrap_or(0))
}

fn job_cost(jobs: &[JobUsage], feature: &str) -> Option<f64> {
    let costs: Vec<f64> = jobs.iter().filter(|j| j.feature == feature).filter_map(|j| j.cost_usd).collect();
    (!costs.is_empty()).then(|| costs.iter().sum())
}

fn job_tokens(jobs: &[JobUsage], feature: &str) -> Option<u64> {
    let tokens: Vec<u64> = jobs.iter().filter(|j| j.feature == feature).filter_map(|j| j.total_tokens).collect();
    (!tokens.is_empty()).then(|| tokens.iter().sum())
}

struct FeatureRow {
    feature: String,
    created_at: Option<Stamp>,
    closed: bool,
    cells: usize,
    reworked: usize,
    rework_rate: Option<f64>,
    judge_revision_rate: Option<f64>,
    reaction_coverage: Option<f64>,
    cycle_secs: Option<i64>,
    gate_reopens: Option<usize>,
    fix_first: Option<usize>,
    cost_usd: Option<f64>,
    total_tokens: Option<u64>,
}

struct Scorecard {
    rows: Vec<FeatureRow>,
}

fn scorecard(
    report: &OutcomesReport,
    inputs: &ScoreInputs,
    since: Option<Stamp>,
    feature_filter: Option<&str>,
) -> Scorecard {
    let mut names: Vec<&String> = report.tallies.keys().chain(inputs.lanes.keys()).collect();
    names.sort();
    names.dedup();
    let empty = CellTally::default();
    let mut rows: Vec<FeatureRow> = names
        .into_iter()
        .filter(|f| feature_filter.is_none_or(|want| want == f.as_str()))
        .filter_map(|f| {
            let created_at = inputs.lanes.get(f).copied();
            if let Some(bound) = since
                && created_at.is_none_or(|c| c < bound)
            {
                return None;
            }
            let t = report.tallies.get(f).unwrap_or(&empty);
            let closed_at = inputs.closes.get(f).copied();
            Some(FeatureRow {
                feature: f.clone(),
                created_at,
                closed: closed_at.is_some(),
                cells: t.cells,
                reworked: t.reworked,
                rework_rate: pct(t.reworked, t.cells),
                judge_revision_rate: pct(t.judge_revisions, t.judge_records),
                reaction_coverage: pct(t.with_reaction, t.cells),
                cycle_secs: cycle_secs(created_at, closed_at, t.newest_capped),
                gate_reopens: gate_reopens(&inputs.gate_rows, f),
                fix_first: fix_first(&report.fixes, f),
                cost_usd: job_cost(&inputs.jobs, f),
                total_tokens: job_tokens(&inputs.jobs, f),
            })
        })
        .collect();
    rows.sort_by(|a, b| {
        (a.created_at.is_none(), a.created_at, &a.feature).cmp(&(b.created_at.is_none(), b.created_at, &b.feature))
    });
    Scorecard { rows }
}

fn fmt_duration(secs: i64) -> String {
    let (d, h, m) = (secs / 86_400, secs % 86_400 / 3_600, secs % 3_600 / 60);
    if d > 0 {
        format!("{d}d {h}h")
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

fn or_missing<T>(v: Option<T>, fmt: impl Fn(T) -> String) -> String {
    v.map_or_else(|| "not recorded".to_string(), fmt)
}

impl Scorecard {
    fn overall_rework(&self) -> Option<f64> {
        pct(
            self.rows.iter().map(|r| r.reworked).sum(),
            self.rows.iter().map(|r| r.cells).sum(),
        )
    }

    fn header(&self) -> String {
        let closed = self.rows.iter().filter(|r| r.closed).count();
        let rework = match self.overall_rework() {
            Some(r) => format!("rework rate {r}% overall"),
            None => "rework rate not recorded overall".to_string(),
        };
        format!("features: {} (closed {closed}) · {rework}", self.rows.len())
    }

    fn to_text(&self) -> String {
        let mut lines = vec![self.header()];
        for r in &self.rows {
            let percent = |v| or_missing(v, |p: f64| format!("{p}%"));
            lines.push(format!(
                "{}: cells {} · rework {} · judge revisions {} · reactions {} · cycle {} · gate reopens {} · fix-first {} · cost {} · tokens {}",
                r.feature,
                r.cells,
                percent(r.rework_rate),
                percent(r.judge_revision_rate),
                percent(r.reaction_coverage),
                or_missing(r.cycle_secs, fmt_duration),
                or_missing(r.gate_reopens, |n| n.to_string()),
                or_missing(r.fix_first, |n| n.to_string()),
                or_missing(r.cost_usd, |c| format!("${c:.2}")),
                or_missing(r.total_tokens, |n| n.to_string()),
            ));
        }
        lines.join("\n")
    }

    fn to_json(&self) -> Value {
        let features: Vec<Value> = self
            .rows
            .iter()
            .map(|r| {
                json!({
                    "feature": r.feature,
                    "created_at": r.created_at.map(|c| c.to_rfc3339()),
                    "closed": r.closed,
                    "cells": r.cells,
                    "rework_rate": r.rework_rate,
                    "judge_revision_rate": r.judge_revision_rate,
                    "reaction_coverage": r.reaction_coverage,
                    "cycle_time_seconds": r.cycle_secs,
                    "gate_reopens": r.gate_reopens,
                    "fix_first_cells": r.fix_first,
                    "cost_usd": r.cost_usd,
                    "total_tokens": r.total_tokens,
                })
            })
            .collect();
        json!({"header": self.header(), "features": features})
    }
}

fn render_text(r: &OutcomesReport) -> String {
    let mut lines = Vec::new();

    if r.reaction.cells_with_record() == 0 {
        lines.push("worker reaction: not recorded".to_string());
    } else {
        lines.push(format!(
            "worker reaction: {} cells ({} changed) — confirm: {}, partial: {}, challenge: {}, blocked: {}, null: {}, absent: {}",
            r.reaction.cells_with_record(),
            r.reaction.changed(),
            r.reaction.confirm,
            r.reaction.partial,
            r.reaction.challenge,
            r.reaction.blocked,
            r.reaction.null,
            r.reaction.absent
        ));
    }

    if r.dissent.cells_with_record == 0 {
        lines.push("dissent: not recorded".to_string());
    } else {
        lines.push(format!(
            "dissent: {} cells, {} records ({} changed) — accept: {}, reject: {}, escalate: {}, unanswered: {}",
            r.dissent.cells_with_record,
            r.dissent.total_entries(),
            r.dissent.changed(),
            r.dissent.accept,
            r.dissent.reject,
            r.dissent.escalate,
            r.dissent.unanswered
        ));
    }

    if r.judge.cells_with_record == 0 {
        lines.push("semantic judge: not recorded".to_string());
    } else {
        lines.push(format!(
            "semantic judge: {} cells, {} records ({} changed) — PASS: {}, NEEDS_REVISION: {}, other: {}",
            r.judge.cells_with_record,
            r.judge.total_entries(),
            r.judge.changed(),
            r.judge.pass,
            r.judge.needs_revision,
            r.judge.other
        ));
    }

    if r.leader_check.cells_with_record == 0 {
        lines.push("leader check: not recorded".to_string());
    } else {
        let v_str = r.leader_check.verdicts.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join(", ");
        lines.push(format!(
            "leader check: {} cells, {} records ({} changed) — {}",
            r.leader_check.cells_with_record,
            r.leader_check.total_entries(),
            r.leader_check.changed(),
            v_str
        ));
    }

    if r.rework.cells_with_record == 0 {
        lines.push("rework: not recorded".to_string());
    } else {
        lines.push(format!(
            "rework: {} cells ({} changed)",
            r.rework.cells_with_record,
            r.rework.changed()
        ));
    }

    if r.role_reroute.cells_with_record == 0 {
        lines.push("role reroute: not recorded".to_string());
    } else {
        lines.push(format!(
            "role reroute: {} cells, {} records ({} changed)",
            r.role_reroute.cells_with_record,
            r.role_reroute.entries,
            r.role_reroute.changed()
        ));
    }

    if r.plan_departure.cells_with_record == 0 {
        lines.push("plan departure: not recorded".to_string());
    } else {
        lines.push(format!(
            "plan departure: {} cells, {} records — hit an unforeseen obstacle: {}, found a better route: {}, the plan was wrong about a fact: {}, something else had to be fixed first: {}, followed the plan: {}, other: {}",
            r.plan_departure.cells_with_record,
            r.plan_departure.total_entries(),
            r.plan_departure.hit_unforeseen_obstacle,
            r.plan_departure.found_better_route,
            r.plan_departure.plan_wrong_fact,
            r.plan_departure.something_else_fixed_first,
            r.plan_departure.followed_plan,
            r.plan_departure.other
        ));
    }

    if !r.unreadable.is_empty() {
        lines.push(format!("unreadable: {} ({})", r.unreadable.len(), r.unreadable.join(", ")));
    }

    lines.join("\n")
}

fn render_json(r: &OutcomesReport) -> Value {
    let mut m = Map::new();
    m.insert(
        "feature".into(),
        match &r.feature {
            Some(f) => Value::String(f.clone()),
            None => Value::Null,
        },
    );
    m.insert("cells_scanned".into(), json!(r.cells_scanned));
    m.insert("unreadable".into(), json!(r.unreadable));

    m.insert(
        "reaction".into(),
        json!({
            "recorded": r.reaction.cells_with_record() > 0,
            "cells_with_record": r.reaction.cells_with_record(),
            "records": r.reaction.cells_with_record(),
            "changed": r.reaction.changed(),
            "confirm": r.reaction.confirm,
            "partial": r.reaction.partial,
            "challenge": r.reaction.challenge,
            "blocked": r.reaction.blocked,
            "null": r.reaction.null,
            "absent": r.reaction.absent,
        }),
    );

    m.insert(
        "dissent".into(),
        json!({
            "recorded": r.dissent.cells_with_record > 0,
            "cells_with_record": r.dissent.cells_with_record,
            "records": r.dissent.total_entries(),
            "changed": r.dissent.changed(),
            "accept": r.dissent.accept,
            "reject": r.dissent.reject,
            "escalate": r.dissent.escalate,
            "unanswered": r.dissent.unanswered,
            "verdicts": {
                "accept": r.dissent.accept,
                "reject": r.dissent.reject,
                "escalate": r.dissent.escalate,
                "unanswered": r.dissent.unanswered,
            }
        }),
    );

    m.insert(
        "semantic_judge".into(),
        json!({
            "recorded": r.judge.cells_with_record > 0,
            "cells_with_record": r.judge.cells_with_record,
            "records": r.judge.total_entries(),
            "changed": r.judge.changed(),
            "PASS": r.judge.pass,
            "NEEDS_REVISION": r.judge.needs_revision,
            "other": r.judge.other,
            "verdicts": {
                "PASS": r.judge.pass,
                "NEEDS_REVISION": r.judge.needs_revision,
                "other": r.judge.other,
            }
        }),
    );

    m.insert(
        "leader_check".into(),
        json!({
            "recorded": r.leader_check.cells_with_record > 0,
            "cells_with_record": r.leader_check.cells_with_record,
            "records": r.leader_check.total_entries(),
            "changed": r.leader_check.changed(),
            "verdicts": r.leader_check.verdicts,
        }),
    );

    m.insert(
        "rework".into(),
        json!({
            "recorded": r.rework.cells_with_record > 0,
            "cells_with_record": r.rework.cells_with_record,
            "records": r.rework.cells_with_record,
            "changed": r.rework.changed(),
        }),
    );

    m.insert(
        "role_reroute".into(),
        json!({
            "recorded": r.role_reroute.cells_with_record > 0,
            "cells_with_record": r.role_reroute.cells_with_record,
            "records": r.role_reroute.entries,
            "changed": r.role_reroute.changed(),
        }),
    );

    m.insert(
        "plan_departure".into(),
        json!({
            "recorded": r.plan_departure.cells_with_record > 0,
            "cells_with_record": r.plan_departure.cells_with_record,
            "records": r.plan_departure.total_entries(),
            "hit an unforeseen obstacle": r.plan_departure.hit_unforeseen_obstacle,
            "found a better route": r.plan_departure.found_better_route,
            "the plan was wrong about a fact": r.plan_departure.plan_wrong_fact,
            "something else had to be fixed first": r.plan_departure.something_else_fixed_first,
            "followed the plan": r.plan_departure.followed_plan,
            "other": r.plan_departure.other,
            "kinds": {
                "hit an unforeseen obstacle": r.plan_departure.hit_unforeseen_obstacle,
                "found a better route": r.plan_departure.found_better_route,
                "the plan was wrong about a fact": r.plan_departure.plan_wrong_fact,
                "something else had to be fixed first": r.plan_departure.something_else_fixed_first,
                "followed the plan": r.plan_departure.followed_plan,
                "other": r.plan_departure.other,
            }
        }),
    );

    Value::Object(m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn setup_fixture_store() -> (tempfile::TempDir, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        let cells = root.join(".bee").join("cells");
        let arch = cells.join("archive").join("slp-article-gaps");
        std::fs::create_dir_all(&arch).unwrap();
        std::fs::write(root.join(".bee/onboarding.json"), r#"{"completed":true}"#).unwrap();
        std::fs::write(root.join(".bee/state.json"), r#"{"phase":"executing"}"#).unwrap();

        std::fs::write(
            cells.join("c-01.json"),
            r#"{"id":"c-01","feature":"slp-article-gaps","trace":{"reaction":"confirm","deviations":["hit an unforeseen obstacle"]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-02.json"),
            r#"{"id":"c-02","feature":"slp-article-gaps","trace":{"reaction":"partial","deviations":["Found a better route"]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-03.json"),
            r#"{"id":"c-03","feature":"slp-article-gaps","trace":{"reaction":"challenge","deviations":["the plan was wrong about a fact"]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-04.json"),
            r#"{"id":"c-04","feature":"slp-article-gaps","trace":{"reaction":"blocked","deviations":["something else had to be fixed first"]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-05.json"),
            r#"{"id":"c-05","feature":"slp-article-gaps","trace":{"reaction":null,"deviations":["followed the plan"]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-06.json"),
            r#"{"id":"c-06","feature":"slp-article-gaps","trace":{"deviations":["some unclassified deviation",{"text":"found a better route"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-07.json"),
            r#"{"id":"c-07","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"accept"}],"deviations":[{"kind":"hit an unforeseen obstacle","what":"did foo","why":"bar"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-08.json"),
            r#"{"id":"c-08","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"reject"}],"deviations":[]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-09.json"),
            r#"{"id":"c-09","feature":"slp-article-gaps","trace":{"dissent":[{"verdict":"escalate"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-10.json"),
            r#"{"id":"c-10","feature":"slp-article-gaps","trace":{"dissent":[{"claim":"issue"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-11.json"),
            r#"{"id":"c-11","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"PASS"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-12.json"),
            r#"{"id":"c-12","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"NEEDS_REVISION"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-13.json"),
            r#"{"id":"c-13","feature":"slp-article-gaps","trace":{"semantic_judge":[{"verdict":"OTHER"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-14.json"),
            r#"{"id":"c-14","feature":"slp-article-gaps","trace":{"leader_check":[{"verdict":"ok"}]}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-15.json"),
            r#"{"id":"c-15","feature":"slp-article-gaps","trace":{"leader_check":[{"verdict":"gap"}]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-16.json"),
            r#"{"id":"c-16","feature":"slp-article-gaps","trace":{"reopened_for_rework":{"reason":"fix"}}}"#,
        ).unwrap();
        std::fs::write(
            cells.join("c-16b.json"),
            r#"{"id":"c-16b","feature":"slp-article-gaps","trace":{"reopened_at":"2026-11-04T00:00:00Z","reopened_reason":"goal-check miss"}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-17.json"),
            r#"{"id":"c-17","feature":"slp-article-gaps","role_reroutes":[{"from":"code","to":"test"}]}"#,
        ).unwrap();

        std::fs::write(
            arch.join("c-18.json"),
            r#"{"id":"c-18","feature":"slp-article-gaps","trace":{"reaction":"confirm"}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("c-19-other.json"),
            r#"{"id":"c-19","feature":"other-feat","trace":{"reaction":"blocked","deviations":["hit an unforeseen obstacle"]}}"#,
        ).unwrap();

        std::fs::write(
            cells.join("corrupt.json"),
            "not valid json {{{",
        ).unwrap();

        (tmp, root)
    }

    #[test]
    fn test_report_outcomes_fixture_counts_with_feature_filter() {
        let (_tmp, root) = setup_fixture_store();
        let report = collect_outcomes(&root, Some("slp-article-gaps"));

        assert_eq!(report.cells_scanned, 19);
        assert_eq!(report.unreadable.len(), 1);
        assert!(report.unreadable[0].ends_with("corrupt.json"));

        assert_eq!(report.reaction.confirm, 2);
        assert_eq!(report.reaction.partial, 1);
        assert_eq!(report.reaction.challenge, 1);
        assert_eq!(report.reaction.blocked, 1);
        assert_eq!(report.reaction.null, 1);
        assert_eq!(report.reaction.absent, 13);
        assert_eq!(report.reaction.cells_with_record(), 6);
        assert_eq!(report.reaction.changed(), 3);

        assert_eq!(report.dissent.cells_with_record, 4);
        assert_eq!(report.dissent.accept, 1);
        assert_eq!(report.dissent.reject, 1);
        assert_eq!(report.dissent.escalate, 1);
        assert_eq!(report.dissent.unanswered, 1);
        assert_eq!(report.dissent.changed(), 2);

        assert_eq!(report.judge.cells_with_record, 3);
        assert_eq!(report.judge.pass, 1);
        assert_eq!(report.judge.needs_revision, 1);
        assert_eq!(report.judge.other, 1);
        assert_eq!(report.judge.changed(), 1);

        assert_eq!(report.leader_check.cells_with_record, 2);
        assert_eq!(report.leader_check.verdicts.get("ok"), Some(&1));
        assert_eq!(report.leader_check.verdicts.get("gap"), Some(&1));
        assert_eq!(report.leader_check.changed(), 1);

        assert_eq!(report.rework.cells_with_record, 2);
        assert_eq!(report.rework.changed(), 2);

        assert_eq!(report.role_reroute.cells_with_record, 1);
        assert_eq!(report.role_reroute.entries, 1);
        assert_eq!(report.role_reroute.changed(), 1);

        assert_eq!(report.plan_departure.cells_with_record, 7);
        assert_eq!(report.plan_departure.total_entries(), 8);
        assert_eq!(report.plan_departure.hit_unforeseen_obstacle, 2);
        assert_eq!(report.plan_departure.found_better_route, 2);
        assert_eq!(report.plan_departure.plan_wrong_fact, 1);
        assert_eq!(report.plan_departure.something_else_fixed_first, 1);
        assert_eq!(report.plan_departure.followed_plan, 1);
        assert_eq!(report.plan_departure.other, 1);

        let text = render_text(&report);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 8);
        assert!(lines[0].starts_with("worker reaction: 6 cells (3 changed)"));
        assert!(lines[0].contains("confirm: 2, partial: 1, challenge: 1, blocked: 1, null: 1, absent: 13"));
        assert!(lines[1].starts_with("dissent: 4 cells, 4 records (2 changed)"));
        assert!(lines[1].contains("accept: 1, reject: 1, escalate: 1, unanswered: 1"));
        assert!(lines[2].starts_with("semantic judge: 3 cells, 3 records (1 changed)"));
        assert!(lines[2].contains("PASS: 1, NEEDS_REVISION: 1, other: 1"));
        assert!(lines[3].starts_with("leader check: 2 cells, 2 records (1 changed)"));
        assert!(lines[3].contains("gap: 1, ok: 1"));
        assert_eq!(lines[4], "rework: 2 cells (2 changed)");
        assert_eq!(lines[5], "role reroute: 1 cells, 1 records (1 changed)");
        assert_eq!(
            lines[6],
            "plan departure: 7 cells, 8 records — hit an unforeseen obstacle: 2, found a better route: 2, the plan was wrong about a fact: 1, something else had to be fixed first: 1, followed the plan: 1, other: 1"
        );
        assert!(lines[7].starts_with("unreadable: 1"));

        let json = render_json(&report);
        assert_eq!(json["cells_scanned"], 19);
        assert_eq!(json["reaction"]["confirm"], 2);
        assert_eq!(json["reaction"]["changed"], 3);
        assert_eq!(json["dissent"]["accept"], 1);
        assert_eq!(json["dissent"]["changed"], 2);
        assert_eq!(json["semantic_judge"]["NEEDS_REVISION"], 1);
        assert_eq!(json["semantic_judge"]["changed"], 1);
        assert_eq!(json["leader_check"]["verdicts"]["gap"], 1);
        assert_eq!(json["leader_check"]["changed"], 1);
        assert_eq!(json["rework"]["changed"], 2);
        assert_eq!(json["role_reroute"]["changed"], 1);
        assert_eq!(json["plan_departure"]["recorded"], true);
        assert_eq!(json["plan_departure"]["cells_with_record"], 7);
        assert_eq!(json["plan_departure"]["records"], 8);
        assert_eq!(json["plan_departure"]["hit an unforeseen obstacle"], 2);
        assert_eq!(json["plan_departure"]["found a better route"], 2);
        assert_eq!(json["plan_departure"]["the plan was wrong about a fact"], 1);
        assert_eq!(json["plan_departure"]["something else had to be fixed first"], 1);
        assert_eq!(json["plan_departure"]["followed the plan"], 1);
        assert_eq!(json["plan_departure"]["other"], 1);
        assert_eq!(json["plan_departure"]["kinds"]["hit an unforeseen obstacle"], 2);
    }

    #[test]
    fn test_report_outcomes_unfiltered_includes_other_feature() {
        let (_tmp, root) = setup_fixture_store();
        let report = collect_outcomes(&root, None);

        assert_eq!(report.cells_scanned, 20);
        assert_eq!(report.reaction.blocked, 2);
        assert_eq!(report.reaction.changed(), 4);
        assert_eq!(report.plan_departure.cells_with_record, 8);
        assert_eq!(report.plan_departure.total_entries(), 9);
        assert_eq!(report.plan_departure.hit_unforeseen_obstacle, 3);
    }

    #[test]
    fn test_empty_store_prints_not_recorded_never_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        std::fs::create_dir_all(root.join(".bee/cells")).unwrap();

        let report = collect_outcomes(&root, None);
        assert_eq!(report.cells_scanned, 0);

        let text = render_text(&report);
        for line in text.lines() {
            assert!(line.ends_with("not recorded"), "line must end with 'not recorded': {line}");
            assert!(!line.contains(": 0"), "line must not contain ': 0': {line}");
        }

        assert!(text.contains("worker reaction: not recorded"));
        assert!(text.contains("dissent: not recorded"));
        assert!(text.contains("semantic judge: not recorded"));
        assert!(text.contains("leader check: not recorded"));
        assert!(text.contains("rework: not recorded"));
        assert!(text.contains("role reroute: not recorded"));
        assert!(text.contains("plan departure: not recorded"));

        let json = render_json(&report);
        assert_eq!(json["plan_departure"]["recorded"], false);
        assert_eq!(json["plan_departure"]["cells_with_record"], 0);
        assert_eq!(json["plan_departure"]["records"], 0);
    }

    #[test]
    fn test_try_native_flag_parsing() {
        let opts = parse(&[OsString::from("--json")]).unwrap();
        assert!(opts.json);
        assert_eq!(opts.feature, None);

        let opts = parse(&[OsString::from("--feature"), OsString::from("xyz")]).unwrap();
        assert!(!opts.json);
        assert_eq!(opts.feature.as_deref(), Some("xyz"));

        let opts = parse(&[OsString::from("--feature=abc"), OsString::from("--json")]).unwrap();
        assert!(opts.json);
        assert_eq!(opts.feature.as_deref(), Some("abc"));

        assert!(parse(&[OsString::from("--unknown")]).is_none());
    }

    fn git(cwd: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .unwrap()
            .status
            .success();
        assert!(ok, "git {args:?}");
    }

    #[test]
    fn test_report_outcomes_linked_worktree_reads_main_store() {
        let dir = tempfile::tempdir().unwrap();
        let main = dir.path().join("main");
        std::fs::create_dir_all(main.join(".bee/cells")).unwrap();
        std::fs::write(main.join(".bee/onboarding.json"), "{}").unwrap();
        git(&main, &["init", "-q", "."]);
        git(
            &main,
            &[
                "-c",
                "user.email=t@t.t",
                "-c",
                "user.name=t",
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                "i",
            ],
        );
        let wt = dir.path().join("wt");
        git(
            &main,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "wt/x"],
        );

        let main_cell = json!({
            "id": "cell-main",
            "feature": "feat-a",
            "trace": {
                "reaction": "confirm"
            }
        });
        std::fs::write(
            main.join(".bee/cells/cell-main.json"),
            serde_json::to_string(&main_cell).unwrap(),
        )
        .unwrap();

        std::fs::create_dir_all(wt.join(".bee/cells")).unwrap();
        let wt_cell = json!({
            "id": "cell-wt-stale",
            "feature": "feat-a",
            "trace": {
                "reaction": "blocked"
            }
        });
        std::fs::write(
            wt.join(".bee/cells/cell-wt-stale.json"),
            serde_json::to_string(&wt_cell).unwrap(),
        )
        .unwrap();

        std::fs::create_dir_all(main.join(".bee/runtime")).unwrap();
        std::fs::write(
            main.join(".bee/runtime/worktree-grants.json"),
            r#"{"wt": true}"#,
        )
        .unwrap();

        let RootsWt::Go(roots) = resolve_store_root_worktree(&wt) else {
            panic!("worktree did not resolve");
        };
        assert!(roots.linked.as_ref().is_some_and(|l| l.granted()));

        let report = collect_outcomes(&ledger_root(&roots), None);
        assert_eq!(report.cells_scanned, 1);
        assert_eq!(report.reaction.confirm, 1);
        assert_eq!(report.reaction.blocked, 0);
    }

    fn report_root_args(extra: &[&str]) -> Vec<OsString> {
        let mut v: Vec<OsString> = vec!["report".into(), "outcomes".into(), "--json".into()];
        v.extend(extra.iter().map(OsString::from));
        v
    }

    fn report_root_snapshot(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
        fn walk(dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
            let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path()).collect();
            entries.sort();
            for p in entries {
                if p.is_dir() {
                    walk(&p, out);
                } else {
                    out.push((p.clone(), std::fs::read(&p).unwrap()));
                }
            }
        }
        let mut out = Vec::new();
        walk(root, &mut out);
        out
    }

    fn report_root_store(cells: usize) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee/cells")).unwrap();
        std::fs::write(tmp.path().join(".bee/onboarding.json"), "{}").unwrap();
        for i in 0..cells {
            std::fs::write(
                tmp.path().join(format!(".bee/cells/c{i}.json")),
                format!("{{\"id\":\"c{i}\",\"feature\":\"f\",\"trace\":{{\"reaction\":\"confirm\"}}}}"),
            )
            .unwrap();
        }
        tmp
    }

    struct ReportRootCwd {
        _lock: std::sync::MutexGuard<'static, ()>,
        orig: PathBuf,
    }

    impl ReportRootCwd {
        fn enter(path: &Path) -> Self {
            let lock = crate::verbs::drivers::TEST_CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
            let orig = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self { _lock: lock, orig }
        }
    }

    impl Drop for ReportRootCwd {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.orig);
        }
    }

    #[test]
    fn report_root_counts_the_target_stores_cells_not_the_cwd_store() {
        let cwd_store = report_root_store(1);
        let target = report_root_store(3);
        let _cwd = ReportRootCwd::enter(cwd_store.path());
        let opts = parse(&report_root_args(&["--root", target.path().to_str().unwrap()])[2..]).unwrap();
        let RootsWt::Go(roots) = resolve_store_root_worktree(&opts.start(&std::env::current_dir().unwrap())) else {
            panic!("target did not resolve");
        };
        assert_eq!(collect_outcomes(&ledger_root(&roots), None).cells_scanned, 3);
        let plain = parse(&report_root_args(&[])[2..]).unwrap();
        let RootsWt::Go(here) = resolve_store_root_worktree(&plain.start(&std::env::current_dir().unwrap())) else {
            panic!("cwd did not resolve");
        };
        assert_eq!(collect_outcomes(&ledger_root(&here), None).cells_scanned, 1);
    }

    #[test]
    fn report_root_accepts_a_path_inside_the_store_and_the_equals_form() {
        let cwd_store = report_root_store(0);
        let target = report_root_store(2);
        let _cwd = ReportRootCwd::enter(cwd_store.path());
        let inside = format!("--root={}", target.path().join(".bee").display());
        let opts = parse(&report_root_args(&[&inside])[2..]).unwrap();
        let RootsWt::Go(roots) = resolve_store_root_worktree(&opts.start(&std::env::current_dir().unwrap())) else {
            panic!("inside path did not resolve");
        };
        assert_eq!(collect_outcomes(&ledger_root(&roots), None).cells_scanned, 2);
    }

    #[test]
    fn report_root_outside_a_store_exits_with_the_no_root_error() {
        let cwd_store = report_root_store(1);
        let bare = tempfile::tempdir().unwrap();
        let _cwd = ReportRootCwd::enter(cwd_store.path());
        let args = report_root_args(&["--root", bare.path().to_str().unwrap()]);
        assert_eq!(try_native(&args, Instant::now()), Some(ExitCode::FAILURE));
        assert!(!bare.path().join(".bee").exists());
    }

    #[test]
    fn report_root_leaves_every_byte_of_the_target_store_alone() {
        let cwd_store = report_root_store(1);
        let target = report_root_store(2);
        let before = report_root_snapshot(target.path());
        let _cwd = ReportRootCwd::enter(cwd_store.path());
        let args = report_root_args(&["--root", target.path().to_str().unwrap()]);
        assert_eq!(try_native(&args, Instant::now()), Some(ExitCode::SUCCESS));
        assert_eq!(report_root_snapshot(target.path()), before);
        assert!(cwd_store.path().join(".bee/logs/timings.jsonl").exists());
    }

    #[test]
    fn report_root_skips_timing_when_the_cwd_holds_no_store() {
        let bare = tempfile::tempdir().unwrap();
        let target = report_root_store(2);
        let before = report_root_snapshot(target.path());
        let _cwd = ReportRootCwd::enter(bare.path());
        let args = report_root_args(&["--root", target.path().to_str().unwrap()]);
        assert_eq!(try_native(&args, Instant::now()), Some(ExitCode::SUCCESS));
        assert_eq!(report_root_snapshot(target.path()), before);
        assert!(!bare.path().join(".bee").exists());
    }

    fn scorecard_write(root: &Path, rel: &str, body: &str) {
        let path = root.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, body).unwrap();
    }

    fn scorecard_add_sources(root: &Path) {
        scorecard_write(root, ".bee/lanes/slp-article-gaps.json", r#"{"feature":"slp-article-gaps","created_at":"2026-11-01T00:00:00Z"}"#);
        scorecard_write(root, ".bee/lanes/other-feat.json", r#"{"feature":"other-feat","created_at":"2026-11-10T00:00:00Z"}"#);
        scorecard_write(root, ".bee/usage/slp-article-gaps.json", r#"{"feature":"slp-article-gaps","closed_at":"2026-11-03T12:00:00Z"}"#);
        let audit = [
            r#"{"kind":"gate","feature":"slp-article-gaps","note":"gate execution set to false by user"}"#,
            r#"{"kind":"gate","feature":"slp-article-gaps","note":"gate execution set to true by user"}"#,
            r#"{"kind":"gate","feature":"slp-article-gaps","note":"gate shape set to false by auto"}"#,
            r#"{"kind":"gate-bypass","feature":"slp-article-gaps","note":"gate shape set to false by auto: normal"}"#,
            r#"{"kind":"gate-bypass","feature":"other-feat","note":"gate execution set to false by auto: normal"}"#,
            r#"{"kind":"gate-bypass","feature":"other-feat","note":"gate execution set to true by auto: normal"}"#,
            r#"{"kind":"cap","feature":"other-feat","note":"gate execution set to false by user"}"#,
        ];
        scorecard_write(root, ".bee/audit.jsonl", &format!("{}\n", audit.join("\n")));
        scorecard_write(root, ".bee/mailbox/job-a/job.json", r#"{"feature":"slp-article-gaps","usage":{"cost_usd":0.5,"total_tokens":1000}}"#);
        scorecard_write(root, ".bee/mailbox/job-b/job.json", r#"{"feature":"slp-article-gaps","usage":{"cost_usd":0.25,"total_tokens":500}}"#);
        scorecard_write(root, ".bee/mailbox/job-c/job.json", r#"{"feature":"other-feat"}"#);
    }

    fn scorecard_store() -> (tempfile::TempDir, PathBuf) {
        let (tmp, root) = setup_fixture_store();
        scorecard_add_sources(&root);
        scorecard_write(
            &root,
            ".bee/cells/c-20-other.json",
            r#"{"id":"c-20","feature":"other-feat","fixes_feature":"slp-article-gaps","trace":{"capped_at":"2026-11-10T05:30:00Z"}}"#,
        );
        scorecard_write(
            &root,
            ".bee/cells/c-21-other.json",
            r#"{"id":"c-21","feature":"other-feat","trace":{"capped_at":"2026-11-10T01:00:00Z"}}"#,
        );
        (tmp, root)
    }

    fn scorecard_of(root: &Path, since: Option<Stamp>) -> Scorecard {
        scorecard(&collect_outcomes(root, None), &read_inputs(root), since, None)
    }

    fn scorecard_row<'a>(card: &'a Scorecard, feature: &str) -> &'a FeatureRow {
        card.rows.iter().find(|r| r.feature == feature).unwrap()
    }

    fn scorecard_stamp(s: &str) -> Stamp {
        parse_stamp(s).unwrap()
    }

    #[test]
    fn scorecard_metrics_with_a_source_print_values() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        let slp = scorecard_row(&card, "slp-article-gaps");
        assert_eq!(slp.cells, 19);
        assert_eq!(slp.rework_rate, Some(10.5));
        assert_eq!(slp.judge_revision_rate, Some(33.3));
        assert_eq!(slp.reaction_coverage, Some(26.3));
        assert_eq!(slp.cycle_secs, Some(216_000));
        assert_eq!(slp.gate_reopens, Some(2));
        assert_eq!(slp.fix_first, Some(1));
        assert_eq!(slp.cost_usd, Some(0.75));
        assert_eq!(slp.total_tokens, Some(1500));
        assert!(slp.closed);
        let text = card.to_text();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[1],
            "slp-article-gaps: cells 19 · rework 10.5% · judge revisions 33.3% · reactions 26.3% · cycle 2d 12h · gate reopens 2 · fix-first 1 · cost $0.75 · tokens 1500"
        );
    }

    #[test]
    fn scorecard_metrics_without_a_source_print_not_recorded_never_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        scorecard_write(&root, ".bee/cells/a.json", r#"{"id":"a","feature":"bare"}"#);
        let card = scorecard_of(&root, None);
        let bare = scorecard_row(&card, "bare");
        assert_eq!(bare.judge_revision_rate, None);
        assert_eq!(bare.cycle_secs, None);
        assert_eq!(bare.gate_reopens, None);
        assert_eq!(bare.fix_first, None);
        assert_eq!(bare.cost_usd, None);
        assert_eq!(bare.total_tokens, None);
        assert_eq!(
            card.to_text().lines().nth(1).unwrap(),
            "bare: cells 1 · rework 0% · judge revisions not recorded · reactions 0% · cycle not recorded · gate reopens not recorded · fix-first not recorded · cost not recorded · tokens not recorded"
        );
    }

    #[test]
    fn scorecard_lane_without_cells_prints_not_recorded_rates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        scorecard_write(&root, ".bee/lanes/fresh.json", r#"{"feature":"fresh","created_at":"2026-11-01T00:00:00Z"}"#);
        let card = scorecard_of(&root, None);
        let fresh = scorecard_row(&card, "fresh");
        assert_eq!(fresh.cells, 0);
        assert_eq!(fresh.rework_rate, None);
        assert_eq!(fresh.reaction_coverage, None);
        assert_eq!(fresh.cycle_secs, None);
        assert_eq!(card.header(), "features: 1 (closed 0) · rework rate not recorded overall");
        let line = card.to_text().lines().nth(1).unwrap().to_string();
        assert!(line.starts_with("fresh: cells 0 · rework not recorded · judge revisions not recorded · reactions not recorded"), "{line}");
        assert!(!line.contains(" 0%"), "{line}");
    }

    #[test]
    fn scorecard_cycle_time_ends_at_the_usage_closed_at() {
        let (_tmp, root) = scorecard_store();
        scorecard_write(
            &root,
            ".bee/cells/c-22.json",
            r#"{"id":"c-22","feature":"slp-article-gaps","trace":{"capped_at":"2026-11-09T00:00:00Z"}}"#,
        );
        let card = scorecard_of(&root, None);
        assert_eq!(scorecard_row(&card, "slp-article-gaps").cycle_secs, Some(216_000));
    }

    #[test]
    fn scorecard_cycle_time_falls_back_to_the_newest_capped_at() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        let other = scorecard_row(&card, "other-feat");
        assert!(!other.closed);
        assert_eq!(other.cycle_secs, Some(19_800));
        assert!(card.to_text().contains("other-feat: cells 3 · rework 0% · judge revisions not recorded · reactions 33.3% · cycle 5h 30m · "));
    }

    #[test]
    fn scorecard_gate_reopens_count_gate_rows_and_skip_bypass_rows_beside_them() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        assert_eq!(scorecard_row(&card, "slp-article-gaps").gate_reopens, Some(2));
    }

    #[test]
    fn scorecard_gate_reopens_count_bypass_rows_when_no_gate_row_exists() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        assert_eq!(scorecard_row(&card, "other-feat").gate_reopens, Some(1));
    }

    #[test]
    fn scorecard_gate_reopens_ignore_other_kinds_and_true_sets() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().to_path_buf();
        scorecard_write(&root, ".bee/cells/a.json", r#"{"id":"a","feature":"f"}"#);
        scorecard_write(
            &root,
            ".bee/audit.jsonl",
            "{\"kind\":\"cap\",\"feature\":\"f\",\"note\":\"gate shape set to false by user\"}\n{\"kind\":\"gate\",\"feature\":\"f\",\"note\":\"gate shape set to true by user\"}\nnot json\n",
        );
        let card = scorecard_of(&root, None);
        assert_eq!(scorecard_row(&card, "f").gate_reopens, Some(0));
    }

    #[test]
    fn scorecard_fix_first_counts_cells_naming_the_feature() {
        let (_tmp, root) = scorecard_store();
        scorecard_write(
            &root,
            ".bee/cells/c-23.json",
            r#"{"id":"c-23","feature":"other-feat","fixes_feature":"slp-article-gaps"}"#,
        );
        let card = scorecard_of(&root, None);
        assert_eq!(scorecard_row(&card, "slp-article-gaps").fix_first, Some(2));
        assert_eq!(scorecard_row(&card, "other-feat").fix_first, Some(0));
    }

    #[test]
    fn scorecard_fix_first_survives_the_feature_filter_from_another_feature() {
        let (_tmp, root) = scorecard_store();
        scorecard_write(
            &root,
            ".bee/cells/c-23.json",
            r#"{"id":"c-23","feature":"other-feat","fixes_feature":"slp-article-gaps"}"#,
        );
        let unfiltered = scorecard_of(&root, None);
        let want = "slp-article-gaps";
        let filtered = scorecard(&collect_outcomes(&root, Some(want)), &read_inputs(&root), None, Some(want));
        assert_eq!(filtered.rows.len(), 1);
        assert_eq!(scorecard_row(&filtered, want).fix_first, Some(2));
        assert_eq!(scorecard_row(&filtered, want).fix_first, scorecard_row(&unfiltered, want).fix_first);
        assert_eq!(filtered.to_json()["features"][0]["fix_first_cells"], 2);
    }

    #[test]
    fn scorecard_cost_sums_usage_across_two_jobs() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        let slp = scorecard_row(&card, "slp-article-gaps");
        assert_eq!(slp.cost_usd, Some(0.75));
        assert_eq!(slp.total_tokens, Some(1500));
        let other = scorecard_row(&card, "other-feat");
        assert_eq!(other.cost_usd, None);
        assert_eq!(other.total_tokens, None);
    }

    #[test]
    fn scorecard_header_counts_closed_features_beside_overall_rework() {
        let (_tmp, root) = scorecard_store();
        let card = scorecard_of(&root, None);
        assert_eq!(card.header(), "features: 2 (closed 1) · rework rate 9.1% overall");
        let names: Vec<&str> = card.rows.iter().map(|r| r.feature.as_str()).collect();
        assert_eq!(names, ["slp-article-gaps", "other-feat"]);
        assert_eq!(card.to_text().lines().next().unwrap(), card.header());
    }

    #[test]
    fn scorecard_since_a_duration_keeps_features_started_inside_it() {
        let (_tmp, root) = scorecard_store();
        let bound = parse_since("7d", scorecard_stamp("2026-11-15T00:00:00Z")).unwrap();
        assert_eq!(bound, scorecard_stamp("2026-11-08T00:00:00Z"));
        let card = scorecard_of(&root, Some(bound));
        let names: Vec<&str> = card.rows.iter().map(|r| r.feature.as_str()).collect();
        assert_eq!(names, ["other-feat"]);
        assert_eq!(card.header(), "features: 1 (closed 0) · rework rate 0% overall");
        assert_eq!(parse_since("48h", scorecard_stamp("2026-11-15T00:00:00Z")), Some(scorecard_stamp("2026-11-13T00:00:00Z")));
    }

    #[test]
    fn scorecard_since_an_iso_bound_is_inclusive_and_drops_laneless_features() {
        let (_tmp, root) = scorecard_store();
        scorecard_write(&root, ".bee/cells/c-24.json", r#"{"id":"c-24","feature":"no-lane"}"#);
        let at = parse_since("2026-11-10T00:00:00Z", Utc::now()).unwrap();
        let names: Vec<String> = scorecard_of(&root, Some(at)).rows.into_iter().map(|r| r.feature).collect();
        assert_eq!(names, ["other-feat"]);
        let day = parse_since("2026-11-01", Utc::now()).unwrap();
        let names: Vec<String> = scorecard_of(&root, Some(day)).rows.into_iter().map(|r| r.feature).collect();
        assert_eq!(names, ["slp-article-gaps", "other-feat"]);
        let all: Vec<String> = scorecard_of(&root, None).rows.into_iter().map(|r| r.feature).collect();
        assert!(all.contains(&"no-lane".to_string()));
    }

    #[test]
    fn scorecard_flags_parse_by_feature_and_since() {
        let opts = parse(&[OsString::from("--by-feature"), OsString::from("--since"), OsString::from("7d")]).unwrap();
        assert!(opts.by_feature);
        assert!(opts.since.is_some());
        let opts = parse(&[OsString::from("--since=2026-11-01T00:00:00Z"), OsString::from("--by-feature")]).unwrap();
        assert_eq!(opts.since, Some(scorecard_stamp("2026-11-01T00:00:00Z")));
        assert!(parse(&[OsString::from("--since"), OsString::from("soon")]).is_none());
        assert!(parse(&[OsString::from("--since")]).is_none());
        let plain = parse(&[]).unwrap();
        assert!(!plain.by_feature);
        assert!(plain.since.is_none());
    }

    #[test]
    fn scorecard_json_carries_null_for_not_recorded() {
        let (_tmp, root) = scorecard_store();
        let json = scorecard_of(&root, None).to_json();
        assert_eq!(json["header"], "features: 2 (closed 1) · rework rate 9.1% overall");
        let slp = &json["features"][0];
        assert_eq!(slp["feature"], "slp-article-gaps");
        assert_eq!(slp["closed"], true);
        assert_eq!(slp["cycle_time_seconds"], 216_000);
        assert_eq!(slp["cost_usd"], 0.75);
        assert_eq!(slp["total_tokens"], 1500);
        let other = &json["features"][1];
        assert_eq!(other["feature"], "other-feat");
        assert_eq!(other["rework_rate"], 0.0);
        assert!(other["judge_revision_rate"].is_null());
        assert!(other["cost_usd"].is_null());
        assert!(other["total_tokens"].is_null());
        assert_eq!(other["fix_first_cells"], 0);

        let tmp = tempfile::tempdir().unwrap();
        scorecard_write(tmp.path(), ".bee/cells/a.json", r#"{"id":"a","feature":"bare"}"#);
        let bare = scorecard_of(tmp.path(), None).to_json();
        for key in ["created_at", "judge_revision_rate", "cycle_time_seconds", "gate_reopens", "fix_first_cells", "cost_usd", "total_tokens"] {
            assert!(bare["features"][0][key].is_null(), "{key} must be null");
        }
    }

    #[test]
    fn scorecard_sources_leave_the_no_flag_output_on_its_golden() {
        let (_tmp, root) = setup_fixture_store();
        let golden = [
            "worker reaction: 6 cells (3 changed) — confirm: 2, partial: 1, challenge: 1, blocked: 1, null: 1, absent: 13",
            "dissent: 4 cells, 4 records (2 changed) — accept: 1, reject: 1, escalate: 1, unanswered: 1",
            "semantic judge: 3 cells, 3 records (1 changed) — PASS: 1, NEEDS_REVISION: 1, other: 1",
            "leader check: 2 cells, 2 records (1 changed) — gap: 1, ok: 1",
            "rework: 2 cells (2 changed)",
            "role reroute: 1 cells, 1 records (1 changed)",
            "plan departure: 7 cells, 8 records — hit an unforeseen obstacle: 2, found a better route: 2, the plan was wrong about a fact: 1, something else had to be fixed first: 1, followed the plan: 1, other: 1",
        ];
        let before = collect_outcomes(&root, Some("slp-article-gaps"));
        let before_text = render_text(&before);
        let before_json = render_json(&before);
        let lines: Vec<&str> = before_text.lines().collect();
        assert_eq!(lines[..7], golden);
        assert!(lines[7].starts_with("unreadable: 1 ("));

        scorecard_add_sources(&root);
        let after = collect_outcomes(&root, Some("slp-article-gaps"));
        assert_eq!(render_text(&after), before_text);
        assert_eq!(render_json(&after), before_json);
        let after_json = render_json(&after);
        assert!(after_json.get("features").is_none() && after_json.get("header").is_none());
    }
}
