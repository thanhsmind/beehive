# Token audit — ROUTER branch (bee-hive)

Read-only. Paths read: `.bee/tmp/token-audit/criteria.md`, `.bee/expertise/review.md` (to "Where to look"), `AGENTS.md`, `skills/bee-hive/SKILL.md`, all five `skills/bee-hive/references/*.md`, `packages/bee/LEADER.md`, `packages/bee-rs/crates/bee/src/generated/registry_payload.json`, plus the test files named below.

## 1. Load map (`wc -c`)

| File | Bytes | When |
|---|---|---|
| `packages/bee/LEADER.md` (= `.bee/LEADER.md`) | 27,455 | always-loaded, every leader session |
| `skills/bee-hive/SKILL.md` | 9,704 | prepaid per invoke |
| `references/routing-and-contracts.md` | 42,845 | **prepaid in practice**: `SKILL.md:125` labels it "The default" in the References table |
| `references/gates-and-delegation.md` | 52,209 | on-branch: any gate, bypass, fan-out (so nearly every write task reaches it) |
| `references/scout-and-ticks.md` | 25,971 | on-branch: read budget, re-lane, tick wording |
| `references/onboarding.md` | 11,043 | on-branch, rare (`SKILL.md:128`) |
| `references/go-mode.md` | 12,756 | on-branch, `/go` only |

