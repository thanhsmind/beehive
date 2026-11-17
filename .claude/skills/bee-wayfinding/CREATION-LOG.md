# Creation Log: bee-wayfinding

TDD-for-skills pass, wayfc-1. Deepens the conversational craft to the
level of its source pattern (mattpocock wayfinder's grilling /
domain-modeling / prototype skills). Full record: `docs/history/
wayfinding-craft/CONTEXT.md`.

## RED phase: baseline pressure testing

Three pressure scenarios, sonnet, 2026-08-17, run against the
pre-edit skill.

### S1 — charting, time pressure

**Verdict:** FAIL (partial). The destination held, but the agent
promised to finish the breadth-first sweep alone.

**Rationalization (verbatim, from CONTEXT.md):**
> "mình tự làm hết phần còn lại không hỏi gì thêm"

The sweep collapsed into an agent monologue — the map's fog would be
agent-invented, not the user's. Options the agent did offer carried no
recommendation.

### S2 — grilling ticket, fact trap

**Verdict:** FAIL (partial). The never-answer-own-question rule held,
but the question went out bare: no recommended answer, no batched
frontier of follow-up questions, and the fact lookup stayed a "mental
note" instead of a dispatched read. CONTEXT.md records the failure
mode, not a verbatim quote, for this scenario.

### S3 — prototype ticket

**Verdict:** PASS by luck. Self-check verbatim, from CONTEXT.md:
> "The skill does not say how 'cheap' a prototype spike must be … that
> judgment call is mine, not the skill's."

The skill left "cheap" undefined, so the workable shape depended on
the agent's own judgment rather than a rule.

## GREEN phase: content mapped to failures

| RED failure | GREEN section |
|---|---|
| S1: "làm hết ... không hỏi gì thêm" — sweep became a monologue | SKILL.md, Session 1 step 2, amended: the sweep is explicitly an interview with the user in rounds of frontier questions; any agent-suspected fog line the user hasn't confirmed is marked `(agent-suspected)` in "Not yet specified" (also carried into wayfinding-reference.md's "Not yet specified" section rule) |
| S1: options offered with no recommendation | SKILL.md "Interview craft" (new body section) + reference "Interview craft": every frontier question carries the agent's recommended answer — recommending, never answering, for the user |
| S2: bare question, no batched frontier | SKILL.md "Interview craft": ask the whole frontier in one round, numbered; a question depending on an answer still open this round waits for the next round |
| S2: fact lookup stayed a "mental note" | SKILL.md "Interview craft" + reference "Interview craft": a frontier item needing a repo/environment fact is dispatched (gather subagent or direct read), never asked, while the rest of the frontier ships now |
| S3: "the skill does not say how 'cheap' … that judgment call is mine" | reference "Spike rules" (new section): one-command/one-click runnable, no persistence/polish/tests, full state shown after every action, several variants for a "which shape" question, verdict written back to the ticket's `## Answer`, spike stays under `.bee/spikes/` as history |

Cross-references added rather than inlined, per scope: the "grilling"
row in SKILL.md's Ticket types table now points at this skill's own
Interview craft plus bee-shaping's `bee-shaping/references/gray-area-probes.md`
and bee-shaping's own "Interview craft"
(`bee-shaping/references/shaping-reference.md`); the
"prototype" row points at this reference's
new "Spike rules".

## GREEN verification

Re-run 2026-08-17, same three scenarios, same model tier (sonnet),
against the edited files. All three passed:

- S1 (charting, time pressure): sent one round of two numbered
  questions, each with a `➡️` recommendation; facts split out to a
  dispatched read ("phần dữ liệu thực tế mình tự tra, không hỏi bạn");
  no agent monologue — the RED promise "mình tự làm hết phần còn lại
  không hỏi gì thêm" did not recur.
- S2 (grilling, fact trap): read the auth module before replying,
  emitted the `❓ Q1` + `➡️` format, held the decision open for the
  user despite "chốt vậy đi".
- S3 (prototype): cited Spike rules by name — question at the top of
  the file, one double-click to run, two variants side by side,
  state redisplayed after every action, verdict protocol into the
  ticket's Answer.

Residual (not a violation): S3 asked whether placeholder data is
enough fidelity for a "which shape" spike and guessed yes — the
correct default (realism is polish). Left as a known micro-ambiguity
rather than a new rule; promote it only if a future run guesses wrong.

## Register

Matched the existing skill's voice: short declaratives, em-dash
asides, no headers-for-show. No new jargon introduced beyond what the
source pattern (grilling / domain-modeling / prototype) already names.
