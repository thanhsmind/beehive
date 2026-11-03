# Herding Pre-flight Test Seam — Context

**Feature slug:** herding-preflight-test-seam
**Date:** 2026-09-20
**Shaping session:** complete (bugfix brief, small lane)
**Scope:** Quick
**Domain types:** RUN

## What was asked

The user reported it in their own words: a pane opens, does nothing, and does
not close either.

## What was found

Three unit tests added by cell `ihr-2` the same day call `herding::run::run()`
directly. `run()` builds the **real** transport through `transport_for_run`, so
on any machine with `herdr` on PATH each `cargo test` really splits a pane and
really starts `herding.agent_command` — default `claude-sonnet` — handing it:

> Read the file `/tmp/.tmpBs3KFO/.bee/mailbox/job-crashed-1/brief-1.txt` and
> follow its instructions exactly.

The test's `tempfile::tempdir()` is dropped when the test ends, so the brief is
already gone. The agent reads nothing, sits idle, and nothing ever closes the
pane.

The asserts pass throughout — they only check `ExitCode::FAILURE`, and the run
does eventually fail, *after* the pane exists. So the suite stays green while
leaking a billed session every run.

**Evidence (2026-09-20):** 12 leaked sessions, 3.4 MB of transcripts, under
`~/.claude/projects/*crates-bee/` — 5 from the main checkout, 7 from the
`idempotent-herding-receipts` worktree. Every transcript's first user message
names a `/tmp/…` job brief. One leaked pane (`w1:pCJ`) sat idle for hours
until it was closed by hand.

This also broke a contract the file already states at `run.rs:480`: *"tests
inject a fake instead of a real `herdr` on PATH (D7's seam, no process anywhere
in this crate's test suite)"*. Decision D7 placed the pre-flight inside
`fn run` — correctly, for visibility of refusals — and `fn run` is the one
function with no fake seam. Nobody noticed the collision.

The three tests: `run.rs:9670`, `:9692`, `:9714`.

## What will be done

Split **deciding** from **acting**. A new `preflight_receipt(bee_dir, opts) ->
PreflightAction` resolves the round and returns `ReturnReceipt | Refuse |
Proceed` from the filesystem alone. `fn run` becomes the only place that acts
on that action, and it keeps D7's placement exactly — above the transport
choice and above the detached re-launch.

The three tests then assert the returned action and never call `fn run`, so no
test in this crate can reach `transport_for_run`.

Store decision: `contract:herding-preflight-action` (logged 2026-09-20).

## Boundary

One file, `packages/bee-rs/crates/bee/src/herding/run.rs`. No behavior change
for a real user: the same five outcomes, the same refusal text, the same
envelope, the same marker write. Only the seam moves.

## Not in scope

- Any change to the five pre-flight outcomes, the refusal wording, or D5's
  marker write.
- The `.bee/logs` test pollution and the `bee herding prune` question, both
  already separate backlog rows.
- Sweeping the 12 already-leaked session transcripts — they are the user's
  data, not bee's to delete.
