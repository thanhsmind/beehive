# dch-3 — Move the CI cron to 23:00, keeping the daily cadence

**Status:** `[DONE]`

## Outcome

Cron moved (`.github/workflows/ci.yml`: `'0 16 * * *'` -> `'0 23 * * *'`,
comment updated to state the new UTC/local mapping honestly). No
push/pull_request trigger added; verify chain step untouched. Known
partial fix per E9: this moves the daily detection window, it does not
narrow it — main can still carry a red for up to 24 hours and CI still
only files an issue rather than blocking a commit.

Previously blocked on a false positive: the cell's recorded verify
regex checked for `pull_request`/`push:` against the **whole file**
instead of the `on:` block, tripping on 4 pre-existing occurrences of
"pull_request" inside an unrelated job-step comment. The verify has
since been rescoped to the `on:` block only (confirmed by re-reading
the cell) — it now runs and passes clean.

## Verify

`node -e "...isolate on: block, check for pull_request/push..."` (exact
command in the cell's `verify` field):
```
ci.yml: cron 0 23 * * *, on: block carries no push/pull_request trigger
```

## Files + commit

- `.github/workflows/ci.yml`

Full trace: `.bee/cells/dch-3.json`
