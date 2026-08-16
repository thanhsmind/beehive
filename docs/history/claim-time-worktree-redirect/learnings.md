# Learnings — claim-time-worktree-redirect (2026-08-16)

Thin harvest: 2 capped cells, no deviations, no red runs recorded.

- Redirect at claim time beats refusal at write time: `cells claim`/`claim-next`
  now annotate success output with the granted worktree root, so a session
  standing in main learns the correct cwd before it dispatches a worker —
  instead of the worker dying on the write guard later. Annotation is
  fail-open on unresolvable grants; it informs, never refuses.
- The doctrine half (worker cwd self-check, enter-the-worktree) had to land in
  three instruction surfaces at once (bee-swarming, AGENTS.block, worker-cell
  template) plus the knowledge doc — one more instance of the known pattern
  that instruction-layer changes are multi-surface and rot silently when one
  surface is missed.
- Pattern verdicts at close: source-shipped-without-reinstall respected
  (2.6.6 released), shared-index vs path-scoped commit respected,
  refusal-verb-family respected (both claim verbs annotated), arm-refusal-
  after-remedy not applicable (no refusal added).

Nothing promoted: the area bullet was already synced into
docs/knowledge/areas/worktree-parallelism during cwr-2 itself.
