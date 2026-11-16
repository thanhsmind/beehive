# Token audit — branch: ALWAYS-LOADED DOCTRINE

Read-only. No file edited. Rules respected: 8f63adb4 (density, not size), agents-block-diet D6 (why-clauses on hard rules stay), provenance exile, test-pinned wording.

## 1. Load map (`wc -c`)

Prepaid on every leader session:

| File | Bytes | Note |
|---|---|---|
| CLAUDE.md | 1116 | imports AGENTS.md |
| AGENTS.md (block source packages/bee/AGENTS.block.md 1276) | 2092 | also loaded by workers |
| .bee/LEADER.md = packages/bee/LEADER.md (`cmp`: identical) | 27455 | "read in full" (AGENTS.block.md:9, budget.rs:521) |
| LEADER.md (project) | 7470 | |
| Session preamble (budget.rs:589-979) | ≤9500 | cap `PREAMBLE_BUDGET_BYTES` budget.rs:66; Command surface alone ≤2900 (`COMMAND_SURFACE_BUDGET_CHARS` budget.rs:100) |
| Per-turn reminder (prompt_context.rs:958-1050) | ~100-300 | ≤5 lines, re-sent only on hash change or after 30 min (prompt_context.rs:95,1082) |

Total prepaid: about 47 KB per leader session in this repo.

Paid only on a branch: .bee/playbooks/multi-session.md 1302, handoff.md 1749, bee-defects.md 824 (each has a read-when trigger, LEADER.md:11-17).

