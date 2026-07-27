# CONTEXT — rust-port-ledgers

**This feature has no decisions of its own.** It is slice 4 of the `rust-port` epic, opened as
a separate workflow only because the `rust-port` workflow reached `compounding-complete` and
`state start-feature` refuses to reopen a live workflow for the same slug.

## Source of truth

`docs/history/rust-port/CONTEXT.md` — all locked decisions D1–D11 apply verbatim and are cited
by D-ID from this feature's plan and cells. Nothing here supersedes, narrows, or reinterprets
them.

The ones this slice leans on hardest:

- **D3** — every on-disk storage format under `.bee/` stays byte-compatible.
- **D7** — port is incremental behind three harnesses: (a) CLI parity, (b) hook conformance,
  (c) lock conformance. A group flips only when its suites are green. **This slice runs (a) and
  (c); it performs no flip.**
- **D9** — the cross-process lock/lease protocol is part of the frozen contract; mjs and Rust
  must contend the same locks safely during the port.
- **D11** — `.bee/bin/bee.mjs` remains the entry point for the whole port window.

## Standing constraint carried from slice 1

> Ported hooks and groups stay **DARK** (no wiring flip) until the dedicated flip slice.
> — `docs/history/rust-port/reports/validation-slice1.md:26`, restated at `validation-slice2.md:46`

## Outstanding questions

Carried in `plan.md` → "Open questions carried into validating". The unresolved epic-level
questions (Windows contingency for D8, the D6 graph-engine shape, `worktree merge` budget
feasibility) belong to later slices and are untouched here.
