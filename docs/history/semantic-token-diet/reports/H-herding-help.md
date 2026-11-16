# Token audit — HERDING AND CLI HELP (skills/bee-herding, skills/bee-herdr, `--help` surface)

Read-only. No head check (brief: not a diff review). Proof command not run: this round changed no code. Byte counts come from `wc -c` and `sed -n A,Bp | wc -c`.

## 1. Load map

Loop cadence comes from `control_loop.rs:115-120`: dispatch and merge run every 60 s, route every 300 s, supervisor every 900 s. Each iteration is a cold `claude -p`.

**Prepaid**

| Load | Files (bytes) | Total | Frequency |
|---|---|---|---|
| Dispatch iteration | dispatch-prompt.md 717 + SKILL.md 8,720 + role-dispatch.md 31,904 (SKILL.md:30 says "Read your role's reference in full") | **41,341** | every 60 s |
| Merge | merge-prompt.md 897 + SKILL.md 8,720 + role-merge.md 15,829 | 25,446 | owner gesture (loop default is also 60 s) |
| Route tick | route-prompt.md 15,822 (route-prompt.md:10-12 says it is the whole contract) | 15,822 | 300 s |
| Supervisor tick | supervisor-prompt.md 14,721 | 14,721 | 900 s |
| Bootstrap | SKILL.md 8,720 + role-bootstrap.md 3,996 | 12,716 | one-shot |
| bee-herdr invoke | SKILL.md 5,098 | 5,098 | per invoke |

**On-branch only:** operational-invariants.md 37,833; dispatch-dry-run.md 15,001; spawn-proof.md 11,292; README.md 12,425 (role-bootstrap.md:33,74 routes there); wave-runs.md 4,883. `scripts/bootstrap-cockpit.sh` (16,947) is executed, not read. `agents/openai.yaml` (713) is Codex metadata.

