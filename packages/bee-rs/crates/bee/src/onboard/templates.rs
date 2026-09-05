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
    ".bee/bin/bee",
    ".bee/bin/bee.exe",
    ".claude/settings.json.bak",
    ".codex/hooks.json.bak",
    // herding-executor D3/D8: the file mailbox worker-completion contract
    // (job.json, result-N.json, log.txt) — runtime data, never committed.
    // Appended at the end deliberately: the block's order is load-bearing
    // (hashed into the managed ledger), so a new pattern joins at the tail
    // rather than reordering any existing one.
    ".bee/mailbox/",
    // human-mailbox: the letters an unattended run files, and the per-run
    // entry logs they are composed from — runtime data a cap writes on every
    // finish, never committed. Joined at the tail for the same reason as the
    // line above: the block's order is hashed into the managed ledger, so a
    // new pattern must not displace an existing one.
    ".bee/human-mailbox/",
    // pi-result-mailbox D4/D6: the per-orchestrator-session result inbox —
    // one pending marker per DETACHED herding job (`--inbox-session`), claimed
    // and consumed by the Pi drain. Runtime data with a lifetime of one job,
    // never committed; without this line every detached dispatch litters
    // `git status`. Joined at the tail for the same reason as the two lines
    // above: the block's order is hashed into the managed ledger.
    ".bee/result-inbox/",
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
///
/// The full bee workflow structure, seeded with safe defaults. New repos
/// start with the complete role table, workflow settings, and herding
/// skeleton — ready to configure, not a blank slate.
///
/// # Safe defaults
///
/// - `gate_bypass: false` — every gate stops for the human until they opt in.
/// - `ship_visibility: "off"` — no automatic PR drafts.
/// - `worktree_first: "on"` — feature work lives in worktrees.
/// - `uat_stop: "close"` — UAT gate at close, not merge.
/// - No `commands.test` — project-specific, user fills it in.
///
/// # Models
///
/// The full role table ships described, so `bee models show` is self-teaching
/// from the first run. Each `{model, description}` normalizes to `{model}` at
/// resolution time — descriptions are display-only.
///
/// Codex stays all-null by design (`CODEX_AGENTS_NOTE`): codex has no
/// per-agent model selection.
///
/// # Herding
///
/// Ships a working skeleton: `agent_command` names a registry entry, and
/// `agents` defines four common configurations. A repo without herdr/tmux
/// can ignore this block — it only activates when a slot resolves to
/// `{kind: "herding"}`.
pub fn default_config() -> Value {
    json!({
        "hooks": {
            "session-init": true,
            "prompt-context": true,
            "state-sync": true,
            "chain-nudge": true,
            "session-close": true,
            "write-guard": true
        },
        "commands": {},
        "gate_bypass": false,
        "ship_visibility": "off",
        "worktree_first": "on",
        "worktree_cleanup_on_merge": false,
        "uat_stop": "close",
        "uat_before_merge": false,
        "staging_before_merge": false,
        "models": {
            "claude": {
                "code": { "model": "sonnet", "description": "write the cell's code and its tests" },
                "read": { "model": "haiku", "description": "multi-file gathers and codebase scans, read-only" },
                "test": { "model": "sonnet", "description": "author or repair tests, red-first" },
                "docs": { "model": "sonnet", "description": "doc edits and parity sweeps" },
                "plan": { "model": "opus", "description": "planning-shaped work — shaping, drafting cells, plan checks" },
                "extraction": { "model": "haiku", "description": "narrow fact lookups from known locations" },
                "generation": { "model": "sonnet", "description": "fall-through tail: default writer role" },
                "review": { "model": "opus", "description": "independent read-only check of a claim or diff" },
                "advisor": { "model": "fable", "description": "session-class consult for high-risk gates" },
                "supervisor": { "model": "haiku", "description": "cold observer tick — structured observation, decides nothing" },
                "lane-1": { "model": "fable", "description": "blind-lane seat 1 — isolated design proposal" },
                "lane-2": { "model": "opus", "description": "blind-lane seat 2 — isolated design proposal" },
                "lane-3": { "model": "sonnet", "description": "blind-lane seat 3 — isolated design proposal" },
                "hat-facts-gaps": { "model": "opus", "description": "hat: what the spec cannot answer" },
                "hat-risks": { "model": "fable", "description": "hat: what breaks, and can it be undone" },
                "hat-value": { "model": "sonnet", "description": "hat: is this worth its cost" },
                "hat-alternatives": { "model": "opus", "description": "hat: is there a cheaper shape" },
                "hat-user-impact": { "model": "sonnet", "description": "hat: what the user sees and feels" }
            },
            "codex": {
                "code": Value::Null,
                "read": Value::Null,
                "extraction": Value::Null,
                "generation": Value::Null
            }
        },
        "herding": {
            "agent_command": "claude-sonnet",
            "control_command": [
                "claude",
                "-p",
                "{PROMPT}",
                "--model",
                "sonnet",
                "--max-turns",
                "{MAX_TURNS}",
                "--allowedTools",
                "{ALLOWED_TOOLS}"
            ],
            "agents": {
                "claude-sonnet": [
                    "claude",
                    "--model",
                    "sonnet",
                    "--permission-mode",
                    "bypassPermissions"
                ],
                "claude-opus": [
                    "claude",
                    "--model",
                    "opus",
                    "--permission-mode",
                    "bypassPermissions"
                ],
                "claude-fable": [
                    "claude",
                    "--model",
                    "fable",
                    "--permission-mode",
                    "bypassPermissions"
                ],
                "claude-haiku": [
                    "claude",
                    "--model",
                    "haiku",
                    "--permission-mode",
                    "bypassPermissions"
                ]
            }
        }
    })
}

