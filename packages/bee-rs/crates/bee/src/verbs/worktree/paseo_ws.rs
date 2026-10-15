use crate::herding::paseo::{self, PaseoCli, RealPaseoCli};
use crate::verbs::workspace_store as ws;
use serde_json::{json, Value};
use std::path::Path;
use std::time::Duration;

fn recorded_id(record: &serde_json::Map<String, Value>) -> Option<String> {
    record
        .get("paseo_workspace_id")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .map(str::to_string)
}

pub(crate) fn recorded_for_dir(main_root: &Path, dir: &Path) -> Option<String> {
    ws::record_for_dir(main_root, dir).as_ref().and_then(recorded_id)
}

pub(crate) fn ensure_and_record(cli: &dyn PaseoCli, main_root: &Path, dir: &Path) -> Result<String, String> {
    let record = ws::record_for_dir(main_root, dir);
    if let Some(id) = record.as_ref().and_then(recorded_id) {
        return Ok(id);
    }
    let id = paseo::ensure_workspace(cli, main_root, dir)?;
    if let Some(record_id) = record.as_ref().and_then(|r| r.get("id")).and_then(Value::as_str) {
        ws::set_paseo_workspace_id(main_root, record_id, &id).map_err(|e| e.message().to_string())?;
    }
    Ok(id)
}

pub(crate) fn real_cli(main_root: &Path) -> RealPaseoCli {
    let cfg = std::fs::read_to_string(main_root.join(".bee").join("config.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .unwrap_or(Value::Null);
    RealPaseoCli::new(paseo::paseo_command(&cfg)).with_timeout(Duration::from_secs(8))
}

pub(crate) fn register_with_cli(cli: &dyn PaseoCli, main_root: &Path, worktree_root: &Path) -> Value {
    match ensure_and_record(cli, main_root, worktree_root) {
        Ok(id) => json!({ "id": id }),
        Err(error) => json!({ "error": error }),
    }
}

pub(crate) fn register(main_root: &Path, worktree_root: &Path) -> Option<Value> {
    if !paseo::inside_paseo() {
        return None;
    }
    eprintln!("▸ paseo: registering workspace…");
    Some(register_with_cli(&real_cli(main_root), main_root, worktree_root))
}

pub(crate) fn archive_with_cli(cli: &dyn PaseoCli, main_root: &Path, id: &str) {
    let Some(wks) = ws::read_workspace(main_root, id).ok().as_ref().and_then(recorded_id) else {
        return;
    };
    match paseo::archive_workspace(cli, &wks) {
        Ok(()) => eprintln!("paseo: archived workspace {wks}"),
        Err(error) => eprintln!(
            "paseo: could not archive workspace {wks}: {error} — FIX: paseo workspace archive {wks}"
        ),
    }
}

pub(crate) fn archive(main_root: &Path, id: &str) {
    let has_id = ws::read_workspace(main_root, id).ok().as_ref().and_then(recorded_id).is_some();
    if has_id {
        archive_with_cli(&real_cli(main_root), main_root, id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ScriptedCli {
        calls: std::sync::Mutex<Vec<Vec<String>>>,
        reply: Box<dyn Fn(&[String]) -> Result<String, String> + Send + Sync>,
    }

    impl ScriptedCli {
        fn new(reply: impl Fn(&[String]) -> Result<String, String> + Send + Sync + 'static) -> Self {
            Self { calls: std::sync::Mutex::new(Vec::new()), reply: Box::new(reply) }
        }

        fn calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl PaseoCli for ScriptedCli {
        fn call(&self, args: &[String]) -> Result<String, String> {
            self.calls.lock().unwrap().push(args.to_vec());
            (self.reply)(args)
        }
    }

    fn fixture() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let main = tmp.path().join("main");
        let wt = tmp.path().join("main--wt--demo");
        std::fs::create_dir_all(main.join(".bee")).unwrap();
        std::fs::create_dir_all(wt.join("sub")).unwrap();
        ws::register_workspace(
            &main,
            ws::RegisterSpec {
                id: "main--wt--demo",
                kind: "worktree",
                root: &wt.to_string_lossy(),
                branch: Some("wt/demo"),
                base_sha: None,
            },
            "2026-10-15T00:00:00.000Z",
        )
        .unwrap();
        (tmp, main, wt)
    }

    fn creating_cli(reply_create: Result<String, String>) -> ScriptedCli {
        ScriptedCli::new(move |args: &[String]| {
            if args.get(1).map(String::as_str) == Some("create") {
                reply_create.clone()
            } else {
                Ok("[]".to_string())
            }
        })
    }

    #[test]
    fn set_paseo_workspace_id_round_trips_on_the_record() {
        let (_tmp, main, _wt) = fixture();
        ws::set_paseo_workspace_id(&main, "main--wt--demo", "wks_1").unwrap();
        let record = ws::read_workspace(&main, "main--wt--demo").unwrap();
        assert_eq!(record["paseo_workspace_id"], "wks_1");
        assert_eq!(record["branch"], "wt/demo");
    }

    #[test]
    fn record_for_dir_finds_the_record_for_a_subdirectory() {
        let (tmp, main, wt) = fixture();
        let record = ws::record_for_dir(&main, &wt.join("sub")).expect("record");
        assert_eq!(record["id"], "main--wt--demo");
        assert!(ws::record_for_dir(&main, tmp.path()).is_none());
    }

    #[test]
    fn ensure_and_record_records_the_created_workspace() {
        let (_tmp, main, wt) = fixture();
        let cli = creating_cli(Ok("{\"workspaceId\":\"wks_new\"}".to_string()));
        assert_eq!(ensure_and_record(&cli, &main, &wt).unwrap(), "wks_new");
        assert_eq!(recorded_for_dir(&main, &wt).as_deref(), Some("wks_new"));
    }

    #[test]
    fn ensure_and_record_makes_no_call_when_the_record_has_an_id() {
        let (_tmp, main, wt) = fixture();
        ws::set_paseo_workspace_id(&main, "main--wt--demo", "wks_old").unwrap();
        let cli = creating_cli(Ok("{\"workspaceId\":\"wks_new\"}".to_string()));
        assert_eq!(ensure_and_record(&cli, &main, &wt.join("sub")).unwrap(), "wks_old");
        assert!(cli.calls().is_empty());
    }

    #[test]
    fn register_reports_the_error_and_leaves_the_record_unchanged() {
        let (_tmp, main, wt) = fixture();
        let cli = creating_cli(Err("{\"error\":{\"message\":\"daemon down\"}}".to_string()));
        assert_eq!(register_with_cli(&cli, &main, &wt), json!({ "error": "daemon down" }));
        let record = ws::read_workspace(&main, "main--wt--demo").unwrap();
        assert!(record.get("paseo_workspace_id").is_none());
    }

    #[test]
    fn register_is_none_outside_paseo() {
        let (_tmp, main, wt) = fixture();
        assert!(register(&main, &wt).is_none());
    }

    #[test]
    fn archive_with_cli_archives_only_a_recorded_workspace() {
        let (_tmp, main, _wt) = fixture();
        let cli = ScriptedCli::new(|_args: &[String]| Ok("{}".to_string()));
        archive_with_cli(&cli, &main, "main--wt--demo");
        assert!(cli.calls().is_empty());
        ws::set_paseo_workspace_id(&main, "main--wt--demo", "wks_9").unwrap();
        archive_with_cli(&cli, &main, "main--wt--demo");
        assert_eq!(cli.calls(), vec![paseo::workspace_archive_argv("wks_9")]);
    }
}
