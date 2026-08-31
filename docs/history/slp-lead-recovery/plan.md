---
artifact_contract: bee-plan/v1
mode: standard
# approved_gate2: <unset until approval>
---

# Plan: SLP Lead Recovery

Mode: `standard` — 1 risk flag: covered-contract-change (a closed set with a
prompt-parity test asserting it). The 4 high-risk flags died with the machine
spawn; nothing here spends money, starts a process, or writes outside two files
plus a prompt.
Why this is the least workflow that protects the work: the whole feature is one
signal name, one rank, and one prompt paragraph — the proof that matters is the
existing parity test going green with them.

## Requirements (from CONTEXT.md)

- D1 no machine spawn; one `observation` with signal `dead-lead`.
- D2 report only what `bee status --json` already lists under `recovery.candidates`.
- D3 the note carries durable facts and a resume line; transcript text is data.
- D4 the old lead is never machine-closed.
- D6 `dead-lead` outranks `struggling-loop` in `observation_rank`.
- D7 no config switch.

## Load-bearing claims

Labels: `read` = opened that file at that line; `ran` = executed and hold the output.
No `guessed` row survives the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Signals are a closed 6-set; `dead-lead` must be added here | read | `packages/bee-rs/crates/bee/src/verbs/supervisor.rs:296-302` | `pub(crate) const KNOWN_SIGNALS: [&str; 6] = [` `"struggling-loop",` `"big-decision",` `"danger-op",` `"budget-overrun",` `"same-region-resubmit",` `"none",` |
| 2 | A shipped test pins the prompt to those closed sets, so the code and prompt halves are ONE cell — adding the signal alone turns a green test red | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:1069-1075` | `for signal in KNOWN_SIGNALS {` `assert!(body.contains(signal), "supervisor-prompt.md never names the signal {signal:?}");` |
| 3 | An unranked signal falls to 0 and is truncated below `struggling-loop` — D6's reason | read | `packages/bee-rs/crates/bee/src/verbs/supervisor.rs:1947-1954` | `fn observation_rank(signal: &str) -> u8 {` `match signal {` `"danger-op" => 3,` `"big-decision" => 2,` `"struggling-loop" => 1,` `_ => 0,` |
| 4 | Detection needs NO new code or tool: `bee status --json` already returns the candidates, and `bee status` is already in the observer's enumerated surface | ran + read | `bee status --json` → `recovery.candidates`; `packages/bee-rs/crates/bee/src/herding/control_loop.rs:299` | `{"candidates": [{"session_id": "49e28775-6532-4090-9f4e-39c083ae7af2", "lane": "compound-release-knowledge", … "work_signal": "lane", …}]}` and `const SUPERVISOR_ALLOWED_TOOLS: &str = "Bash(.bee/bin/bee status:*),\` |
| 5 | The candidate row already carries every fact D3's note needs | read | `packages/bee-rs/crates/bee/src/verbs/status_full/recovery.rs:398-421` | `row.insert("session_id"…)` `row.insert("lane"…)` `row.insert("runtime"…)` `row.insert("last_heartbeat"…)` `row.insert("work_signal"…)` `row.insert("since"…)` |
| 6 | The base is RED before any change — one pre-existing failure, unrelated to this feature | ran | `cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml` | `test both_installers_build_with_pipelining_disabled ... FAILED` / `panicked at crates/bee/tests/installer_contracts.rs:437:5:` / `install.sh runs the fallback cargo build with pipelining ON again` / `test result: FAILED. 13 passed; 1 failed` |
| 7 | That red is a stale literal assertion, not broken behavior — the script still disables pipelining | read | `scripts/install.sh:304` | `CARGO_BUILD_PIPELINING=false CARGO_TARGET_DIR="$BEE_SRC/packages/bee-rs/target" cargo build --release --manifest-path "$BEE_SRC/packages/bee-rs/Cargo.toml"` |
| 8 | A named commit inserted the token that broke the assertion, so cell 0 fixes the test and not the script | ran | `git log -1 --format='%h %s' -S 'CARGO_TARGET_DIR="$BEE_SRC/packages/bee-rs/target" cargo build' -- scripts/install.sh` | `ba1fe413 Pin the installer build's target dir to the path its binary check reads` |
| 9 | The supervisor already notices stale candidates in prose today — only the vocabulary is missing | ran | `tail .bee/supervisor/observations.jsonl` (main checkout) | `one stale recovery candidate (49e28775, compound-release-knowledge, quiet since 2026-08-25) is dead not looping` |

## Discovery

Three recon workers mapped the supervisor store, the recovery/session machinery,
and the paseo source; a five-seat hat wave then refused the original auto-spawn
shape on evidence (`hat-wave-synthesis.md`). The surviving finding: detection
already works end to end, and the only true gap is that a dead lead has no signal
name, so it lands at rank 0 and gets truncated out of the report that exists to
show it.

## Approach

**Recommended path.** One cell: add `dead-lead` to `KNOWN_SIGNALS`, give it a rank
above `struggling-loop`, and teach the shipped prompt to read
`recovery.candidates` and write the note with a resume line. One fix-first cell
before it, because the base is red.

**Rejected alternatives.**
- Automatic successor spawn — refused by the plan check on 6 blockers and 3 irreversible paths (`hat-wave-synthesis.md`).
- A separate `lead-recovery` mailbox kind — a mailbox row needs an addressee that reads questions at a turn boundary; a dead session never reaches one.
- A config switch — gates a spend that no longer exists.

**Risk map.**

| Component | Risk | Proof needed |
|---|---|---|
| Closed-set widening | LOW — a shipped parity test already guards the pair | That test green, in the same cell |
| Rank change | LOW — one match arm | Unit test asserting `dead-lead` outranks `struggling-loop` |
| Prompt wording | LOW — prose the tick reads | Parity test + the wording names the resume line |
| False positives | MEDIUM — a stale-but-alive lead gets named | Mitigated by shape: the note is a question to a human, never an action. The wave's 0/2 precision was fatal to a spawn and is acceptable for a note. |

## Shape

| # | Cell | Files | Proof |
|---|------|-------|-------|
| 0 | **Fix-first: the base is red.** `both_installers_build_with_pipelining_disabled` pins the literal `CARGO_BUILD_PIPELINING=false cargo build --release`; commit `ba1fe413` inserted `CARGO_TARGET_DIR=…` between the two tokens, so a correct line reads as wrong. Assert the contract — the fallback build's own line carries the setting — not adjacency. Behavior unchanged. | `crates/bee/tests/installer_contracts.rs` | The named test green, then the full suite green |
| 1 | `dead-lead`: add to `KNOWN_SIGNALS`, add its `observation_rank` arm above `struggling-loop`, and teach `supervisor-prompt.md` to read `recovery.candidates` from `bee status --json` and record the note with the resume line. Code and prompt in ONE cell — claim 2. | `verbs/supervisor.rs`, `skills/bee-herding/references/supervisor-prompt.md` | `the_shipped_prompt_pins_the_record_verbs_own_closed_sets` green, the set-consistency tests green, a new rank test, then the full suite |

## Test matrix

- **Set consistency** — `KNOWN_SIGNALS` widened without breaking the ordered set assertions.
- **Prompt parity** — the shipped prompt names every signal, `dead-lead` included (the existing test).
- **Rank** — `dead-lead` outranks `struggling-loop` and sorts below `danger-op`.
- **Empty** — no candidates: the tick still writes its `silence` row; nothing changes.
- **Adversarial data** — a candidate's lane name or transcript path is treated as data in the note, never as a command fragment the tick executes.

## Open Questions

None.

## Out of scope

- Any machine spawn, kill, close, or archive (D1, D4).
- A supervisor that decides matters on the human's behalf.
- The abandoned-lane nudge (recorded as a deferred idea in CONTEXT.md).
