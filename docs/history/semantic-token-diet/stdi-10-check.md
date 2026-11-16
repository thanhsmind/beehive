# stdi-10 — check of stdi-9 (bee-hive moves)

Checked commit `4e5ae32d9` (trailer `cell: stdi-9`) against CONTEXT.md D2 and D4 and audit `reports/B-router.md`.
Line anchors point at the tree at `4e5ae32d9`. "old" means the parent `4e5ae32d9^`.

## Truths

- FAIL — every rule removed from the bee-hive body still has a home. Two cuts have no live home. Most removed rules do have one (see "Moved rules" below). First gap: the craft list cut from old `routing-and-contracts.md:255-268`. Second gap: two headless clauses cut from old `SKILL.md:101-106`. See "Defects".
- PASS — no do-not-touch line from `reports/B-router.md` section 4 was removed. Hat wave heading `gates-and-delegation.md:180` and seat table `:209-221`. `` `class` ∈ `` at `scout-and-ticks.md:42`. Onboarding three-step contract at `SKILL.md:71-74`. AskUserQuestion limits at `gates-and-delegation.md:60-65`. Approval vs information question at `gates-and-delegation.md:92`. Advisor-ref timing law at `gates-and-delegation.md:228-233`. "Do not auto-resume. Ever." at `routing-and-contracts.md:45` and the fresh-session boundary at `:47`. Leader-check payload and doors at `routing-and-contracts.md:139-140`. The explicit-user-word Discovery route at `SKILL.md:37-39`. The diff only stripped decision ids from lines near the seat table (`gates-and-delegation.md:210`).
- PASS — every heading cited from `packages/bee-rs` still resolves. "Hat wave" (`verbs/drivers/prepare.rs:209`, `verbs/drivers/tests.rs:1528`) → `gates-and-delegation.md:180`. "Gate bypass mode" (`verbs/workflow_store/record.rs:79`) → `gates-and-delegation.md:67`. "Re-lane checkpoint" (`verbs/state_group/workflows.rs:944`, `verbs/state_group/tests.rs:2305`) → `scout-and-ticks.md:59`. `` `class` ∈ `` (`tests/route_class_parity.rs:54`) → `scout-and-ticks.md:42`. The three headings the diff removed (Skill Catalog, First-Skill Routing, The agent runs the machinery), and Hard rules, have no hit in `packages/bee-rs` or `packages/bee`.

## Moved rules

- `SKILL.md` References row: "The default" became real triggers → `SKILL.md:95`, mirrored at `routing-and-contracts.md:3` (F1, F23).
- Hard rules, P1 blocks and never build on red → `packages/bee/LEADER.md:171-173`.
- Hard rules, locked decisions are cited and never reinterpreted → `LEADER.md:56-58`; edit boundary → `AGENTS.md:18-19`.
- Hard rules, the handoff rule (`agents-context-handoff-65`) → `SKILL.md:23`.
- Hard rules, leader completeness check → `routing-and-contracts.md:131-140`.
- Hard rules, done, green and fixed only beside fresh output → `AGENTS.md:28`; capture line at close → `LEADER.md:274-279`.
- Hard rules, the agent runs every bee command → `LEADER.md:48-51`.
- Hard rules, work language, one tick per step, a red is never silenced → `LEADER.md:303-320`.
- Hard rules, form rules bend out loud and boundary rules never bend → `LEADER.md:78`, `gates-and-delegation.md:364`.
- Hard rules, lanes scale ceremony, never memory → `LEADER.md:275`, `routing-and-contracts.md:145`.
- bee-swarming row: never zero execution workers, implementer plus a separate tester, inline execution refused → `LEADER.md:244-252`, `routing-and-contracts.md:129`.
- bee-shaping row: parking → `skills/bee-shaping/SKILL.md:111-116`; Qualify on request → `SKILL.md:47`, `bee-shaping/SKILL.md:93`.
- Gates prose: Gates 1-2 are the default chain and Gate 3 is additive → `gates-and-delegation.md:16-17`; `uat_stop` placement → `gates-and-delegation.md:25`, `LEADER.md:70`; gates belong to the user → `AGENTS.md:19`; plain-language layer with a linked report → `gates-and-delegation.md:50-52`; bypass is user-set and never headless → `SKILL.md:65`, `gates-and-delegation.md:73`.
- Headless: never ask, never self-approve, defer into Outstanding Questions → `SKILL.md:86`, `gates-and-delegation.md:104-110`.
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

- D-a: four craft rules cut from old `routing-and-contracts.md:255-268` have no live home. They are: give concrete units for anything over a minute; state a failure as cause, fix and who acts, quoting the shortest decisive line; file a tangent and mention it once at the close; protocol and record surfaces keep their ids. `rg` over `packages/bee/LEADER.md`, `skills/bee-hive` and `skills/bee-unslop` finds none of them. Audit F11 sanctioned cutting only the copies that LEADER.md already holds.
- D-b: two headless clauses cut from old `SKILL.md:101-106` have no live home. They are: each deferred question carries the evidence a later human pass would start from, never guessed or self-answered; and the run still makes recommendations where the evidence supports one, labeled with confidence. `gates-and-delegation.md:104-114` and `go-mode.md:129-131` lack both.
- D-c (minor): the clause "in doubt, serial" from old `gates-and-delegation.md:138` is gone. `routing-and-contracts.md:111-113` now says only a named conflict makes a cell wait, so the cut may be deliberate. It was not recorded as a departure.
- D-d (minor, why-clauses): two why-clauses were cut. One is the mark-the-wait why (old `routing-and-contracts.md:397`: an external reader sees "waiting on you" instead of "idle"). The other is the scout-and-ticks why (old `scout-and-ticks.md:180`: an every-turn rule cannot live only behind an on-demand reference). The rules themselves survive at `LEADER.md:323-327` and `LEADER.md:316-318`.

Fix route for D-a and D-b, not applied here: restore the clauses in `routing-and-contracts.md` "Communication contract" and `gates-and-delegation.md` "Headless mode".

## Verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test pointer_integrity --test route_class_parity`

- In a clean detached checkout of `4e5ae32d9`: pointer_integrity 10 passed, 0 failed; route_class_parity 2 passed, 0 failed.
- In the shared worktree as found: pointer_integrity 9 passed, 1 failed. All 13 broken pointers name `bee-planning/references/planning-reference.md`. Another cell's uncommitted work renamed that file in the working tree. The failure does not come from stdi-9.
