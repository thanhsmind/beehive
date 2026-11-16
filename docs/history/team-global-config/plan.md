---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: Team config in its own file, and a user-global config layer

## Summary

The owner can keep the team table (which model runs which job) in its own
file, `.bee/team.json`, apart from the rest of the project config. The
owner also gets one config file, `~/.config/beehive/config.json`, that
every project reads and only waggledance writes, so a value wanted everywhere is set once. A
project's own files still win over the global file. The global file can
never turn off a guard, set gate bypass, or switch hooks off. Old projects
keep working with no change; `bee onboard` offers a one-command move, and
`bee config explain` says which file a value came from.

Mode: `high-risk` — 3 flags: audit-security (the write guard and its governed keys), public-contracts (a new file, a new verb, new flags), multi-domain (config reader, hooks, guard, verbs, onboarding).
Why this is the least workflow that protects the work: a new config file is a new door past the guard, and every hook reads config, so the reader and the guard each need their own cell and proof.

## Requirements (from CONTEXT.md)

- D1: `.bee/team.json` holds the team table at the same key paths.
- D2: precedence default < global < config.json < team.json < config.local.json.
- D3: global path `$BEEHIVE_CONFIG_DIR`, `$XDG_CONFIG_HOME/beehive`, `~/.config/beehive`.
- D4: global may not set `guards`, `gate_bypass`, `hooks`.
- D5: one ordered layer reader for every merged read.
- D6: guard covers team.json; agent writes to the global file refused.
- D7: `bee config` `--global`, layer-aware `explain` and `validate`.
- D8: `bee config split-team`.
- D9: onboarding seeds team.json and offers split-team.
- D10: beehive runs on team.json; docs and knowledge describe the layers.
- C1, C2, C3: the contracts and their test-name markers.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The strict merged reader is one function in state.rs | read | packages/bee-rs/crates/bee/src/state.rs:161 | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 2 | A fail-open copy of the reader lives in compaction | read | packages/bee-rs/crates/bee/src/hooks/compaction.rs:124 | `pub(crate) fn read_config_failopen(root: &Path) -> Map<String, Value> {` |
| 3 | A second fail-open copy lives in the session preamble | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:50 | `pub(crate) fn read_config_raw_open(root: &Path) -> JMap {` |
| 4 | The overlay merge has a shared home in state.rs and a byte-copy in the preamble | read | packages/bee-rs/crates/bee/src/state.rs:134 | `pub(crate) fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {` |
| 4b | The preamble keeps its own copy of the merge | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs:24 | `pub(crate) fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {` |
| 4c | Herding reads its agent table from config.json alone | read | packages/bee-rs/crates/bee/src/herding/wave.rs:803 | `let cfg_path = main_root.join(".bee").join("config.json");` |
| 4d | The paseo job verb reads config.json alone | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:890 | `let config_path = main_root.join(".bee").join("config.json");` |
| 4e | Worktree bootstrap copies config.json only | read | packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs:974 | `let config = copy_if_absent("config.json")?;` |
| 4f | Onboarding seeds every role slot, so a project always shadows a global team value | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:199 | `"team": {` |
| 5 | The models-to-team fold is one function | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:303 | `pub(crate) fn fold_team_key(map: &mut Map<String, Value>) -> bool {` |
| 6 | The guard path gate names config.json by literal | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:93 | `normalized == ".bee/config.json"` |
| 7 | The guard compares merged text of two files only | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:1160 | `pub(crate) fn config_merged_text(tracked: Option<&str>, overlay: Option<&str>) -> Option<String> {` |
| 8 | The write guard turns off when its hook is disabled in config | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:226 | `if !hook_enabled(&store_root_pb, HOOK_NAME) {` |
| 9 | The config verb hard-codes two file paths | read | packages/bee-rs/crates/bee/src/verbs/config.rs:14 | `const TRACKED: &str = ".bee/config.json";` |
| 10 | explain has its own function | read | packages/bee-rs/crates/bee/src/verbs/config.rs:221 | `fn run_explain(ctx: &GCtx, key: &str) -> Result<(Value, String), String> {` |
| 11 | validate has its own function | read | packages/bee-rs/crates/bee/src/verbs/config.rs:259 | `fn run_validate(ctx: &GCtx) -> (Value, String) {` |
| 12 | The governed-change check bee config set uses is shared with the guard | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:1431 | `pub(crate) fn config_governed_change_deny(` |
| 13 | ~/.config/beehive already exists as bee's user dir | read | packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs:38 | `pub(crate) fn global_perf_dir() -> PathBuf {` |
| 14 | Onboarding seeds the team table inside config.json | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:199 | `"team": {` |
| 15 | The detector reads a herding value from config.json directly | read | packages/bee-rs/crates/bee/src/verbs/detector.rs:608 | `match read_json(&control.join(".bee").join("config.json")) {` |
| 16 | The preamble's legacy-models warning reads config.json directly | read | packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:786 | `let raw_config = read_json_object(&root.join(".bee").join("config.json")).unwrap_or_default();` |
| 17 | The cross-project guard reads guards from config.json directly | read | packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs:116 | `let config = read_json(&store_root.join(".bee").join("config.json"));` |
| 18 | The config verbs are registered as config.<verb> in the registry | read | packages/bee-rs/crates/bee/src/generated/registry_payload.json:1 | `"name":"config.validate","invoke":"bee config validate"` |
| 19 | The team table is more than half of beehive's config.json | ran | `jq '.team\|tostring\|length' .bee/config.json` | `8648` of `15382` bytes |

## Discovery

