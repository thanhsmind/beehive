use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::write_guard::config_governed_change_deny;
use crate::jsjson;
use crate::state::merge_config_overlay;
use crate::verbs::knowledge::{g_prelude, js_bool_flag, pre_json_scan, GCtx, GPre};
use crate::verbs::reservations::{keys_known, parse_flags, Flags, FlagV};
use crate::verbs::status_full::validate_models_config;
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const TRACKED: &str = ".bee/config.json";
const LOCAL: &str = ".bee/config.local.json";
const DEFAULTS_SECTION: &str = "docs/config-reference.md \"Other keys\"";

type Layer = Option<Map<String, Value>>;

fn rel(local: bool) -> &'static str {
    if local {
        LOCAL
    } else {
        TRACKED
    }
}

fn read_layer(root: &Path, rel: &str) -> Result<Layer, String> {
    match read_json(&root.join(rel)) {
        ReadJson::Missing => Ok(None),
        ReadJson::Parsed(Value::Object(m)) => Ok(Some(m)),
        ReadJson::Corrupt => Err(format!(
            "{rel} exists but is not valid JSON. Nothing changed. FIX: repair the file by hand, then run the command again."
        )),
        ReadJson::Parsed(_) => Err(format!(
            "{rel} is not a JSON object. Nothing changed. FIX: repair the file by hand, then run the command again."
        )),
    }
}

fn split_key(key: &str) -> Result<Vec<&str>, String> {
    let parts: Vec<&str> = key.split('.').collect();
    if parts.iter().any(|p| p.is_empty()) {
        return Err(format!("--key \"{key}\" has an empty segment. FIX: name the key with dot-notation, e.g. herding.enabled."));
    }
    Ok(parts)
}

fn get_at<'a>(map: Option<&'a Map<String, Value>>, parts: &[&str]) -> Option<&'a Value> {
    let (head, rest) = parts.split_first()?;
    let value = map?.get(*head)?;
    if rest.is_empty() {
        Some(value)
    } else {
        get_at(value.as_object(), rest)
    }
}

fn set_at(map: &mut Map<String, Value>, parts: &[&str], value: Value) {
    let (head, rest) = parts.split_first().expect("a split key has a segment");
    if rest.is_empty() {
        map.insert(head.to_string(), value);
        return;
    }
    let slot = map.entry(head.to_string()).or_insert_with(|| Value::Object(Map::new()));
    if !slot.is_object() {
        *slot = Value::Object(Map::new());
    }
    set_at(slot.as_object_mut().expect("made an object above"), rest, value);
}

fn unset_at(map: &mut Map<String, Value>, parts: &[&str]) -> bool {
    let (head, rest) = parts.split_first().expect("a split key has a segment");
    if rest.is_empty() {
        return map.shift_remove(*head).is_some();
    }
    let Some(Value::Object(child)) = map.get_mut(*head) else { return false };
    let removed = unset_at(child, rest);
    if removed && child.is_empty() {
        map.shift_remove(*head);
    }
    removed
}

fn merged(tracked: &Layer, local: &Layer) -> Value {
    let base = Value::Object(tracked.clone().unwrap_or_default());
    match local {
        Some(over) => merge_config_overlay(&base, &Value::Object(over.clone())),
        None => base,
    }
}

fn tracked_warning(tracked: &Layer, parts: &[&str], key: &str, local: bool) -> Value {
    if local && get_at(tracked.as_ref(), parts).is_some() {
        Value::String(format!(
            "warning: {TRACKED} also sets \"{key}\"; the {LOCAL} value wins at read time."
        ))
    } else {
        Value::Null
    }
}

fn governed_refusal(key: &str, target: &str, before: &Value, after: &Value) -> Option<String> {
    let before_text = before.to_string();
    let after_text = after.to_string();
    if let Some(reason) = config_governed_change_deny(target, Some(&before_text), &after_text) {
        return Some(format!("refused, nothing changed. {reason}"));
    }
    config_governed_change_deny(target, Some(&after_text), &before_text).map(|_| {
        format!(
            "refused, nothing changed. \"{key}\" adds a governed config key the config write guard protects. FIX: ask the user to edit this mapping directly in {target}; agents may not alter resolved role models or guard configuration."
        )
    })
}

fn problem_id(p: &crate::verbs::status_full::Problem) -> String {
    format!("{}|{}|{}", p.code, p.runtime.unwrap_or(""), p.slot.as_deref().unwrap_or(""))
}

fn model_refusal(before: &Value, after: &Value) -> Option<String> {
    let had: Vec<String> = validate_models_config(Some(before)).iter().map(problem_id).collect();
    let introduced: Vec<String> = validate_models_config(Some(after))
        .into_iter()
        .filter(|p| !had.contains(&problem_id(p)))
        .map(|p| format!("[{}] {}", p.code, p.message))
        .collect();
    if introduced.is_empty() {
        return None;
    }
    Some(format!(
        "refusing to write, nothing changed — this would make the config invalid:\n{}",
        introduced.join("\n")
    ))
}

