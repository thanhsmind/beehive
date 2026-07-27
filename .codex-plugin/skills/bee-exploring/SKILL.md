---
name: bee-exploring
description: >-
  Turn a fuzzy feature request into locked decisions in docs/history/<feature>/CONTEXT.md. Use when a request has gray areas or unstated product decisions that would make planning guess. Not for implementation research, cell creation, or code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# exploring

If `.bee/onboarding.json` is missing or stale, stop and invoke `bee-hive`.

Exploring turns fuzzy intent into locked decisions in `docs/history/<feature>/CONTEXT.md`. Scout bees find the flowers; they do not build the comb.

## Hard Gates

- **Batch independent questions into one message; serialize only dependent ones** (mechanics in step 4). *Independent* = its answer changes the framing of, and makes redundant, no other pending question — ask all of a phase's independent survivors in ONE `AskUserQuestion` message (the tool takes up to 4 questions, each with a `header` ≤12 chars and 2–4 options — overflow makes the harness reject the whole call as "Invalid tool parameters"; full schema in bee-hive `routing-and-contracts.md` → Gate Presentation Contract). *Dependent/branching* = its wording or its very existence hinges on another question's answer — ask it alone, only after that answer lands. Never blind-bundle: a question a prior answer could moot is dependent, not independent, and never rides in the batch.
- Do not answer your own question — even when you are sure of the answer.
- Do not research implementation, propose architecture, create cells, or write code — the sole exception is a throwaway SEE mock under `.bee/spikes/<feature>/mocks/` (P11, decision 0020; step 4).
- Do not invoke planning yourself. End by handing the user to `bee-planning`.
- Gather-altitude steps — step 1 scope reads, step 3 gray-area scout digest — delegate as I/O workers (extraction/generation tier) per the Delegation contract (D2/D3, `bee-hive/references/routing-and-contracts.md`); any other ad-hoc dispatch beyond the fresh-eyes reviewer's named review slot defaults to the generation slot, and ceiling requires the [bee-tier: ceiling] marker plus a one-line justification.

## Flow

0. **Enter the feature atomically (from `idle`).** A fresh session is phase `idle`.
   The transition into exploring is **one call** — `node .bee/bin/bee.mjs state
   start-feature --feature "<slug>" --mode "<mode>"` — which moves `idle →
   exploring`, sets the feature and mode, and resets all four gates in a single
   guarded mutation. Do this FIRST, before any `state set`. Do **not** hand-write
   `state set --owner exploring --phase exploring` from `idle`: the owner guard
   requires `--owner` to equal the pre-mutation phase, so from `idle` that call is
   refused and the session bounces back to routing (the idle→exploring loop). Once
   `start-feature` has run, phase is `exploring` and the step-6 `state set --owner
   exploring …` below matches. If a feature is already active (phase is not `idle`
   or a terminal phase), skip this — you are resuming, not starting.

1. **Scope**
   - Classify: `Quick`, `Standard`, or `Deep`.
   - Read the critical patterns — with a bundle, `docs/knowledge/index.md`'s `## Critical patterns` section; with no bundle, `docs/history/learnings/critical-patterns.md` — and `.bee/state.json` if present.
   - If the request spans independent subsystems, pick one and defer the rest.
   - **Backlog flip (D11a):** when this feature matches an existing PBI, run `node .bee/bin/bee.mjs backlog pbi status --id <id> --to in-flight --feature <slug>` same turn (one move — status and slug together), then `node .bee/bin/bee.mjs backlog render --write`; if the request never passed through the backlog, run `node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` first, then the status flip. This is the only place a PBI goes `in-flight` (id/verbs + merge rules live in the scribing reference; prose-ruled, never hook-enforced — D7). A PBI whose status is `parked` is the one exception — see the Brief check below before touching it.
   - **Brief check (D9):** if `docs/history/<feature>/CONTEXT.md` already holds a `bee-qualifying` park brief under `Outstanding Questions` → `Resolve Before Planning`, load it instead of a fresh quick-scout: its evidence is settled ground, and gray-area questions come only from what the brief still leaves open.
   - If `.bee/config.json` lacks `commands` (setup/start/test/verify), run detection first: `node .bee/bin/lib/commands_detect.mjs` prints JSON candidates from the repo's manifests. Present the candidates as **one** pre-filled confirmation question (`key: value — source`), still skippable; fall back to the open question when detection finds nothing. Write only user-confirmed values to `.bee/config.json` `commands`. Never invent command values (docs/09 item 1, D3 of harness10).

2. **Domain**
   - Classify each applicable type:
     - `SEE`: user-visible surface
     - `CALL`: API, CLI, webhook, SDK, service interface
     - `RUN`: job, script, service, or pipeline
     - `READ`: docs, emails, reports, notifications
     - `ORGANIZE`: data model, file layout, taxonomy, config
   - Load `references/gray-area-probes.md` and pick only relevant probes.

