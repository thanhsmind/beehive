# diet-4 — bee-swarming thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-swarming/SKILL.md` rewritten to the D7 thin-body
doctrine: 24,676 → **8,187 bytes** (≤ 8,192). All exiled text landed in
`skills/bee-swarming/references/swarming-reference.md` (new "Single
execution worker in full", "Operating Contract in full", and
"Fresh-session handoff in full" sections; the pre-existing "Native Worktree
Integration Transaction" and "Threat model and protected attestation"
sections already carried the worktree-dispatch detail the body used to
restate, so that block became a pointer with no new reference text needed);
new `references/provenance.md` maps every body rule to its decision IDs.
Baseline lowered to 8187, `bee-swarming` appended to `migrated[]`,
`notes["bee-swarming"]` deleted. Regen obligation ran in-cell: plugin mirror
render, onboard `--apply`, release manifest `--write`/`--check`. Full verify
chain green.

**Files:** `skills/bee-swarming/SKILL.md`,
`skills/bee-swarming/references/swarming-reference.md`,
`skills/bee-swarming/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-4.json`.

## Side-by-side behavior checks (P5 acceptance)

### s1 — lane scaling for tiny/small routes identically

Before (`Single execution worker (tiny/small lanes)`):

> For `tiny` and `small`, the merged Gate 2+3 question and the frozen-judge
> check stay with the orchestrator, but implementation itself runs through
> **one dispatched execution worker** (AO14) — a lighter direct Agent
> dispatch under the same execution contract as a swarm worker (same worker
> prompt template, same status-token protocol, same reservation and cap
> discipline), never a full bee-swarming wave: no wave analysis, no
> reviewers, no panels. …

After (`Lane scaling — single worker vs full wave`):

> | `tiny`/`small` | Merged Gate 2+3 + frozen-judge stay with the
> orchestrator; implementation runs through **one dispatched execution
> worker** under the full execution contract (same template, status tokens,
> reservation/cap discipline) — never a wave: no wave analysis, no
> reviewers, no panels. …

Same trigger (tiny/small), same merged-gate ownership, same single-worker
mechanism (same template/status-tokens/reservation-cap contract), same
"never a wave" prohibition. A tiny bugfix cell still dispatches exactly one
execution worker under the identical contract.

### s2 — rare branch: rescue-ladder consult-budget rule survives verbatim

Before (`[BLOCKED] Rescue Ladder`):

> A `[BLOCKED]` arriving here already spent its consult budget for that
> claim (D1/D3) — the 2-consult cap is per claim, not per cell lifetime, so
> a rung-1 (more context) re-dispatch grants the re-claimed cell a **fresh**
> budget. The ladder's three rungs are otherwise byte-unchanged.

After:

> A `[BLOCKED]` here already spent its consult budget for that claim — the
> cap is per claim, not per cell lifetime, so a rung-1 re-dispatch grants
> the re-claimed cell a fresh budget. A reservation conflict is rescued by
> adjusting reservations or cell scope — never by telling workers to be
> careful.

Rare-branch rule (a `[BLOCKED]` re-entering the ladder after a prior
consult) is byte-for-byte the same claim: per-claim not per-cell-lifetime,
fresh budget on a rung-1 re-dispatch. Decision IDs (D1/D3) moved to
`references/provenance.md` ("`[BLOCKED]` here already spent its claim
consult budget…" row); meaning unchanged.

### s3 — worktree dispatch eligibility reachable via the new pointer

Before (~1,900-char body section, `Opt-in Native Worktree Dispatch` +
`Protected pre-dispatch attestation` + `Worktree integration — attestation
first`):

> Normal native isolation is eligible only for an enabled Claude Code wave
> with at least two workers; tiny/small's single execution-worker dispatch
> and single-worker waves stay in the shared checkout. The enabling
> implementation itself is serialized in that checkout as
> `worktree-isolation-1 → worktree-isolation-2 → worktree-isolation-3` …
> `worktree-isolation-4` is the sole validation-only one-worker exception …
> [full attestation field list + three-check gate with typed halts]

After (~500-char pointer):

> Opt-in Git-consistency mode for an enabled wave of 2+ workers (never
> tiny/small's single-worker dispatch); enabling cells
> `worktree-isolation-1..3` serialize first, `worktree-isolation-4` is the
> sole one-worker validation exception. Protected pre-dispatch attestation,
> the three-check re-attest gate (identity/ancestry/diff containment, typed
> halts), and the full threat model: `references/swarming-reference.md`
> ("Native Worktree Integration Transaction", "Threat model and protected
> attestation").

The pointer resolves to sections that were **already present** in
`references/swarming-reference.md` before this migration (from an earlier
feature) and independently state the identical eligibility rule, attestation
field list, and three-check gate (identity/ancestry/diff-containment, same
typed halts `WORKTREE_IDENTITY_MISMATCH` / `WORKTREE_BASE_ANCESTRY_MISMATCH`
/ `WORKTREE_RESERVED_DIFF_MISMATCH`) — the body section was a duplicate
restatement; deleting the duplicate and pointing at the existing full
protocol drops zero information.

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` matches in the body (`skill_budget_fence.mjs` bare run: 0 findings); the rule → decision-ID map is `references/provenance.md` (20 rows).
- `skill_lint` anchor integrity: both quoted-heading pointers ("Single execution worker in full", "Fresh-session handoff in full") plus the multi-heading pointers ("Native Worktree Integration Transaction"/"Threat model and protected attestation", "Operating Contract in full"/"Model Tiers — Config-Driven, Runtime-Keyed") all resolve.
- `bee:only claude` / `bee:only codex` marker grammar stays valid — `skill_render` 27/27 green, including the gate test that refuses a malformed marker with zero writes.
- Deviation (recorded in trace): none — no architectural gaps or bugs found in the touched scope; the pre-existing duplication between the body's worktree-dispatch section and the reference's already-complete protocol was resolved by pointer (not a deviation, since D7/D8 explicitly call for exiling restated detail to an existing reachable reference).
