# Validation — rust-port-ledgers (slice 4 of the rust-port epic)

Lane: **high-risk** · Cells: `rpl-1` … `rpl-8` · Plan: `docs/history/rust-port-ledgers/plan.md`

## Reality gate

| Check | Verdict | Evidence |
|---|---|---|
| MODE FIT | PASS | 6 risk flags counted in `plan.md` incl. the audit/security hard-gate flag (this slice ports `SECRET_CONTENT_PATTERNS` / `INJECTION_PATTERNS` / `datamark`, `packages/bee/lib/decisions.mjs:11-26,1047-1054`). High-risk is the floor, not a choice. |
| REPO FIT | PASS | `cargo build --release --manifest-path crates/Cargo.toml` → `Finished \`release\` profile ... in 0.42s` (green today). The dispatch site to extend is real (`crates/queen-bee/src/main.rs:20-36`, flat single-word `match`); the parity harness is real (`crates/bee-parity/src/main.rs:343-438` `check_one_leg`, `differ::diff_legs` at `differ.rs:45-60`); the fixture generator is real (`crates/queen-bench/src/main.rs:48` `Some("--generate") => cmd_generate(...)`). |
| ASSUMPTIONS | PASS with constraints | See the feasibility matrix. One assumption failed and produced a cell repair (fixture coverage). |
| SMALLER PATH | PASS | Merging `rpl-2` (intent) into `rpl-1` was considered and rejected: `rpl-1` is deliberately seam-only so a red in the first group cannot be confused with a red in the harness. Merging `capture` into `intent` was rejected: capture depends on the security-critical `datamark` port, intent does not. |
| PROOF SURFACE | PASS with a repair | The `--cmd-check` arm diffs stdout + exit + the resulting file tree, and every scenario carries a seeded-mutation negative control (`bee-parity/src/main.rs:427-432`) — a rig that reports zero diff on real divergence is red. **Gap found:** the fixture generator writes no capture queue and no intent store, so those scenarios would have run over empty stores and proven nothing. Repaired into `rpl-2` and `rpl-3` as an explicit obligation (below). |

## Feasibility matrix

| # | Assumption | Risk | Proof required | Evidence | Result |
|---|---|---|---|---|---|
| 1 | The parity runner can be generalized from `status` to arbitrary argv | HIGH — the whole slice's proof surface depends on it | read the runner's argv construction | `crates/bee-parity/src/runner.rs:88-121` — the command is already built generically (`Command::new("node").arg(bee_mjs)` vs `Command::new(queen_bee)`, then `for arg in leg.args()`); only the `Leg` enum is status-specific. Generalizing to an argv slice is a mechanical change, not a redesign. | PASS |
| 2 | The regex crate already on hand can express every ported pattern | HIGH — an approximated security pattern is worse than none | read every pattern and check for lookaround / backreferences | `crates/bee-core/Cargo.toml:23` → `regex-lite = "0.1"`. Patterns at `packages/bee/lib/decisions.mjs:12-17` (secrets), `:22-25` (injection), `:1049-1051` (datamark) use only character classes, `\b`, non-capturing `(?:…)`, `{n,}` and the `i`/`g` flags — **no lookahead, lookbehind, or backreference**. All expressible in `regex-lite`; no new dependency, so D8's static/musl posture is untouched. | PASS |
| 3 | JS `String.prototype.trim()` and Rust `str::trim()` agree | MEDIUM — silent one-character drift in every datamarked field | compare the whitespace sets | They do **not** fully agree: JS `trim()` strips U+FEFF (BOM); Rust's `char::is_whitespace` (Unicode `White_Space`) does not. `datamark` ends in `.trim()` (`decisions.mjs:1052`). | FAIL → repaired into `rpl-3`'s corpus as a required case |
| 4 | The rejection error text can be reproduced byte-for-byte | MEDIUM — parity of a refusal is still parity | read the throw site | `packages/bee/lib/decisions.mjs:280-283` interpolates `${pattern}` — the **JS regex literal source**, e.g. `/\bAKIA[0-9A-Z]{16}\b/`. The Rust side must emit that exact JS-source spelling, not its own pattern's `Display`. | FAIL → repaired into `rpl-3` as an explicit obligation |
| 5 | The D9 lock is already ported and consumable | HIGH — `rpl-4` and `rpl-6` both depend on it | confirm the module exists | `crates/bee-core/src/lock.rs` (1126 lines), ported and conformance-tested in slice 0 (`rust-port-3`). Consumed, never reimplemented. | PASS |
| 6 | `readCell` exists in Rust for the reviews group's scope resolution | MEDIUM — `rpl-7` assumes it | find the function | `crates/bee-core/src/cells.rs:168` → `pub fn read_cell(root: &Path, id: &str) -> Option<Cell>`. | PASS |
| 7 | The generated fixture contains all six ledger stores | HIGH — an empty store makes a clean parity diff meaningless | grep the generator | `crates/queen-bench/src/fixture.rs` covers decisions, reservations, backlog, cells, review candidates + sessions, git commits, transcript tail (`fixture.rs:4-24`). It contains **zero** references to `capture-queue` or `intent`. | FAIL → repaired into `rpl-2` and `rpl-3` |
| 8 | No verb of the six groups is silently dropped | HIGH — a missed verb is an invisible hole in the port | enumerate the live CLI and diff against the cells | `bee <group> --help --json` run for all six: decisions 8, backlog 11, capture 4, reviews 7, feedback 4, intent 4 = **38 verbs**, every one named in a cell's `action`. | PASS |
| 9 | The dependency graph is schedulable | MEDIUM | run the scheduler | `bee cells schedule --feature rust-port-ledgers --json` → 8 waves, `cycles: []`, `unsatisfiable_deps: []`, `empty_files: []`. The fully-serial wave shape is expected, not a defect: every cell writes `crates/queen-bee/src/`, so file reservations serialize them regardless of the dependency edges. | PASS |

