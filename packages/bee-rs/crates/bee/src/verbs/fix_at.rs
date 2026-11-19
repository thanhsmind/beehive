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

    fn finding(ts: &str, layer: &str, title: &str, feature: &str) -> Value {
        json!({"ts": ts, "type": "finding", "title": title, "layer": layer, "feature": feature})
    }

    fn close(ts: &str, layer: Option<&str>, title: &str) -> Value {
        match layer {
            Some(l) => json!({"ts": ts, "type": "backlog-closed", "title": title, "layer": l}),
            None => json!({"ts": ts, "kind": "backlog-closed", "title": title}),
        }
    }

    fn state(ledger: &Ledger, layer: &str) -> GroupState {
        ledger.groups.iter().find(|g| g.layer == layer).unwrap().state
    }

    #[test]
    fn fix_at_ledger_folds_one_group_per_layer_and_normalized_title() {
        let rows = vec![
            finding("2026-01-01", "fix-at:check", "Strip  Prefix", "a"),
            finding("2026-01-02", "fix-at:check", "strip prefix", "b"),
            finding("2026-01-03", "fix-at:architecture", "strip prefix", "c"),
            finding("2026-01-04", "other", "strip prefix", "d"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.groups.len(), 2);
        let check = ledger.groups.iter().find(|g| g.layer == "fix-at:check").unwrap();
        assert_eq!(check.findings, 2);
        assert_eq!(check.title, "strip prefix");
        assert_eq!(check.oldest_open_ts.as_deref(), Some("2026-01-01"));
        assert_eq!(check.features, vec!["a", "b"]);
        assert_eq!(check.state, GroupState::Open);
    }

    #[test]
    fn fix_at_ledger_close_turns_closed_and_a_newer_finding_turns_recurred() {
        let mut rows = vec![finding("2026-01-01", "fix-at:check", "t", "a"), close("2026-01-02", None, "T")];
        assert_eq!(state(&Ledger::from_rows(&rows), "fix-at:check"), GroupState::Closed);
        rows.push(finding("2026-01-03", "fix-at:check", "t", "b"));
        let ledger = Ledger::from_rows(&rows);
        let group = &ledger.groups[0];
        assert_eq!(group.state, GroupState::Recurred);
        assert_eq!(group.oldest_open_ts.as_deref(), Some("2026-01-03"));
        assert_eq!(group.features, vec!["b"]);
        assert!(group.latest_close.is_some());
    }

    #[test]
    fn fix_at_ledger_layered_close_matches_only_its_own_layer() {
        let rows = vec![
            finding("2026-01-01", "fix-at:check", "t", "a"),
            finding("2026-01-01", "fix-at:architecture", "t", "a"),
            close("2026-01-02", Some("fix-at:check"), "t"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(state(&ledger, "fix-at:check"), GroupState::Closed);
        assert_eq!(state(&ledger, "fix-at:architecture"), GroupState::Open);
    }

    #[test]
    fn fix_at_ledger_current_epoch_feature_follows_the_latest_close() {
        let rows = vec![
            finding("2026-01-01", "fix-at:check", "t", "old"),
            close("2026-01-02", None, "t"),
            finding("2026-01-03", "fix-at:check", "t", "new"),
        ];
        let ledger = Ledger::from_rows(&rows);
        assert_eq!(ledger.current_epoch_feature("fix-at:check", "T"), Some("new".to_string()));
        assert_eq!(ledger.current_epoch_feature("fix-at:architecture", "t"), None);
        let closed = Ledger::from_rows(&rows[..2]);
        assert_eq!(closed.current_epoch_feature("fix-at:check", "t"), None);
    }
}
