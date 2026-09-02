---
artifact_contract: bee-plan/v1
mode: high-risk
# approved_gate2: <unset until approval>
---

# Plan: Verification In The Flow

Mode: `high-risk` — 4 risk flags: public-contracts, covered-contract-change, proof-weakening, multi-domain
Why this is the least workflow that protects the work: D6 deletes proof a shipped door is documented to run, and D1/D2 change a path contract every host repo inherits. Both are one-way once released, and neither is caught by any test that exists today.

**Revision note.** This is the post-hat-wave draft. The five-seat wave (`84485816`) changed it in eleven places; every change is marked **[wave]** at its point of use. The wave's own synthesis is § Wave synthesis.

## Requirements (from CONTEXT.md)

- **D1** `d0e3c3a0` — fixed skill name `verify-app` in every repo; never `bee-` prefixed.
- ~~**D2**~~ `28140420` — **RETIRED at the gate by D8.**
- **D8** `9f4f90f0` — source stays nested at `.bee/verify/verify-app/`; the renderer's subdirectory walk is unchanged. D3's branch reads `.bee/verify/verify-app/SKILL.md`.
- **D3** `65592f3f` — `bee onboard` branches two ways on the verification skill's existence: absent → `bee-verifying`, present → `bee-verify-upkeep`. Onboard offers, never generates. *(Rationale corrected at this step — § Discovery F1, decision `57064e88`.)*
- **D4** `c93a6948` — the feature map is READ-FIRST: index at shaping, matching feature file at planning and in the worker brief.
- **D5** `036e8a79` — the drive rides the cap proof line as `green:live`; one case added to the proof-by-change-type list.
- **D6** `2a8eac15` — no composition into `commands.test`. Supersedes `verification-ships-to-hosts` D2 (`d79baa77`).
- **D7** `2effbe54` — bee regenerates its own `verify-bee` into this shape.

## Load-bearing claims

