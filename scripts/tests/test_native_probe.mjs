#!/usr/bin/env node
// Covers codex-native-transport cnt-2 (D3a/Δ2-amended — authoritative,
// decisions c0cba64e/760e9b05 — and D4):
// - classifyNativeTransport(evidence): pure classification table.
// - writeNativeTransportProbe / readNativeTransportClassification: the
//   version+config-scoped probe record and its validity legs (g22-3
//   doctor-attest pattern, but a SEPARATE gitignored file — Δ2), config_scope
//   hashing all four verdict-determining flags (Δ2-amended).
// - doctorNativeTransportUnlock: the D4 informational doctor row that only
//   NAMES the unlock, never applies it.
//
// Self-contained, no framework — mirrors scripts/tests/test_lib_mirror.mjs and
// scripts/tests/test_verify_manifest.mjs's style.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");

const {
  classifyNativeTransport,
  NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY,
} = await import(path.join(REPO_ROOT, "packages", "bee", "lib", "dispatch-guard.mjs"));

const {
  writeNativeTransportProbe,
  readNativeTransportClassification,
  nativeTransportConfigScopeHash,
  nativeTransportProbePath,
  doctorCodexVersion,
  doctorCodexFeaturesList,
  doctorNativeTransportUnlock,
} = await import(path.join(REPO_ROOT, "packages", "bee", "bee.mjs"));

let failures = 0;
function check(cond, label, detail) {
  if (cond) {
    console.log(`PASS test_native_probe: ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL test_native_probe: ${label}`);
    if (detail !== undefined) console.error(`      ${JSON.stringify(detail)}`);
  }
}

function mkFixtureRoot() {
  return fs.mkdtempSync(path.join(os.tmpdir(), "test-native-probe-"));
}

// Live evidence (adapts to whether codex is actually installed here) that
// should read back VALID — matches the shape writeNativeTransportProbe
// derives its config_scope from (D3a's two live-observable flags; the other
// two stay null, matching what a real `codex features list` read reports).
function liveEvidence(extra = {}) {
  const liveFlags = doctorCodexFeaturesList();
  const base = liveFlags
    ? {
        multi_agent: liveFlags.multi_agent ? liveFlags.multi_agent.enabled : null,
        multi_agent_v2: liveFlags.multi_agent_v2 ? liveFlags.multi_agent_v2.enabled : null,
      }
    : { multi_agent: null, multi_agent_v2: null };
  return { ...base, ...extra };
}

// ─── classifyNativeTransport(evidence) — D3a pure classification table ────

check(
  classifyNativeTransport(undefined) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "classifyNativeTransport(undefined) -> native_budget_only (unknown/absent evidence stays inert)",
  classifyNativeTransport(undefined),
);
check(
  classifyNativeTransport(null) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "classifyNativeTransport(null) -> native_budget_only",
  classifyNativeTransport(null),
);
check(
  classifyNativeTransport("not-an-object") === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "classifyNativeTransport(malformed non-object) -> native_budget_only",
  classifyNativeTransport("not-an-object"),
);
check(
  classifyNativeTransport({}) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "classifyNativeTransport({}) -> native_budget_only (no signal at all)",
  classifyNativeTransport({}),
);
check(
  classifyNativeTransport({ multi_agent: true }) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "classifyNativeTransport({multi_agent:true}) -> native_budget_only (E3: multi_agent stable, v2/override unproven)",
  classifyNativeTransport({ multi_agent: true }),
);
check(
  classifyNativeTransport({ multi_agent: false }) === NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY,
  "classifyNativeTransport({multi_agent:false}) -> external_cli_only (D3a: the ONLY external trigger)",
  classifyNativeTransport({ multi_agent: false }),
);
check(
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: true }) === NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  "classifyNativeTransport({multi_agent:true, multi_agent_v2:true, override_spawn_accepted:true}) -> native_model_override (D3a: all three conditions)",
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: true }),
);
check(
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: false }) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "multi_agent_v2 true but override NOT accepted -> native_budget_only (all three conditions required)",
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: false }),
);
check(
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: false, override_spawn_accepted: true }) === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
  "override accepted but multi_agent_v2 NOT enabled -> native_budget_only (all three conditions required)",
  classifyNativeTransport({ multi_agent: true, multi_agent_v2: false, override_spawn_accepted: true }),
);
check(
  classifyNativeTransport({ multi_agent: false, multi_agent_v2: true, override_spawn_accepted: true }) === NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY,
  "multi_agent:false wins even if v2+override look positive (D3a: external_cli_only is the ONLY multi_agent:false outcome — evidence, not inference)",
  classifyNativeTransport({ multi_agent: false, multi_agent_v2: true, override_spawn_accepted: true }),
);
check(
  classifyNativeTransport({ multi_agent_v2: true, override_spawn_accepted: true }) === NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
  "multi_agent absent (not === false) + v2 + override accepted -> native_model_override (D3a: multi_agent !== false, not 'multi_agent === true')",
  classifyNativeTransport({ multi_agent_v2: true, override_spawn_accepted: true }),
);

