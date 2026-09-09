// bee team — the role table read out loud (team-config-rename D1).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   team show [--runtime claude|codex|opencode] [--json]
//
// WHY IT EXISTS. An agent that has to pick a role for a dispatch had exactly
// one way to learn what the roles MEAN: open `.bee/config.json` and parse
// `team.<runtime>` by hand. D1 makes that a verb — "lấy thông tin models từ
// config nên là 1 verb trong bee để nhận trọn bộ không cần phải viết code
// đọc" — so the table has one reader and the descriptions have one home.
//
// WHY IT READS RAW. `normalize_models` exists to feed RESOLUTION, and it
// deliberately drops `description`: a normalized slot is what the dispatcher
// acts on, and a sentence written for a human would be dead weight (and a
// silent behaviour surface) down there. That strip is exactly what makes it
// the wrong source for this verb — the whole point here is the sentence. So
// this module reads `read_config_raw(root)["team"]` and carries every slot
// through VERBATIM: string slots, `{kind:"cli"}` slots, `{kind:"herding"}`
// slots, junk slots normalize would have dropped, all of it. A slot shape
// this file understands is a slot shape it renders more prettily, never a
// slot shape it filters.
//
// WHAT `source` MEANS. `configured` — the runtime's own table in the config
// names this role. `default` — it does not, and bee ships a built-in for it
// (`drivers::default_models`), which is what a dispatch would fall back to.
// Built-ins are added only for the runtimes bee actually ships them for
// (`drivers::RUNTIMES`); a runtime key the operator invented gets its own
// configured rows and nothing invented on top.
//
// READ-ONLY, and nothing in the resolution path is touched (D5): no write, no
// lock, no normalization, no dispatch code reached.

