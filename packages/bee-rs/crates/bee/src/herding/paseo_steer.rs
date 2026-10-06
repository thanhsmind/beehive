use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) const STEER_SCRIPT: &str = include_str!("paseo_steer.mjs");

pub(crate) fn steerable(provider: &str) -> bool {
    matches!(provider, "claude" | "codex" | "opencode")
}

pub(crate) fn cli_package_dir(
    command: &str,
    path_lookup: impl Fn(&str) -> Option<String>,
) -> Option<PathBuf> {
    let bin_path = if Path::new(command).is_absolute() || command.contains('/') || (cfg!(windows) && command.contains('\\')) {
        let p = PathBuf::from(command);
        if p.exists() {
            Some(p)
        } else {
            None
        }
    } else {
        let path_val = path_lookup("PATH")?;
        let mut found = None;
        for dir in std::env::split_paths(&path_val) {
            let candidate = dir.join(command);
            if candidate.is_file() || candidate.exists() {
                found = Some(candidate);
                break;
            }
            #[cfg(windows)]
            {
                for ext in [".exe", ".cmd", ".bat"] {
                    let cand_ext = dir.join(format!("{command}{ext}"));
                    if cand_ext.is_file() || cand_ext.exists() {
                        found = Some(cand_ext);
                        break;
                    }
                }
                if found.is_some() {
                    break;
                }
            }
        }
        found
    }?;

    let canonical = std::fs::canonicalize(&bin_path).ok()?;
    let mut current = canonical.parent();
    while let Some(dir) = current {
        let pkg_path = dir.join("package.json");
        if pkg_path.is_file() {
            if let Ok(raw) = std::fs::read_to_string(&pkg_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if val.get("name").and_then(|n| n.as_str()) == Some("@getpaseo/cli") {
                        return Some(dir.to_path_buf());
                    }
                }
            }
        }
        current = dir.parent();
    }
    None
}

pub(crate) fn steer_argv(cli_dir: &str, agent_id: &str, text: &str) -> Vec<String> {
    vec![
        "--input-type=module".to_string(),
        "-e".to_string(),
        STEER_SCRIPT.to_string(),
        cli_dir.to_string(),
        agent_id.to_string(),
        text.to_string(),
    ]
}

pub(crate) fn parse_steer_output(stdout: &str) -> Result<(), String> {
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('{') {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                if let Some(ok) = val.get("ok").and_then(|v| v.as_bool()) {
                    if ok {
                        return Ok(());
                    } else {
                        let err = val
                            .get("error")
                            .and_then(|e| e.as_str())
                            .unwrap_or("unknown error from paseo steer helper");
                        return Err(err.to_string());
                    }
                }
            }
        }
    }
    Err("no JSON response from paseo steer helper".to_string())
}

pub(crate) trait SteerRunner: Send + Sync {
    fn steer(&self, cli_dir: &Path, agent_id: &str, text: &str) -> Result<(), String>;
}

pub(crate) struct RealSteerRunner;

impl SteerRunner for RealSteerRunner {
    fn steer(&self, cli_dir: &Path, agent_id: &str, text: &str) -> Result<(), String> {
        let cli_dir_str = cli_dir.to_str().ok_or_else(|| "invalid utf-8 in cli_dir".to_string())?;
        let args = steer_argv(cli_dir_str, agent_id, text);
        let output = Command::new("node")
            .args(&args)
            .stdin(Stdio::null())
            .output()
            .map_err(|e| format!("failed to spawn node: {e}"))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        match parse_steer_output(&stdout) {
            Ok(()) => Ok(()),
            Err(e) => {
                if !output.status.success() && !stderr.trim().is_empty() && e == "no JSON response from paseo steer helper" {
                    Err(stderr.trim().to_string())
                } else {
                    Err(e)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steerable_set() {
        assert!(steerable("claude"));
        assert!(steerable("codex"));
        assert!(steerable("opencode"));
        assert!(!steerable("pi"));
        assert!(!steerable("unknown"));
        assert!(!steerable(""));
    }

    #[test]
    fn steer_argv_shape() {
        let argv = steer_argv("/path/to/cli", "agent-123", "hello steer");
        assert_eq!(
            argv,
            vec![
                "--input-type=module",
                "-e",
                STEER_SCRIPT,
                "/path/to/cli",
                "agent-123",
                "hello steer"
            ]
        );
    }

    #[test]
    fn cli_package_dir_on_a_temp_tree_with_a_symlinked_bin() {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path();

        let pkg_dir = base.join("node_modules").join("@getpaseo").join("cli");
        let pkg_bin_dir = pkg_dir.join("bin");
        std::fs::create_dir_all(&pkg_bin_dir).unwrap();

        let pkg_json = pkg_dir.join("package.json");
        std::fs::write(&pkg_json, r#"{"name": "@getpaseo/cli", "version": "0.10.3"}"#).unwrap();

        let real_bin = pkg_bin_dir.join("paseo");
        std::fs::write(&real_bin, "#!/bin/sh\necho paseo\n").unwrap();

        let bin_dir = base.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        let symlink_bin = bin_dir.join("paseo");

        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_bin, &symlink_bin).unwrap();
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&real_bin, &symlink_bin).unwrap();

        let bin_dir_str = bin_dir.to_str().unwrap().to_string();
        let resolved = cli_package_dir("paseo", |k| if k == "PATH" { Some(bin_dir_str.clone()) } else { None });
        let expected = std::fs::canonicalize(&pkg_dir).unwrap();
        assert_eq!(resolved, Some(expected));

        let missing = cli_package_dir("missing_tool", |k| if k == "PATH" { Some(bin_dir_str.clone()) } else { None });
        assert_eq!(missing, None);

        let no_path = cli_package_dir("paseo", |_| None);
        assert_eq!(no_path, None);
    }

    #[test]
    fn parse_steer_output_validates_json_lines() {
        assert_eq!(parse_steer_output("{\"ok\":true}\n"), Ok(()));
        assert_eq!(
            parse_steer_output("{\"ok\":false,\"error\":\"daemon error\"}\n"),
            Err("daemon error".to_string())
        );
        assert_eq!(
            parse_steer_output("extra log line\n{\"ok\":true}\n"),
            Ok(())
        );
        assert_eq!(
            parse_steer_output("not json\n"),
            Err("no JSON response from paseo steer helper".to_string())
        );
    }
}
