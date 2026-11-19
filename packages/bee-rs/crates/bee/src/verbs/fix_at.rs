use super::backlog::{self, is_finding_row};
use super::feedback::{normalize_title, read_jsonl, resolve_in_scope, Scope};
use super::mailbox::{self, Entry};
use super::mailbox_digest::{normalize_shape, MIN_SHAPE_WORDS};
use crate::fsutil::append_jsonl;
use crate::textutil::truncate_chars_head;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

pub(crate) const LAYERS: [&str; 3] = ["fix-at:check", "fix-at:architecture", "fix-at:rejudge"];

const SRC_BACKLOG: &str = ".bee/backlog.jsonl";

const UNATTRIBUTED: &str = "unattributed";
const REJUDGE: &str = "fix-at:rejudge";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GroupState {
    Open,
    Closed,
    Recurred,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct Group {
    pub layer: String,
    pub title: String,
    pub key: String,
    pub state: GroupState,
    pub findings: usize,
    pub oldest_open_ts: Option<String>,
    pub newest_finding_ts: String,
    pub latest_close: Option<Value>,
    pub features: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Ledger {
    pub groups: Vec<Group>,
}

fn field<'a>(row: &'a Value, name: &str) -> &'a str {
    row.get(name).and_then(Value::as_str).unwrap_or("")
}

fn is_close(row: &Value) -> bool {
    field(row, "type") == "backlog-closed" || field(row, "kind") == "backlog-closed"
}

fn title_key(row: &Value) -> String {
    normalize_title(row.get("title").unwrap_or(&Value::Null))
}

fn close_matches(close: &Value, layer: &str, key: &str) -> bool {
    let close_layer = field(close, "layer");
    (!LAYERS.contains(&close_layer) || close_layer == layer) && title_key(close) == key
}

struct Pending<'a> {
    title: &'a str,
    findings: Vec<&'a Value>,
}

impl Ledger {
    pub(crate) fn from_rows(rows: &[Value]) -> Ledger {
        let closes: Vec<&Value> = rows.iter().filter(|r| is_close(r)).collect();
        let mut pending: BTreeMap<(&str, String), Pending> = BTreeMap::new();
        for row in rows.iter().filter(|r| is_finding_row(r)) {
            let Some(layer) = LAYERS.iter().copied().find(|l| *l == field(row, "layer")) else {
                continue;
            };
            let slot = pending
                .entry((layer, title_key(row)))
                .or_insert_with(|| Pending { title: "", findings: Vec::new() });
            slot.findings.push(row);
        }
        let groups = pending
            .into_iter()
            .map(|((layer, key), mut slot)| {
                let newest_finding_ts = slot.findings.iter().map(|r| field(r, "ts")).max().unwrap_or("").to_string();
                slot.title = slot
                    .findings
                    .iter()
                    .rev()
                    .find(|r| field(r, "ts") == newest_finding_ts)
                    .map(|r| field(r, "title"))
                    .unwrap_or("");
                let latest_close = closes
                    .iter()
                    .filter(|c| close_matches(c, layer, &key))
                    .max_by(|a, b| field(a, "ts").cmp(field(b, "ts")))
                    .map(|c| (*c).clone());
                let close_ts = latest_close.as_ref().map(|c| field(c, "ts").to_string());
                let state = match &close_ts {
                    None => GroupState::Open,
                    Some(c) if c.as_str() > newest_finding_ts.as_str() => GroupState::Closed,
                    Some(_) => GroupState::Recurred,
                };
                let epoch: Vec<&&Value> = slot
                    .findings
                    .iter()
                    .filter(|r| close_ts.as_deref().is_none_or(|c| field(r, "ts") > c))
                    .collect();
                let mut features: Vec<String> = Vec::new();
                for r in &epoch {
                    let feature = field(r, "feature").to_string();
                    if !features.contains(&feature) {
                        features.push(feature);
                    }
                }
                Group {
                    layer: layer.to_string(),
                    title: slot.title.to_string(),
                    key,
                    state,
                    findings: slot.findings.len(),
                    oldest_open_ts: epoch.iter().map(|r| field(r, "ts")).min().map(str::to_string),
                    newest_finding_ts,
                    latest_close,
                    features,
                }
            })
            .collect();
        Ledger { groups }
    }