Labels: `read` = the bytes below were opened at the anchor; `ran` = the command was executed this session and this is its output; `guessed` = unverified. No `guessed` row survives the gate. Multi-line evidence joins with ` / ` and keeps every source prefix, including `//`.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The `bee-*` skill sync DELETES a bee-named target directory absent from bee's own source, so D1's refusal of a `bee-` prefix is not a preference | read | `packages/bee-rs/crates/bee/src/onboard/skills.rs:954-967` | `fn foreign_bee_skill_in_target_is_removed_but_non_bee_is_untouchable() {` / `write(&target.join("bee-legacy").join("SKILL.md"), "old");` / `write(&target.join("my-own-skill").join("SKILL.md"), "mine");` / `assert_eq!(items[0]["action"], "remove_skill");` / `assert_eq!(items[0]["skill"], "bee-legacy");` |
| 2 | The ownership axiom behind claim 1 is stated in the renderer itself | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:359-362` | `// and that is the whole design (plan.md, "Render, never prune"). The `bee-*`` / `// sync may prune a target directory absent from source because bee MINTS` / `// those names and nobody else does; `verify-` is a generic English word whose` / `// source lives in the mutable host repo.` |
| 3 | D3's originally recorded rationale is FALSE against current code: the offer is already an `else if`, not nested in the retired-key branch | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:203-217` | `if commands.is_some_and(\|c\| c.contains_key("verify")) {` / `} else if !has_test {` / `// so this arm is unreachable for it. It is the only arm that reaches` / `// a repo onboarded after 2.1.0, which never had `commands.verify`.` |
| 4 | The REAL D3 gap: the offer is gated on the repo declaring no test command, so any repo with a `commands.test` never sees it | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:209` · `templates.rs:304` | `} else if !has_test {` · `pub const NO_TEST_VERIFICATION_OFFER: &str = "This project has no command that proves it works. …"` |
| 5 | An existing test asserts today's behavior directly — a declared test command draws NO notice — so D3 has a free red-first | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:465-473` | `fn a_declared_test_command_draws_no_notice_in_any_shape() {` / `for declared in [json!("cargo test"), json!(["cargo test", "cargo clippy"])] {` / `stale_notices_for(json!({"commands": {"test": declared}})).is_empty(),` |
| 6 | The notice constants are held to a no-internal-terms hygiene test that the new upkeep constant must also pass — and it bans the string `.bee` | read | `packages/bee-rs/crates/bee/src/onboard/notices.rs:475-489` | `fn the_verification_offer_speaks_to_the_user_without_internal_terms() {` / `for banned in [".bee", "commands.", "config.json", "gate", "cap ", "proof"] {` |
| 7 | The `.bee/verify/` renderer enumerates SUBDIRECTORIES on purpose — the inner directory is the skill's namespace, which is what D2 proposes to remove | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:495-499` | `for entry in entries {` / `// Enumeration is the WHOLE root, deliberately not` / `// `list_bee_skill_entries` — that helper filters on the `bee-` prefix,` / `// which is bee's own deletion domain and says nothing about a host's` / `// generated skill. Here the containing directory IS the namespace.` |
| 8 | No `.bee/verify/` directory exists on this machine — checked twice, including gitignored paths | ran | `fd -t d -H --glob '.bee/verify' ~/Projects` and the same with `-I` | (no output — zero matches, both runs) |
| 9 | **`bee-verifying` has never shipped in a release**, so claim 8's local result generalises: no external host can hold the nested shape or a composed `commands.test` | ran | `git tag --contains 55b9ce9a4690cd366f7ec37e7ffe8931bca79aea` · `git tag --sort=-v:refname \| head -3` | (no output — the commit is in no tag) · `v2.31.0` / `v2.30.0` / `v2.29.0` |
| 10 | `green:live` already means exactly what D5's drive proof is, so D5 needs no new proof vocabulary | read | decision `cb7b14b7` (`proof-strength-and-expiry`), `.bee/decisions.jsonl` | `Meanings are pinned at the definition — live: the real product was driven and its result inspected; unit: automated tests passed; static: compiled/type-checked/linted/parity-checked with nothing executed.` |
| 11 | The proof-by-change-type list D5 extends is ONE bullet with three parenthesised cases — so D5 is a fourth case, not a new bullet | read | `packages/bee/AGENTS.block.md:112-115` | `- The agent owns test scope: pick the proof your change type needs` / `(code → related tests green; docs → parity/pointer checks; behavior` / `→ judge verdict), run it yourself, and record it on the cap as a` / `proof line `<command> — <result> — <scope reason>`.` |
| 12 | The read-first rule D4 extends is a two-line sentence naming exactly one state layer | read | `packages/bee/AGENTS.block.md:200-201` | `` `docs/knowledge/` is the state layer: read it first, sync it when`` / `behavior changes.` |
| 13 | D6 deletes three sections of `bee-verifying`, one of which records a non-obvious WHY that must survive the deletion | read | `skills/bee-verifying/SKILL.md:141,170,187` (sections); quote at `:196-198` | `bee's render does not carry the executable bit into the copies it writes into the` / `runtime skill homes (source `0755`, every rendered copy `0644`), so a bare path` / `fails with `Permission denied` the first time a host onboards` |
| 14 | `bee-verify-upkeep` opens with a locate-the-target step that a fixed name deletes outright | read | `skills/bee-verify-upkeep/SKILL.md:49` | `0. **Locate the target.** Find the verification skill to maintain: the` |
| 15 | Skill-touching cells cannot run concurrently in this repo | read | `docs/history/pstack-gaps/CONTEXT.md:69` | `Every skill file has generated copies under five plugin trees and the regen chain rewrites one shared release manifest — skill-touching cells are SERIAL, never concurrent (`pstack-adoption` plan rows 14-15).` |
| 16 | Each of the two verify skills has SIX copies — one source plus five rendered trees — which is what claim 15's constraint costs in practice | ran | `fd -t d -H '^bee-verifying$\|^bee-verify-upkeep$' .` | `./.agents/skills/…` / `./.claude-plugin/skills/…` / `./.claude/skills/…` / `./.codex-plugin/skills/…` / `./.opencode/skills/…` / `./skills/…` (each name, six paths) |
| 17 | `rule_index_parity.rs` exists and compares marker SETS, so it does NOT catch an unregenerated body edit — the D4/D5 doctrine risk | read | `packages/bee-rs/crates/bee/tests/rule_index_parity.rs` exists; `packages/bee/AGENTS.block.md` carries 12 `rule:` markers | `ls tests/` → `rule_index_parity.rs` · `rg -c 'rule: ' packages/bee/AGENTS.block.md` → `12` |
| 18 | The 12 product files: 3 Rust (`notices.rs`, `templates.rs`, `tests.rs`), 5 skills (`bee-verifying`, `bee-verify-upkeep`, `bee-shaping`, `bee-planning`, `bee-swarming`), 1 doctrine (`AGENTS.block.md`), 3 dogfood (`.bee/verify/` tree) | read | the anchors of claims 4, 6, 11, 13, 14 plus `.claude/skills/verify-bee/` | (each file named above was opened at an anchor in this table or in § Discovery) |

## Discovery

Three reality touches and one wave changed the draft.

- **F1 — D3's rationale was wrong (claims 3, 4).** CONTEXT claimed the onboard offer is nested inside the retired-`commands.verify` branch and so never fires. The current code already fixed that; `verification-ships-to-hosts` shipped the repair and this CONTEXT repeated the pre-fix description. Corrected in CONTEXT.md, logged as `57064e88`. The decision stands and its real scope is **larger**: the offer's gate is `!has_test`, so a repo that declares a test command — most repos, bee's own included — gets no verification notice at all, and no repo of any kind gets an upkeep pointer.
- **F2 — D2 cost a deliberate design and bought nothing D1 had not already bought (claims 7, 8, 9). RESOLVED: D2 retired, D8 locked.** The renderer enumerates subdirectories on purpose and says so: "the containing directory IS the namespace". D1's fixed name already yields a constant single path, `.bee/verify/verify-app/SKILL.md`, exactly as checkable for D3's branch. All five wave seats reached this independently; the user chose the nested shape at the gate. The renderer is now untouched by this feature. **[wave]**
- **F3 — D6's deletion carries a WHY that must not die with it (claim 13).** The composition sections hold the reason rendered copies are invoked as `bash <path>`: source `0755`, rendered copy `0644`. D6 deletes the composition, not that fact. It relocates to the cap-proof guidance, where the drive is now invoked.
- **F4 — the migration population is provably zero, with an expiry date (claims 8, 9). [wave]** `bee-verifying` is in no release tag. So no external host can hold the nested shape, a rendered `verify-*` copy, or a composed `commands.test`. This is what makes D6 free today. **It expires at the next release tag** — see § Named constraint.

## Named constraint — this feature lands before the next release tag

Claim 9 is the load-bearing fact under "no migration needed", and it is perishable. If a release is cut between `verification-ships-to-hosts` and slice 2 landing, two things become public and irreversible: the per-project `verify-<app>` naming that D1 replaces with a constant, and the instruction to compose the drive into `commands.test` that D6 removes. A released host carrying either needs a migration this plan does not contain.

D8 removes the third and sharpest edge outright. The retired D2 would have made an older binary render a flattened root's `features/` directory as a skill literally named `features` into all three runtime homes — a non-`bee-` name that nothing in bee can ever prune (claim 1's mechanism excludes it by design). With the nesting kept, an older binary reads the tree exactly as it does today.

