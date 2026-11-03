# paseo-answers — advisor consult

This feature is slice 3 of paseo-pi. Its consult is the five-seat paseo-pi
hat wave of 2026-10-05: `docs/history/paseo-pi/reports/hat-wave.md`,
section "Next slices". Named deviation: no second wave runs for this slice.

Slice-3 findings and where the plan takes them:

- A question ends the round, so the answer goes to the same idle agent
  (hat-alternatives fact 3; hat-facts-gaps D): pa-1, pa-2.
- Pi workers keep the steer-file drain (hat-alternatives fact 4): pa-3.
- A Node helper with the daemon WebSocket only for mid-turn steering of a
  non-Pi worker; no Rust WebSocket client (hat-alternatives): pa-3.
- Record the outcome before any fallback, and never fall back to an
  interrupting send (hat-risks 3): pa-2 writes the redispatch record with
  its mode; pa-3 refuses instead of falling back.

Dismissed: hat-value "drop slice 3" — the user asked for it (2026-10-06).