    pub(crate) fn load(root: &Path) -> Option<Ledger> {
        let real_root = dunce::canonicalize(root).ok()?;
        let Scope::Ok(path) = resolve_in_scope(&real_root, SRC_BACKLOG) else {
            return None;
        };
        Some(Ledger::from_rows(&read_jsonl(&path).rows))
    }

    pub(crate) fn current_epoch_feature(&self, layer: &str, title: &str) -> Option<String> {
        let key = normalize_title(&Value::String(title.to_string()));
        self.groups
            .iter()
            .find(|g| g.layer == layer && g.key == key)
            .and_then(|g| g.features.first().cloned())
    }
}

pub(crate) fn shape_key(text: &str) -> Option<String> {
    let normalized = normalize_shape(text);
    let head: Vec<&str> = normalized.split_whitespace().take(MIN_SHAPE_WORDS).collect();
    (head.len() == MIN_SHAPE_WORDS).then(|| head.join(" "))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Filing {
    New,
    Skip,
    UnderClosed(String),
}

fn close_ts(group: &Group) -> Option<&str> {
    group.latest_close.as_ref().map(|c| field(c, "ts"))
}

fn matching_group<'a>(ledger: &'a Ledger, layer: &str, title: &str) -> Option<&'a Group> {
    let exact = normalize_title(&Value::String(title.to_string()));
    let shape = shape_key(title);
    let hits = ledger
        .groups
        .iter()
        .filter(|g| g.layer == layer && (g.key == exact || (shape.is_some() && shape_key(&g.title) == shape)));
    hits.clone()
        .find(|g| g.state != GroupState::Closed)
        .or_else(|| hits.max_by(|a, b| close_ts(a).cmp(&close_ts(b))))
}

pub(crate) fn alias_title(ledger: &Ledger, layer: &str, title: &str) -> Filing {
    match matching_group(ledger, layer, title) {
        None => Filing::New,
        Some(g) if g.state == GroupState::Closed => Filing::UnderClosed(g.title.clone()),
        Some(_) => Filing::Skip,
    }
}

