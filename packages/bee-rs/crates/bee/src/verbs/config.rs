use crate::config_layers::{
    global_config_dir, merge_layers, Layer, LayerSet, GLOBAL_FORBIDDEN_KEYS, TEAM_FILE_KEYS,
};
use crate::fsutil::{read_json, write_json_atomic, ReadJson};
use crate::hooks::write_guard::config_governed_change_deny;
use crate::jsjson;
use crate::verbs::knowledge::{g_prelude, js_bool_flag, pre_json_scan, GCtx, GPre};
use crate::verbs::reservations::{keys_known, parse_flags, Flags, FlagV};
use crate::verbs::status_full::validate_models_config;
use serde_json::{json, Map, Value};
use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

const TRACKED: &str = ".bee/config.json";
const TEAM: &str = ".bee/team.json";
const LOCAL: &str = ".bee/config.local.json";
const DEFAULTS_SECTION: &str = "docs/config-reference.md \"Other keys\"";

type Obj = Option<Map<String, Value>>;

fn rel(local: bool) -> &'static str {
    if local {
        LOCAL
    } else {
        TRACKED
    }
}

fn layer_name(layer: Layer) -> &'static str {
    match layer {
        Layer::Global => "global",
        Layer::Tracked => "tracked",
        Layer::Team => "team",
        Layer::Local => "local",
    }
}

fn layer_file(root: &Path, layer: Layer) -> String {
    match layer {
        Layer::Global => match Layer::Global.path(root, global_config_dir().as_deref()) {
            Some(path) => path.display().to_string(),
            None => "the global config file".to_string(),
        },
        Layer::Tracked => TRACKED.to_string(),
        Layer::Team => TEAM.to_string(),
        Layer::Local => LOCAL.to_string(),
    }
}

