# Gate 1 — design-os-checks

- Brief: `docs/history/design-os-checks/CONTEXT.md` (D1-D11).
- Approval: auto, gate bypass `full`.
- Recommendation taken: lock D1-D11 as written. D9 is the named guess: a 0.6.2 baseline
  ratchet that counts only the families the baseline knows, so the ds-usage family added
  in 0.6.2 does not fail every host's UI cells until a person re-records.
- Evidence: a copy of anphabe-goglbe (51 pages). `ui gate` 0.6.1 — no regressions.
  `ui gate` 0.6.2 — most pages rise from 0 to up to 330 errors, all in `ds-usage`.
