---
name: bee-how
description: >-
  Answer how a part of this repo actually runs, as a path of steps a reader can open. Use when the user types /bee-how or asks how a command, hook, flow or subsystem of this repo runs or where its parts live. Not for why it is built this way (bee-why), a paced explanation to understand a topic (bee-teach), an outside library (bee-researching), or changing code.
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The trace fans its read-only workers out through the vendored bee binary's dispatch door. Absent, the trace runs inline and the account says so.
---

# How

This skill is a door, never a copy. It scopes a how question, invokes bee-researching, loads `bee-researching/references/trace-and-provenance.md`, and runs "Trace". Trace owns every step and the answer shape. Never restate, shorten or reorder a step of it here.

## Run

Read-only. Write no brief file. The account is the reply.

## Scope the question

- When the question is part how and part why, answer the how part. The why part becomes the one next action, as a ready `/bee-why <question>`.
- A question about an outside library goes to bee-researching.

## Reply

- First line: the answer in one sentence.
- Then Trace's answer shape. Keep each UNFOLLOWED step as the procedure writes it.
- Close on exactly one next action.
- At a gate, the gate wins, per `bee-hive/references/routing-and-contracts.md` ("Communication contract").

## Headless

`mode:headless`, or a one-shot run with no live human: deliver the account in one pass, with the one next action at the end.

## Handoff

Account delivered. Invoke bee-hive skill.
