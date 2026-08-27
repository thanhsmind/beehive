# Distill — "Agent Harness Specification" (Supervisor · Leader · Advisor · Peer, 6 Thinking Hats)

- Date: 2026-08-26 · Source: claude.ai public artifact `a9391123` (`agent-harness-spec-final.md`, Vietnamese, self-contained harness spec, "Final, gộp v1–v3")
- Read: FULL text captured (2026-08-27, via reader proxy) — saved at docs/history/research/agent-harness-slap-fulltext.md. The earlier tail caveat no longer applies.
- Context: feeds the closed map docs/discovery/slp-supervisor-lead-peer/MAP.md and the shaping of feature 1 (slp-supervisor-heartbeat).

## Bottom line

`Upstream` This spec is a sibling of SLP with a different center of gravity: **one request in → one Final Report out**, everything between run by agents under hard-coded loop ceilings. Its mantra: "Supervisor route, Leader nghĩ, Advisor gỡ, Peer làm, QA chấm, Orchestrator đếm."
Most of its load-bearing ideas are ALREADY decided in the slp map or already exist in bee. Four concrete pieces are worth stealing into the slp features; one naming trap must be avoided; one idea ("silence is consent") is a user decision, flagged, not taken.

## What it maps to (confirmations — no action)

| Harness spec concept | Where it already lands |
|---|---|
| `raw_request` verbatim in the envelope, "nguyên văn human, không chỉnh sửa"; verbatim UP the chain, compressed only DOWN (context_slice) | `Local` decided: 3899fa60 + 9c0104e0 (bee intent anchor into every dispatch) |
| Silent Brainstorming: 5 Hats parallel, absolute isolation before synthesis | `Local` decided: 9cffdfb5 + 5981246b (blind lanes over advisor dispatches) |
| "Không ai tự chấm bài mình" — independent QA Checker, separate call, Given/When/Then per criteria, max 2 fails | `Local` exists: bee-review tier, judge-debt (`bee cells judge-record`), reviewer isolation contract |
| "Luật vật lý nằm ở code" — non-LLM orchestrator holds loop counters, call ceilings, timeouts; LLM never widens its own limits | `Local` exists partially: cell budgets (`max_claims/max_failed_attempts/max_same_signature`), herding control-loop timeouts/backoff; per-request LLM-call ceilings do not exist |
| 3-speed lanes, upgrade-only mid-flight, deterministic R0 rule before an LLM classifier | `Local` exists: bee lanes + `bee route` / `bee herding classify-lane`; upgrade-only matches bee's escalate-never-relax posture |
| Advisor: gỡ HOW, không làm hộ, không sửa spec, stateless, cắm cờ `van_de_thuoc_spec` | `Local` exists: advisor consult (read-only, one call); the how/what/unclear triage matches bee-swarming's rescue ladder |
| Escalation always carries 2–3 suggested options so the human only picks | `Local` exists as bee's question craft (bee-shaping interview rules) |

## Four steals (worth taking)

1. **Two stuck-detection rules** for the supervisor feature's signal set (extends da7cb49b's day-1 list): `Upstream` `vuot_uoc_luong` — wall-clock or tool-calls > 2× the task's own estimate, measured by the harness, not self-reported; and `sua_lap_cho_cu` — two consecutive submissions differing only in the same region. Both are computable from bee's existing surfaces (cell trace attempts, activity transitions) plus one estimate field.
2. **An 80%-style *measured* budget trigger lives in the harness, not the agent** — their Orchestrator measures and INJECTS the overrun into the Supervisor's input. Confirms the slp map's stance that the 80% warning is harness telemetry (named in da7cb49b), and says who computes it: the deterministic layer.
3. **R0: a deterministic keyword list that grows one keyword per incident** before any LLM classifier, with `lane_source` recorded (`rule | classifier | supervisor_upgrade`). Cheap-learning loop for `bee route`: when a low-lane task causes an incident, its keyword joins the always-high-lane list. Candidate one-line improvement to bee route, independent of the slp features.
4. **do_tin (confidence) × door type as the escalation predicate**: assume freely on two-way doors; assume-and-log on one-way doors with high/medium confidence; ALWAYS escalate one-way + low confidence, regardless of autonomy level. A crisp, checkable rule the slp night-watch queue (9f5cd250) can reuse to sort "queue silently" vs "UrgentAlert".