/// onboard_bee.mjs runtimeFiles (computePlan step 2, l. 3079–3085): the
/// reservations skeleton.
pub fn default_reservations() -> Value {
    json!({ "reservations": [] })
}

/// config-sample-herding D3: the bee repo's own `.bee/config-sample.json`,
/// embedded at compile time so the release binary and the sample can never
/// drift apart — the sample is annotated documentation for `.bee/config.json`,
/// including the `herding` block. `bee onboard` seeds this into a fresh host
/// repo create-if-missing (plan.rs runtime-file list, apply.rs's content
/// match arm), so a release user sees the full commented sample without
/// visiting the bee repo. Path is relative to this source file:
/// crates/bee/src/onboard/ up six to the repo root.
pub const CONFIG_SAMPLE_JSON: &str = include_str!("../../../../../../.bee/config-sample.json");

pub const CRITICAL_PATTERNS_STUB: &str = "# Critical Patterns\n\nMandatory pre-planning / pre-execution context for this repository.\nbee-capturing appends hard-won patterns here; keep it short and current.\n\n(none captured yet)\n";

pub const READING_MAP_STUB: &str = "# Reading Map\n\nWhere each area of this project lives. bee-capturing owns this file: it is\nupdated whenever an area spec is created or moved. Read this before any broad\nsearch — it answers \"where does X live\" without a grep.\n\n| Area | Spec | Code entry points |\n|---|---|---|\n| (none mapped yet — run a bee-capturing bootstrap pass) | | |\n";

pub const SYSTEM_OVERVIEW_STUB: &str = "# System Overview\n\nOne-page, technology-agnostic description of what this system does and how its\nareas fit together. bee-capturing owns this file; it is the first read for any\nhuman or agent new to the repository.\n\n(not written yet — run a bee-capturing bootstrap pass to fill this in)\n";

pub const CLAUDE_MD_IMPORT_SECTION: &str = "## bee\n\nThis repo uses bee. The bare import below loads the BEE operating block from\nAGENTS.md at context-load time. Never wrap it in backticks; that disables it.\n\n@AGENTS.md\n";

/// `# Project Rules\n\n${CLAUDE_MD_IMPORT_SECTION}` (l. 331–333).
pub fn claude_md_template() -> String {
    format!("# Project Rules\n\n{CLAUDE_MD_IMPORT_SECTION}")
}

