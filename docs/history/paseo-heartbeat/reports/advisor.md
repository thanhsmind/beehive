# paseo-heartbeat — advisor consult

This feature is slice 2 of paseo-pi. Its consult is the five-seat paseo-pi
hat wave of 2026-10-05: `docs/history/paseo-pi/reports/hat-wave.md`,
section "Next slices". Named deviation: no second wave runs for this slice.

The four slice-2 findings and where the plan takes them:

- Leader-only guard (hat-risks 8): `isPaseoLeader` — phb-1, phb-2.
- One tick at a time (hat-risks 8): the tick-running flag — phb-2.
- No-news turn exempt from the continuation nudge (hat-risks 9): phb-2.
- A minimal, quiet no-news turn (hat-user-impact 2): the one-word ok text —
  phb-1.

New fact found while planning: the result drain already wakes the leader
for worker results every 2 seconds (`result-inbox.ts:49`), so the heartbeat
tick runs only the broker.

Dismissed: none.