pub(crate) fn filing_detail(better: &str, source: &str, wrong: &str, filing: &Filing) -> String {
    match filing {
        Filing::UnderClosed(_) => format!("{better} (from {source}; reworded from: {wrong})"),
        _ => format!("{better} (from {source})"),
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SweepReport {
    pub filed: usize,
    pub rejudged: usize,
}

fn file_row(control: &Path, ledger: &mut Ledger, layer: &str, title: &str, detail: &str) -> bool {
    let row = backlog::backlog_finding_row("finding", title, detail, "P3", layer, UNATTRIBUTED);
    if append_jsonl(&backlog::backlog_jsonl_path(control), &Value::Object(row)).is_err() {
        return false;
    }
    if let Some(fresh) = Ledger::load(control) {
        *ledger = fresh;
    }
    true
}

fn layer_of(entry: &Entry) -> &str {
    match entry.fix_at.as_deref().map(str::trim) {
        Some("") | None => "none",
        Some(layer) => layer,
    }
}

pub(crate) fn sweep_unfiled(root: &Path) -> SweepReport {
    let control = crate::hooks::session_init::control_root_for(root);
    let mut ledger = Ledger::load(&control).unwrap_or_default();
    let runs: Vec<(String, Vec<Entry>)> = mailbox::runs_with_entries(&control)
        .into_iter()
        .map(|run| {
            let entries = mailbox::read_entries(&control, &run).into_iter().filter(Entry::is_reflection).collect();
            (run, entries)
        })
        .collect();
    let mut report = SweepReport::default();
    for (run, entries) in &runs {
        for entry in entries {
            let layer = match layer_of(entry) {
                "check" => "fix-at:check",
                "architecture" => "fix-at:architecture",
                _ => continue,
            };
            let wrong = truncate_chars_head(&entry.what, backlog::BACKLOG_MAX_TITLE);
            if wrong.trim().is_empty() {
                continue;
            }
            let filing = alias_title(&ledger, layer, &wrong);
            let title = match &filing {
                Filing::Skip => continue,
                Filing::New => wrong.clone(),
                Filing::UnderClosed(closed) => {
                    let newer = matching_group(&ledger, layer, &wrong)
                        .and_then(close_ts)
                        .is_none_or(|c| entry.at.as_str() > c);
                    if !newer {
                        continue;
                    }
                    closed.clone()
                }
            };
            let better = entry.better.as_deref().unwrap_or_default();
            let detail = filing_detail(better, &format!("run {run}"), &entry.what, &filing);
            if file_row(&control, &mut ledger, layer, &title, &detail) {
                report.filed += 1;
            }
        }
    }
    let mut by_key: BTreeMap<String, Vec<(&str, &Entry)>> = BTreeMap::new();
    for (run, entries) in &runs {
        for entry in entries.iter().filter(|e| matches!(layer_of(e), "doctrine" | "none")) {
            if let Some(key) = shape_key(&entry.what) {
                by_key.entry(key).or_default().push((run, entry));
            }
        }
    }
    for (key, hits) in by_key {
        let groups: Vec<&Group> =
            ledger.groups.iter().filter(|g| shape_key(&g.title).as_deref() == Some(key.as_str())).collect();
        if groups.iter().any(|g| g.state != GroupState::Closed) {
            continue;
        }
        let latest_close = groups.iter().filter_map(|g| close_ts(g)).max();
        let counted: Vec<&(&str, &Entry)> =
            hits.iter().filter(|(_, e)| latest_close.is_none_or(|c| e.at.as_str() > c)).collect();
        let distinct: BTreeSet<&str> = counted.iter().map(|(run, _)| *run).collect();
        let Some((_, earliest)) = counted.iter().min_by(|a, b| a.1.at.cmp(&b.1.at)).map(|h| **h) else {
            continue;
        };
        if distinct.len() < 2 {
            continue;
        }
        let title = truncate_chars_head(&earliest.what, backlog::BACKLOG_MAX_TITLE);
        let detail = format!(
            "Labelled doctrine or none in {} runs ({}): decide again whether a check or a code change stops it.",
            distinct.len(),
            distinct.iter().copied().collect::<Vec<_>>().join(", ")
        );
        if file_row(&control, &mut ledger, REJUDGE, &title, &detail) {
            report.rejudged += 1;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const CHECK: &str = "fix-at:check";
    const ARCH: &str = "fix-at:architecture";

    fn finding(ts: &str, layer: &str, title: &str, feature: &str) -> Value {
        json!({"ts": ts, "type": "finding", "title": title, "layer": layer, "feature": feature})
    }

    fn typed_close(ts: &str, layer: Option<&str>, title: &str) -> Value {
        match layer {
            Some(l) => json!({"ts": ts, "type": "backlog-closed", "title": title, "layer": l}),
            None => json!({"ts": ts, "type": "backlog-closed", "title": title}),
        }
    }

    fn group<'a>(ledger: &'a Ledger, layer: &str) -> &'a Group {
        ledger.groups.iter().find(|g| g.layer == layer).unwrap()
    }

    #[test]
    fn fix_at_ledger_whitespace_variant_titles_fold_into_one_open_group() {
        let rows = vec![
            finding("2026-01-02T00:00:00Z", CHECK, "  strip prefix ", "b"),
            finding("2026-01-01T00:00:00Z", CHECK, "Strip  Prefix", "a"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.groups.len(), 1);
        let g = &ledger.groups[0];
        assert_eq!(g.state, GroupState::Open);
        assert_eq!(g.findings, 2);
        assert_eq!(g.oldest_open_ts.as_deref(), Some("2026-01-01T00:00:00Z"));
        assert!(g.latest_close.is_none());
    }

    #[test]
    fn fix_at_ledger_newer_close_turns_the_group_closed_and_is_its_latest_close() {
        let closing = typed_close("2026-01-03T00:00:00Z", None, "Strip Prefix");
        let rows = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "strip prefix", "a"),
            typed_close("2026-01-02T00:00:00Z", None, "strip prefix"),
            closing.clone(),
            finding("2026-01-01T12:00:00Z", CHECK, "strip prefix", "b"),
        ];
        let ledger = Ledger::from_rows(&rows);
        let g = group(&ledger, CHECK);
        assert_eq!(g.state, GroupState::Closed);
        assert_eq!(g.latest_close.as_ref(), Some(&closing));
        assert_eq!(g.oldest_open_ts, None);
    }

    #[test]
    fn fix_at_ledger_finding_newer_than_the_close_turns_recurred_from_that_finding() {
        let rows = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "t", "a"),
            typed_close("2026-01-02T00:00:00Z", None, "t"),
            finding("2026-01-04T00:00:00Z", CHECK, "t", "c"),
            finding("2026-01-03T00:00:00Z", CHECK, "t", "b"),
        ];
        let ledger = Ledger::from_rows(&rows);
        let g = group(&ledger, CHECK);
        assert_eq!(g.state, GroupState::Recurred);
        assert_eq!(g.findings, 3);
        assert_eq!(g.oldest_open_ts.as_deref(), Some("2026-01-03T00:00:00Z"));
    }

    #[test]
    fn fix_at_ledger_check_and_architecture_with_one_title_are_two_groups() {
        let rows = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "same title", "a"),
            finding("2026-01-02T00:00:00Z", ARCH, "same title", "b"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.groups.len(), 2);
        assert_eq!(group(&ledger, CHECK).findings, 1);
        assert_eq!(group(&ledger, ARCH).findings, 1);
    }

    #[test]
    fn fix_at_ledger_ignores_other_layers_and_non_finding_rows() {
        let rows = vec![
            finding("2026-01-01T00:00:00Z", "other", "t", "a"),
            json!({"ts": "2026-01-02T00:00:00Z", "type": "finding", "title": "t", "feature": "a"}),
            json!({"ts": "2026-01-03T00:00:00Z", "kind": "pbi", "type": "finding", "title": "t", "layer": CHECK}),
            finding("2026-01-04T00:00:00Z", CHECK, "t", "b"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.groups.len(), 1);
        let g = group(&ledger, CHECK);
        assert_eq!(g.findings, 1);
        assert_eq!(g.oldest_open_ts.as_deref(), Some("2026-01-04T00:00:00Z"));
    }

    #[test]
    fn fix_at_ledger_current_epoch_feature_names_only_the_current_epoch_owner() {
        let rows = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "t", "old"),
            typed_close("2026-01-02T00:00:00Z", None, "t"),
            finding("2026-01-03T00:00:00Z", CHECK, "t", "new"),
        ];
        let recurred = Ledger::from_rows(&rows);
        assert_eq!(recurred.current_epoch_feature(CHECK, " T "), Some("new".to_string()));
        assert_eq!(recurred.current_epoch_feature(ARCH, "t"), None);
        let closed = Ledger::from_rows(&rows[..2]);
        assert_eq!(closed.current_epoch_feature(CHECK, "t"), None);
        let open = Ledger::from_rows(&rows[..1]);
        assert_eq!(open.current_epoch_feature(CHECK, "t"), Some("old".to_string()));
    }

    #[test]
    fn fix_at_ledger_kind_backlog_closed_row_without_type_closes_the_group() {
        let closing = json!({"ts": "2026-01-02T00:00:00Z", "kind": "backlog-closed", "title": "t"});
        let rows = vec![finding("2026-01-01T00:00:00Z", CHECK, "t", "a"), closing.clone()];
        let ledger = Ledger::from_rows(&rows);
        let g = group(&ledger, CHECK);
        assert_eq!(g.state, GroupState::Closed);
        assert_eq!(g.latest_close.as_ref(), Some(&closing));
    }

    #[test]
    fn fix_at_ledger_layered_close_closes_its_layer_and_layerless_close_closes_both() {
        let findings = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "t", "a"),
            finding("2026-01-01T00:00:00Z", ARCH, "t", "a"),
        ];
        let mut layered = findings.clone();
        layered.push(typed_close("2026-01-02T00:00:00Z", Some(CHECK), "t"));
        let ledger = Ledger::from_rows(&layered);
        assert_eq!(group(&ledger, CHECK).state, GroupState::Closed);
        assert_eq!(group(&ledger, ARCH).state, GroupState::Open);
        let mut layerless = findings;
        layerless.push(typed_close("2026-01-02T00:00:00Z", None, "t"));
        let ledger = Ledger::from_rows(&layerless);
        assert_eq!(group(&ledger, CHECK).state, GroupState::Closed);
        assert_eq!(group(&ledger, ARCH).state, GroupState::Closed);
    }

    #[test]
    fn fix_at_ledger_non_finding_row_type_on_a_fix_at_layer_is_not_counted() {
        let rows = vec![
            json!({"ts": "2026-01-01T00:00:00Z", "type": "note", "title": "t", "layer": CHECK, "feature": "a"}),
            json!({"ts": "2026-01-02T00:00:00Z", "type": "finding", "title": "u", "layer": ARCH, "feature": "a"}),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.groups.len(), 1);
        assert_eq!(ledger.groups[0].layer, ARCH);
    }

    fn sandbox() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join(".bee")).unwrap();
        tmp
    }

    fn seed_rows(root: &Path, rows: &[Value]) {
        let path = backlog::backlog_jsonl_path(root);
        for row in rows {
            append_jsonl(&path, row).unwrap();
        }
    }

    fn reflect(root: &Path, run: &str, at: &str, wrong: &str, layer: &str) {
        let mut entry = Entry::reflection(at, wrong, "do it better", layer);
        entry.at = at.to_string();
        mailbox::append_entry(root, run, &entry).unwrap();
    }

    fn backlog_rows(root: &Path) -> Vec<Value> {
        read_jsonl(&backlog::backlog_jsonl_path(root)).rows
    }

    const WRONG: &str = "Ran the whole suite for a one-line doc fix";

    #[test]
    fn fix_at_shape_key_is_the_first_four_normalized_words_and_none_under_four() {
        assert_eq!(shape_key("Ran the WHOLE suite again.").as_deref(), Some("ran the whole suite"));
        assert_eq!(shape_key("ran 12 tests of 30 files"), shape_key("Ran 7 tests of 99 more"));
        assert_eq!(shape_key("only three words"), None);
    }

    #[test]
    fn fix_at_alias_title_skips_a_reworded_open_group_and_files_under_a_closed_one() {
        let rows = vec![
            finding("2026-01-01T00:00:00Z", CHECK, "Ran the whole suite for a doc fix", "a"),
            finding("2026-01-01T00:00:00Z", ARCH, "Reserved a file after writing it", "a"),
            typed_close("2026-01-02T00:00:00Z", Some(ARCH), "Reserved a file after writing it"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(alias_title(&ledger, CHECK, WRONG), Filing::Skip);
        assert_eq!(alias_title(&ledger, CHECK, "Something else entirely different here"), Filing::New);
        assert_eq!(
            alias_title(&ledger, ARCH, "Reserved a file after writing it twice"),
            Filing::UnderClosed("Reserved a file after writing it".to_string())
        );
        assert_eq!(alias_title(&ledger, CHECK, "Reserved a file after writing it twice"), Filing::New);
    }

    #[test]
    fn fix_at_sweep_files_an_unfiled_check_reflection_once_under_unattributed() {
        let tmp = sandbox();
        let root = tmp.path();
        reflect(root, "run-a", "2026-01-05T00:00:00Z", WRONG, "check");
        let first = sweep_unfiled(root);
        assert_eq!(first, SweepReport { filed: 1, rejudged: 0 });
        let rows = backlog_rows(root);
        assert_eq!(rows.len(), 1);
        assert_eq!(field(&rows[0], "feature"), "unattributed");
        assert_eq!(field(&rows[0], "layer"), CHECK);
        assert_eq!(field(&rows[0], "title"), WRONG);
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    #[test]
    fn fix_at_sweep_leaves_a_mistake_close_already_filed_alone() {
        let tmp = sandbox();
        let root = tmp.path();
        seed_rows(root, &[finding("2026-01-04T00:00:00Z", CHECK, WRONG, "demo")]);
        reflect(root, "run-a", "2026-01-05T00:00:00Z", WRONG, "check");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    #[test]
    fn fix_at_sweep_files_a_reworded_recurrence_under_the_closed_title_and_turns_it_recurred() {
        let tmp = sandbox();
        let root = tmp.path();
        seed_rows(
            root,
            &[
                finding("2026-01-01T00:00:00Z", CHECK, WRONG, "demo"),
                typed_close("2026-01-02T00:00:00Z", Some(CHECK), WRONG),
            ],
        );
        reflect(root, "run-old", "2026-01-01T12:00:00Z", WRONG, "check");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        reflect(root, "run-new", "2026-01-03T00:00:00Z", "Ran the whole suite again, for typo fixes", "check");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 1, rejudged: 0 });
        let rows = backlog_rows(root);
        let filed = rows.last().unwrap();
        assert_eq!(field(filed, "title"), WRONG);
        assert!(field(filed, "detail").contains("reworded from: Ran the whole suite again, for typo fixes"));
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(group(&ledger, CHECK).state, GroupState::Recurred);
        assert_eq!(sweep_unfiled(root), SweepReport::default());
    }

    #[test]
    fn fix_at_sweep_files_one_rejudge_row_for_a_doctrine_key_seen_in_two_runs() {
        let tmp = sandbox();
        let root = tmp.path();
        reflect(root, "run-a", "2026-01-05T00:00:00Z", "Skipped reading the spec before editing code", "doctrine");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        reflect(root, "run-a", "2026-01-05T01:00:00Z", "Skipped reading the spec before editing code", "doctrine");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        reflect(root, "run-b", "2026-01-06T00:00:00Z", "Skipped reading the spec before editing tests", "none");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 0, rejudged: 1 });
        let rows = backlog_rows(root);
        assert_eq!(rows.len(), 1);
        assert_eq!(field(&rows[0], "layer"), REJUDGE);
        assert_eq!(field(&rows[0], "feature"), "unattributed");
        assert_eq!(field(&rows[0], "title"), "Skipped reading the spec before editing code");
        assert!(field(&rows[0], "detail").contains("run-a, run-b"));
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    #[test]
    fn fix_at_sweep_holds_a_kept_rejudge_close_until_two_newer_runs_repeat_the_key() {
        let tmp = sandbox();
        let root = tmp.path();
        let wrong = "Skipped reading the spec before editing code";
        seed_rows(
            root,
            &[
                finding("2026-01-02T00:00:00Z", REJUDGE, wrong, "unattributed"),
                typed_close("2026-01-03T00:00:00Z", Some(REJUDGE), wrong),
            ],
        );
        reflect(root, "run-a", "2026-01-01T00:00:00Z", wrong, "doctrine");
        reflect(root, "run-b", "2026-01-02T00:00:00Z", wrong, "none");
        reflect(root, "run-c", "2026-01-04T00:00:00Z", wrong, "doctrine");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        reflect(root, "run-d", "2026-01-05T00:00:00Z", wrong, "none");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 0, rejudged: 1 });
    }

    #[test]
    fn fix_at_sweep_does_not_rejudge_a_key_an_open_check_row_already_carries() {
        let tmp = sandbox();
        let root = tmp.path();
        seed_rows(root, &[finding("2026-01-01T00:00:00Z", CHECK, "Skipped reading the spec first", "a")]);
        reflect(root, "run-a", "2026-01-05T00:00:00Z", "Skipped reading the spec before editing code", "doctrine");
        reflect(root, "run-b", "2026-01-06T00:00:00Z", "Skipped reading the spec before editing tests", "none");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    const LINT: &str = "Pushed before running the clippy lint";

    #[test]
    fn fix_at_intake_a_check_reflection_with_no_close_files_once_as_unattributed() {
        let tmp = sandbox();
        let root = tmp.path();
        reflect(root, "run-solo", "2026-02-01T00:00:00Z", LINT, "check");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 1, rejudged: 0 });
        let rows = backlog_rows(root);
        assert_eq!(rows.len(), 1);
        assert_eq!(field(&rows[0], "type"), "finding");
        assert_eq!(field(&rows[0], "severity"), "P3");
        assert_eq!(field(&rows[0], "layer"), CHECK);
        assert_eq!(field(&rows[0], "feature"), "unattributed");
        assert_eq!(field(&rows[0], "title"), LINT);
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    #[test]
    fn fix_at_intake_a_check_reflection_older_than_the_latest_close_is_not_filed() {
        let tmp = sandbox();
        let root = tmp.path();
        seed_rows(
            root,
            &[
                finding("2026-02-01T00:00:00Z", CHECK, LINT, "demo"),
                typed_close("2026-02-03T00:00:00Z", Some(CHECK), LINT),
            ],
        );
        reflect(root, "run-before", "2026-02-02T00:00:00Z", LINT, "check");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 2);
        assert_eq!(group(&Ledger::from_rows(&backlog_rows(root)), CHECK).state, GroupState::Closed);
    }

    #[test]
    fn fix_at_intake_a_doctrine_key_in_two_runs_files_one_rejudge_row_and_one_run_files_none() {
        let tmp = sandbox();
        let root = tmp.path();
        let twice = "Answered the user before reading the log";
        let once = "Guessed the config path without looking";
        reflect(root, "run-a", "2026-02-01T00:00:00Z", twice, "doctrine");
        reflect(root, "run-a", "2026-02-01T01:00:00Z", once, "doctrine");
        reflect(root, "run-b", "2026-02-02T00:00:00Z", "Answered the user before reading anything", "none");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 0, rejudged: 1 });
        let rows = backlog_rows(root);
        assert_eq!(rows.len(), 1);
        assert_eq!(field(&rows[0], "layer"), REJUDGE);
        assert_eq!(field(&rows[0], "title"), twice);
        assert!(rows.iter().all(|r| field(r, "title") != once));
    }

    #[test]
    fn fix_at_intake_a_rejudge_group_closed_with_a_kept_row_survives_a_second_sweep() {
        let tmp = sandbox();
        let root = tmp.path();
        let wrong = "Answered the user before reading the log";
        reflect(root, "run-a", "2026-02-01T00:00:00Z", wrong, "doctrine");
        reflect(root, "run-b", "2026-02-02T00:00:00Z", wrong, "none");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 0, rejudged: 1 });
        seed_rows(
            root,
            &[json!({"ts": "2099-01-01T00:00:00Z", "type": "backlog-closed", "title": wrong, "layer": REJUDGE, "kept": "a doctrine line is the right home"})],
        );
        assert_eq!(group(&Ledger::from_rows(&backlog_rows(root)), REJUDGE).state, GroupState::Closed);
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 2);
    }

    #[test]
    fn fix_at_intake_an_open_check_finding_with_the_same_key_suppresses_the_rejudge_row() {
        let tmp = sandbox();
        let root = tmp.path();
        seed_rows(root, &[finding("2026-01-15T00:00:00Z", CHECK, "Answered the user before checking CI", "a")]);
        reflect(root, "run-a", "2026-02-01T00:00:00Z", "Answered the user before reading the log", "doctrine");
        reflect(root, "run-b", "2026-02-02T00:00:00Z", "Answered the user before reading anything", "none");
        assert_eq!(sweep_unfiled(root), SweepReport::default());
        assert_eq!(backlog_rows(root).len(), 1);
    }

    #[test]
    fn fix_at_intake_the_sweep_files_a_reworded_mistake_under_the_closed_title_keeping_the_wording() {
        let tmp = sandbox();
        let root = tmp.path();
        let closed = "Shared the cache between two workers";
        let reworded = "Shared the cache between all workers again";
        seed_rows(
            root,
            &[
                finding("2026-02-01T00:00:00Z", ARCH, closed, "demo"),
                typed_close("2026-02-02T00:00:00Z", Some(ARCH), closed),
            ],
        );
        reflect(root, "run-late", "2026-02-03T00:00:00Z", reworded, "architecture");
        assert_eq!(sweep_unfiled(root), SweepReport { filed: 1, rejudged: 0 });
        let rows = backlog_rows(root);
        assert_eq!(rows.len(), 3);
        let filed = &rows[2];
        assert_eq!(field(filed, "title"), closed);
        assert_eq!(field(filed, "layer"), ARCH);
        assert_eq!(field(filed, "feature"), "unattributed");
        assert!(field(filed, "detail").contains(&format!("reworded from: {reworded}")));
        assert_eq!(group(&Ledger::from_rows(&rows), ARCH).state, GroupState::Recurred);
    }
}
