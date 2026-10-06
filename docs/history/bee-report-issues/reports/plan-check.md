# Plan check — bee-report-issues (hat wave, five seats)

```text
PLAN CHECK
Work: slice 1 — bri-1, bri-2, bri-3 (whole feature)

STRUCTURE
BLOCKERS:
- key links / bri-1 routed the verb through router.rs / router.rs:287 hands off to verbs::try_native; the chain lives in verbs/mod.rs (feedback::try_native at :104) / FIXED: files and key_link now name verbs/mod.rs; router.rs dropped.
- completeness / bri-1 verify filter `report` skips the flag-count ratchet / catalog.rs:819 PINNED_FLAG_COUNT 216 / FIXED: verify adds distinct_flag_vocabulary and registry; action states the bump.
- completeness / bri-3 verify filter `parity` matches no test fn / tests named in agents_block_render_parity.rs:131, rule_index_parity.rs:133,162 / FIXED: verify uses --test agents_block_render_parity --test rule_index_parity.
WARNINGS:
- key links / bri-3 missed the rule-index row and rendered AGENTS.md / FIXED: both added, rule id agents-bee-defect-report.
- scope / AGENTS.windows.md is a PowerShell tail, wrong target / FIXED: dropped.
- coverage / bee-evolving step 1 says rank is the only surface; reading an issue body is a second one / FIXED: bri-3 amends that sentence with the one exception.
- facts / claim 10 (config set accepts any key) was unproven / RAN: `bee config set` is not built into this binary; its remedy says edit .bee/config.json. Claim replaced; bri-3 sets the key in beehive's tracked config.

CELLS  (reviewed: 3)
CRITICAL FLAGS: none open after the fixes below.
MINOR FLAGS:
- bri-2 / `ref` cannot ride build_entry (fixed field set, feedback.rs:1335-1343) / suggestion applied: insert after build_entry.
- bri-1 / has_injection on evidence can refuse a pasted transcript / accepted: refusal prints the body for an edit and re-run.
CLEAN CELLS: bri-3

SUMMARY: The shape holds (hat-alternatives: SMALLER PATH pass, no cheaper shape honors D1-D3). Risks and value findings tighten bri-1 and bri-2 without changing a locked decision.
```

## Risk, value and user-impact findings — disposition

| Seat | Finding | Disposition |
|---|---|---|
| risks | Scrub misses emails, IPs, internal URLs, `--token x`, `~/`, home substrings | Applied: bri-1 refuses on these; `~/` rewritten; home substring is a final backstop refusal |
| risks | Verb walks around the worker outward guard | Applied: bri-1 calls the same outward guard check and refusal |
| risks | Hostile titles that pass has_injection | Applied: bri-2 keeps only issues whose author login is in `bee_report.trusted_authors` (default: repo owner); others go to dropped[]. gh has no authorAssociation field (ran) |
| risks | Labels need collaborator rights; label-create retry | Applied: no retry; on label failure create unlabeled with `[bee-report]` title prefix and say a maintainer must label it; leader creates the label once at rollout |
| risks | Host config can redirect the target repo | Applied: `owner/name` shape check; target repo printed on every result |
| risks | Dedupe search breaks on quotes/qualifiers | Applied: query sanitized; search failure falls to create |
| risks | gh stderr in output | Applied: never in --json or the body |
| value | Fence stripping removes bee's own output | Applied: separate `--output` field, scanned and path-scrubbed, fences kept |
| value | Fields too thin to fix cold | Applied: `--command` required, `--exit-code` added; version and OS auto |
| value | Dedupe makes every issue frequency 1 | Applied: ingest reads comment count; entry carries `count` = 1 + comments; cluster frequency sums `count` (default 1) |
| value | Version not used at Gate A | Out of scope, named |
| value | Ingest silently off when dogfood_repos set | Applied: skipped_reason `dogfood_repos configured` — not reachable while the arm delegates; recorded as out of scope instead |
| user-impact | Silent public post | Applied: plain output prints the URL and scrub counts; doctrine says tell the human in one line after filing |
| user-impact | Agent reports its own mistakes as bee bugs | Applied: doctrine names what is and is not a bee defect |
| user-impact | Near-duplicate titles | Applied: dedupe compares normalize_title of open issue titles |
| user-impact | No gh auth / offline | Applied: scrubbed body saved under `.bee/reports/`; doctrine says tell the human once, no retry loop |
| user-impact | Windows CI and the fake gh script | Applied: fake-gh tests are `#[cfg(unix)]` |
| user-impact | Reporter never learns the fix shipped | Applied: bee-evolving step names a closing comment with the release |
| alternatives | Use `bee mailbox reflect --upstream` | Rejected: two meanings on one verb, wrong record shape, not cheaper |