// ─── readNativeTransportClassification — invalid/absent legs ──────────────

{
  const root = mkFixtureRoot();
  const result = readNativeTransportClassification(root);
  check(
    result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY && result.valid === false && result.reason === "no_probe_record",
    "no probe record on disk -> native_budget_only, reason no_probe_record",
    result,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  const writeRoot = mkFixtureRoot();
  const readRoot = mkFixtureRoot();
  writeNativeTransportProbe(writeRoot, { codexVersion: "codex-cli 0.144.4", evidence: { multi_agent: true } });
  // Copy the written record verbatim into a DIFFERENT root's .bee/ dir so
  // only repo_identity differs between write-time and read-time.
  fs.mkdirSync(path.join(readRoot, ".bee"), { recursive: true });
  fs.copyFileSync(nativeTransportProbePath(writeRoot), nativeTransportProbePath(readRoot));
  const result = readNativeTransportClassification(readRoot);
  check(
    result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY && result.valid === false && result.reason === "identity_changed",
    "a probe record copied into a different repo checkout invalidates -> reason identity_changed",
    result,
  );
  fs.rmSync(writeRoot, { recursive: true, force: true });
  fs.rmSync(readRoot, { recursive: true, force: true });
}

{
  const root = mkFixtureRoot();
  writeNativeTransportProbe(root, { codexVersion: "codex-cli 0.0.0-deliberately-wrong", evidence: { multi_agent: true } });
  const result = readNativeTransportClassification(root);
  check(
    result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY && result.valid === false && result.reason === "version_changed",
    "a recorded codex_version that no longer matches the live binary invalidates -> reason version_changed",
    result,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  // Reach leg 3 (config-scope integrity) by matching leg 1 (same root) and
  // leg 2 (the REAL live codex_version) exactly, then hand-corrupt the
  // stored hash so it no longer matches the recorded scope.
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  writeNativeTransportProbe(root, { codexVersion: liveVersion, evidence: liveEvidence() });
  const recordPath = nativeTransportProbePath(root);
  const record = JSON.parse(fs.readFileSync(recordPath, "utf8"));
  record.config_scope_hash = "0000000000000000000000000000000000000000000000000000000000000";
  fs.writeFileSync(recordPath, JSON.stringify(record, null, 2), "utf8");
  const result = readNativeTransportClassification(root);
  check(
    result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY && result.valid === false && result.reason === "config_scope_corrupt",
    "a config_scope_hash that no longer matches the recorded config_scope invalidates -> reason config_scope_corrupt",
    result,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  // Internally-consistent (hash matches scope) but the scope disagrees with
  // the LIVE codex-observable subset (multi_agent / multi_agent_v2) — only
  // exercised when `codex features list` is actually resolvable here.
  const liveFlags = doctorCodexFeaturesList();
  if (liveFlags && liveFlags.multi_agent) {
    const root = mkFixtureRoot();
    const liveVersion = doctorCodexVersion().value;
    writeNativeTransportProbe(root, {
      codexVersion: liveVersion,
      evidence: { multi_agent: !liveFlags.multi_agent.enabled },
    });
    const result = readNativeTransportClassification(root);
    check(
      result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY && result.valid === false && result.reason === "flag_state_changed",
      "a hash-consistent config_scope that disagrees with the LIVE codex features list invalidates -> reason flag_state_changed",
      result,
    );
    fs.rmSync(root, { recursive: true, force: true });
  } else {
    console.log("SKIP test_native_probe: flag_state_changed leg — `codex features list` unresolvable in this environment");
  }
}

{
  // Happy path: every leg matches live state (adapts to whether codex is
  // actually installed here) -> valid, classification round-trips exactly
  // what the writer computed from evidence.
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  const written = writeNativeTransportProbe(root, { codexVersion: liveVersion, evidence: liveEvidence({ multi_agent: true }) });
  const result = readNativeTransportClassification(root);
  check(
    result.valid === true && result.reason === null && result.classification === written.classification,
    "every validity leg matching live state -> valid:true, classification round-trips the writer's verdict",
    { written, result },
  );
  check(
    result.classification === NATIVE_TRANSPORT_NATIVE_BUDGET_ONLY,
    "the round-tripped classification for {multi_agent:true, no v2/override} is native_budget_only (D3a default, matches E3 on 0.144.4)",
    result.classification,
  );
  check(
    written.config_scope_hash === nativeTransportConfigScopeHash(written.config_scope) &&
      Object.keys(written.config_scope).sort().join(",") === "hide_spawn_agent_metadata,multi_agent,multi_agent_v2,tool_namespace",
    "config_scope carries all four verdict-determining flags (Δ2-amended), hash matches the stored scope",
    written,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  // Writer classification for the other two outcomes. Checked against the
  // RAW written record (not the validated reader): forcing multi_agent_v2/
  // override_spawn_accepted to true on a host where they are genuinely
  // false (this build: E3, multi_agent_v2 under-development/false) is by
  // construction evidence that disagrees with live reality — the reader
  // CORRECTLY invalidates that (flag_state_changed, already covered by its
  // own dedicated test above); that is the safety behavior working, not a
  // bug. What's under test here is only the writer's evidence->
  // classification mapping, independent of live-validity.
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  const written = writeNativeTransportProbe(root, {
    codexVersion: liveVersion,
    evidence: liveEvidence({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: true }),
  });
  check(
    written.classification === NATIVE_TRANSPORT_NATIVE_MODEL_OVERRIDE,
    "writer: multi_agent+multi_agent_v2+override_spawn_accepted evidence classifies as native_model_override",
    written,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  const written = writeNativeTransportProbe(root, { codexVersion: liveVersion, evidence: liveEvidence({ multi_agent: false }) });
  check(
    written.classification === NATIVE_TRANSPORT_EXTERNAL_CLI_ONLY,
    "writer: multi_agent:false evidence classifies as external_cli_only",
    written,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

check(
  nativeTransportConfigScopeHash({ a: 1, b: 2 }) === nativeTransportConfigScopeHash({ b: 2, a: 1 }),
  "nativeTransportConfigScopeHash is key-order independent (stable hash for a stable key set)",
);
check(
  nativeTransportConfigScopeHash({ a: 1 }) !== nativeTransportConfigScopeHash({ a: 2 }),
  "nativeTransportConfigScopeHash changes when a value changes",
);

// ─── doctorNativeTransportUnlock — D4: names the unlock, never applies it ──

{
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  writeNativeTransportProbe(root, { codexVersion: liveVersion, evidence: liveEvidence({ multi_agent: true }) });
  const row = doctorNativeTransportUnlock(root, { multi_agent_v2: { maturity: "under development", enabled: false } });
  check(
    row.row === "native_transport_unlock" && row.status === "ok",
    "doctorNativeTransportUnlock returns a well-formed row (native_budget_only + flag shipped)",
    row,
  );
  check(
    typeof row.evidence === "string" && row.evidence.includes("multi_agent_v2") && row.evidence.includes("hide_spawn_agent_metadata"),
    "the unlock row NAMES both flags (features.multi_agent_v2, hide_spawn_agent_metadata) when the build ships the flag",
    row.evidence,
  );
  check(
    !("blocking" in row) && !("degrades" in row),
    "the unlock row is never blocking and never degrading (D4: informational only, never load-bearing)",
    row,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  const root = mkFixtureRoot();
  const liveVersion = doctorCodexVersion().value;
  writeNativeTransportProbe(root, { codexVersion: liveVersion, evidence: liveEvidence({ multi_agent: true }) });
  const row = doctorNativeTransportUnlock(root, null);
  check(
    row.status === "ok" && row.evidence.includes("does not ship the multi_agent_v2 feature flag"),
    "when the codex build does not ship the flag at all, the row says there is nothing to unlock",
    row.evidence,
  );
  fs.rmSync(root, { recursive: true, force: true });
}

{
  // doctorNativeTransportUnlock calls readNativeTransportClassification
  // internally, so a VALID native_model_override reading here needs the
  // live `codex features list` to genuinely report multi_agent_v2 enabled
  // (this build, E3, reports it under-development/false) — only exercised
  // when the live host can actually produce that combination; skipped
  // otherwise rather than faking a live-vs-recorded mismatch this row's
  // own reader would (correctly) reject.
  const liveFlags = doctorCodexFeaturesList();
  if (liveFlags && liveFlags.multi_agent_v2 && liveFlags.multi_agent_v2.enabled) {
    const root = mkFixtureRoot();
    const liveVersion = doctorCodexVersion().value;
    writeNativeTransportProbe(root, {
      codexVersion: liveVersion,
      evidence: liveEvidence({ multi_agent: true, multi_agent_v2: true, override_spawn_accepted: true }),
    });
    const row = doctorNativeTransportUnlock(root, { multi_agent_v2: { maturity: "stable", enabled: true } });
    check(
      row.status === "ok" && row.evidence.includes("no unlock needed"),
      "when classification is already native_model_override, the row says no unlock is needed",
      row.evidence,
    );
    fs.rmSync(root, { recursive: true, force: true });
  } else {
    console.log("SKIP test_native_probe: doctorNativeTransportUnlock 'no unlock needed' branch — live multi_agent_v2 is not enabled on this build");
  }
}

// ─── mirror sanity: this repo's real .bee/bin/bee.mjs and .bee/bin/lib/
// dispatch-guard.mjs must expose the exact same exports (test_lib_mirror
// already proves byte-identity for dispatch-guard.mjs; this proves bee.mjs's
// runtime copy — not covered by test_lib_mirror since it only compares
// packages/bee/lib <-> .bee/bin/lib — carries the same native-transport exports
// as the template it was copied from).
{
  const runtimeBee = await import(path.join(REPO_ROOT, ".bee", "bin", "bee.mjs"));
  check(
    typeof runtimeBee.readNativeTransportClassification === "function" &&
      typeof runtimeBee.writeNativeTransportProbe === "function" &&
      typeof runtimeBee.doctorNativeTransportUnlock === "function",
    ".bee/bin/bee.mjs (the runtime copy) exports the same native-transport-probe functions as packages/bee/bee.mjs",
  );
}

if (failures > 0) {
  console.error(`FAIL test_native_probe: ${failures} check(s) failed`);
  process.exit(1);
}
console.log("PASS test_native_probe: all checks green");
