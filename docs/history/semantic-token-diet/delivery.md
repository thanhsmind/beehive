# Semantic token diet — Delivery record

**Feature slug:** semantic-token-diet
**Date:** 2026-11-16
**Base commit:** `b95c87ddb`
**Regen commit:** `d2ac1ce50` (`stdi-19`)

## Decisions and delivering cells

| Decision | Summary | Delivering cells | Verification / checker cells |
|---|---|---|---|
| D1 | One-off semantic diet; keep every rule, why-clause, and test-pinned wording; no size gate | All writers (`stdi-1`, `stdi-3`, `stdi-5`, `stdi-7`, `stdi-9`, `stdi-11`, `stdi-13`, `stdi-15`, `stdi-17`, `stdi-19`) | `stdi-2`, `stdi-4`, `stdi-6`, `stdi-8`, `stdi-10`, `stdi-12`, `stdi-14`, `stdi-16`, `stdi-18` |
| D2 | Fix stale and contradicting lines in owning files (`bee-hive`, `bee-planning`, playbooks, `bee-knowledge-upkeep`, `bee-herding`) | `stdi-9` (`4e5ae32d9`), `stdi-11` (`7c19ea731`), `stdi-13` (`dd1901a39`, `637bc8988`), `stdi-17` (`4a930b498`) | `stdi-10`, `stdi-12`, `stdi-14`, `stdi-18` |
| D3 | Move Execute/Tester contract to `skills/bee-swarming/references/execute.md`; `worker-cell.md` owns Result form; worker prompts load `execute.md` instead of the whole skill | `stdi-15` (`4a97259bc`, `51be84e9e`), `stdi-19` (`d2ac1ce50`) | `stdi-16` (`b90050ace`) |
| D4 | `bee-hive` stops marking `routing-and-contracts.md` as default read; shrink `LEADER.md` copies to pointers | `stdi-9` (`4e5ae32d9`, `68f2b6abf`), `stdi-19` (`d2ac1ce50`) | `stdi-10` (`2abff7a03`, `2d6de1cf9`) |
| D5 | Split `planning-reference.md` into `references/cells.md` and `references/plan-artifact.md`; repoint inbound links | `stdi-11` (`7c19ea731`, `fd64f2fe3`), `stdi-19` (`d2ac1ce50`) | `stdi-12` (`44745d680`, `35ef28cda`) |
| D6 | Move rare waggledance and CLI-before-MCP mechanics from `LEADER.md` to `packages/bee/playbooks/`; fold trigger index and deep contracts | `stdi-7` (`16491f2bc`), `stdi-19` (`d2ac1ce50`) | `stdi-8` (`3ef81c10c`) |
| D7 | Preamble trailer points routing at `bee orient` and `bee status --json` only for missing detail; drop `Proof-per-change-type` preamble line | `stdi-1` (`b519417c6`) | `stdi-2` (verified 6 existing pins) |
| D8 | Herding brief renders `# Proof command` only when the job has a cell id | `stdi-3` (`66ca1f554`) | `stdi-4` (`c611721a7`) |
| D9 | Single-command help prints first sentence, `flags:`, then remaining text under `details:`; trim 7 heavy descriptions and fix `control-loop` roles | `stdi-5` (`73d8810d5`) | `stdi-6` (`be207c310`) |
| D10 | Split `role-dispatch.md` at the interlock into `role-dispatch.md` and `role-dispatch-spawn.md`; move measurements to `spawn-proof.md`; point `operational-invariants.md` at `bee herding status --job` | `stdi-17` (`4a930b498`, `40c9b80ca`), `stdi-19` (`d2ac1ce50`) | `stdi-18` (`93ea0eefb`, `429dee0f5`) |
| D11 | Out-of-scope follow-ups filed as proposed PBIs (`p-806a3911`, `p-77bfcb18`, `p-6c3e4223`, `p-b97e839a`, `p-6b237338`) | Filed at planning | — |

## Bytes each branch stops loading

Measured from `git cat-file -s` comparing `b95c87ddb` (pre-feature) to `d2ac1ce50` (`stdi-19`).

| Branch | Before (bytes) | After (bytes) | Stopped loading (bytes) | How |
|---|---|---|---|---|
| Leader session start (`packages/bee/LEADER.md` + `LEADER.md`) | 34,925 | 25,231 | 9,694 | `packages/bee/LEADER.md` 27,455 → 20,960 (-6,495 B); `LEADER.md` 7,470 → 4,271 (-3,199 B) (D6) |
| Session-start preamble (trailer + proof line when `commands.test` is set) | 433 | 191 | 242 | `TRAILER_LINE` 236 → 191 B (-45 B); `Proof-per-change-type` line removed (-197 B) (D7) |
| Router default (`bee-hive` `SKILL.md` + default `routing-and-contracts.md`) | 52,549 | 7,519 | 45,030 | `SKILL.md` 9,704 → 7,519 B (-2,185 B); `routing-and-contracts.md` (42,845 → 32,765 B, -10,080 B) no longer loaded by default; `gates-and-delegation.md` also shrank 52,209 → 47,136 B (-5,073 B) (D4) |
| Tiny-lane planning (`bee-planning` `SKILL.md` + `cells.md` vs `planning-reference.md`) | 44,882 | 26,383 | 18,499 | `SKILL.md` 12,817 → 12,612 B (-205 B); `cells.md` (13,771 B) loaded instead of `planning-reference.md` (32,065 B, -18,294 B); standard/high-risk also load `plan-artifact.md` (15,074 B) (D5) |
| Worker execute (`bee-swarming` `execute.md` vs `SKILL.md` + `worker-cell.md`) | 27,040 | 13,764 | 13,276 | Worker loads `references/execute.md` (6,829 B) instead of `SKILL.md` (was 20,053 B, now 12,182 B, -13,224 B on branch); `worker-cell.md` 6,987 → 6,935 B (-52 B) (D3) |
| Idle herding dispatch tick (`bee-herding` `SKILL.md` + `role-dispatch.md`) | 40,624 | 19,448 | 21,176 | `SKILL.md` 8,720 → 7,311 B (-1,409 B); `role-dispatch.md` 31,904 → 12,137 B (-19,767 B), with spawn half in `role-dispatch-spawn.md` (13,521 B) (D10) |
| Non-cell herding brief (`render_brief`) | Project-dependent | 0 for `# Proof command` | ~170 on this repo | `# Proof command` header and project test command omitted when `spec.cell_id` is `None` (D8) |
| CLI help (`registry_payload.json`) | 358,978 | 358,437 | 541 | 7 heaviest descriptions trimmed and `control-loop` fixed; flags moved before `details:` so callers see flags before truncation (D9) |
