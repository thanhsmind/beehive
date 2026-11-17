---
name: bee-knowledge-setup
description: >-
  Build a project's docs/knowledge/ bundle from the tree, once. Use when the repo has no docs/knowledge/ bundle and the user asks to set up, create or bootstrap the knowledge base (kho tri thức). Not for auditing an existing bundle (bee-knowledge-upkeep) or recording one fact (bee-capturing).
disable-model-invocation: true
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies:
    bee-cli:
      kind: command
      command: .bee/bin/bee
      missing_effect: degraded
      reason: The bundle is written, anchored, indexed and graded by the vendored bee binary. Absent, the concept files can be written by hand but nothing seeds their anchors or proves the bundle.
---

# Knowledge setup — build the bundle once

A repo with no `docs/knowledge/` has nowhere for what its work learns to land.
This skill stands the bundle up once, from the tree. You write it for the next
agent, who reads it cold, mid-task, and has never seen the repo.

Read `.bee/expertise/knowledge.md` before writing a concept: it holds the craft
(what belongs, what never enters, how to write the orientation file). Run
`bee knowledge consolidate` once and keep its printed writing rules beside you
while you write. This skill restates neither.

## Read the tree, never interview

Every concept comes from reading the repo: manifests, entry points, the
directory layout, the existing docs, the commands that build and test. Ask the
user only for what no file can answer, and never to dictate a concept's content.

## Pass

0. **Bundle absent.** If `docs/knowledge/` already holds a concept, stop and
   route to `bee-knowledge-upkeep`: setup never touches an existing bundle.

1. **Stand it up.** When `docs/specs/` holds specs, run `bee knowledge bootstrap`;
   it imports one area concept per spec and names every gap. Otherwise write each
   concept with `bee knowledge new --type area --title <title> --summary <summary> --area <area>`.
   Either way, one area concept per real subsystem you find in the tree: a unit
   with its own entry point and its own observable behavior, never one per
   directory.

2. **Orientation file.** Write it at the one fixed path
   `docs/knowledge/areas/project/overview.md`, as an area concept derived by
   reading the tree, in the shape `.bee/expertise/knowledge.md` ("The orientation
   file") describes. Link each area concept from it.

3. **Anchor.** Run `bee knowledge seed-anchors` so every area concept names the
   code files it describes and the first freshness grade is honest.

4. **Prove.** Run `bee knowledge check --strict` and `bee knowledge index --check`.
   Fix what they name, regenerate the indexes with `bee knowledge index`, and
   run both again until they pass. A bundle that was never checked is a draft.

5. **Offer the upkeep loop.** Point the user at `bee-knowledge-upkeep` for keeping
   the bundle honest as the code changes. Suggest a cadence only if they ask.

Setup edits only `docs/knowledge/` and never product code. Commit the bundle as
one change.
