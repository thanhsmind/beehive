# Scheduled curator — test report (scur-4)

Verdict: pass. No production defect found.

## Cell verify

`cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- upkeep session_preamble` — 113 passed; 0 failed; 1 ignored (includes the new promote test and the session_preamble budget tests at COMMAND_SURFACE_BUDGET_CHARS 3000).

## Sandbox run

Binary: `packages/bee-rs/target/release/bee` (carries `deferred-queue complete`; the shared `/home/thanhsmind/.cache/cargo-target/release/bee` is stale and does not). Sandbox: `.bee/tmp/scur4-sandbox`. `bee onboard` refuses outside a bee checkout, so `.bee/onboarding.json` was written the way the upkeep_group test helper does.

```
$ bee config set --key curator.enabled --value true
config set: curator.enabled = true
[bee] config set 3ms
exit 0
$ bee capture add --outcome sandbox stub --lane standard
Queued capture stub c436b187-ab8c-4b8a-bf85-7ce8df496ff4. Flush via bee-capturing at wrap-up, before compact/clear, or next session (decision 0017).
[bee] capture add 1ms
exit 0
$ bee upkeep due --kind curator --json
{
  "schema": "bee-upkeep-due/v1",
  "any_due": true,
  "items": [
    {
      "kind": "curator",
      "skill": "bee-curating",
      "invoke": "/bee-curating",
      "present": true,
      "due": true,
      "reasons": [
        {
          "code": "capture_queue",
          "detail": "1 capture stub(s) wait in the queue (threshold 1)"
        }
      ],
      "info": [],
      "cadence_hours": 24,
      "last_run": null
    }
  ]
}
exit 10
$ bee audit log --kind curator-run --feature curator --note clean — test
Logged audit entry e9ff8594-4cc0-47ab-a33c-b679abe27636 (curator-run, curator).
[bee] audit log 2ms
exit 0
$ bee upkeep due --kind curator --json
{
  "schema": "bee-upkeep-due/v1",
  "any_due": false,
  "items": [
    {
      "kind": "curator",
      "skill": "bee-curating",
      "invoke": "/bee-curating",
      "present": true,
      "due": false,
      "reasons": [],
      "info": [
        {
          "code": "capture_queue",
          "detail": "1 capture stub(s) wait in the queue (threshold 1)"
        },
        {
          "code": "cadence_not_elapsed",
          "detail": "last curator run is inside the 24h cadence; next eligible at 2026-11-23T00:46:09+00:00"
        }
      ],
      "cadence_hours": 24,
      "last_run": {
        "at": "2026-11-22T00:46:09.243Z",
        "outcome": "clean",
        "note": "clean — test"
      }
    }
  ]
}
exit 0
$ bee deferred-queue complete --help --brief
bee deferred-queue complete
    Settle queue work as done.
  --id (str) — Queue item id to complete. Use this or --kind with --feature.
  --kind (str) — With --feature: complete the item of this kind: capture, scribe, review, promot…
  --feature (str) — With --kind: complete the item of this feature.
  --owner (str) — Owner session id. Omitted: resolved from BEE_SESSION_ID, CLAUDE_CODE_SESSION_ID…
[bee] deferred-queue complete 2ms
exit 0
```

Truths: due true / capture_queue / exit 10; after the curator-run row due false, info cadence_not_elapsed, exit 0; deferred-queue complete prints its flags.

## Skill against D3

`skills/bee-curating/SKILL.md` has `disable-model-invocation: true`, opens with `bee upkeep due --kind curator --json` (line 49), caps promote at 10 and scribing at 5, uses `bee deferred-queue complete --kind promote --feature` and `bee state scribing-run`, and ends step 6 with `bee audit log --kind curator-run --feature curator --note "<clean|changed|blocked> — <one line>"`. Matches D3.

## Test gap against D2

scur-1 tests covered capture_queue, cadence, kinds and thresholds, config fallback, cadence_hours and the audit kind. Missing: the `promote_unsettled` reason. Added `an_unapplied_promote_proposal_is_curator_debt_and_a_kinds_filter_hides_it` to the upkeep_group tests module. `scribing_orphaned` stays untested: it needs a capped behavior-change cell fixture.

## Skipped

Clippy and the full suite (leader note: do not run the full suite).