fn read_raw(root: &Path, layer: Layer) -> Result<Obj, String> {
    let Some(path) = layer.path(root, global_config_dir().as_deref()) else { return Ok(None) };
    let rel = layer_file(root, layer);
    match read_json(&path) {
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

fn read_layer(root: &Path, layer: Layer) -> Result<Obj, String> {
    Ok(read_raw(root, layer)?.map(|m| layer.restrict(m)))
}

fn read_layers(root: &Path) -> Result<LayerSet, String> {
    Layer::ORDER.iter().map(|layer| Ok((*layer, read_layer(root, *layer)?))).collect()
}

fn layer_of(set: &LayerSet, layer: Layer) -> &Obj {
    &set.iter().find(|(l, _)| *l == layer).expect("every layer is in the set").1
}

fn with_layer(set: &LayerSet, layer: Layer, replacement: Obj) -> LayerSet {
    set.iter().map(|(l, m)| (*l, if *l == layer { replacement.clone() } else { m.clone() })).collect()
}

fn merged_value(set: &LayerSet) -> Value {
    Value::Object(merge_layers(set))
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

fn tracked_warning(tracked: &Obj, parts: &[&str], key: &str, local: bool) -> Value {
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
    let set = read_layers(&ctx.root)?;
    let layer = if to_local { Layer::Local } else { Layer::Tracked };
    let target = rel(to_local);
    let warning = tracked_warning(layer_of(&set, Layer::Tracked), &parts, key, to_local);
    let mut edited = layer_of(&set, layer).clone().unwrap_or_default();
    if !apply(&mut edited, &parts) {
        return Ok((false, warning));
    }
    let before = merged_value(&set);
    let after = merged_value(&with_layer(&set, layer, Some(edited.clone())));
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

fn run_get(ctx: &GCtx, key: &str, local: bool, global: bool) -> Result<(Value, String), String> {
    if global && local {
        return Err("--global and --local cannot be combined. FIX: pass one of them, or neither to read the tracked file.".to_string());
    }
    let parts = split_key(key)?;
    let tracked = read_layer(&ctx.root, Layer::Tracked)?;
    let layer = if global {
        read_layer(&ctx.root, Layer::Global)?
    } else if local {
        read_layer(&ctx.root, Layer::Local)?
    } else {
        tracked.clone()
    };
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
        "global": global,
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

fn explain_source(set: &LayerSet, parts: &[&str]) -> &'static str {
    Layer::ORDER
        .iter()
        .rev()
        .find(|layer| get_at(layer_of(set, **layer).as_ref(), parts).is_some())
        .map_or("unset", |layer| layer_name(*layer))
}

fn run_explain(ctx: &GCtx, key: &str) -> Result<(Value, String), String> {
    let parts = split_key(key)?;
    let set = read_layers(&ctx.root)?;
    let effective = get_at(merged_value(&set).as_object(), &parts).cloned();
    let value_in = |layer: Layer| get_at(layer_of(&set, layer).as_ref(), &parts).cloned();
    let source = explain_source(&set, &parts);
    let show = |v: &Option<Value>| v.as_ref().map(jsjson::stringify).unwrap_or_else(|| "(not set)".into());
    let global_file = layer_file(&ctx.root, Layer::Global);
    let text = match source {
        "unset" => format!(
            "{key} is not set in the global config ({global_file}), {TRACKED}, {TEAM} or {LOCAL}; the built-in default applies — see {DEFAULTS_SECTION}."
        ),
        _ => format!(
            "{key} = {} (from {source})\n  global  {global_file}: {}\n  tracked {TRACKED}: {}\n  team    {TEAM}: {}\n  local   {LOCAL}: {}",
            show(&effective),
            show(&value_in(Layer::Global)),
            show(&value_in(Layer::Tracked)),
            show(&value_in(Layer::Team)),
            show(&value_in(Layer::Local))
        ),
    };
    let result = json!({
        "key": key,
        "effective": effective.unwrap_or(Value::Null),
        "source": source,
        "global": value_in(Layer::Global).unwrap_or(Value::Null),
        "tracked": value_in(Layer::Tracked).unwrap_or(Value::Null),
        "team": value_in(Layer::Team).unwrap_or(Value::Null),
        "local": value_in(Layer::Local).unwrap_or(Value::Null),
    });
    Ok((result, text))
}

fn problem(code: &str, file: &str, key: &str, message: String) -> Value {
    json!({"code": code, "runtime": null, "slot": null, "file": file, "key": key, "message": message})
}

fn leaf_paths(map: &Map<String, Value>, prefix: &str, out: &mut Vec<String>) {
    for (key, value) in map {
        let path = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
        match value {
            Value::Object(child) if !child.is_empty() => leaf_paths(child, &path, out),
            _ => out.push(path),
        }
    }
}

fn leaves(map: &Map<String, Value>) -> Vec<String> {
    let mut out = Vec::new();
    leaf_paths(map, "", &mut out);
    out
}

fn foreign_paths(raw: &Map<String, Value>, kept: &Map<String, Value>, prefix: &str, out: &mut Vec<String>) {
    for (key, value) in raw {
        let path = if prefix.is_empty() { key.clone() } else { format!("{prefix}.{key}") };
        match (kept.get(key), value) {
            (None, _) => out.push(path),
            (Some(Value::Object(inner)), Value::Object(raw_inner)) => foreign_paths(raw_inner, inner, &path, out),
            _ => {}
        }
    }
}

fn parts_of(path: &str) -> Vec<&str> {
    path.split('.').collect()
}

fn check_layers(raws: &[(Layer, Obj)], global_file: &str) -> (Vec<Value>, Vec<String>) {
    let raw_of = |layer: Layer| raws.iter().find(|(l, _)| *l == layer).and_then(|(_, m)| m.as_ref());
    let mut problems = Vec::new();
    let team_kept = raw_of(Layer::Team).map(|raw| Layer::Team.restrict(raw.clone()));
    if let (Some(raw), Some(kept)) = (raw_of(Layer::Team), team_kept.as_ref()) {
        let mut foreign = Vec::new();
        foreign_paths(raw, kept, "", &mut foreign);
        for path in foreign {
            problems.push(problem(
                "team-foreign-key",
                TEAM,
                &path,
                format!("{TEAM} sets \"{path}\", which is not a team-table key and is not read. FIX: move it to {TRACKED} or remove it."),
            ));
        }
    }
    if let Some(kept) = team_kept.as_ref() {
        for path in leaves(kept) {
            if get_at(raw_of(Layer::Tracked), &parts_of(&path)).is_some() {
                problems.push(problem(
                    "duplicate",
                    TEAM,
                    &path,
                    format!("\"{path}\" is set in both {TRACKED} and {TEAM}; {TEAM} wins. FIX: remove it from {TRACKED}."),
                ));
            }
        }
    }
    if let Some(global) = raw_of(Layer::Global) {
        for key in GLOBAL_FORBIDDEN_KEYS {
            if global.contains_key(key) {
                problems.push(problem(
                    "global-forbidden-key",
                    global_file,
                    key,
                    format!("{global_file} sets \"{key}\", which a global config may not set and is ignored. FIX: remove it from that file."),
                ));
            }
        }
    }
    let project: Vec<Option<Map<String, Value>>> = [Layer::Tracked, Layer::Team, Layer::Local]
        .iter()
        .map(|layer| raw_of(*layer).map(|m| layer.restrict(m.clone())))
        .collect();
    let mut supplied: Vec<String> = raw_of(Layer::Global)
        .map(|m| leaves(&Layer::Global.restrict(m.clone())))
        .unwrap_or_default()
        .into_iter()
        .filter(|path| project.iter().all(|layer| get_at(layer.as_ref(), &parts_of(path)).is_none()))
        .collect();
    supplied.sort();
    (problems, supplied)
}

fn run_validate(ctx: &GCtx) -> (Value, String) {
    let mut problems = Vec::new();
    let mut raws = Vec::new();
    for layer in Layer::ORDER {
        match read_raw(&ctx.root, layer) {
            Ok(parsed) => raws.push((layer, parsed)),
            Err(message) => {
                let file = layer_file(&ctx.root, layer);
                problems.push(problem("config-unparseable", &file, "", message));
                raws.push((layer, None));
            }
        }
    }
    let (found, supplied) = check_layers(&raws, &layer_file(&ctx.root, Layer::Global));
    problems.extend(found);
    if raws.iter().any(|(_, m)| m.is_some()) {
        let restricted: LayerSet = raws.iter().map(|(l, m)| (*l, m.clone().map(|m| l.restrict(m)))).collect();
        for p in validate_models_config(Some(&merged_value(&restricted))) {
            problems.push(json!({"code": p.code, "runtime": p.runtime, "slot": p.slot, "file": null, "key": null, "message": p.message}));
        }
    }
    let mut text = if problems.is_empty() {
        "config validate: OK — the four config files parse and the team config has no problems.".to_string()
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
    if !supplied.is_empty() {
        text.push_str(&format!("\nthe global config supplies: {}", supplied.join(", ")));
    }
    let result = json!({"ok": problems.is_empty(), "problem_count": problems.len(), "problems": problems, "global_supplied": supplied});
    (result, text)
}

struct SplitPlan {
    tracked: Map<String, Value>,
    team: Map<String, Value>,
    moved: Vec<String>,
    dropped: Vec<String>,
}

fn team_leaf_paths(team_file: &Map<String, Value>) -> Vec<String> {
    let mut out = Vec::new();
    match team_file.get("team") {
        Some(Value::Object(runtimes)) => {
            for (runtime, slots) in runtimes {
                match slots {
                    Value::Object(slots) => out.extend(slots.keys().map(|slot| format!("team.{runtime}.{slot}"))),
                    _ => out.push(format!("team.{runtime}")),
                }
            }
        }
        Some(_) => out.push("team".to_string()),
        None => {}
    }
    if let Some(Value::Object(agents)) = get_at(Some(team_file), &["herding", "agents"]) {
        out.extend(agents.keys().map(|name| format!("herding.agents.{name}")));
    }
    if get_at(Some(team_file), &["herding", "agent_command"]).is_some() {
        out.push("herding.agent_command".to_string());
    }
    out
}

fn plan_split_team(set: &LayerSet, raw_team: &Obj, drop_global: bool) -> Result<SplitPlan, String> {
    let mut tracked = layer_of(set, Layer::Tracked).clone().unwrap_or_default();
    let mut team = raw_team.clone().unwrap_or_default();
    let mut moved = Vec::new();
    for path in TEAM_FILE_KEYS {
        let parts = parts_of(path);
        let Some(value) = get_at(Some(&tracked), &parts).cloned() else { continue };
        match get_at(Some(&team), &parts) {
            Some(existing) if *existing != value => {
                return Err(format!(
                    "{TEAM} already sets \"{path}\" to a different value than {TRACKED}. Nothing changed. FIX: keep the value you want in {TEAM}, remove \"{path}\" from {TRACKED}, then run the command again."
                ));
            }
            Some(_) => {}
            None => set_at(&mut team, &parts, value),
        }
        unset_at(&mut tracked, &parts);
        moved.push(path.to_string());
    }
    let mut dropped = Vec::new();
    if drop_global {
        let global = layer_of(set, Layer::Global).as_ref();
        for path in team_leaf_paths(&team) {
            let parts = parts_of(&path);
            let equal = match (get_at(global, &parts), get_at(Some(&team), &parts)) {
                (Some(g), Some(t)) => g == t,
                _ => false,
            };
            if equal {
                unset_at(&mut team, &parts);
                dropped.push(path);
            }
        }
    }
    let compared = |s: &LayerSet| if drop_global { s.clone() } else { with_layer(s, Layer::Global, None) };
    let after = with_layer(
        &with_layer(set, Layer::Tracked, Some(tracked.clone())),
        Layer::Team,
        Some(Layer::Team.restrict(team.clone())),
    );
    if merged_value(&compared(set)) != merged_value(&compared(&after)) {
        return Err(format!(
            "the merged config would change. Nothing changed. FIX: run `bee config validate`, resolve the problems it names in {TRACKED} and {TEAM}, then run the command again."
        ));
    }
    Ok(SplitPlan { tracked, team, moved, dropped })
}

fn run_split_team(ctx: &GCtx, drop_global: bool) -> Result<(Value, String), String> {
    let set = read_layers(&ctx.root)?;
    let raw_team = read_raw(&ctx.root, Layer::Team)?;
    let plan = plan_split_team(&set, &raw_team, drop_global)?;
    let tracked_changed = plan.tracked != layer_of(&set, Layer::Tracked).clone().unwrap_or_default();
    let team_changed = plan.team != raw_team.unwrap_or_default();
    if team_changed {
        write_json_atomic(&ctx.root.join(TEAM), &Value::Object(plan.team.clone()))
            .map_err(|e| format!("could not write {TEAM}: {e}"))?;
    }
    if tracked_changed {
        write_json_atomic(&ctx.root.join(TRACKED), &Value::Object(plan.tracked.clone()))
            .map_err(|e| format!("could not write {TRACKED}: {e}"))?;
    }
    let result = json!({"moved": plan.moved, "dropped": plan.dropped, "drop_global": drop_global});
    let text = if plan.moved.is_empty() && plan.dropped.is_empty() {
        format!("config split-team: nothing to move; {TRACKED} holds no team table.")
    } else {
        let mut text = format!("config split-team: moved {} to {TEAM}.", plan.moved.join(", "));
        if !plan.dropped.is_empty() {
            text.push_str(&format!(" Dropped {} equal to the global config: {}.", plan.dropped.len(), plan.dropped.join(", ")));
        }
        text.push_str(&format!(" Commit {TRACKED} and {TEAM} together."));
        text
    };
    Ok((result, text))
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
        "split-team" => ("config split-team", &[]),
        "set" => ("config set", &["key", "value", "string", "local"]),
        "unset" => ("config unset", &["key", "local"]),
        "explain" => ("config explain", &["key"]),
        "validate" => ("config validate", &[]),
        _ => return None,
    };
    let toks: Vec<&str> = args[2..].iter().map(|a| a.to_str()).collect::<Option<Vec<_>>>()?;
    if toks.contains(&"--help") {
        return None;
    }
    let global = toks.contains(&"--global");
    let drop_global = toks.contains(&"--drop-global");
    if (global && cmd != "config get") || (drop_global && cmd != "config split-team") {
        return None;
    }
    let toks: Vec<&str> = toks.into_iter().filter(|t| *t != "--global" && *t != "--drop-global").collect();
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
        "config get" => run_get(&ctx, &key, local, global),
        "config split-team" => run_split_team(&ctx, drop_global),
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

#[cfg(test)]
mod layered_tests {
    use super::*;

    fn obj(value: Value) -> Obj {
        Some(value.as_object().unwrap().clone())
    }

    fn set(global: Value, tracked: Value, team: Value, local: Value) -> LayerSet {
        vec![
            (Layer::Global, obj(global).map(|m| Layer::Global.restrict(m))),
            (Layer::Tracked, obj(tracked)),
            (Layer::Team, obj(team).map(|m| Layer::Team.restrict(m))),
            (Layer::Local, obj(local)),
        ]
    }

    #[test]
    fn config_global_explain_source_is_the_highest_layer_that_sets_the_key() {
        let s = set(
            json!({"a": 1, "b": 1, "c": 1, "d": 1}),
            json!({"b": 2, "c": 2, "d": 2}),
            json!({"team": {"claude": {"code": 3}}, "herding": {"agent_command": "x"}}),
            json!({"d": 4}),
        );
        for (key, source) in [
            ("a", "global"),
            ("b", "tracked"),
            ("team.claude.code", "team"),
            ("herding.agent_command", "team"),
            ("d", "local"),
            ("missing", "unset"),
        ] {
            assert_eq!(explain_source(&s, &parts_of(key)), source, "{key}");
        }
    }

    #[test]
    fn config_global_check_layers_names_file_and_key_for_each_problem_kind() {
        let raws = vec![
            (Layer::Global, obj(json!({"guards": {}, "gate_bypass": "total", "hooks": {}, "ok": 1}))),
            (Layer::Tracked, obj(json!({"herding": {"agent_command": "a"}}))),
            (Layer::Team, obj(json!({"herding": {"agent_command": "b", "enabled": true}, "extra": 1}))),
            (Layer::Local, None),
        ];
        let (problems, _) = check_layers(&raws, "/g/config.json");
        let find = |code: &str, key: &str| {
            problems.iter().find(|p| p["code"] == json!(code) && p["key"] == json!(key)).unwrap_or_else(|| panic!("{code} {key}"))
        };
        assert_eq!(find("team-foreign-key", "extra")["file"], json!(".bee/team.json"));
        assert_eq!(find("team-foreign-key", "herding.enabled")["file"], json!(".bee/team.json"));
        assert_eq!(find("duplicate", "herding.agent_command")["file"], json!(".bee/team.json"));
        for key in GLOBAL_FORBIDDEN_KEYS {
            assert_eq!(find("global-forbidden-key", key)["file"], json!("/g/config.json"));
        }
        assert_eq!(problems.len(), 6);
    }

    #[test]
    fn config_global_check_layers_lists_only_keys_no_project_file_sets() {
        let raws = vec![
            (Layer::Global, obj(json!({"only": 1, "shared": 2, "nested": {"x": 1, "y": 2}, "guards": {"z": 1}}))),
            (Layer::Tracked, obj(json!({"shared": 3, "nested": {"y": 9}}))),
            (Layer::Team, None),
            (Layer::Local, None),
        ];
        let (_, supplied) = check_layers(&raws, "g");
        assert_eq!(supplied, vec!["nested.x".to_string(), "only".to_string()]);
    }

    fn real_table() -> Value {
        json!({
            "team": {"pi": {"code": {"agent": "a"}, "read": {"agent": "b"}}},
            "herding": {"enabled": true, "agent_command": "a", "agents": {"a": ["run"], "b": ["go"]}}
        })
    }

    fn plan(global: Value, tracked: Value, team: Value, drop: bool) -> Result<SplitPlan, String> {
        let s = set(global, tracked, team.clone(), json!({}));
        plan_split_team(&s, &obj(team), drop)
    }

    #[test]
    fn split_team_plan_moves_the_table_and_prunes_what_it_empties() {
        let p = plan(json!({}), real_table(), json!({}), false).unwrap();
        assert_eq!(p.moved, vec!["team", "herding.agents", "herding.agent_command"]);
        assert_eq!(Value::Object(p.tracked), json!({"herding": {"enabled": true}}));
        assert_eq!(Value::Object(p.team)["herding"]["agents"], real_table()["herding"]["agents"]);
        let only = json!({"team": {"pi": {"code": 1}}, "herding": {"agent_command": "a"}});
        let p = plan(json!({}), only, json!({}), false).unwrap();
        assert_eq!(Value::Object(p.tracked), json!({}));
    }

    #[test]
    fn split_team_plan_is_idempotent() {
        let first = plan(json!({}), real_table(), json!({}), false).unwrap();
        let second = plan(json!({}), Value::Object(first.tracked.clone()), Value::Object(first.team.clone()), false).unwrap();
        assert!(second.moved.is_empty() && second.dropped.is_empty());
        assert_eq!(second.tracked, first.tracked);
        assert_eq!(second.team, first.team);
    }

    #[test]
    fn split_team_plan_refuses_a_conflicting_value_and_names_the_key() {
        let err = plan(json!({}), real_table(), json!({"herding": {"agent_command": "other"}}), false).err().unwrap();
        assert!(err.contains("herding.agent_command") && err.contains("Nothing changed"), "{err}");
    }

    #[test]
    fn split_team_plan_finishes_from_equal_values_in_both_files() {
        let p = plan(json!({}), real_table(), real_table(), false).unwrap();
        assert_eq!(p.moved.len(), 3);
        assert_eq!(Value::Object(p.team)["team"], real_table()["team"]);
        assert_eq!(Value::Object(p.tracked), json!({"herding": {"enabled": true}}));
    }

    #[test]
    fn split_team_plan_drop_global_removes_equal_leaves_and_keeps_differing_ones() {
        let global = json!({
            "team": {"pi": {"code": {"agent": "a"}, "read": {"agent": "different"}}},
            "herding": {"agent_command": "a", "agents": {"a": ["run"]}}
        });
        let p = plan(global, real_table(), json!({}), true).unwrap();
        assert_eq!(p.dropped, vec!["team.pi.code", "herding.agents.a", "herding.agent_command"]);
        let team = Value::Object(p.team);
        assert_eq!(team["team"], json!({"pi": {"read": {"agent": "b"}}}));
        assert_eq!(team["herding"], json!({"agents": {"b": ["go"]}}));
    }

    #[test]
    fn split_team_plan_without_drop_global_ignores_the_global_layer() {
        let global = json!({"herding": {"agent_command": "a"}});
        let p = plan(global, real_table(), json!({}), false).unwrap();
        assert!(p.dropped.is_empty());
        assert_eq!(Value::Object(p.team)["herding"]["agent_command"], json!("a"));
    }
}
