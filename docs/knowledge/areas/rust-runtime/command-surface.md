---
type: bee.area
title: "Compiled runtime: the command surface — flow verbs, aliases, and the plumbing namespace"
description: "Why the CLI shows a small default surface instead of its whole registry, which verbs earn a place in it, how a flow verb is an alias rather than a second implementation, what every porcelain verb owes its caller at the point of contact, and why drift detection still hashes the full registry the split hides."
tags: [rust-runtime, cli, porcelain, plumbing, help]
timestamp: 2026-08-03
bee:
  id: rust-runtime-command-surface
  lifecycle: active
  areas: [rust-runtime]
  required_context: [areas/rust-runtime/overview.md]
  decisions: [harness-refocus-P2, 412e9b3a, harness-refocus-P4]
  sources: [docs/specs/porcelain.md, docs/handbook/register.md, packages/bee-rs/crates/bee/src/router.rs, packages/bee-rs/crates/bee/src/generated/registry_payload.json]
  authoritative_for: "rust-runtime: the porcelain/plumbing command surface and the teach-at-point-of-contact contract"
---

# Compiled runtime — the command surface

The CLI carries far more commands than any one session needs. Showing all of
them by default made the help output too large to hold in a prompt, which in
practice meant agents did not read it and skills grew prose copies of the flow
instead — the exact inversion this surface exists to undo.

## Purpose

Keep the DEFAULT surface small enough that an agent can read it, without
removing or renaming anything. The split is a **presentation contract**: every
command that ever worked still works and is still invocable by its original
name. What changes is only which ones a bare `bee --help` puts in front of you.

## Entry Points & Triggers

- `bee --help` (text or `--json`) — the porcelain surface only.
- `bee --help --all` (text or `--json`) — the full registry, every entry
  carrying its own `surface` value.
- `bee <group> --help` — unchanged: every match in the group, regardless of
  surface. A scoped question deserves a scoped answer, not a filtered one.
- `bee <verb> --help` — when exactly ONE entry renders, the compact flags
  line becomes a detail block: one line per flag — `--name`, `*` when
  required, the type in parens, then the registry parameter description
  (bee-help-verb-detail D1). A flag with no description falls back to its
  compact form; `json` stays filtered under the header note. Multi-entry
  renders keep the compact line — detail only where asked, so token cost
  stays bounded to the verb in question (D3). The read-once rule rides the
  session briefing's command-surface sentence (D2): before a verb's FIRST use
  in a session, read its help — never guess flags; one read per verb, never
  re-read. The always-loaded instruction layer does NOT carry the rule today:
  D2's other half was written into the rendered instruction file instead of
  the managed source it is generated from, and the next regeneration undid
  it — an Open Gap, tracked in the backlog.
- `bee internal <group> <verb>` — the explicit plumbing namespace.

## Data Dictionary

| Term | Meaning |
|---|---|
| `porcelain` | A registry entry's `surface` value marking it part of the default flow surface — what `bee --help` shows |
| `plumbing` | Everything else. Still shipped, still invocable, listed under `--all`. **An absent `surface` field reads as plumbing** — the safe default, since a new command is unproven until someone decides it belongs in the flow |
| flow verb | A porcelain spelling that is an ALIAS: argv is rewritten and the proven verb runs. One implementation, one test set, two names |
| teach-at-point-of-contact | The contract every porcelain verb owes its caller (Behaviors below) |

## Behaviors & Operations

**Every porcelain verb ends by naming the next action.** Its text output — and
its refusals — close with what to run next, or what decision the caller now
owes, in plain language. Blocked-when: never; a verb that has nothing to say
next says so rather than trailing off. What the caller observes: an agent can
complete a tiny task from `bee orient` plus the outputs of the verbs it is led
through, with **no skill preloaded**. That is the acceptance test for this
contract, and it is why the surface exists at all: the flow lives in the
outputs, so prose does not have to sequence the machine.

**A flow verb is an alias, never a second implementation.** `bee route`,
`bee shape`, `bee gate` and `bee finish` rewrite argv and dispatch into
`state route`, `intent set`, `state gate` and `cells finish` respectively. What
changes: nothing but the spelling — flags pass through untouched. Why it
matters: two implementations of one operation is how two callers come to
disagree about what the operation does, and the alias table exists precisely so
that can never happen here.