At about 4 bytes per token, one dispatch iteration costs about 10.3k tokens. Over a full day of polling (1,440 iterations) that is about 14.9M tokens. [unverified: how much prompt caching absorbs across cold `-p` processes]

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite / move | est. bytes saved | risk |
|---|---|---|---|---|---|---|
| 1 | 4 | role-dispatch.md:203-543 | §5-§8, `--dry-run` and the quick reference: 21,398 B, read in full every iteration. Most iterations end at §2 (bypass), §4 (no free slot, or `fallback`) or §5 (interlock off; it is off now: `bee herding status` → `"enabled":false`). | Split at the interlock. role-dispatch.md keeps §0-§5's interlock check and ends with "interlock true and a slot free → read `role-dispatch-spawn.md`". §5 set build through §8 moves to that file. Change SKILL.md:30 "in full" to "in full, following its own hand-off". Alternative in code: `control-loop --role dispatch` runs `interlock` and the bypass check before it spawns at all. | ~20 KB per idle iteration | med. Code comments cite §-numbers (pane_verbs.rs:63,649; wave.rs:16,1283; herding.rs:18), so keep the numbering. No test pins this file (rg over packages/ found none). |
| 2 | 1 | role-dispatch.md:522-543 | The quick-reference table restates §1-§8 inside the same file, which is already read whole. | Delete it, or keep it and delete nothing else; one of the two has to go. If finding 1 lands, the table only belongs in the spawn file. | 3,084 | low |
| 3 | 3 | role-dispatch.md:219-228, 366-397, 314-317, 434-440, 115, 120-124 | Incident narration: "measured false"; the "60/30/15 … 173x50 cut to 13 rows" split measurements; backlog id p-0b2812f8; "§4 no longer counts panes"; "pre-D18 binary"; a paragraph on the plain-text output form, which the role never reads. | Keep each rule as one sentence ("right only when parent is your own pane, else down; ratio = share the parent keeps"). Move the measurements to spawn-proof.md or references/provenance.md. | ~3,500 | low. The rules stay; only the evidence moves. |
| 4 | 7 | role-dispatch.md:224-226; README.md:42 | Says the `bee herding enable|disable|status` verbs "are not built into the current binary". | `status` is built: `bee herding status --help` prints its flags and `bee herding status` returns `{"enabled":false,…}`. Only enable/disable are unbuilt (`bee herding --help`: "NOT BUILT INTO THIS BINARY"). Rewrite as "touch the marker; no enable verb exists; `bee herding status` reads it". | ~100, plus a stops-misleading fix | low |
| 5 | 7 | agents/openai.yaml:3 | "the bee-herding cockpit's three roles — bootstrap … dispatch … merge"; "(no --role given)" | Route, broker and supervisor are missing (SKILL.md:28-70; control_loop.rs:102-110). Replace with SKILL.md:4's description. | ~350 | low |
| 6 | 7 | `bee herding control-loop --help` (registry_payload.json) | "--role* — dispatch or merge"; "control-loop.sh itself stays in place, unreferenced, until the cell that rewires bootstrap-cockpit.sh removes it"; "D13: byte-identical to the pre-D4 hardcoded default" | `fd control-loop.sh` finds nothing. The roles are dispatch, merge, route, supervisor and broker (control_loop.rs:102-110). Drop the script sentence and the pre-D4 narration; list all 5 roles. | ~500 | low |
| 7 | 4 | SKILL.md:122-135 | The Waves paragraph (836 B). SKILL.md:124 itself says "no role calls it", yet every dispatch and merge iteration pays for it. | Collapse it to the existing References row at SKILL.md:145. wave-runs.md:19,55,69 already holds the three surprises. | ~750 | low |
| 8 | 3 | SKILL.md:13-23 (frontmatter `reason:`) | 1,794 B of dependency reasons carrying decision ids (tmux-herding-transport D1, tmux-herding-cockpit D3, D2) and "no Node runtime is involved" | One sentence per reason ("required when `herding.transport` is X"). The ids go to provenance. | ~1,100 | low [unverified: whether a skill-tree validator reads `reason` length] |
| 9 | 1 | SKILL.md:112-120; role-dispatch.md:420-432; operational-invariants.md:67-144 | The runtime adapter has three homes. Dispatch pays for two of them every iteration. | operational-invariants.md owns it. Dispatch §8 keeps the argv rule it acts on. SKILL.md keeps a one-line pointer. | ~400 | low |
| 10 | 1 | SKILL.md:72-83 | The role boundary restates the "only X" clauses already in each role paragraph (SKILL.md:41,50,58,66), and each role file restates it again (route-prompt.md:14-35; role-dispatch.md:5-6). | Fold each role's "never" list into its own paragraph. Keep "About to take another role's action? Stop — wrong section." | ~400 | med. These are safety boundaries; keep every "never" item. |
| 11 | 3/5 | route-prompt.md:395-404 | "The interval is a guess … plan.md OQ5 … Nobody has measured" (413 B, every 300 s tick). The agent cannot change its own interval. | Move to provenance. The pinned needles (control_loop.rs:1882-1886) do not include this text. | 413 | low |
| 12 | 3 | route-prompt.md:64, 308 | Headings carry decision ids, including a full UUID "decision 4a395ea7-503d-4117-b84b-96b657490f70" | Drop the ids from the live text. | ~120 | low |
| 13 | 2 (sink) | operational-invariants.md:161 | "The transport a run picked is reported back: `bee herding status --json`" | I measured `bee herding status --json` at **454,357 B** and text mode at 195,587 B. Neither is spilled. Point at `bee herding status --job <id>` (LEADER.md:284 already prescribes the one-line form). | up to 450 KB per read | low |
| 14 | 1 | role-dispatch.md:161-174; role-merge.md:66-75 | The four-condition "finished" test is spelled out in two role files. | Same tokens per iteration (each role reads only its own file), but the two copies can drift. A future `finished` field in `bee worktree list --json` would remove both. | 0 now | — |

**bee-herdr (5,098 B):** already dense. The only candidate is SKILL.md:82-83 ("Transport is declared, never sniffed"), which duplicates bee-herding SKILL.md:18. It is cheap to keep.

### CLI help surface — verdict: yes, it is an agent token sink

The doctrine comes from `session_preamble/budget.rs:161`: "before the FIRST use of a verb in a session, read `bee <verb> --help` … one read per verb".

Full help sizes, measured with the spill files included:

