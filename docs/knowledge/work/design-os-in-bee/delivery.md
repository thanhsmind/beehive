---
type: bee.delivery
title: design-os-in-bee — delivery
description: "Delivery record proposed by bee knowledge promote for work item design-os-in-bee: 8 capped cell(s), 31 recorded deviation(s)."
timestamp: 2026-10-24
bee:
  id: design-os-in-bee-delivery
  lifecycle: active
  areas: [onboarding]
  required_context: [docs/history/design-os-in-bee/CONTEXT.md, docs/history/design-os-in-bee/plan.md]
  sources: [docs/history/design-os-in-bee/CONTEXT.md, docs/history/design-os-in-bee/plan.md, .bee/cells/dob-1.json, .bee/cells/dob-2.json, .bee/cells/dob-3.json, .bee/cells/dob-4.json, .bee/cells/dob-5.json, .bee/cells/dob-6.json, .bee/cells/dob-7.json, .bee/cells/dob-8.json]
---

# design-os-in-bee — Delivery

## What shipped

- **dob-1** — Re-cap after the owner resolved the judge's authority finding (decision f4730af1): vendored design-os taste in packages/bee/design, commit c004f6eb5 (5 file(s) changed)
- **dob-2** — Design cells dispatch as the new bee-design agent; every other role keeps bee-build (12 file(s) changed)
- **dob-3** — bee onboard --design-packs installs bee-design plus the picked packs into every repo skill home, prunes dropped packs, refuses unknown names before any write, offers the pick to hosts with UI files, and adds team.<runtime>.design once to upgraded configs (8 file(s) changed)
- **dob-4** — A gather or reviewer dispatch with role design now pins bee-gather or bee-review; a design cell still pins bee-design; each payload passes the model guard (2 file(s) changed)
- **dob-5** — bee design gate ratchets ui gate results against the baseline, refuses --write while a cell is claimed, records last-run.json, and the design skill, agent and preamble name it (11 file(s) changed)
- **dob-6** — In a host that picked design packs, the cap refuses a UI cell whose verify lacks bee design gate, needs the not-run phrase when last-run.json says not-run, refuses any cell listing the baseline file; cells add warns on stderr (5 file(s) changed)
- **dob-7** — plugin-first onboarding installs bee-design and the picked packs only, prunes un-picked design dirs, touches no other skill (4 file(s) changed)
- **dob-8** — install.sh --design-packs and install.ps1 -DesignPacks forward the design pick to onboarding, both print onboarding's notices after install, docs list the flag (6 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **dob-1** — `.bee/bin/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — re-run by the leader at worktree head 1fde4cd06: release manifest 562 files match; 40 test binaries, 4942 passed, 0 failed, 28 ignored; run with cargo already on PATH (host guard refuses the PATH exp…
- **dob-2** — `.bee/bin/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the cell verify run in full: release manifest 562 files match; workspace 4899 passed, 0 failed, 28 ignored; the four new tests ran and passed (drivers a_design_cell_pins_bee_design_and_every_other_ro…
- **dob-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — touched onboard skills, apply, plan, mod, notices, templates, source and tests; the full declared suite ran with no failure (bee bin 4417 passed, 24 ignored, every integration target ok); cargo was a…
- **dob-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the cell verify over the whole workspace, every target green (bee unit 4402 passed, 0 failed, 24 ignored; every integration target ok); run as plain cargo on PATH because the host guard refuses the C…
- **dob-5** — `.bee/bin/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the cell verify in full on the final tree: release-manifest --check matched 562 files, then the whole declared cargo suite exited 0 (4935 passed, 0 failed, 28 ignored; the ignored ones are the suite'…
- **dob-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — full declared suite: bee unit 4424 passed 0 failed 24 ignored, every integration and fleet target 0 failed; run with cargo already on PATH (host guard refuses $VAR expansion, so the PATH prefix was s…
- **dob-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — declared suite, run with cargo already on PATH (the same binary the PATH prefix names) and BEE_OPENCODE_SUITE_ALLOW_SKIP=1 because opencode on PATH is a wrapper on this machine: exit 0, 40 test binar…
- **dob-8** — `.bee/bin/bee dev release-manifest --check && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` — the cell's declared verify on commit 0997848f5: release manifest matches (562 files), full bee-rs suite 4942 passed, 0 failed, 28 ignored across 40 test binaries, including 4 new installer_contracts …

## Deviations

- **dob-1** — re-capped by the leader instead of a worker — the cell was reopened only by the NEEDS_REVISION authority verdict, which the owner resolved; no file changed — something else had to be fixed first
- **dob-2** — Pinned on tier_token (the cell's declared or approved-plan role, or an explicit --role) rather than marker_role — marker_role can only be design when tier_token is, and tier_token also covers an escalated or fallen-through design cell — found a better route
- **dob-2** — Made every_rendered_agent_is_reachable_through_the_one_door in prepare.rs also dispatch a design cell and deleted its comment that said the cell pin is never role-driven — adding bee-design to ROLE_AGENTS makes that test demand the door return it, and the comment became false — hit an unforeseen obstacle
- **dob-2** — Updated existing expectations: drivers/tests.rs pinned_types_match_the_rendered_bee_agents now expects agent_for_role(design) to be bee-design, templates.rs seeded-name list gains design, onboard/tests.rs pi table count 18 to 19, model_guard.rs repo_config and the migrated fixture carry design — each asserted the old table — hit an unforeseen obstacle
- **dob-2** — Left onboard/agents.rs, status_full/store.rs, status_full/tests.rs, cells/tests.rs and tests/opencode_plugin_contracts.rs untouched — they read AGENT_ROLES_BY_NAME or ROLE_AGENTS generically or use fixed fixtures and stayed green — found a better route
- **dob-2** — Reverted the .gitattributes BEE block and the .bee/onboarding.json rewrite that bee dev regen produced — both lie outside the cell's files and the gitattributes block is unrelated pending onboard drift — hit an unforeseen obstacle
- **dob-2** — Ran bee dev regen with the freshly built binary from the cargo target dir instead of .bee/bin/bee — the vendored binary does not know bee-design and would not render it — hit an unforeseen obstacle
- **dob-3** — The offer names the exact command (bee onboard --apply --design-packs with the chosen names, or none) — a cold agent cannot act on the offer without the flag, and the agent, never the user, runs bee commands — found a better route
- **dob-3** — The UI scan skips every hidden directory, not only .git, beside node_modules, target and dist, with a depth cap of 6 and an entry cap of 10000 — hidden trees such as .venv carry vendored html that would draw a false offer — found a better route
- **dob-3** — Only the sync_skill and refresh_legacy_global_skill arms changed to read the source recorded on each plan item; remove_skill never read skill_sync.source_root — the cell named remove_skill as a source reader — the plan was wrong about a fact
- **dob-3** — Updated three existing pi-table tests: team.claude now also gains design, two byte-identical fixtures carry a design row so the pi assertion stays isolated, and the ledger key list gains design_role_offered — the new design row legitimately changes those configs — hit an unforeseen obstacle
- **dob-3** — Three new tests never went red: the shipped design root pin, the offer wording test and the no-UI-file host guard — each pins a state that exists before the behavior, so no stub can fail it — hit an unforeseen obstacle
- **dob-4** — ran the verify as plain cargo with BEE_OPENCODE_SUITE_ALLOW_SKIP=1 instead of the PATH-prefixed spelling — the host guard refuses the CARGO_HOME expansion and opencode on PATH is a wrapper — hit an unforeseen obstacle
- **dob-5** — Trimmed the session-preamble command-surface header by 15 characters in packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs (reserved as an additional path under w-dob-5): 'everything at once' to 'all at once' and 'nearly every command also takes' to 'most commands also take'; the D2 read-once words are untouched — the new design group pushed the section to 2610 of its 2600-character budget, and decision 0cd42844 keeps the budget and trims wording instead — something else had to be fixed first
- **dob-5** — Pages, gate-baseline.json and last-run.json resolve under the checkout the gate runs in (StoreRoots.work_root), claimed cells under the main store (main_root), and config under the store root — the cell named no root for a linked worktree, and dob-6's design_rule reads last-run.json from both the cells root and the feature worktree, so this matches it — hit an unforeseen obstacle
- **dob-5** — The baseline version field records the pinned spec ease-design@0.6.1 instead of running the tool's --version as the field script does — one fewer external call, and npx runs exactly that pin — found a better route
- **dob-5** — bee dev regen also rewrote .bee/config.json (add_design_role) besides .gitattributes and .bee/onboarding.json; all three were reverted, none committed — something else had to be fixed first
- **dob-6** — finish_support.rs is listed but unchanged; design_rule reuses its commit_trailer_history_root as-is — no edit was needed — found a better route
- **dob-6** — last-run.json is read at the store root and at the feature worktree root — the cap runs on the main store while the gate may write in the worktree, and a missed not-run record would hide the gap owner answer 1 keeps visible — found a better route
- **dob-6** — tests.rs: cells_add_judge_obligation_run split so a new cells_add_child_output returns the child Output with stderr — the add warning is only visible on stderr — hit an unforeseen obstacle
- **dob-6** — two of the seven new tests (no-pick host unchanged, gate or non-UI cell accepted) passed at red by design: they pin the unchanged accept path; the other five failed first with expected a thrown refusal or a missing stderr warning — the plan was wrong about a fact
- **dob-6** — capped with --sync-ack for the workflow-state sync door — the cell names no skill file and plan.md sends the concept sync to capture — something else had to be fixed first
- **dob-6** — sync-ack: the cell's approved files name no skill (affects_skills is empty); plan.md routes the skill text for the design gate to dob-5 and the workflow-state concept sync to capture after the feature
- **dob-7** — the no-pick case rides as the first guard step of the pick test, not as its own red-first test — the base already writes no skill dir in plugin-first, so a no-pick test cannot fail first — the plan was wrong about a fact
- **dob-7** — compute_design_sync refuses one skill home that resolves outside the repo (per target, no aggregate block), with a unix unit test a_design_only_sync_refuses_a_skill_home_outside_the_repo — without it a symlinked .claude/skills would receive design packs outside the project, which repo-copy already refuses — found a better route
- **dob-7** — deleted the plan.rs comment sentence saying a --plugin-source run installs no skills at all — it is now false — something else had to be fixed first
- **dob-7** — deleted untracked packages/bee-rs/crates/bee/.bee/logs/timings.jsonl left by the full suite run (a dispatch authorize test writes timings under the crate cwd) so a sibling commit cannot sweep it; the test leak itself is not fixed here — something else had to be fixed first
- **dob-8** — install.sh's plan-failure message now carries onboard's stdout, so a refused --design-packs value names the allowed packs — before, a typo gave a bare 'Onboarding plan failed.' with the reason dropped — hit an unforeseen obstacle
- **dob-8** — Notices print from the apply output's notice: lines, not the plan JSON, so an up_to_date re-run prints none — the plan's notices are computed before the pick is recorded and would show the design offer even when --design-packs was given — found a better route
- **dob-8** — The install.sh end-to-end tests run from a sandbox source (symlinked tree, copied skills/, the test's own bee binary, a no-op cargo shim, a plugin-state fixture, a sandbox HOME) — install.sh has no seam for a prebuilt binary inside a checkout, and onboarding refuses a symlinked skills tree as blocked_no_source — hit an unforeseen obstacle
- **dob-8** — README's bee onboard synopsis and INSTALL's bee onboard flag table also list --design-packs — both onboard references lacked the flag the installers now pass through — found a better route

## Provenance

Proposed by `bee knowledge promote --work design-os-in-bee` from 8 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/design-os-in-bee/CONTEXT.md`, `docs/history/design-os-in-bee/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
