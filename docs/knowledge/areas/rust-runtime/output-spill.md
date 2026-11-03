---
type: bee.area
title: "Compiled runtime: output spill — large read-verb text goes to a file"
description: "How an opted-in repository keeps a large text output of a read verb out of the caller's context: which verbs and runs qualify, the threshold key and its override, the head, marker and tail the caller sees, where the full output is kept, and the Windows gap."
tags: [rust-runtime, cli, output, spill]
timestamp: 2026-11-03
bee:
  id: rust-runtime-output-spill
  lifecycle: active
  areas: [rust-runtime]
  required_context: [areas/rust-runtime/command-surface.md]
  decisions: ["cli-output-spill D1 (allowlist of read verbs, text stdout only, never --json, never a terminal)", "cli-output-spill D2 (opt-in key output_spill_bytes, BEE_OUTPUT_FULL override, head two thirds, marker, tail one quarter)", "cli-output-spill D3 (Unix file-descriptor redirect, Windows pass-through)"]
  sources: [docs/history/cli-output-spill/CONTEXT.md, packages/bee-rs/crates/bee/src/output_spill.rs, packages/bee-rs/crates/bee/src/main.rs, packages/bee-rs/crates/bee/tests/output_spill.rs]
  authoritative_for: "rust-runtime: when a bee command's stdout is spilled to a file and what the caller sees instead"
---

# Compiled runtime — output spill

An agent reads every byte a command prints. Some read verbs print more than an
agent needs at once: `bee status` prints about 12 KB and `bee --help --all`
about 157 KB. Output spill keeps the start and the end of such an output in
view and puts the full output in a file the agent can open when it needs it.

## When a run spills

All of these must be true (cli-output-spill D1, D2):

- The repository's `.bee/config.json` sets `output_spill_bytes` to a positive
  integer. An absent key or `0` turns spill off. This repository sets `8000`.
- The argv starts with `status`, `cells list`, `decisions list`,
  `backlog list`, `knowledge context` or `knowledge search`, or it carries
  `--help`. The list has one home: `SPILL_VERBS` in
  `packages/bee-rs/crates/bee/src/output_spill.rs`.
- The argv carries no `--json`. JSON output is a machine contract, so it never
  spills.
- stdout is not a terminal. A person at a terminal can scroll.
- `BEE_OUTPUT_FULL` is not `1`. Set `BEE_OUTPUT_FULL=1` to get the full output
  for one run.

## What the caller sees

When the output is at or under the threshold, the caller gets the exact bytes,
and no file is kept. When it is over the threshold, the caller gets:

1. The leading whole lines that fit in two thirds of the threshold.
2. One marker line:
   `[bee] output spilled: <omitted> of <total> bytes omitted — full output: .bee/tmp/output/<verb>-<utc-stamp>.txt`
3. The trailing whole lines that fit in one quarter of the threshold.

The file keeps every byte. The path is relative to the work root. The exit code
and stderr do not change, and the `[bee] <verb> <ms>ms` footer still goes to
stderr.

## How it works

`main()` hands the router call to `output_spill::run`. On Unix, when a run
qualifies, bee points file descriptor 1 at the spill file, runs the verb, and
then restores the descriptor (cli-output-spill D3). This one wrapper covers all
of the print sites without changing them. Any failure to create the file or to
redirect the descriptor runs the verb without the wrapper.

## Gaps

- Windows: output passes through unchanged (cli-output-spill D3).
- The installed `.bee/bin/bee` gets this behavior only after a release that
  includes it.
- `.bee/tmp/output/` is not cleaned. Each spilled run adds one file.

## Pointers

- `packages/bee-rs/crates/bee/src/output_spill.rs`
- `packages/bee-rs/crates/bee/src/main.rs`
- `packages/bee-rs/crates/bee/tests/output_spill.rs`
