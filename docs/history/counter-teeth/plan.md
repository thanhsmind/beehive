# counter-teeth — plan

Lane: standard (2 flags: public-contracts, proof-weakening; ~8 product
files). Route record: absent by recorded deviation 3baa41f6 (route verb
broken — see D5). All decisions cited from CONTEXT.md D1-D6.

## Shape

Six cells, one slice. Order matters where D5 forces it (ct-1 before ct-5)
and where the red base forces it (ct-0 first). Each cell follows D6's
sequencing law — a test proving the counter/condition computes correctly
lands BEFORE the flip to refusal, in the same cell, red-first.

- **ct-0 — fix-first: worktree-red test.**
  `hooks::state_sync::tests::renew_cross_worktree_holds_renews_active_session_rows_only`
  (state_sync.rs:1439-1483) passes from the main checkout and fails when
  the suite runs from a linked worktree — it resolves main_root from
  process cwd instead of the injected tmp root, violating the "a test run
  under a configuration must assert the configuration is live" pattern.
  Every worktree cap in this feature would go red on it. Make the test
  hermetic (inject the root; no cwd dependence). Evidence: single-test run
  red from beehive--wt--counter-teeth, green from main, same commit.
- **ct-1 — fix route granted-arm (D5, fix-first).** Port the retired Node
  arm of `run_route`: when `code_touching && any_granted_worktree`, resolve
  the grant set; a grant for the TARGET feature routes the worktree block
  to that grant (or allows the set with a worktree notice); a grant for a
  DIFFERENT feature must not poison the call. Kill the `Err2::Ex` bail;
  a genuine refusal must name the actual conflict, never surface as
  `unsupported_argument_shape`. Tests: route --set succeeds with a foreign
  grant present; succeeds with own-feature grant; refusal message contract
  pinned.
- **ct-2 — close refuses uncaptured behavior_change cells (D1).** In
  `bee close`: compute the closing feature's behavior_change cells lacking
  capture; non-empty and no `capture-deferral` decision naming the feature
  → refuse, listing cell ids + both remedies. Tests: red on uncaptured,
  green after capture, green with deferral decision, message contract.
- **ct-3 — orient capture-queue blocker (D2).** In `bee orient`: pending
  stubs ≥ 10 OR oldest pending stub > 7 days → the capture-queue line
  moves from offer to `blockers[]`. Tests: boundary at 9/10 stubs, age
  boundary, flushed stubs never count.
- **ct-4 — cells tier ceiling refusal (D3).** In `bee cells tier`:
  assigning `ceiling` when post-assignment ceiling share of tiered cells
  would exceed 40% refuses, naming current share and threshold; `--reason`
  overrides and is stored on the cell tier record. Tests: boundary at
  exactly 40%, refusal message, reason override persisted.
- **ct-5 — claim route-record deny from second claim (D4).** In
  `cells claim`: no route record → first claim keeps the stderr warning,
  second and later claims refuse naming `bee route --set`. Depends on ct-1
  (remedy must work). Tests: first-claim warn preserved, second-claim
  refusal, claim after route --set passes.

Existing tests that assert current warn-only/advisory behavior are updated
in the same cell that flips the behavior — never weakened, always replaced
by the stronger assertion (proof-weakening flag acknowledged: each such
test change is named in the cell's done-report).

## SMALLER PATH check

Could a cheaper shape honor D1-D6? Dropping ct-1 breaks D5 (deny with a
broken remedy). Collapsing the four flips into one cell couples four
public-contract changes into one commit and one revert unit — worse.
Constants instead of config keys is already the cheaper swap (locked in
D2/D3). PASS — five cells is the smallest honest shape.

## Verify

`commands.test` (cargo test --release) at every cap via `bee cells
finish`; green base established before first claim. Message-contract tests
pin every new refusal string prefix, mirroring the existing refusal
headline test precedent (router.rs).

## Later slices

None — one slice.
