// onboard::templates — every literal constant of
// packages/bee/scripts/onboard_bee.mjs, byte-for-byte.
//
// Provenance: onboard_bee.mjs lines 117–333 (markers, gitignore block,
// HOOK_FILENAMES, DEFAULT_STATE/DEFAULT_CONFIG, the four create-only stubs)
// plus RETIRED_HELPERS (l. 1814), CODEX_STATUS_LINE_BLOCK (l. 2570),
// COMMAND_KEYS (l. 2592), STALE_ADVISOR_KEY_WARNING (l. 2642) and
// HEADER_POINTER_CANDIDATES (l. 2194).
//
// These strings are rendered into host repos and hashed into the managed
// ledger, so a single byte of drift here is a C2 break. The unit tests at the
// bottom pin the shapes the Node script asserts on.

use serde_json::{json, Value};

pub const ONBOARDING_SCHEMA_VERSION: &str = "1.0";
pub const MARKER_START: &str = "<!-- BEE:START -->";
pub const MARKER_END: &str = "<!-- BEE:END -->";
pub const GITIGNORE_MARKER_START: &str = "# BEE:START";
pub const GITIGNORE_MARKER_END: &str = "# BEE:END";

/// onboard_bee.mjs GITIGNORE_BLOCK_PATTERNS (l. 139–223) — order is
/// load-bearing (the block is rendered by `join("\n")` and hashed).
pub const GITIGNORE_BLOCK_PATTERNS: &[&str] = &[
    ".bee/state.json",
    ".bee/reservations.json",
    ".bee/workers/",
    ".bee/logs/",
    ".bee/capture-queue.jsonl",
    ".bee/feedback-digest.json",
    ".bee/.inject-cache.json",
    ".bee/HANDOFF.json",
    ".bee/spikes/",
    ".bee/manifest-hash.json",
    ".bee/sessions/",
    ".bee/claims/",
    ".bee/runtime/",
    ".bee/cache/",
    ".bee/intent/",
    ".bee/locks/",
    ".bee/doctor-attest.json",
    ".bee/native-transport-probe.json",
    ".bee/config.local.json",
    ".bee/tmp/",
    ".bee/backups/",
    "*.[0-9]*-*-*.tmp",
    ".claude-plugin/skills.tmp-*/",
    ".claude-plugin/skills.old-*/",
    ".codex-plugin/skills.tmp-*/",
    ".codex-plugin/skills.old-*/",
    ".bee/tmp_*",
    ".bee/patch-*.json",
];

/// onboard_bee.mjs HOOK_FILENAMES (l. 225–248) — the vendoring order is the
/// order `listPluginHooks()` filters, so it drives managed.repo_hooks key
/// order in the ledger.
pub const HOOK_FILENAMES: &[&str] = &[
    "adapter.mjs",
    "bee-codex-subagent-audit.mjs",
    "bee-session-init.mjs",
    "bee-prompt-context.mjs",
    "bee-write-guard.mjs",
    "tokenize-command.mjs",
    "bee-state-sync.mjs",
    "bee-chain-nudge.mjs",
    "bee-session-close.mjs",
    "bee-model-guard.mjs",
    "bee-tools-logger.mjs",
];

/// onboard_bee.mjs RETIRED_HELPERS (l. 1814–1824).
pub const RETIRED_HELPERS: &[&str] = &[
    "bee_status.mjs",
    "bee_cells.mjs",
    "bee_reservations.mjs",
    "bee_decisions.mjs",
    "bee_state.mjs",
    "bee_backlog.mjs",
    "bee_capture.mjs",
    "bee_reviews.mjs",
    "bee_feedback.mjs",
];

/// onboard_bee.mjs DEFAULT_STATE (l. 250–259). Built as an ordered Value so
/// `JSON.stringify(_, null, 2)` reproduces the literal's key order.
pub fn default_state() -> Value {
    json!({
        "schema_version": "1.0",
        "phase": "idle",
        "feature": Value::Null,
        "mode": Value::Null,
        "approved_gates": {
            "context": false,
            "shape": false,
            "execution": false,
            "review": false
        },
        "workers": [],
        "summary": "",
        "next_action": "Invoke bee-hive."
    })
}

/// onboard_bee.mjs DEFAULT_CONFIG (l. 261–287).
pub fn default_config() -> Value {
    json!({
        "hooks": {
            "session-init": true,
            "prompt-context": true,
            "write-guard": true,
            "state-sync": true,
            "chain-nudge": true,
            "session-close": true
        },
        "lanes": {},
        "capabilities": {},
        "gate_bypass": false,
        "models": {
            "claude": { "extraction": "haiku", "generation": "sonnet" },
            "codex": { "extraction": Value::Null, "generation": Value::Null }
        }
    })
}

/// onboard_bee.mjs runtimeFiles (computePlan step 2, l. 3079–3085): the
/// reservations skeleton.
pub fn default_reservations() -> Value {
    json!({ "reservations": [] })
}

pub const CRITICAL_PATTERNS_STUB: &str = "# Critical Patterns\n\nMandatory pre-planning / pre-execution context for this repository.\nbee-capturing appends hard-won patterns here; keep it short and current.\n\n(none captured yet)\n";