use crate::state::read_config_raw;
use crate::verbs::drivers::{
    declared_model_for, default_models, normalize_models, resolve_role, Resolved, RUNTIMES,
};
use crate::verbs::knowledge::{g_prelude, pre_json_scan, GPre};
use crate::verbs::reservations::{js_trim, keys_known, parse_flags, FlagV};
use serde_json::{Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

pub(crate) const SOURCE_CONFIGURED: &str = "configured";
pub(crate) const SOURCE_DEFAULT: &str = "default";

/// The teaching line the text rendering opens with. An agent that ran this
/// verb once should never go back to parsing the config by hand.
const TEACH: &str = "team — who does which job. Pick team members by the description column; the transport is configuration. A role's `description` is written in \
.bee/config.json under team.<runtime>.<role>, and this verb is how it is read; \
never parse that file by hand.";

/// Resolve a slot's model and transport for display and --json.
fn resolve_slot_display(
    cfg: &Value,
    role: &str,
    slot: &Value,
    rt: &str,
    normalized: &Map<String, Value>,
) -> (Option<String>, Option<String>) {
    if slot.is_null() {
        return (None, None);
    }
    let resolved = resolve_role(normalized, &[role], rt, "gather");
    match &resolved {
        Resolved::Model { model, .. } | Resolved::Native { model, .. } => {
            (Some(model.clone()), Some("native".to_string()))
        }
        Resolved::Herding { agent, .. } => {
            let dm = declared_model_for(cfg, &resolved, rt);
            let tr = match agent {
                Some(a) => format!("herding: {a}"),
                None => "herding".to_string(),
            };
            (dm, Some(tr))
        }
        Resolved::Cli { .. } | Resolved::Refused { .. } => {
            let dm = declared_model_for(cfg, &resolved, rt);
            (dm, Some("cli".to_string()))
        }
        Resolved::Inherit => {
            (Some("session default".to_string()), Some("native".to_string()))
        }
        Resolved::Budget => (None, None),
    }
}

/// One role's row: the slot exactly as the config wrote it, plus where it came
/// from, the description, and the resolved model and transport fields.
fn row(
    role: &str,
    slot: &Value,
    source: &str,
    model: Option<String>,
    transport: Option<String>,
) -> Value {
    let description = slot
        .as_object()
        .and_then(|o| o.get("description"))
        .and_then(Value::as_str)
        .map(|d| Value::String(d.to_string()))
        .unwrap_or(Value::Null);
    let mut m = Map::new();
    m.insert("role".into(), Value::String(role.to_string()));
    m.insert("source".into(), Value::String(source.to_string()));
    m.insert("description".into(), description);
    m.insert(
        "model".into(),
        model.map(Value::String).unwrap_or(Value::Null),
    );
    m.insert(
        "transport".into(),
        transport.map(Value::String).unwrap_or(Value::Null),
    );
    // VERBATIM: no normalize, no trim, no key filtering.
    m.insert("slot".into(), slot.clone());
    Value::Object(m)
}

/// Every runtime name this repo knows: the ones bee ships defaults for, then
/// any extra table the config names (an operator's own runtime key is still
/// their table, and hiding it would send them back to the config file).
fn known_runtimes(raw_models: Option<&Map<String, Value>>) -> Vec<String> {
    let mut names: Vec<String> = RUNTIMES.iter().map(|r| r.to_string()).collect();
    if let Some(raw) = raw_models {
        for key in raw.keys() {
            if !names.iter().any(|n| n == key) {
                names.push(key.clone());
            }
        }
    }
    names
}

/// The whole result, built from a raw `models` or `team` or root config value.
/// `runtime` filters to one table; `None` is every runtime the config or the defaults know.
pub(crate) fn build_table(raw_models: Option<&Value>, runtime: Option<&str>) -> Result<Value, String> {
    let (cfg_val, team_val) = match raw_models {
        Some(v) if v.is_object() => {
            let obj = v.as_object().unwrap();
            if obj.contains_key("team") || obj.contains_key("models") {
                let team = obj.get("team").or_else(|| obj.get("models"));
                (v.clone(), team.cloned().unwrap_or_else(|| Value::Object(Map::new())))
            } else {
                let mut c = Map::new();
                c.insert("team".into(), v.clone());
                (Value::Object(c), v.clone())
            }
        }
        _ => (Value::Object(Map::new()), Value::Object(Map::new())),
    };
    let normalized = normalize_models(Some(&team_val));
    let raw = team_val.as_object();
    let mut names = known_runtimes(raw);
    if let Some(want) = runtime {
        if !names.iter().any(|n| n == want) {
            return Err(format!(
                "bee team show: --runtime {want:?} is not a runtime this repo knows. Legal: {}.",
                names.join(", ")
            ));
        }
        names.retain(|n| n == want);
    }

    let mut runtimes = Vec::new();
    let mut roles_total = 0usize;
    let mut configured_total = 0usize;
    let mut default_total = 0usize;
    for rt in &names {
        let table = raw.and_then(|r| r.get(rt)).and_then(Value::as_object);
        let mut rows = Vec::new();
        if let Some(table) = table {
            for (role, slot) in table {
                configured_total += 1;
                let (model, transport) =
                    resolve_slot_display(&cfg_val, role, slot, rt, &normalized);
                rows.push(row(role, slot, SOURCE_CONFIGURED, model, transport));
            }
        }
        // Built-ins only where bee actually ships them, and only for a role
        // the config left unnamed — a configured slot is never shadowed.
        if RUNTIMES.contains(&rt.as_str()) {
            for (role, slot) in default_models(rt) {
                if table.is_some_and(|t| t.contains_key(&role)) {
                    continue;
                }
                default_total += 1;
                let (model, transport) =
                    resolve_slot_display(&cfg_val, &role, &slot, rt, &normalized);
                rows.push(row(&role, &slot, SOURCE_DEFAULT, model, transport));
            }
        }
        roles_total += rows.len();
        let mut m = Map::new();
        m.insert("runtime".into(), Value::String(rt.clone()));
        m.insert("roles".into(), Value::Array(rows));
        runtimes.push(Value::Object(m));
    }

    let mut counts = Map::new();
    counts.insert("runtimes".into(), Value::from(runtimes.len()));
    counts.insert("roles".into(), Value::from(roles_total));
    counts.insert(SOURCE_CONFIGURED.into(), Value::from(configured_total));
    counts.insert(SOURCE_DEFAULT.into(), Value::from(default_total));

    let mut result = Map::new();
    result.insert("runtimes".into(), Value::Array(runtimes));
    result.insert("counts".into(), Value::Object(counts));
    result.insert("note".into(), Value::String(TEACH.to_string()));
    Ok(Value::Object(result))
}

/// The raw table for a repo root. `read_config_raw` is the config layer's own
/// reader (tracked config + the local overlay, corrupt-tolerant); nothing is
/// normalized on the way out.
pub(crate) fn team_table(root: &Path, runtime: Option<&str>) -> Result<Value, String> {
    let config = read_config_raw(root);
    build_table(Some(&Value::Object(config)), runtime)
}

/// Legacy alias for `team_table`.
#[allow(dead_code)]
pub(crate) fn models_table(root: &Path, runtime: Option<&str>) -> Result<Value, String> {
    team_table(root, runtime)
}

fn render(result: &Value) -> String {
    let mut lines = vec![TEACH.to_string()];
    let empty = Vec::new();
    let runtimes = result.get("runtimes").and_then(Value::as_array).unwrap_or(&empty);
    for entry in runtimes {
        let rt = entry.get("runtime").and_then(Value::as_str).unwrap_or("?");
        let roles = entry.get("roles").and_then(Value::as_array).unwrap_or(&empty);
        lines.push(String::new());
        lines.push(format!("{rt} — {} role(s)", roles.len()));
        if roles.is_empty() {
            lines.push("  (no roles — this runtime has no table and no built-in defaults)".to_string());
            continue;
        }
        let width_role = roles
            .iter()
            .filter_map(|r| r.get("role").and_then(Value::as_str))
            .map(str::len)
            .max()
            .unwrap_or(0);
        let width_source = roles
            .iter()
            .filter_map(|r| r.get("source").and_then(Value::as_str))
            .map(|s| s.len() + 2)
            .max()
            .unwrap_or(0);
        let width_desc = roles
            .iter()
            .map(|r| r.get("description").and_then(Value::as_str).unwrap_or("").len())
            .max()
            .unwrap_or(0);
        let width_model = roles
            .iter()
            .map(|r| match r.get("model") {
                Some(Value::String(s)) => s.len(),
                _ => {
                    let tr = r.get("transport").and_then(Value::as_str).unwrap_or("");
                    if tr.starts_with("herding") {
                        "model chosen by the agent".len()
                    } else if r.get("slot").is_some_and(Value::is_null) {
                        "unset".len()
                    } else {
                        1
                    }
                }
            })
            .max()
            .unwrap_or(0);

        for role_row in roles {
            let role = role_row.get("role").and_then(Value::as_str).unwrap_or("?");
            let source = role_row.get("source").and_then(Value::as_str).unwrap_or("?");
            let source_col = format!("[{source}]");
            let description = role_row.get("description").and_then(Value::as_str).unwrap_or("");
            let model_str = match role_row.get("model") {
                Some(Value::String(s)) => s.as_str(),
                _ => {
                    let tr = role_row.get("transport").and_then(Value::as_str).unwrap_or("");
                    if tr.starts_with("herding") {
                        "model chosen by the agent"
                    } else if role_row.get("slot").is_some_and(Value::is_null) {
                        "unset"
                    } else {
                        "-"
                    }
                }
            };
            let transport_str = role_row.get("transport").and_then(Value::as_str).unwrap_or("-");

            let line = format!(
                "  {role:<width_role$}  {source_col:<width_source$}  {description:<width_desc$}  {model_str:<width_model$}  {transport_str}"
            );
            lines.push(line);
        }
    }
    lines.join("\n")
}


pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "team" {
        return None;
    }
    if args.get(1)?.to_str()? != "show" {
        return None;
    }
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    if !keys_known(&flags, &["runtime"]) {
        return None;
    }
    let runtime = match flags.get("runtime") {
        Some(FlagV::S(s)) => Some(js_trim(s).to_string()),
        _ => None,
    };

    let ctx = match g_prelude("team show", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };

    match team_table(&ctx.root, runtime.as_deref()) {
        Err(message) => Some(ctx.fail(&message)),
        Ok(result) => {
            let text = render(&result);
            Some(ctx.emit(&result, &text, 0))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn table(raw: &Value, runtime: Option<&str>) -> Value {
        build_table(Some(raw), runtime).expect("the fixture names a legal runtime")
    }

    fn roles_of<'a>(result: &'a Value, runtime: &str) -> &'a Vec<Value> {
        result
            .get("runtimes")
            .and_then(Value::as_array)
            .unwrap()
            .iter()
            .find(|r| r.get("runtime").and_then(Value::as_str) == Some(runtime))
            .unwrap_or_else(|| panic!("no {runtime} table in {result}"))
            .get("roles")
            .and_then(Value::as_array)
            .unwrap()
    }

    fn role<'a>(result: &'a Value, runtime: &str, name: &str) -> &'a Value {
        roles_of(result, runtime)
            .iter()
            .find(|r| r.get("role").and_then(Value::as_str) == Some(name))
            .unwrap_or_else(|| panic!("no {runtime}.{name} row in {result}"))
    }

    /// D1's whole point: the slot arrives byte-for-byte, description included.
    /// `normalize_models` would have dropped the sentence here.
    #[test]
    fn a_described_role_is_carried_through_verbatim_with_its_description() {
        let raw = json!({
            "claude": {
                "code": {"model": "opus", "description": "write the cell's Rust code and its tests"}
            }
        });
        let result = table(&raw, None);
        let code = role(&result, "claude", "code");
        assert_eq!(code["source"], json!(SOURCE_CONFIGURED));
        assert_eq!(
            code["slot"],
            json!({"model": "opus", "description": "write the cell's Rust code and its tests"}),
            "the slot lost a key on the way out — this verb reads RAW"
        );
        assert_eq!(code["description"], json!("write the cell's Rust code and its tests"));
    }

    /// Every documented slot shape, plus one normalize would have thrown away.
    #[test]
    fn every_slot_shape_survives_including_ones_normalize_drops() {
        let raw = json!({
            "claude": {
                "read": {"kind": "herding", "agent": "agy-flash", "description": "read-only scans"},
                "review": {"kind": "cli", "command": "codex exec -", "promptVia": "stdin"},
                "junk": 7
            },
            "codex": {"generation": "gpt-5.5"}
        });
        let result = table(&raw, None);
        assert_eq!(
            role(&result, "claude", "read")["slot"],
            json!({"kind": "herding", "agent": "agy-flash", "description": "read-only scans"})
        );
        assert_eq!(
            role(&result, "claude", "review")["slot"],
            json!({"kind": "cli", "command": "codex exec -", "promptVia": "stdin"}),
            "promptVia is not a normalized key — a raw read keeps it anyway"
        );
        assert_eq!(
            role(&result, "claude", "junk")["slot"],
            json!(7),
            "a slot normalize_models drops is still what the config says"
        );
        assert_eq!(role(&result, "codex", "generation")["slot"], json!("gpt-5.5"));
    }

    /// A role only the built-ins know is shown, and marked as such.
    #[test]
    fn a_role_only_the_built_in_defaults_know_is_marked_default() {
        let raw = json!({"claude": {"code": {"model": "opus"}}});
        let result = table(&raw, None);
        let review = role(&result, "claude", "review");
        assert_eq!(review["source"], json!(SOURCE_DEFAULT));
        assert_eq!(review["slot"], json!("opus"), "claude's built-in review model");
        assert_eq!(review["description"], json!(null), "a built-in ships no sentence");
        // The configured row is never shadowed by a built-in of the same name.
        assert_eq!(role(&result, "claude", "code")["source"], json!(SOURCE_CONFIGURED));
        assert_eq!(
            roles_of(&result, "claude")
                .iter()
                .filter(|r| r["role"] == json!("code"))
                .count(),
            1,
            "a configured role must appear exactly once"
        );
    }

    /// A configured slot wins over the built-in for the same role.
    #[test]
    fn a_configured_role_hides_the_built_in_of_the_same_name() {
        let raw = json!({"claude": {"review": {"model": "fable", "description": "house reviewer"}}});
        let result = table(&raw, None);
        let review = role(&result, "claude", "review");
        assert_eq!(review["source"], json!(SOURCE_CONFIGURED));
        assert_eq!(review["slot"], json!({"model": "fable", "description": "house reviewer"}));
    }

    #[test]
    fn no_runtime_flag_shows_every_runtime_the_config_or_defaults_know() {
        let raw = json!({"claude": {}, "acme": {"code": "acme/big"}});
        let result = table(&raw, None);
        let names: Vec<&str> = result["runtimes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["runtime"].as_str().unwrap())
            .collect();
        // The four names bee itself knows (drivers::RUNTIMES — `pi` joined
        // them with pi-support D5), in that order, then the operator's own.
        assert_eq!(names, vec!["claude", "codex", "opencode", "pi", "acme"]);
        // An operator's own runtime key gets its table and no invented rows.
        assert_eq!(roles_of(&result, "acme").len(), 1);
        assert_eq!(role(&result, "acme", "code")["source"], json!(SOURCE_CONFIGURED));
    }

    #[test]
    fn the_runtime_flag_filters_to_one_table() {
        let raw = json!({"claude": {"code": {"model": "opus"}}, "codex": {"generation": "gpt-5.5"}});
        let result = table(&raw, Some("codex"));
        assert_eq!(result["runtimes"].as_array().unwrap().len(), 1);
        assert_eq!(result["runtimes"][0]["runtime"], json!("codex"));
        assert_eq!(result["counts"]["runtimes"], json!(1));
    }

    #[test]
    fn an_unknown_runtime_is_refused_with_the_legal_names() {
        let raw = json!({"claude": {}});
        let err = build_table(Some(&raw), Some("gemini")).unwrap_err();
        assert!(err.contains("gemini"), "{err}");
        assert!(err.contains("claude, codex, opencode"), "{err}");
    }

    /// No config at all is still an answer: the built-ins are the table.
    #[test]
    fn a_repo_with_no_models_config_shows_the_built_ins() {
        let result = build_table(None, None).unwrap();
        assert_eq!(result["counts"][SOURCE_CONFIGURED], json!(0));
        assert_eq!(role(&result, "claude", "generation")["slot"], json!("sonnet"));
        assert_eq!(role(&result, "codex", "generation")["slot"], json!(null));
        assert_eq!(role(&result, "opencode", "review")["source"], json!(SOURCE_DEFAULT));
    }

    #[test]
    fn counts_add_up_to_the_rows_actually_rendered() {
        let raw = json!({"claude": {"code": {"model": "opus"}}});
        let result = table(&raw, None);
        let rendered: usize = result["runtimes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["roles"].as_array().unwrap().len())
            .sum();
        assert_eq!(result["counts"]["roles"], json!(rendered));
        assert_eq!(
            result["counts"][SOURCE_CONFIGURED].as_u64().unwrap()
                + result["counts"][SOURCE_DEFAULT].as_u64().unwrap(),
            rendered as u64
        );
    }

    /// The seam this verb exists for: the file on disk, read raw. A test that
    /// only ever fed `build_table` a literal would pass with the config read
    /// wired to nothing at all.
    #[test]
    fn the_table_is_read_off_the_config_file_on_disk() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        std::fs::write(
            root.join(".bee").join("config.json"),
            r#"{"team":{"claude":{"code":{"model":"opus","description":"from disk"}}}}"#,
        )
        .unwrap();
        let result = team_table(root, None).unwrap();
        assert_eq!(role(&result, "claude", "code")["description"], json!("from disk"));
        assert_eq!(
            role(&result, "claude", "code")["slot"],
            json!({"model": "opus", "description": "from disk"})
        );

        // Alias config still works via fold at load
        let tmp2 = tempfile::tempdir().unwrap();
        let root2 = tmp2.path();
        std::fs::create_dir_all(root2.join(".bee")).unwrap();
        std::fs::write(
            root2.join(".bee").join("config.json"),
            r#"{"models":{"claude":{"code":{"model":"opus","description":"from legacy models"}}}}"#,
        )
        .unwrap();
        let result2 = team_table(root2, None).unwrap();
        assert_eq!(role(&result2, "claude", "code")["description"], json!("from legacy models"));
    }

    /// Read-only means read-only: a `team show` must leave the store exactly
    /// as it found it.
    #[test]
    fn reading_the_table_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let config = root.join(".bee").join("config.json");
        std::fs::write(&config, r#"{"team":{"claude":{"code":"opus"}}}"#).unwrap();
        let before = std::fs::read_dir(root.join(".bee")).unwrap().count();
        let raw_before = std::fs::read_to_string(&config).unwrap();
        team_table(root, None).unwrap();
        let after = std::fs::read_dir(root.join(".bee")).unwrap().count();
        assert_eq!(before, after, "reading the role table created or removed a store file");
        assert_eq!(raw_before, std::fs::read_to_string(&config).unwrap());
    }

    #[test]
    fn the_text_rendering_names_the_source_and_keeps_the_description() {
        let raw = json!({
            "claude": {"code": {"model": "opus", "description": "write the cell's code"}}
        });
        let text = render(&table(&raw, Some("claude")));
        assert!(text.contains("team.<runtime>.<role>"), "the teaching line is missing: {text}");
        assert!(text.contains("code"), "{text}");
        assert!(text.contains("[configured]"), "{text}");
        assert!(text.contains("write the cell's code"), "{text}");
        assert!(text.contains("opus"), "{text}");
        assert!(text.contains("native"), "{text}");
        assert!(text.contains("[default]"), "the built-ins are missing from the text: {text}");
    }

    #[test]
    fn team_show_columns_order_on_claude_and_pi() {
        let raw = json!({
            "claude": {
                "code": {"model": "opus", "description": "write the code"}
            },
            "pi": {
                "plan": {"model": "gpt-5", "description": "shape the plan"}
            }
        });
        for rt in &["claude", "pi"] {
            let text = render(&table(&raw, Some(rt)));
            let role_name = if *rt == "claude" { "code" } else { "plan" };
            let line = text.lines().find(|l| l.contains(role_name)).expect("found role line");
            let pos_role = line.find(role_name).unwrap();
            let pos_source = line.find("[configured]").unwrap();
            let desc_str = if *rt == "claude" { "write the code" } else { "shape the plan" };
            let pos_desc = line.find(desc_str).unwrap();
            let model_str = if *rt == "claude" { "opus" } else { "gpt-5" };
            let pos_model = line.find(model_str).unwrap();
            let pos_transport = line.find("native").unwrap();

            assert!(pos_role < pos_source, "role before source in {line}");
            assert!(pos_source < pos_desc, "source before desc in {line}");
            assert!(pos_desc < pos_model, "desc before model in {line}");
            assert!(pos_model < pos_transport, "model before transport in {line}");
        }
    }

    #[test]
    fn team_show_json_preserves_slot_and_adds_model_and_transport() {
        let raw = json!({
            "claude": {
                "code": {"model": "opus", "description": "write code"}
            }
        });
        let result = table(&raw, Some("claude"));
        let code = role(&result, "claude", "code");
        assert_eq!(code["slot"], json!({"model": "opus", "description": "write code"}));
        assert_eq!(code["model"], json!("opus"));
        assert_eq!(code["transport"], json!("native"));
    }

    #[test]
    fn an_unset_built_in_reads_as_unset_rather_than_null() {
        let text = render(&build_table(None, Some("codex")).unwrap());
        assert!(text.contains("unset"), "{text}");
        assert!(!text.contains("null"), "{text}");
    }
}