pub const CODEX_TRANSPORT_DIAGNOSTIC: &str = "bee: hook transport unavailable (no git root)";

/// rust-port R6: the binary is the only runtime, so "no binary" is its own
/// visible fail-open arm rather than a silent node fallback.
pub const CODEX_BINARY_MISSING_DIAGNOSTIC: &str = "bee: hook binary missing (.bee/bin/bee)";

pub const CODEX_STATUS_LINE_BLOCK: &str = "status_line = [\"current-dir\", \"git-branch\", \"model-with-reasoning\", \"context-remaining\", \"five-hour-limit\", \"weekly-limit\", \"used-tokens\"]\nstatus_line_use_colors = true\n";

/// onboard_bee.mjs COMMAND_KEYS (l. 2592) — its own copy of state.mjs's list.
pub const COMMAND_KEYS: &[&str] = &["setup", "start", "test"];

pub const STALE_ADVISOR_KEY_WARNING: &str = "advisor mode was removed in 0.1.23; the top-level advisor key in .bee/config.json is ignored — delete it. (This does not affect the models.<runtime>.advisor slot, which is separate and still valid.)";

/// The `commands.verify` retirement (2.1.0). Two shapes, because the damage
/// differs: with a `test` recorded the key is merely dead weight; without one
/// the host just lost every test gate it had.
pub const RETIRED_VERIFY_KEY_WARNING: &str = "commands.verify was retired in 2.1.0; .bee/config.json still has one and it is now ignored — delete it. commands.test is the one declared test command: CI runs it on every push, and a worker reaches for it (or a related subset) as its cap proof; no bee door runs it itself.";

pub const RETIRED_VERIFY_KEY_NO_TEST_WARNING: &str = "commands.verify was retired in 2.1.0 and .bee/config.json declares NO commands.test — this repo currently has no test gate at all. FIX: move your verify command to commands.test (a fast subset is better than nothing), or set commands.test to \"none\" if this repo is deliberately test-free. Note: \"none\" on commands.verify no longer declares a no-test repo.";

/// The ONE name a generated verification skill carries, in every repo,
/// forever (verification-in-the-flow D1). Content differs per project; the
/// name never does, which is what lets every bee surface name the skill in
/// literal text and lets onboard check its existence at one fixed path. It is
/// deliberately NOT `bee-`prefixed: the `bee-*` skill sync prunes any
/// bee-named target directory absent from bee's own source, so a
/// `bee-verify-app` would be deleted from every runtime skill home on the
/// next `--apply`.
pub const VERIFY_APP_SKILL_NAME: &str = "verify-app";

/// The two verification OFFERS and the one upkeep POINTER — the three shapes
/// onboard's verification arm can print (verification-in-the-flow D3).
///
/// All three are written in plain language on purpose: notices are printed
/// verbatim, so an agent that pastes one must not leak a config path, a key
/// name or a bee term to the person reading it. Onboard only points at a
/// skill; it generates nothing, updates nothing and runs nothing.
///
/// The two offers carry different FIRST sentences because they answer
/// different states: a repo with no test command has nothing at all, while a
/// tested repo has plenty — just nothing that drives the product. One wording
/// for both would open with a sentence that is false for half its readers.
/// Both close on the same anti-nag instruction: onboard re-runs on every
/// version mismatch, so an offer already refused must not be made again.
pub const NO_TEST_VERIFICATION_OFFER: &str = "This project has no command that proves it works. The bee-verifying skill can build one together with the user: it studies the project, writes a check that drives the real product, and leaves a single command anyone can run. Offer it — say what it does, then ask, but first look through the project's recorded decisions for an earlier refusal: if the user already said no to this, say nothing. The choice is the user's to accept or refuse: never assume a yes, and build nothing before they answer.";

