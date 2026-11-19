use super::backlog::is_finding_row;
use super::feedback::{normalize_title, read_jsonl, resolve_in_scope, Scope};
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) const LAYERS: [&str; 3] = ["fix-at:check", "fix-at:architecture", "fix-at:rejudge"];

const SRC_BACKLOG: &str = ".bee/backlog.jsonl";

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
}
