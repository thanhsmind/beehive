# knowledge-usable — plan

Lane: standard (class feature, covered-contract-change: preamble/close outputs existing tests assert). Source of scope: CONTEXT.md U1-U9 (each cites its PBI). One umbrella feature — named deviation from one-feature-per-PBI, logged; cells stay small and independently capped.

## Shape — 9 cells, 3 waves (file-overlap driven)

Wave 1 (no file overlap):
- ku-1 (U1) preamble pull line — session_preamble/render.rs (+test)
- ku-2 (U2) dangling-path finding — knowledge/check.rs (+test)
- ku-3 (U3) flush pressure — session_close/nudges.rs, config validate, close door detail (+test)
- ku-9 (U9) bootstrap verb — knowledge/bootstrap.rs (new), routing.rs, registry (+test)

Wave 2 (after wave 1; overlaps preamble/close/knowledge files):
- ku-4 (U4) promote convergence — drivers/close.rs, preamble promote lines (+test)
- ku-5 (U5) anchor widening — knowledge/anchor.rs, session_preamble/budget.rs (+test)
- ku-6 (U6) critical bar + re-grade — okf-profile area spec, docs/knowledge/patterns/* frontmatter, index regen
- ku-7 (U7) close pattern check — drivers/close.rs (after ku-4), cells finish record (+test)

Wave 3:
- ku-8 (U8) recurrence report — knowledge/report.rs (new), signatures on ≥3 criticals, registry (+test)

Verify per cell: full cargo suite (declared commands.test). ku-6 additionally: `bee knowledge check` clean + index regenerated.

## Smaller-path check

Could U5+U6 collapse into one cell? No — different files (code vs bundle data), different failure modes; separate caps keep the re-grade reviewable. Could U7 skip close integration and be a standalone verb? It would lose the moment where evidence gathers — the door is the point. Shape stands.

## Risk

- close.rs is touched by ku-4 then ku-7 — serialized inside wave 2 (ku-7 deps ku-4).
- Registry payload is hand-maintained JSON — ku-8/ku-9 keep tests/registry_contracts.rs green (ks-1 precedent).
- ku-6 is judgment over 101 files — per-pattern reasons in the commit body; target ≤~30 critical.

## Rollback

Each cell is one commit on wt/knowledge-search; revert cell-wise. Verbs are additive; preamble/close line changes revert clean.
