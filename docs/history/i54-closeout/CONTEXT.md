# CONTEXT — i54-closeout

Close every remaining open item from the issue #54 cross-check against v1.16.0.
Scope was investigated with file:line anchors before this feature opened (four
parallel gathers, 2026-07-24); this document locks the resolution decisions.

## Locked decisions

- **D1 — Dispatch schema converges on live-probed truth.** The three-way
  doc/helper/guard mismatch (`swarming-reference.md:134` says
  `{task_name, message, fork_turns}`; `dispatch-prepare.mjs:389-397` emits
  `{agent_type, message}`; `dispatch-guard.mjs:197` matches `agent_type` only)
  is resolved by probing the LIVE codex CLI on this machine (0.145.0) during
  validating, then aligning all three surfaces on the observed schema. The
  guard's coverage only widens (it must evaluate every spawn_agent payload
  shape the helper can emit and the doc teaches); the unmarked-spawn deny is
  never weakened. Round-trip tests must cover the doc-canonical shape through
  the guard — the exact untested direction #54 flagged.
- **D2 — run_verify gets per-suite timeout + heartbeat.** Default per-suite
  timeout 300 s, overridable via `BEE_VERIFY_SUITE_TIMEOUT_MS`; on expiry the
  suite is killed and reported as a distinct TIMEOUT failure. A heartbeat line
  (~30 s cadence) names the suites still running. The hermetic env scrub
  (verify-pipeline knowledge, R-hermetic) is untouched.
- **D3 — Knowledge budget is lane-scaled, default unchanged.** `knowledge
  context` gains optional `--lane tiny|small|standard|high-risk` mapping to
  budget presets (8k / 12k / 20k / 30k). Bare `--budget` keeps working and the
  default stays 20000 — no caller breaks. The preamble's recommended command
  picks the preset from the current mode. Budget cut semantics (prefix cut,
  critical-patterns exception — okf B5/B6) are untouched.
- **D4 — Herding spawn becomes config-driven, claude-default.** The working
  agent / control-pane command moves from a hardcoded `claude ...` string to a
  config template (`.bee/config.json` `herding.agent_command`, placeholder
  substitution) whose DEFAULT is byte-equivalent to today's claude command —
  zero behavior change without explicit config. A codex example is documented.
  Full codex-native herding (event loop, pane protocol) stays out of scope.
- **D5 — Doctor names the dual-source state instead of staying silent.**
  `doctorHookSourcesCodex` reports explicitly when BOTH `hooks/hooks.json`
  (plugin projection) and `.codex/hooks.json` (repo fallback) are present,
  keeps `active: unknown` honesty, and states the row-B1 premise it relies on.
  A both-present test locks the behavior. No hook file is deleted (D9).
- **D6 — Bypass matrix gets a consistency test, not a generator.** A verify
  suite parses the off/normal/full/total matrices in `README.md` and
  `skills/bee-bypass-gate/SKILL.md` and fails on semantic drift. A canonical
  generator is deliberately NOT built this round — the test is the cheaper
  honest guard; revisit only if a third copy ever appears.
- **D7 — Lane writes auto-resolve like lane reads.** `resolveMutationTarget`
  (state set / state gate / scribing-run) auto-resolves the calling session →
  bound lane when `--lane` is omitted, symmetric with `resolvePipeline` on the
  read path. Explicit `--lane` always wins; an unbound session still writes
  the default record; missing/corrupt lane still refuses loudly (B12/B13
  untouched). Session identity is self-resolved at operation moment (B22).
- **D8 — Capability pin bumps only on observed evidence.** Run the live canary
  (`scripts/canary_codex.mjs`) against codex 0.145.0; `PROBED_CODEX_VERSION`
  bumps to 0.145.0 with the probe record as evidence. Capability matrix rows
  change only per what the probe actually observed (R18: never judge an
  envelope no probe has seen).
- **D9 — No hook-file surgery.** `plugin.json → hooks/claude-hooks.json` is
  intentional and the file parses; `hooks/hooks.json` is a rendered hook
  target asserted by `test_conformance.mjs:427`. Neither file is removed or
  renamed in this feature.
- **D10 — Canonical sources only; vendored trees follow.** Every edit lands in
  the canonical source (`skills/`, `hooks/`, `scripts/`, root docs). The
  vendored `.bee/bin` and projections are synced via self-onboard `--apply`
  before verify — never hand-edited.
- **D11 — AGENTS.md shortening is descoped to its own docs-lane pass.** The
  #54 "rút gọn AGENTS.md" ask is deliberately NOT bundled here: AGENTS.md is
  the auto-loaded law file with migration-tested teaching surfaces; a length
  edit deserves its own reviewed docs pass, not a rider on an 8-cell feature.
  The lane-budget line update (D3) is the only AGENTS.md touch this feature.

## Out of scope

- Full codex-native herding runtime (adapter only, per D4).
- Bypass-matrix canonical generator (test only, per D6).
- AGENTS.md restructuring (per D11).
- Any weakening of the unmarked-spawn deny or the hook-source exclusivity
  proof gates (hook-runtime knowledge constraints).
