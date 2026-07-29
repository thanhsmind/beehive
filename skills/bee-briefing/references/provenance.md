# Provenance — bee-briefing body rules

The body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Briefing is invoked conditionally — `bee-planning` only calls it where the fan-out table earns a brief | decision 0009 | The second-load cost of a consolidated doc is only worth paying where the fan-out table says so; below high-risk the caller may skip it entirely |
| `small`-lane mini-brief renders only when a `plan.md` exists | planning D4 | `plan.md` itself is opt-in for `small` (D3/D4); a mini-brief has nothing to consolidate without it |
| Proposed Approach projects from `approach.md`, or from `plan.md`'s `## Approach` section when folded in | decision 0009 | Separate discovery/approach files exist only for L2+ discovery or high-risk; else the approach lives inside `plan.md` |
| The section→source projection walk and walkthrough reconstruction dispatch as generation-tier I/O workers; the two authored sections stay on the session model | Delegation contract D2/D3 | Mechanical gather/render steps fan out down-tier; decide-altitude (the two authored sections) never delegates |
| The brief is a projection of the truth artifacts, never their sole change site | extends D12 (Projection Rule) | Truth lives in `CONTEXT.md`/`plan.md`/cells/reports; the human-layer document never overrides its own sources |
| Drift fires on cell changes only, since `plan.md` freezes at Gate 2 | planning D1; briefing D9 (prose v1) | Once `plan.md` is frozen at Gate 2 it can no longer drift; only the current-slice cells can still change after Gate 2 approval |
| Walkthrough quiz offer (3-5 questions, sourced only from the walkthrough's own sections) | P10, decision 0020 | Mechanizes the gate litmus — "a gate the user cannot restate is dead" — without ever blocking or being forced |