What the body forces on every run: only `routing-and-contracts.md`, via the word "default" (`SKILL.md:125`). Every other reference has a conditional trigger. So one bee-hive invoke actually costs about 52.5 KB (9.7 + 42.8), on top of the 27.5 KB of LEADER.md that is already loaded.

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite or move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| F1 | 4 | `SKILL.md:125` | names `routing-and-contracts.md` "The default" | Give that row real triggers instead (resume or handoff, lane ceremony, completeness check, communication or question format) and drop "default". This turns 42.8 KB prepaid into on-branch | ~42,800 per invoke that doesn't need it | med: content agents need at resume time must stay reachable through those triggers |
| F2 | 1 | `SKILL.md:109-116` | the Hard rules list | Every bullet restates always-loaded text: 111 = `LEADER.md:191-198`; 114 = `LEADER.md:224-225,344-349` and `AGENTS.md`; 115 = `LEADER.md:45-47,373-391`; 116 = `LEADER.md:78-86,345`; 113 repeats `SKILL.md:48` in the same body. Delete the section. Keep only the `agents-context-handoff-65` clause from 112, which LEADER.md does not carry, as one line under Start | ~1,000 | low: every rule keeps an always-loaded home |
| F3 | 1 | `SKILL.md:43-54`, `routing-and-contracts.md:5-23`, `:38-60`, `LEADER.md:131-152` | the skill catalog has four homes | `SKILL.md`'s table owns it. Delete the Skill Catalog (`r&c:5-23`). Fold the few First-Skill rows the body lacks (surface-scope check `:60`, merge-with-unreviewed `:58`, gate-bypass toggle `:53`) into the body table, then delete `r&c:38-59` | ~4,500 on-branch | med: `:46-47` (D1/D6 brief rule) has to survive somewhere, e.g. bee-shaping |
| F4 | 1 | `SKILL.md:48` | the bee-swarming row restates never-zero-execution-workers, the inline refusal and the completeness check | Cut it to "Merged gate approved, cells open — orchestrate or execute one cell in the feature worktree". The rule already lives at `LEADER.md:273-283` | ~450 | low |
| F5 | 1 | `SKILL.md:58-76` | three gates, uat placement, bypass levels, bypass ≠ headless | uat placement repeats `LEADER.md:63-68` and `g&d:20-34`. "Gates belong to the user" repeats `LEADER.md:41-47`. Cut to 3 lines: gate names, "uat is never auto-approved", and a pointer to `g&d` "Gate bypass mode" | ~900 | low: the level table already has one home (`g&d:81-86`) |
| F6 | 4 | `SKILL.md:98-107` | the Headless section | Rare branch. bypass ≠ headless is stated 4 times (`SKILL.md:73-74,103-105`, `g&d:75,118-119`). Replace with one line: "Headless: never ask, never self-approve → `g&d` 'Headless mode'" | ~520 | low |
| F7 | 4 | `SKILL.md:87-92` | orientation-entry debt and the Workspace protocol procedure | These fire only after an onboard. Move them into `onboarding.md`. Keep one pointer line, because the `workspace protocol:` notice may show on an otherwise up_to_date session [unverified] | ~400 | med (see unverified) |
| F8 | 1 | `r&c:297-313` | Triage line, claim labels, source order, first reaction | `r&c:301,311,313` restate `LEADER.md:200-213` almost word for word. Keep only what LEADER.md lacks: the examples (`:303-309,315-330`) and the four-part pushback (`:334-341`) | ~1,300 | low |
| F9 | 1 | `r&c:390-403` | "Mark the wait" | Near-verbatim copy of `LEADER.md:393-402`, with D1–D4 provenance on top. Delete it; LEADER.md owns it | ~900 | low |
| F10 | 1+7 | `r&c:360-366` | "The agent runs the machinery" | Restates `LEADER.md:45-47`. The text claims "`AGENTS.md` states the same law", but AGENTS.md now holds only the BEE block. Delete the section, together with the pointer to it at `SKILL.md:115` (gone anyway under F2) | ~550 | low |
| F11 | 1 | `r&c:240-243,255-258,343-358` | pre-send check, evidence before claims, the doc-path rule, the work-language litmus | Already said at `LEADER.md:377-379,404-407,224-225`. Keep the turn-shape example (`:271-286`) and the catch-me-up shape (`:288-295`). Cut the duplicated sentences | ~1,200 | low |
| F12 | 2 | `r&c:437-448`, `LEADER.md:71-74` | Helper CLI quick reference | Echoes `bee --help`, plus `LEGACY_HELPER_RE` transition narration (the regex still exists in `hooks/cli_shape.rs`, so the narration is not stale, but it changes no agent action). Replace with one line: "`bee` = `.bee/bin/bee`; `bee <cmd> --help` is the reference" | ~550 | low |
| F13 | 1 | `r&c:153` vs `g&d:138` | concurrency limit sentence ("reservations are the proof and the police … never a fixed count") | Copied verbatim in both places. Keep it in `r&c` "Concurrency law"; `g&d:138` points there | ~350 | low |
| F14 | 1 | `g&d:137` | Transport bullet (2.5 KB) | The "pick the team member by job … config, never the leader's choice" sentence repeats `LEADER.md:256-258`. The guard rewrite/deny details are what the model-guard hook prints (class 2). Keep the role and transport mapping; cut the hook-behavior narration | ~1,000 | med: `--role` refusal semantics are worth keeping |
| F15 | 1 | `g&d:138` | Execution worker bullet (2.7 KB) | Restates `LEADER.md:273-283` (implementer + tester, never inline) and the `bee route --lane tiny` refusal also stated at `r&c:173`. Keep the class definition (authority, registry, reservations, status tokens) | ~900 | low |
| F16 | 3 | `g&d:36-42` | "Reading older records" (gate renumbering history) | Shrink to one line: "Pre-validation-diet records: old Gate 4 = today's Gate 3; 'execution gate' = merged Gate 2" | ~300 | low |
| F17 | 3/7 | `g&d:139` | "Known measurement gap … Slice 3's job" | History, and probably stale: `prepare.rs:1576-1590` (`append_prepare_record`) writes a `dispatch.jsonl` row for every prepare [unverified whether the cli-gather branch hits it] | ~160 | low |
| F18 | 3 | `g&d:150-151,205,213,221,229,233,248` | decision ids and "used to draw … now superseded" in live text | Move them to `references/provenance.md` (provenance-exile convention) | ~600 | low |
| F19 | 2/4 | `g&d:108` | how the stop hook mechanizes bypass (loop-guard key) | Hook internals; the agent's action is unchanged. Cut to one sentence: "The Stop hook forces continuation at a covered pending gate and prescribes the merged `--merge` approval" | ~650 | low |
| F20 | 1 | `go-mode.md:60-64` vs `g&d:10-14` | fixed gate wording, two homes | Keep `g&d`; go-mode points there. No test pins the strings (rg over packages/scripts: no hit) | ~400 | low |
| F21 | 7 | `scout-and-ticks.md:177-181,229-230` | "AGENTS.md's Communication section states the rule bare … switches stated as rule in AGENTS.md" | False now. AGENTS.md holds no Communication section, and `rg 'quiet\|ship_visibility' packages/bee/LEADER.md AGENTS.md` returns nothing. Repoint the tick rule to `LEADER.md` "Communication" and make this file the home of the two switches | ~250 (and fixes a lie) | low |
| F22 | 7 | `scout-and-ticks.md:240-241` | "The router's Session Scout section keeps the invariants" | No such section exists (`rg 'Session Scout' skills/bee-hive` hits only this file). The preamble-first rule (`:243-247`) restates `LEADER.md:104-107` and `SKILL.md:20-22`. Delete `:240-241`; shrink `:245-247` to the "re-run `bee status --json` before routing" trigger | ~700 | low |
| F23 | 7 | `routing-and-contracts.md:3`, `SKILL.md:67` | "compact bootstrap in SKILL.md"; "Gates belong to the user (AGENTS.md)" | AGENTS.md still says this (BEE block), so `:67` is fine. `:3` contradicts "The default"; fix it together with F1 | ~0 | low |

