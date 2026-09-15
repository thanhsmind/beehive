---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Pi stage dispatch

## Summary

A Pi session will be able to ask advisors and hats for a second view, and send
work to every stage, without tools Pi does not have. Each hat runs in the
background, comes back named by its seat, and carries its own point of view.
The leader reads the full answer, not a one-line summary. The dispatch command
and the advisor record work from inside the feature worktree. Claude and Codex
keep their behavior, except that hats on every runtime now name their seat.

Mode: `standard` — 3 risk flags: public-contracts, covered-contract-change, multi-domain
Why this is the least workflow that protects the work: the dispatch payload is a
public contract with pinned tests, and four separate areas change, but nothing
touches auth, data loss, or an external provider.

Class playbook: `skills/bee-planning/references/planning-reference.md` ("Class playbooks" → "feature").

## Requirements (from CONTEXT.md)

- D1: Every dispatch and result-collection step Pi reads in its skills is one Pi can run; Codex keeps its own steps.
- D2: The Pi detached-delivery instruction names a session token the agent can really get; a test runs the command taken from that instruction.
- D3: A Pi hat wave keeps its 10-minute budget; seats run in parallel, results arrive named by seat, a late seat is dropped and named.
- D4: The leader gets each worker's full answer on every Pi dispatch kind.
- D5: A hat dispatch carries its seat's perspective and instrument from their one home, on every runtime, without `--brief-file`.
- D6: A Pi leader inside its feature worktree can run the dispatch door without leaving it.
- D7: An advisor, hat, or reviewer dispatch for a feature runs in that feature's worktree.
- D8: Pi contract tests cover advisor, hat, reviewer, and cell dispatch, plus one detached drain round trip.
- D9: Pi dispatch stays herding-only; version labels say 0.84–0.85.