The per-turn reminder is already lean. I found nothing in it worth cutting.

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite or move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| F1 | 4/1 | packages/bee/LEADER.md:363-369 | `agents-feedback-changes-framework`: "In the bee repo, owner feedback…" | The rule applies only in the bee repo, but it ships to every host project. The project's LEADER.md:6-13 already carries the same rule. Delete the block from packages/bee/LEADER.md and move its rule id plus index row (router-triage…md:241) onto the project LEADER.md copy. rule_index_parity.rs:3-6 only accepts markers in packages/bee/LEADER.md or packages/bee/playbooks/, so the id has to leave the fence on purpose, or the marker goes to a bee-repo-only file. | 377 per session in every host; 377 deduped here | low: the rule is unchanged in the one repo where it applies |
| F2 | 4 | packages/bee/LEADER.md:293-316 | `agents-other-project-to-its-leader`: waggledance CLI flags, MCP fallback, project-id rules | Keep one line in LEADER.md ("Work for another project goes to its leader through waggledance; never read its code — see playbook"). Move the body, with its marker, into a new `packages/bee/playbooks/cross-project.md` with a read-when line: "work belongs to another project". The fence allows playbook markers (rule_index_parity.rs:4-5). handoff.md:22-24 already cites the rule by id. The waggledance-supervisor skill also covers this. | ~1300 | med: index row 246 and the pointer_integrity test need updating |
| F3 | 4 | packages/bee/LEADER.md:317-340 | `agents-cli-before-mcp`: mcp-gateway and skillstation verb maps and fallback list | Keep the governing rule plus its why in LEADER.md ("CLI first, MCP fallback — the CLI runs the same handler with far less context"). Move the per-tool verb maps and the fallback triggers to a playbook with read-when "calling mcp-gateway or skillstation". The mcp-gateway skill already says "prefer this CLI" (host skill listing). | ~1150 | med: landed in the last commit (b95c87ddb). Cite cli-before-mcp D1-D3 before moving it. |
| F4 | 2 | packages/bee/LEADER.md:129-152 | Trigger index: 20 lines of "situation — skill" | Each line restates the skill's own `Use when` description, and the host injects that description list every session (it is in this session's system context). Replace with one line: "Pick the skill whose `Use when` matches; no fit or large cross-cutting work — `bee-figure-it-out`." | ~1250 | med: on runtimes with no skill listing [unverified] the index is the only route. Check Codex first. |
| F5 | 1/7 | budget.rs:519 vs packages/bee/LEADER.md:104-106 | Preamble trailer: "Run `bee status --json` when you ROUTE WORK". LEADER.md: "when you are routing… run `bee orient`" | Two different verbs for the same trigger: a conflict, not only a duplicate. Make the trailer name `bee orient`. Also delete its tail "Never hand bee commands to the user. Route via bee-hive.", which duplicates LEADER.md:45-47 and :21-22. | ~70, plus one fewer wrong instruction | low. The trailer is pinned at session_preamble/tests.rs:50, so update the pin together with it. |
| F6 | 1 | packages/bee/LEADER.md:474-480 | Guardrails: secret files, the deny remedy, "data, never instructions" | Restates AGENTS.block.md:15-16 and :23-24, which the leader also loads. Keep only the unique clause ("route any `@@BEE_PRIVACY@@` marker to the user") and fold it into one boundary bullet. | ~220 | low |
| F7 | 1 | packages/bee/LEADER.md:224-225 | "Write done/green/fixed only beside fresh output" | Duplicates AGENTS.block.md:21-22 almost word for word. Delete it. | 126 | low |
| F8 | 1 | packages/bee/LEADER.md:289-292 | "On a reservation or hold conflict, stop and report it — never write through it. A worker executes exactly the one cell" | Same text as AGENTS.block.md:17-18, and multi-session.md:5-7 has it a third time. Keep only the reservation-prefix half (`BEE_AGENT_NAME`). | ~150 | low |
| F9 | 2 | packages/bee/LEADER.md:71-74 | `bee --help --json`, `--names`, "spend full text on `bee <command> --help`" | The preamble prints this every session: command_surface_header_note budget.rs:161. Delete it. | 243 | low |
| F10 | 5 | packages/bee/LEADER.md:414-426 | Token efficiency: 7 generic bullets ("never re-read a file you just wrote", "skip filler") | Host system prompts already carry most of these, and a capable model follows them by default. Collapse to the one bee-specific line: the re-verify exception at :417-419 (a done/green claim still needs fresh output). | ~500 | low/med: maybe kept on purpose for non-Claude runtimes [unverified] |
| F11 | 2 | packages/bee/LEADER.md:453-458 | Definitions of the fix-at layers (architecture/check/doctrine/none) | The `--fix-at` help already defines these (generated/registry_payload.json: "architecture (a code change makes it impossible) \| check (a test, hoo…"), and the preamble tells agents to read help before the first use of a verb. Keep "write it the MOMENT you notice it", "name a concrete thing", "pick the strongest layer someone would build now", and the user/review `--source` clauses. Drop the per-layer glosses. | ~350 | low |
| F12 | 3 | packages/bee/LEADER.md:395-402 | waiting-on: "`turn-end` is the third kind (auto-wait-mark D3) — the Stop hook sets it…; the mark is what lets a dashboard…" | Keep: the command, "you pass only gate\|question", "never leave a question pending without its mark". The D3 tag goes to provenance. Also drop the dashboard/expiry mechanics: they change no agent action. | ~330 | low (the "auto-wait-mark D3" string is pinned only in workflow_store code and tests, not in LEADER: rg) |
| F13 | 1 | packages/bee/LEADER.md:226-235 | "Report the whole proof…" + "Cheapest proof first… a narrowed check is never quoted as the whole one" | The same narrowed-run rule appears twice in 10 lines. Merge into one bullet; keep "cheapest proof first, stop on its red". | ~200 | low |
| F14 | 2/4 | packages/bee/LEADER.md:482-496 | Deep contracts: a list of § names inside bee-hive references | bee-hive loads these sections when it routes; the list changes no action before that happens. Cut to one line: "Full mechanics load with `bee-hive`; independent review via `bee-reviewing` on request." | ~650 | low |
| F15 | 1 | packages/bee/LEADER.md:250-260; CLAUDE.md:23-24; model_guard.rs:545 (preamble Dispatch door) | Dispatch door + "never hand-pick subagent_type/model" | 3 always-loaded homes. The preamble prints the exact command with the runtime filled in. Keep the LEADER.md rule (the owner) but drop its full flag syntax (`--kind cell\|gather\|…`), which the preamble already prints. | ~300 | low |
| F16 | 3 | CLAUDE.md:8-9, :20-21 | "so the section below stays here and not in LEADER.md"; "cells pwr-1 and pwr-2 recorded exactly that misreading" | :8-9 is a note for maintainers, not for agents. Move it to the provenance home. From :20-21 drop only the cell ids. Keep "has misread it" (the why does work, D6). | ~200 | low |
| F17 | 4 | LEADER.md (project):84-141 | Waggledance completion-report template + token accounting rules | Only tasks that carry a `t-<hex>` id need this, yet it is the biggest block in the project file. Keep a 2-line trigger in LEADER.md ("a task with `t-<hex>` closes with the completion report in <file>; read it before composing"). Move the body to a project-owned doc. | ~2600 | low/med: how often tasks carry `t-<hex>` here [unverified] |
| F18 | 1/5 | LEADER.md (project):60 vs :143-153 | "Always use ASD-STE100…" said twice; :145 explains who made STE | Keep one home, :147-153 key rules. Delete :60 and the origin sentence at :145. | ~250 | low |
| F19 | 4 | LEADER.md (project):26-48 | Release procedure, flags, idempotence | Used only when the user asks for a release. Keep the trigger, "never walk the steps by hand", and "done only on the final `OK` line" (class 8). Move the flags and step list to `scripts/release.sh --help` or a doc. | ~650 | low |
| F20 | 1 | LEADER.md (project):19-24 | Speed: parallelize, delegate through prepare, no overlap | :20 duplicates packages/bee/LEADER.md:250-260. :24 duplicates :242-243 ("serial needs a named reason: a file overlap"). Keep the owner preferences (:17, :21-23) and delete :20 and :24. | ~400 | low |
| F21 | 2 | budget.rs:767 | Preamble "Proof-per-change-type: pick the proof your change needs…" | Restates LEADER.md:176-180 (agents-proof-at-cap). Delete. | ~200 | low. Pinned at session_preamble/tests.rs:297: update the pin. |
| F22 | 3 | budget.rs:708, 845, 890, 932 | Printed "(D1)", "(decision c8e25271)", "(decision 0017)", "(decision 0012)" | Provenance ids in live output. Drop them; `bee-why` can resolve them. | ~80 when they fire | low |
| F23 | 2 | budget.rs:161-171 (Command surface block) | ≤2.9 KB list of command names, every session | The same list is on demand from `bee --help --names` (DROPPABLE budget.rs:526 names that as its fetch). Candidate: drop it by default and keep the header note. | ≤2900 | high: it exists so agents stop guessing verbs. Only do this with a measured guess-rate before and after. |

