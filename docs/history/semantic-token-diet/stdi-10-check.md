# stdi-10 — check of stdi-9 (bee-hive moves)

Re-check after the restore. stdi-9 has two commits with trailer `cell: stdi-9`: `4e5ae32d9` (the trim) and `68f2b6abf` (the restore).
Checked both against CONTEXT.md D2 and D4 and audit `reports/B-router.md`.
Line anchors point at the tree at `68f2b6abf`. "old" means `4e5ae32d9^`.
The first check (commit `2abff7a03`) failed truth 1 on defects D-a and D-b. Both are now fixed.

## Truths

- PASS — every rule removed from the bee-hive body still has a home. The four craft rules (old D-a) are back at `routing-and-contracts.md:207`. The two headless clauses (old D-b) are back at `gates-and-delegation.md:108-110`. Every other removed rule has a home listed under "Moved rules".
- PASS — no do-not-touch line from `reports/B-router.md` section 4 was removed. Hat wave heading `gates-and-delegation.md:182` and seat table `:217-223`. `` `class` ∈ `` at `scout-and-ticks.md:42`. Onboarding three-step contract at `SKILL.md:71-74`. AskUserQuestion limits at `gates-and-delegation.md:56-65`. Approval vs information question at `gates-and-delegation.md:92`. Advisor-ref timing law at `gates-and-delegation.md:230-235`. "Do not auto-resume. Ever." at `routing-and-contracts.md:45` and the fresh-session boundary at `:47`. Leader-check payload and doors at `routing-and-contracts.md:139-140`. The explicit-user-word Discovery route at `SKILL.md:37-39`. The restore only added text inside two paragraphs.
- PASS — every heading cited from `packages/bee-rs` still resolves. "Hat wave" (`verbs/drivers/prepare.rs:209`, `verbs/drivers/tests.rs:1505`) → `gates-and-delegation.md:182`. "Gate bypass mode" (`verbs/workflow_store/record.rs:79-80`) → `gates-and-delegation.md:67`. "Re-lane checkpoint" (`verbs/state_group/tests.rs:2305`) → `scout-and-ticks.md:59`. `` `class` ∈ `` (`tests/route_class_parity.rs:54`) → `scout-and-ticks.md:42`. The three headings the trim removed (Skill Catalog, First-Skill Routing, The agent runs the machinery), and Hard rules, have no hit in `packages/bee-rs` or `packages/bee`.

## Moved rules

