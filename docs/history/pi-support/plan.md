---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Pi Support

Mode: `high-risk` — 4 risk flags: external-systems (Pi harness), public-contracts (hook catalog + dispatch door), covered-contract-change (belt parity test widens), multi-domain (Rust + TS + config + docs)
Why this is the least workflow that protects the work: the belt IS the enforcement layer — a translation bug is a silent guard bypass, so full plan + review wave + advisor consult + hard gates.

Review wave: reviewer dispatch 7f5b51fe (FAIL, 6 P1 + 4 P2 + 5 P3) and advisor consult f4be2431 (proceed-with-conditions, 4) — this revision folds in every named fix and condition. Key reversals from rev 1: NO `Runtime::Pi`, NO fourth projection (the catalog file itself names TS-plugin belts as exclusions); D5 enforced at full width (every non-herding resolution refuses, escalation included); model-guard is a NAMED EXCLUSION on Pi.

## Requirements (from CONTEXT.md)

- D1: repo-local `.pi/extensions/bee-guard.ts`, shipped by an onboard copy step; no global install.
- D2: helpers stay the first belt; event map `tool_call`/`before_agent_start`/`tool_result`/`agent_settled`.
- D3: blocking fail CLOSED; advisory fail OPEN.
- D4: catalog coverage for the Pi belt, parity derived never hand-listed. **Named deviation (reviewer P1-1):** `hook_manifests.rs:44-52` explicitly excludes TS-plugin belts from `Runtime` ("the plugin file is the projection" — OpenCode precedent), so D4's letter (catalog rows) is fulfilled the OpenCode way: the belt joins the PARITY TEST's derived row set (test-side), the named-exclusion comment extends to Pi, and `devtools/mod.rs:540` gains the `"pi" => None` label arm. D4's purpose (derived coverage) survives; its letter (a fourth projection) is structurally refused by `hook_manifests_match_disk`.
- D5 at FULL width: on runtime pi, EVERY slot resolution that is not `Resolved::Herding` refuses by name — plain-string `Model`, `Native`, `Cli`, `Budget`, and the escalation path (`Resolved::Inherit`): an escalated cell on pi refuses with the remedy "Pi has no subagent surface — run the escalated cell inline in the session" (store `7f9c8518`).
- D6: models.pi values per the settled table; a matrix row asserts the table VALUES resolve (not only refusals).
- D7 (user-confirmed): mailbox transport OUT → `pi-result-mailbox`; docs carry the not-production-until-mailbox caveat (advisor condition 3).
- D8 (user-chosen) with carrier SETTLED (reviewer P2-2): no new flag, no Rust. `session_start` → `bee hook session-init` once (cached, `/reload`-idempotent); each `before_agent_start` → the EXISTING `bee hook prompt-context` verb unchanged — its output is already the per-prompt delta, and in linked worktrees it already delegates to slim mailbox lines. pis-1 stays TS-only.

## Discovery

