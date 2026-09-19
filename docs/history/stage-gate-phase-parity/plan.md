# Stage gate phase parity — plan

**Feature slug:** stage-gate-phase-parity
**Route:** class=bugfix · lane=tiny · flags=[covered-contract-change] · product files=1
**Date:** 2026-09-20

## Summary

On the Pi runtime, bee narrows the model's active tool list once per turn. The
verdict comes from the `stage-tools` hook. That hook keeps its own list of
phases that may write. The list disagrees with `write_guard`, which is the
authority on when a write is allowed.

The result: in phase `grooming`, `idle` and `compounding-complete` the model
loses `edit` and `write`. `grooming` is a phase whose whole job is to change
code, and `write_guard` allows a source write there.

The fix deletes the second list. `stage-tools` asks the write guard instead, so
one fact keeps one home.

## Revision 1 — corrected after the independent read

The first draft claimed `write_guard` permits writes in all three phases. That
is false. The guard sorts phases into four groups, not two, and rev 1 follows
all four:

| Guard group | Phases | Tool gate |
|---|---|---|
| gated (`is_gated_phase`) | `exploring`, `planning` | narrow |
| terminal (`is_terminal_phase`) — writes refused outside `.bee/`, `docs/`, `plans/`, `AGENTS.md` | `idle`, `compounding-complete` | **open**, see the trade below |
| unrecognized (`!is_known_phase`) — EVERY write refused | a typo, a newer bee's phase | **narrow** |
| open | `swarming`, `reviewing`, `scribing`, `compounding`, `grooming` | open |

**The terminal-phase trade, chosen on purpose.** A tool list cannot say "only
under these paths". Narrowing `idle` would block the docs-lane edit the guard
allows there (`write_guard/tests.rs` proves `docs/plan.md` is permitted at
`idle`). Opening it hands the model a tool whose call on a source path earns the
guard's named intake refusal, which carries its own remedy. Rev 1 opens, and
takes the legible refusal over the blocked-but-permitted edit.

**The unrecognized-phase case** is not a trade: the guard refuses every write,
so write tools there only spend turns on calls that always deny. Rev 1 narrows.

## Discovery — the reality touch

Ran the shipped hook against a sandbox `.bee` store, one phase per run
(`.bee/bin/bee hook stage-tools`, `approved_gates.execution = false`):

```
idle                 tools=read,bash
exploring            tools=read,bash
planning             tools=read,bash
swarming             tools=read,bash,edit,write,find,grep,ls,powershell
reviewing            tools=read,bash,edit,write,find,grep,ls,powershell
scribing             tools=read,bash,edit,write,find,grep,ls,powershell
compounding          tools=read,bash,edit,write,find,grep,ls,powershell
grooming             tools=read,bash
compounding-complete tools=read,bash
```

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The hook's write-open set is a hand-kept list of four phases, separate from the write guard | read | `packages/bee-rs/crates/bee/src/hooks/stage_tools.rs:70-76` | `let is_after_gate = matches!(phase, "swarming" \| "reviewing" \| "scribing" \| "compounding");` |
| 2 | The write guard's GATED set is only `exploring` and `planning` | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:573-575` | `matches!(phase, Value::String(s) if s == "exploring" \|\| s == "planning")` |
| 2b | `idle` and `compounding-complete` are TERMINAL, a third group the first draft missed | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs:569-571` | `matches!(phase, Value::String(s) if s == "idle" \|\| s == "compounding-complete")` |
| 2c | A terminal-phase write is refused outside four prefixes while `guards.idle_gate` is on | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:602-616`, `guards.rs:42-43` | `if idle_gate_on && !under_allowed_prefix_intake(&normalized) { return Ok(WV::Deny(intake_refusal(` with `GATE_ALLOWED_PREFIXES_INTAKE: [&str; 4] = [".bee/", "docs/", "plans/", "AGENTS.md"];` |
| 2d | An unrecognized phase is refused for EVERY write, so write tools there are useless | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/checks.rs:689-696` | `bee phase guard: phase \"{}\" is not a recognized phase — writing \"{}\" is refused rather than silently allowed through an unhandled state.` |
| 2e | The guard's own phase list is importable, so the tests need no copy of it | read | `packages/bee-rs/crates/bee/src/hooks/write_guard/store.rs:18-25` | `pub(crate) const KNOWN_PHASES: &[&str] = &[` … `pub(crate) fn is_known_phase(phase: &Value) -> bool {` |
| 3 | `grooming`, `idle` and `compounding-complete` are known phases, so they are not typos | read | `packages/bee-rs/crates/bee/src/verbs/status_full/mod.rs:14-17` | `const KNOWN_PHASES: [&str; 9] = [ "idle", "exploring", "planning", "swarming", "reviewing", "scribing", "compounding", "grooming", "compounding-complete", ];` |
| 4 | The shipped hook really answers read-only for those three phases | ran | `.bee/bin/bee hook stage-tools` against a sandbox store, one run per phase | `grooming             tools=read,bash` — full output under `## Discovery` |
| 5 | The eight names in `FULL_TOOL_SET` are exactly Pi's eight built-in tools, so no built-in is lost | ran | `rg -n 'built-in tools' ~/.local/share/mise/installs/pi/0.85.1/pi/docs/extensions.md` | `2082:Extensions can override built-in tools (`read`, `bash`, `powershell`, `edit`, `write`, `grep`, `find`, `ls`)` |
| 6 | The four-phase list was the writing worker's own choice, not a locked decision | read | `docs/history/pi-native-stage-driver/plan.md:186,232` | `Slice 4 headlines: hard per-stage tool gate with a named re-open command, a user notice and a model notice (D4, D12)` — no phase is named anywhere in the cell packet |

