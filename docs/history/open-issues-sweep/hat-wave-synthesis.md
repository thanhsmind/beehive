# Hat wave synthesis: open-issues-sweep

Seats: hat-facts-gaps, hat-alternatives, hat-user-impact (opus advisors).

```text
PLAN CHECK
Work: slice 1, cells ois-1..ois-10

STRUCTURE
BLOCKERS:
- ois-2 root cause / unclaim already calls release_claim_file_best_effort through mutate_cell; the release returns silently on a stale .adopting gate (.bee/claims/det-2.adopting dated at the #108 claim) / rewritten: break stale gates in claims.rs, red test with a seeded stale gate
- ois-8 files / the reminder renders in verbs/status_full/orient.rs and hooks/session_preamble/render.rs / added both plus the preamble tests
- ois-10 anchor and verify / flags live in generated/registry_payload.json and PINNED_FLAG_COUNT pins the distinct flags / action and verify fixed
WARNINGS:
- ois-7 header sits after frontmatter in docs/knowledge/index.md; re-export path via write_guard mod / applied
- ois-3 D4 signal / use the follow-up ls call result, not a TCP probe / applied
- ois-4 stale door-audit lines in roots.rs and an old pinned test / roots.rs added, test inverted; preamble read split named as a known limit
- ois-6 control state naming another feature / handed state decides / applied
- ois-8 dead regen branch / removed
- ois-9 mtime unreliable and blocked items kept onboard at changes_needed / recorded hashes, mtime only as first-run fallback, blocked items excluded / applied
- D3 worker could release the leader session / session release is a no-op in a herding worker / applied
- D5 base: one flaky heartbeat test on the 2026-10-17 push run; latest main CI green

CELLS  (reviewed: 10)
CRITICAL FLAGS: none left after the fixes above
MINOR FLAGS: ois-5 FIX names the store id vs D-label; ois-8 reminder wording says apply or skip first (both applied)
CLEAN CELLS: ois-1, ois-5, ois-6

SUMMARY: Coverage of D1-D12 is complete. Three blockers were real and are fixed in the plan; the rest were folded into cell actions. Not taken: listing frozen-plan deferral lines as non-blocking at close (scope), a live-owner check under unclaim --force (force is the operator's explicit override).
```
