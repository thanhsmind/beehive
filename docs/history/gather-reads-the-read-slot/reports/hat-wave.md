# Hat wave — gather-reads-the-read-slot (plan step)

Three seats, standard lane: `hat-facts-gaps` (opus), `hat-alternatives`
(opus), `hat-user-impact` (sonnet). Wave opened 2026-09-02 (decision
fd19a0ee). Synthesis by the leader; findings landed in plan.md and
CONTEXT.md before this record was written.

## What the wave changed in the plan

1. **Tail without `extraction`** (leader finding, confirmed by user-impact's
   codex/legacy walk): the gather walk is `[read, generation]`. A walk
   through `extraction` moved every legacy host's gathers (extraction +
   generation, no `read` — the THREE_SLOTS fixture) to the cheapest model
   silently. `cell_role_list("read")` keeps its own tail; the two lists
   differ on purpose (D1).
2. **Winner recorded on the default-gather path only** (D2). Alternatives'
   amendment (a) — record it on every path — was adopted, then WITHDRAWN on
   facts-gaps G1: with the winner as `marker_role`, `pinned_agent_type`
   re-pins a review-less host's reviewer onto `bee-gather`. The "latent
   reviewer defect" it was meant to close does not exist: `review` is a
   built-in name and the guard's `resolve_tier` walks `[review,
   generation]` the same way prepare does. D3 now states the rule: agent
   from the ASKED name and the kind, marker from the winner.
5. **D8 added** (facts-gaps G2): the herding-fallback contract keyed on the
   kind's head slot on both sides; `read` has no built-in default, so
   prepare would drop a `read` slot's fallback and the guard would stop
   admitting `generation`'s. Both halves key on the winner.
6. **G3 / G6 / G9 answered in place**: the gather pin fires only with no
   `--role`; `DISPATCH_KINDS` order is pinned by test and every kind-naming
   FIX appends `--role <role>`; the parity list carries workers.md:14, and
   WORK-05 (areas.md:37) quotes the generic-type FIX, which D7 does not
   touch. A live verify-app proof on a `read`-configured sandbox rides the
   resolver cell's cap.
3. **D7 added** (alternatives, Q5): `bare_dispatch_denied`'s FIX teaches
   hand-naming first and spells `bee-gather = generation`; it now leads
   with `bee dispatch prepare` and derives its agent list from
   `ROLE_AGENTS`. workers.md:14 quotes it and rides along in the docs cell.
4. **Cells run concurrently** (alternatives, amendment b): resolver (src)
   and docs cells are file-disjoint; only the surface cell waits.

## Kept as drafted, with the seat's reason

- Kind-pinned `bee-gather` (D3) — alternatives Q3: adding an
  `("extraction","bee-gather")` row flips a bare `[bee-tier: extraction]`
  general-purpose dispatch from silent repair to a hard deny
  (`model_guard.rs:1710`, `agents.len() > 1`), a live behaviour break.
- Known gap left open (bare-name `bee-gather` resolves `generation` in the
  guard) — alternatives Q4: closing it is ~15 lines but flips a verdict
  class (deny → allow on a `read: model` host); its own cell, its own
  matrix. The gap is smaller than CONTEXT first said: it lives in
  `role_for_agent`, not in mrs-29's `agents_for_role`.
- Claude agent files unconditional (D6) — agent-model-unpin D2 stands.

## User-impact record

- This host: no visible change — `read` and `generation` are the same pane;
  only the audit name moves (`generation` → `read`).
- Fresh host: default gathers move sonnet → haiku, which is what the seeded
  `read` slot's own description promised all along. Nothing prompts the
  user to re-read `bee models show`; doctrine B14 and dispatch.md carry it.
  No changelog exists in this repo.
- `read: null`: falls to `generation` — turning the slot off does not turn
  gathers off.
- codex/pi: byte-identical models (no `read` key → `generation`).
- Description first sentences (D5): the seat's drafts name the exact kind
  per agent (`--kind gather`, `--kind gather --role extraction`,
  `--kind cell`, `--kind reviewer`); the surface cell uses that shape.
- Gray areas recorded: `bee models show` does not say which row feeds
  gathers; a host that sets `read` DEARER than `generation` gets dearer
  gathers (their config, obeyed); `economics.logical_tier` consumers —
  only the guard's audit line reads it (`model_guard.rs:1190`), no
  role-mix counter keys on the literal `generation` for gathers.

## Seat digests (verbatim, trimmed)

### hat-alternatives (opus)
Q1 KEEP + amendments (a) unconditional winner record, (b) src/docs cells concurrent. Q2 KEEP one list — overtaken: the tails differ by consumer (see D1). Q3 KEEP kind pin (ROLE_AGENTS row would flip extraction repair to deny at model_guard.rs:1710). Q4 gap stays a separate cell (~15 lines, flips a verdict class; lives in role_for_agent guard.rs:297, read at model_guard.rs:1056). Q5 CHEAPER: bare_dispatch_denied model_guard.rs:1119/:1131/:1137 — lead with prepare, derive pairs from ROLE_AGENTS; workers.md:14 quotes it.

### hat-user-impact (sonnet)

1 this host: same pane, audit name only. 2 fresh host: sonnet→haiku for default gathers, nothing prompts a re-read of models show. 3 read:null skips the slot, gathers still run. 4 codex/pi identical. 5 descriptions reach the moment of choice (the harness prints them in its agent list); drafts name the kind per agent. 6 gray areas as listed above. 7 SEE mocks: this host {tool:Bash, command: herding run --agent agy-flash, logical_tier: read}; fresh host {tool:Agent, subagent_type: bee-gather, model: haiku, marker [bee-tier: read]}.

### hat-facts-gaps (opus)

Claims table 20/20 MATCH (rows 12-13 re-run). Truth table over [read, generation]: model/herding/cli/null/absent shapes all hold except two — G2 fallback (both halves keyed on the asked head, not the winner). Blockers: G1 unconditional winner re-pins reviewer onto bee-gather; G2 fallback desync; G3 gather pin under-specified (must be role-less only). Warnings: G4 explicit --role byte change (moot after D2 narrowed); G5 cli read on opencode removes bee-gather.md (same class as cli generation today); G6 DISPATCH_KINDS order load-bearing + FIX can name the wrong door on a split host (answered: --role <t> appended); G7 preamble door-line order (generation still first via cell kind); G8 rendered_from/slot keys move to read; G9 prose claims without rows and parity list gaps. Rubric: codex falls to generation unchanged; pi refusal names the resolved slot; proof was unit-only — a live gather + reviewer on a read-configured sandbox added.
