# Feasibility gather — state-query-surface

I/O gather worker (generation tier), read-only. Full digest of the 5 feasibility questions.

## Q1 — decision event has NO structural cell/feature field
`decide` event = `{id, type, date, decision, rationale, alternatives, scope, source, confidence, tags?}`
(`lib/decisions.mjs:300-338`; `tags` present only if non-empty, `decisions.mjs:330`). Cell ids
(`si-1`) appear only in free text or code comments (`lib/cells.mjs:2114`), never as a data field.
Grep for `"si-1"` in `.bee/decisions.jsonl` → 0 hits. **Structural join is a dead end** →
`--cell/--feature` must be word-boundary text match over `decision`+`rationale`+`alternatives`.

## Q2 — `--text` is substring; collision confirmed
`filterDecisionEvents` (`bee.mjs:1666-1703`): `--tag` exact (1671-76), `--scope/--area` exact
(1677-80), `--since` inclusive (1681-87), `--text` **substring** via `haystacks.some(h=>h.includes(term))`
(1695) — so `--text 'si-1'` matches `si-10`. Word-boundary discipline exists nearby
(`sweepDecisionCitations`, `decisions.mjs:382-383`, `\b…\b`) but was not applied to `--text`.

## Q3 — backlog friction/finding schema (two generations)
Legacy: `{ts, kind:"friction"|"finding", feature, title, detail, impact, source, severity?}`
(`.bee/backlog.jsonl:38-41`). Current (`handleBacklogAdd`, `bee.mjs:2800-2850`, row 2825-33):
`{ts, type, title, detail, severity, layer, feature}` — `type` not `kind`, `layer` replaced `impact`.
PBI rows are `kind:'pbi'` (disjoint), folded by `foldPbis` (`lib/backlog.mjs:92`), skipped by
`collectFeedback` (`lib/feedback.mjs:506`). Read/fold loop: `lib/feedback.mjs:470-509`. Registry:
`command-registry.mjs:1082-1107`. **A query verb must accept both `kind` and `type`.**

## Q4 — scribing-runs ledger
Rows `{ts, feature, areas[]}` (`.bee/logs/scribing-runs.jsonl:1-5`; `run.at` fallback also read at
`cells.mjs:2108-2112`). Path `scribingLedgerPath` (`cells.mjs:2081-83`); writer `appendScribingLedger`
(`cells.mjs:2096-2106`); reader `readScribingLedger` (`cells.mjs:2085-87`), consumed by
`globalScribingDebt` (`cells.mjs:2128+`, per-feature max `bestStampMs`). `--show/--last` reuses these.

## Q5 — hook guard pattern
`bee-write-guard.mjs` already inspects Bash command content: `extractBashTargets` (784-87,
`guards.mjs:967-999`), `checkGitBashCommand` (834-37, `guards.mjs:391+`), `checkCliShape` (605-38).
Add exported `checkBinLibImportBashCommand(command)` in `guards.mjs`, wire into the Bash branch
alongside `checkGitBashCommand` (~834-848), surface via the existing ERROR/WHY/FIX convention.
Test bed: **`hooks/test_write_guard.mjs`** (the Bash `tool_input.command` allow/deny scaffold —
rows ~284-367,844,921-926; runs via `runWrapper`). NOTE: an earlier draft of this line named
`test_hook_contracts.mjs`, which is WRONG — that harness has no `tool_input.command` machinery
(reservation/editPayload rows only). Corrected after cell review. Negative control:
file-based `node x.mjs` lib import PASSES, inline `node -e import(bin/lib)` reach DENIED.
