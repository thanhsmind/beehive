# packages-engine-move-1 — report

**[DONE]**

Moved the onboarding/distribution engine (`onboard_bee.mjs`, `plugin_distribution.mjs`, and 3 test
suites) from `skills/bee-hive/scripts/` to `packages/bee/scripts/`, fixed every live reference to
the old path, and ran the full regen chain. `skills/` is now truly instruction-only. Full verify
green (105 suites), `release_manifest --check` green, acceptance rg clause clean.

Files touched: see `.bee/cells/packages-engine-move-1.json` (`trace.files_changed`) for the
complete list — engine internals, test fixtures, every caller (installers, canary, ledger/bump/okf
scripts, run_verify, bee.mjs/cells.mjs hints, LLM.md/INSTALL.md/README.md), and the 4 regenerated
projections.

Commit: `8452de6` — one commit, cell id in the message.

Full trace and verification evidence: `.bee/cells/packages-engine-move-1.json`.

## Deviations (in-closure, recorded in the cap evidence too)

- `new_suite_reason` declared for the 3 git-mv'd test suites (capCell's new-file detector can't
  distinguish a rename from a genuinely new suite) — they are pre-existing suites relocated by
  `git mv`, not new coverage.
- The stored cell `verify` string's acceptance rg clause lacked a `--glob '!**/CREATION-LOG.md'`
  exclusion for one pre-existing historical reference (`skills/bee-evolving/CREATION-LOG.md:98`)
  that the cell's own action item 9 and `validation-slice1.md` both declare exempt "by scope." Ran
  the clause with that glob added to honor the declared exemption; every live-surface path is
  independently clean with or without it.

No Advisor Consults on this claim.
