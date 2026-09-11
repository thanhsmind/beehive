---
name: bee-why
description: >-
  Answer why a part of this repo is the way it is, from the recorded evidence and never from memory. Use when the user types /bee-why or asks for the recorded reason, with evidence, behind a rule, value, guard or design of this repo, or what a change must keep. Not for how the code runs (bee-how), a paced explanation to understand a topic (bee-teach), locking or superseding a decision (bee-shaping), or an outside library (bee-researching).
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The decision log and the knowledge bundle are searched through the vendored bee binary. Absent, those rows are reported UNSWEPT with the reason.
---

# Why

This skill is a door, never a copy. It scopes a why question, invokes bee-researching, loads `bee-researching/references/trace-and-provenance.md`, and runs "Provenance sweep". The sweep owns every step, the confidence tiers and the answer shape. Never restate, shorten or reorder a step of it here.

## Run

Read-only. Write no brief file. The account is the reply.

## Scope the question

- Pin the target before the sweep: the `path:line` range, the symbols, and the search terms.
- When the target is vague, write your reading of it as the first line of the reply, then sweep that reading. Never stop to ask.
- Sweep every row. The user asked for the reasons, so a narrow sweep is not the default here.
- A guess inside the question is one hypothesis to check, per the sweep's phrasing rules.
- When the user means to change the code, end with the sweep's constraint set.
- When the question is part why and part how, answer the why part. The how part becomes the one next action, as a ready `/bee-how <question>`.
- A question about an outside library goes to bee-researching.

## Reply

- First line: the answer in one sentence, with its confidence tier.
- Then the sweep's answer shape. Keep the tier words, the empty rows and the UNSWEPT rows as the procedure writes them.
- Close on exactly one next action.
- At a gate, the gate wins, per `bee-hive/references/routing-and-contracts.md` ("Communication contract").

## Headless

`mode:headless`, or a one-shot run with no live human: deliver the account in one pass, with the one next action at the end.

## Handoff

Account delivered. Invoke bee-hive skill.
