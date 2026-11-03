# feature

1. Name the DATA shape first — the record the feature turns on, before any
   verb, flag or screen reads it.
2. Build a walking skeleton end to end, with no stub left on that path —
   wiring breaks then show up first, while they are still cheap.
3. Build the CURRENT slice only, because a cell for a later slice is planned on
   guesses; headline the rest and leave it out of the cells.
4. Prove the user-visible path, not only the unit sitting under it — the user
   walks the path, never the unit.
5. Sync `docs/knowledge/` in the SAME change when behavior moves — it is the
   state layer, not a write-up.
6. Model an event-order flow with the `statechart` skill, so that TLC walks
   every step order and finds the stuck state a unit test never reaches —
   payment hold, capture and refund with retry, a message queue, a re-run or
   idempotent job, a multi-state workflow, a lock or lease. Do it before the
   flow's code, when the host has that skill. Each break it reports stays a
   guess until a test on the real code fails the same way. With no
   `statechart` skill on the host, skip this step and say so.

A green unit behind a path nobody can walk is not a result.

Proof line: the user-visible path driven, with its evidence (`green:live`).