**The constraint:** no release tag between now and slice 2's merge. If one is cut, this plan grows a migration cell and re-gates. Recorded here because no test can enforce it.

## Approach

Recommended path: three slices, **split by artifact kind, not by decision** — Rust first, then all text in one serial pass, then the dogfood. **[wave: hat-alternatives]** The original draft split slice 1 as "Rust + the two skill files", which forced two serial passes over the same skill files and two `bee dev regen` runs (claims 15, 16). Moving the line saves one of each. Between slices the skill bodies still say `verify-<app>` while the binary checks `verify-app`; that skew is worktree-local and invisible to hosts, and claim 9 says there are no hosts.

Rejected alternatives:

- *One slice for everything* — 12 product files across three artifact kinds with a serial constraint on six copies of each skill file. Caps as one commit, loses per-area proof.
- *Doctrine first (D4/D5), contract second* — the read-first rules would name a path and a constant that do not exist yet.
- *Two slices (merge doctrine and dogfood)* — viable, but the live drive then serves as proof for both and neither gets its own.
- *Keep D6's composition and add the drive to the proof line too* — the drive runs twice per cap and the red-result ambiguity D6 exists to remove survives.

## Decisions taken at this step (CONTEXT's deferred questions and the wave's open ends)

These were planning's to settle. Each is settled here, not carried to the gate.

