---
type: bee.pattern
title: A fake-CLI test that resolves anything through PATH races the onboard test that empties PATH
description: "Tests that fake the skillstation CLI must inject the runner or use absolute interpreter paths, because an onboard test rewrites the process-wide PATH while the suite runs in parallel; a new verb also trips the session-preamble command-surface character budget."
tags: [tests, path, skillstation, fake-cli, registry, budget]
timestamp: 2026-11-14
bee:
  id: pattern-20261114-fake-cli-test-races-the-path-emptying-test
  lifecycle: active
  areas: [rust-runtime]
  anchors: [packages/bee-rs/crates/bee/src/verbs/skillstation_group.rs, packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs]
  sources: ["skillstation-aliases cell ssa-1 (2026-11-14)"]
  polarity: pitfall
  critical: false
---

# A fake-CLI test that resolves anything through PATH races the onboard test that empties PATH

Read when: you test a verb that shells out to `skillstation` (or any CLI) with a fake script, or you add a verb to the registry and the session-preamble budget test goes red.

`onboard::tests::an_absent_skillstation_cli_is_a_loud_skip_and_the_onboard_still_applies` sets the process-wide `PATH` to an empty directory. A fake-CLI test running in the same binary then fails at random: `Command::new("sh")`, or a `cat` inside the fake script, is no longer found. The failure looks like a skipped row, not a spawn error, so it reads as a logic bug.

Rejected: putting the fake first on `PATH` (the plan's first shape) — it needs the same global mutation and loses to the other test. Chosen: the verb's core takes the runner as a parameter; the production caller passes `run_skillstation`, the test passes a closure that runs the fake script as `/bin/sh <script>` and whose script sets its own `PATH`. Also run a freshly written script through the interpreter, not by exec: exec of a just-written file can fail with "text file busy" while other threads fork.

Separately, every registry entry counts toward the command-surface section of the session preamble. Adding a group can cross the character budget; the budget constant is the place to raise it, deliberately, and the test names the section it measured.