## One trap (do not import)

`Upstream` Their **Supervisor is a router-dispatcher** (small stateful model deciding who acts next — bee's herding control loop + orchestrator already own this). The slp map's **Supervisor is an observer** that only asks open questions (322695d6, c80debd7). Same word, opposite powers. Import mechanisms, never the name — in bee vocabulary their "Supervisor" is the herding control loop, their "Leader" is the orchestrator session, their "Advisor/QA" are the advisor/review tiers, their "Hats" are blind lanes.

## One flagged idea (user's call, not taken)

`Upstream` Autonomy Policy B, "**Silence is consent**": on a one-way door the system notifies async with a recommendation; if the human stays silent X minutes (30–60 configurable), it proceeds with the recommendation. This CONFLICTS with bee's standing rule that gates are never self-approved (gate_bypass is the one recorded exception) and would extend 9f5cd250's "non-urgent asks queue silently". `Inference` It could exist in bee only as a new opt-in bypass mode. Flag for the user at slp-supervisor-heartbeat shaping; not a decision here.

## The 5 Hats in detail (from the full text, §11.3, §12.4–12.8, §13–14)

`Upstream` The hats are NOT design lanes — they are a **structured review
fan-out over a request before anything is built**, on a mid-class model,
each hat a fixed perspective with an explicit NON-OVERLAP clause:

| Hat | Scope (layer) | Signature moves | Forbidden |
|---|---|---|---|
| Trắng W — data & spec | L1 Data Contract, L2 Happy Path | schema completeness; step 1→N triggers; **CRUD Lifecycle check** (a Create must answer where Read/Update/Delete-Archive live) | sentiment; proposing solutions (Green's job) |
| Đen B — risk & edges | L3 Failure/Edge, L4 NFR | **Truth Table Test** (every IF needs its ELSE — a missing ELSE is one finding); timeout/idempotency/retry-dup/race/dirty-data; NFR must carry numbers; severity "cao" = ships → loses money/data/trust | re-checking basic schema (White's job) |
| Vàng Y — value | quick wins | value-vs-cost ranking, scope-to-ship-earliest ordering | listing risks (Black's job) |
| Xanh Lá G — alternatives | simplification | one SIMPLER way per complex part; buy-vs-build; v1/v2 cut; each proposal carries its trade-off | — |
| Đỏ R — UX & intuition | friction | walk the flow as the user; unproven hunches allowed if described concretely; `de_xuat` may stay empty | — |

Blue hat = the Leader itself (synthesis) — hence 5 agents, not 6.

Shared machinery: one `HatPayload` schema (findings with id-prefix W/B/Y/G/R,
severity, layer, ≤12-word title, proposal; `pushback: true` valid ONLY with a
non-empty `missing_context`); the **5-Layer rubric** (Data Contract / Happy
Path / Failure-Edge / NFR / DoD) embedded in the classifier and in White/Black;
and an **anti-fabrication validation** at synthesis — every `finding_ids` the
Leader cites must exist in the original HatPayloads ("chống Leader bịa"),
one retry on violation.

**Where this lands in bee** (`Local`): the hats map onto bee-reviewing's
specialist reviewers and the judge tier, NOT onto blind lanes (5981246b's
lanes generate designs from a byte-identical brief; hats critique one
request from disjoint angles — different instrument, both valid). Worth
taking: (5) the 5-Layer rubric + Truth Table + CRUD Lifecycle as
reviewer/judge checklist lines — this is exactly the SLP book's "10–20
generic anti-pattern bullets"; (6) the anti-fabrication citation check at
convergence — a blind-lane dossier's citations must resolve against the
verbatim proposals it synthesizes; (7) `pushback` hygiene — an objection
is only valid when it names what is missing.

## Second pass — the tail's good points (full text §11.8–17, appendices; absorbed as D 66c4c251)

`Upstream`

- **10-metric human audit** (§16): the human reads 10 numbers every ~20
  tasks (~10 min) instead of logs — lane ratio 70/20/10, wrong
  assumptions <15% ("fix the Context Store, not the Leader"),
  escalations/task <0.5 (high = the Leader is too timid to assume),
  BLOCKED <10%, QA first-pass >60%, human-reversed one-way decisions ≈0,
  supervisor-overreach reversals ≈0, self-answered pushback 30–60%
  (too HIGH is also unhealthy = sloppy answers), advisor resolution
  >60%, spec-flag rate 10–25% (≈0% means the advisor is quietly working
  around bad specs — dangerous). Two-sided health bands are the craft.
- **Earned autonomy**: metric 6 (human-reversed one-way decisions) held
  at zero across 40–60 tasks is the CONDITION for raising Autonomy
  Policy B→A. Autonomy is a track record, not a config default.
- **Report honesty as the metric** (§12.2): the Leader "is judged by the
  honesty of the report, not the completion rate" — the anti-rosy-report
  incentive, stated inside the prompt.
- **Ledger sorted by impact-if-wrong** (App. B): the assumption ledger
  in the final report sorts by `anh_huong_neu_sai` descending, so the
  human's first three lines catch the most dangerous assumption. Taken
  for the WakeReport ordering (9f5cd250).
- **"Trò chơi điện thoại" guard** (App. B): forwarding UP is verbatim;
  the orchestrator COMPARES forwarded payload length against the
  original to catch silent compression. Mechanical complement to
  3899fa60/9c0104e0.
- **Log-before-act**: "chưa ghi Assumption/Decision thì kết quả không
  hợp lệ" — matches bee's decision-before-continue invariant.
- **7-step build roadmap** (App. A): schemas + code validation FIRST
  (no LLM until validators exist); one model plays every role until the
  loop works end to end; split the small-model Supervisor out only at
  step 5 — "never debug three LLM layers at once". Sequencing advice
  for slp-supervisor-heartbeat's plan.
- **Acceptance pack** (App. C): three sample tasks including one
  deliberately under-specified — the run must show a logged assumption
  plus a correct escalation. Same test-scenario style as SLP's A1–A7.
- **Claude Code mapping** (§17.1): envelope as `runs/<id>/envelope.json`
  (resume across sessions), call ceilings counted by hooks, level-B
  escalation as a pending file plus a script timer — confirms
  c706053e's timeout belongs to the deterministic layer, never the model.
- **Lazy loading**: deep Context Store docs load only when a pushback
  asks for them — never pre-stuffed into prompts.

## Also seen, thinner than what bee has

- Their decision record (`DecisionEntry` with `cua`, `chi_phi_dao_nguoc`, `nguoi_quyet`) and Assumption Ledger are one JSON envelope per request; bee's decision log is repo-wide, append-only, with supersession, triggers and tags — strictly stronger. The one field bee lacks: an explicit **reversal-cost** (`chi_phi_dao_nguoc`) on the decision record; today it lives in prose.
- Per-lane total-LLM-call ceilings (Làn 1 ≤ 8, 2 ≤ 15, 3 ≤ 30) — bee budgets per cell, not per request; worth remembering if runaway cost ever shows up.

## Next step

Feed this brief plus the two research digests into bee-shaping for `slp-supervisor-heartbeat`: take steals 1–2 into the feature's signal/telemetry shape, take steal 4 into the wake-queue sort rule, and put the "silence is consent" flag in front of the user as one shaping question. Steal 3 (R0 keyword growth) is a separate tiny backlog item on `bee route`.
