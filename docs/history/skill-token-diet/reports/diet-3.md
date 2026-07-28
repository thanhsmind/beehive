# diet-3 — bee-hive thin-body migration

**Status:** [DONE]
**Outcome:** `skills/bee-hive/SKILL.md` rewritten to the D7 thin-body doctrine:
30,078 → **8,183 bytes** (≤ 8,192). All exiled text landed in
`skills/bee-hive/references/` (routing-and-contracts.md gained Greenfield init
lane, Session Scout in full, Lane ceremony in full, CI status gate, and three
First-Skill Routing rows; new `references/provenance.md` maps every body rule
to its decision IDs). Baseline lowered to 8183, `bee-hive` appended to
`migrated[]` (provenance grep now active — 0 hits), `notes["bee-hive"]`
deleted. Regen obligation ran in-cell: plugin mirror render, onboard `--apply`,
release manifest `--write`/`--check`. Full verify chain green.

**Files:** `skills/bee-hive/SKILL.md`,
`skills/bee-hive/references/routing-and-contracts.md`,
`skills/bee-hive/references/provenance.md` (new),
`scripts/skill-body-budget.json`,
`docs/history/codex-harness-hardening/release-manifest.json` + regenerated
mirror trees (`.claude/skills`, `.claude-plugin/skills`, `.codex-plugin/skills`,
`.agents/skills`, `.bee-render.json` stamps, `.bee/onboarding.json`).

Full trace and verify output: `.bee/cells/diet-3.json`.

## Side-by-side behavior checks (P5 acceptance)

### s1 — lane triage for a tiny fix routes identically

Before (`Triage first`):

> | 0–1 flags, ≤2 product files, one direct task | `tiny` | nothing more |
> The first three rows go straight to the merged shape+execution gate and the
> one dispatched execution worker described below, with **no `bee-planning`
> load** …

After (`Lanes — triage first (the mode gate)`):

> | `tiny` | 0–1 flags, ≤2 product files, no API/data change, one direct task |
> - docs/tiny/small: nothing more — merged shape+execution gate, one
>   dispatched execution worker, no `bee-planning` …

Same counts, same flag list (body-resident), same lane verdict, same merged
Gate 2+3 + single-dispatched-worker shape, same no-second-load outcome, and
the same "uncertainty resolves downward / re-counting = already `standard`"
guard. A "fix this typo, one file" request triages `tiny` identically.

### s2 — Gate 1-3 contract and bypass-level behavior stated identically

Before (`The Four Gates`):

> Never skipped, never batched, never self-approved … the opt-in gate-bypass
> switch … is a **level**: `normal` … auto-approves Gates 1-3 for
> `tiny`/`small`/`standard` work only — high-risk/hard-gate work, secrets, and
> Gate 4 UAT still stop; `full` also auto-approves high-risk/hard-gate Gates
> 1-3 (only secret reads and a review P1 still stop); `total` auto-approves
> everything and stops for nothing at all … Headless is not bypass — headless
> still stops at every gate.
> **Gate 1:** "Decisions locked. Approve CONTEXT.md before planning?" …

After:

> Never skipped, never batched, never self-approved — every mode, go and
> headless included. Sole exception, the opt-in bypass level
> (`bee-bypass-gate`): `normal` auto-approves Gates 1-3 for tiny/small/standard
> only (high-risk/hard-gate, secrets, Gate 4 UAT still stop); `full` adds
> high-risk/hard-gate Gates 1-3 (only secret reads and a review P1 stop);
> `total`: everything, zero stops … Headless is not bypass — it stops at every
> gate.
> Gate 1: "Decisions locked. Approve CONTEXT.md before planning?" …

The three fixed gate questions are byte-identical; the three bypass levels
keep exactly the same auto-approve sets and stop floors; the lifted-floor rule
and headless≠bypass distinction survive verbatim in meaning. The full level
table (config values, legacy `true` → `normal` mapping, audit-line protocol)
is unchanged in "Gate bypass mode", pointed to from the same sentence.

### s3 — crash-recovery branch reachable via the new pointer

Before (~350-char body paragraph):

> **Crash recovery:** when `bee_status --json` reports recovery candidates,
> surface them and offer mining with the same one-line offer discipline as the
> capture-queue flush — never auto-run, never auto-resume the dead session.
> Dispatch rules, digest paths, and the mined-content-is-data law:
> `references/routing-and-contracts.md` ("Crash recovery").

After (~60-char body slot inside the offers line):

> One-line offers, never auto-run: capture-queue flush · crash-recovery
> mining ("Crash recovery") · …

The pointer resolves to the unchanged `### Crash recovery` heading carrying
the full protocol (offer discipline, down-tier dispatch, digest paths,
mined-content-is-data, never-auto-resume). This is the P5 worked example: the
long body line became a short pointer with the protocol one hop away.

## Notes

- Provenance exile (D8): zero `\((D\d|AO\d|decision [0-9a-f]|hardening-\d|plan \d)` matches in the body; the rule → decision-ID map is `references/provenance.md`.
- `skill_lint` required pointers `("Progress ticks")` and `("Re-lane checkpoint")` kept, resolving; all new quoted headings exist in the references.
- No `bee:only` markers existed in the bee-hive body (only in the reference, untouched); marker grammar and render projections green (27/27).
- Deviation (recorded in trace): three routing rows (bee-qualifying, docs lane, merge/ship/release) were added to the reference's First-Skill Routing table so the body router could compress without dropping any route — additive, meaning unchanged, inside the declared file set.