1. **The existence check reads SOURCE, not a rendered home** — `.bee/verify/verify-app/SKILL.md`. **[wave: G2]** A fresh clone before any `--apply` has the source but may have no rendered copy; reading a rendered home would flip the branch on clone state rather than on whether the repo has a verification skill.
2. **No preamble verification line.** **[wave: G5 — CONTEXT deferred Q2]** D1 makes the name a constant, so a preamble field would restate a constant every session. The doctrine lines name it directly instead.
3. **A composed host is left alone, recorded.** **[wave: R3]** bee never edits `commands.test`, and claim 9 says the population is zero. `bee-verify-upkeep` gains one line: a test command containing a drive path is reported to the user with the reason, never rewritten.
4. **State A and state B carry different first sentences.** **[wave: hat-user-impact]** The existing constant opens "This project has no command that proves it works" — false for a repo that declares a test command. Two constants, both passing the hygiene test of claim 6 (which bans the string `.bee`, so neither notice may name the path).
5. **Anti-nag: the generate offer is not re-offered blind.** **[wave: impact 1]** Onboard re-runs on every version mismatch, and bee tags several releases a day. The generate notice therefore instructs the agent to search the decision log for a recorded refusal before offering, the way `bee-verifying` already instructs for its own second question. The upkeep notice is a pointer, not a prompt: it names the skill and stops.
6. **D4 is recorded as an unproven theory with a named falsifier.** **[wave: hat-value]** No evidence in this repo shows an agent failing for want of a feature map. The falsifier: after two mapped features, check whether plan risk maps actually cite feature-file gotchas. If they do not, D4's shaping tier is reverted.
7. **D4's absent-map behavior is silence; its stale-map behavior is not.** **[wave: impact 3]** Absent or empty map → the rule checks the fact first and proceeds, per AGENTS.md's own "a rule that presupposes an environment fact checks that fact first". A stale map at planning is the dangerous case and gets no silent trust: the worker brief carries the feature file's path plus its last-modified date, and the cell says so.
8. **D5 lands in TWO places, one of them step-proximal.** **[wave: hat-value]** The AGENTS.block.md case is the single home for the fact. A pointer beside `bee-swarming`'s cap step is what actually fires at the moment a proof line is written; a case in the middle of a long always-loaded contract, alone, is text that rots.
9. **The supersede names `bbedc1d2` as mooted alongside `d79baa77`.** **[wave: hat-value]** `bbedc1d2` designs a second consent moment whose only purpose is gating the write to `commands.test`. D6 removes that write, so the decision would otherwise dangle as active guidance for a mechanism that no longer exists.
10. **Slice 3 removes the old twin in the same commit.** **[wave: R5]** `.claude/skills/verify-bee/` is non-`bee-` named, so no bee path will ever prune it; two verification skills would coexist and their maps would drift. `git rm` is a must-have line on the slice-3 cell, not a nicety.

## Risk map

| Component | Risk | Proof needed |
|---|---|---|
| `plan.rs` renderer | **NONE — D8 removed this component.** The subdirectory walk is untouched | the existing verify-render tests stay green, unchanged |
| `notices.rs` branch (D3) | MEDIUM — changes what every host sees at onboard; claim 5's test flips red on its own | red-first over all SIX states (skill × test × legacy-key) |
| **Doctrine regen drift (D4, D5)** | **HIGH [wave: R4]** — `rule_index_parity.rs` compares marker sets only (claim 17), and CI has no regen-diff step, so an edit to `AGENTS.block.md` without regen leaves the file agents actually load silently stale | a byte-parity assertion: render the block, compare to `AGENTS.md` |
| Rendered-copy overwrite of map memory | MEDIUM **[wave: R1]** — a teammate editing a rendered `verify-app/features/` file loses it at the next `--apply`; D4 raises the stakes by making the map doctrine-level memory | the skill body states "edit only the source" at the top of the map section |
| `bee-verifying` deletions (D6) | MEDIUM — a large removal that must preserve one non-obvious fact (claim 13) | doc parity check; the `0644` fact provably present at its new home |
| Cross-surface name skew | MEDIUM **[wave]** — between slices, and after any partial revert, the binary and the skill bodies can disagree about `verify-app` with no test to catch it | one text-parity test over the constant, in the `route_class_parity.rs` shape |
| Read-first load points (D4) | LOW — pointer lines into three skills | text parity |
| `verify-bee` regeneration (D7) | MEDIUM — mostly a MOVE, not authorship **[wave: hat-alternatives]**; a broken harness is invisible until something needs proving | drive one mapped feature end to end, `green:live` |
| Release cut mid-feature | HIGH, unenforceable **[wave: R2]** | § Named constraint; human discipline, stated because no test can hold it |

## Shape

**Slice 1 — Rust only (current slice).** `notices.rs` branches on the existence of `.bee/verify/verify-app/SKILL.md` instead of on `!has_test`; `templates.rs` gains the reworded generate constant and one new upkeep constant; `tests.rs`/`notices.rs` tests cover six states. No skill file, no doctrine file, so nothing serial and nothing to regen. Proven by `cargo test`.

