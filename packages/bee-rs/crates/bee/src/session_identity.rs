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
}
