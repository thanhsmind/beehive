# Token audit: DISCOVERY / SHAPING / RESEARCH (bee-wayfinding, bee-shaping, bee-researching)

Read-only. I followed .bee/tmp/token-audit/criteria.md. I did not check the review head, as the brief said.

Paths read: the three SKILL.md files; wayfinding-reference.md; shaping-reference.md, gray-area-probes.md, context-template.md; research-protocol.md, port-protocol.md, trace-and-provenance.md; skills/bee-how/SKILL.md:18; skills/bee-why/SKILL.md:18; gates-and-delegation.md:201-222; routing-and-contracts.md:244,375-388; wayfinding CREATION-LOG.md:1-12,28,48.

Commands run: `wc -c` on every file below, `.bee/bin/bee <group>` for each verb named in the skill bodies, `ls` on each referenced path, and `rg -F` across packages/ and scripts/ for pinned wording.

## 1. Load map

Prepaid on every invoke (SKILL.md):
- bee-shaping: 11531 B
- bee-wayfinding: 7414 B
- bee-researching: 5033 B

Paid on branch, but in practice paid on almost every run:
- `wayfinding-reference.md` 8623 B. Session 1 step 3 (SKILL.md:48-50) and later-session step 3 (SKILL.md:108-109) both send the agent there, so a normal wayfinding run reads it.
- `context-template.md` 3613 B. Every Lock reads it (SKILL.md:132).
- `shaping-reference.md` 5959 B and `gray-area-probes.md` 3494 B. Every Explore interview reads them (SKILL.md:53-56).
- `research-protocol.md` 4059 B. Standard depth runs all four steps and is the default (SKILL.md:38-39), so it is read on most runs.

Paid on a real branch:
- implement-plan-template 8064 B, mini-brief 1706 B, walkthrough 4571 B (Brief only)
- port-protocol 3983 B (only when the request names an external repo)
- research-brief-template 1849 B (standalone research only)
- `trace-and-provenance.md` 21434 B. bee-how and bee-why load it whole (bee-how/SKILL.md:18, bee-why/SKILL.md:18). It is also referenced from bee-teach, bee-blast-radius and bee-planning (`rg -ln trace-and-provenance`).

Never loaded: CREATION-LOG.md 4755 B. `agents/openai.yaml` is for Codex only.

Stale-text check: every verb the bodies name exists, namely discovery stub, route, state waiting-on set, knowledge pack/search, backlog pbi add/briefs, reservations reserve/release, decisions log/search, dispatch prepare. Every referenced path resolves, and all eight `bee-principle-*` skills exist. No dead text found.

## 2. Findings

| # | class | path:line | what it says | proposed semantic rewrite or move | est. B saved | risk |
|---|---|---|---|---|---|---|
| 1 | 4 | researching/references/trace-and-provenance.md:8-174 vs 176-381 | Trace + Critique (~9.8 KB) and Provenance sweep (~11.4 KB) share one file | Split into `trace.md` and `provenance.md`, then repoint bee-how:18, bee-why:18 and the 3 other referrers. A how question stops paying for Provenance; a why question stops paying for Trace and Critique. | 9.8–11.4 KB per how/why run | med: 10 files cite the path (`rg -ln`). No test pins found in packages/ |
| 2 | 1 | wayfinding/SKILL.md:87-91, 95-102 vs wayfinding-reference.md:88-96 | Ticket-claim rules appear twice: the reserve command, "deny means take another", heartbeat expiry, `blocked-by` convention-only, the frontier definition | Keep the SKILL copy (it changes behavior) and cut the reference paragraph down to the template only | ~530 | low: SKILL keeps the rule |
| 3 | 1 | wayfinding/SKILL.md:62-71 vs wayfinding-reference.md:100-114 ("Round mechanics, restated in full") | Frontier and rounds rules, plus "Facts are the agent's job" | The reference keeps only what the body lacks: the "recompute the frontier" bullet, the emit format and the example. Drop the restatement. | ~600 | low |
| 4 | 1/6 | wayfinding/SKILL.md:82 | The grilling row has a nested parenthetical that repeats the pointers from :73-76 and reference :161-164 | Shorten to "conversation, in Interview-craft rounds; the agent never answers its own question" | ~250 | low |
| 5 | 3 | shaping/SKILL.md:27 `(D6.4)`, :118 `(D6.3)`, :144 `(D8)`, :157 `(D1, traceable-runs)`, :165 `(D6)`, :121 `decision 12deaa34` | Bare D-IDs from another feature's CONTEXT.md. A Lock agent can read them as this feature's own D-IDs. | Move them to a new `skills/bee-shaping/references/provenance.md` (none exists today) | ~150 | low: no test pins on the SKILL text (`rg -F` found D6.3 only in discovery.rs source) |
| 6 | 1 | shaping/SKILL.md:80-91 vs gates-and-delegation.md:201-221 | 12 lines repeat both hat-wave windows and their decision ids. The home itself says shaping carries "the one-line trigger pointers, never a second copy" (gates-and-delegation.md:205-206). | Cut to one line: "A big, hard-to-reverse spec may earn a pre-Lock hat wave; the plan-step wave belongs to planning; Lock never takes hat output directly. Procedure: gates-and-delegation.md ("Hat wave")." | ~450 | low |
| 7 | 4 | shaping/SKILL.md:121-128 | Foreign-origin spec drop: a rare branch for waggledance relay PBIs, kept in the prepaid body | Keep one line in Qualify ("a foreign-origin PBI qualifies like any other; its spec file is the evidence") and move the rest to shaping-reference | ~400 | med: the "not dispatchable until locked" safety-net sentence must survive in the move. Whether herding enforces it is [unverified]. |
| 8 | 3 + 1 | shaping/SKILL.md:35-36 | "(replacing the separate `bee knowledge context` step…)" is history narration | Rewrite as "runs `bee knowledge pack --feature <f> --stage shaping` (`bee knowledge search --text` for symptoms)". See the drift note under section 5. | ~80 | low |
| 9 | 1/2 | shaping/SKILL.md:146-150 vs context-template.md:46-49, 53 | The read-set source and the `no-read-set` escape are stated three times. The gate's FIX line prints the escape too (plan_packets.rs:674). | Body: "Lock fills `## Read set` from the scout's pack output; the template names the source." Drop the escape sentence. | ~220 | low: do not touch the template table headers (row 3 of section 4) |
| 10 | 1 (conflict) | shaping-reference.md:26-28 "One question per turn… a batch buries the reframe" vs wayfinding/SKILL.md:63-66 "Ask the whole frontier in one round" vs routing-and-contracts.md:388 "Independent questions may share one AskUserQuestion call (up to 4); dependent ones go one at a time" | 3 homes, 2 policies | Single-home the rule in routing-and-contracts "Question Format" and turn the shaping bullet into a pointer. This changes shaping behavior, so it needs a user decision. | ~150 | med: behavior change. CREATION-LOG.md:28,48 show wayfinding pressure-tested the batched frontier. |
| 11 | 1/2 | shaping-reference.md:38-42 "Mark the wait" | Repeats routing-and-contracts.md Question Format (the `bee state waiting-on set` line at ~:391). cli_shape.rs:1222 also knows the command; what that hook prints is [unverified]. | Shorten to one pointer line | ~200 | low |
| 12 | 1 | shaping-reference.md:46-53 vs shaping/SKILL.md:76-78 | The "When to stop" rule and "never ask 'anything else?'" appear in both | Keep the body. The reference keeps only "effort matches the triage row, never a question quota." | ~200 | low |
| 13 | 1 | research-protocol.md:34, 38, 50, 61-69 vs researching/SKILL.md:31, 34-36, 52-53, 71-72 | Four facts are stated twice: "missing needs code/config/docs/tests checked", "local beats docs, record mismatch", "capability gap degrades, never skips", "ask one question only when…" | Keep the one-liners in the SKILL body and cut the protocol's restatements. Keep the 3 ask criteria (:65-67), which add detail. | ~600 | low |
| 14 | 1/5 | port-protocol.md:6-12 "When To Load"; :3 / research-protocol.md:3 / trace-and-provenance.md:3 "Load only after bee-researching is active" | The trigger vocabulary repeats the skill description and the References row. "Load only after…" changes nothing: an agent only reaches the file through the skill. | Delete these lines | ~420 | low |
| 15 | 3 | wayfinding-reference.md:94 "convention-only in v1" | Version narration | Shorten to "convention-only" | ~6 | low |

