use serde_json::json;
use std::path::{Path, PathBuf};

#[cfg(unix)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    std::os::unix::fs::symlink(source, target)
}

#[cfg(windows)]
fn create_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    if source.is_dir() {
        std::os::windows::fs::symlink_dir(source, target)
    } else {
        std::os::windows::fs::symlink_file(source, target)
    }
}

fn unlink_symlink(path: &Path) {
    if std::fs::remove_file(path).is_err() {
        let _ = std::fs::remove_dir(path);
    }
}

fn ensure_symlink(src: &Path, dst: &Path, notes: &mut Vec<String>) -> Result<(), String> {
    match std::fs::symlink_metadata(dst) {
        Ok(meta) => {
            if meta.file_type().is_symlink() {
                let matches = std::fs::read_link(dst)
                    .map(|target| target == src)
                    .unwrap_or(false);
                if !matches {
                    unlink_symlink(dst);
                    create_symlink(src, dst).map_err(|e| {
                        format!("could not create symlink at {}: {e}", dst.display())
                    })?;
                }
            } else {
                notes.push(format!(
                    "existing real {} at {} was not replaced with symlink to {}",
                    if meta.is_dir() { "directory" } else { "file" },
                    dst.display(),
                    src.display()
                ));
            }
        }
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            create_symlink(src, dst).map_err(|e| {
                format!("could not create symlink at {}: {e}", dst.display())
            })?;
        }
        Err(err) => {
            return Err(format!("could not inspect {}: {err}", dst.display()));
        }
    }
    Ok(())
}