/// The second offer: this repo tests its code but nothing drives its product.
pub const TESTED_REPO_VERIFICATION_OFFER: &str = "This project's tests check that its code is correct, but nothing starts the real product and looks at what a user would see. The bee-verifying skill can build that second check together with the user: it studies the project, writes a check that drives the product itself, and leaves a single command anyone can run. The tests stay exactly as they are — this is a separate command beside them. Offer it — say what it does, then ask, but first look through the project's recorded decisions for an earlier refusal: if the user already said no to this, say nothing. The choice is the user's to accept or refuse: never assume a yes, and build nothing before they answer.";

/// The pointer, for a repo that already HAS the verification skill. A pointer,
/// never a prompt: it names the skill that maintains the check and stops.
/// There is nothing here for the user to accept — the skill exists already.
pub const VERIFICATION_UPKEEP_POINTER: &str = "This project already carries its own verification skill — the one that drives the real product. The bee-verify-upkeep skill keeps it true: it re-reads what the product does now, refreshes the checks that have drifted behind it, and reports whatever it cannot repair on its own.";

/// onboard_bee.mjs HEADER_POINTER_CANDIDATES (l. 2194–2198).
pub const HEADER_POINTER_CANDIDATES: &[&str] =
    &["README.md", "docs/specs/system-overview.md", "docs/specs/reading-map.md"];

/// onboard_bee.mjs RENDER_RUNTIMES / RENDER_SCHEMA / RENDER_SIDECAR /
/// SKILLS_VERSION_STAMP (l. 685–719). `opencode` joined this list under D1
/// (opencode-support oc-4): the marker grammar (`render.rs::classify_marker_line`)
/// accepts it as a valid `<!-- bee:only opencode -->` label because the
/// ONBOARDING SYNC PATH now renders a real opencode target
/// (`.opencode/skills/`, via the already-runtime-agnostic
/// `apply_sync_skill`/`render_skill_bytes`).
pub const RENDER_RUNTIMES: &[&str] = &["claude", "codex", "opencode"];
pub const RENDER_SCHEMA: &str = "bee-render/2";
pub const RENDER_SIDECAR: &str = ".bee-render.json";
pub const SKILLS_VERSION_STAMP: &str = ".bee-skills-version.json";

/// onboard_bee.mjs REPO_SKILL_TARGETS (l. 393–396): (kind, path segments).
/// `repo-opencode` joined this list in opencode-support oc-13 (S5): `bee
/// onboard --apply` now drives the same runtime-agnostic sync writer
/// against `.opencode/skills/` that the claude/agents targets already use —
/// the same idempotent "copy when missing or drifted" behavior, no separate
/// code path.
pub const REPO_SKILL_TARGETS: &[(&str, &[&str])] = &[
    ("repo-claude", &[".claude", "skills"]),
    ("repo-agents", &[".agents", "skills"]),
    ("repo-opencode", &[".opencode", "skills"]),
];

/// onboard_bee.mjs AGENT_TIER_BY_NAME (l. 1933–1937), rebased onto ROLE by
/// model-role-split D2/D3 (store `06e49368`, `3c9d6262`).
///
/// An agent no longer names a COST TIER that a private table turns into a
/// model. It declares the ORDERED ROLE LIST it serves, best first, and
/// `onboard::agents` walks that list through the one shared resolver
/// (`verbs::drivers::resolve_role`) — the same resolver `bee dispatch
/// prepare` and the model guard read. A host that configures a role in
/// `models.<runtime>` therefore sees that role in the rendered agent file,
/// with no second parser to keep in step.
///
/// The NAMES here are today's names on purpose. Which role names bee
/// publishes as shipped config defaults is decision D3 (store `3c9d6262`)
/// and belongs to the cell that gives a cell its `role` field; this table
/// only changes the MECHANISM, so it introduces no name bee did not already
/// resolve. When D3 lands its published names, it prepends them to these
/// lists and fall-through keeps every existing host rendering exactly what
/// it renders today.
///
/// The lists are also where `resolveAgentTierModel`'s one hard-coded
/// special case went: `bee-review` used to fall back to the generation model
/// through an `if tier == "review"` branch, which is plain fall-through
/// spelled by hand. `bee-extract` deliberately does NOT fall through — a
/// null extraction slot removes the file today and must keep removing it.
pub const AGENT_ROLES_BY_NAME: &[(&str, &[&str])] = &[
    // bee-gather asks for the READ job first and only falls through to
    // generation, which bee-build owns outright: gather reads, build writes.
    // The list is the same one `drivers::tier_role_list("read")` walks
    // (gather-reads-the-read-slot D1/D4) — read, then generation, never
    // extraction — so the opencode render pin, the sync record and the
    // status drift check all reach it through the one shared resolver. The
    // role decides the model; what the agent may DO is the agent file's own
    // contract.
    ("bee-build", &["generation"]),
    ("bee-gather", &["read", "generation"]),
    ("bee-extract", &["extraction"]),
    ("bee-review", &["review", "generation"]),
];

