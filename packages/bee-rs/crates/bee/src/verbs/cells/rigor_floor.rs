use super::*;
use crate::fsutil::{read_json, ReadJson};
use crate::jsjson;
use crate::hooks::write_guard::{glob_match, glob_tokens, GlobTok};
use crate::state as bstate;
use crate::verbs::state_group::{route_belongs_to_feature, ROUTE_FLAG_VALUES};
use crate::verbs::workflow_store::{find_live_workflow, list_workflows, read_lane_display};
use serde_json::{Map, Value};
use std::path::Path;

const BUILT_IN_SOURCE: &str = "built-in";
const REPO_SOURCE: &str = ".bee/config.json";
const DATA_MODEL: &[&str] = &["data-model"];

const BUILT_IN_ROWS: [(&str, &str, &[&str], bool); 8] = [
    ("**/migrations/**", "high-risk", DATA_MODEL, true),
    ("**/migrate/**", "high-risk", DATA_MODEL, true),
    ("**/alembic/versions/**", "high-risk", DATA_MODEL, true),
    ("**/schema.sql", "high-risk", DATA_MODEL, true),
    ("**/*.schema.sql", "high-risk", DATA_MODEL, true),
    ("**/schema.prisma", "high-risk", DATA_MODEL, true),
    (".github/workflows/release*", "standard", &[], true),
    (".github/workflows/publish*", "standard", &[], true),
];

struct FloorRow {
    glob: String,
    tokens: Vec<GlobTok>,
    min_lane: String,
    flags: Vec<String>,
    owner_first: bool,
    source: &'static str,
}

pub(crate) struct FloorHit {
    pub(crate) path: String,
    pub(crate) glob: String,
    pub(crate) source: &'static str,
    pub(crate) min_lane: String,
    pub(crate) flags: Vec<String>,
    pub(crate) owner_first: bool,
}

pub(crate) fn rigor_lane_rank(lane: &str) -> Option<u8> {
    match lane {
        "docs" => Some(0),
        "tiny" => Some(1),
        "small" => Some(2),
        "standard" | "spike" => Some(3),
        "high-risk" => Some(4),
        _ => None,
    }
}

fn repo_row(value: &Value) -> Option<FloorRow> {
    let row = value.as_object()?;
    let glob = row.get("glob").and_then(Value::as_str).map(js_trim).filter(|g| !g.is_empty())?;
    let min_lane = row.get("min_lane").and_then(Value::as_str).filter(|l| rigor_lane_rank(l).is_some())?;
    let flags = match row.get("flags") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(items)) => items
            .iter()
            .map(|f| f.as_str().filter(|f| ROUTE_FLAG_VALUES.contains(f)).map(str::to_string))
            .collect::<Option<Vec<String>>>()?,
        Some(_) => return None,
    };
    let owner_first = match row.get("owner_first") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return None,
    };
    Some(FloorRow {
        glob: glob.to_string(),
        tokens: glob_tokens(glob),
        min_lane: min_lane.to_string(),
        flags,
        owner_first,
        source: REPO_SOURCE,
    })
}

fn floor_rows(root: &Path) -> Vec<FloorRow> {
    let mut rows: Vec<FloorRow> = BUILT_IN_ROWS
        .iter()
        .map(|(glob, min_lane, flags, owner_first)| FloorRow {
            glob: glob.to_string(),
            tokens: glob_tokens(glob),
            min_lane: min_lane.to_string(),
            flags: flags.iter().map(|f| f.to_string()).collect(),
            owner_first: *owner_first,
            source: BUILT_IN_SOURCE,
        })
        .collect();
    match bstate::read_config_raw(root).get("rigor_floor") {
        None | Some(Value::Null) => {}
        Some(Value::Array(items)) => {
            for item in items {
                match repo_row(item) {
                    Some(row) => rows.push(row),
                    None => eprintln!(
                        "WARNING: .bee/config.json rigor_floor row {item} is malformed and was skipped. A row is {{\"glob\": \"<path glob>\", \"min_lane\": \"docs|tiny|small|spike|standard|high-risk\", \"flags\": [<route flags>], \"owner_first\": true|false}}."
                    ),
                }
            }
        }
        Some(other) => eprintln!(
            "WARNING: .bee/config.json rigor_floor must be an array of rows; {other} was skipped."
        ),
    }
    rows
}

pub(crate) fn rigor_floor_hits(root: &Path, files: &[String]) -> Vec<FloorHit> {
    let rows = floor_rows(root);
    let mut hits = Vec::new();
    for file in files {
        let path = normalize_cell_path(file);
        let chars: Vec<char> = path.chars().collect();
        for row in rows.iter().filter(|r| glob_match(&r.tokens, &chars)) {
            hits.push(FloorHit {
                path: path.clone(),
                glob: row.glob.clone(),
                source: row.source,
                min_lane: row.min_lane.clone(),
                flags: row.flags.clone(),
                owner_first: row.owner_first,
            });
        }
    }
    hits
}

pub(crate) fn owner_first_hit(root: &Path, files: &[String]) -> Option<FloorHit> {
    rigor_floor_hits(root, files).into_iter().find(|h| h.owner_first)
}

fn route_with_lane(route: Option<&Value>) -> Option<Map<String, Value>> {
    let route = route?.as_object()?;
    route.get("lane").and_then(Value::as_str)?;
    Some(route.clone())
}

