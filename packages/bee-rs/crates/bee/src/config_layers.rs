use crate::fsutil::{read_json, warn_corrupt_json, ReadJson};
use crate::state::merge_config_overlay;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Layer {
    Global,
    Tracked,
    Team,
    Local,
}

impl Layer {
    pub(crate) const ORDER: [Layer; 4] = [Layer::Global, Layer::Tracked, Layer::Team, Layer::Local];

    pub(crate) fn path(self, root: &Path, global_dir: Option<&Path>) -> Option<PathBuf> {
        match self {
            Layer::Global => global_dir.map(|dir| dir.join("config.json")),
            Layer::Tracked => Some(root.join(".bee").join("config.json")),
            Layer::Team => Some(root.join(".bee").join("team.json")),
            Layer::Local => Some(root.join(".bee").join("config.local.json")),
        }
    }

    fn restrict(self, mut object: Map<String, Value>) -> Map<String, Value> {
        match self {
            Layer::Global => {
                for key in GLOBAL_FORBIDDEN_KEYS {
                    object.shift_remove(key);
                }
                object
            }
            Layer::Team => keep_paths(&object, &TEAM_FILE_KEYS),
            Layer::Tracked | Layer::Local => object,
        }
    }
}

pub(crate) const GLOBAL_FORBIDDEN_KEYS: [&str; 3] = ["guards", "gate_bypass", "hooks"];

pub(crate) const TEAM_FILE_KEYS: [&str; 3] = ["team", "herding.agents", "herding.agent_command"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CorruptPolicy {
    Strict,
    FailOpen,
}

pub(crate) type LayerSet = Vec<(Layer, Option<Map<String, Value>>)>;

fn keep_paths(source: &Map<String, Value>, paths: &[&str]) -> Map<String, Value> {
    let mut kept = Map::new();
    for path in paths {
        let mut segments = path.split('.').collect::<Vec<_>>();
        let leaf = segments.pop().unwrap_or_default();
        let mut cursor = Some(source);
        for segment in &segments {
            cursor = cursor.and_then(|m| m.get(*segment)).and_then(Value::as_object);
        }
        let Some(value) = cursor.and_then(|m| m.get(leaf)) else { continue };
        let mut target = &mut kept;
        for segment in &segments {
            target = target
                .entry((*segment).to_string())
                .or_insert_with(|| Value::Object(Map::new()))
                .as_object_mut()
                .expect("keep_paths only inserts objects on the way down");
        }
        target.insert(leaf.to_string(), value.clone());
    }
    kept
}

#[cfg(not(test))]
pub(crate) fn global_config_dir() -> Option<PathBuf> {
    use crate::hooks::session_close::{env_nonempty, node_homedir};
    if let Some(dir) = env_nonempty("BEEHIVE_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(xdg) = env_nonempty("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("beehive"));
    }
    Some(PathBuf::from(node_homedir()).join(".config").join("beehive"))
}

#[cfg(test)]
pub(crate) fn global_config_dir() -> Option<PathBuf> {
    None
}

pub(crate) fn build_layers(root: &Path, global_dir: Option<&Path>, policy: CorruptPolicy) -> LayerSet {
    Layer::ORDER
        .iter()
        .map(|layer| {
            let parsed = layer.path(root, global_dir).and_then(|file| match read_json(&file) {
                ReadJson::Missing => None,
                ReadJson::Corrupt => {
                    warn_corrupt_json(&file);
                    if policy == CorruptPolicy::Strict && *layer == Layer::Global {
                        eprintln!(
                            "bee: the global config file {} is unreadable and was skipped; run `bee config validate`.",
                            file.display()
                        );
                    }
                    None
                }
                ReadJson::Parsed(Value::Object(m)) => Some(layer.restrict(m)),
                ReadJson::Parsed(_) => None,
            });
            (*layer, parsed)
        })
        .collect()
}

