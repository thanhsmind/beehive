// session_identity.rs — ordered runtime environment lookup for session identity (pihp-1, D6).
//
// Shared order across all session-owned commands:
// 1. BEE_SESSION_ID
// 2. CLAUDE_CODE_SESSION_ID
// 3. PI_SESSION_ID
//
// Explicit command flags always take precedence before this environment lookup.
// Durable single-live-session fallback stays with commands that own durable stores.

fn js_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || c == '\u{feff}')
}

pub(crate) const SESSION_ENV_VARS: [&str; 3] = [
    "BEE_SESSION_ID",
    "CLAUDE_CODE_SESSION_ID",
    "PI_SESSION_ID",
];

pub(crate) fn resolve_env_session_id_from<F>(lookup: F) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    for &var in &SESSION_ENV_VARS {
        if let Some(val) = lookup(var) {
            let trimmed = js_trim(&val);
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
static AMBIENT_PI_SESSION_ID: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();

#[cfg(test)]
fn read_initial_pi_session_id() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(bytes) = std::fs::read("/proc/self/environ") {
            for entry in bytes.split(|&b| b == 0) {
                if let Ok(s) = std::str::from_utf8(entry) {
                    if let Some(val) = s.strip_prefix("PI_SESSION_ID=") {
                        let trimmed = js_trim(val);
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
            return None;
        }
    }
    std::env::var("PI_SESSION_ID")
        .ok()
        .map(|s| js_trim(&s).to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
fn ambient_pi_session_id() -> Option<&'static str> {
    AMBIENT_PI_SESSION_ID
        .get_or_init(read_initial_pi_session_id)
        .as_deref()
}

/// Ordered environment-only session identity lookup.
pub(crate) fn env_session_id() -> Option<String> {
    resolve_env_session_id_from(|key| {
        let val = std::env::var(key).ok()?;
        #[cfg(test)]
        if key == "PI_SESSION_ID" {
            if let Some(ambient) = ambient_pi_session_id() {
                if val == ambient {
                    return None;
                }
            }
        }
        Some(val)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CallerSession {
    pub(crate) id: String,
    pub(crate) runtime: String,
}

pub(crate) fn locate_caller_from<F>(lookup: F) -> Option<CallerSession>
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(val) = lookup("PI_SESSION_ID") {
        let trimmed = js_trim(&val);
        if !trimmed.is_empty() {
            return Some(CallerSession {
                id: trimmed.to_string(),
                runtime: "pi".to_string(),
            });
        }
    }
    if let Some(val) = lookup("CODEX_THREAD_ID") {
        let trimmed = js_trim(&val);
        if !trimmed.is_empty() {
            return Some(CallerSession {
                id: trimmed.to_string(),
                runtime: "codex".to_string(),
            });
        }
    }
    if let Some(val) = lookup("BEE_SESSION_ID") {
        let trimmed_id = js_trim(&val);
        if !trimmed_id.is_empty() {
            if let Some(rt_val) = lookup("BEE_RUNTIME") {
                let trimmed_rt = js_trim(&rt_val);
                if !trimmed_rt.is_empty() {
                    return Some(CallerSession {
                        id: trimmed_id.to_string(),
                        runtime: trimmed_rt.to_string(),
                    });
                }
            }
        }
    }
    if let Some(val) = lookup("CLAUDE_CODE_SESSION_ID") {
        let trimmed = js_trim(&val);
        if !trimmed.is_empty() {
            return Some(CallerSession {
                id: trimmed.to_string(),
                runtime: "claude".to_string(),
            });
        }
    }
    None
}

pub(crate) fn locate_caller() -> Option<CallerSession> {
    locate_caller_from(|key| {
        let val = std::env::var(key).ok()?;
        #[cfg(test)]
        if key == "PI_SESSION_ID" {
            if let Some(ambient) = ambient_pi_session_id() {
                if val == ambient {
                    return None;
                }
            }
        }
        Some(val)
    })
}

pub(crate) fn read_caller_session_cwd(root: &std::path::Path, session_id: &str) -> Option<String> {
    let session = crate::verbs::state_group::store::read_session(root, session_id).ok()??;
    let activity = session.get("activity")?.as_object()?;
    let cwd = activity.get("cwd")?.as_str()?;
    let trimmed = js_trim(cwd);
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[allow(dead_code)]
pub(crate) fn caller_session_cwd(root: &std::path::Path, session_id: &str) -> Option<String> {
    read_caller_session_cwd(root, session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pi_session_identity_resolves_pi_when_others_absent() {
        let sid = resolve_env_session_id_from(|k| match k {
            "PI_SESSION_ID" => Some("pi-session-123".into()),
            _ => None,
        });
        assert_eq!(sid.as_deref(), Some("pi-session-123"));
    }

    #[test]
    fn pi_session_identity_precedence_bee_over_claude_over_pi() {
        let sid = resolve_env_session_id_from(|k| match k {
            "BEE_SESSION_ID" => Some("bee-id".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("claude-id".into()),
            "PI_SESSION_ID" => Some("pi-id".into()),
            _ => None,
        });
        assert_eq!(sid.as_deref(), Some("bee-id"));

        let sid = resolve_env_session_id_from(|k| match k {
            "CLAUDE_CODE_SESSION_ID" => Some("claude-id".into()),
            "PI_SESSION_ID" => Some("pi-id".into()),
            _ => None,
        });
        assert_eq!(sid.as_deref(), Some("claude-id"));
    }

    #[test]
    fn pi_session_identity_empty_or_whitespace_is_skipped() {
        let sid = resolve_env_session_id_from(|k| match k {
            "BEE_SESSION_ID" => Some("   ".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("".into()),
            "PI_SESSION_ID" => Some("  pi-clean  ".into()),
            _ => None,
        });
        assert_eq!(sid.as_deref(), Some("pi-clean"));
    }

    #[test]
    fn pi_session_identity_returns_none_when_all_absent() {
        let sid = resolve_env_session_id_from(|_| None);
        assert!(sid.is_none());
    }

    #[test]
    fn ambient_pi_session_id_is_isolated_or_preserved() {
        let initial = ambient_pi_session_id();
        if let Some(ambient) = initial {
            assert!(!ambient.is_empty());
        }
    }

    #[test]
    fn locator_prefers_innermost_codex_over_inherited_claude() {
        let caller = locate_caller_from(|k| match k {
            "CODEX_THREAD_ID" => Some("codex-id".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("claude-id".into()),
            _ => None,
        });
        assert_eq!(
            caller,
            Some(CallerSession {
                id: "codex-id".into(),
                runtime: "codex".into(),
            })
        );
    }

    #[test]
    fn locator_skips_bare_bee_session_id_without_runtime() {
        // Fall through to Claude
        let caller = locate_caller_from(|k| match k {
            "BEE_SESSION_ID" => Some("bare-bee-id".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("claude-id".into()),
            _ => None,
        });
        assert_eq!(
            caller,
            Some(CallerSession {
                id: "claude-id".into(),
                runtime: "claude".into(),
            })
        );

        // All alone -> None
        let caller_alone = locate_caller_from(|k| match k {
            "BEE_SESSION_ID" => Some("bare-bee-id".into()),
            _ => None,
        });
        assert_eq!(caller_alone, None);

        // Blank runtime -> None / skipped
        let caller_blank_rt = locate_caller_from(|k| match k {
            "BEE_SESSION_ID" => Some("bare-bee-id".into()),
            "BEE_RUNTIME" => Some("   ".into()),
            _ => None,
        });
        assert_eq!(caller_blank_rt, None);
    }

    #[test]
    fn locator_resolves_bee_session_id_with_runtime() {
        let caller = locate_caller_from(|k| match k {
            "BEE_SESSION_ID" => Some("opencode-session".into()),
            "BEE_RUNTIME" => Some("opencode".into()),
            _ => None,
        });
        assert_eq!(
            caller,
            Some(CallerSession {
                id: "opencode-session".into(),
                runtime: "opencode".into(),
            })
        );
    }

    #[test]
    fn locator_returns_none_for_empty_env() {
        let caller = locate_caller_from(|_| None);
        assert_eq!(caller, None);
    }

    #[test]
    fn locator_resolves_pi_first() {
        let caller = locate_caller_from(|k| match k {
            "PI_SESSION_ID" => Some("pi-id".into()),
            "CODEX_THREAD_ID" => Some("codex-id".into()),
            "CLAUDE_CODE_SESSION_ID" => Some("claude-id".into()),
            _ => None,
        });
        assert_eq!(
            caller,
            Some(CallerSession {
                id: "pi-id".into(),
                runtime: "pi".into(),
            })
        );
    }

    #[test]
    fn read_caller_session_cwd_from_fixture_record() {
        let tmp = tempfile::tempdir().unwrap();
        let sessions_dir = tmp.path().join(".bee").join("sessions");
        std::fs::create_dir_all(&sessions_dir).unwrap();

        let record_content = serde_json::json!({
            "id": "sess-test-1",
            "activity": {
                "cwd": "/path/to/worktree"
            }
        });
        std::fs::write(
            sessions_dir.join("sess-test-1.json"),
            serde_json::to_string(&record_content).unwrap(),
        ).unwrap();

        let cwd = read_caller_session_cwd(tmp.path(), "sess-test-1");
        assert_eq!(cwd.as_deref(), Some("/path/to/worktree"));

        // Missing session
        assert_eq!(read_caller_session_cwd(tmp.path(), "sess-nonexistent"), None);

        // Missing activity or cwd
        let incomplete_record = serde_json::json!({
            "id": "sess-test-2",
            "activity": {}
        });
        std::fs::write(
            sessions_dir.join("sess-test-2.json"),
            serde_json::to_string(&incomplete_record).unwrap(),
        ).unwrap();
        assert_eq!(read_caller_session_cwd(tmp.path(), "sess-test-2"), None);
    }
}
