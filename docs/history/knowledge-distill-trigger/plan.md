# knowledge-distill-trigger — plan (v2)

**Lane:** standard · **Route flags:** data-model (decisions.jsonl field),
covered-contract-change (close doors, decisions log surface) · **Files:** ~12 product
**v1 rejected by plan-check** — C1 self-deadlock (promote-generated
`required_context` can never resolve inside the bundle), C1/C2 file overlap,
C3 caller breakage undercounted. v2 fixes the generator, serializes C3
after C2, and names every caller surface.

## Goal

The knowledge/decision layer stops accreting silently: stale pointers in
touched areas block close (D1), deferred decisions live in a registry that
surfaces until resolved (D2), every new decision declares its relation to
prior ones at write time (D3), and the new mechanisms then clean the
existing debt (D4).

## Shape — slice 1: 3 cells; C1 ∥ C2, then C3 (C3 reads C2's store)

### C1 — knowledge-freshness close door + the resolution bug it exposes (D1)

Files: `verbs/drivers/close.rs`, `verbs/knowledge/check.rs`,
`verbs/knowledge/promote.rs`, `verbs/drivers/tests.rs`, plus the 6
`areas/` pointer repairs (under `packages/bee-rs/crates/bee/src/` and
`docs/knowledge/areas/`).

- **Fix the generator/check mismatch first** (plan-check CRITICAL):
  `promote.rs:549-554` writes `required_context` =
  `docs/history/<f>/CONTEXT.md`,`plan.md`, but `check.rs:542-563` resolves
  `required_context` only inside the bundle — so every promoted delivery
  file is born dangling (13/13 current warnings). Fix: `check.rs`
  resolves `required_context` entries against the bundle FIRST, then the
  repo root — a docs/history path that exists on disk is not dangling.
  This erases the 13 `dangling_required_context` warnings by correctness,
  not repair.
- **Scoped door**: filter `CheckReport.warnings` post-walk to file
  prefixes `areas/<touched-area>/` + `work/<feature>/` (touched areas via
  `touched_bundle_areas`, `close.rs:895-918`). `dangling_source` +
  `dangling_required_context` in scope → `blocking: true`, detail naming
  each file + remedy; `not_canonical`/`invalid_evidence_state` stay
  report-only detail (named limitation: prose contradictions — the "dark
  guards" class — have no machine detector; they are S2-c distill work,
  recorded here, not silently dropped).
- **Arm clean**: this cell also repairs the 6 existing `areas/` dangling
  sources (decision-memory/backlog-store.md, doctrine-layer, hook-runtime
  ×2, workflow-state ×2 — retired paths → Rust equivalents or
  removal-with-reason), so the door never taxes an unrelated in-flight
  feature at arming time. `patterns/`+`work/okf-foundation/` pointers are
  outside door scope and wait for S2-a.
- **Escape valve** (D1): recorded deferral decision, shape of
  `has_capture_deferral_decision` (`close.rs:663-676`).
- Door slots after `pattern-check` in the exit chain (`close.rs:1266`
  precedent) and into all three doors vecs (`:1061`, `:1111`, `:1188`).
- Tests: dangling source in touched area → refuse; untouched area →
  clear; deferral → non-blocking named; no bundle → absent; promoted
  delivery with docs/history required_context → resolves clean.

### C2 — deferred-decision trigger registry (D2) — orient-only surfacing

Files: new `verbs/triggers/` module + dispatch wiring
(`registry_payload.json` + `registry_contracts.rs` expectations),
`verbs/status_full/orient.rs`, tests. No close.rs (disjoint from C1).

- Store at the CONTROL root (shared across worktrees, like
  `no_route_claim_counts_dir(control)`, `handlers_write.rs:815`):
  `.bee/triggers/<slug>__<short8>.json`, one file per trigger, atomic
  write, fail-open read. Record: `id`, `decision` (short8), `condition`
  (prose), `tier` (`predicate`|`manual`), optional `predicate`
  (`path-exists:<p>`|`path-missing:<p>`), `status`
  (`waiting`|`due`|`resolved`), dates, `outcome` (on resolve).
- Verbs: `triggers add --decision <id> --condition <text>
  [--predicate ...]`, `triggers list [--due]`, `triggers resolve
  --id <id> --outcome <text>`. Resolve writes the outcome into the
  trigger record ONLY — it never logs a decision itself (no C3
  dependency); the capture discipline owns any follow-up decision.
- Evaluation on read: a true predicate flips `waiting → due` and persists
  (precedent: orient already writes via `sweep_on_orient`,
  `orient.rs:260-289`). `manual` never auto-fires — surfaces as
  awaiting-confirmation.