Plan decisions from the hat wave: `15109d96`.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = command executed, output held. Evidence is a verbatim substring of the anchor.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The skill tree Pi reads is rendered for Codex. | read | .agents/skills/.bee-render.json:3 | `"target_runtime": "codex",` |
| 2 | The `.agents` skill root always renders one runtime, Codex. | read | packages/bee-rs/crates/bee/src/onboard/render.rs:367 | `"repo-agents" => "codex",` |
| 3 | The dev render marker grammar has no `pi` label today. | read | packages/bee-rs/crates/bee/src/devtools/skill_trees.rs:49 | `const MARKER_RUNTIMES: [&str; 3] = ["claude", "codex", "opencode"];` |
| 4 | Onboarding, which writes `.agents/skills`, checks labels against its own list, also without `pi`. | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:352 | `pub const RENDER_RUNTIMES: &[&str] = &["claude", "codex", "opencode"];` |
| 5 | A second Pi-only skill tree would collide by name with `.agents/skills`, and Pi keeps only the first. | read | /home/thanhsmind/.local/share/mise/installs/pi/0.85.1/pi/docs/skills.md:189 | `Name collisions (same name from different locations) warn and keep the first skill found.` |
| 6 | The detached note points at a token source that does not exist. | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:98 | `the session id the pi preamble shows you` |
| 7 | Pi's bash tool exposes the session id as an environment variable. | read | /home/thanhsmind/.local/share/mise/installs/pi/0.85.1/pi/docs/environment-variables.md:26 | `Current session ID` |
| 8 | An existing test pins the old note wording and must change. | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:5371 | `note.contains("session id") && note.contains("preamble"),` |
| 9 | The inbox marker names the job but no seat. | read | packages/bee-rs/crates/bee/src/herding/run.rs:2184 | `m.insert("job_id".into(), Value::String(opts.job_id.clone()));` |
| 10 | `herding run` accepts a caller job id, but the allocator owns uniqueness, so the seat needs its own field. | read | packages/bee-rs/crates/bee/src/herding/run.rs:310 | `"--job-id" => {` |
| 11 | A dry run writes no inbox marker, so a test round trip cannot rely on a dry run for the marker. | read | packages/bee-rs/crates/bee/src/herding/run.rs:2205 | `if !opts.dry_run {` |
| 12 | The Pi drain injects `report_path` but not the report body. | read | .pi/extensions/bee-guard.ts:751 | `push("report_path", result.report_path)` |
| 13 | The advisor prompt opens with one generic line and no seat block. | read | packages/bee/prompts/advisor.md:1 | `Advisor consult: produce an independent digest/opinion on the given question. Read-only.` |
| 14 | `dispatch prepare` refuses inside a granted worktree. | read | packages/bee-rs/crates/bee/src/verbs/mod.rs:180 | `refused inside a granted feature worktree` |
| 15 | The refusal already carries main's root, so a verb can serve from it. | read | packages/bee-rs/crates/bee/src/roots.rs:635 | `Roots::Unsupported(Unsupported::GrantedWorktree { main_root })` |
| 16 | Prepare returns at the refusal before any work. | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:3348 | `Roots::Unsupported(why) => {` |
| 17 | The advisor record also refuses inside a granted worktree, so a Pi leader there cannot record the wave. | ran | `.bee/bin/bee state advisor-ref show --json` (from the feature worktree) | `refused inside a granted feature worktree` |
| 18 | A non-cell dispatch without `--feature` takes the session's bound lane. | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1463 | `crate::verbs::state_group::session_binding(session_id, root)` |
| 19 | After that, every runtime falls back to main's default `state.json` feature for the worktree cwd; this stays. | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:1814 | `.or_else(\|\| match crate::fsutil::read_json(&root.join(".bee").join("state.json")) {` |
| 20 | With `--feature`, an advisor herding payload already runs in the feature worktree. | ran | `.bee/bin/bee dispatch prepare --runtime claude --kind advisor --role hat-user-impact --feature pi-stage-dispatch --json` (from main) | `--cwd \"/home/thanhsmind/Projects/goglbe/beehive--wt--pi-stage-dispatch\"` |
| 21 | A Pi advisor payload carries a 30-minute ceiling today. | ran | `.bee/bin/bee dispatch prepare --runtime pi --kind advisor --purpose "test hat" --json` (from main) | `--agent \"pi-gpt-6-astra\" --ceiling 1800` |
| 22 | The hat wave budget is 10 minutes. | read | skills/bee-hive/references/gates-and-delegation.md:261 | `One wave, wall-clock ceiling 10 minutes` |
| 23 | Pi dispatch tests cover one extracted gather only. | read | packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:2183 | `fn injected_dispatch_guidance_extracts_and_executes_pi_runtime_herding() {` |
| 24 | `.pi/extensions` is a shipped release root, so touching it needs the manifest check. | read | packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:83 | `".pi/extensions",` |

## Discovery

Three read-only reviews on 2026-09-15: Pi 0.84.4→0.85.1 API diff (no breakage),
a code walk of the Pi advisor/hat/stage path, and a mine of 68 Pi session logs.
The nine causes and their evidence are in `CONTEXT.md` ("Why now"). The hat
wave then found the live state.json fallback (claim 19), the onboarding label
list (claim 4), and the advisor record refusal (claim 17).

## Approach

**Recommended path.**

- **Skills (D1).** Add `pi` to both marker label lists (claims 3, 4). The
  `.agents/skills` root, which onboarding writes and Pi and Codex both read,
  keeps `bee:only codex` and `bee:only pi` blocks. `.claude/skills`,
  `.opencode/skills`, `.claude-plugin/skills`, and `.codex-plugin/skills` strip
  Pi blocks (the Codex plugin tree is not read by Pi). The render sidecar
  schema stays as it is. Every `bee:only pi` block opens with the words
  "On Pi:". Pi blocks go where a stage tells the leader how to dispatch or
  collect: the hat wave moves, swarming result collection, reviewing
  collection, and the unmarked native-subagent line in bee-swarming.
- **Payload note (D2, D4).** The Pi note names `--inbox-session "$PI_SESSION_ID"`,
  tells the leader to run in the foreground when that variable is empty, and
  says on both paths that the full answer is the file at `report_path`.