pub const READING_MAP_STUB: &str = "# Reading Map\n\nWhere each area of this project lives. bee-capturing owns this file: it is\nupdated whenever an area spec is created or moved. Read this before any broad\nsearch — it answers \"where does X live\" without a grep.\n\n| Area | Spec | Code entry points |\n|---|---|---|\n| (none mapped yet — run a bee-capturing bootstrap pass) | | |\n";

pub const SYSTEM_OVERVIEW_STUB: &str = "# System Overview\n\nOne-page, technology-agnostic description of what this system does and how its\nareas fit together. bee-capturing owns this file; it is the first read for any\nhuman or agent new to the repository.\n\n(not written yet — run a bee-capturing bootstrap pass to fill this in)\n";

pub const CLAUDE_MD_IMPORT_SECTION: &str = "## bee\n\nThis repo uses bee. The bare import below loads the BEE operating block from\nAGENTS.md at context-load time. Never wrap it in backticks; that disables it.\n\n@AGENTS.md\n";

/// `# Project Rules\n\n${CLAUDE_MD_IMPORT_SECTION}` (l. 331–333).
pub fn claude_md_template() -> String {
    format!("# Project Rules\n\n{CLAUDE_MD_IMPORT_SECTION}")
}

pub const CODEX_TRANSPORT_DIAGNOSTIC: &str = "bee: hook transport unavailable (no git root)";

pub const CODEX_STATUS_LINE_BLOCK: &str = "status_line = [\"current-dir\", \"git-branch\", \"model-with-reasoning\", \"context-remaining\", \"five-hour-limit\", \"weekly-limit\", \"used-tokens\"]\nstatus_line_use_colors = true\n";

/// onboard_bee.mjs COMMAND_KEYS (l. 2592) — its own copy of state.mjs's list.
pub const COMMAND_KEYS: &[&str] = &["setup", "start", "test", "verify"];

pub const STALE_ADVISOR_KEY_WARNING: &str = "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

/// onboard_bee.mjs HEADER_POINTER_CANDIDATES (l. 2194–2198).
pub const HEADER_POINTER_CANDIDATES: &[&str] =
    &["README.md", "docs/specs/system-overview.md", "docs/specs/reading-map.md"];

/// onboard_bee.mjs RENDER_RUNTIMES / RENDER_SCHEMA / RENDER_SIDECAR /
/// SKILLS_VERSION_STAMP (l. 685–719).
pub const RENDER_RUNTIMES: &[&str] = &["claude", "codex"];
pub const RENDER_SCHEMA: &str = "bee-render/2";
pub const RENDER_SIDECAR: &str = ".bee-render.json";
pub const SKILLS_VERSION_STAMP: &str = ".bee-skills-version.json";

/// onboard_bee.mjs REPO_SKILL_TARGETS (l. 393–396): (kind, path segments).
pub const REPO_SKILL_TARGETS: &[(&str, &[&str])] =
    &[("repo-claude", &[".claude", "skills"]), ("repo-agents", &[".agents", "skills"])];

/// onboard_bee.mjs AGENT_TIER_BY_NAME (l. 1933–1937).
pub const AGENT_TIER_BY_NAME: &[(&str, &str)] = &[
    ("bee-gather", "generation"),
    ("bee-extract", "extraction"),
    ("bee-review", "review"),
];

/// onboard_bee.mjs AGENT_TIER_DEFAULTS_CLAUDE (l. 1946) — order matters: it
/// drives `resolved`'s iteration in resolveAgentTierModel.
pub const AGENT_TIER_DEFAULTS_CLAUDE: &[(&str, &str)] =
    &[("extraction", "haiku"), ("generation", "sonnet"), ("review", "opus")];

pub const CODEX_AGENTS_NOTE: &str = "Codex has no per-agent model selection (DEFAULT_MODELS.codex is all-null by design, packages/bee/lib/state.mjs) - tiers are enforced as a read budget + output cap in the worker prompt instead. No agent files are rendered under .agents/ (AO11).";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsjson;

    #[test]
    fn default_state_serializes_like_node() {
        let text = format!("{}\n", jsjson::stringify_pretty(&default_state()));
        assert!(text.starts_with("{\n  \"schema_version\": \"1.0\",\n  \"phase\": \"idle\",\n"));
        assert!(text.ends_with("\"next_action\": \"Invoke bee-hive.\"\n}\n"));
        // Key order is the literal's, not alphabetical.
        let state = default_state();
        let keys: Vec<&str> = state.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(
            keys,
            vec![
                "schema_version",
                "phase",
                "feature",
                "mode",
                "approved_gates",
                "workers",
                "summary",
                "next_action"
            ]
        );
    }

    #[test]
    fn default_config_keeps_literal_order_and_nulls() {
        let v = default_config();
        let keys: Vec<&str> = v.as_object().unwrap().keys().map(|k| k.as_str()).collect();
        assert_eq!(keys, vec!["hooks", "lanes", "capabilities", "gate_bypass", "models"]);
        assert!(v["models"]["codex"]["extraction"].is_null());
    }

    #[test]
    fn stubs_end_with_a_single_newline() {
        for s in [CRITICAL_PATTERNS_STUB, READING_MAP_STUB, SYSTEM_OVERVIEW_STUB, CLAUDE_MD_IMPORT_SECTION] {
            assert!(s.ends_with('\n'));
            assert!(!s.ends_with("\n\n"));
        }
        assert_eq!(claude_md_template(), format!("# Project Rules\n\n{CLAUDE_MD_IMPORT_SECTION}"));
    }
}