- Surfacing: ONE orient blocker line ("N trigger(s) due, M awaiting
  confirmation"), helper in the blocker sequence (`orient.rs:322-388`,
  template `capture_queue_blocker_line` `:174-189`). A due trigger routes
  through `bee backlog add` by the human/agent reading the line — it
  never self-executes. Close-door surfacing deferred (D-idea, backlog).
- Corrupt file: fail-open to a visible "unreadable trigger <file> —
  remedy: delete the file" line; never a crash.
- Tests: round-trip; predicate flip persists; corrupt-file line; orient
  line only when due/pending exists; store resolves to control root from
  a worktree.

### C3 — required relation on decisions log (D3 + D2's write-path law)

After C2 lands (needs trigger-id resolution). Files:
`verbs/decisions/verbs_read.rs`, validation fn (scanners.rs or sibling),
`verbs/state_group/set_gate.rs` (3 internal `LogParams` callers:
`:213,:243,:270`), `hooks/session_close/nudges.rs` (`:330,:411,:421`
nudge text), `generated/registry_payload.json` (decisions.log `required`
+ example) with `registry_contracts.rs` green, decisions tests; plus
instruction surfaces: `packages/bee/AGENTS.block.md` (+ regen),
`skills/bee-hive/references/scout-and-ticks.md:79`,
`skills/bee-hive/references/gates-and-delegation.md:75` (gate-bypass
audit line — a silent refusal here kills the audit trail),
`skills/bee-capturing/references/promotion.md:156`,
`docs/07-contracts.md:144`, `docs/03-workflow.md:156`.

- Required flag on `decisions log`: `--relation
  supersedes:<id>[,...] | touches:<id>[,...] | none`. `supersedes:`
  reuses `resolve_supersedes_target` (`read.rs:275-308`) and the existing
  `supersedes` field; `touches:` persists a new resolved-id array;
  `none` persists `relation:"none"`.
- Refusal without the flag quotes up to 3 dcc-1 candidates
  (`conflict_candidates`, `read.rs:454-503`) and teaches the flag in one
  line. Internal callers (set_gate waivers) pass explicit `none`.
- **D2's law**: deferral-shaped decision text ("defer", "for now",
  "revisit when/if", "later than this feature") without a `--trigger
  <id>` naming a registered C2 trigger refuses with a create-the-trigger
  teach line — mirror of the existing dsh-1 prose guard
  (`verbs_read.rs:275-317`). `--trigger` persists the trigger id on the
  decision. This enforces "no deferred condition outside the registry"
  at the only door conditions enter through.
- Grandfathering: readers tolerate absent fields on old records; only new
  `decide` events owe the flag; `supersede`/`tag`/`redact` exempt. dsh-1
  still wins over `--relation none`.
- Tests: refusal lists candidates; each relation form persists;
  unresolvable touches id refuses; deferral prose without --trigger
  refuses, with --trigger passes; legacy-line reads unaffected; set_gate
  waivers still log; registry contract walk green.

## Slice 2 — backfill (D4), headlines only, cells cut after slice 1

- S2-a: repair remaining dangling sources — 23 `patterns/` + 9
  `work/okf-foundation/` (retired paths → Rust equivalents or
  removal-with-reason). Proof: full `bee knowledge check` reports zero
  dangling warnings (direct run — the C1 door alone cannot prove
  `patterns/`, which sits outside its scope).
- S2-b: formal `decisions supersede` for confirmed unmarked reversals
  (cli-performance → rust-port, P76 shape-supersede, + any found);
  register the 5 orphan deferred conditions as C2 triggers.
- S2-c: distill the changelog-prose files —
  `workflow-state/gates.md` "Closing a feature",
  `workflow-state/worktree-isolation.md`, `rust-runtime/overview.md` —
  present tense, contradicted lines replaced, lifecycle honest.

## Verify

`commands.test`: `PATH="$HOME/.cargo/bin:$PATH" cargo test --release
--manifest-path packages/bee-rs/Cargo.toml` at every `bee cells finish`.
C3 additionally leaves the regen chain clean (`bee dev regen`,
`bee dev release-manifest --check`) and `registry_contracts.rs` green.

## Cost if wrong

- C1 too strict → deferral valve + area scoping; rollback = demote door
  to report-only (one bool). The generator fix stands on its own merit.
- C2 additive; rollback = ignore the store.
- C3 changes a write contract every agent uses — refusal must teach the
  fix in one line; every scripted call site above is updated in the same
  cell (a migration is not done until its instructions are); rollback =
  optional flag.
