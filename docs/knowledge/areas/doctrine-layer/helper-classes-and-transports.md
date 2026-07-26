---
type: bee.area
title: Doctrine Layer — helper classes and transports
description: "The two worker classes the delegation layer names, the read-only capability surface a gathering helper is spawned with, partial-return fan-out, and the gather-only external-command tier."
timestamp: 2026-07-21
bee:
  id: doctrine-layer-helper-classes-and-transports
  lifecycle: active
  areas: [doctrine-layer]
  required_context: [areas/doctrine-layer/overview.md, areas/doctrine-layer/delegation-threshold.md]
  decisions: ["040f8ef0 (read-only analyst spawn + partial-return fan-out, B7/R11)", D1/D2/D3 delegation contract]
  sources: ["compounding-fanout-hardening (cell cfh-1, 2026-07-17, flushed capture stub d3417cb2)", "advisor-and-orchestration Slice 2A-ii (cells ao-2aii-1/ao-2aii-2, 2026-07-17)", "advisor-and-orchestration Slice 2A-iii (cells ao-2aiii-1/ao-2aiii-2 — dispatch-boundary enforcement + gather-purpose routing prose, 2026-07-17)", "advisor-and-orchestration Slice 5 (cell ao-5-1 — execution-worker class, tiny/small single-worker execution, AO14, 2026-07-17)", "docs/specs/doctrine-layer.md#B7", "docs/specs/doctrine-layer.md#B7a", "docs/specs/doctrine-layer.md#B8", "docs/specs/doctrine-layer.md#R11", "docs/specs/doctrine-layer.md#R12", "docs/specs/doctrine-layer.md#P6", "docs/specs/doctrine-layer.md#P7"]
  authoritative_for: "doctrine-layer: helper classes and transports"
---

# Doctrine Layer — Helper Classes and Transports

Once a step crosses the delegation threshold, two questions remain: *what class
of helper* receives it, and *what may that helper do*. The prohibition on
writing is carried by a capability boundary, never by a sentence in a prompt;
and a helper tier backed by an external command is a gather transport only.

## Behaviors & Operations

**B7 — A gathering helper is spawned without write ability, and a fan-out
synthesizes from what returned rather than waiting for a full set.** Trigger:
one or more helpers are dispatched to read and analyze — the settled case is the
closing stage's parallel analyst fan-out. What happens: each such helper runs
with a read-only capability surface — the prohibition on writing is carried by
what the helper *can do*, never by a sentence in its prompt, because a prompt
sentence is advice and a capability boundary is a wall (observed: an analyst
told "write no files" in prose implemented and committed source unrequested). A
dispatch that fails at creation is surfaced and re-dispatched exactly once; an
identical second failure ends retrying, and synthesis proceeds from whichever
helpers did return. Synthesis never requires all-of-N returns. What each actor
observes: the orchestrator never hangs waiting on a fixed helper count
(observed: a session stuck indefinitely "waiting for 3 background agents" when
one dispatch had died at creation), and no gathering helper can modify the
project no matter what its instructions say (decision 040f8ef0).

**B7a — Two helper classes, distinguished by authority and state effects.** The
delegation layer names exactly two worker classes. An **I/O-offload helper**
(gather/extract/review) holds no authority and mutates no workflow state: it
never registers, never reserves, never caps — it returns a digest and vanishes.
An **execution worker** implements exactly one assigned unit of work: it
registers in the worker registry, reserves the files it will touch, and its
result feeds a cap — and since the smallest lanes also execute through one such
dispatched worker, "zero subagents" for a small piece of work means zero
*ceremony* helpers (reviewers, panels), never zero I/O helpers and never zero
execution workers. Independent reviewers and checkers are neither class: they
are review-class dispatches with no execution authority. The class is defined
by what the dispatch may *do*, never by which mechanism launched it. The
orchestrator authors the smallest lanes' completion report itself, from the
worker's verbatim diff plus the orchestrator's own fresh verification re-run
(AO14; ao-5-1, 2026-07-17).

**B8 — A helper tier backed by an external command serves gathers only, and its
output is accepted only between declared markers.** Trigger: a helper tier is
configured as an external command-line assistant (a different vendor's model
driven through its own command) and a dispatch resolves that tier. What happens:
resolving it **for a read-only gather** yields the external command; resolving it
**for unit execution** yields a typed refusal from the resolution machinery
itself — the boundary is enforced in code, not by guidance text, because an
external command runs in its own working directory where the workflow's own
bookkeeping would land in a phantom copy and every record would be written where
the workflow never reads (observed by probe). A gather through an external
command runs the configured command **exactly as written** (nothing appended),
feeds the task in on standard input, hands the command only absolute locations,
and treats the printed output **between declared framing markers** as the digest;
output missing its markers, or an empty digest, is a **failed run surfaced
loudly** — never accepted as silent success. Such a gather creates no work unit,
no reservation, and no worker registration. Known gap, assigned not omitted:
these runs do not yet appear in the dispatch audit log. What each actor observes:
the human's configured command is the whole invocation contract; configuration
checking refuses a command with no declared prompt transport, any command
carrying a known auto-approve/bypass token, and — on **advice-class** slots
(adviser, reviewer), which are read-only by rule — a command carrying a known
write-granting sandbox token (a blocklist of known-bad tokens, stated honestly
as such, never a positive read-only guarantee); and unit execution can never
route to the external path until it earns its own proof (decisions 34398e69,
4ec5be1a; advice-class refusal per AO8, cell ao-2b-2).

## Business Rules

- **R11** — A read-and-analyze helper is dispatched with a read-only capability
  surface, never merely a read-only instruction; and a parallel fan-out's
  synthesis proceeds from partial returns — one re-dispatch per failed creation,
  then synthesize from what came back, never an unbounded wait for all-of-N
  (040f8ef0).
- **R12** — An external-command helper tier is gather-only: the resolution
  machinery returns a typed refusal when such a tier is resolved for unit
  execution, and a caller must explicitly declare the gather purpose to receive
  the command. Purpose defaults to the refused side; malformed purpose values
  fail safe to refusal, and the refusal is a returned value, never a crash — the
  resolution sits under a fail-open guard path (34398e69, 4ec5be1a). The
  boundary is also enforced at the dispatch checkpoint: an in-family helper
  dispatch declaring an external-command tier is refused with a corrective
  message routing to the gather path, and the routing procedures teach the
  explicit gather-purpose form as the documented way to reach the command
  (2A-iii, 6b155218).

## Pointers (implementation)

- B7/R11's settled case: `skills/bee-compounding/SKILL.md` §2 (analysts pinned
  to the runtime's read-only agent type — Claude Code `Explore` — with
  event-driven wait, one re-dispatch, partial-return synthesis); RED→GREEN
  record in `skills/bee-compounding/CREATION-LOG.md` amendment 2026-07-17.
- B8/R12 implementation: `resolveTier(root, slot, runtime, {for:'gather'|'cell'})`
  in `packages/bee/lib/state.mjs` (default `'cell'`, refusal
  `{type:'refused', reason:'cli_tier_gather_only'}`); the cli gather branch +
  `<<<BEE_DIGEST … BEE_DIGEST>>>` delimiter contract in
  `skills/bee-hive/references/routing-and-contracts.md` § Delegation contract,
  census-anchored in `templates/tests/test_lib.mjs`; feature
  advisor-and-orchestration Slice 2A-ii (cells ao-2aii-1/ao-2aii-2, 2026-07-17).
