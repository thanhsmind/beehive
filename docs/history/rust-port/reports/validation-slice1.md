# rust-port — Validation Report (Slice 1: hooks port)

Date: 2026-07-26 · Lane: high-risk · Verdict: **READY WITH CONSTRAINTS** (post-repair)

## Shape

6 cells, waves (7,8) → 9 → 10 → 11 → 12. Flip REMOVED from this slice (decision 2026-07-26): onboard_bee's merge reverts hand-edited wiring (pattern filter `onboard_bee.mjs:2214-2242`, hash tracking `:3252-3261`; canonical source `packages/bee/hooks/catalog.mjs`) — all flips move to a dedicated slice executed once at the catalog source. Heavy hooks (session-init, prompt-context, state-sync, chain-nudge, session-close) and the statusline data path belong to Slice 2 / flip slice (plan.md Slice-1 row superseded by decision, plan file itself stays frozen).

## Spikes / evidence

- Registry dump: YES — 116 entries, 129 KB JSON, file-based script (`.bee/spikes/rust-port/dump_registry.mjs`).
- Onboard-revert behavior: traced with quotes (merge-not-overwrite, hash-tracked; hand-edits reverted).
- cargo test semantics probed live: two positional filters = hard arg error; unmatched name filter exits 0 (the zero-test-green trap); `--test <missing-target>` exits 101 — hence every verify moved to `--test <target>` form.

## Panel (NOT READY → repaired) + cold-pickup (4× CRITICAL → repaired)

- **B1/B2 + findings 1/5/8/11 (vacuous verifies):** all six verifies now use `cargo test --test <named-target>`; every cell carries a minimum-passed truth.
- **B3 (oracle triviality — load-bearing):** node hooks early-return 0 when `state.mjs` is missing from the root (`bee-write-guard.mjs:668`) or the hook is disabled (`:675`) — every conformance cell now mandates seeded temp roots (`.bee/bin/lib/` + hooks/ + onboarding.json + enabling config.json), asserts the node oracle's non-trivial verdict on deny fixtures, and includes a negative-control unseeded-root fixture.
- **B4 (missing deny classes):** write-guard split 3 ways — 9 core (gate/intake, reservations, holds, linked-invalid, containment, grants), 11 Bash path (extractBashTargets, checkGitBashCommand + bsg-1, internals-reach, CLI-shape/validate-args), 12 read side (checkRead, read-size/binary sniff, privacy marker, scout, apply_patch, AskUserQuestion autofix). All six panel-named classes have a home; none silently deferred.
- **W2:** holds/workspace/claims readers grew cell 8. **W3:** tokenizer oracle diffs BOTH mjs copies. **W4:** all 7 dispatch-guard deny classes enumerated in cell 10. **W5:** cell 11's verify prefixes the registry dump; per-root cache seeding stated. **W6:** 9→10→11→12 serialized via deps. **W8:** registry delivery at flip = release/onboard-generated managed artifact; loader shape final now. **W9:** invalid-source + advisory-vs-context fixtures added to cell 7.
- **Hook-enabled toggle (cold-pickup #2):** runtime honors `hookEnabled`; disabled-hook fixture per hook.
- **Git-spawn parity decision:** checkGitBashCommand keeps node's git spawn in Slice 1 (fires only on git-mutation-shaped commands; D5 tension named; gix revisit at flip slice).

## Constraints carried into execution

1. Ported hooks stay DARK (no wiring flip) until the dedicated flip slice.
2. Oracle seeding is mandatory rig infrastructure — an unseeded root is a rig bug, never a pass.
3. Test-target names are load-bearing (hook_conformance, guard_support, writeguard_core/bash/read, modelguard_conformance).
4. Registry cache: generated, gitignored, dev-produced by script; producer moves to release machinery at flip.

## Approval block

Gates 1–2 approved (feature-level). Advisor consult for Slice 1: `reports/advisor-slice1.md`; recorded via advisor-ref (projection seam workaround per P1 friction on file). Gate 3 re-approval for Slice 1 auto-approved under bypass total after the consult; audit decision logged.