- **Seat name (D3).** A new `herding run --seat <name>` flag. The marker and
  the result envelope carry `seat`; the Pi drain shows a `seat:` row. On Pi
  payloads only, prepare appends `--seat "<role>"` to every non-cell herding
  command that names a role, and a `hat-*` command caps its ceiling at 600
  seconds (the wave budget; a lower configured value wins).
- **Seat block (D5).** On every runtime, a `hat-*` advisor prompt gets a seat
  block: the role name and a pointer to its row in bee-hive
  `references/gates-and-delegation.md` ("Hat wave"). The configured
  description is not copied — it is a stale second copy.
- **Worktree door (D6, D7).** `dispatch prepare` and `state advisor-ref
  record|show` serve from `main_root` when run inside a granted worktree.
  A non-cell dispatch resolves its feature: `--feature`, then the bound lane,
  then the granted worktree it runs in, then (unchanged) main's default
  `state.json` feature. Every other verb keeps its refusal.

**Rejected alternatives.**

- A separate `.pi/skills` tree: Pi keeps the first of two same-named skills
  and warns (claim 5) — 34 warnings per session and a new onboarding root.
- Seat name as `--job-id` or `--nickname`: the allocator owns job ids and the
  drain orders by them; the nickname already names the worker in its ack file.
- Copying the configured hat description into the prompt: a stale second copy.
- Removing the state.json fallback: changes Claude and Codex outside D5.
- A Pi wrapper that `cd`s to main: the write guard refuses it (F7).

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Serving two verbs from a worktree | MEDIUM — a wrong root would read the worktree store | psd-1 | same payload and same advisor-ref result from worktree and from main |
| Marker grammar + dual render | MEDIUM — other trees gain Pi text | psd-4, psd-5 | render tests; no `On Pi:` in the four other trees after regen |
| Seat flag + drain row | LOW | psd-2 | herding tests, drain row |
| Note, seat, ceiling, seat block | MEDIUM — changes pinned payload bytes | psd-3 | updated prepare tests; Claude/Codex bytes change only by the hat seat block |
| Live Pi wave | MEDIUM — only real proof of D3 and the token match | psd-8 | `green:live` evidence |