## Cell repairs applied

Applied via `bee cells update` (open cells, plan fields only) before Gate 3:

- **rpl-2** — added the obligation to extend `queen-bench::fixture` with an intent store, so intent scenarios diff over real data rather than an absent directory.
- **rpl-3** — added three obligations: extend the fixture with a capture queue; include U+FEFF in the datamark corpus (matrix row 3); reproduce the JS regex-literal source spelling inside the rejection message (matrix row 4). Also split the cell's two concerns explicitly — `datamark` (neutralize, `decisions.mjs:1047-1054`) is not the same code path as `assertSafeContent` (reject and throw, `decisions.mjs:277-292`); both are in scope and each needs its own oracle cases.

## Plan-checker (adversarial) — BLOCKERS OPEN (6), all repaired

| # | Finding | Repair |
|---|---|---|
| B1 | **The mandated fixture source cannot express the scenarios the cells demand.** `queen-bench --generate` produces monotone filler: every decisions row is type `decide` with an identical date, no `tags` key, no supersede/redact rows (`fixture.rs:287`); every backlog row is type `proposal` (`fixture.rs:296`) — so *neither* of the two row schemas `rpl-9`'s fold branches on is present. Ids are zero-padded, so byte order equals numeric order and `rpl-7`'s `review-1`/`review-2`/`review-10` case is unbuildable. No capture queue, intent store, decisions-archive, `docs/decisions/index.md`, `docs/backlog.md`, or learnings tree. | `rpl-1` obligation (D): `--generate` stays the authority for the **baseline** store — never hand-authored — but per-scenario seeding on the clone is now explicitly permitted and required. Every affected cell carries a "seed what you assert" truth and a prohibition against clean-diff acceptance over the unseeded store. |
| B2 | **`rpl-1`'s registry-derived refusal text is unrunnable over its own fixture.** The refusal is built from the registry entry's `parameters.properties` (`bee.mjs:7186-7188`); `load_registry` resolves `.bee/cache/command-registry.json` + `.bee/bin/lib/command-registry.mjs` (`write_guard.rs:1254-1255`) and `fixture::generate` writes neither, so it errors on every scenario. | `rpl-1` obligation (B): seed both into the fixture. Safe for the tree diff because `.bee/cache` is already in `EXCLUDED_PREFIXES` (`differ.rs:28`) — the point is that both legs resolve the *same* registry, not that it is diffed. |
| B3 | **The seeded-mutation negative control cannot fire for any ledger command.** `mutate.rs:25-34` flips `.bee/state.json`'s `phase` and refuses unless the file byte-equals `FIXTURE_STATE_BODY` (`mutate.rs:16`); `main.rs:428-432` then errors if the flip produced no stdout diff. **No** ledger verb prints `phase`, so every ledger scenario would fail its own control. | `rpl-1` obligation (C): the mutation target becomes per-scenario — each scenario declares the store it reads and the control perturbs *that*. A scenario registered with no mutation target is refused at registration, never silently skipped. |
| B4 | **`feedback.mjs:976` was misclassified as a `localeCompare` site** in both the plan and `rpl-8`. It is a compound **raw code-unit** comparator (rank desc → `first_seen` asc → key asc via bare `<`/`>`). Porting it as locale collation is exactly the divergence class the cell exists to prevent. | `rpl-8` reclassifies it explicitly; the real `localeCompare` sites are `:648`, `:649`, `:819` only. |
| B5 | **A shared constant with no owning cell, whose consumer runs first.** `bee.mjs:3687` `backlogAllowedTypes()` = `[...Object.keys(KIND_ALIASES), ...NORMALIZED_KINDS].sort()` — both exported from `feedback.mjs:67,:98` — **is** the `backlog add --type` refusal text, a byte-parity surface. `rpl-6` runs before `rpl-8`, so nothing stopped it capping with a hardcoded second list (a D7 violation). | Both constants assigned to `rpl-3`'s shared bee-core module; `rpl-6` gains a truth on the refusal text and a prohibition against any hardcoded copy. |
| B6 | **`rpl-8` named two of feedback's four sources.** `SRC_CELLS = .bee/cells` (`feedback.mjs:108`, read at `:531-553`) and `SRC_LEARNINGS = docs/history/learnings` (`:109`, read at `:573`) were missing; the cited `:106`/`:107` are label constants, not read sites. Dep set, `read_first`, scenarios and the "no bare filesystem read" prohibition all under-covered by half. | `rpl-8` names all four sources with a scenario each; `crates/bee-core/src/cells.rs` added to `read_first`. |