- `SKILL.md` References row: "The default" became real triggers → `SKILL.md:95`, mirrored at `routing-and-contracts.md:3` (F1, F23).
- Hard rules, P1 blocks and never build on red → `packages/bee/LEADER.md:171-173`.
- Hard rules, locked decisions are cited and never reinterpreted → `LEADER.md:56-58`; edit boundary → `AGENTS.md:18-19`.
- Hard rules, the handoff rule (`agents-context-handoff-65`) → `SKILL.md:23`.
- Hard rules, leader completeness check → `routing-and-contracts.md:131-140`.
- Hard rules, done, green and fixed only beside fresh output → `AGENTS.md:28`; capture line at close → `LEADER.md:274-279`.
- Hard rules, the agent runs every bee command → `LEADER.md:48-51`.
- Hard rules, work language, one tick per step, a red is never silenced → `LEADER.md:303-320`.
- Hard rules, form rules bend out loud and boundary rules never bend → `LEADER.md:78`, `gates-and-delegation.md:366`.
- Hard rules, lanes scale ceremony, never memory → `LEADER.md:275`, `routing-and-contracts.md:145`.
- bee-swarming row: never zero execution workers, implementer plus a separate tester, inline execution refused → `LEADER.md:244-252`, `routing-and-contracts.md:129`.
- bee-shaping row: parking → `skills/bee-shaping/SKILL.md:111-116`; Qualify on request → `SKILL.md:47`, `bee-shaping/SKILL.md:93`.
- Gates prose: Gates 1-2 are the default chain and Gate 3 is additive → `gates-and-delegation.md:16-17`; `uat_stop` placement → `gates-and-delegation.md:25`, `LEADER.md:70`; gates belong to the user → `AGENTS.md:19`; plain-language layer with a linked report → `gates-and-delegation.md:50-52`; bypass is user-set and never headless → `SKILL.md:65`, `gates-and-delegation.md:73`.
- Headless: never ask, never self-approve, defer into Outstanding Questions → `SKILL.md:86`, `gates-and-delegation.md:104-112`.
- Onboarding: orientation entry and the workspace protocol notice, condensed in place → `SKILL.md:77-78`.
- Skill Catalog and First-Skill Routing rows → `SKILL.md:44-60`. Research in a scoped feature → `SKILL.md:52`. Resume → `SKILL.md:56` and `routing-and-contracts.md:34-47`. Docs lane → `SKILL.md:57`, `routing-and-contracts.md:123-125`. Pure question → `SKILL.md:58`. Merge or ship with unreviewed work → `SKILL.md:51`, `skills/bee-reviewing/SKILL.md:18-21`. Brief renders nothing for tiny or spike → `skills/bee-shaping/references/implement-plan-template.md:156`. Surface-scope check → `SKILL.md:60`. Developing bee itself is maintainer work → `skills/bee-evolving/SKILL.md:40`.
- Five rules, pre-send check and litmus → `LEADER.md:329-332`; evidence before claims → `AGENTS.md:28`.
- Craft paragraph: bare repo-relative doc path and absolute worktree path → `LEADER.md:307-309`; ids and counts trail the work → `LEADER.md:319-320`; unslop and technical-writing routing → `routing-and-contracts.md:207-212`.
- Triage line → `LEADER.md:179`; claim labels → `LEADER.md:182-188`; source order → `LEADER.md:189-192`; first reaction → `LEADER.md:194`. The examples stay at `routing-and-contracts.md:245-271` (F8).
- The agent runs the machinery → `LEADER.md:48-51` (F10).
- Mark the wait → `LEADER.md:323-327`; turn-end kind, hook clearing and heartbeat expiry → `provenance.md:11` (F9).
- Helper CLI quick reference → `routing-and-contracts.md:347`; the `LEGACY_HELPER_RE` narration was cut as F12 allows.
- Transport, pick the team member by job → `LEADER.md:227-232`; marker refusal and guard rewrite → `packages/bee-rs/crates/bee/src/hooks/model_guard.rs:142`, `:343-351` (F14).
- Execution worker, proof checked and never re-run at the doors → `LEADER.md:167`, `routing-and-contracts.md:129`; machine-aware concurrency limit → `routing-and-contracts.md:109`; legal reasons for serial → `routing-and-contracts.md:113` (F13, F15).
- Stop-hook internals, both-fields pending and the loop guard → `hooks/session_close/nudges.rs:414-455`, `hooks/session_close/tests.rs:92` (F19).
- cli gather "Known measurement gap" → cut as history (F17).
- Herding branch, deployment, blind lanes, lane seats, hat-wave decision ids → `provenance.md:13-21` (F18).
- Gate renumbering history → `gates-and-delegation.md:36`, `provenance.md:6` (F16).
- Fixed gate wording in go-mode → `go-mode.md:62` points to `gates-and-delegation.md` "The three gates" (F20).
- Scout and ticks, the tick rule's home → `LEADER.md` "Communication", `scout-and-ticks.md:177` (F21); the dead "Session Scout section" pointer was cut and preamble-first → `LEADER.md:103-107`, `SKILL.md:22` (F22).

## Defects

- D-a: fixed. The four craft rules sit at `routing-and-contracts.md:207` under "Communication contract" (`:171`). The wording matches old `routing-and-contracts.md:256-264`. The other craft clauses stay at their LEADER.md homes, as F11 allows.
- D-b: fixed. Both headless clauses sit at `gates-and-delegation.md:108-110` under "Headless mode" (`:104`). The wording matches old `SKILL.md:102-103` and `:106`.
- D-c (minor, still open): the clause "in doubt, serial" from old `gates-and-delegation.md:138` is gone. `routing-and-contracts.md:111-113` now says only a named conflict makes a cell wait, so the cut may be deliberate. It was not recorded as a departure.
- D-d (minor, still open, why-clauses): the mark-the-wait why (old `routing-and-contracts.md:397`) and the scout-and-ticks why (old `scout-and-ticks.md:180`) are still cut. The rules survive at `LEADER.md:323-327` and `LEADER.md:316-318`.

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test route_class_parity`

- In the worktree at `68f2b6abf`, clean: pointer_integrity 10 passed, 0 failed; route_class_parity 2 passed, 0 failed.
- The red pointer test from the first check is gone. Commit `7c19ea731` landed the planning-reference split and fixed the `scout-and-ticks.md:44` and `routing-and-contracts.md:129` pointers.
