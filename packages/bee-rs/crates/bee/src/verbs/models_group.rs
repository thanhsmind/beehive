// bee models — the role table read out loud (models-show-verb D1).
//
// Verbs served natively (exact argv shapes only — see the probe):
//   models show [--runtime claude|codex|opencode] [--json]
//
// WHY IT EXISTS. An agent that has to pick a role for a dispatch had exactly
// one way to learn what the roles MEAN: open `.bee/config.json` and parse
// `models.<runtime>` by hand. D1 makes that a verb — "lấy thông tin models từ
// config nên là 1 verb trong bee để nhận trọn bộ không cần phải viết code
// đọc" — so the table has one reader and the descriptions have one home.
//
// WHY IT READS RAW. `normalize_models` exists to feed RESOLUTION, and it
// deliberately drops `description`: a normalized slot is what the dispatcher
// acts on, and a sentence written for a human would be dead weight (and a
// silent behaviour surface) down there. That strip is exactly what makes it
// the wrong source for this verb — the whole point here is the sentence. So
// this module reads `read_config_raw(root)["models"]` and carries every slot
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
use crate::verbs::drivers::{default_models, RUNTIMES};
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
const TEACH: &str = "models — the role table bee dispatches from. A role's `description` is written in \
.bee/config.json under models.<runtime>.<role>, and this verb is how it is read; \
never parse that file by hand.";

/// One role's row: the slot exactly as the config wrote it, plus where it came
/// from and the description lifted out for readers who only want the sentence.
fn row(role: &str, slot: &Value, source: &str) -> Value {
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

/// The whole result, built from a raw `models` value. `runtime` filters to one
/// table; `None` is every runtime the config or the defaults know.
pub(crate) fn build_table(raw_models: Option<&Value>, runtime: Option<&str>) -> Result<Value, String> {
    let raw = raw_models.and_then(Value::as_object);
    let mut names = known_runtimes(raw);
    if let Some(want) = runtime {
        if !names.iter().any(|n| n == want) {
            return Err(format!(
                "bee models show: --runtime {want:?} is not a runtime this repo knows. Legal: {}.",
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
                rows.push(row(role, slot, SOURCE_CONFIGURED));
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
                rows.push(row(&role, &slot, SOURCE_DEFAULT));
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
pub(crate) fn models_table(root: &Path, runtime: Option<&str>) -> Result<Value, String> {
    let config = read_config_raw(root);
    build_table(config.get("models"), runtime)
}

/// A slot on one line. The description is printed as prose beside the row, so
/// it is dropped from the JSON echo here rather than shown twice — every OTHER
/// key of the slot is rendered exactly as stored.
fn slot_display(slot: &Value) -> String {
    match slot {
        Value::Null => "unset".to_string(),
        Value::Object(o) => {
            let rest: Map<String, Value> =
                o.iter().filter(|(k, _)| *k != "description").map(|(k, v)| (k.clone(), v.clone())).collect();
            if rest.is_empty() {
                "(description only)".to_string()
            } else {
                crate::jsjson::stringify(&Value::Object(rest))
            }
        }
        other => crate::jsjson::stringify(other),
    }
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
        let width = roles
            .iter()
            .filter_map(|r| r.get("role").and_then(Value::as_str))
            .map(str::len)
            .max()
            .unwrap_or(0);
        for role_row in roles {
            let role = role_row.get("role").and_then(Value::as_str).unwrap_or("?");
            let source = role_row.get("source").and_then(Value::as_str).unwrap_or("?");
            let slot = role_row.get("slot").unwrap_or(&Value::Null);
            let mut line =
                format!("  {role:<width$}  [{source}]  {}", slot_display(slot), width = width);
            if let Some(description) = role_row.get("description").and_then(Value::as_str) {
                line.push_str(&format!(" — {description}"));
            }
            lines.push(line);
        }
    }
    lines.join("\n")
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "models" {
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

    let ctx = match g_prelude("models show", json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };

    match models_table(&ctx.root, runtime.as_deref()) {
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
        assert_eq!(names, vec!["claude", "codex", "opencode", "acme"]);
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
            r#"{"models":{"claude":{"code":{"model":"opus","description":"from disk"}}}}"#,
        )
        .unwrap();
        let result = models_table(root, None).unwrap();
        assert_eq!(role(&result, "claude", "code")["description"], json!("from disk"));
        assert_eq!(
            role(&result, "claude", "code")["slot"],
            json!({"model": "opus", "description": "from disk"})
        );
    }

    /// Read-only means read-only: a `models show` must leave the store exactly
    /// as it found it.
    #[test]
    fn reading_the_table_writes_nothing() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir_all(root.join(".bee")).unwrap();
        let config = root.join(".bee").join("config.json");
        std::fs::write(&config, r#"{"models":{"claude":{"code":"opus"}}}"#).unwrap();
        let before = std::fs::read_dir(root.join(".bee")).unwrap().count();
        let raw_before = std::fs::read_to_string(&config).unwrap();
        models_table(root, None).unwrap();
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
        assert!(text.contains("models.<runtime>.<role>"), "the teaching line is missing: {text}");
        assert!(text.contains("code"), "{text}");
        assert!(text.contains("[configured]"), "{text}");
        assert!(text.contains("write the cell's code"), "{text}");
        assert!(text.contains(r#"{"model":"opus"}"#), "{text}");
        assert!(
            !text.contains("\"description\":\"write the cell's code\""),
            "the description is printed twice: {text}"
        );
        assert!(text.contains("[default]"), "the built-ins are missing from the text: {text}");
    }

    #[test]
    fn an_unset_built_in_reads_as_unset_rather_than_null() {
        let text = render(&build_table(None, Some("codex")).unwrap());
        assert!(text.contains("unset"), "{text}");
        assert!(!text.contains("null"), "{text}");
    }
}
