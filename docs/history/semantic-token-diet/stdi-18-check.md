# stdi-18 — check of stdi-17 (role-dispatch split)

Round 2. Checked both commits with trailer `cell: stdi-17`: 4a930b498 (the split) and 40c9b80ca (the dry-run fix). Checked against CONTEXT.md D1 and D10 and `reports/H-herding-help.md`. Line numbers are at 40c9b80ca. Old line numbers come from `role-dispatch.md` at 4a930b498~1.

Verdict: pass. The round 1 defect is fixed. The `--dry-run` section now sits in `role-dispatch.md` before §1, so every dry-run tick reads it.

## Truths

- PASS — an idle tick that stops at §2, §4 or §5 never needs `role-dispatch-spawn.md`. §2 ends the tick at `skills/bee-herding/references/role-dispatch.md:56-60`. §4 fallback and command failure end it at `role-dispatch.md:107-132`. §4 at cap skips the set at `role-dispatch.md:104-106`. §5 ends it at `role-dispatch.md:222-224`. The hand-off at `role-dispatch.md:234-235` fires only when the interlock is on and a slot is free. The `--dry-run` rules are now at `role-dispatch.md:17-30`, before §1, so a dry-run tick knows not to rename the pane or send text before it reaches any stop. The spawn file no longer holds a copy (`role-dispatch-spawn.md` ends at line 239). `skills/bee-herding/SKILL.md:47-48` names `role-dispatch.md` as the `--dry-run` protocol, which is true again.
- PASS — every rule from old §5 to §8 has one home. Each rule is listed below. The old quick-reference table (old 522-543) was a copy of rules that §1 to §8 already hold, so it had no rule of its own.
- PASS — no do-not-touch line was removed. The lane rule (old 250-281) is word for word at `role-dispatch-spawn.md:27-58`. Fallback and failure are never zero (old 83-118) at `role-dispatch.md:98-132`. One parenthetical was cut there ("the error envelope a stale or pre-D18 `bee` binary returns"), and the report's own finding 3 names that line (old 115) for cutting. "Always pass `--pane`" (old 58-63) is unchanged at `role-dispatch.md:73-78`. The loud record-worker failure (old 476-482) is at `role-dispatch-spawn.md:214-222`. The blind-retry line (old 462-463) is at `role-dispatch-spawn.md:202-204`. `SKILL.md` safety boundaries are untouched: the 4a930b498 hunks are lines 10-33 and 119-132 of the new file only, and 40c9b80ca does not touch it. bee-herdr is not in the diff. The route-prompt needles are covered by the green `control_loop` tests below.

## Moved or cut rules

- Interlock gate and its why — `role-dispatch.md:211-232`. Cut: "measured false" and "no home anywhere else" (incident narration, finding 3).
- Enable marker gesture and `status` reads it — `role-dispatch.md:229-231`, `skills/bee-herding/README.md:42,101` (finding 4).
- Occupancy plain-text form — cut from the role, which reads `--json`. Home: the output format at `packages/bee-rs/crates/bee/src/herding/wave.rs:972`.
- Four dispatchable conditions (a)-(d) — `role-dispatch-spawn.md:7-23`, word for word.
- Two-key lane gate, Key 2 refusal, checkable CoS — `role-dispatch-spawn.md:25-96`.
- CoS authoring gap tracked as `p-0b2812f8` — cut from live text. Home: the backlog record in the store (`.bee/backlog.jsonl` holds the id). The rule stays at `role-dispatch-spawn.md:91`.
- Rank, overlap skip, announce, end quietly — `role-dispatch-spawn.md:98-120`.
- Worktree, split-then-start, no-focus split — `role-dispatch-spawn.md:126-136`.
- Split parent and direction rule, ratio is the parent's share — `role-dispatch-spawn.md:138-148`. The measurements and the why (aspect-ratio splits starved width, the caller's pane lost rows) are at `skills/bee-herding/references/spawn-proof.md:243-247`. The sentence "width narrows at most once ... no worker pane drops below the width the transport needs to accept a submitted prompt" is restored at `spawn-proof.md:245` by 40c9b80ca.
- Settle before agent-start — `role-dispatch-spawn.md:150-156`, word for word.
- Close the pane on agent-start failure — `role-dispatch-spawn.md:164-169`.
- Config-driven argv and typed error on a bad token 0 — `role-dispatch-spawn.md:171-181`, plus `operational-invariants.md` "Runtime adapter".
- Opening instruction self-names with the bare slug, and why — `role-dispatch-spawn.md:183-189`.
- Never `-p`, bypassPermissions is accepted risk — `role-dispatch-spawn.md:191-196`.
- Confirm exactly one new pane, no blind retry — `role-dispatch-spawn.md:198-204`.
- Record the spawn, loud failure — `role-dispatch-spawn.md:205-222`. Cut: the id "herding-orchestration D18" and "§4 no longer counts panes".
- Give the human their view back — `role-dispatch-spawn.md:224-234`. Soft note: the why "a loop yanks the owner away on every spawn" is now only the heading. The focus fact is at `spawn-proof.md:110-117`.
- `--dry-run` — `role-dispatch.md:17-30`, word for word with the old text. Fixed in 40c9b80ca. See the first truth.
- SKILL.md Waves paragraph — `SKILL.md` References row for `wave-runs.md`, with `skills/bee-herding/references/wave-runs.md:5-12` (not for backlog work, no interlock or gate), `:21` (no panes or worktrees), `:55-63` (one bad name stops the wave), `:69-76` (`success` is false).
- SKILL.md dependency reasons — `SKILL.md:13,18,23`. Never auto-detected: `operational-invariants.md:155-156`. One vocabulary: `operational-invariants.md:223`. Decision ids live in the store.
- openai.yaml role list — `skills/bee-herding/agents/openai.yaml:3` names all six roles (finding 5).
- Transport kind report — `operational-invariants.md:161` now names `bee herding status --job <id>` (finding 13, D10).
- Route interval is a guess — cut from `route-prompt.md`. Home: `packages/bee-rs/crates/bee/src/herding/control_loop.rs:106,117` (plan.md OQ5, a starting guess).
- Route heading decision ids — cut (finding 12). The decisions live in the store.

## Other observations

- `role-dispatch.md:3` still says "The full protocol." It is now the first half.
- `role-dispatch.md` does not say in words to end the tick when the interlock is on and no slot is free. §4 at `role-dispatch.md:104-106` forbids building a set, and the spawn file is not read, so the tick still ends without a spawn.

## Verify

Command: the cell verify (`cargo test --release ... --bin bee -- control_loop` then `--test pointer_integrity`). Exit 0.

- `--bin bee -- control_loop`: two filtered runs, 1 passed and 75 passed, 0 failed.
- `--test pointer_integrity`: 10 passed, 0 failed.

No test reads the dispatch role text, so the verify does not prove the first truth. That truth rests on the read above.