fn write_edit(
    ctx: &GCtx,
    key: &str,
    to_local: bool,
    apply: impl FnOnce(&mut Map<String, Value>, &[&str]) -> bool,
) -> Result<(bool, Value), String> {
    let parts = split_key(key)?;
    let tracked = read_layer(&ctx.root, TRACKED)?;
    let local = read_layer(&ctx.root, LOCAL)?;
    let target = rel(to_local);
    let warning = tracked_warning(&tracked, &parts, key, to_local);
    let mut edited = if to_local { local.clone() } else { tracked.clone() }.unwrap_or_default();
    if !apply(&mut edited, &parts) {
        return Ok((false, warning));
    }
    let before = merged(&tracked, &local);
    let after = if to_local {
        merged(&tracked, &Some(edited.clone()))
    } else {
        merged(&Some(edited.clone()), &local)
    };
    if let Some(reason) = governed_refusal(key, target, &before, &after).or_else(|| model_refusal(&before, &after)) {
        return Err(reason);
    }
    write_json_atomic(&ctx.root.join(target), &Value::Object(edited))
        .map_err(|e| format!("could not write {target}: {e}"))?;
    Ok((true, warning))
}

fn with_warning(text: String, warning: &Value) -> String {
    match warning.as_str() {
        Some(w) => format!("{text}\n{w}"),
        None => text,
    }
}

fn run_get(ctx: &GCtx, key: &str, local: bool) -> Result<(Value, String), String> {
    let parts = split_key(key)?;
    let tracked = read_layer(&ctx.root, TRACKED)?;
    let layer = if local { read_layer(&ctx.root, LOCAL)? } else { tracked.clone() };
    let value = get_at(layer.as_ref(), &parts).cloned();
    let warning = tracked_warning(&tracked, &parts, key, local);
    let text = match &value {
        Some(v) => format!("{key} = {}", jsjson::stringify(v)),
        None => format!("config get: \"{key}\" is not set."),
    };
    let result = json!({
        "key": key,
        "present": value.is_some(),
        "value": value.unwrap_or(Value::Null),
        "local": local,
        "warning": warning,
    });
    let text = with_warning(text, &result["warning"]);
    Ok((result, text))
}

fn run_set(ctx: &GCtx, key: &str, raw: &str, as_string: bool, local: bool) -> Result<(Value, String), String> {
    let value = if as_string {
        Value::String(raw.to_string())
    } else {
        serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
    };
    let stored = value.clone();
    let (_, warning) = write_edit(ctx, key, local, |map, parts| {
        set_at(map, parts, stored);
        true
    })?;
    let flag = if local { " --local" } else { "" };
    let text = with_warning(format!("config set{flag}: {key} = {}", jsjson::stringify(&value)), &warning);
    Ok((json!({"key": key, "value": value, "local": local, "warning": warning}), text))
}

fn run_unset(ctx: &GCtx, key: &str, local: bool) -> Result<(Value, String), String> {
    let (removed, warning) = write_edit(ctx, key, local, unset_at)?;
    let flag = if local { " --local" } else { "" };
    let text = if removed {
        format!("config unset{flag}: removed {key}")
    } else {
        format!("config unset{flag}: \"{key}\" is not set in {}; nothing changed.", rel(local))
    };
    let text = with_warning(text, &warning);
    Ok((json!({"key": key, "removed": removed, "local": local, "warning": warning}), text))
}

fn run_explain(ctx: &GCtx, key: &str) -> Result<(Value, String), String> {
    let parts = split_key(key)?;
    let tracked = read_layer(&ctx.root, TRACKED)?;
    let local = read_layer(&ctx.root, LOCAL)?;
    let effective = merged(&tracked, &local);
    let effective = get_at(effective.as_object(), &parts).cloned();
    let tracked_value = get_at(tracked.as_ref(), &parts).cloned();
    let local_value = get_at(local.as_ref(), &parts).cloned();
    let source = if local_value.is_some() {
        "local"
    } else if tracked_value.is_some() {
        "tracked"
    } else {
        "unset"
    };
    let show = |v: &Option<Value>| v.as_ref().map(jsjson::stringify).unwrap_or_else(|| "(not set)".into());
    let text = match source {
        "unset" => format!(
            "{key} is not set in {TRACKED} or {LOCAL}; the built-in default applies — see {DEFAULTS_SECTION}."
        ),
        _ => format!(
            "{key} = {} (from {})\n  tracked {TRACKED}: {}\n  local   {LOCAL}: {}",
            show(&effective),
            rel(source == "local"),
            show(&tracked_value),
            show(&local_value)
        ),
    };
    let result = json!({
        "key": key,
        "effective": effective.unwrap_or(Value::Null),
        "source": source,
        "tracked": tracked_value.unwrap_or(Value::Null),
        "local": local_value.unwrap_or(Value::Null),
    });
    Ok((result, text))
}