/// onboard_bee.mjs AGENT_TIER_DEFAULTS_CLAUDE (l. 1946) — bee's own baked-in
/// model per role for the claude agent files. It is the SEED the host's
/// `models.claude` overlays, never a resolver: `onboard::agents` hands the
/// seeded map to `verbs::drivers::resolve_role` and reads the answer.
///
/// The `TIER` in the name is the retiring cost word and outlives this cell on
/// purpose — the identifier sweep is the `tier`-retirement slice's, and
/// `verbs::status_full::store` reads the opencode twin below by this name.
pub const AGENT_TIER_DEFAULTS_CLAUDE: &[(&str, &str)] =
    &[("extraction", "haiku"), ("generation", "sonnet"), ("review", "opus")];

pub const CODEX_AGENTS_NOTE: &str = "Codex has no per-agent model selection (DEFAULT_MODELS.codex is all-null by design) - tiers are enforced as a read budget + output cap in the worker prompt instead. No agent files are rendered under .agents/ (AO11).";

/// opencode-support oc-14: OpenCode's own per-tier model defaults, mirroring
/// AGENT_TIER_DEFAULTS_CLAUDE's role but for the free, zero-config
/// `opencode/*` provider (the only live provider verified on this machine —
/// opencode-support oc-11/discovery.md). `models.opencode.<slot>` in
/// `.bee/config.json` overrides a slot exactly like `models.claude.<slot>`
/// does; unconfigured stands on these baked-in names rather than the
/// model-guard dispatch default of Null, because these agent files pin a
/// real model regardless (structural enforcement, plan.md's model-guard
/// fallback row).
///
/// model-role-split D2: this stays a SEED, not a second resolver. It is why
/// `onboard::agents` seeds the map it hands `resolve_role` instead of letting
/// the resolver fall to `drivers::default_models`, whose opencode entries are
/// all null — an opencode agent file with no `model:` line is not a file.
pub const AGENT_TIER_DEFAULTS_OPENCODE: &[(&str, &str)] = &[
    ("extraction", "opencode/ling-3.0-tiny-free"),
    ("generation", "opencode/big-pickle"),
    ("review", "opencode/nemotron-3-ultra-free"),
];