3. **Re-lane checkpoint** — once the scout's touch set is counted: measured evidence may demote `standard` → `small` once (files within threshold, zero hard-gate flags, zero open gray areas — all three). Never `tiny`, never twice. Log it, tick it. Rule: `bee-hive/references/routing-and-contracts.md` ("Re-lane checkpoint").
4. **Gray Areas**
   - Generate 2–4 unstated *product* decisions that would otherwise make planning guess.
   - **When step 1's Brief check loaded a park brief:** draw candidates only from what the brief itself still marks unclear — skip the quick-scout entirely, its evidence already covers that ground.
   - Do a **quick scout only** — one keyword pass, then read 2–3 relevant files (skip this when a brief was loaded, per the bullet above):
     ```bash
     rg "<feature-keyword>" src app packages --glob "*.{ts,tsx,js,jsx,py,md}" | head -20
     ```
   - Cite the existing patterns you found in your questions ("today, exports go through `src/report/csv.ts` — should this follow that?").
   - Exclude implementation choices, performance tuning, and new scope. If a candidate question only matters to the implementer, it belongs to planning, not here.
   - **Pre-classify for batching (the delegated pre-pass produces this — Hard Gates §Delegation bullet):** tag each candidate question *independent* or *dependent*, and for every dependent one name the question + answer it hinges on (the dependency edge). This slate — questions + tags + edges — is the input to step 4's batching; producing it inside the step-3 worker means the interactive phase opens already holding the whole plan instead of composing each question inline, one round-trip at a time.

5. **Socratic Locking**
   - **Ask in the fewest rounds the dependencies allow.** After the materiality test and the gate-bypass split (below) have removed every question that should not be asked, read the step-3 slate's independent/dependent tags and: put ALL surviving *independent* questions of a phase into ONE `AskUserQuestion` message (up to 4); ask each *dependent* question alone, in its own message, only after the answer it hinges on lands — where it is often dropped outright once that answer arrives (materiality). Batching changes the number of rounds, never the quality of a question: each stays concise, single-choice where possible, **outcome-framed** ("what breaks for users if…"), CONTEXT / QUESTION / RECOMMENDATION / options format.
   - Start broad, then narrow: the broad questions are the ones others depend on, so they lead — the independent batch first, then the dependents it gates.
   - **Materiality test (P20):** every candidate question passes three checks before it is asked — **material** (the answer changes scope, architecture, UX, data model, or acceptance criteria), **grounded** (cites scout evidence or a concrete uncertainty, never generic preference), **answerable** (the user can pick an option, approve a default, or supply a reference). A failing question is never asked: pin it as a labeled assumption for Context Assembly to write into CONTEXT.md, or hand it to planning if only the implementer cares about the answer.
   - **Gate-bypass refinement — information vs approval (decisions 0010/dcf01d7b/a93994d3).** Read the active level (`gate_bypass_level`). Under `full`/`total`, split every candidate question in two: an **approval** question — one where the agent already has a confident best answer and the user would only rubber-stamp it — is **NOT asked**; lock it from that recommendation (record it as a decision with its D-ID) and move on. Only an **information** question — one whose answer the agent genuinely cannot determine from evidence with a confident default, because it turns on a preference or knowledge only the user holds — is still asked, even under `total`. The litmus is one line: *"do I already have a confident best answer?"* — yes → proceed with it; no, and only the user can supply it → ask. This never gags a real information need (the user explicitly wants to still be asked for those); it only stops the agent asking merely to be approved. Under `off`/`normal`, ask per the materiality test as usual.
   - **Blindspot pass — teach before asking (P9, decision 0020):** when the user signals unfamiliarity with a gray area's domain — says so, answers with guesses ("chắc là…"), or asks what the options mean — invert for that area: explain the 2–3 concepts needed to answer well (one short outcome-framed message, no jargon), *then* ask. A decision locked from a guessed answer is a fake decision. The user can also request a full "blindspot pass" by name: sweep the unknown-unknowns (what good looks like, common potholes, prior art in this repo) before locking begins.
   - **SEE mock — react instead of describe (P11, decision 0020):** for a `SEE` gray area the user knows-when-they-see-it but cannot describe, you MAY build a throwaway HTML mock (2–4 variants, fake data, zero wiring) under `.bee/spikes/<feature>/mocks/` and lock the decision from the user's reaction, citing the chosen variant. This is the ONE exception to "exploring never writes code": mock files only, only under `.bee/spikes/`, never imported by anything, never promoted to production (spike-code rule applies).
   - After each answer, confirm the decision back and assign a stable ID: `D1`, `D2`, `D3`…
   - When an answer settles the meaning of a fuzzy domain word, confirm the term back and pin it like a decision (P21); Context Assembly writes all pinned terms into CONTEXT.md's `Terms` section, and scribing inherits them into the spec's Data Dictionary.
   - If one answer contains several decisions: lock the one your question asked about, echo the others as candidate decisions to confirm one at a time.
   - Scope creep (new features, adjacent work): mark it deferred with one line, return to the current question.

