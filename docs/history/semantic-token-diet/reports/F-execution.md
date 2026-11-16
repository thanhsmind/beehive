# Token audit: EXECUTION branch (bee-swarming, agents/*.tmpl, prompts/*.md)

Read-only. No files edited. Lens: `.bee/tmp/token-audit/criteria.md`.

Paths read: skills/bee-swarming/SKILL.md (whole); references/swarming-reference.md (headings, 5-20, 601-760); references/worker-details.md (1-125); packages/bee/agents/*.tmpl (all 6); packages/bee/prompts/{worker-cell,peer-boot,advisor,gather,reviewer}.md; packages/bee/LEADER.md (258-302); packages/bee/playbooks/handoff.md (rg); drivers/tests.rs:10861-10990; generated/registry_payload.json (CLI help, read with python -I).

Skipped: `.bee/bin/bee <verb> --help`. The worker-guard hook refused it ("`bee dispatch` is not allowed for a Paseo worker"), so I read the same help text from `packages/bee-rs/crates/bee/src/generated/registry_payload.json`.

## 1. Load map (`wc -c`)

| When | File | Bytes |
|---|---|---|
| Always (each Claude leader: Agent-tool listing) | the 6 agent `description:` lines | 3,127 total (comb 559, elder 158, forager 633, guard 656, scout 632, worker 489) |
| Per orchestrator invoke | skills/bee-swarming/SKILL.md | 20,053 |
| **Per native worker** | bee-worker body (2,104) + rendered worker-cell.md (golden render ≈6,386 + cell JSON) + **SKILL.md 20,053** (prompt line 74 and agent body line 10 both say "Load the bee-swarming skill") + AGENTS.md 2,092 | **≈30.6 KB per worker** |
| Per herding worker | peer-boot.md 836 + herding variant of worker-cell.md (no skill-load line; tests.rs:10983 asserts its absence) + the mailbox wrapper (Rust-built, outside this branch) | — |
| Per guard, forager, scout or elder dispatch | agent body + reviewer/gather/advisor.md | 0.4–2.0 KB |
| On branch | references/swarming-reference.md | 63,837 |
| On branch | references/worker-details.md | 21,508 |

SKILL.md sections by byte count: front matter + intro 823 · **Orchestrate 21-170 = 10,744** · Execute 171-262 = 5,860 · Tester 263-281 = 981 · Hard rules to end 1,645. A native implementer acts on none of Orchestrate and none of Tester. That is **≈11.7 KB per worker that changes no worker action**, multiplied by the wave size.

Likely prepaid in practice: SKILL.md:239-242 sends the *worker* to swarming-reference.md "Result Formats". A Read of that file pulls all 64 KB (830 lines, under the 2,000-line default). [unverified: I did not check worker transcripts to see whether workers actually open it.]

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite or move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| F1 | 4 | worker-cell.md:74; bee-worker.md.tmpl:10; bee-comb.md.tmpl:10 | "Load the bee-swarming skill (Execute section) for the full worker contract" | Move SKILL.md "Execute (worker)" (171-262) and "Tester contract" into `references/execute.md`, leaving one pointer line in SKILL.md. Point the prompt line at that file ("Read skills/bee-swarming/references/execute.md"). Drop the agent-body copy of the instruction. | ≈11.7 KB × each native worker; ≈6.8 KB off every orchestrator invoke | **med**: GOLDEN_NATIVE_CELL_PROMPT (drivers/tests.rs:10861, asserted at :10957) pins the line byte-for-byte; the herding test (:10983) asserts the old string is absent. The skill-trigger fixture `tests/fixtures/skill-triggers.json:97` expects bee-swarming for "You are the worker for this one cell id", so keep that clause in the description. |
| F2 | 1 | SKILL.md:69-73 **and** SKILL.md:289 (same sentence twice in one file); LEADER.md:261-263 | "Dispatches must follow the cell's recorded role … `role-reroute` … `bee cells reroute`" | Cut both SKILL copies. LEADER.md (always loaded) owns the rule, and `bee cells reroute` help already requires the tagged decision (registry_payload.json). | ≈700 per invoke | low: `rg -F "Dispatches must follow the cell's recorded role" packages` finds no test hit |
| F3 | 1 + 7 | worker-cell.md:80; bee-worker.md.tmpl:14; bee-comb.md.tmpl:14 | "Shape what you leave behind: prefer deletion … (`skills/bee-swarming/references/worker-details.md`)" | Keep it in worker-cell.md only, since herding workers get it too, and delete it from both agent bodies (a native worker reads it twice now). Drop the citation: `rg -i 'prefer deletion\|smallest diff\|simpler than\|threaded' references/` gives 0 hits, and worker-details.md:40's only "craft" line is about red-before-green. The citation points to a 21 KB file that does not hold the rule. | ≈370 + 60 per native worker | med: the golden render pins worker-cell.md:80, so update it in the same change |
| F4 | 1 | bee-worker.md.tmpl:15,17; worker-cell.md:86,88,99; SKILL.md:235-238,256-261 | commit trailer `cell: <id>`, the 5 status tokens, the finish verb and its "runs NO tests" note | The rendered prompt is the one home. Cut the agent body to "Follow your task prompt's Contract; it carries the commit, finish and status rules" and drop the matching SKILL Execute copies (or keep them only in F1's reference). | ≈600 per native worker | low on the agent bodies (no test pins found); med on worker-cell (golden) |
| F5 | 1 + 7 | swarming-reference.md:652 claims "This section is the one home of the Result form"; worker-cell.md:90-97 holds the live form; SKILL.md:242 routes workers to the reference | The reference form (:656) has 2-part `mistakes` and no `knowledge_used`/`knowledge_wrong`/`baseline`; `rg PARKED swarming-reference.md` gives 0 hits. The prompt form has 3-part `mistakes` ("— <fix-at>") and the knowledge keys. | Make worker-cell.md the named home. SKILL.md:242 says "the Result form in your dispatch prompt", and the reference keeps only the orchestrator-side reading (the partial stop, dissent once, the four layers) with a pointer. | Avoids a possible 64 KB read per worker [unverified frequency]; ≈1.2 KB of duplicated schema | med: this is real drift between two forms, so pick the true schema against `cells finish --report` validation first |
| F6 | 7 | swarming-reference.md:601-648 "Worker Prompt Template" | A hand copy of the prompt. "Nicknames are Minions character names" is stale: `dispatch wave` help says auto `w-<cell id>`, and `rg Minion packages/bee-rs/crates/bee/src` gives 0. Its token list omits [PARKED]. It says "never re-run `status --brief`". | Replace with one line: "The rendered template is packages/bee/prompts/worker-cell.md; `bee dispatch prepare` fills it." | ≈3 KB on branch | low: no test pins "Worker Prompt Template" or "Minions" |
| F7 | 2 | SKILL.md:48-62 | Restates the help text for `bee dispatch wave`: claim+reserve+payload, typed `skipped` reasons, feature resolution, `--limit`, the `{ran,native,skipped}` value, `.bee/workers/<id>-wave-run.json` | Keep only what the agent must do: "launch only `native` items yourself, then cap each; a herding cell with a `cell:` commit and green proof is capped by `bee herding run`, otherwise `bee cells finish --from-job`." Help covers the rest. | ≈700 per invoke | low |
| F8 | 2 | SKILL.md:243-252; worker-details.md:11-21 | "`bee close` and `bee worktree merge` check that recorded proof … CI runs the full declared command on every push" | Echoes the help for `bee cells finish` ("No door runs tests … CI runs the full declared commands.test"). Keep the tag definitions in the prompt (worker-cell.md:90-93) and cut the narration in SKILL. | ≈350 per invoke/worker | low |
| F9 | 3 (provenance) | SKILL.md:79-80 "(decision `d9007aca`)"; SKILL.md:233-234 "(grade rule: docs/history/knowledge-freshness-mechanic/CONTEXT.md D4)"; bee-guard.md.tmpl:7 "— decision 0021" | decision ids inside live bodies | Move them to a new skills/bee-swarming/references/provenance.md, the same convention used by bee-evolving and bee-writing-skills. The bee-guard id is paid on every review dispatch. | ≈150 | low: `rg -F "decision 0021" packages` finds no test pin |
| F10 | 1 | SKILL.md:165 (902 B); playbooks/handoff.md:17-27; LEADER.md:290-302 | A context mark never stops a wave, plus the full `waggledance dispatch … --follow-up-of … --handoff` door | Shrink to: "A context mark never stops a wave; a requested switch or feature close follows `.bee/playbooks/handoff.md`." LEADER.md:14 already routes there. | ≈700 per invoke | low: no test pin |
| F11 | 1 | SKILL.md:46-47 vs SKILL.md:287 | "never by 'spawn both carefully'" vs "never by being careful" | Keep the hard rule at :287 and cut the step-1 copy. | ≈110 | low |
| F12 | 4 + 2 | SKILL.md:177-185 | The two contract guards (`CONTRACT_RETIRED`/`UNSETTLED`/`UNCITED`) and the "passed over silently" note | Both refuse with typed errors that "name their own remedy" and "mutate nothing", and SKILL.md:174-175 already says "when a verb refuses, its message names the fix". Move to worker-details.md. | 642 per worker | low |
| F13 | 4 | SKILL.md:145-160 | the `uat_stop: "merge"` branch plus staging fallback | Applies only under non-default config. Move it to swarming-reference.md and keep one pointer line; the default branch stays. | ≈1 KB per invoke | med [unverified: how often hosts set `uat_stop: merge`] |
| F14 | 5 | bee-forager:13, bee-scout:13, bee-elder:13 | "You do not register in the swarm registry and take no reservations …" | These three agents have only Read/Grep/Glob, so they cannot run `state worker add` or `reservations` at all. The line changes nothing they can do. Delete it there; keep it in bee-guard, which has Bash. | ≈150–270 each per dispatch | low |
| F15 | 6 | 5 of 6 agent descriptions (comb, forager, guard, scout, worker) | "Reached only through `bee dispatch prepare` (it may return a pane command instead of this type) — never name this type by hand." | This sits in every Claude leader's Agent listing, which is always loaded. The model-guard hook (LEADER.md:258-260) already refuses hand-picks. Shorten to "Only via `bee dispatch prepare`." | ≈90 × 5 = 450 always loaded | low-med: it is the leader's first signal; no test pin (`rg -F "Reached only through" packages` is empty outside the templates) |

## 3. Top 3 for this branch (saved tokens × frequency)

1. **F1**: stop loading the full SKILL.md into every native worker. That saves ≈11.7 KB × workers per wave, plus ≈6.8 KB per orchestrator invoke. The golden test must be updated with it.
2. **F5 + F6**: make worker-cell.md the single home of the Result form and the prompt template. This removes a drifted schema (2-part vs 3-part `mistakes`, no knowledge keys, no [PARKED]) and the 64 KB read a worker may take to reach it.
3. **F3 + F4**: strip the duplicates from the agent bodies (shape rule, trailer, tokens, finish) and drop the dead worker-details.md citation. That saves ≈1 KB per native worker and one false read target.

## 4. Do-not-touch (class 8)

- SKILL.md:187-189: the original request passes VERBATIM under a DO-NOT-PARAPHRASE header. Pinned in drivers/tests.rs and docs/knowledge/areas/workflow-state/dispatch.md.
- SKILL.md:77-80: "never read a worker's transcript, pane log … for progress". This rule itself saves tokens; keep it and drop only the decision id (F9).
- SKILL.md:89-95: "the worker's word is never the evidence", the leader completeness check.
- SKILL.md:124-127: "the slice boundary is never a user question, never a 'say go'". A compact rule against a stop that keeps recurring.
- SKILL.md:195-203: installs, an unnamed contract change, or trading quality for a technical target means `[BLOCKED]`. Each is a safety boundary.
- worker-cell.md:16-19: the native worktree cwd self-check (zero edit attempts).
- worker-cell.md:86: "a bare id alone fails the cap". This why-clause prevents a real refusal.
- bee-worker.md.tmpl:12: on a reservation or hold conflict, return `[BLOCKED]` and never write through it.
- bee-worker.md.tmpl:16: the cold-pickup rule (report the gap, don't guess).
- bee-comb.md.tmpl:21: "never write that it ran" for the design gate when Node is missing.
- SKILL.md:290: two workers failing on one root cause stop the wave.

## Outside this lens, noted for follow-up

- The `bee finish` help (registry_payload.json) says it will "run the project's declared tests … red refuses with the failing excerpt". The `bee cells finish` help says "No door runs tests". The finish help text contradicts itself, and SKILL.md:239 tells workers to use `bee finish` while worker-cell.md:99 uses `bee cells finish`.
- The mailbox brief wrapper that herding workers receive (about 7 KB in this job's brief-1.txt) is per dispatch and probably worth its own audit. It is built in Rust and lies outside this branch's files.