/// opencode-support oc-14: the per-agent OpenCode `permission:` deny list —
/// a capability profile keyed by AGENT NAME, not by tier (unlike the model).
/// Only `bee-build` may edit; only `bee-build` and `bee-review` may run
/// `bash`. Every agent denies `task`/`todowrite`/`webfetch`/`websearch`/`lsp`
/// (mirrors oc-11's hand-authored `.opencode/agent/bee-*.md` baseline,
/// verified live against opencode 1.18.16's `opencode agent list`).
pub const AGENT_OPENCODE_PERMISSION_DENY: &[(&str, &[&str])] = &[
    ("bee-build", &["task", "todowrite", "webfetch", "websearch", "lsp"]),
    ("bee-gather", &["edit", "bash", "task", "todowrite", "webfetch", "websearch", "lsp"]),
    ("bee-extract", &["edit", "bash", "task", "todowrite", "webfetch", "websearch", "lsp"]),
    ("bee-review", &["edit", "task", "todowrite", "webfetch", "websearch", "lsp"]),
];

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
        // The full workflow structure: hooks, commands, workflow settings, models, herding.
        assert_eq!(
            keys,
            vec![
                "hooks",
                "commands",
                "gate_bypass",
                "ship_visibility",
                "worktree_first",
                "worktree_cleanup_on_merge",
                "uat_stop",
                "uat_before_merge",
                "staging_before_merge",
                "models",
                "herding"
            ]
        );
        assert!(v["models"]["codex"]["extraction"].is_null());
        // Herding skeleton is present.
        assert!(v["herding"]["agent_command"].as_str().is_some());
        assert!(v["herding"]["agents"].as_object().is_some());
    }

    /// The full role table is now shipped — all roles bee asks for, plus the
    /// lanes and hats. Each claude role carries a description.
    #[test]
    fn default_config_publishes_the_full_role_table() {
        let v = default_config();

        // Claude ships the full table.
        let claude_table = v["models"]["claude"].as_object().unwrap();
        let claude_names: Vec<&str> = claude_table.keys().map(|k| k.as_str()).collect();
        assert_eq!(
            claude_names,
            vec![
                "code",
                "read",
                "test",
                "docs",
                "plan",
                "extraction",
                "generation",
                "review",
                "advisor",
                "supervisor",
                "lane-1",
                "lane-2",
                "lane-3",
                "hat-facts-gaps",
                "hat-risks",
                "hat-value",
                "hat-alternatives",
                "hat-user-impact"
            ],
            "claude"
        );

        // Codex stays minimal — all null by design (CODEX_AGENTS_NOTE).
        let codex_table = v["models"]["codex"].as_object().unwrap();
        let codex_names: Vec<&str> = codex_table.keys().map(|k| k.as_str()).collect();
        assert_eq!(codex_names, vec!["code", "read", "extraction", "generation"], "codex");
        for name in ["code", "read", "extraction", "generation"] {
            assert!(v["models"]["codex"][name].is_null(), "codex.{name} must stay null");
        }
    }

    /// Every claude role ships with a description so `bee models show` is
    /// self-teaching from the first run.
    #[test]
    fn every_claude_role_has_a_description() {
        let v = default_config();
        let claude_table = v["models"]["claude"].as_object().unwrap();

        for (name, slot) in claude_table {
            let obj = slot
                .as_object()
                .unwrap_or_else(|| panic!("claude.{name} must be a {{model, description}} object"));
            assert!(
                obj.get("model").and_then(Value::as_str).is_some_and(|m| !m.is_empty()),
                "claude.{name} lost its model"
            );
            assert!(
                obj.get("description").and_then(Value::as_str).is_some_and(|d| !d.is_empty()),
                "claude.{name} ships no description"
            );
        }
    }

    #[test]
    fn stubs_end_with_a_single_newline() {
        for s in [CRITICAL_PATTERNS_STUB, READING_MAP_STUB, SYSTEM_OVERVIEW_STUB, CLAUDE_MD_IMPORT_SECTION] {
            assert!(s.ends_with('\n'));
            assert!(!s.ends_with("\n\n"));
        }
        assert_eq!(claude_md_template(), format!("# Project Rules\n\n{CLAUDE_MD_IMPORT_SECTION}"));
    }

    #[test]
    fn config_sample_json_embed_parses_and_carries_the_herding_key() {
        // A moved or renamed source file must break the build loudly (the
        // include_str! path fails to compile), not silently ship an empty
        // or stale sample. This test guards the second failure mode: the
        // path still resolves, but to content that no longer parses or has
        // drifted behind the herding-key addition (config-sample-herding D1).
        let v: Value = serde_json::from_str(CONFIG_SAMPLE_JSON)
            .expect("embedded .bee/config-sample.json must parse as JSON");
        assert!(v.get("herding").is_some(), "embedded config-sample.json is missing the herding key");
    }
}
