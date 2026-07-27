# arp-1 evidence — GH #86, advisor-ref record never persisted

## Why it was invisible in this repo

This checkout has **zero live workflow records**, so every mutation took the
`!wf` fallback branch, which writes the caller's full object directly. That is
why `advisor_ref` persisted here twice today while the reporter's repo lost it
every time. The bug needs a live workflow record — the normal state of any
actively-worked msn-10 feature.

## Reproduced, on a fixture with one live workflow record

```
$ bee state start-feature --feature repro86 --mode high-risk
workflow records: 1

$ bee state advisor-ref record --advisor fable --digest-file digest.txt
Recorded advisor_ref (advisor "fable", feature "repro86").

$ bee state advisor-ref show
No advisor_ref recorded.

$ python3 -c "print(list(json.load(open('.bee/state.json')).keys()))"
['schema_version','phase','feature','mode','approved_gates','workers','summary','next_action']

$ bee state gate --name execution --approved true
gate: execution approval refused for high-risk work — the advisor consult is
missing or stale (AO3/AO13). Reason(s): no advisor_ref recorded.
```

Success reported, nothing written, Gate 3 shut forever.

## After the fix, same fixture shape

```
$ bee state advisor-ref record --advisor fable --digest-file digest.txt
Recorded advisor_ref (advisor "fable", feature "repro86").

$ bee state advisor-ref show
advisor="fable" feature="repro86" consulted_at=… stale=false

advisor_ref present: True

$ bee state gate --name execution --approved true
Gate "execution" set to true.
```

Lane path, separately:

```
$ bee state advisor-ref record --advisor fable --digest-file digest.txt --lane lane86
$ bee state advisor-ref show --lane lane86
advisor="fable" feature="lane86" consulted_at=… stale=false
lane advisor_ref present: True
```

## The fix

`writeStateRecordThroughProjection` and `writeLaneRecordThroughProjection`
patched the workflow record with a five-field allowlist and then rebuilt the
projection by **re-reading it from disk**, so any ad hoc field the caller set on
`updated` was discarded before reaching a file. Both now land the caller's full
record (`writeState` / `writeLane`) **before** the rebuild. The D1 fields that
write lays down are re-derived from the workflow record moments later, so the
record stays authoritative for everything it owns; only the fields it does not
own survive from the caller.

## One test row updated, and why that is not a weakening

`sss-1` asserted `afterScribe.last_scribing_run === undefined`. Its own comment
says what it was: *"documents the seam this cell works around: the record stamp
does not survive the workflow-projection rebuild"* — a characterization of a
known bug, not a contract. Fixing the seam makes that line false. It now asserts
the stamp **does** reach disk, and the row's real assertions are untouched: the
close still succeeds with no waiver, and the negative control still blocks a
cell capped after the last scribing-run. `sss-1`'s durable-ledger fallback
becomes corroboration rather than the only proof.

The other four red rows were cascade — `test_msn_invariants` reuses
`test_cli_state.mjs` and fails loud when that suite is red as a whole.

## Suite

```
PASS run_verify: 108 suite(s), concurrency=5, wall=86603ms
EXIT=0
```

No new test authored (behavior cells cap on `existing-targeted-green` since
`fs-1`); the live reproduction above is the evidence.