**Slice 2 — every text surface in ONE serial pass, then one regen.** `bee-verifying` (constant name and path, delete the three composition sections, relocate the `0644` fact), `bee-verify-upkeep` (delete step 0, add the composed-host report line), `AGENTS.block.md` (D4 read-first mention, D5 fourth case), `bee-shaping` (index read), `bee-planning` (feature-file read), `bee-swarming` (worker-brief carry plus D5's cap-step pointer). Then `bee dev regen`, the new byte-parity assertion, and the name-parity test.

**Slice 3 — the dogfood.** Move `.claude/skills/verify-bee/` to the chosen source shape, `git rm` the old tree, render into all three homes, and drive one mapped feature for a `green:live` proof.

Cells are prepared for slice 1 only, after the gate. Slices 2 and 3 stay one-line headlines.

## Test matrix

| Case | Kind | Where |
|---|---|---|
| skill absent + no test command → generate notice (state A wording) | unit, red-first | `onboard/notices.rs` |
| skill absent + test declared → generate notice (state B wording, differs from A) | unit, red-first | `onboard/notices.rs` |
| skill present + no test command → upkeep notice | unit, red-first | `onboard/notices.rs` |
| skill present + test declared → upkeep notice | unit, red-first | `onboard/notices.rs` |
| legacy `commands.verify` key + skill absent → retirement warning still wins **[wave: R6]** | unit, red-first | `onboard/notices.rs` |
| legacy `commands.verify` key + skill present → stated arm order, no double notice **[wave: R6]** | unit, red-first | `onboard/notices.rs` |
| the new upkeep constant passes the no-internal-terms hygiene test (claim 6) | unit | `onboard/notices.rs` |
| a `bee-`prefixed host skill is still pruned (claim 1 stays true) | unit, unchanged | `onboard/skills.rs:954` |
| source shape renders into all three `REPO_SKILL_TARGETS` homes | unit | `onboard/tests.rs` |
| rendered copy is `0644`; the documented invocation is `bash <path>` | unit | `onboard/tests.rs` |
| `AGENTS.md` is byte-identical to the rendered `AGENTS.block.md` **[wave: R4]** | text parity, NEW | `tests/` |
| the `verify-app` constant reads the same in the binary and in every skill body **[wave]** | text parity, NEW | `tests/` |
| the D4 read-first mention and the D5 fourth case are present in `AGENTS.md` **[wave: G6]** | text parity | `tests/rule_index_parity.rs` or the new file |
| bee's own `verify-app` drives one mapped feature | live | `green:live` proof line, slice 3 |

## Wave synthesis

Five seats, one draft, `84485816`. What each changed:

- **facts-gaps** — one BLOCKER: claim 12's anchor named `pstack-adoption/CONTEXT.md`, but the quoted bytes live in `pstack-gaps/CONTEXT.md:69`. Fixed (now claim 15). Three drifted line numbers fixed. Three prose-only load-bearing claims promoted to rows (5, 17, 18). Gaps G1/G2/G5/G6 answered in § Decisions taken and § Test matrix. **One correction back to the seat:** it asserted `bee-verifying` "has shipped in released bee"; `git tag --contains` says otherwise (claim 9), and hat-user-impact found the same independently.
- **alternatives** — the slice boundary reshuffle (§ Approach), D3's minimum shape (reword one constant, add one), D5 as a fourth parenthesised case rather than a new bullet, D7 scoped as a move. Verdict on Q: option (a).
- **user-impact** — the four-state SEE mock, from which decisions 4 and 5 come; the mapless-repo case (decision 7); the nag exposure. Verdict on Q: indifferent.
- **risks** — R1 through R6, of which R2 became § Named constraint and R4 became the risk map's new HIGH row. Also cleared the biggest fear outright: every deletion path in bee was traced, and a non-`bee-` named `verify-app` source tree is read-only to bee in all of them. Verdict on Q: (b) creates a real old-binary skew hazard.
- **value** — D2 is ceremony with negative value; D4 is a theory (decision 6); D5 needs the step-proximal pointer (decision 8); the supersede must name `bbedc1d2` (decision 9). Named the 80/20 subset as D1+D3+D6+D5, with D4's shaping tier and D7 as the first things to defer under pressure.

Cutting D2 also deletes the plan's only original HIGH-risk component — the renderer rewrite — and both remaining HIGH rows (regen drift, release timing) are ones the wave added.

## Open Questions

**None. The one open question was answered at the gate.**

**Q — Did D2's flattening stand?** No. The user chose to keep the nesting after seeing
both shapes with their costs. D2 (`28140420`) is retired; D8 (`9f4f90f0`) locks
`.bee/verify/verify-app/`. The renderer is untouched by this feature, the plan's only
original HIGH-risk component is gone, and a second host app skill remains representable.