## Smaller path check

Is there a cheaper shape? Adding `"grooming"` to the existing list is one word.
It fails: the two lists still disagree, so the same defect returns on the next
phase bee adds, and the unrecognized-phase hole stays open. Asking the guard is
smaller still — it deletes a list instead of extending one, and it inherits the
guard's own phase groups rather than restating them. PASS on the ask-the-guard
shape.

## Cells — current slice

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| `sgpp-1` | Derive the Pi stage tool gate from the write guard's own rule | `hooks/stage_tools.rs` | — | On Pi, a grooming session keeps `edit` and `write` instead of losing them, and a session whose phase bee does not recognize no longer gets tools every write would refuse | `cargo test -p bee --lib hooks::stage_tools` green, red first |

```json
[
  {
    "id": "sgpp-1",
    "feature": "stage-gate-phase-parity",
    "title": "Derive the Pi stage tool gate from the write guard's own rule",
    "lane": "tiny",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/paths.rs",
      "docs/history/stage-gate-phase-parity/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Make the stage-tools hook take its write-open verdict from the write guard instead of its own list. Today run_inner builds `is_after_gate` from a hand-kept four-phase match and then sets `execution_is_open = (!is_gated && is_after_gate) || gate_approved`. That list disagrees with `write_guard::is_gated_phase`, which gates only `exploring` and `planning`, so `grooming`, `idle` and `compounding-complete` wrongly come back read-only. Replace the whole computation with the four-group rule in \u00a7 Revision 1: approved execution gate wins outright; otherwise narrow when `is_gated_phase(phase)` OR `!is_known_phase(phase)`, and open everything else (terminal phases included). Delete `is_after_gate`. Import `is_known_phase` and `KNOWN_PHASES` from `write_guard` rather than re-copying either. Extract the mapping into one pure function, `pub(crate) fn allowed_tools_for(phase: &str, gate_approved: bool) -> &'static [&'static str]`, and have run_inner call it, so the rule is testable without stdout capture. Do not change READ_ONLY_TOOLS or FULL_TOOL_SET: the eight names are exactly Pi's eight built-in tools. Do not change the JSON key set the hook prints — the Pi belt reads `stage`, `stage_name`, `allowed_tools`, `user_message`/`user_notice` and `model_message`/`model_notice`, and dropping any of them breaks the belt. Keep the hook advisory: it still fails open and still returns SUCCESS on every path. Work RED FIRST: add the failing cases before the fix and record the red output. The existing test `stage_tools_decision_logic` asserts the `is_after_gate` local you are deleting; that is a structure assertion, so replace it with behavior assertions over `allowed_tools_for` rather than repairing it. Keep `stage_tools_returns_read_only_in_planning_phase_without_gate` and `stage_tools_fails_open_without_bee_store` working as they are.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --manifest-path packages/bee-rs/Cargo.toml -p bee --lib hooks::stage_tools",
    "must_haves": {
      "truths": [
        "grooming, idle and compounding-complete return the full tool set when the execution gate is unapproved",
        "a phase the write guard does not recognize returns read,bash, because the guard refuses every write there",
        "exploring and planning still return read,bash when the execution gate is unapproved",
        "an approved execution gate still returns the full tool set in every phase",
        "the hook's printed JSON keys are unchanged, so the Pi belt still parses it",
        "the phase-to-tools rule has one home and is derived from write_guard::is_gated_phase"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/stage_tools.rs",
          "substantive": "allowed_tools_for follows the four-group rule, is_after_gate deleted, KNOWN_PHASES imported not copied, and tests covering the literal phase table, the unrecognized phase, and the approved_gates.execution read out of state.json"
        }
      ]
    }
  }
]
```

## Out of scope — filed, not fixed here

The Pi belt intersects the allowed list with every tool it can see
(`.pi/extensions/bee-guard.ts:2546`). Pi's eight built-ins survive, but any tool
an extension or a skill registers is dropped at every stage, including the
stages that allow the full set. That is a separate defect in a separate file
and gets its own backlog row.