A read-only gather mapped every config reader (report:
`.bee/mailbox/job-1794822276497-3431058-1/report-1.md` in the main
checkout). Every merged read goes through `read_config_raw` or one of two
fail-open copies (claims 1-3); three places read `config.json` directly for
values (claims 15-17), and the hat wave found more: the herding, paseo, report and status readers (claims 4c-4e; the full list is in tgc-2's action). Claim 17 stays as it is: it reads only `guards.*`,
which D4 keeps out of the global layer and D1 keeps out of team.json, so a
layer reader would give it the same value. No code reads a user-global
settings file today (claim 13 is a log dir), so D3 is the first one.

## Approach

Recommended path: seven serial cells. tgc-1 builds the one ordered layer
reader and moves the three merged readers onto it (D2-D5, D13, C1). tgc-2
moves every direct config value read (herding, paseo, report, status,
worktree bootstrap) onto it. tgc-3 extends the write guard to team.json and
the global file, with the global layer in its comparison (D6, D12, D13,
C2). tgc-4 adds the read-only global layer to `bee config`, the
layer-aware explain and validate, and split-team with `--drop-global` (D7,
D8, D11, C3). tgc-5 changes the onboarding seed and adds the split-team
notice (D9, D11). tgc-6 is an independent test cell: a tester that did not
write the code drives the built binary in a sandbox. tgc-7 writes the
docs and the delivery. After the merge the leader installs the new binary,
then runs `bee config split-team` on beehive and checks `bee team show`
is unchanged (D10, D13). The waggledance side of D12 (its writer for the
global file) goes to the waggledance leader as a separate request.

Architect skipped: the data shape is the existing merge object, one more
file at the same key paths and one more file below it; the only new type
is the ordered layer list, and C1 pins its order in one place.

Rejected: a separate `team` key format in team.json (a second shape to
merge); global wins over project (owner chose 2A); a global file that only
holds team and herding (owner chose 3A); moving the table automatically at
onboard (D9: the owner's choice); letting agents write the global file
(D6).

Waves: serial, tgc-1 → tgc-2 → tgc-3 → tgc-4 → tgc-5 → tgc-6 → tgc-7. Reason: every cell builds the one `bee` crate in one worktree, so a sibling's half-written edit breaks another cell's build (a single scarce resource), and tgc-2 to tgc-5 call the reader tgc-1 adds (a real dependency). After the merge the leader reinstalls `.bee/bin/bee` BEFORE the beehive split, so no binary that reads two files ever runs on a split config (D13).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"tgc-1 to tgc-5, tgc-8 and tgc-10 write Rust and their red-first tests."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"tgc-6, tgc-9 and tgc-11: a tester that shares no context with the implementers drives the built binary, re-proves each cell contract and adds tests."},
    {"stage":"docs","classification":"required","role":"docs","reason":"tgc-7 syncs config-reference, knowledge and delivery."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"High-risk: all five seats run the plan check."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"High-risk seat."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"High-risk seat; new file, verb and notice are user-facing."},
    {"stage":"hat-risks","classification":"required","role":"hat-risks","reason":"High-risk seat; the guard change."},
    {"stage":"hat-value","classification":"required","role":"hat-value","reason":"High-risk seat."},
    {"stage":"slice-judge","classification":"required","role":"review","reason":"One read-only checklist judge over tgc-1 to tgc-5. Independent review stays user-invoked."},
    {"stage":"writing","classification":"not-applicable","role":"writer","reason":"tgc-7 (docs role) writes the knowledge sync and delivery."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release runs through scripts/release.sh only when the owner asks."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The config map gather already ran."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, locked by the owner."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Risk map

- A hook reads a different config after the reader change — HIGH — the one fact: with no team.json and no global file, the merged object must be byte-identical to today's for every reader — Lands in: tgc-1 — proof: a `config_layers` test that compares the new reader with today's output on a fixture holding config.json and config.local.json only, and the full suite.
- team.json becomes a way past the governed-key guard — HIGH — the one fact: the path gate and the merged comparison include team.json — Lands in: tgc-3 — proof: `team_guard` tests that refuse a slot change written to team.json by Edit, Write and Bash.
- The global file turns off a guard in every project — HIGH — the one fact: `guards`, `gate_bypass`, `hooks` are dropped from the global layer at read time — Lands in: tgc-1 — proof: a `config_layers` test with each key in the global file and `hook_enabled` still true.
- split-team loses or changes a slot — MEDIUM — the one fact: the merged view before and after is equal, checked inside the verb — Lands in: tgc-4 — proof: a `split_team` test on a copy of beehive's real config.json.
- Herding reads its agent table from config.json alone and loses it after a split — HIGH — the one fact: wave.rs:803, job_verbs.rs:890 and herding.rs:1461 read config.json directly (claims 4c, 4d) — Lands in: tgc-2 — proof: `config_layers_reader` tests with the table only in team.json, and tgc-6's wave resolution check after split.
- A global team value never applies because every project seeds a full table — HIGH — the one fact: the seed fills every slot (claim 4f) and the project wins — Lands in: tgc-4, tgc-5 — proof: `split_team` --drop-global test and tgc-6's onboard drive with a global team.
- An agent shadows a global slot or points bee at its own global dir — HIGH — the one fact: the guard lets an ADD pass and env is per process — Lands in: tgc-3 — proof: `team_guard` tests for the shadowing add and the env assignment.
- The main checkout runs an old binary on a split config — HIGH — the one fact: `.bee/bin/bee` is machine-local and reads two files — Lands in: leader step after merge — proof: reinstall, check a new string in the installed binary, then split; undo is git revert of the split commit.

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature/feature.md` (cited, not copied). Step 3 (statechart) does not apply: no event-order flow. Step 8 (formal verification) skipped: the host has no `create-formal-verification-skill`.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| One reader | every merged read uses the ordered layers | D2-D5 | 1 | `config_layers` tests + full suite |
| Guard | team.json and the global file are guarded | D6 | 1 | `team_guard` tests |
| Verbs | global flags, explain, validate, split-team | D7, D8 | 1 | `config_global`, `split_team` tests |
| Onboarding | seed team.json, offer split-team | D9 | 1 | `team_json` onboarding tests |
| Proof and dogfood | live drive, beehive on team.json, docs | D10 | 2 | sandbox drive, `bee team show` |

SMALLER PATH: only a global file and no team.json is cheaper, but the owner chose to split the team table (1B). Only team.json and no global is cheaper, but it leaves the "change once for all projects" ask undone. Merging the guard into the reader cell is cheaper in ceremony, but puts the security change in a cell whose proof is about reads. PASS.

PLAN CHECK (five seats, one round): `reports/hat-wave.md`. BLOCKERS, all closed in this draft: herding readers of config.json (tgc-2 added); a global team value always shadowed (D11, owner chose 1A); the guard ignoring the global layer (tgc-3); the rollout order (leader step after reinstall). The owner answered the agent-write question: only waggledance writes the global file (D12).

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| tgc-1 | Read config through one ordered list of layers | config_layers.rs (new), main.rs, state.rs, hooks/compaction.rs, hooks/session_preamble/state.rs | — | a value in the global file shows in the merged config of any project; a project value wins | `cargo test … --bin bee -- config_layers` |
| tgc-2 | Move every direct config value read onto the layered reader | herding.rs, herding/{wave,job_verbs,broker,control_loop}.rs, verbs/worktree/{paseo_ws,registry}.rs, verbs/{report,detector}.rs, verbs/status_full/store.rs | tgc-1 | herding finds its agent table in team.json; a new worktree gets team.json | `cargo test … --bin bee -- config_layers_reader herding` |
| tgc-3 | Guard team.json and the global config file | hooks/write_guard/guards.rs, main.rs, tests.rs | tgc-2 | an agent edit of a slot in team.json is refused; an agent write to the global file, or a Bash that sets BEEHIVE_CONFIG_DIR, is refused | `cargo test … --bin bee -- team_guard config_governed` |
| tgc-4 | Add the read-only global layer, layer-aware explain and validate, and split-team to bee config | verbs/config.rs, verbs/mod.rs, generated/registry_payload.json, tests/config_verbs.rs | tgc-3 | `bee config explain` names the layer; `bee config split-team [--drop-global]` moves the table and says to commit both files | `cargo test … -- config_global split_team` + `--test config_verbs` |
| tgc-5 | Seed team.json at first onboard and offer split-team | onboard/templates.rs, plan.rs, notices.rs, apply.rs, mod.rs, tests.rs | tgc-4 | a new project gets team.json, or no team table when the global file has one; an old one gets a split-team notice | `cargo test … --bin bee -- team_json onboard` |
| tgc-6 | Drive the global layer and split-team end to end | tests/config_verbs.rs, .bee/verify/verify-app/features/ | tgc-5 | sandbox evidence for every path above | verify-app drive + added tests |
| tgc-8 | Refuse global-config writes and governed team.json changes in a herded pane | hooks/write_guard/worker_tree.rs, hooks/write_guard/tests.rs | tgc-6 | a herded worker's write to the global file, or a governed team.json change, is refused | `cargo test … --bin bee -- herded_team_guard` |
| tgc-7 | Document the config layers and write the delivery | docs/config-reference.md, docs/knowledge/**, delivery.md (new) | tgc-8 | config-reference and five concepts describe the layers | `bee knowledge check` |
| tgc-9 | Independent tester pass over the code and docs cells | packages/bee-rs/crates/bee/tests/config_verbs.rs, docs/history/team-global-config/reports/tester.md (new) | tgc-1, tgc-2, tgc-3, tgc-4, tgc-7, tgc-8 | each code cell's contract re-proved by a tester that shares no context with its implementer; docs pointers checked | `cargo test … --bin bee -- config_layers team_guard herded_team_guard config_global split_team` + `--test config_verbs` |
| tgc-10 | Refuse a Bash write that names the global config file through ~, $HOME, $XDG_CONFIG_HOME or $BEEHIVE_CONFIG_DIR | hooks/write_guard/guards.rs, hooks/write_guard/tests.rs | tgc-9 | `echo {} > ~/.config/beehive/config.json` is refused in a herded pane | `cargo test … --bin bee -- global_path_spelling team_guard herded_team_guard` |
| tgc-11 | Tester pass over the global path spellings | tests/config_verbs.rs, reports/tester.md | tgc-10 | each spelling is refused through the built binary in both panes | `cargo test … --test config_verbs` |

```json
[
  {
    "id": "tgc-1",
    "feature": "team-global-config",
    "title": "Read config through one ordered list of layers",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "21aebfc1-1ae9-4092-a3b9-507ae6731d7b",
      "16275c9a-eeae-4245-b937-5179c89c02a0",
      "65ef1514-58f6-41e9-89ba-99f4661b5636"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "packages/bee-rs/crates/bee/src/main.rs",
      "packages/bee-rs/crates/bee/src/state.rs",
      "packages/bee-rs/crates/bee/src/hooks/compaction.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/state.rs",
      "packages/bee-rs/crates/bee/src/hooks/compaction.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_preamble/state.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
      "packages/bee-rs/crates/bee/src/hooks/session_close/perf.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D1-D5 and contract C1. Anchors: state.rs `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` and `pub(crate) fn merge_config_overlay(base: &Value, overlay: &Value) -> Value {`; hooks/compaction.rs `pub(crate) fn read_config_failopen(root: &Path) -> Map<String, Value> {`; hooks/session_preamble/state.rs `pub(crate) fn read_config_raw_open(root: &Path) -> JMap {`; verbs/drivers/models.rs `pub(crate) fn fold_team_key(map: &mut Map<String, Value>) -> bool {`; hooks/session_close/perf.rs `pub(crate) fn global_perf_dir() -> PathBuf {` (the XDG order to follow). (1) New module config_layers.rs, declared in main.rs. It owns: an enum naming the four file layers in precedence order Global, Tracked, Team, Local (the order lives only here); the global config path per D3 (`BEEHIVE_CONFIG_DIR` non-empty, else `XDG_CONFIG_HOME/beehive`, else home `.config/beehive`, file `config.json`); a layer builder that takes a repo root and returns each layer's parsed object or none, with a corrupt-file policy parameter (strict: warn like read_config_raw does today and read as absent; fail-open: read as absent silently, matching each fail-open copy's current behavior); and a pure merge that folds the layers lowest first with merge_config_overlay. Per layer before merge: in the Team layer keep only `team`, `herding.agents`, `herding.agent_command` and drop every other key; in the Global layer drop `guards`, `gate_bypass` and `hooks` (D4). Keep a public helper that lists the forbidden global keys and the team.json allowed keys, so tgc-2 and tgc-3 import one list. (2) read_config_raw, read_config_failopen and read_config_raw_open become thin calls of the builder plus merge with their current corrupt policy and their current post-steps (advisor strip, fold) unchanged; no reader may build its own layer list. (3) Direct readers outside these three move in tgc-2, not here. (4) Byte-identity: with no team.json and no global file, every reader returns exactly what it returns today. Tests must set BEEHIVE_CONFIG_DIR to a temp dir so no test reads the real home. Red first, test names containing config_layers: each adjacent precedence pair (global<tracked, tracked<team, team<local); a team.json foreign key is ignored; each of guards, gate_bypass, hooks in the global file is dropped and hook_enabled stays true for write-guard; a corrupt layer reads as empty for both policies; mixed `models` and `team` across layers (config.json models, config.local.json team) gives today's result; with only config.json and config.local.json the output equals today's for all three readers. No code comments. Quote the matched test count, above zero. Plan-check additions (CONTEXT D13): fold_team_key runs ONCE on the merged object, after the merge, exactly as read_config_raw does today (state.rs:186); never per layer. Corrupt policy is a three-row truth table taken from today's code, not two values: read_config_raw warns and strips advisor (state.rs:168-171, :185); read_config_failopen warns and does not strip advisor (compaction.rs:128-131); read_config_raw_open is silent (session_preamble/jsval.rs:124-128). Each reader keeps its row. Delete the byte-copy of merge_config_overlay in hooks/session_preamble/state.rs (~line 24) and call the one in state.rs. Hermetic tests: the builder takes the global dir as an explicit argument; the default resolver returns no global dir under cfg(test), so no existing or new unit test reads the developer's real global file; integration tests set BEEHIVE_CONFIG_DIR on the child process only. A corrupt global file: the strict reader's warning names the global file path and says to run bee config validate.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- config_layers",
    "must_haves": {
      "truths": [
        "A key set only in the global config file appears in the merged config of any project",
        "A key set in config.json, team.json or config.local.json wins over the same key in the global file, in that order",
        "guards, gate_bypass and hooks in the global file never reach the merged config",
        "A key in team.json other than team, herding.agents and herding.agent_command never reaches the merged config",
        "With no team.json and no global file every reader returns what it returned before",
        "The layer order is written in one place",
        "The models-to-team fold runs once after the merge, so mixed models and team across layers give today's result",
        "No unit test reads the real global config file"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/config_layers.rs",
          "substantive": "layer enum, global path, builder, merge, key lists and config_layers tests"
        }
      ],
      "key_links": [
        "read_config_raw, read_config_failopen and read_config_raw_open call the config_layers builder"
      ],
      "prohibitions": [
        "No reader builds its own layer list",
        "No test reads the real user home",
        "No code comments"
      ],
      "open_choices": [
        "The names of the enum and functions"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-2",
    "feature": "team-global-config",
    "title": "Move every direct config value read onto the layered reader",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-1"
    ],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "21aebfc1-1ae9-4092-a3b9-507ae6731d7b"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/wave.rs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding/broker.rs",
      "packages/bee-rs/crates/bee/src/herding/control_loop.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/paseo_ws.rs",
      "packages/bee-rs/crates/bee/src/verbs/worktree/registry.rs",
      "packages/bee-rs/crates/bee/src/verbs/report.rs",
      "packages/bee-rs/crates/bee/src/verbs/detector.rs",
      "packages/bee-rs/crates/bee/src/verbs/status_full/store.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "packages/bee-rs/crates/bee/src/state.rs",
      "docs/knowledge/areas/bee-herding/thin-peer-boot.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/bee-herding/thin-peer-boot.md"
    ],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D5 and D13 for the direct readers. Each of these reads a config VALUE from .bee/config.json alone, so after a split-team the team table (herding.agents, herding.agent_command) or a global value would be missing. Move each to the merged view from config_layers (the same view read_config_raw returns for that root; for herding that root is the main checkout, as today): herding.rs:659, :689, :702, :1461, :1543 (repo_uses_paseo reads herding.agents), :1676 (read_command_template_tokens); herding/wave.rs:803 (resolve_agent_command reads agent_command and agents); herding/job_verbs.rs:890 (PaseoSpec::from_config); herding/broker.rs:316; herding/control_loop.rs:575; verbs/worktree/paseo_ws.rs:32 (herding.paseo.command); verbs/report.rs:113 (bee_report.repo); verbs/detector.rs:608 (herding.budget.tokens_per_job); verbs/status_full/store.rs:487 and :496 (open each: move it if it reads a value, keep it if it reads the tracked file as a file, and say which in the cap). Each reader keeps its current behavior on a corrupt or absent file (Null or None as today). verbs/worktree/registry.rs:974 copy_if_absent copies team.json beside config.json when the main store has it. Keep doctor.rs:593 and hooks/session_preamble/budget.rs:786 on config.json alone (they warn about the legacy models key in that file only), and keep hooks/write_guard/cross_project.rs (it reads guards only, which neither new layer may hold). Red first, test names containing config_layers_reader: with herding.agents and herding.agent_command only in team.json, resolve_agent_command, repo_uses_paseo and PaseoSpec::from_config resolve the same values as with them in config.json; a herding value only in the global file reaches the reader; a worktree bootstrap copies team.json. Existing herding tests stay green. Set the global dir explicitly in tests. No code comments. Quote the matched test count, above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- config_layers_reader herding",
    "must_haves": {
      "truths": [
        "Herding resolves herding.agents and herding.agent_command from team.json the same as from config.json",
        "A herding value set only in the global file reaches the herding readers",
        "A new worktree gets team.json when the main checkout has it",
        "No reader of a config value outside the legacy-models warnings and cross_project reads config.json alone"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/herding.rs",
          "substantive": "herding readers on the layered view"
        }
      ],
      "key_links": [
        "the herding readers call the config_layers merged view"
      ],
      "prohibitions": [
        "No change to what any key means",
        "No code comments"
      ],
      "open_choices": [
        "Whether a reader takes the merged map or a typed value"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-3",
    "feature": "team-global-config",
    "title": "Guard team.json and the global config file",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-2"
    ],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "16275c9a-eeae-4245-b937-5179c89c02a0",
      "65ef1514-58f6-41e9-89ba-99f4661b5636",
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md",
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md"
    ],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D6, D12, D13 and contract C2. Anchors in hooks/write_guard/guards.rs: the path gate `normalized == \".bee/config.json\"` (line ~93), `pub(crate) fn config_merged_text(tracked: Option<&str>, overlay: Option<&str>) -> Option<String> {`, `pub(crate) fn config_governed_change_deny(` and its path check for the two config files (~1437); in hooks/write_guard/main.rs the arm commented `config-role-key-guard` (~650-720) that refuses Bash on a config file and builds old and new merged text from disk. (1) `.bee/team.json` joins `.bee/config.json` and `.bee/config.local.json` everywhere the guard names config files: the path gate, the Bash refusal, the apply_patch arm, and config_governed_change_deny's path check. (2) The merged comparison builds both sides from all project layers in config_layers order (tracked, team, local), applying the Team-layer key filter from config_layers, so a governed change written to any of the three files is judged on the merged view, and a move between config.json and team.json that leaves the merged view equal is allowed. Generalize config_merged_text to take the layer texts in order and delegate to the config_layers merge and Team filter, never a second merge loop; update its callers (verbs/config.rs governed_refusal calls config_governed_change_deny; keep that call compiling with the same meaning for tracked and local, tgc-3 extends it). (3) Any Edit, Write, Bash redirect or apply_patch whose target resolves to the global config file path from config_layers is refused with a denial that says only waggledance writes that file. Also refuse a Bash command that names the global config path as an argument, and a Bash command that assigns BEEHIVE_CONFIG_DIR or XDG_CONFIG_HOME (an agent pointing bee at a dir it wrote would supply herding.agent_command, an executed command). (4) Do not change which keys are governed. Red first, test names containing team_guard: an Edit and a Write to team.json that change or remove a team slot are refused; adding a slot passes; a Bash write to team.json is refused; moving the team table from config.json to team.json with the merged view unchanged passes (Edit of both files in sequence, judged per write against the disk state); a write to the global config path is refused for Edit, Write and Bash; existing config.json and config.local.json guard tests stay green. Set BEEHIVE_CONFIG_DIR in tests. No code comments. Quote the matched test count, above zero. Plan-check additions (D13): both sides of the merged comparison sit on top of the on-disk global layer (with D4's forbidden keys dropped), so an agent ADD of a project slot that shadows a slot the global file sets is a change to the effective value and is refused. An unparseable team.json reads as an absent layer, so a write of invalid JSON to team.json diffs as a removal and is refused; the first-write pass in main.rs (~686, neither file exists) counts team.json too. The move test writes team.json first, then removes the keys from config.json; removing first is a removal and is refused, and a test pins that. Tests: global sets team.claude.code, an Edit adding a different value to each of the three project files is refused; invalid JSON to team.json is refused; `BEEHIVE_CONFIG_DIR=/tmp/x bee team show` in Bash is refused; a Bash command naming the global path is refused.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- team_guard config_governed",
    "must_haves": {
      "truths": [
        "An agent edit that changes or removes a team slot in .bee/team.json is refused",
        "Moving the team table between config.json and team.json with an equal merged view is allowed",
        "Any agent write to the global config file is refused and names the user as the editor",
        "The set of governed keys does not change",
        "An agent cannot add a project slot that shadows a slot the global file sets",
        "An invalid JSON write to team.json is refused",
        "A Bash command that assigns BEEHIVE_CONFIG_DIR or XDG_CONFIG_HOME, or names the global config path, is refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
          "substantive": "team.json in the path gate and the layered merged comparison, plus team_guard tests"
        }
      ],
      "key_links": [
        "the guard's merged comparison uses the config_layers order and Team filter"
      ],
      "prohibitions": [
        "No new governed key and no governed key removed",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-4",
    "feature": "team-global-config",
    "title": "Add the read-only global layer, layer-aware explain and validate, and split-team to bee config",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-3"
    ],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "21aebfc1-1ae9-4092-a3b9-507ae6731d7b",
      "16275c9a-eeae-4245-b937-5179c89c02a0",
      "8a66d22b-9787-450f-9e86-3bf2a9cd4bc7",
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/verbs/config.rs",
      "packages/bee-rs/crates/bee/src/verbs/mod.rs",
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/tests/config_verbs.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/knowledge/areas/rust-runtime/command-surface.md",
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "packages/bee-rs/crates/bee/src/verbs/config.rs",
      "packages/bee-rs/crates/bee/tests/config_verbs.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/rust-runtime/command-surface.md"
    ],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D7, D8, D11, D12, D13 and contract C3 in verbs/config.rs. Anchors: `const TRACKED: &str = \".bee/config.json\";`, `const LOCAL`, `fn read_layer`, `fn governed_refusal`, `fn write_edit`, `fn run_explain(ctx: &GCtx, key: &str) -> Result<(Value, String), String> {`, `fn run_validate(ctx: &GCtx) -> (Value, String) {`, `pub fn try_native(args: &[OsString], t0: Instant) -> Option<ExitCode> {`. Use config_layers for the paths, order, forbidden global keys and team.json allowed keys; write none of them again. (1) get takes `--global`, which reads only the global config file. set and unset take no `--global` (D12: only waggledance writes that file); `--global` on set or unset is an unknown flag. `--global` with `--local` on get refuses. Governed keys keep refusing, with the same FIX line. (2) explain: source is the highest layer that sets the key, one of global, tracked, team, local, unset; the output carries each layer's value (null when absent) and the effective merged value. Keep the existing JSON keys `tracked` and `local` and add `global` and `team`. (3) validate checks all four files: a file that is not valid JSON or not an object; a team.json key outside the allowed list; a key set in both config.json and team.json (problem kind duplicate, team.json wins); each forbidden key present in the global file; and the existing team-config problems on the merged view. Each problem names the file and key. (4) New verb `bee config split-team`: moves team, herding.agents and herding.agent_command from .bee/config.json to .bee/team.json verbatim, creating team.json when absent and merging into it when present; prunes a herding object left empty in config.json; computes the merged view of the project layers before and after and refuses with zero change when they differ; refuses with zero change and names the key when team.json already holds a different value for a moved key; a run with nothing to move exits 0 and says so. Write order: team.json first, then config.json, each atomic (temp file then rename), so a crash between the two leaves both files holding equal values, and a rerun from that state finishes cleanly. team.json is formatted like config.json (2-space pretty JSON with a trailing newline). --json output lists the moved keys. (5) Register `--global` on config.get only, and the new config.split-team verb with its `--drop-global` flag, in generated/registry_payload.json with descriptions, and update the config.explain and config.validate descriptions to name the four layers. That file is one compact JSON line and is hand-edited: change it by a JSON round-trip that keeps its compact byte format and key order. If a test pins a verb or flag count, update it only where the test names it. Red first, test names containing config_global or split_team (unit tests in config.rs, integration tests in tests/config_verbs.rs setting BEEHIVE_CONFIG_DIR): explain names each of the five sources; validate reports each problem kind; get --global reads only the global file; set and unset refuse --global; --global with --local refuses; split-team moves, leaves an equal merged view, is idempotent, refuses a conflicting value, and works on a copy of a real config with herding.agents. No code comments. Quote the matched test count, above zero. Plan-check additions: delete the local `fn merged` in verbs/config.rs (~85-91) and use the config_layers merge. split-team's human output and --json say to commit .bee/config.json and .bee/team.json together. D11: `bee config split-team --drop-global` also removes each team-table leaf (a role slot under team.<runtime>, an entry under herding.agents, herding.agent_command) whose value equals the global layer's value at the same path, from both config.json and team.json, and prunes objects left empty; the effective merged view (global included) must be equal before and after, checked inside the verb, or it refuses with zero change. Without --drop-global it never compares with global. validate lists the keys the global layer supplies (keys set in global and in no project file). Tests: split-team from the in-between state (both files hold equal values) finishes cleanly; --drop-global removes equal leaves, keeps differing ones, and leaves the effective view equal; validate lists global-supplied keys.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- config_global split_team && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test config_verbs --test registry_contracts",
    "must_haves": {
      "truths": [
        "bee config explain names the source layer as global, tracked, team, local or unset",
        "bee config validate reports team.json foreign keys, config.json and team.json duplicates, and forbidden global keys, each with file and key",
        "bee config split-team moves the team table with an equal merged view, is idempotent, and refuses a conflicting value with zero change",
        "The registry lists --global and config.split-team",
        "bee config has no command that writes the global file",
        "bee config split-team writes team.json before config.json and tells the user to commit both",
        "bee config split-team --drop-global removes project team values equal to global and keeps the effective view equal"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/verbs/config.rs",
          "substantive": "--global, layered explain and validate, split-team and their tests"
        }
      ],
      "key_links": [
        "verbs/config.rs takes paths and key lists from config_layers"
      ],
      "prohibitions": [
        "No governed key becomes writable through bee config",
        "No code comments"
      ],
      "open_choices": [
        "The text of each validate problem"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-5",
    "feature": "team-global-config",
    "title": "Seed team.json at first onboard and offer split-team",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-4"
    ],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "8a66d22b-9787-450f-9e86-3bf2a9cd4bc7"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/host-project-artifacts.md"
    ],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D9 and D11. Anchors: onboard/templates.rs the seeded config object holding `\"team\": {` (~199), `agent_command` (~229) and `agents` (~241), the create-if-missing seed of the sample config (~302-307), and the team.pi insertion (~279-287); onboard/plan.rs `pi_team_target` and `design_role_target` (~921, ~941) and the config edit planning (~1095-1142); onboard/notices.rs `pub fn commands_notices` (the notice posture to follow). (1) A first onboard (no .bee/config.json yet) writes the team table (team, herding.agents, herding.agent_command) into .bee/team.json and the rest into .bee/config.json; the merged view equals what one config.json held before. (2) The team.pi and design-role offers find and edit the team table in whichever project file holds it (team.json when it holds team, else config.json), reading the merged view through config_layers. (3) An onboard of a project whose .bee/config.json holds a team or herding.agents key adds one notice that says the team table can move to .bee/team.json with `bee config split-team`, that the move keeps every value, and to ask the owner first; it never moves it. No notice when config.json holds none of the three keys. (4) .bee/team.json is tracked: it must not be added to any gitignore list the onboarding writes. Red first, test names containing team_json: a first onboard writes team.json with the three keys and config.json without them, and the merged view equals the old seed; an onboard with team in config.json returns the split-team notice; an onboard with team only in team.json returns no such notice; the team.pi offer edits team.json when team lives there. No code comments. Quote the matched test count, above zero. Plan-check additions: derive the two seed files by splitting the one default_config() object on the config_layers team-key list; write no second team literal. D11: when the global layer (through config_layers) sets team, a first onboard writes no team table into the project (no team.json, and none in config.json); herding.agents and herding.agent_command follow the same rule when the global layer sets them. The split-team notice also says to commit both files together, and when the global layer sets team it names `bee config split-team --drop-global`. Tests: with a global team, a first onboard writes no team table; with a global team and a project team in config.json, the notice names --drop-global.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- team_json onboard",
    "must_haves": {
      "truths": [
        "A first onboard writes the team table to .bee/team.json and not to .bee/config.json",
        "An onboard of a project with the team table in config.json prints one notice that offers bee config split-team",
        "Onboarding never moves the team table on its own",
        "The team.pi and design-role offers edit the file that holds the team table",
        "With a global team table, a first onboard seeds no team table into the project"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/templates.rs",
          "substantive": "team.json seed"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/notices.rs",
          "substantive": "split-team notice and team_json tests"
        }
      ],
      "key_links": [
        "onboarding reads the merged view through config_layers"
      ],
      "prohibitions": [
        "Never move an existing team table during onboard",
        "No code comments"
      ],
      "open_choices": [
        "The notice wording"
      ]
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-6",
    "feature": "team-global-config",
    "title": "Drive the global layer and split-team end to end",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": [
      "tgc-5"
    ],
    "decisions": [
      "21aebfc1-1ae9-4092-a3b9-507ae6731d7b",
      "16275c9a-eeae-4245-b937-5179c89c02a0",
      "8a66d22b-9787-450f-9e86-3bf2a9cd4bc7",
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/config_verbs.rs",
      ".bee/verify/verify-app/features/config-layers.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      ".bee/verify/verify-app/SKILL.md",
      ".bee/verify/verify-app/features/README.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "You are the tester; you did not write tgc-1 to tgc-5. Verify docs/history/team-global-config/CONTEXT.md D2, D4, D6, D8, D11, D12 on the real built binary. Build once (`cargo build --release --manifest-path packages/bee-rs/Cargo.toml`) and drive target/release/bee through the verify-app skill (.bee/verify/verify-app/SKILL.md) in a throwaway onboarded sandbox, with BEEHIVE_CONFIG_DIR set on each bee process to a sandbox dir that the test harness itself writes (you are the harness, not an agent under the guard), so the real home is never read or written. Drive and capture evidence for: a hand-written global file value shows in `bee config explain` with source global, and a project value for the same key wins; a global file with gate_bypass, hooks and guards keys: `bee config validate` reports each and the merged config holds none; a corrupt global file warns with its path; with a global team table a first onboard seeds no team table and `bee team show` shows the global slots; on a sandbox whose config.json holds the team table, onboard prints the split-team notice, `bee config split-team` moves it, `bee team show --json` is identical before and after, a second run is a no-op, and `--drop-global` with an equal global table removes the project copy and keeps `bee team show` identical; after the split, the herding agent command a wave resolves is unchanged. Add integration tests in tests/config_verbs.rs only for paths tgc-4's tests do not already cover. Write .bee/verify/verify-app/features/config-layers.md (what it is, how a user reaches it, how to drive it, gotchas) and its README index row. A red finding is reported, not fixed: return BLOCKED with the evidence. No code comments.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test config_verbs",
    "must_haves": {
      "truths": [
        "The built binary reads a global value and lets a project value win",
        "The merged config never holds gate_bypass, hooks or guards from the global file",
        "split-team and split-team --drop-global leave bee team show output identical and are idempotent on the built binary",
        "The verify-app feature file for config layers exists and is indexed"
      ],
      "artifacts": [
        {
          "path": ".bee/verify/verify-app/features/config-layers.md",
          "substantive": "how to drive config layers and split-team"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "Never read or write the real ~/.config/beehive",
        "Never fix product code; report a red finding",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "tgc-8",
    "feature": "team-global-config",
    "title": "Refuse global-config writes and governed team.json changes in a herded pane",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-6"
    ],
    "decisions": [
      "a0f6985d-2d1f-46de-b93b-25fa649f5434",
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba",
      "09763647-9caf-4532-bfa6-4ddb380901ad"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/worker_tree.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/worker_tree.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/mod.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Implement docs/history/team-global-config/CONTEXT.md D14. A herded pane (BEE_HERDING_WORKER=1) routes write-guard to `write_guard::run_herded` in worker_tree.rs, whose `emit_for` runs only `whole_tree_denial`. Make `emit_for` also refuse, before the tree check: (1) any write to the global config file, through the same `global_config_write_denial(tool_name, tool_input, root, cwd)` that main.rs calls (Edit, Write, MultiEdit, apply_patch, a Bash naming the path, a Bash assigning BEEHIVE_CONFIG_DIR or XDG_CONFIG_HOME); (2) a governed change in .bee/config.json or .bee/team.json, through the same governed-key check main.rs uses for those paths (find it next to the `global_config_write_denial` call in main.rs; reuse it, never copy it). Keep the onboarding.json and hook_enabled preconditions that whole_tree_denial already applies. Every other guard stays open in the herded pane (herding-worker-standalone D3). Red first: tests whose names contain herded_team_guard in hooks/write_guard/tests.rs, calling `run_herded` or `emit_for` with a hook payload: a Write to the global file is refused (set BEEHIVE_CONFIG_DIR to a temp dir), a Bash that assigns BEEHIVE_CONFIG_DIR is refused, an Edit that changes a team slot in .bee/team.json is refused, a plain source-file Write passes. Watch them fail before the change. No code comments. Quote the matched test count, above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- herded_team_guard team_guard",
    "must_haves": {
      "truths": [
        "A herded pane write to the global config file is refused",
        "A herded pane governed change in .bee/team.json is refused",
        "A herded pane plain source write still passes"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/worker_tree.rs",
          "substantive": "emit_for applies the global and governed checks"
        }
      ],
      "key_links": [
        "worker_tree.rs calls the same global_config_write_denial and governed check as main.rs"
      ],
      "prohibitions": [
        "No code comments",
        "No second copy of the guard logic"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-7",
    "feature": "team-global-config",
    "title": "Document the config layers and write the delivery",
    "lane": "high-risk",
    "role": "docs",
    "status": "open",
    "deps": [
      "tgc-8"
    ],
    "decisions": [
      "465ea1e4-92ea-4529-9c96-e7435559897f",
      "21aebfc1-1ae9-4092-a3b9-507ae6731d7b",
      "16275c9a-eeae-4245-b937-5179c89c02a0",
      "65ef1514-58f6-41e9-89ba-99f4661b5636",
      "8a66d22b-9787-450f-9e86-3bf2a9cd4bc7",
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba"
    ],
    "files": [
      "docs/config-reference.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md",
      "docs/knowledge/areas/rust-runtime/command-surface.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "docs/history/team-global-config/delivery.md",
      "docs/knowledge/areas/bee-herding/thin-peer-boot.md"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/config-reference.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "docs/knowledge/areas/hook-runtime/governed-paths-and-the-intake-gate.md",
      "docs/knowledge/areas/rust-runtime/command-surface.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "docs/knowledge/areas/bee-herding/thin-peer-boot.md"
    ],
    "action": "Implement the docs half of docs/history/team-global-config/CONTEXT.md D10. Load the bee-technical-writing skill. (1) docs/config-reference.md: a section near the top that names the four files and the default in precedence order, the global path and its env vars, that only waggledance writes the global file (D12), the keys the global file may not set and why, what team.json holds, `bee config split-team` and `--drop-global` (D11), and `get --global`; fix every line that says config is read from two files. (2) Sync the knowledge concepts in files: config read order and layers (write-guard-request-shapes), the guard's coverage of team.json and the global file (governed-paths-and-the-intake-gate), the new flags and verb (command-surface), the team.json seed and notice (host-project-artifacts), and herding reading the layered view (thin-peer-boot). Cite CONTEXT.md D-numbers, never raw decision ids. (3) Write docs/history/team-global-config/delivery.md: concepts changed, known limits (the legacy models warnings read config.json only; the waggledance writer is a separate request), and the rollout: install the new binary first, then split; undo with git revert of the split commit. One commit. beehive's own split is NOT in this cell: the leader runs it after the merge and the reinstall (D13).",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "docs/config-reference.md names the four layers in order, the global path, the waggledance-only writer and the forbidden global keys",
        "The five knowledge concepts describe the new behavior and cite D-numbers",
        "delivery.md names the rollout order and the undo"
      ],
      "artifacts": [
        {
          "path": "docs/config-reference.md",
          "substantive": "the config layers section"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No raw decision ids in area docs",
        "No change to .bee/config.json in this cell"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "tgc-9",
    "feature": "team-global-config",
    "title": "Independent tester pass over the code and docs cells",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": [
      "tgc-1",
      "tgc-2",
      "tgc-3",
      "tgc-4",
      "tgc-7",
      "tgc-8"
    ],
    "decisions": [
      "70e20aff-e422-4e0a-bacc-3b1b03d13c8f",
      "09763647-9caf-4532-bfa6-4ddb380901ad",
      "ab46d221-2783-472a-bec7-db80111b7a9d",
      "a0f6985d-2d1f-46de-b93b-25fa649f5434"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/config_verbs.rs",
      "docs/history/team-global-config/reports/tester.md"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/history/team-global-config/plan.md",
      "packages/bee-rs/crates/bee/src/config_layers.rs",
      "docs/config-reference.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "You are the tester for cells tgc-1, tgc-2, tgc-3, tgc-4, tgc-7 and tgc-8; you share no context with their implementers. Read CONTEXT.md D1-D14 and C1-C3. For each code cell, read its diff (git log on this branch names each commit by its `cell: <id>` trailer) and check its contract against the decisions, not against the implementer's own tests: C1 config-layers (precedence for each adjacent pair, forbidden global keys dropped, team.json foreign keys ignored, corrupt layer read as empty) for tgc-1 and tgc-2; C2 team-guard (governed team.json change refused, equal-view move allowed, global-file write refused) for tgc-3; C3 config-verbs for tgc-4; D14 herded pane for tgc-8. Where a contract point has no test that would fail if the behavior broke, add one to tests/config_verbs.rs (integration, BEEHIVE_CONFIG_DIR set to a temp dir) whose name contains config_tester, and watch it fail by a temporary local break that you revert before the commit. For tgc-7, check that every path and D-number cited in docs/config-reference.md and the five concepts it changed resolves (file exists, D-number exists in CONTEXT.md). Write docs/history/team-global-config/reports/tester.md: one row per cell, its contract points, the test that proves each, any gap found and what you added. A defect in production code is reported, never fixed here. No code comments. Quote the matched test counts, above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- config_layers team_guard herded_team_guard config_global split_team && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test config_verbs",
    "must_haves": {
      "truths": [
        "Each of tgc-1, tgc-2, tgc-3, tgc-4, tgc-8 has its contract points mapped to a test that fails when the behavior breaks",
        "Every path and D-number tgc-7 cites resolves",
        "reports/tester.md lists each cell, its proof and any gap"
      ],
      "artifacts": [
        {
          "path": "docs/history/team-global-config/reports/tester.md",
          "substantive": "per-cell tester verdicts"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No production code change",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  },
  {
    "id": "tgc-10",
    "feature": "team-global-config",
    "title": "Refuse a Bash write that names the global config file through ~, $HOME, $XDG_CONFIG_HOME or $BEEHIVE_CONFIG_DIR",
    "lane": "high-risk",
    "role": "code",
    "status": "open",
    "deps": [
      "tgc-9"
    ],
    "decisions": [
      "d628826a-cedd-4bac-b5c3-1ef87b66d1ba",
      "a0f6985d-2d1f-46de-b93b-25fa649f5434",
      "09763647-9caf-4532-bfa6-4ddb380901ad"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/history/team-global-config/reports/tester.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/config_layers.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Fix the defect in docs/history/team-global-config/reports/tester.md (section 'Defect found'). CONTEXT.md D12 says the guard refuses a Bash command that names the global config path; D14 applies it in a herded pane. The Bash arm of `global_config_write_denial` in hooks/write_guard/guards.rs matches the literal absolute path only, so `echo {} > ~/.config/beehive/config.json`, `$HOME/.config/beehive/config.json`, `${HOME}/...`, `$XDG_CONFIG_HOME/beehive/config.json` and `$BEEHIVE_CONFIG_DIR/config.json` pass. Make the Bash arm also refuse a command whose text names the global file through each of those spellings: expand a leading `~` and the `$HOME`/`${HOME}`, `$XDG_CONFIG_HOME`/`${XDG_CONFIG_HOME}` and `$BEEHIVE_CONFIG_DIR`/`${BEEHIVE_CONFIG_DIR}` references in each token, using the current env values, and compare the result with the resolved global file path from config_layers. A token that resolves to the global file through ANY of these spellings is refused, even when the env var would resolve differently (the agent can set it in the same command). Take the global path from config_layers; never restate the path rules. A Bash command that only reads the file (cat, rg) is refused too, as today for the literal path. Every other write stays as it is. Red first: tests whose names contain global_path_spelling in hooks/write_guard/tests.rs, through `emit_for` in worker_tree.rs and through the hooked guard, one per spelling, plus one that a Bash write to an unrelated `~/.config/other/config.json` still passes. Watch them fail before the change. No code comments. Quote the matched test count, above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- global_path_spelling team_guard herded_team_guard",
    "must_haves": {
      "truths": [
        "A herded pane Bash write to the global file through ~, $HOME, $XDG_CONFIG_HOME or $BEEHIVE_CONFIG_DIR is refused",
        "A Bash write to an unrelated file under ~/.config still passes"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
          "substantive": "the Bash arm resolves the path spellings"
        }
      ],
      "key_links": [
        "guards.rs takes the global path from config_layers"
      ],
      "prohibitions": [
        "No code comments",
        "No second copy of the global path rules"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "tgc-11",
    "feature": "team-global-config",
    "title": "Tester pass over the global path spellings",
    "lane": "high-risk",
    "role": "test",
    "status": "open",
    "deps": [
      "tgc-10"
    ],
    "decisions": [
      "09763647-9caf-4532-bfa6-4ddb380901ad",
      "a0f6985d-2d1f-46de-b93b-25fa649f5434"
    ],
    "files": [
      "packages/bee-rs/crates/bee/tests/config_verbs.rs",
      "docs/history/team-global-config/reports/tester.md"
    ],
    "read_first": [
      "docs/history/team-global-config/CONTEXT.md",
      "docs/history/team-global-config/reports/tester.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "You are the tester for tgc-10; you share no context with its implementer. Through the built binary (`bee hook write-guard` with a hook payload on stdin, HOME and XDG_CONFIG_HOME pointed at temp dirs, BEEHIVE_CONFIG_DIR unset and set), prove in tests/config_verbs.rs, with names containing config_tester_spelling: each spelling (~, $HOME, ${HOME}, $XDG_CONFIG_HOME, $BEEHIVE_CONFIG_DIR) of the global file is refused in a herded pane (BEE_HERDING_WORKER=1) and in a hooked pane; an unrelated ~/.config/other/config.json write passes in the herded pane. Watch each fail under a temporary local revert of tgc-10's change, then restore it before the commit. Append a tgc-10 row and a 'Defect closed' line to docs/history/team-global-config/reports/tester.md. A production defect is reported, never fixed here. No code comments. Quote the matched test count, above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test config_verbs",
    "must_haves": {
      "truths": [
        "Each global path spelling is refused through the built binary in both panes",
        "reports/tester.md records tgc-10 and the closed defect"
      ],
      "artifacts": [
        {
          "path": "docs/history/team-global-config/reports/tester.md",
          "substantive": "the tgc-10 row"
        }
      ],
      "key_links": [],
      "prohibitions": [
        "No production code change",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Contract | Cell | Test marker |
|---|---|---|
| C1 config-layers | tgc-1, tgc-2 | `config_layers`, `config_layers_reader` |
| C2 team-guard | tgc-3, tgc-8 | `team_guard`, `herded_team_guard` |
| C3 config-verbs | tgc-4, tgc-6 | `config_global`, `split_team` |
| D9, D11 onboarding | tgc-5 | `team_json` |
| D10 dogfood | leader, after reinstall | `bee team show` diff |
