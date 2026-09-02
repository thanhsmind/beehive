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
/// `lanes: {}` and `capabilities: {}` were dropped here (2026-08-02). Every
/// new repo was seeded with both and `config-sample.json` documented them as
/// "reserved … leave {} unless a bee release documents a key" — but NOTHING
/// has ever read either one. A config key with no reader is not a reserved
/// extension point, it is a promise the code does not keep: it invites a host
/// to configure something that cannot take effect. Removing them is
/// behaviour-neutral for the same reason it is safe — an existing config that
/// still carries them keeps working, because bee ignores unknown top-level
/// keys.
///
/// # What `models` ships, and why exactly these names
///
/// model-role-split D3 (store `3c9d6262`), the PUBLISHING half. bee ships a
/// config default only for a role name bee's own dispatch sites ask for, and
/// a published default is a name bee CONSUMES — never a suggestion nobody
/// reads. Asking and publishing are different acts (store `561e1bda`): a name
/// bee asks for still resolves by fall-through without being written here, so
/// most asked-for names do not belong in a fresh file. The test each
/// candidate had to pass is the sharpest one this codebase can state:
///
/// > Ship a default only for a name a host's own `models.<runtime>` must
/// > carry for bee's own dispatch door to accept it.
///
/// `verbs::drivers::guard::known_roles` is that door — the keys of
/// `models.<runtime>` that DECLARE a role (`role_is_declarable`: every key
/// there, except an `advisor` whose own floor-less resolver answers nothing),
/// union `ceiling` — and `bee dispatch prepare` REFUSES
/// (`role_not_configured`) any role outside it. The union over every
/// `slot_for_kind` answer that used to sit in that derivation is gone
/// (`c2ef2f9f`), and with it the reachability it lent `advisor`. Note what
/// `models.<runtime>`'s keys already include after `normalize_models`:
/// everything `drivers::default_models` seeds for that runtime, which is why
/// three of the six names below are reachable with no config key at all. The
/// six names bee asks for, run through it:
///
/// | role | who asks for it | reachable with no config key? | shipped |
/// |---|---|---|---|
/// | `code` | the execution default in every cell dispatch's list; D9 backfills 504 of 506 cells onto it | **no** — `drivers::default_models` has no entry on any runtime | **added** |
/// | `read` | the head of the read dispatch's list; D9's `role` for the extraction cells | **no** — same | **added** |
/// | `review` | `slot_for_kind("reviewer")`, `bee-review`'s role list | yes — `default_models` seeds it | no |
/// | `advisor` | `slot_for_kind("advisor")`, `resolve_advisor` | **no** — nothing seeds it, and a null-valued key is OFF rather than a configuration | no |
/// | `generation` | the tail every ordered list ends with; `slot_for_kind("cell")` today | yes — `default_models` seeds it | kept |
/// | `extraction` | `bee-extract`'s sole role; the read list's middle entry | yes — `default_models` seeds it | kept |
///
/// Two names are added; none is removed. Dropping `extraction` or
/// `generation` was considered and refused: both are what the historical role
/// lists END on (`cell_role_list`, `tier_role_list`), and a file that
/// publishes the job names without the tail they fall through to teaches half
/// the resolution. Neither would become UNREACHABLE by being dropped —
/// `default_models` seeds both on every runtime — which corrects an earlier
/// version of this note rather than changing the call.
///
/// `review` and `advisor` are asked for and deliberately NOT shipped:
/// - `review` already resolves without a key, and the documented unset-review
///   -> generation fall-through is a deliberate cost posture. Writing
///   `"review": "opus"` here would silently move every new host's reviews onto
///   the expensive model — a product call, not a publishing one.
/// - `advisor` has NO fall-through (decision `4faf1de9`): unconfigured means
///   "no advisor", and since `role_is_declarable` an explicit `null` means the
///   same thing at every door rather than only at some of them. A value would
///   switch the advisor on for every new host; a `null` would publish an
///   off-switch for something that is already off. Neither is shippable, so
///   the key stays out.
///
/// The two added values are today's models on purpose, so nothing moves:
/// `code` takes what cell execution runs on now (`generation` -> sonnet) and
/// `read` takes what a read runs on now (`extraction` -> haiku). `models.codex`
/// stays all-null by design (`CODEX_AGENTS_NOTE`: codex has no per-agent model
/// selection). And this function seeds a NEW `.bee/config.json` only —
/// `apply.rs`'s `create_runtime_file` arm is create-if-missing — so no existing
/// host's config changes meaning.
///
/// models-show-verb D3: each seeded claude role carries a `description`, so a
/// fresh install ships the SELF-TEACHING table rather than four bare model
/// names an agent has to guess the meaning of. The shape moves from a bare
/// string to `{model, description}` — a documented leaf that
/// `normalize_tier_value` already accepted for years, and one it normalizes to
/// `{model}` by dropping the description, so resolution answers exactly the
/// models the bare strings answered. `bee models show` is what reads the
/// descriptions back (the raw table, description intact); the normalized view
/// the dispatcher resolves against never sees them, which is what keeps
/// resolution blind. Codex stays all-null: a description on a slot that
/// selects no model would document a lever codex does not have.
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
        "gate_bypass": false,
        // Job names first — the two a host actually edits — then the
        // historical tail every ordered role list ends with. Each claude slot
        // is `{model, description}` (D3): the description is the ONE place a
        // role's meaning is written down, and `bee models show` is how it is
        // read back.
        "models": {
            "claude": {
                "code": { "model": "sonnet", "description": "write the cell's code and its tests" },
                "read": { "model": "haiku", "description": "multi-file gathers and scans, read-only" },
                "extraction": { "model": "haiku", "description": "narrow fact lookups from known locations" },
                "generation": { "model": "sonnet", "description": "fall-through tail, the default writer role" }
            },
            "codex": {
                "code": Value::Null,
                "read": Value::Null,
                "extraction": Value::Null,
                "generation": Value::Null
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
        // Every key here must have a reader. `lanes`/`capabilities` were
        // seeded for years with none — see default_config's note.
        assert_eq!(keys, vec!["hooks", "gate_bypass", "models"]);
        assert!(v["models"]["codex"]["extraction"].is_null());
    }

    /// model-role-split D3 (store `3c9d6262`, `561e1bda`): the published set
    /// is exactly the role names a host's own `models.<runtime>` must carry
    /// for `known_roles` to accept them, plus the historical tail. See
    /// `default_config`'s table for the per-name reasoning.
    #[test]
    fn default_config_publishes_only_the_roles_bee_asks_for() {
        let v = default_config();
        for runtime in ["claude", "codex"] {
            let table = v["models"][runtime].as_object().unwrap();
            let names: Vec<&str> = table.keys().map(|k| k.as_str()).collect();
            // The job names lead; the historical tail follows and is never
            // dropped (`561e1bda`: a list that ends before it would skip an
            // existing host's configured model).
            assert_eq!(names, vec!["code", "read", "extraction", "generation"], "{runtime}");
            // Asked for, deliberately unshipped: both already resolve without
            // a key, and writing either one would decide something that is
            // not this function's to decide.
            assert!(!table.contains_key("review"), "{runtime} must not ship a review default");
            assert!(!table.contains_key("advisor"), "{runtime} must not ship an advisor default");
        }
        // Nothing moves for a fresh host: the added job names carry the very
        // models the tail already resolved to. Read through `model`, because
        // models-show-verb D3 made each claude slot a `{model, description}`
        // object — the descriptions differ on purpose, the models must not.
        assert_eq!(
            v["models"]["claude"]["code"]["model"],
            v["models"]["claude"]["generation"]["model"]
        );
        assert_eq!(
            v["models"]["claude"]["read"]["model"],
            v["models"]["claude"]["extraction"]["model"]
        );
        // codex stays all-null by design (CODEX_AGENTS_NOTE).
        for name in ["code", "read", "extraction", "generation"] {
            assert!(v["models"]["codex"][name].is_null(), "codex.{name} must stay null");
        }
    }

    /// models-show-verb D3 (CONTEXT.md): a fresh install ships the role table
    /// already explained. Two halves, and BOTH have to hold at once — a
    /// description that cost the host its models would be a worse trade than
    /// no description at all.
    #[test]
    fn a_fresh_seed_explains_every_claude_role_and_still_resolves_to_the_same_models() {
        let v = default_config();

        // Half one: every claude role bee publishes carries a non-empty
        // description, and it is a `{model, description}` object rather than
        // a bare string.
        for name in ["code", "read", "extraction", "generation"] {
            let slot = v["models"]["claude"][name]
                .as_object()
                .unwrap_or_else(|| panic!("claude.{name} must be a {{model, description}} object"));
            assert!(
                slot.get("model").and_then(Value::as_str).is_some_and(|m| !m.is_empty()),
                "claude.{name} lost its model"
            );
            assert!(
                slot.get("description").and_then(Value::as_str).is_some_and(|d| !d.is_empty()),
                "claude.{name} ships no description — the fresh host cannot read what the role means"
            );
        }
        // Codex stays null: no per-agent model selection, so nothing to
        // describe (CODEX_AGENTS_NOTE, and D3 says so in as many words).
        for name in ["code", "read", "extraction", "generation"] {
            assert!(v["models"]["codex"][name].is_null(), "codex.{name} must stay null");
        }

        // Half two: resolution is BLIND to the change. Asked at the door a
        // dispatch actually uses, every seeded role must resolve to exactly
        // the model the bare-string seed resolved to. (The normalized MAPS
        // differ in leaf shape — `{model}` where a string stood — which is
        // why the comparison is the resolver's answer, not the map.)
        let described = crate::verbs::drivers::normalize_models(Some(&v["models"]));
        let bare = crate::verbs::drivers::normalize_models(Some(&json!({
            "claude": {
                "code": "sonnet",
                "read": "haiku",
                "extraction": "haiku",
                "generation": "sonnet"
            },
            "codex": {
                "code": Value::Null,
                "read": Value::Null,
                "extraction": Value::Null,
                "generation": Value::Null
            }
        })));
        for name in ["code", "read", "extraction", "generation"] {
            let now = crate::verbs::drivers::resolve_role(&described, &[name], "claude", "cell");
            let before = crate::verbs::drivers::resolve_role(&bare, &[name], "claude", "cell");
            assert_eq!(
                now, before,
                "the seeded description changed what role {name} resolves to"
            );
        }
        // And the description itself never reaches the resolved answer.
        assert_eq!(
            crate::verbs::drivers::resolve_role(&described, &["code"], "claude", "cell"),
            crate::verbs::drivers::Resolved::Model { model: "sonnet".into(), effort: None }
        );
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