6. **Context Assembly**
   - Write `docs/history/<feature-slug>/CONTEXT.md` from `references/context-template.md`.
   - Include boundary, domain types, locked decisions table with D-IDs, pinned terms, scout paths, canonical references, open questions, and deferred ideas.
   - **Deferred Ideas also feed the product backlog (D8):** each Deferred Ideas entry that is real future work gets `node .bee/bin/bee.mjs backlog pbi add --title "<story>" --cos "<CoS>"` (then `backlog render --write`) in the same turn (announce-then-do) — the CONTEXT.md list is the record for this feature, the PBI is the durable product-level intent. Do not wait to be asked.
   - Concrete language only. No placeholders, TODOs, or vague preferences.
   - **Fresh-eyes review:** spawn one reviewer with no conversation history (slot: `review`, decision 0021 — default opus on Claude, falls back to generation) — **in the background where the runtime supports it** (decision 0017): keep assembling CONTEXT.md, keep talking to the user; the review blocks nothing until Gate 1. Collect the verdict before presenting the gate — Gate 1 is never presented with the review still outstanding. It checks completeness, contradictions, vague decisions, missing D-IDs, and blockers. Fix findings and re-review — max two loops, then present remaining doubts to the user.

7. **State And Handoff**
   - Update state:
     ```
     node .bee/bin/bee.mjs state set --owner exploring --phase exploring --feature "<feature>" --summary "Exploring complete. CONTEXT.md is ready for planning." --next-action "Gate 1, then invoke bee-planning."
     ```
   - **Gate-bypass check FIRST (routing-and-contracts.md §Gate bypass, decisions 0010/dcf01d7b).** Before presenting anything, read the active level: `node .bee/bin/bee.mjs status --json` → `gate_bypass_level`. If the level bypasses Gate 1 for this feature's lane — `normal` covers `tiny`/`small`/`standard` non-hard-gate; `full`/`total` cover **every** lane incl. high-risk/hard-gate (the human lifted that floor) — then **DO NOT present the question below.** Instead: take the CONTEXT.md as locked (the recommended path), set `approved_gates.context` yourself (`node .bee/bin/bee.mjs state gate --name context --approved true`), log a one-line audit decision (`decisions log --decision "auto-approved Gate 1 (bypass): <feature>" --rationale "..."`), post a short non-question line `⚡ auto-approved Gate 1 (bypass) — locking CONTEXT.md, invoking bee-planning`, and **continue straight to bee-planning**. Only when the level does NOT cover this gate (`off`, or `normal` on high-risk/hard-gate) do you present the human layer and ask.
   - Present **Gate 1** (only if not bypassed above) per the Gate Presentation Contract (bee-hive routing reference): plain-language layer in chat — what we decided / why trustworthy / cost if wrong / what you are deciding — in the user's language, CONTEXT.md linked not pasted; then verbatim: "Decisions locked. Approve CONTEXT.md before planning?"
   - CONTEXT.md is the source of truth for every downstream agent; decision IDs are stable and cited, never reinterpreted.

## Headless

With `mode:headless`: no Socratic dialogue. Lock only decisions the request states explicitly (still with D-IDs); write every gray area into the `Outstanding Questions` section of CONTEXT.md and of the terminal report instead of asking. Gate 1 is never self-approved — the report ends "awaiting Gate 1 approval".

## Red Flags

- **blind-bundling** — batching a dependent question a prior answer could moot, or dumping unclassified questions together (independent questions SHOULD batch; dependent ones must not); or a question answered by the asker
- serializing *independent* questions into separate one-per-message rounds when a single batched message would do
- a question asked that fails the materiality test — immaterial, ungrounded, or unanswerable
- deep implementation analysis or architecture proposals during exploring
- creating cells or writing code (except a `.bee/spikes/<feature>/mocks/` SEE mock per decision 0020)
- a SEE mock imported by production code, or surviving outside `.bee/spikes/`
- teaching skipped when the user is visibly guessing — a decision locked from a guess
- locking a "decision" that is really an implementation choice
- scope creep absorbed instead of deferred
- CONTEXT.md with placeholders, or skipping the fresh-eyes review
- skipping decision locking because "the user seemed to imply it"

Violating the letter of the rules is violating the spirit of the rules.

References: `references/gray-area-probes.md`, `references/context-template.md`.

Decisions captured and CONTEXT.md written. Invoke bee-planning skill.
