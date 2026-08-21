# Worker Brief Expertise Section — Plan

**Feature:** worker-brief-expertise · **Lane:** standard · **Date:** 2026-08-20
**Decisions:** CONTEXT.md D1–D4 (cited below, never restated as new choices)
**Review:** one-agent review wave ran 2026-08-20; both P1s and all P2s folded in below.

## Shape

Two dispatch paths render a worker brief today; both gain the same optional
Expertise section, fed by the dispatcher, rendered only when entries exist (D1).

### Entry format (one spelling, both paths)

One `--expertise` flag, given once; its value holds one entry per line
(`dispatch prepare`'s shared `Flags` parser is last-value-wins — a repeatable
flag cannot survive it, per review P1):

```
--expertise "<path> :: <purpose> :: <read this to …>
<path> :: <purpose> :: <read this to …>"
```

- Each non-empty line splits on ` :: ` into exactly three non-empty segments;
  anything else refuses with a typed message (malformed input is a validated
  row, per the injectable-control-token pattern — never silently dropped).
- `<path>` renders absolute, joined onto the **worktree root** when relative —
  the same base the brief's sibling `files` list already joins onto
  (`mailbox.rs:130`; knowledge lives in the working tree, never the control
  store). The flag does not police the path — the dispatcher's judgment picks
  it (D2 targets are `skills/…` reference files and bee knowledge files).
- Both commands register the flag in
  `packages/bee-rs/crates/bee/src/generated/registry_payload.json` — the
  hand-edited flag source (decision 3358743e) — and `catalog.rs`'s
  `PINNED_FLAG_COUNT` bumps 177 → 178 once (distinct names, so the second
  command's registration adds nothing).

### Slice 1 (all of it — no later slice)

**Cell 1 — herding run: plumb, render, rescope (code)**
Files: `packages/bee-rs/crates/bee/src/herding/run.rs`,
`packages/bee-rs/crates/bee/src/herding/mailbox.rs`,
`packages/bee-rs/crates/bee/src/generated/registry_payload.json`,
`packages/bee-rs/crates/bee/src/catalog.rs` (PINNED_FLAG_COUNT bump).
- `parse_options`: `--expertise` parsed into
  `Vec<ExpertiseEntry {path, purpose, read_to}>` per the entry format above;
  malformed → typed refusal.
- `BriefSpec` gains `expertise: &[ExpertiseEntry]`; `render_brief` renders,
  between `# Task` and `# Working directory (absolute)` (D1):

  ```
  # Expertise — read these before you start

  The dispatcher picked these files for this task. Read each one before
  working; they carry the know-how the task needs. Reading them is allowed
  and expected — they do not pull you into any workflow.

    - <abs path> — <purpose>. Read it to <read-to>.
  ```

  Zero entries: section omitted entirely (D1).
- D3 rescope of the opening clause: "Never run any `bee` command." and the
  "Never claim, cap, or write workflow state under .bee/ - writing your
  mailbox result file (described below) is the ONE exception." sentence stay
  in force verbatim; the blanket "IGNORE any bee or agent-workflow
  instructions" sentence (the brief's only skip-instruction, per review) is
  reworded to scope the ignoring to *workflow participation*, and to state
  that files listed under Expertise are yours to read. The rendered brief
  must contain no sentence that tells the worker to skip the listed files (D3).
- `job.json` gains `"expertise": [{path, purpose, read_to}]`; the
  `--continue` path reads entries back from `job.json` (fresh flags win).
  Recorded deviation from the existing continue precedent (task comes from
  flags only): entries persist because round N+1 keeps the same job, and the
  dispatcher should not have to restate them.
- Tests (mailbox.rs + run.rs unit tests, existing style): render with 2
  entries / with 0 entries; malformed refusal; job.json round-trip on
  continue; dry-run brief carries the section.

**Cell 2 — dispatch prepare: same entries for Task-tool workers (code)**
Files: `packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs`,
`packages/bee/prompts/worker-cell.md`,
`packages/bee-rs/crates/bee/src/generated/registry_payload.json`,
`docs/history/codex-harness-hardening/release-manifest.json` (regen output).
- `bee dispatch prepare` accepts the same single `--expertise` flag (one
  entry per line), same parse + typed refusal. Two wiring points the review
  pinned: `keys_known` at `prepare.rs:1104` gains `"expertise"` (else the
  verb silently delegates), and `prompt_body_for` gains an explicit
  `expertise` parameter threaded from the call site at `prepare.rs:622` —
  `learned_context` is derived inside that function, so the CLI-fed value
  needs its own param, not the same derivation.
- `worker-cell.md` gains a `{{#if expertise}}` block titled "Expertise —
  dispatcher-picked; read/load before implementing" (a Task-tool worker may
  be told to load a skill directly, D4). No template-test edit needed:
  `real_templates_render_end_to_end` auto-discovers `{{…}}` vars.
- `packages/bee/prompts/**` is a release-manifest inventory root: run the
  full regen chain inside this cell (`bee dev regen`) and list the manifest
  file above.
- Serial after Cell 1 — both cells hand-edit `registry_payload.json` (the
  real overlap; catalog.rs's count bump lands once, in Cell 1).

**Cell 3 — skill prose (docs)**
Files: `skills/bee-swarming/SKILL.md` (orchestrator dispatch step),
`skills/bee-swarming/references/swarming-reference.md` (prompt template
details), `skills/bee-herding/references/operational-invariants.md` (its
verbatim herding-run flag enumeration at lines ~195–198 gains
`--expertise` — D2 names the herding dispatch surface too),
`docs/history/codex-harness-hardening/release-manifest.json` (regen output).
- One short rule in the swarming files: the dispatcher composes Expertise
  entries leader-style — path, purpose, read-to — via `--expertise`, choosing
  from bee's own skill references and knowledge files; optional,
  judgment-driven, never auto-derived (D2). No restating of the template text.
- `skills/**` is a release-manifest inventory root: full regen chain inside
  this cell.
- Checked and closed (CONTEXT.md deferred question):
  `skills/bee-herding/references/dispatch-prompt.md` is 13 lines and composes
  no `herding run` command — no change there.
- Knowledge-area sync (`docs/knowledge/areas/bee-herding/handing-a-foreign-agent-its-brief.md`)
  belongs to capture, not this cell.

## Smaller path check

Considered: skip Cell 2 and let the swarming orchestrator paste expertise
into free text. Fails D4 — the locked decision names the swarming brief shape
explicitly. Considered: auto-derive entries from knowledge context. Rejected
by D2 (no derivation machinery). Shape stands as the smallest honoring D1–D4.

## Proof

- Cells 1–2: `cargo test --release --manifest-path packages/bee-rs/Cargo.toml`
  scoped to touched crates' related tests, recorded per cap; cells 2–3 add
  `bee dev release-manifest --check`. CI runs the full declared command on
  every push.
- Cell 3: pointer/parity check — prose cites the flag spelling exactly as
  `--help` prints it.

## Rollback

Additive flag + optional section: reverting the commits restores the prior
brief byte-for-byte (zero-entry renders are unchanged by construction).
