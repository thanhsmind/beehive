# worker-guard read-only mode refuses rg patterns that contain > inside quotes

## Symptom

A read-only herded gather worker is denied a plain rg search when its quoted pattern contains >, for example a Rust return arrow ->. Two of five knowledge-upkeep readers stopped partial because of it.

## Evidence

Probe with BEE_HERDING_READ_ONLY=1 BEE_HERDING_JOB_ID=probe-ku piped to bee hook worker-guard: rg -n "fn scorecard" gives exit 0; rg -n "fn read_inputs.*->" gives exit 2 denied. The leader-side write-guard shows the same class: a Bash call whose quoted text holds > was read as a write to a file.

## Output

```text
bee worker-guard denied this call: `rg -n "fn read_inputs.*->" packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs` is not allowed — this herded run is read-only. You may still write your ack, report and result files in your job mailbox (.bee/mailbox/probe-ku). FIX: put a blocker in your result file, or send it with `bee herding notify`.
```

## Suspected root cause

hooks/worker_guard.rs mode_b_blocked checks command.contains('>') on the whole stripped command; strip_redirects keeps quoted text in its output, so a > inside a quoted argument is read as a redirect.

## Command

```sh
rg -n "fn read_inputs.*->" packages/bee-rs/crates/bee/src/verbs/report_outcomes.rs
```

## Exit code

not given

## Environment

- bee version: 2.122.0
- OS: linux x86_64