### Warnings accepted and folded in

- **W1/W3 (regex):** the cell named the wrong crate. Corrected to `regex-lite`, with the ASCII-`\b` separator case added; the unwritable lone-surrogate corpus item now requires a bytes-level transport or an explicit stated drop.
- **W2:** the three artifacts were anchored to one range holding only `datamark`; arrays split to `:11-18` / `:21-25`. More importantly, `capture.mjs:19,26` iterates the pattern arrays to **reject at write time** and never calls `datamark` — so the audit surface is the **refusal message**, now its own truth.
- **W4 (anchor drift):** four handler ranges corrected — decisions `1924-2229`, backlog `3670-4007` (`handleBacklogFindings` at `3997-4007` was outside), feedback `4448-4513` (`handleFeedbackRank` at `4504-4513` was outside), intent `4059-4134`.
- **W5:** `rpl-8`'s deps on `rpl-5`/`rpl-6` were not code dependencies — `feedback.mjs` imports nothing from `backlog.mjs` and parses both stores itself. Loosened to `deps: ["rpl-3"]`.
- **W6:** `cells.rs` documents its id sort as a deliberate byte-order narrowing of mjs's numeric `localeCompare` — the exact comparator class `rpl-7` exists to get right. `rpl-7` gains a prohibition against reusing it.
- **W7 (scope):** three cells sat at or past the plan's own reference size. **`rpl-6` split** into part 1 (counts/rank/badges/add/propose/findings + git argv + allowed-type text) and **`rpl-9`** (pbi.\* + render/`--check` + the race proof); **`rpl-5` split** into the query verbs and **`rpl-10`** (the index renderer + drift check, the single highest byte-exactness risk in the slice).
- **W9:** the per-group unknown-**verb** usage fallback (`bee.mjs:6586-6609`) was owned by nobody. `rpl-1` owns the mechanism; each group cell asserts its own line.
- **W10:** `feedback digest --out` **deliberately** bypasses the containment helper (`bee.mjs:4475-4481`). `rpl-8` now states this and adds an `--out` escape scenario, plus a prohibition against "fixing" it.
- **W11:** both remaining plan open questions were answered from source and folded in — the digest write is in the handler (`bee.mjs:4479`), and no ledger verb reads stdin, so argv scenarios suffice.
- **W12:** `read_cell` is sufficient for `rpl-7`; the one narrow gap (`Cell.id` is not `#[serde(default)]`) is recorded, not blocking.

