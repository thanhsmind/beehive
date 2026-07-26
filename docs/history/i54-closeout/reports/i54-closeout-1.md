# i54-closeout-1 — worker report

Status: [DONE]

Outcome: helper + guard + doc converge on the live-probed codex 0.145.0
spawn_agent schema (D1, validation-canary.md). `dispatch prepare`'s ordinary
codex branch emits the doc-canonical `{task_name, message, fork_turns: "none"}`
shape (no `agent_type` — the field does not exist in the 0.145.0 schema, R18);
`evaluateCodexSpawn` now judges EVERY `spawn_agent` payload by tool name plus
the anchored `[bee-tier:]` marker in `message` — the doc-canonical shape gets a
real verdict instead of a silent noOpinion, the legacy 0.144.4
`{agent_type: "worker"}` shape is evaluated identically, the unmarked-spawn
deny is never weakened, and override fields stay unread (R18/B19 pass-through
gap intact); `swarming-reference.md`'s Spawn row states the probed schema and
helper parity. Round-trip tests cover both directions #54 flagged
(helper-emitted → guard, constructed doc-canonical → guard), red-first then
green.

Files touched:
- skills/bee-hive/templates/lib/dispatch-prepare.mjs
- skills/bee-hive/templates/lib/dispatch-guard.mjs
- skills/bee-swarming/references/swarming-reference.md
- scripts/test_dispatch_prepare.mjs (headline reshaped + doc-canonical/legacy round-trip check)
- hooks/test_model_guard.mjs (rows 47-49 fail-open→deny; new rows 58-59)
- docs/history/codex-harness-hardening/release-manifest.json (+ vendored .bee/bin and plugin projections via the regen chain)

Verify: `node scripts/test_dispatch_prepare.mjs && node hooks/test_model_guard.mjs && node scripts/release_manifest.mjs --check` — exit 0 (32 passed / ALL PASS / 510 files match).

Full trace and verification evidence: `.bee/cells/i54-closeout-1.json`.