pub fn ensure_pi_agent_dir(
    main_root: &Path,
    agent: &str,
    home: &Path,
) -> Result<(PathBuf, Vec<String>), String> {
    let agent_dir = main_root.join(".bee").join("runtime").join("pi-agent").join(agent);
    std::fs::create_dir_all(&agent_dir)
        .map_err(|e| format!("could not create {}: {e}", agent_dir.display()))?;

    let skills_dir = agent_dir.join("skills");
    if !skills_dir.exists() {
        std::fs::create_dir_all(&skills_dir)
            .map_err(|e| format!("could not create {}: {e}", skills_dir.display()))?;
    }

    let extensions_dir = agent_dir.join("extensions");
    if !extensions_dir.exists() {
        std::fs::create_dir_all(&extensions_dir)
            .map_err(|e| format!("could not create {}: {e}", extensions_dir.display()))?;
    }

    let desired_settings = json!({
        "defaultProjectTrust": "always",
        "quietStartup": true
    });
    let settings_file = agent_dir.join("settings.json");
    let needs_settings_write = match std::fs::read_to_string(&settings_file) {
        Ok(raw) => match serde_json::from_str::<serde_json::Value>(&raw) {
            Ok(val) => val != desired_settings,
            Err(_) => true,
        },
        Err(_) => true,
    };
    if needs_settings_write {
        crate::fsutil::write_json_atomic(&settings_file, &desired_settings)
            .map_err(|e| format!("could not write {}: {e}", settings_file.display()))?;
    }

    let mut notes = Vec::new();
    let pi_agent_home = home.join(".pi").join("agent");
    let auth_src = pi_agent_home.join("auth.json");
    if !auth_src.exists() {
        return Err(format!(
            "missing auth.json at {} — FIX: log in with pi once",
            auth_src.display()
        ));
    }
    ensure_symlink(&auth_src, &agent_dir.join("auth.json"), &mut notes)?;

    for name in ["models.json", "models-store.json", "npm"] {
        let src = pi_agent_home.join(name);
        if src.exists() {
            ensure_symlink(&src, &agent_dir.join(name), &mut notes)?;
        }
    }

    Ok((agent_dir, notes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_pi_agent_dir_creates_links_and_settings() {
        let tmp_root = tempfile::tempdir().unwrap();
        let tmp_home = tempfile::tempdir().unwrap();
        let pi_home = tmp_home.path().join(".pi").join("agent");
        std::fs::create_dir_all(&pi_home).unwrap();
        std::fs::write(pi_home.join("auth.json"), r#"{"token":"test-auth"}"#).unwrap();
        std::fs::write(pi_home.join("models.json"), r#"{"models":[]}"#).unwrap();
        std::fs::write(pi_home.join("models-store.json"), r#"{"store":[]}"#).unwrap();
        let npm_dir = pi_home.join("npm");
        std::fs::create_dir_all(&npm_dir).unwrap();
        std::fs::write(npm_dir.join("package.json"), "{}").unwrap();

        let (dir, notes) = ensure_pi_agent_dir(tmp_root.path(), "w-worker", tmp_home.path()).unwrap();
        assert!(notes.is_empty());
        assert_eq!(dir, tmp_root.path().join(".bee/runtime/pi-agent/w-worker"));

        let auth_link = dir.join("auth.json");
        assert!(std::fs::symlink_metadata(&auth_link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_link(&auth_link).unwrap(), pi_home.join("auth.json"));

        let models_link = dir.join("models.json");
        assert!(std::fs::symlink_metadata(&models_link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_link(&models_link).unwrap(), pi_home.join("models.json"));

        let models_store_link = dir.join("models-store.json");
        assert!(std::fs::symlink_metadata(&models_store_link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_link(&models_store_link).unwrap(), pi_home.join("models-store.json"));

        let npm_link = dir.join("npm");
        assert!(std::fs::symlink_metadata(&npm_link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_link(&npm_link).unwrap(), npm_dir);

        let settings_raw = std::fs::read_to_string(dir.join("settings.json")).unwrap();
        let settings_val: serde_json::Value = serde_json::from_str(&settings_raw).unwrap();
        assert_eq!(settings_val.get("defaultProjectTrust").and_then(|v| v.as_str()), Some("always"));
        assert_eq!(settings_val.get("quietStartup").and_then(|v| v.as_bool()), Some(true));

        assert!(dir.join("skills").is_dir());
        assert_eq!(std::fs::read_dir(dir.join("skills")).unwrap().count(), 0);

        assert!(dir.join("extensions").is_dir());
        assert_eq!(std::fs::read_dir(dir.join("extensions")).unwrap().count(), 0);
    }

    #[test]
    fn ensure_pi_agent_dir_is_idempotent() {
        let tmp_root = tempfile::tempdir().unwrap();
        let tmp_home = tempfile::tempdir().unwrap();
        let pi_home = tmp_home.path().join(".pi").join("agent");
        std::fs::create_dir_all(&pi_home).unwrap();
        std::fs::write(pi_home.join("auth.json"), r#"{"token":"test-auth"}"#).unwrap();

        let (dir1, notes1) = ensure_pi_agent_dir(tmp_root.path(), "w-worker", tmp_home.path()).unwrap();
        assert!(notes1.is_empty());

        let (dir2, notes2) = ensure_pi_agent_dir(tmp_root.path(), "w-worker", tmp_home.path()).unwrap();
        assert!(notes2.is_empty());
        assert_eq!(dir1, dir2);

        let auth_link = dir2.join("auth.json");
        assert!(std::fs::symlink_metadata(&auth_link).unwrap().file_type().is_symlink());
        assert_eq!(std::fs::read_link(&auth_link).unwrap(), pi_home.join("auth.json"));
    }

    #[test]
    fn ensure_pi_agent_dir_keeps_and_reports_real_file_where_link_belongs() {
        let tmp_root = tempfile::tempdir().unwrap();
        let tmp_home = tempfile::tempdir().unwrap();
        let pi_home = tmp_home.path().join(".pi").join("agent");
        std::fs::create_dir_all(&pi_home).unwrap();
        std::fs::write(pi_home.join("auth.json"), r#"{"token":"home-auth"}"#).unwrap();
        let npm_dir = pi_home.join("npm");
        std::fs::create_dir_all(&npm_dir).unwrap();

        let agent_dir = tmp_root.path().join(".bee/runtime/pi-agent/w-worker");
        std::fs::create_dir_all(&agent_dir).unwrap();
        let real_auth = agent_dir.join("auth.json");
        std::fs::write(&real_auth, r#"{"token":"local-real-auth"}"#).unwrap();
        let real_npm = agent_dir.join("npm");
        std::fs::create_dir_all(&real_npm).unwrap();

        let (dir, notes) = ensure_pi_agent_dir(tmp_root.path(), "w-worker", tmp_home.path()).unwrap();
        assert_eq!(dir, agent_dir);
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().any(|n| n.contains("auth.json")));
        assert!(notes.iter().any(|n| n.contains("npm")));

        assert!(!std::fs::symlink_metadata(&real_auth).unwrap().file_type().is_symlink());
        let content = std::fs::read_to_string(&real_auth).unwrap();
        assert_eq!(content, r#"{"token":"local-real-auth"}"#);

        assert!(!std::fs::symlink_metadata(&real_npm).unwrap().file_type().is_symlink());
        assert!(real_npm.is_dir());
    }

    #[test]
    fn ensure_pi_agent_dir_refuses_when_auth_missing() {
        let tmp_root = tempfile::tempdir().unwrap();
        let tmp_home = tempfile::tempdir().unwrap();

        let err = ensure_pi_agent_dir(tmp_root.path(), "w-worker", tmp_home.path()).unwrap_err();
        assert!(err.contains("FIX: log in with pi once"), "{err}");
    }
}
