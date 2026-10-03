# Context: pi-slp-dispatch-return

Source: slice 1 of `docs/history/research/seatworks-slp-pi-small-models.md`
(findings 2, 3 and 6). The user said "làm đi" (do it) on 2026-10-03, after the
report named slice 1 as the next step. The target small model and a failing
transcript were not supplied; slice 1 fixes defects that hold on every model,
so it does not wait for them.

## Locked decisions

- **D1 — One order for a herded cell worker** (store `4029e89b`). A cell
  dispatched on a herding channel renders a cell task with no bee
  bookkeeping: no bee-build agent body, no claim, reserve or `cells finish`
  step. The worker implements, runs the narrowest proof, commits once with
  the `cell: <id>` trailer, and returns status, summary, files and a
  three-part proof line in its mailbox result. The leader checks the
  artifacts and caps the cell. Native cell dispatch stays byte-identical.
  Honors herding-worker-standalone D1 and worker-brief-expertise D3
  (`b8e4f393`).
- **D2 — The Pi dispatch tools carry the prepare inputs** (store `ee22fb67`).
  `bee_dispatch` accepts stage, feature, claim and expertise; `bee_advisor`
  accepts stage and feature. Each value goes to `bee dispatch prepare`
  unchanged. Prepare stays the only validator (pi-1-0-upgrade D5).
- **D3 — The verdict tool binds to its job** (store `180c5b3c`). No
  `BEE_HERDING_JOB_ID` → refuse and write nothing; the newest-mailbox
  fallback goes. `status: done` with an empty or blank proof → refuse. A
  result file already present for the current round → refuse. Each refusal
  names one fix.
- **D4 — The leader hears the next step** (store `26c6bd37`). A done result
  with a `cell_id`, injected by the Pi result inbox, gains one fixed line
  outside the data fence: the cell stays claimed until the leader checks
  the artifacts and caps it with `bee cells finish`. Other results render
  as before.

## Out of scope

Plan-revision binding on results and the per-role operation packet are
slice 2 of the research report. Recovery and measurement are slices 3 and 4.
