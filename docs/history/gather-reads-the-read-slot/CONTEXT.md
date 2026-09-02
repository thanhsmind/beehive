# gather-reads-the-read-slot — locked context

Feature: a gather follows the config's `read` slot. The dispatch door already
picks the transport from `.bee/config.json` — a `{kind: herding}` slot opens
a pane, a `{model}` slot returns a subagent — but the `gather` kind still
resolves the tier-era `generation` slot, so the `read` slot every config
describes as "multi-file gathers and codebase scans" never decides a gather.

User's words (2026-09-02): "Goal của tôi luôn là theo config nếu là herding
thì là mở pane. Bình thường thì có thể gọi sub agent. Vậy mới dynamic đúng ý
tôi. Như vậy phải thay thế bỏ đi các cấu hình agent như hiện tại. Xem và chỉnh
lại architech cho đúng."

## Locked decisions

- **D1 (user goal, 2026-09-02):** A gather dispatched with no `--role` asks
  for the READ job: `slot_for_kind("gather")` is `read`, and the tier-shaped
  walk for `read` is `[read, generation]` (`tier_role_list`). NOT through
  `extraction`: extraction was the tier era's cheapest slot and never the
  gather slot, so a legacy host that configures `extraction` + `generation`
  and no `read` must keep its gathers on `generation` byte for byte — a
  walk through `extraction` would move them to the cheapest model silently,
  the exact defect 561e1bda's tail rule exists to prevent. The read-shaped
  CELL list (`cell_role_list("read")` = `[read, extraction, generation]`)
  is a different consumer with a different history (B8 backfilled
  `tier: extraction` cells to `role: read`) and stays as it is; the two
  lists differ on purpose and each says why. Whatever shape the winning
  slot has is obeyed exactly: a herding slot returns the Bash pane command,
  a model slot returns the Agent payload. A host with no `read` key lands on
  `generation` — byte-identical to today's routing.
- **D2:** The name the dispatch travels under is the name that WON the walk
  (marker `[bee-tier: <winner>]`, `economics.logical_tier`) on the two
  paths whose asked name may be one bee ships no built-in model for: the
  cell-role path (already so) and the DEFAULT-GATHER path (new). `read` is
  not in `default_models("claude")`, so a marker saying `read` on a host
  with no `read` key is a name the guard refuses — the winner is always a
  configured or built-in name. Every other path (explicit `--role`,
  reviewer, advisor, escalation) keeps today's marker bytes: their asked
  names are built-in (`review`) or configured by the caller, and the
  guard's `resolve_tier` walks the same `tier_role_list` prepare does, so
  they already resolve. (Hat-wave amendment "record the winner on every
  path" was withdrawn: it would have re-pinned a review-less host's
  reviewer onto `bee-gather` through `pinned_agent_type(winner)`.)
- **D3:** The rendered agent is chosen from the ASKED name and the kind,
  never from the winner: `--kind cell` pins `bee-build`; a `--kind gather`
  with NO `--role` pins `bee-gather` whichever spelling of the read job won;
  everything else keeps `pinned_agent_type(<asked role>)` — so `--role
  extraction` (or `read`) keeps B11's `bee-extract`, `--kind reviewer` keeps
  `bee-review` on a review-less host, `--role <other>` keeps its agent. No
  dispatch that resolves today changes its agent; only the default
  gather's MODEL SLOT moves.
- **D4:** `bee-gather` declares the read job. `AGENT_ROLES_BY_NAME` lists it
  as `["read", "generation"]` (the D1 list); the template body names the
  read role; the opencode render pin and the status drift check follow that
  list through the shared resolver, as they do for every other agent.
  `ROLE_AGENTS` (the guard's agent-unique inverse table) is NOT touched: the
  bare-name fence keeps its historical `generation` read
  (agent-model-unpin D2 and mrs-29 both lean on it), and the residual
  difference — a hand-named `bee-gather` under a split config resolves
  `generation` while `dispatch prepare` resolves `read` — is recorded below
  as a known gap, not closed here.
- **D5:** Every rendered agent template's `description:` opens by naming the
  ONE door: the type is reached through `bee dispatch prepare` (which may
  return a pane command instead), never named by hand. The Claude harness
  shows that description in its agent list, which is where the hand-naming
  in the reported failure started.
- **D7:** The model-guard's bare-dispatch FIX (`bare_dispatch_denied`,
  both branches) leads with `bee dispatch prepare` as the first remedy and
  no longer spells `bee-gather = generation`; the rendered-agent list it
  offers is derived from `ROLE_AGENTS`, never retyped. That text is what
  the caller in the reported failure reads, and today it teaches
  hand-naming as the first move — the habit D5 exists to unteach.
- **D8:** The herding-fallback contract keys on the WINNER, both halves.
  Prepare publishes `payload.fallback` from `default_models(runtime)[winner]`
  — a `read` winner has no built-in default, so `read: {kind: herding,
  fallback: "default"}` publishes no fallback field, exactly as `advisor`
  does today ("no resolvable default leaves the payload byte-identical to a
  slot with no fallback"). The guard's `configured_model_set` mirror walks
  each kind's `tier_role_list` with `resolve_role_named` and looks the
  default up by the name that won, so a host with no `read` key and
  `generation: {kind: herding, fallback: "default"}` keeps admitting
  `sonnet` — prepare and guard cannot disagree (a2f85972's rule: exactly
  the set prepare can publish). Every guard FIX that names a kind for a
  role appends `--role <role>`, so the door it names resolves THAT role on
  a split host rather than the kind's head.
- **D6 (kept, cited):** Claude agent files render unconditionally
  (agent-model-unpin D1/D2, 2026-08-26): a pane-shaped slot never removes a
  file, because `bee-build` also serves the model-shaped `code`/`test` roles
  through the `code → generation` alias. The user's "bỏ đi các cấu hình
  agent" is honoured as: remove the tier-era BINDING (gather = generation),
  not the files — the files carry the read-only tool permissions a bare
  `general-purpose` cannot express.

## Known gaps (named, not closed)

- The model-guard's bare-name branch resolves `bee-gather` as `generation`
  (`role_for_agent`), while `dispatch prepare --kind gather` now resolves
  `read` first. On a host whose `read` and `generation` slots differ, a
  hand-named `bee-gather` and a prepared gather can land on different
  transports. The door text (D5) and the FIX text (D7) both point at
  prepare. Closing the gap is small (~15 lines: a resolver-aware helper
  beside `role_for_agent` walking `AGENT_ROLES_BY_NAME`, one call site at
  the guard's pinned-type branch, one test) and does NOT touch mrs-29's
  `agents_for_role` count — but it flips a verdict class (a hand-named
  `bee-gather` goes from `herding-tier-denied` to ALLOWED on a
  `read: <model>` host), which is an allow-widening with its own matrix.
  Its own cell, after this one lands.
- `docs/product-description/delegation/workers.md` line 38 still describes
  a `model:` frontmatter line the unpin removed; out of scope here.