| verb | full bytes | longest line | decision ids | history words |
|---|---|---|---|---|
| dispatch prepare | 9,265 | 4,304 | 0 | 4 |
| cells cap | 8,711 | 1,106 | 5 | 0 |
| worktree merge | 6,590 | 4,493 | 5 | 7 |
| herding run | 4,872 | 1,855 | 5 | 0 |
| state set | 2,839 | 1,409 | 0 | 2 |
| close | 2,109 | 1,717 | 4 | 1 |

These six verbs alone cost 34,386 B (about 8.6k tokens) per session that uses them. Across the registry, the largest entries are cells.finish at 10,610 B and dispatch.prepare at 10,356 B; the median entry is 1,179 B (237 commands, 368,916 B in total, from `registry_payload.json`).

The brief's figure of about 9 KB for `dispatch prepare` is the full size. Inline, the agent sees 5,827 B; the remaining 3,587 B are spilled to `.bee/tmp/output/…`.

**The spill hides flags, not prose.** The cut lands in the middle of the flags list:
- For `dispatch prepare`, the hidden flags are `--role --cell --worker --purpose --force-ownership --claim --session-id --expertise --brief-file`. `--cell` and `--worker` are the flags a cell dispatch requires.
- For `cells cap`, the hidden flags are `--no-mistakes --fix-at --knowledge-used`.

So an agent that obeys "read --help once" either guesses flags or reads the spill file as well, which brings `dispatch prepare` to about 15 KB in total.

**What fills the help text:**
- Single-paragraph descriptions over 4 KB, with no structure to skim.
- History: "legacy `--cleanup` … now a no-op", "cleanup-by-default replaced its old opt-in meaning", "report-less legacy cap (pre-contract)", "renders byte-identically to before".
- Decision ids and justifications: "D12 (#102): a host worktree guard refuses…", "uat-gate-before-merge D1", "full-suite-once D3".
- Internals the caller cannot act on: the holdout hash arm, `knowledge_holdout_percent`, the `prompt_sha256`/`protocol_sha256` record fields.

**Proposal:**
- (a) Split each registry entry into a `summary` that `--help` prints (what the verb does, how it refuses, at most 2 traps) and a `details` field that `--help --full` prints or that lives in docs/knowledge.
- (b) Never spill the flags block: truncate prose first.
- (c) Strip decision ids and history from `registry_payload.json`. The rg search found no other source of the description text.
- Estimated result: `dispatch prepare` drops from 9.2 KB to about 2 KB.
- Risk: med. `registry_contracts.rs:1-16` pins shape and examples, not description wording [unverified: I did not rg every sentence against the tests].

## 3. Top 3 by saved bytes × frequency

1. Split role-dispatch.md at the interlock, or pre-gate it in `control-loop`: about 20 KB saved on each idle 60 s iteration (findings 1-3).
2. Restructure the help surface (summary/details split, flags never spilled): about 25 KB saved per session that follows the `--help` doctrine, and it repairs the hidden-flag hole.
3. Change operational-invariants.md:161 to `bee herding status --job <id>`, and spill or page `herding status`: avoids a 454 KB read.

## 4. Do-not-touch (class 8)

- role-dispatch.md:250-281: the two-key lane rule with its fail-open evidence ("Treat `lane_safe:true` as 'no obvious keyword hit'", plus the live counterexamples). This why-clause is what Key 2 depends on.
- role-dispatch.md:83-118: `fallback` and command failure must never be read as zero.
- role-dispatch.md:58-63: "Always pass `--pane`", with its why.
- role-dispatch.md:476-482: the record-worker failure must be reported loudly.
- role-dispatch.md:462-463: "a blind retry is how a cold loop turns one mistake into 1440 a day".
- SKILL.md:85-110: the safety boundaries.
- bee-herdr SKILL.md:27-30: send-keys fires every newline as Enter.
- bee-herdr SKILL.md:66-72: the ENVELOPE TRAP.
- bee-herdr SKILL.md:91-97: the Red Flags.
- route-prompt.md and supervisor-prompt.md text matching the needles pinned at control_loop.rs:1328-1364 and 1882-1886.

## Commands run (read-only)

- `wc -c`, `cat -n` and `sed -n` over the branch files.
- `.bee/bin/bee <verb> --help` for the 11 verbs above, plus the spill files.
- `.bee/bin/bee herding status [--json] | wc -c`.
- `rg` over `packages/` for pinning tests and help-text homes.
- `fd control-loop.sh` (no match).