fn run_validate(ctx: &GCtx) -> (Value, String) {
    let mut problems = Vec::new();
    let mut layers = Vec::new();
    for file in [TRACKED, LOCAL] {
        match read_layer(&ctx.root, file) {
            Ok(layer) => layers.push(layer),
            Err(message) => {
                problems.push(json!({"code": "config-unparseable", "runtime": null, "slot": null, "message": message}));
                layers.push(None);
            }
        }
    }
    if layers.iter().any(Option::is_some) {
        for p in validate_models_config(Some(&merged(&layers[0], &layers[1]))) {
            problems.push(json!({"code": p.code, "runtime": p.runtime, "slot": p.slot, "message": p.message}));
        }
    }
    let text = if problems.is_empty() {
        "config validate: OK — both config files parse and the team config has no problems.".to_string()
    } else {
        problems
            .iter()
            .map(|p| {
                let at = match (p["runtime"].as_str(), p["slot"].as_str()) {
                    (Some(rt), Some(slot)) => format!(" team.{rt}.{slot}:"),
                    _ => String::new(),
                };
                format!("[{}]{at} {}", p["code"].as_str().unwrap_or(""), p["message"].as_str().unwrap_or(""))
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let result = json!({"ok": problems.is_empty(), "problem_count": problems.len(), "problems": problems});
    (result, text)
}

fn str_flag(flags: &Flags, name: &str) -> Option<String> {
    match flags.get(name) {
        Some(FlagV::S(s)) => Some(s.clone()),
        _ => None,
    }
}

pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {
    if args.first()?.to_str()? != "config" {
        return None;
    }
    let (cmd, known): (&'static str, &[&str]) = match args.get(1)?.to_str()? {
        "get" => ("config get", &["key", "local"]),
        "set" => ("config set", &["key", "value", "string", "local"]),
        "unset" => ("config unset", &["key", "local"]),
        "explain" => ("config explain", &["key"]),
        "validate" => ("config validate", &[]),
        _ => return None,
    };
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.iter().any(|t| *t == "--help") {
        return None;
    }
    let pre_json = pre_json_scan(&toks);
    let (flags, json) = parse_flags(&toks)?;
    if !keys_known(&flags, known) {
        return None;
    }
    let local = js_bool_flag(&flags, "local")?;
    let as_string = js_bool_flag(&flags, "string")?;
    let key = str_flag(&flags, "key").filter(|k| !k.is_empty());
    let value = str_flag(&flags, "value");
    if (known.contains(&"key") && key.is_none()) || (known.contains(&"value") && value.is_none()) {
        return None;
    }

    let ctx = match g_prelude(cmd, json, pre_json, t0)? {
        GPre::Go(c) => c,
        GPre::Emitted(code) => return Some(code),
    };
    let key = key.unwrap_or_default();
    let outcome = match cmd {
        "config get" => run_get(&ctx, &key, local),
        "config set" => run_set(&ctx, &key, &value.unwrap_or_default(), as_string, local),
        "config unset" => run_unset(&ctx, &key, local),
        "config explain" => run_explain(&ctx, &key),
        _ => {
            let (result, text) = run_validate(&ctx);
            let code = if result["ok"] == Value::Bool(true) { 0 } else { 1 };
            return Some(ctx.emit(&result, &text, code));
        }
    };
    Some(match outcome {
        Ok((result, text)) => ctx.emit(&result, &text, 0),
        Err(message) => ctx.fail(&format!("bee {cmd}: {message}")),
    })
}

#[cfg(test)]
mod cross_project_allow_tests {
    use super::governed_refusal;
    use serde_json::json;

    #[test]
    fn cross_project_allow_cannot_be_added_through_config_set() {
        let before = json!({});
        let after = json!({ "guards": { "cross_project_allow": ["/somewhere"] } });
        let reason = governed_refusal("guards.cross_project_allow", ".bee/config.json", &before, &after)
            .expect("the governed key is refused");
        assert!(reason.starts_with("refused, nothing changed."), "{reason}");
        assert!(reason.contains("guards.cross_project_allow"), "{reason}");
    }
}

#[cfg(test)]
mod cross_project_supervisor_tests {
    use super::governed_refusal;
    use serde_json::json;

    #[test]
    fn cross_project_supervisor_cannot_be_set_through_config_set() {
        let before = json!({});
        let after = json!({ "guards": { "supervises": "waggledance" } });
        let reason = governed_refusal("guards.supervises", ".bee/config.json", &before, &after)
            .expect("the governed key is refused");
        assert!(reason.starts_with("refused, nothing changed."), "{reason}");
        assert!(reason.contains("guards.supervises"), "{reason}");
    }
}
