# Plan check — hat wave, design-os-checks

Seats: hat-facts-gaps, hat-alternatives, hat-user-impact (opus, advisor kind). All three hit the
5-hour rate limit on first dispatch and were resumed after it reset.

```text
PLAN CHECK
Work: slice 1 (doc-1, doc-2, doc-3)

STRUCTURE
BLOCKERS:
- coverage: D3/D5 left web and figma packs uncovered / plan "Out of scope" / FIXED: doc-3 adds web and figma lists; parity scans every pack, no contract counts as empty.
- deps: doc-2 verify filter `design` also ran doc-1 and doc-3 tests / plan doc-2 verify / FIXED: filter `design_rule`, tests prefixed design_rule_; shared build unit note in each cell.
WARNINGS:
- coverage: 76ac close and core-gap backlog had no owner / FIXED: leader steps b and c.
- coverage: D9 note narrowed to "has errors" / FIXED: note on any family the baseline lacks.
- completeness: doc-3 verify did not prove pin, UPSTREAM.json, a7ad5f1 removal / FIXED: rg checks added to verify.
- completeness: advisory line could count as a failure / FIXED: action and truth pin it out of regressions and --write.
- completeness: artifact linter failure and empty artifacts JSON unspecified / FIXED in doc-1 action.
- key links: area spec had no owner / FIXED: leader step d (capture).

CELLS  (reviewed: 3)
CRITICAL FLAGS:
- doc-1: tenant-lint JSON has no warningCount (hat-alternatives, ran) / FIXED: missing counts as 0.
- doc-1: old baseline with marked artifacts goes red before anyone can re-record (hat-user-impact) / FIXED: new D12, advisory until re-record.
MINOR FLAGS:
- doc-3: worker never learns the markers or flow file name / FIXED: charts SKILL.md states them.
- doc-1: advisory line placement / FIXED: once per run, before the summary, also under --write.
- gray, recorded not fixed: --json has no structured field for uncounted families; last-run.json has no artifact count; a host's own design.ui_globs does not gain flow files (D10 by design).
CLEAN CELLS: doc-2 (after the verify fix)

SUMMARY: Two structural blockers and two critical cell flags, all fixed in plan.md before the gate.
Kept: marker detection (D6), family sum (D9), bee-owned renderer, three cells. Rejected cheaper
paths are listed in plan.md "Approach".
```