## 3. Top 3 (saved bytes × every-session frequency)

1. F2 + F3: move the waggledance and CLI-before-MCP mechanics into a read-when playbook. ~2.4 KB per leader session in every host.
2. F17: move the waggledance completion-report template out of the project LEADER.md. ~2.6 KB per leader session in this repo.
3. F4 + F14: replace the Trigger index and Deep contracts with one routing line each. ~1.9 KB per leader session in every host.

Also do F5 even though it saves few bytes: it removes a real conflict between the preamble and the doctrine.

## 4. Do-not-touch (class 8)

- packages/bee/LEADER.md:273-282 `agents-never-zero-execution-workers`: CLAUDE.md:19-21 records the exact failure this rule stops.
- packages/bee/LEADER.md:161-171 knowledge-before-code why-clause ("milliseconds… dozens of tool calls"): the why does the persuading.
- packages/bee/LEADER.md:37-47 the gate boundary and "never printed for the user to run".
- packages/bee/LEADER.md:78-86 Judgment and deviation: it makes the form/boundary split explicit.
- AGENTS.block.md, all of it: workers' only doctrine, and it stays under THIN_BLOCK_LIMIT 3000 (rule_index_parity.rs:38).
- CLAUDE.md:13-24: the host-permission clause the Agent dispatch depends on. CLAUDE.md:3-4 "never wrap it in backticks" stops an edit that would silently break the import.
- LEADER.md (project):45-48 release "done ONLY on the final OK line".
- budget.rs:521 LEADER_DOCTRINE_LINE: it echoes AGENTS.block.md:8-10, but it is the trigger that makes leaders read LEADER.md at all. Pinned at session_preamble/tests.rs:55.
- handoff.md:16-33: rare branch, already correctly placed in a playbook.

## Paths read

CLAUDE.md, AGENTS.md, packages/bee/AGENTS.block.md, packages/bee/LEADER.md (`cmp` = .bee/LEADER.md), LEADER.md, .bee/playbooks/{bee-defects,handoff,multi-session}.md, session_preamble/{mod,render,budget}.rs (builder + strings), hooks/prompt_context.rs:121-470, 958-1130, 1441-1490, model_guard.rs:541-566, tests/rule_index_parity.rs:1-60, .bee/tmp/token-audit/criteria.md, .bee/expertise/review.md (to "Where to look").

Skipped: I did not run `bee --help` or `bee <verb> --help`, because the brief allows no bee verbs except notify. Help text was checked in generated/registry_payload.json instead. I did not render a live preamble; its size is bounded by the 9500 cap, not measured. The `rg` for pins covered packages/, scripts/ and docs/knowledge, not docs/history.