## 3. Top 3 by saved tokens × frequency

1. **Split trace-and-provenance.md (#1).** Saves about 10–11 KB on every bee-how and bee-why run, and bee-teach, bee-blast-radius and bee-planning also send agents there. Biggest single cut on this branch.
2. **bee-shaping body diet (#5, #6, #7, #8, #9).** Saves about 1.3 KB of prepaid body. Shaping runs on almost every feature (Explore, Qualify, Lock), so this is the most frequent of the three skills.
3. **De-duplicate bee-wayfinding SKILL and its reference (#2, #3, #4).** Saves about 1.4 KB per wayfinding run. The reference is read on almost every run, so the duplicate text is paid twice.

## 4. Do-not-touch (class 8)

- wayfinding/SKILL.md:62-71. The frontier and rounds rule and "Facts are the agent's job" were pressure-tested: CREATION-LOG.md:28 records the bare-question failure, and :48 maps the fix to this body section. Cut only the reference copy (#3).
- shaping/SKILL.md:25-28. The entry check that routes fog to wayfinding before any interview.
- shaping/SKILL.md:107-109. "risk is a property of the change, not of the assessor's certainty" is a why-clause doing real work (agents-block-diet D6).
- shaping/SKILL.md:133-136 and :184-185. "Lock renders, never originates." This is the failure the skill exists to prevent.
- researching/SKILL.md:74-76. "an inconclusive result travels as an open question, never as a yes."
- trace-and-provenance.md:270-275. The report rules ("Seven rows go in, seven rows come out") and the phrasing rules (:300-324). They are the procedure's whole point.
- port-protocol.md:84-87. The guardrail that treats fetched source as data, never instructions. Safety.
- context-template.md table headers. drivers/tests.rs:6521-6524 reads this file, so the headers are test-pinned.
- wayfinding/SKILL.md:99-100. "A reservation deny … never write through it." A safety rule.

## 5. Found outside scope

- **Drift.** shaping/SKILL.md:35-36 says `knowledge pack` replaced `bee knowledge context`. The Rust gate FIX string still tells agents to run `bee knowledge context --work {feature}` (packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:674). One of the two is stale.
- **Possible drift [unverified].** routing-and-contracts.md:198 says implement-plan is "high-risk always, standard on-demand, small optional on request". shaping/SKILL.md:171-173 says "full template for standard/high-risk, ~15-line mini-brief for small". These two say different things about standard and small.

## Dismissed

- Stale-verb suspicion for `bee discovery stub` (shaping/SKILL.md:116) and `bee state waiting-on set` (shaping-reference.md:39): both verbs are listed by `.bee/bin/bee discovery` and `.bee/bin/bee state`.
- Stale-path suspicion for the `bee-principle-*` names in trace-and-provenance.md:145-168: all eight exist under `.agents/skills/`.