**The flow spelling is the one to write down.** Where a flow verb exists, docs
and skills use it (`bee gate`, not `bee state gate`). Both work forever; the
convention is about what a reader learns from the page. This is checked by the
maintainer checklist in `docs/handbook/writing-skills.md`, not by a guard —
prose-ruled, and worth a friction entry when it drifts.

**The plumbing namespace is real in both directions.** `bee internal <group>
<verb>` dispatches identically once the prefix is stripped, and it **refuses a
flow verb** — "call it as `bee gate`, without `internal`". A boundary that only
guards one direction teaches the wrong shape.

**Drift detection still hashes the FULL registry.** The manifest check never
narrows to the porcelain subset. Stated as a rule because the failure it
prevents is silent: a presentation split that also narrowed what drift-checking
compares would let a plumbing change land unnoticed, and the split would have
bought a smaller help output at the price of a blind guard.

## Actors & Access

| Actor | May |
|---|---|
| An agent session | Call every verb on both surfaces; the split is presentation, never permission |
| A dispatched worker | Same — it reads the porcelain surface because that is what its prompt points at, not because plumbing is closed to it |
| The registry | Declare each command's `surface`; a missing value reads as plumbing |

## Business Rules

- R1 — Nothing is renamed or removed by the split. Every existing command name
  keeps working.
- R2 — A registry entry with no `surface` field is plumbing.
- R3 — Group-scoped help ignores the surface split entirely.
- R4 — Manifest drift detection hashes the full registry, never the porcelain
  subset.
- R5 — A flow verb rewrites argv into an existing verb. Adding one never adds
  an implementation or a test set.
- R6 — `bee internal` refuses a flow verb by name, pointing at the flow
  spelling.
- R7 — A new verb is not shipped until the registry names it: it enters
  the registry record (so the full help surface lists it) and the pinned
  flag-count assertion moves with it in the same change (wayfinding-flow
  wayf-6, 2026-08-17: discovery list/stub registered, count 156→158).

## Edge Cases Settled

- **A failed `bee finish` is always completable stepwise.** `cells cap` and
  `reservations release` stay available as plumbing, so a verb that bundles
  several steps never becomes the only road through them.
- **`bee orient` never computes state a second way.** It reuses the status
  builder. A second derivation of "where am I" is a second answer waiting to
  disagree with the first.
- **Per-command text help renders the FULL flag surface.** `bee <command>
  --help` prints a `--flag*:type` line for every flag the registry declares —
  optional ones included — never only the required set; the always-accepted
  json/help pair is stated once in the header instead (harness-audit-hardening
  hah-4, verbs/help.rs, 2026-08-07).
- **A hook entry point without a hook name is usage, never a panic.** Bare
  `bee hook` and `bee hook --help` print the usage line and exit cleanly; the
  hook dispatcher never panics on a missing subcommand (harness-audit-hardening
  hah-5, hooks/mod.rs, 2026-08-07).

## Open Gaps

- The porcelain set is a judgement about what a session needs, re-made when the
  flow changes. There is no mechanical test that a given verb *belongs* in it —
  only that whatever is declared is presented consistently.

## Pointers

- The alias table and the per-register detail: `docs/handbook/register.md`.
- The dispatcher and its alias rewriting: `packages/bee-rs/crates/bee/src/router.rs`.
- Per-command contracts: `bee --help --all --json`.

## Sources and provenance

- `docs/specs/porcelain.md` — the migrated source; it survives as a pointer stub
  and is superseded by this concept (2026-08-03).
- `docs/handbook/register.md` — the flow-verb alias table, human-facing.
- `packages/bee-rs/crates/bee/src/router.rs` — where argv rewriting actually happens.
- `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — the `surface`
  value per command.
- Decisions: harness-refocus P2 (the porcelain surface itself — flow belongs in
  CLI verbs whose outputs name the next action, not in prose that sequences the
  machine); `412e9b3a` with test-simple (the proof-tier parameters `bee finish`
  inherited from `cells cap` are deleted); harness-refocus P4 (`bee orient` is
  the router's own entry point, and the hive skill is a thin router around it).