- `.opencode/plugins/bee-guard.ts` — sibling belt: candidate-binary chain, one throwing blocking runner, advisory never-throw wrappers, D6 repair semantics; repaired tools (`question`/`task`) are PASS-THROUGH — the reason `updatedInput` lands safely (`:193-201`).
- `hook_manifests.rs:44-52` — named exclusion for TS-plugin belts; `:442-457` panics on a missing generated projection file. A fourth projection cannot exist (Pi has no hook-config surface).
- Dispatch door is gated in THREE places (reviewer Q2): `prepare.rs:29` `DISPATCH_RUNTIMES` (used at `:1038`, `:2061` prepare, `:2542` wave), the hand-maintained `generated/registry_payload.json` enum on `dispatch.prepare` AND `dispatch.wave` (CLI-shape guard refuses before prepare runs — reproduced), and `devtools/mod.rs:533-545` label arms.
- `prepare.rs:1507-1668` — payload match: `Native`/`Cli`/`Herding`/codex-catchall/`_ => Agent`; `Resolved::Model` (plain string) falls to Agent today (P1-2); `prepare.rs:1347-1353` escalation → `Resolved::Inherit` → Agent (P1-3).
- `release_manifest.rs:74-89` `INVENTORY_ROOTS` carries `.opencode/plugins`; a shipped belt file outside a root goes green-while-installing-nothing (`:342-346`); two tests bracket the root set.
- `onboard/`: `copy_opencode_plugin` spans apply.rs, plan.rs:558-564, source.rs:45,61, tests.rs:398-432 — five sites, mirrored for pi.
- Pi repair channel: NO documented `updatedInput` equivalent; handler chaining proves handler-to-handler visibility only (reviewer Q3). Therefore: repaired-verdict tools stay PASS-THROUGH on the Pi belt (no field-name translation for them), mirroring OpenCode — recorded, not assumed.
- Pi extension default export receives no `directory` arg (unlike OpenCode) — cwd from `process.cwd()`/ctx; fixtures must set it.
- Research briefs live untracked in the MAIN checkout only (P2-1) — copied into this worktree at `docs/history/research/` before any cell dispatches.

## Approach

Mirror the OpenCode belt in Pi's idiom, with four hardenings from the wave:

1. **Tool coverage is enumerated, never shape-guessed** (advisor cond. 1): pis-1 enumerates Pi 0.84.3's full built-in tool registry (from the installed binary's docs/source) into an explicit map; the recorded unknown-tool posture is FAIL-SAFE — any tool name outside the map (custom `pi.registerTool` tools from sibling extensions included) routes to write-guard as a write-capable call. A fixture drives an unmapped name.
2. **Passivity is per-call, not load-time** (advisor cond. 2): each handler checks for the `.bee` DIRECTORY (cwd + git-common-dir main root) at call time — an in-session `bee onboard` starts guarding without `/reload`. Directory present + binary missing = BLOCK (D3). Linked-worktree edge (extension in worktree, binary only at main root) gets a fixture.
3. **model-guard is a NAMED EXCLUSION on Pi** (advisor cond. 4, reviewer P1-6): Pi has no Agent/Task surface, so a model-guard row would be a vacuous name-match green. Excluded in the `bee-guard.ts:49-57` style, asserted BY NAME in the parity test.
4. **The dispatch door closes every non-herding exit on pi** (P1-2/P1-3/P2-3): one refusal helper covers `Model`/`Native`/`Cli`/`Budget`/`Inherit` for runtime pi in both `prepare` and `wave`, each refusal naming the slot and the herding requirement.

Rejected alternatives:
- `Runtime::Pi` + fourth generated projection — refused by the catalog's own named-exclusion law and `hook_manifests_match_disk` (P1-1).
- Reverse field-name translation for repair verdicts — no documented Pi input-mutation contract; pass-through mirrors the sibling belt's only safe path (P2-4).
- A new `--slim` prompt-context flag — crosses D8 into Rust and breaks pis-1's TS-only disjointness; the existing verb already emits the per-prompt delta (P2-2).
- Wrapper binary / JSON hooks — as rev 1.

Risk map: TS blocking path / HIGH / stub-binary fixtures (deny, allow, crash, absent, ask-verdict, unparseable, unmapped tool) · dispatch-door non-herding refusals incl. escalation + wave / HIGH / driver unit tests · registry enum + CLI-shape guard / MEDIUM / guard-permits row · release inventory root / MEDIUM / release-manifest --check proof · advisory path / MEDIUM / never-throw fixtures · onboard 5-site copy / LOW / contract test · docs / LOW / pointer checks.

## Shape

One slice (walking skeleton: a Pi session in this repo is guarded and context-fed end to end), 4 cells:

1. `pis-1` (role: code, TS only) — `.pi/extensions/bee-guard.ts`: binary chain; enumerated tool map + fail-safe unknown-tool routing to write-guard; blocking runner (deny/crash/missing-binary/ask/unparseable → `{block, reason}`); repaired-verdict tools pass-through; advisory wrappers (session-init once cached + per-turn `bee hook prompt-context`; `tool_result`→state-sync; `agent_settled`→turn-end); per-call `.bee`-directory passivity; model-guard named-exclusion comment. Cites D1, D2, D3, D8.
2. `pis-2` (role: code, Rust+JSON) — dispatch door: `DISPATCH_RUNTIMES` + `generated/registry_payload.json` (prepare AND wave enums) gain `pi`; `models.rs` `RUNTIMES` widens; non-herding refusal helper wired in prepare + wave (D5 full width, escalation arm included); `devtools/mod.rs` `"pi" => None` label arm; `hook_manifests.rs` named-exclusion comment extends to Pi; onboard `copy_pi_extension` across apply/plan/source/tests; `release_manifest.rs` `.pi/extensions` root + regenerated `docs/history/codex-harness-hardening/release-manifest.json`. Cites D4 (deviation), D5, D6.
3. `pis-3` (role: test, deps: pis-1 ONLY — P3-4) — `tests/pi_plugin_contracts.rs`: fixture suite over the real extension under node with a stub bee binary and a stub `pi` object (deny/allow/crash/absent/ask/unparseable/unmapped-tool/no-.bee-passive/worktree-edge/reload-idempotence rows); extend the parity test to derive the Pi belt from the TS source (keep the test's NAME — P3-1 — two docs cite it), asserting the model-guard exclusion by name. Cites D3, D4.
4. `pis-4` (role: docs, deps: pis-2) — `.bee/config-sample.json` `models.pi` block per D6; `docs/config-reference.md` (P3-3); hook-runtime knowledge area gains the Pi belt row; the NOT-PRODUCTION-until-`pi-result-mailbox` caveat beside every pi-herding mention (advisor cond. 3). Cites D5, D6, D7.

Dependencies: pis-1 ∥ pis-2 (disjoint); pis-3 after pis-1; pis-4 after pis-2.

## Test matrix

High-risk — applicable edge dimensions:
- **Trust boundary**: deny→block; crash→block; `.bee` present + binary missing→block; ask→block; unparseable stdout→block; UNMAPPED tool name→write-guard (fail-safe).
- **Absent/empty**: no `.bee` dir→passive per call (zero registrations' effects, stderr quiet); `.bee` appears mid-session→guarding starts without reload.
- **Contract/compat**: parity test derives Pi rows from the TS source; model-guard exclusion asserted by name; `hook_manifests_match_disk` untouched (no fourth projection); registry contract tests green with the widened enum.
- **Input malformation**: shapeless `tool_call` input → blocking blocks, advisory swallows.
- **Config (D5 full width)**: on pi — plain-string slot refuses; `kind:"cli"` refuses; `kind:"native"` refuses; escalated cell / `--role ceiling` refuses with the inline-session remedy; `dispatch wave --runtime pi` takes the same refusals (P2-3); herding slot returns the herding-exec payload; unconfigured role refuses by name; D6 TABLE VALUES resolve to their agents (P3-2).
- **Registry/guard**: `--runtime pi` passes the CLI-shape guard after the enum widens (P1-4).
- **Release**: `bee dev release-manifest --check` green with the `.pi/extensions` root (P1-5).
- **Idempotence/reload**: double `session_start` → preamble cached once (D8).
- **Cross-platform**: `bee.exe` in the chain; execFile only, never `sh -c`.
- Not applicable, named: data migration; performance beyond D8 token cost; rollout (repo-local file, reversible by deletion).

## Out of scope

- `pi-result-mailbox` (D7): transport, envelope, steer/trigger injection.
- Paseo-side work; `bee herding run` result-contract changes.
- Renaming the parity test; porting config verbs; any fourth generated projection.