## Cell review (cold pickup) — 10 CRITICAL across 7 of 8 cells, all repaired

Three distinct root causes, not ten problems:

1. **The shared `--cmd-check` verify was group-blind** — six of the ten CRITICALs. Once `rpl-1` lands its smoke scenarios, a bare `--cmd-check` exits 0, so a worker who ported the group and registered *zero* scenarios still got green. Repaired by mandating `--cmd-check --group <name>` (non-zero exit on zero registered scenarios, registered count printed) and pinning every cell's verify to its own group.
2. **`rpl-1` pointed at the wrong oracle for its own headline requirement** — it named `bee-core/src/validate_args.rs`, but `packages/bee/lib/validate-args.mjs:90` explicitly disclaims that job in its own comment, and the real producer is `bee.mjs:7186-7188`. The cell demanded byte-for-byte mjs wording while carrying **zero** mjs files in `read_first`. Repaired; the mjs oracle files are now first in the list.
3. **`rpl-3`'s regex-sufficiency claim was factually wrong** — see below.

### The finding that mattered most

`rpl-3` asserted regex-lite sufficiency from a construct inventory that **omitted `\s`**, which appears in five of the ported patterns. `regex-lite`'s `\s` is ASCII-only; JS's is not. Measured on this machine against the real module, JS `\s` matches U+00A0, U+1680, U+2000–U+200A, U+2028, U+2029, U+202F, U+205F, U+3000 **and U+FEFF**. So `ignore all previous instructions` is rejected by mjs and would have been **accepted** by the port — a parity red that is simultaneously a content-safety bypass of the injection guard. The mandated corpus covered U+FEFF in leading/trailing position only, so `datamark_oracle` would have gone green on it.

Switching to the full `regex` crate does **not** fix this: that crate's `\s` is Unicode `White_Space`, which excludes U+FEFF, so it diverges from JS in the other direction. The repair is to hand-enumerate the JS `\s` set as an explicit class and stay on `regex-lite` — no new dependency, D8's static/musl posture preserved. The `\b`/`\w`/`\d` half of the original reasoning was sound and stands.

## Verdict

**READY WITH CONSTRAINTS.** Both independent passes opened findings; all 6 BLOCKERs and all 10 CRITICALs are repaired in the cells, and the slice is now **10 cells** (`rpl-1` … `rpl-10`), zero cycles, zero unsatisfiable deps.

Constraints carried into execution:

- Every cell's verify is group-scoped and must fail on zero registered scenarios — `rpl-1` owns that mechanism and is the blocking dep for all nine others.
- Per-scenario seeding on the cloned fixture is mandatory wherever a scenario asserts something the monotone baseline store cannot express.
- No pattern may use a bare `\s`.
- The two lock implementations and the mixed `feedback` comparators stay replicated, never consolidated; a consolidation PBI is filed for after mjs retires.

## Gate 3

Auto-approved under gate-bypass level `total` (`⚡`), after the AO2b advisor consult
(`reports/advisor-slice4.md`) and with the audit decision logged. Recorded separately: the
AO3/AO13 precondition could not be satisfied through the documented path because
`bee state advisor-ref record` reports success but persists nothing — filed as a P1
harness-issue in `.bee/backlog.jsonl`, and only then was `advisor_ref` hand-written into
`.bee/state.json` using the anchors the verb itself computed, both independently re-derived and
confirmed matching before the write.