Stale-verb sweep: every verb the branch names exists in `registry_payload.json` (237 commands): `cells leader-check`, `state handoff adopt`, `state advisor-ref record`, `feature start`, `knowledge context`, `onboard`, `state waiting-on set`, `cells schedule`, `cells judge-record`, `cells reroute`, `staging add`, `state worker add`, and others. Constants exist too: `COMMIT_TRAILER_WINDOW`, `LEADER_CHECK_DOOR_INTRODUCED_AT`, `DEFAULT_TAIL_MAX_BYTES`, `SEAT_ROLES`, `.inject-cache.json`. The only dead text is the pointers in F10, F21 and F22. `g&d:255` cites `advisor_ref.rs:166-180` as the place the anchors are stamped, but those lines are the staleness *comparison*. That line anchor has drifted [low].

## 3. Top 3 by saved tokens × frequency

1. F1: retrigger `routing-and-contracts.md` away from "The default". About 42.8 KB off most bee-hive invokes.
2. F2+F5+F6: strip the SKILL.md sections that repeat LEADER.md (Hard rules, the Gates prose, Headless). About 2.4 KB on every invoke, all already in always-loaded text.
3. F3+F8+F9+F11: collapse the four-home skill catalog and the communication and claim-label copies in `routing-and-contracts.md`. About 7.9 KB from whichever reference stays the hot one.

## 4. Do not touch (class 8)

- `g&d` "Hat wave" heading and its seat table (`:201`, `:238-244`). `verbs/drivers/tests.rs:1507` pins "references/gates-and-delegation.md, section \"Hat wave\"" and agents are sent to "the row for this role".
- `scout-and-ticks.md:42`: the "`class` ∈" line is pinned by `tests/route_class_parity.rs:54`.
- `SKILL.md:80-83`: the three-step onboarding contract (`changes_needed` → approval → `--apply`, never silently). Compact, and it guards a write.
- `g&d:64-71`: the AskUserQuestion schema limits ("header ≤ 12 characters … #1 cause of the error"). This rule exists because agents kept violating it.
- `g&d:98`: approval vs information question under bypass. The why-clause does real work (D6 of agents-block-diet).
- `g&d:251-256`: the advisor-ref timing law. A ref recorded too early refuses its own gate, so the why is load-bearing.
- `r&c:89` "Do not auto-resume. Ever." and `r&c:91-95` (planned-next adoption fires only at a fresh-session boundary). Safety rule.
- `r&c:183-184`: the leader-check payload rules and doors. They mirror the refusals of a real verb.
- `SKILL.md:36-38`: an explicit user word routes straight to Discovery. A compact routing rule found nowhere in LEADER.md.

Test pins checked: `rg -F` over `packages scripts` (non-md) for the gate-wording strings, "Lanes scale ceremony, never memory", "Do not auto-resume" and "Every heading quoted in this body" found no test pin. `knowledge_first_nudge.rs:128` and `cells/tests.rs:1108` only use `skills/bee-hive/SKILL.md` as a path, not its wording.

## Proof run

The brief's proof command (`cargo clippy … -- -D warnings && cargo test --release --no-fail-fast …`) exited 0. Clippy was clean. The tests had 49 `test result: ok` lines, totalling passed=5918, failed=0, ignored=30. The release test build printed compiler warnings (duplicated attribute, unused `mut`, unreachable `else`, unused variable `root`) in targets that clippy's default scope does not cover. They have nothing to do with this audit, which changed no files.
