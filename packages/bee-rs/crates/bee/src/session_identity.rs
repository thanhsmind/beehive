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

/// Ordered environment-only session identity lookup.
pub(crate) fn env_session_id() -> Option<String> {
    resolve_env_session_id_from(|key| std::env::var(key).ok())
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
}