pub(crate) fn merge_layers(layers: &LayerSet) -> Map<String, Value> {
    let merged = layers
        .iter()
        .filter_map(|(_, parsed)| parsed.as_ref())
        .fold(Value::Object(Map::new()), |acc, layer| {
            merge_config_overlay(&acc, &Value::Object(layer.clone()))
        });
    match merged {
        Value::Object(m) => m,
        _ => Map::new(),
    }
}

pub(crate) fn read_merged(root: &Path, policy: CorruptPolicy) -> Map<String, Value> {
    merge_layers(&build_layers(root, global_config_dir().as_deref(), policy))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn put(path: PathBuf, value: Value) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, value.to_string()).unwrap();
    }

    struct Fixture {
        root: tempfile::TempDir,
        global: tempfile::TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            Fixture { root: tempfile::tempdir().unwrap(), global: tempfile::tempdir().unwrap() }
        }
        fn layer(&self, layer: Layer, value: Value) {
            put(layer.path(self.root.path(), Some(self.global.path())).unwrap(), value);
        }
        fn raw(&self, layer: Layer, text: &str) {
            let path = layer.path(self.root.path(), Some(self.global.path())).unwrap();
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        fn merged(&self, policy: CorruptPolicy) -> Map<String, Value> {
            merge_layers(&build_layers(self.root.path(), Some(self.global.path()), policy))
        }
    }

    #[test]
    fn config_layers_global_key_reaches_merged_config() {
        let f = Fixture::new();
        f.layer(Layer::Global, json!({"ship_visibility": "draft-pr"}));
        assert_eq!(f.merged(CorruptPolicy::Strict)["ship_visibility"], json!("draft-pr"));
    }

    #[test]
    fn config_layers_tracked_beats_global() {
        let f = Fixture::new();
        f.layer(Layer::Global, json!({"k": "global", "only_global": 1}));
        f.layer(Layer::Tracked, json!({"k": "tracked"}));
        let merged = f.merged(CorruptPolicy::Strict);
        assert_eq!(merged["k"], json!("tracked"));
        assert_eq!(merged["only_global"], json!(1));
    }

    #[test]
    fn config_layers_team_beats_tracked() {
        let f = Fixture::new();
        f.layer(Layer::Tracked, json!({"team": {"claude": {"generation": "tracked"}}}));
        f.layer(Layer::Team, json!({"team": {"claude": {"generation": "team"}}}));
        assert_eq!(f.merged(CorruptPolicy::Strict)["team"]["claude"]["generation"], json!("team"));
    }

    #[test]
    fn config_layers_local_beats_team() {
        let f = Fixture::new();
        f.layer(Layer::Team, json!({"team": {"claude": {"generation": "team"}}}));
        f.layer(Layer::Local, json!({"team": {"claude": {"generation": "local"}}}));
        assert_eq!(f.merged(CorruptPolicy::Strict)["team"]["claude"]["generation"], json!("local"));
    }

    #[test]
    fn config_layers_team_file_foreign_keys_are_ignored() {
        let f = Fixture::new();
        f.layer(
            Layer::Team,
            json!({
                "team": {"a": 1},
                "herding": {"agents": {"x": 1}, "agent_command": "run", "max_parallel": 9},
                "ship_visibility": "draft-pr"
            }),
        );
        let merged = f.merged(CorruptPolicy::FailOpen);
        assert_eq!(merged["team"], json!({"a": 1}));
        assert_eq!(merged["herding"], json!({"agents": {"x": 1}, "agent_command": "run"}));
        assert!(!merged.contains_key("ship_visibility"));
    }

    #[test]
    fn config_layers_global_forbidden_keys_are_dropped() {
        let f = Fixture::new();
        f.layer(
            Layer::Global,
            json!({
                "guards": {"cross_project": false},
                "gate_bypass": "total",
                "hooks": {"write-guard": false},
                "kept": true
            }),
        );
        let merged = f.merged(CorruptPolicy::Strict);
        for key in GLOBAL_FORBIDDEN_KEYS {
            assert!(!merged.contains_key(key), "{key} leaked from the global file");
        }
        assert_eq!(merged["kept"], json!(true));
    }

    #[test]
    fn config_layers_global_hooks_false_keeps_write_guard_enabled() {
        let f = Fixture::new();
        f.layer(Layer::Global, json!({"hooks": {"write-guard": false}}));
        let merged = f.merged(CorruptPolicy::Strict);
        let enabled = !matches!(
            merged.get("hooks").and_then(|h| h.get("write-guard")),
            Some(Value::Bool(false))
        );
        assert!(enabled);
    }

    #[test]
    fn config_layers_corrupt_layer_reads_as_empty_for_both_policies() {
        for policy in [CorruptPolicy::Strict, CorruptPolicy::FailOpen] {
            for layer in Layer::ORDER {
                let f = Fixture::new();
                f.layer(Layer::Tracked, json!({"keep": 1}));
                if layer != Layer::Tracked {
                    f.raw(layer, "{not json");
                }
                let merged = f.merged(policy);
                if layer != Layer::Tracked {
                    assert_eq!(merged, json!({"keep": 1}).as_object().unwrap().clone());
                }
            }
            let f = Fixture::new();
            f.raw(Layer::Tracked, "{not json");
            assert!(f.merged(policy).is_empty());
        }
    }

    #[test]
    fn config_layers_models_and_team_across_layers_fold_once_after_merge() {
        let f = Fixture::new();
        f.layer(Layer::Tracked, json!({"models": {"claude": {"generation": "haiku"}}}));
        f.layer(Layer::Local, json!({"team": {"claude": {"generation": "sonnet"}}}));
        let mut merged = f.merged(CorruptPolicy::Strict);
        assert!(merged.contains_key("models"));
        crate::verbs::drivers::fold_team_key(&mut merged);
        assert_eq!(merged["team"], json!({"claude": {"generation": "sonnet"}}));
        assert!(!merged.contains_key("models"));
    }

    #[test]
    fn config_layers_tracked_and_local_only_matches_the_plain_overlay() {
        let f = Fixture::new();
        let tracked = json!({"a": {"x": 1, "y": [1, 2]}, "b": "t", "c": 3});
        let local = json!({"a": {"y": [9]}, "b": "l", "d": 4});
        f.layer(Layer::Tracked, tracked.clone());
        f.layer(Layer::Local, local.clone());
        let expected = merge_config_overlay(&tracked, &local);
        let expected_keys: Vec<_> = expected.as_object().unwrap().keys().cloned().collect();
        let merged = f.merged(CorruptPolicy::Strict);
        assert_eq!(Value::Object(merged.clone()), expected);
        assert_eq!(merged.keys().cloned().collect::<Vec<_>>(), expected_keys);
    }

    #[test]
    fn config_layers_default_global_dir_is_absent_under_test() {
        assert!(global_config_dir().is_none());
    }

    #[test]
    fn config_layers_readers_with_only_tracked_and_local_match_each_other() {
        let tmp = tempfile::tempdir().unwrap();
        put(tmp.path().join(".bee/config.json"), json!({"models": {"c": 1}, "advisor": {"a": 1}, "k": {"x": 1}}));
        put(tmp.path().join(".bee/config.local.json"), json!({"k": {"y": 2}}));
        let strict = crate::state::read_config_raw(tmp.path());
        let open = crate::hooks::session_preamble::read_config_raw_open(tmp.path());
        let failopen = crate::hooks::compaction::read_config_failopen(tmp.path());
        assert_eq!(strict, open);
        let mut with_advisor = strict.clone();
        with_advisor.insert("advisor".to_string(), json!({"a": 1}));
        assert_eq!(failopen.get("advisor"), Some(&json!({"a": 1})));
        failopen.iter().filter(|(k, _)| *k != "advisor").for_each(|(k, v)| assert_eq!(strict.get(k), Some(v)));
        assert_eq!(strict["k"], json!({"x": 1, "y": 2}));
        assert_eq!(strict["team"], json!({"c": 1}));
    }
}