fn feature_route_record(root: &Path, feature: &str) -> Option<Map<String, Value>> {
    let lane = read_lane_display(root, feature).ok().flatten();
    if let Some(route) = route_with_lane(lane.as_ref().and_then(|l| l.get("route"))) {
        return Some(route);
    }
    let ReadJson::Parsed(Value::Object(state)) = read_json(&bstate::state_path(root)) else { return None };
    let route = route_with_lane(state.get("route"))?;
    route_belongs_to_feature(&route, &Value::String(feature.to_string())).then_some(route)
}

fn execution_gate_auto(root: &Path, feature: &str) -> bool {
    let Ok(workflows) = list_workflows(root) else { return false };
    let Some(gate) = find_live_workflow(&workflows, feature)
        .and_then(|wf| wf.get("gates"))
        .and_then(|g| g.get("execution"))
    else {
        return false;
    };
    let approved = gate.get("approved") == Some(&Value::Bool(true))
        || gate.get("state").and_then(Value::as_str) == Some("approved");
    approved && gate.get("actor").and_then(Value::as_str) == Some("auto")
}

fn string_list(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => Vec::new(),
    }
}

fn highest_lane<'a>(lanes: impl Iterator<Item = &'a str>) -> &'a str {
    lanes.max_by_key(|l| rigor_lane_rank(l).unwrap_or(0)).unwrap_or("docs")
}

fn union_flags<'a>(base: &[String], hits: impl Iterator<Item = &'a FloorHit>) -> Vec<String> {
    let mut out = base.to_vec();
    for hit in hits {
        for flag in &hit.flags {
            if !out.contains(flag) {
                out.push(flag.clone());
            }
        }
    }
    out
}

fn below_floor_refusal(
    verb: &str,
    id: &str,
    feature: &str,
    hits: &[FloorHit],
    route: &Map<String, Value>,
) -> Option<String> {
    let route_lane = route.get("lane").and_then(Value::as_str).unwrap_or("");
    let route_rank = rigor_lane_rank(route_lane).unwrap_or(0);
    let route_flags = string_list(route.get("flags"));
    let short = hits.iter().find(|h| {
        rigor_lane_rank(&h.min_lane).unwrap_or(0) > route_rank
            || h.flags.iter().any(|f| !route_flags.contains(f))
    })?;
    let same_path: Vec<&FloorHit> = hits.iter().filter(|h| h.path == short.path).collect();
    let floor_lane = highest_lane(same_path.iter().map(|h| h.min_lane.as_str()));
    let floor_flags = union_flags(&[], same_path.iter().copied());
    let owner_note = if same_path.iter().any(|h| h.owner_first) {
        " This path is owner-first: the merged shape+execution gate for this feature also needs a human approval, never --actor auto."
    } else {
        ""
    };
    let need_lane = highest_lane(hits.iter().map(|h| h.min_lane.as_str()).chain([route_lane]));
    let need_flags = union_flags(&route_flags, hits.iter());
    let class = route.get("class").and_then(Value::as_str).unwrap_or("feature");
    let files = match route.get("product_files") {
        Some(Value::Number(n)) => n.as_f64().map(jsjson::js_f64_to_string).unwrap_or_else(|| n.to_string()),
        _ => "<n>".to_string(),
    };
    let flags_arg = if need_flags.is_empty() { String::new() } else { format!(" --flags {}", need_flags.join(",")) };
    Some(format!(
        "{verb}: RIGOR_FLOOR — cell \"{id}\" touches \"{}\", which matches the rigor-floor row \"{}\" (source: {}). Its floor is lane \"{floor_lane}\" with flags [{}], but feature \"{feature}\" is routed lane \"{route_lane}\" with flags [{}]. A floor can be raised, never lowered.{owner_note} FIX: raise the route — `bee route --set --class {class} --lane {need_lane}{flags_arg} --files {files}` — then retry; never drop the file to pass. The write is refused; nothing was written.",
        short.path,
        short.glob,
        short.source,
        floor_flags.join(", "),
        route_flags.join(", "),
    ))
}

pub(crate) fn rigor_floor_refusal(root: &Path, cell: &Map<String, Value>, verb: &str) -> Option<String> {
    let feature = cell.get("feature").and_then(Value::as_str).map(js_trim).filter(|f| !f.is_empty())?;
    let files = string_list(cell.get("files"));
    let hits = rigor_floor_hits(root, &files);
    if hits.is_empty() {
        return None;
    }
    let id = cell.get("id").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or("(unknown id)");
    if let Some(route) = feature_route_record(root, feature) {
        if let Some(refusal) = below_floor_refusal(verb, id, feature, &hits, &route) {
            return Some(refusal);
        }
    }
    let owner = hits.iter().find(|h| h.owner_first)?;
    if !execution_gate_auto(root, feature) {
        return None;
    }
    Some(format!(
        "{verb}: GATE_OWNER_FIRST — cell \"{id}\" touches \"{}\", which matches the owner-first rigor-floor row \"{}\" (source: {}), but feature \"{feature}\"'s execution gate was approved by --actor auto. An owner-first cell needs a human-approved gate at every gate_bypass level. FIX: stop; do not retry; do not pass --actor user. Run `bee state waiting-on set --kind gate --subject \"{feature} owner-first gate\"`, show the gate to the human, and end the turn. The write is refused; nothing was written.",
        owner.path, owner.glob, owner.source
    ))
}
