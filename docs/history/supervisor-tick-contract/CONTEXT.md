# Supervisor Tick Contract — Context

**Feature slug:** supervisor-tick-contract
**Date:** 2026-08-31
**Shaping session:** complete (foreign-origin spec drop, Qualified per decision 12deaa34 — evidence is the spec file itself, no interactive interview needed)
**Scope:** Quick
**Domain types:** READ | ORGANIZE

## Feature Boundary

Close out PBI `sup-20260831-7f3a` (the bee-side half of "make `bee supervisor`
actually run") by resolving its four named gaps, and land the resolution as
docs, one knowledge-area addition, and decision-log records — no product
source code changes, because investigation showed two of the four gaps are
already shipped and the other two resolve to documentation/decision work,
not new code.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Gap 1 (mailbox write-only) is stale. `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` already reads `pending_delivery_for_session` and delivers pending mailbox rows at the target session's next turn boundary, on both the ordinary and linked-worktree paths, tested. No code change. (decision `50d29046`) | The xia's `rg "supervisor pending"` search missed the reader because it is named `pending_delivery_for_session`, not that literal string. |
| D2 | Gap 2 (`--role supervisor` wiring) is stale. `packages/bee-rs/crates/bee/src/herding/control_loop.rs` already parses `--role dispatch\|merge\|supervisor`, resolves `models.claude.supervisor`, and spawns `supervisor-prompt.md` on `--once`, tested. No code change. (decision `80aa9db1`) | The spec checked bee 2.29.0 (waggledance's vendored copy); beehive main is 2.30.0. |
| D3 | Gap 3 (externally-triggered tick contract) closes by documenting the existing `bee herding control-loop --role supervisor --once --main-root PATH` invocation as the external-trigger primitive — a plain subprocess spawn, no tmux/herdr pane needed, no new verb, no signature change to `bee supervisor`'s 10 verbs. (decision `051b87ba`) | Matches the pattern waggledance already uses elsewhere: spawn the target repo's own bee CLI at its root. A second trigger mechanism would fragment the surface waggledance's own PBI `sup-20260831-b2e1` is building on. |
| D4 | Gap 4 (notebook shape) closes with no new CLI validation. `supervisor-prompt.md`'s two-sentence, name-what-you-saw wording rule is a deliberate causal-lite substitute for Demonthorn §4.2's seven-field shape, not an oversight — c80debd7 chose terseness on purpose. A parked PBI (`p-bd1debcd`) captures the heavier shape as future work, conditional on the terse shape proving insufficient. (decision `17e9c1f6`) | Adding mandatory O/E/M/I/Q/R/Esc fields now would fight a locked, high-confidence prior decision without new user sign-off, and the spec itself warned against redundant validation. |

### Agent's Discretion

The relayed instruction ("execute sup-20260831-7f3a") explicitly delegated
triage order and scope: "Bốn khoảng trống trong spec. Tự triage thứ tự và
phạm vi — bạn là lead, không phải tôi." All four gap resolutions above are
this session's own judgment calls, made from direct code/doc inspection
rather than trusting the spec's claims at face value.

## Terms

| Term | Meaning in this feature |
|------|--------------------------|
| External-trigger contract | The documented behavior of `bee herding control-loop --role supervisor --once --main-root PATH` when invoked by a caller outside the target repo's own agent session (e.g. waggledance) — not a new CLI surface. |

## Existing Code Context

### Reusable Assets

- `packages/bee-rs/crates/bee/src/hooks/prompt_context.rs` — turn-boundary mailbox delivery (D1), already complete.
- `packages/bee-rs/crates/bee/src/herding/control_loop.rs` — `Role::Supervisor`, `--once`, `--main-root`, subprocess spawn via `Command::new(argv[0]).spawn()` (D2, D3), already complete.
- `skills/bee-herding/references/supervisor-prompt.md` — the wording contract D4 evaluates.
- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` — the knowledge-area page this feature extends with the D3 contract writeup.

### Integration Points

- `docs/knowledge/areas/bee-herding/the-supervisor-observer-and-its-interventions.md` — add an "External trigger (cross-repo callers)" section documenting D3.
- `docs/discovery/spec-drops/sup-20260831-7f3a.md` — the spec this feature resolves; no edit needed, it is provenance, not a living doc.

## Canonical References

- `docs/discovery/slp-human-up/tickets/004-waggledance-supervisor-feasibility.md` — prior research establishing the "spawn the target repo's own bee CLI at its root" pattern D3 reuses.
- `/home/thanhsmind/Projects/goglbe/waggledance/docs/history/research/demonthorn-supervisor-xia.md` — the xia this spec drop originated from (read-only, external repo).

## Outstanding Questions

None blocking planning. The only genuinely open design question (the notebook's causal shape, gap 4) is resolved by deferral, not left open — see `p-bd1debcd`.

## Deferred Ideas

- Enforcing Demonthorn §4.2's full seven-field notebook shape in `bee supervisor record` — filed as parked PBI `p-bd1debcd`, revisit only if the current terse shape proves insufficient in practice.