Waves: W1 {psd-1, psd-2, psd-4} in parallel (disjoint files). W2 {psd-3} waits
on psd-1 (same files) and psd-2 (the flag). W3 {psd-5, psd-6, psd-7} wait on
psd-3 (exact flag and note text); psd-5 also waits on psd-4. W4 {psd-8} waits on
psd-5 and psd-6.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "00c73e8ff91b0ac1e8d67be4c51374e021fa3a10b4387ecc9d1fbbb0440fbd9d",
  "stages": [
    {"stage":"planning","classification":"required","role":"plan","reason":"The leader drafts cells and the synthesis."},
    {"stage":"fact-extraction","classification":"conditional","role":"extraction","condition":"A narrow implementation fact is needed.","reason":"This role owns known-location extraction."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"A broader read-only digest is needed.","reason":"This role owns read-only repository lookup."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"The plan must be checked for missing facts and structure."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Four design choices have cheaper candidates."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"The Pi user and the Pi leader agent read this behavior."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Standard lane opens three seats."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Standard lane opens three seats."},
    {"stage":"implementation","classification":"required","role":"code","reason":"Rust, TypeScript, and their tests change together."},
    {"stage":"test-and-live-proof","classification":"required","role":"test","reason":"Pi contract tests and one live Pi wave prove the path."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"Skills, rendered trees, config reference, and knowledge must agree."},
    {"stage":"independent-review","classification":"conditional","role":"review","condition":"The user explicitly requests independent review.","reason":"Review remains a separate user-invoked pass."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The three hats supply the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended supervisor loop is needed."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"Causes are verified; no competing whole designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"Causes are verified; no competing whole designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"Causes are verified; no competing whole designs."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every planned job has a specific role."}
  ]
}
```

## Shape

Epic map — outcome: a Pi leader completes a hat wave and every stage dispatch
end to end. Basis: claims 1–24.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Door | prepare and advisor-ref from a worktree, feature scope | F7, F8, claim 17 | 1 | prepare and advisor-ref tests |
| Delivery | token, seat name, full answer, budget | F2–F5 | 1 | herding, drain, prepare tests |
| Perspective | hat seat block | F6 | 1 | prepare tests |
| Instructions | Pi skill blocks in the shared tree | F1 | 1 | render tests, regen parity |
| Proof | Pi contract tests + live wave | F9 | 1 | `--test pi_plugin_contracts`, `green:live` |
| Control plane | the other control verbs from a worktree (`gate`, `cells`, `state set`) | Pi logs P4: the most repeated refusal | 2 | re-triaged after slice 1's live run |

Slice 1 is the hat-wave and dispatch path end to end. Slice 2 (headline only,
no cells): the remaining control-plane verbs a relocated Pi leader runs.

## Cells — current slice preview

```json
[
  {
    "id":"psd-1",
    "feature":"pi-stage-dispatch",
    "role":"code",
    "lane":"standard",
    "title":"Serve dispatch prepare and the advisor record from inside a granted feature worktree",
    "action":"Per D6 and D7. In run_dispatch_prepare, when resolve_store_root returns Unsupported::GrantedWorktree { main_root }, continue with main_root as the store root instead of emitting the refusal; keep the LinkInvalid and no-root arms unchanged. Do the same for bee state advisor-ref record and show: find where they reach emit_unsupported_root (rg -n emit_unsupported_root packages/bee-rs/crates/bee/src/verbs/state_group) and serve those two from main_root only. Every other verb keeps its refusal. For a non-cell dispatch with no --feature, resolve the feature in this order: the session's bound lane (today, prepare.rs session_binding), then the granted worktree the command runs in (new), then main's default .bee/state.json feature (today, unchanged). Write the tests first: prepare run from a granted worktree returns the same payload as the same call from main; a non-cell dispatch from that worktree with no --feature and no bound lane carries --cwd of that worktree even when main's state.json names another feature; advisor-ref show from the worktree returns the same result as from main; another state verb still refuses there.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers:: advisor_ref",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/roots.rs","packages/bee-rs/crates/bee/src/verbs/mod.rs","packages/bee-rs/crates/bee/src/verbs/state_group/advisor_ref.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs","packages/bee-rs/crates/bee/src/verbs/state_group/*"],
    "deps":[],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["dispatch prepare run inside a granted worktree returns a payload instead of the granted-worktree refusal","state advisor-ref record and show work inside a granted worktree against main's store","A non-cell dispatch from a granted worktree with no --feature and no bound lane carries --cwd of that worktree"],"prohibitions":["No other verb stops refusing inside a granted worktree","The fallback to main's default state.json feature stays for every runtime","Cell dispatch feature resolution is unchanged"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-2",
    "feature":"pi-stage-dispatch",
    "role":"code",
    "lane":"standard",
    "title":"Name each detached herding result by its seat",
    "action":"Per D3 and D9. Add an optional --seat <name> flag to bee herding run. Write it into the result-inbox marker (write_inbox_marker, beside job_id) and into the run's JSON result envelope as seat. In .pi/extensions/bee-guard.ts renderResultInjection, push a seat row right after job_id when the marker carries one. In bee-guard.ts, change only the plain version label comment that says Pi 0.84.x has no interactive permission prompt event to say 0.84–0.85; leave comments that name which Pi docs or binary were read unchanged. Write tests first in herding/run.rs for the marker and envelope field, and for an absent flag leaving both byte-identical. Refresh the release manifest because .pi/extensions is a shipped root.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee herding::run && node --check .pi/extensions/bee-guard.ts && .bee/bin/bee dev release-manifest --write && .bee/bin/bee dev release-manifest --check",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","packages/bee-rs/crates/bee/src/herding/run.rs",".pi/extensions/bee-guard.ts"],
    "files":["packages/bee-rs/crates/bee/src/herding/run.rs",".pi/extensions/bee-guard.ts","docs/history/codex-harness-hardening/release-manifest.json"],
    "deps":[],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["herding run --seat hat-risks writes seat hat-risks into the inbox marker and the JSON envelope","The Pi drain injection shows a seat row when the marker has one","Without --seat the marker and envelope are byte-identical to before"],"prohibitions":["Job id allocation is unchanged","Comments naming the Pi docs or binary that were read keep their version"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-3",
    "feature":"pi-stage-dispatch",
    "role":"code",
    "lane":"standard",
    "title":"Carry seat, budget, session token, and full-answer pointer on dispatch payloads",
    "action":"Per D2, D3, D4, D5. (a) Rewrite HERDING_DETACHED_DELIVERY_PI: the detached run appends --inbox-session \"$PI_SESSION_ID\" (the variable Pi's bash tool exports); when that variable is empty, run in the foreground; on both paths the JSON summary is one line and the full answer is the file at report_path. Update the_pi_herding_payload_names_the_inbox_session_flag to assert PI_SESSION_ID, foreground, and report_path in the note. (b) On runtime pi only, every non-cell herding command that resolved a role appends --seat \"<role>\". (c) On runtime pi only, a hat-* herding command uses a ceiling of min(configured ceiling, 600). (d) Add a seat block to packages/bee/prompts/advisor.md, rendered on every runtime only for hat-* roles, holding the role name and the pointer: bee-hive skill, references/gates-and-delegation.md, section Hat wave, the row for this role. Pass it from the non-cell render arm. Do not copy the configured description. Tests: a non-hat advisor prompt is byte-identical to before; a Claude hat payload differs only by the seat block; a Pi hat payload carries --seat and the capped ceiling; a Claude herding hat payload carries neither. Refresh the release manifest (prompts are shipped).",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers:: prompts && .bee/bin/bee dev release-manifest --write && .bee/bin/bee dev release-manifest --check",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee/prompts/advisor.md","packages/bee-rs/crates/bee/src/devtools/prompts.rs","skills/bee-hive/references/gates-and-delegation.md"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs","packages/bee/prompts/advisor.md","docs/history/codex-harness-hardening/release-manifest.json"],
    "deps":["psd-1","psd-2"],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["The Pi detached note names --inbox-session \"$PI_SESSION_ID\", the foreground case, and report_path","A Pi hat-* herding command carries --seat and a ceiling no higher than 600","A hat-* advisor prompt on every runtime names its seat and the Hat wave home","A non-hat advisor prompt is byte-identical to before"],"prohibitions":["No hat instrument text or configured description is copied into the prompt","Claude and Codex herding commands do not gain --seat or the ceiling cap","Pi dispatch stays herding-only"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-4",
    "feature":"pi-stage-dispatch",
    "role":"code",
    "lane":"standard",
    "title":"Render Pi blocks into the shared agents skill tree only",
    "action":"Per D1. Add pi to MARKER_RUNTIMES in devtools/skill_trees.rs and to RENDER_RUNTIMES in onboard/templates.rs. Make the onboarding render for the repo-agents root keep both bee:only codex and bee:only pi blocks (Pi and Codex both read .agents/skills). The repo-claude and repo-opencode roots, and the dev render-skill-trees outputs .claude-plugin/skills and .codex-plugin/skills, strip pi blocks like any off-runtime block. Keep the .bee-render.json sidecar schema unchanged. Write tests first: a source with claude, codex, and pi blocks renders codex and pi text into the agents root, only claude text into the claude root, and no pi text into the opencode root or either plugin tree; a marker-free file stays byte-identical everywhere; a bee:only pi marker passes marker validation.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- render skill_trees",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","packages/bee-rs/crates/bee/src/onboard/render.rs","packages/bee-rs/crates/bee/src/onboard/templates.rs","packages/bee-rs/crates/bee/src/devtools/skill_trees.rs","packages/bee-rs/crates/bee/src/devtools/mod.rs"],
    "files":["packages/bee-rs/crates/bee/src/onboard/render.rs","packages/bee-rs/crates/bee/src/onboard/templates.rs","packages/bee-rs/crates/bee/src/devtools/skill_trees.rs","packages/bee-rs/crates/bee/src/onboard/tests.rs"],
    "deps":[],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["The agents skill root keeps codex and pi blocks","The claude root, the opencode root, and both plugin trees contain no pi block text","Marker-free skill files render byte-identical","A bee:only pi marker validates in both marker label lists"],"prohibitions":["No change to Claude-rendered skill bytes for existing sources","No render sidecar schema change"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-5",
    "feature":"pi-stage-dispatch",
    "role":"docs",
    "lane":"standard",
    "title":"Write the Pi dispatch and collection steps into the stage skills",
    "action":"Per D1, D3, D4. Add bee:only pi blocks, each opening with the words On Pi:, in skills/bee-hive/references/gates-and-delegation.md (Hat wave, The moves: launch every seat's payload in the background in one shell call with --inbox-session \"$PI_SESSION_ID\", keep working, match each injected result by its seat row, read report_path for the full answer, drop and name a seat still missing at 10 minutes, and treat a job_id already handled as a replay), in skills/bee-swarming/references/swarming-reference.md (a Pi result-collection row beside the Codex table: read report_path, never the summary alone), in skills/bee-swarming/SKILL.md (beside the Native subagents line: Pi dispatches through herding only), and in skills/bee-reviewing/SKILL.md (collect each reviewer's report_path before synthesis). Build the candidate binary, run its dev regen, and refresh the release manifest with it.",
    "verify":"export PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\"; cargo build --release --manifest-path packages/bee-rs/Cargo.toml -p bee && CANDIDATE=\"$(cargo metadata --no-deps --format-version 1 --manifest-path packages/bee-rs/Cargo.toml | jq -r .target_directory)/release/bee\" && \"$CANDIDATE\" dev regen && rg -q 'On Pi:' .agents/skills/bee-hive/references/gates-and-delegation.md && ! rg -q 'On Pi:' .claude/skills .opencode/skills .claude-plugin/skills .codex-plugin/skills && \"$CANDIDATE\" dev release-manifest --write && \"$CANDIDATE\" dev release-manifest --check",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","skills/bee-hive/references/gates-and-delegation.md","skills/bee-swarming/references/swarming-reference.md","skills/bee-swarming/SKILL.md","skills/bee-reviewing/SKILL.md"],
    "files":["skills/bee-hive/references/gates-and-delegation.md","skills/bee-swarming/references/swarming-reference.md","skills/bee-swarming/SKILL.md","skills/bee-reviewing/SKILL.md",".agents/skills/*",".claude/skills/*",".opencode/skills/*",".claude-plugin/skills/*",".codex-plugin/skills/*","docs/history/codex-harness-hardening/release-manifest.json"],
    "deps":["psd-3","psd-4"],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["The agents tree Hat wave section tells Pi to launch seats detached with $PI_SESSION_ID, match by seat, read report_path, and drop late seats by name","No other rendered tree contains On Pi: text","Every stage that collects worker results names report_path for Pi"],"prohibitions":["The hat wave procedure keeps its single home; no second copy"]},
    "affects_skills":["skills/bee-hive/references/gates-and-delegation.md","skills/bee-swarming/references/swarming-reference.md","skills/bee-swarming/SKILL.md","skills/bee-reviewing/SKILL.md"],
    "affects_specs":[]
  },
  {
    "id":"psd-6",
    "feature":"pi-stage-dispatch",
    "role":"test",
    "lane":"standard",
    "title":"Prove Pi advisor, hat, reviewer, and cell dispatch plus a detached round trip",
    "action":"Per D8 and D2. Extend tests/pi_plugin_contracts.rs beside injected_dispatch_guidance_extracts_and_executes_pi_runtime_herding, reusing its preamble extraction and the write_marker, inbox_dir, job_mailbox, and await_injections helpers. For kinds advisor, advisor with --role hat-facts-gaps, reviewer, and a claimed cell: take the dispatch command from the injected Pi preamble, run it with --runtime pi, and assert a herding payload; the hat payload carries --seat hat-facts-gaps and a ceiling no higher than 600 and a stdin naming the seat. Round trip: take the session token from the harness session (the same value getSessionId returns), set PI_SESSION_ID to it, build the detached command by appending the flag text found in the payload's note, run it with --dry-run and assert the parsed options carry that token; then, because a dry run writes no marker, write the marker with write_marker plus a seat field and a finished result, and assert the drain injects one message with the seat row and report_path, and that a non-done status still shows its seat row.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pi_plugin_contracts",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/herding/run.rs"],
    "files":["packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs"],
    "deps":["psd-3"],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["Pi advisor, hat, reviewer, and cell dispatch each return a herding payload from the injected command","The detached flag text taken from the note parses with the harness session token","The drain injects a seat row and report_path for done and non-done results"],"prohibitions":["No test writes the flag text itself instead of reading it from the payload note"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-7",
    "feature":"pi-stage-dispatch",
    "role":"docs",
    "lane":"standard",
    "title":"Sync Pi dispatch facts into the config reference and knowledge",
    "action":"Per D9 and the feature playbook's knowledge sync. In docs/config-reference.md, change the Pi 0.84.x label to 0.84–0.85 and document herding run --seat and the Pi hat ceiling cap. In docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md, add the drain seat row and the $PI_SESSION_ID token source. In docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md, record that a hat prompt carries its seat block on every runtime and that dispatch prepare and advisor-ref serve from a granted worktree. Extend existing entries; add no new document.",
    "verify":"rg -n 'seat' docs/config-reference.md docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md && ! rg -n 'Pi 0\\.84\\.x' docs/config-reference.md",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","docs/config-reference.md","docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "files":["docs/config-reference.md","docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"],
    "deps":["psd-3"],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["The config reference documents --seat and the Pi hat ceiling cap","The knowledge areas state the token source, the seat row, and the worktree door"],"prohibitions":["No second parity document or event catalog"]},
    "affects_skills":[],
    "affects_specs":["docs/knowledge/areas/hook-runtime/catalog-projections-and-activation.md","docs/knowledge/areas/doctrine-layer/model-roles-and-escalation.md"]
  },
  {
    "id":"psd-8",
    "feature":"pi-stage-dispatch",
    "role":"test",
    "lane":"standard",
    "title":"Drive a real Pi hat wave end to end in a sandbox",
    "action":"Per D2, D3, D4, D6, D7. Build and install the candidate binary into a fresh onboarded sandbox repo with a granted feature worktree and a plan.md. Open a real Pi 0.85.1 session in that worktree through a herdr pane. Have it run the three default hat dispatches exactly as the Pi skill text says, detached with $PI_SESSION_ID. Observe and write into .bee/verify/verify-app/features/pi-hat-wave.md (create it, and add its row to the features README): each dispatch prepare succeeded from the worktree; each hat ran in the worktree; each result was injected with its seat row, which proves the bash PI_SESSION_ID equals the drain token; the leader read each report_path; the wave finished inside 10 minutes; bee state advisor-ref record from that bound Pi leader succeeded or refused, with the exact output; and whether the Pi bash tool applied a default timeout. Include the Pi session log path so the evidence can be re-read.",
    "verify":"rg -n 'seat: hat-facts-gaps' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'advisor-ref record' .bee/verify/verify-app/features/pi-hat-wave.md && rg -n 'pi-hat-wave' .bee/verify/verify-app/features/README.md",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md",".bee/verify/verify-app/features/README.md","skills/bee-hive/references/gates-and-delegation.md"],
    "files":[".bee/verify/verify-app/features/pi-hat-wave.md",".bee/verify/verify-app/features/README.md"],
    "deps":["psd-5","psd-6"],
    "decisions":["a20cf301","15109d96"],
    "must_haves":{"truths":["A real Pi session completes a three-seat hat wave inside 10 minutes with seat-named results","Every dispatch prepare and the advisor record in that run succeed from the feature worktree","The feature file records how to drive the wave and the session log path"],"prohibitions":["No hand-edited payload in the live run"]},
    "affects_skills":[],
    "affects_specs":[]
  },
  {
    "id":"psd-9",
    "feature":"pi-stage-dispatch",
    "role":"plan",
    "lane":"standard",
    "title":"Keep the declared hat seat when an unconfigured hat falls through to advisor",
    "action":"Per D3 and D5, fix-first after psd-6 found the gap. In prepare.rs an undeclared hat seat (no team.<runtime> entry for it) sets fallen_through_seat and rebinds canonical_role to advisor; hat_seat is then derived from marker_role, which carries the rebound role, so a Pi hat that falls through gets --seat advisor, no 600-second ceiling cap, and no seat block. Derive hat_seat (and so the --seat value, the Pi ceiling cap, and the prompt seat block) from the caller's declared hat name, using fallen_through_seat when it is set, while model resolution stays on the advisor slot. Write the test first: with team.pi carrying no hat-* entries, --kind advisor --role hat-risks on runtime pi returns --seat \"hat-risks\", a ceiling no higher than 600, and a prompt naming seat hat-risks; with team.claude carrying no hat-* entries, the claude prompt names seat hat-risks and the command gains no --seat.",
    "verify":"PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- drivers::",
    "read_first":["docs/history/pi-stage-dispatch/CONTEXT.md","docs/history/pi-stage-dispatch/plan.md","packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"],
    "files":["packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs","packages/bee-rs/crates/bee/src/verbs/drivers/tests.rs"],
    "deps":["psd-3"],
    "decisions":["a20cf301","15109d96","c5aff99f"],
    "must_haves":{"truths":["An unconfigured hat seat on Pi carries --seat with its own hat name and a ceiling no higher than 600","An unconfigured hat seat prompt names its own seat on every runtime"],"prohibitions":["The unconfigured hat still resolves its model through the advisor slot","Configured hat seats and non-hat roles are unchanged"]},
    "affects_skills":[],
    "affects_specs":[]
  }
]
```

### Plan revision 1 (2026-09-15)

psd-6 found that an unconfigured hat seat falls through to `advisor` in
`prepare.rs` (`fallen_through_seat`, then `hat_seat` reads the rebound
`marker_role`), so a host with no `team.pi` hat entries loses the seat name,
the Pi ceiling cap, and the seat block (D3, D5). psd-9 is the fix-first cell
(decision `c5aff99f`). psd-8 was already persisted with deps psd-5 and psd-6;
it runs after psd-9 by orchestration, a named serial edge, not by a changed dep.

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| prepare from granted worktree | happy | JSON payload returned, no `refused inside a granted feature worktree` |
| advisor-ref show from granted worktree | happy | same output as the same call from main |
| other state verb from granted worktree | error | still prints `refused inside a granted feature worktree` |
| non-cell dispatch from worktree, no flag, main state.json names another feature | edge | command carries `--cwd` of the worktree it ran in |
| non-cell dispatch from main, no flag, no lane | edge | same bytes as before the change |
| `--seat` absent | edge | marker and envelope bytes equal the pre-change fixture |
| Pi note | happy | note contains `$PI_SESSION_ID`, `foreground`, and `report_path` |
| Pi hat ceiling with config 300 | edge | command carries `--ceiling 300` |
| Claude herding hat payload | edge | no `--seat`, ceiling unchanged; prompt has the seat block |
| non-hat advisor prompt | edge | prompt bytes equal the pre-change fixture on main and head |
| drain with a non-done result | error | injection shows `seat:` and the non-done status |
| rendered trees after regen | edge | `rg 'On Pi:'` finds text only under `.agents/skills` |
| live Pi wave | happy | three injected results with `seat:` rows inside 10 minutes |

## Open Questions

- Does the Pi bash tool apply a default timeout when none is passed? psd-8 records it; the plan does not rest on it, because seats run detached.
- Does the bash `PI_SESSION_ID` equal the drain token in a real session? Documented (claim 7), proven only by psd-8's seat-row injection.

## Out of scope

- Slice 2: the other control-plane verbs from a granted worktree (headline above).
- The herding pane that stalls on a worker CLI survey (backlog).
- Herding CLI shape friction (backlog).
- A native Pi worker transport (decision `9f5c6d17`).
