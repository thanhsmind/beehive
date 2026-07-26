#!/usr/bin/env node
// canary_codex.mjs — real-codex canary for the installed bee hook chain
// (GH #22 P1-7, decision D9, advisor R7).
//
// Proves, against a *live* codex-cli invocation plus the *installed* hook
// files (not the source `hooks/` tree), that:
//   P1 — SessionStart fires bee-session-init.mjs                (real codex exec)
//   P2 — an unmarked Codex spawn_agent is denied                (synthetic, through the installed hook)
//   P3 — a [bee-tier: ...]-marked spawn is allowed               (synthetic, through the installed hook)
//   P4 — update_plan reaches bee-state-sync.mjs                  (synthetic, through the installed hook)
//   P5 — a pre-Gate-3 source write is blocked                    (synthetic, through the installed hook)
//
// Why P2-P5 are synthetic-through-installed-hook rather than driving a real
// model turn: getting codex's own model to (a) call spawn_agent with an
// exact marker/no-marker message, (b) call update_plan, and (c) attempt a
// source write, on demand and deterministically, is not controllable from
// the outside — the model decides whether/when to call a tool. Piping a
// synthetic PreToolUse/PostToolUse envelope on stdin into the EXACT hook
// file codex would invoke (`.bee/bin/hooks/<name>.mjs --source=repo`, read
// from the fixture's *installed* .codex/hooks.json, cwd = fixture root)
// proves the installed chain's allow/deny decision without needing a live
// model call to cooperate. P1 is a real `codex exec` run because the
// SessionStart observable (a session record written to disk) needs no tool
// call from the model at all — it fires at session bootstrap.
//
// Fixture install path (cheaper-reliable-path decision): this canary calls
// packages/bee/scripts/onboard_bee.mjs directly (--apply --repo-hooks)
// rather than scripts/install.sh. install.sh's repo-copy mode also probes
// and (for a real `codex`/`claude` on PATH) *mutates* the runtime's plugin
// list (`codex plugin remove bee@bee`) as part of its transition step —
// exactly the kind of touch this canary must never risk against a real
// developer machine's ~/.codex. onboard_bee.mjs alone reproduces the exact
// artifact set a repo-copy install leaves behind (.codex/hooks.json,
// .bee/bin/hooks/*, .bee/bin/lib/*, a fresh .bee store) with zero plugin
// CLI calls of any kind.
//
// Isolation (advisor R7 — BINDING): every codex invocation below runs with
// a per-run `mktemp -d` CODEX_HOME (env override, never the real one) and
// a per-run temp fixture repo (unique per run — concurrent-safe). Both are
// removed in a `finally`, whether the probes pass or fail.
//
// A subtlety proven live while building this canary and worth pinning here:
// `--dangerously-bypass-hook-trust` bypasses per-HOOK review trust only. It
// does NOT bypass per-PROJECT trust (the `[projects."<path>"] trust_level =
// "trusted"` entries codex's own config.toml tracks) — a project codex has
// never trusted still runs the requested turn, but the installed hook chain
// never fires and leaves no crash log (a silent, fail-safe no-op, not an
// error). This canary's throwaway CODEX_HOME therefore always seeds a
// `config.toml` trusting the fixture path before invoking codex, and always
// passes `-C <fixture>` explicitly (never relies on process cwd) so the
// trusted path and the path codex actually treats as its project are the
// exact same string.
//
// Skip guard: no `codex` binary on PATH -> print the message below and
// exit 0 (this is what keeps the nightly workflow green on a runner with no
// codex install).
//
// --probe-selftest (codex-native-transport cnt-5, plan.md R4): an OFFLINE
// invariant check for the native-override probe leg below (P6/P7) — never
// invokes the `codex` binary at all, so it is meaningful evidence even on a
// runner with no codex install (unlike P6/P7 themselves, which need a real
// codex-cli + live turn and only run when codexOnPath()). It proves the one
// safety invariant D4 depends on: writing the multi_agent_v2 /
// hide_spawn_agent_metadata unlock flags touches ONLY a per-run isolated
// CODEX_HOME's config.toml, never the real one (whatever CODEX_HOME/~/.codex
// already holds on this machine is read before and after and must be
// byte-identical).

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..");
const ONBOARD_SCRIPT = path.join(REPO_ROOT, "packages", "bee", "scripts", "onboard_bee.mjs");
const BEE_CLI = path.join(REPO_ROOT, ".bee", "bin", "bee.mjs");

// The exact D3/D4 unlock TOML (PROBE EVIDENCE FOLD, decision daa01646): the
// nested `[features.multi_agent_v2]` table carrying an EXPLICIT
// `enabled = true` alongside `hide_spawn_agent_metadata = false` (variant
// v2d, .bee/spikes/codex-native-transport/probe-v1v3.md — the nested table
// WITHOUT `enabled` does NOT flip the flag; a table without `enabled` alone
// is not enough). NOTE (deviation, cnt-5): an earlier draft of this
// function ALSO wrote a flat `multi_agent_v2 = true` scalar directly under
// `[features]` alongside this nested table — invalid TOML, since the same
// key path cannot be both a boolean value and a table (confirmed live:
// `codex debug models` failed with "duplicate key" against the resulting
// config.toml). v2d alone already flips multi_agent_v2 to true (V2 spike
// evidence table, variant v2d) so the flat line was redundant as well as
// broken; removed.
function nativeTransportUnlockToml() {
  return (
    "[features]\n" +
    "suppress_unstable_features_warning = true\n" +
    "\n" +
    "[features.multi_agent_v2]\n" +
    "enabled = true\n" +
    "hide_spawn_agent_metadata = false\n"
  );
}

// Best-effort read of the real (non-isolated) CODEX_HOME's config.toml, so a
// caller can snapshot it before writing anything and assert byte-identity
// after — the D4 non-mutation proof. Never throws; a missing file reads as
// `null` (which is itself a valid "no file existed" snapshot to compare
// against).
function readRealCodexConfigToml() {
  const realCodexHome = process.env.CODEX_HOME || path.join(os.homedir(), ".codex");
  const realConfigPath = path.join(realCodexHome, "config.toml");
  try {
    return { path: realConfigPath, content: fs.readFileSync(realConfigPath, "utf8") };
  } catch {
    return { path: realConfigPath, content: null };
  }
}

// Runs entirely offline (no `codex` invocation anywhere in this function —
// that is the property under test, not just an assertion about it): writes
// the D3/D4 unlock TOML into a throwaway `mktemp` CODEX_HOME, confirms it
// landed there, and confirms the real CODEX_HOME's config.toml (snapshotted
// before the write) is untouched. Returns { ok, detail }.
function runProbeSelftest() {
  const before = readRealCodexConfigToml();
  const isolatedHome = mkTempDir("bee-canary-selftest-codex-home-");
  let ok = true;
  const notes = [];
  try {
    const isolatedConfigPath = path.join(isolatedHome, "config.toml");
    fs.writeFileSync(isolatedConfigPath, nativeTransportUnlockToml());

    const isolatedContent = fs.readFileSync(isolatedConfigPath, "utf8");
    const wroteFlags =
      isolatedContent.includes("[features.multi_agent_v2]") &&
      isolatedContent.includes("hide_spawn_agent_metadata = false") &&
      isolatedContent.includes("enabled = true");
    notes.push(
      wroteFlags
        ? `isolated CODEX_HOME (${isolatedHome}) config.toml carries the unlock flags`
        : `FAIL: isolated CODEX_HOME config.toml is missing the expected unlock flags`,
    );
    ok = ok && wroteFlags;

    // Regression guard for the duplicate-key bug found live while building
    // this probe leg (deviation note above nativeTransportUnlockToml): a
    // flat `multi_agent_v2 = true` scalar directly under `[features]`
    // alongside the `[features.multi_agent_v2]` table is invalid TOML —
    // string-matching alone missed it (the earlier version of this check
    // passed on that broken output). `codex debug models` parses config.toml
    // with a real TOML parser, so a genuinely-parseable single-table-per-key
    // document is asserted here directly rather than trusted from substrings.
    const noDuplicateKeyShape = !/^\s*multi_agent_v2\s*=\s*true\s*$/m.test(isolatedContent);
    notes.push(
      noDuplicateKeyShape
        ? "no flat multi_agent_v2 scalar under [features] (would duplicate-key against the nested table)"
        : "FAIL: flat multi_agent_v2 scalar present alongside [features.multi_agent_v2] table — invalid TOML",
    );
    ok = ok && noDuplicateKeyShape;

    const after = readRealCodexConfigToml();
    const realUnchanged = before.path === after.path && before.content === after.content;
    notes.push(
      realUnchanged
        ? `real CODEX_HOME config.toml (${after.path}) is byte-identical before/after (${
            after.content === null ? "still absent" : "still present, unchanged"
          })`
        : `FAIL: real CODEX_HOME config.toml (${after.path}) CHANGED — D4 invariant violated`,
    );
    ok = ok && realUnchanged;

    const neitherPathIsReal = isolatedConfigPath !== after.path;
    notes.push(
      neitherPathIsReal
        ? "isolated config path and real config path are distinct filesystem locations"
        : "FAIL: isolated path resolved to the real CODEX_HOME path",
    );
    ok = ok && neitherPathIsReal;
  } finally {
    fs.rmSync(isolatedHome, { recursive: true, force: true });
  }
  return { ok, detail: notes.join("; ") };
}

// --probe (codex-native-transport cnt-5, plan.md V1/V3 + advisor R4): the
// REAL native-override probe leg, distinct from --probe-selftest above and
// from the P1-P5 canary suite below — a separate mode because it forces the
// under-development multi_agent_v2 feature and runs a live model turn
// (cost/flakiness unsuited to riding along on every default canary run).
// Builds its fixture the SAME proven way as buildFixture() below
// (onboard_bee.mjs --apply --repo-hooks, the full onboarded hook chain) —
// the .bee/spikes/codex-native-transport/probe.mjs hand-rolled single-hook
// approach never observed a fired PreToolUse hook despite an accepted spawn
// (probe-v1v3.md V3); this reuses the method that DID observe it
// (codex-native-runtime-v2 capability-matrix row D1). Every codex invocation
// runs against a per-run isolated CODEX_HOME (D4) that is torn down in a
// `finally`, and the real CODEX_HOME's config.toml is snapshotted before and
// asserted byte-identical after — the same non-mutation proof as
// runProbeSelftest(), just against the real client instead of a throwaway.
const NATIVE_PROBE_TIMEOUT_MS = 120_000;

function seedNativeProbeCodexHome(codexHome, fixtureRealPath) {
  const realCodexHome = process.env.CODEX_HOME || path.join(os.homedir(), ".codex");
  const realAuth = path.join(realCodexHome, "auth.json");
  try {
    if (fs.existsSync(realAuth)) {
      fs.copyFileSync(realAuth, path.join(codexHome, "auth.json"));
    }
  } catch {
    // best-effort only, same as seedCodexHome() below.
  }
  const config = `[projects."${fixtureRealPath}"]\ntrust_level = "trusted"\n\n` + nativeTransportUnlockToml();
  fs.writeFileSync(path.join(codexHome, "config.toml"), config);
}

function codexModelCatalog(codexHome) {
  const r = run(process.platform === "win32" ? "codex.cmd" : "codex", ["debug", "models"], {
    env: { ...process.env, CODEX_HOME: codexHome },
    timeout: 20_000,
  });
  let modelIds = [];
  try {
    const parsed = JSON.parse(r.stdout);
    const arr = Array.isArray(parsed) ? parsed : parsed.models || parsed.data || [];
    modelIds = arr.map((m) => (typeof m === "string" ? m : m.id || m.slug || m.model)).filter(Boolean);
  } catch {
    modelIds = [...new Set([...(r.stdout || "").matchAll(/"(?:id|slug)"\s*:\s*"([^"]+)"/g)].map((m) => m[1]))];
  }
  return { status: r.status, modelIds };
}

function codexLiveFeaturesList(codexHome) {
  const r = run("codex", ["features", "list"], { env: { ...process.env, CODEX_HOME: codexHome }, timeout: 20_000 });
  const flags = {};
  for (const rawLine of (r.stdout || "").split("\n")) {
    const line = rawLine.trimEnd();
    const match = /^(\S+)\s+(.+?)\s+(true|false)\s*$/.exec(line);
    if (match) flags[match[1]] = { maturity: match[2].trim(), enabled: match[3] === "true" };
  }
  return flags;
}

// Prepends a capture hook into the ONBOARDED fixture's own
// `.codex/hooks.json` PreToolUse `spawn_agent` matcher group — ahead of the
// real installed bee-model-guard.mjs entry, never replacing it. This proves
// the exact installed chain still runs (V3's method requirement) while
// independently observing the tool_input envelope regardless of the guard's
// own allow/deny verdict. The capture script always returns `{}`/exit 0, so
// it can never itself deny a spawn.
function injectSpawnAgentCaptureHook(fixtureRoot, captureLogPath, markerPath) {
  const hooksPath = path.join(fixtureRoot, ".codex", "hooks.json");
  const hooksJson = JSON.parse(fs.readFileSync(hooksPath, "utf8"));
  const spawnGroup = (hooksJson.hooks?.PreToolUse || []).find((g) => g.matcher === "spawn_agent");
  if (!spawnGroup) {
    throw new Error("onboarded .codex/hooks.json has no PreToolUse spawn_agent matcher group — cannot observe V3");
  }
  const captureScriptPath = path.join(fixtureRoot, ".bee-canary-v3-capture.mjs");
  fs.writeFileSync(
    captureScriptPath,
    [
      "#!/usr/bin/env node",
      "import fs from 'node:fs';",
      `const LOG = ${JSON.stringify(captureLogPath)};`,
      `const MARK = ${JSON.stringify(markerPath)};`,
      "try { fs.writeFileSync(MARK, 'invoked ' + new Date().toISOString()); } catch {}",
      "let data = '';",
      "process.stdin.on('data', (c) => (data += c));",
      "process.stdin.on('end', () => {",
      "  try { fs.appendFileSync(LOG, data.trim() + '\\n'); } catch {}",
      "  process.stdout.write('{}');",
      "  process.exit(0);",
      "});",
    ].join("\n"),
  );
  spawnGroup.hooks.unshift({ type: "command", command: `node ${captureScriptPath}` });
  fs.writeFileSync(hooksPath, JSON.stringify(hooksJson, null, 2));
}

// Runs the live turn and returns raw, unjudged observations. Never throws
// for a negative V1/V3 answer (CONTEXT D3: either answer is a valid green) —
// only an infrastructure failure (fixture/codex-home build) throws, and a
// D4 isolation breach is reported via `isolation_ok: false` for the caller
// to fail loudly on.
function runNativeTransportProbeLeg() {
  const before = readRealCodexConfigToml();
  const fixtureRoot = buildFixture();
  const codexHome = mkTempDir("bee-canary-native-probe-codex-home-");
  const captureLogPath = path.join(codexHome, "v3-envelope.jsonl");
  const markerPath = path.join(codexHome, "v3-hook-invoked.marker");
  const result = {
    codex_version: null,
    features_list: null,
    override_model: null,
    live_exec_status: null,
    live_exec_stderr_trim: null,
    turn_error_message: null,
    hook_invoked: false,
    envelope_count: 0,
    override_fields_observed: false,
    override_spawn_accepted: null,
    raw_envelopes: [],
    isolation_ok: null,
    error: null,
  };
  try {
    seedNativeProbeCodexHome(codexHome, fixtureRoot);
    injectSpawnAgentCaptureHook(fixtureRoot, captureLogPath, markerPath);

    const versionProbe = run("codex", ["--version"]);
    result.codex_version = (versionProbe.stdout || "").trim() || null;

    const catalog = codexModelCatalog(codexHome);
    const overrideModel = catalog.modelIds.find((m) => /gpt-5/i.test(m)) || catalog.modelIds[0] || null;
    result.override_model = overrideModel;

    if (!overrideModel) {
      result.error = "no catalog model id resolved from `codex debug models` — override spawn not attempted";
      return result;
    }

    const prompt = [
      "This is a diagnostic test of the collaboration tool schema, not a real task.",
      "Call the spawn_agent tool right now with these exact arguments, including every field",
      "even if you believe the schema does not declare it:",
      `agent_type="worker", task_name="cnt5_v3_probe", fork_turns="none", model="${overrideModel}",`,
      'reasoning_effort="low", message="[bee-tier: generation] reply with the single word OK and stop".',
      "Call the tool immediately in your first turn, do not ask for confirmation, do not explain first.",
      "After the call, report in one sentence exactly what codex's own tool response or error said,",
      "verbatim, not your interpretation.",
    ].join(" ");

    const live = run(
      "codex",
      ["exec", "-C", fixtureRoot, "--json", "--ephemeral", "--dangerously-bypass-hook-trust", prompt],
      {
        cwd: fixtureRoot,
        env: { ...process.env, CODEX_HOME: codexHome },
        timeout: NATIVE_PROBE_TIMEOUT_MS,
        input: "",
      },
    );
    result.live_exec_status = live.status;
    result.live_exec_stderr_trim = String(live.stderr || "").slice(0, 800);
    // The `--json` event stream (not stderr) is where a rejected turn
    // reports itself — extract the last `turn.failed`/`error` item's message
    // verbatim rather than guessing from exit status alone (observed live:
    // an API-level 400 on `tools` shows up here, exit status 1, empty
    // stderr beyond the harmless bypass/under-development warnings).
    const errorLine = String(live.stdout || "")
      .trim()
      .split("\n")
      .reverse()
      .find((l) => l.includes('"type":"turn.failed"') || l.includes('"type":"error"'));
    if (errorLine) {
      try {
        const parsed = JSON.parse(errorLine);
        result.turn_error_message = (parsed.error && parsed.error.message) || parsed.message || errorLine;
      } catch {
        result.turn_error_message = errorLine;
      }
    }
    result.override_spawn_accepted =
      live.status === 0 &&
      !result.turn_error_message &&
      !/error=|failed to parse function arguments|unknown field/i.test(live.stderr || "");

    result.hook_invoked = fs.existsSync(markerPath);
    if (fs.existsSync(captureLogPath)) {
      const lines = fs.readFileSync(captureLogPath, "utf8").trim().split("\n").filter(Boolean);
      result.envelope_count = lines.length;
      result.raw_envelopes = lines.map((l) => {
        try {
          return JSON.parse(l);
        } catch {
          return { raw: l };
        }
      });
      result.override_fields_observed = result.raw_envelopes.some(
        (e) =>
          e?.tool_input &&
          ("model" in e.tool_input || "reasoning_effort" in e.tool_input || "fork_turns" in e.tool_input),
      );
    }

    result.features_list = codexLiveFeaturesList(codexHome);
  } finally {
    const after = readRealCodexConfigToml();
    result.isolation_ok = before.path === after.path && before.content === after.content;
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
    fs.rmSync(codexHome, { recursive: true, force: true });
  }
  return result;
}

// Runs the probe leg, prints the raw observation, and persists the machine
// record via writeNativeTransportProbe (bee.mjs, cnt-2). Exit code reflects
// infrastructure health only — the V1/V3 observation itself never fails the
// mode (D3: either answer is a valid green); only a D4 isolation breach or a
// hard crash returns non-zero.
async function runProbeMode() {
  let leg;
  try {
    leg = runNativeTransportProbeLeg();
  } catch (e) {
    console.error(`canary --probe: FAIL — infra error: ${e && e.message ? e.message : e}`);
    return 1;
  }

  if (leg.isolation_ok === false) {
    console.error(
      "canary --probe: FAIL — D4 invariant violated: real CODEX_HOME config.toml changed during the probe",
    );
    return 1;
  }

  console.log(`codex_version: ${leg.codex_version}`);
  console.log(`multi_agent: ${leg.features_list ? leg.features_list.multi_agent?.enabled : null}`);
  console.log(`multi_agent_v2: ${leg.features_list ? leg.features_list.multi_agent_v2?.enabled : null}`);
  console.log(`override_model: ${leg.override_model}`);
  console.log(`live_exec_status: ${leg.live_exec_status}`);
  if (leg.turn_error_message) console.log(`turn_error_message: ${leg.turn_error_message}`);
  console.log(`override_spawn_accepted: ${leg.override_spawn_accepted}`);
  console.log(`hook_invoked (installed onboarded chain): ${leg.hook_invoked}`);
  console.log(`envelope_count: ${leg.envelope_count}`);
  console.log(`override_fields_observed in PreToolUse tool_input: ${leg.override_fields_observed}`);
  if (leg.error) console.log(`note: ${leg.error}`);

  const flags = leg.features_list || {};
  const evidence = {
    multi_agent: flags.multi_agent ? flags.multi_agent.enabled : null,
    multi_agent_v2: flags.multi_agent_v2 ? flags.multi_agent_v2.enabled : null,
    hide_spawn_agent_metadata: false,
    tool_namespace: null,
    override_spawn_accepted: leg.override_spawn_accepted,
    turn_error_message: leg.turn_error_message,
    v3_hook_invoked: leg.hook_invoked,
    v3_override_fields_observed: leg.override_fields_observed,
    v3_envelope_count: leg.envelope_count,
  };
  const { writeNativeTransportProbe } = await import(pathToFileURL(BEE_CLI).href);
  const record = writeNativeTransportProbe(REPO_ROOT, { codexVersion: leg.codex_version, evidence });
  console.log(`classification: ${record.classification}`);
  console.log("machine record written: .bee/native-transport-probe.json");

  return 0;
}

// codex exec's own auth-retry loop (observed live: ~5 reconnect attempts
// against the Responses API before giving up) can run ~15-20s even when the
// turn ultimately fails — P1's observable (the session record) is written
// well before that, at session bootstrap, but the timeout still has to
// outlast a slow/CI runner.
const CODEX_TIMEOUT_MS = 60_000;
const HOOK_TIMEOUT_MS = 15_000;

function codexOnPath() {
  const probe = spawnSync("codex", ["--version"], { stdio: "ignore" });
  return !(probe.error && probe.error.code === "ENOENT");
}

function run(cmd, args, opts = {}) {
  return spawnSync(cmd, args, { encoding: "utf8", ...opts });
}

function mkTempDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

// Build one throwaway fixture repo: `git init`, then the repo-copy hook
// projection via onboard_bee.mjs --apply --repo-hooks (see file header for
// why this, not install.sh). Returns the fixture's realpath (codex resolves
// project trust against the canonical path, not a possibly-symlinked one).
function buildFixture() {
  const fixture = mkTempDir("bee-canary-fixture-");
  const init = run("git", ["init", "--quiet"], { cwd: fixture });
  if (init.status !== 0) {
    throw new Error(`git init failed in fixture: ${init.stderr || init.stdout}`);
  }
  run("git", ["config", "user.email", "canary@bee.local"], { cwd: fixture });
  run("git", ["config", "user.name", "bee canary"], { cwd: fixture });

  const onboard = run(
    process.execPath,
    [ONBOARD_SCRIPT, "--repo-root", fixture, "--apply", "--repo-hooks", "--runtime", "codex", "--no-claude-md"],
    { cwd: REPO_ROOT },
  );
  if (onboard.status !== 0) {
    throw new Error(
      `onboard_bee.mjs --apply --repo-hooks failed (exit ${onboard.status}):\n${onboard.stdout}\n${onboard.stderr}`,
    );
  }
  if (!fs.existsSync(path.join(fixture, ".codex", "hooks.json"))) {
    throw new Error("onboard_bee.mjs reported success but .codex/hooks.json is missing from the fixture");
  }
  return fs.realpathSync(fixture);
}

// Seed a throwaway CODEX_HOME: best-effort real auth (so P1 can complete a
// real turn when this machine has codex credentials — never required, see
// header note: the SessionStart observable fires even on an auth failure)
// plus mandatory project trust for the fixture path.
function seedCodexHome(codexHome, fixtureRealPath) {
  const realCodexHome = process.env.CODEX_HOME || path.join(os.homedir(), ".codex");
  const realAuth = path.join(realCodexHome, "auth.json");
  try {
    if (fs.existsSync(realAuth)) {
      fs.copyFileSync(realAuth, path.join(codexHome, "auth.json"));
    }
  } catch {
    // best-effort only: a missing/unreadable auth.json just means P1's turn
    // will fail on auth — the SessionStart observable is unaffected.
  }
  const config = `[projects."${fixtureRealPath}"]\ntrust_level = "trusted"\n`;
  fs.writeFileSync(path.join(codexHome, "config.toml"), config);
}

// Invoke the fixture's OWN installed hook file (not this repo's hooks/
// source tree) exactly as .codex/hooks.json's rendered command does:
// `node <fixture>/.bee/bin/hooks/<name>.mjs --source=repo`, payload on
// stdin, cwd = fixture root.
function invokeInstalledHook(fixtureRoot, hookFile, payload) {
  const hookPath = path.join(fixtureRoot, ".bee", "bin", "hooks", hookFile);
  return run(process.execPath, [hookPath, "--source=repo"], {
    cwd: fixtureRoot,
    input: JSON.stringify(payload),
    timeout: HOOK_TIMEOUT_MS,
  });
}

const results = [];
function record(name, ok, detail) {
  results.push({ name, ok });
  console.log(`${ok ? "PASS" : "FAIL"} ${name}: ${detail}`);
}

async function runProbes() {
  const fixtureRoot = buildFixture();
  const codexHome = mkTempDir("bee-canary-codex-home-");
  try {
    seedCodexHome(codexHome, fixtureRoot);
    const codexEnv = { ...process.env, CODEX_HOME: codexHome };

    // ---- P1 (real codex exec): SessionStart fires bee-session-init.mjs ----
    const sessionsDir = path.join(fixtureRoot, ".bee", "sessions");
    const before = fs.existsSync(sessionsDir) ? new Set(fs.readdirSync(sessionsDir)) : new Set();
    const p1 = run(
      "codex",
      ["exec", "-C", fixtureRoot, "--json", "--dangerously-bypass-hook-trust", "reply with the single word: canary"],
      { cwd: fixtureRoot, env: codexEnv, timeout: CODEX_TIMEOUT_MS },
    );
    const after = fs.existsSync(sessionsDir) ? fs.readdirSync(sessionsDir) : [];
    const newSessions = after.filter((f) => !before.has(f));
    record(
      "P1 SessionStart hook fired [real codex exec]",
      newSessions.length > 0,
      newSessions.length > 0
        ? `.bee/sessions/${newSessions[0]} created by the installed bee-session-init.mjs (codex exec exit ${p1.status})`
        : `no new .bee/sessions/*.json after codex exec (exit ${p1.status}); stderr: ${String(p1.stderr || "").slice(0, 300)}`,
    );

    // ---- P2 (synthetic-through-installed-hook): unmarked spawn_agent denied ----
    const p2 = invokeInstalledHook(fixtureRoot, "bee-model-guard.mjs", {
      hook_event_name: "PreToolUse",
      cwd: fixtureRoot,
      session_id: "canary-p2",
      tool_name: "spawn_agent",
      tool_input: { agent_type: "worker", message: "do the thing without a tier marker" },
    });
    record(
      "P2 unmarked spawn_agent denied [synthetic-through-installed-hook]",
      p2.status === 2,
      `bee-model-guard.mjs exit ${p2.status} (expected 2); stderr: ${String(p2.stderr || "").slice(0, 200)}`,
    );

    // ---- P3 (synthetic-through-installed-hook): marker-anchored spawn allowed ----
    const p3 = invokeInstalledHook(fixtureRoot, "bee-model-guard.mjs", {
      hook_event_name: "PreToolUse",
      cwd: fixtureRoot,
      session_id: "canary-p3",
      tool_name: "spawn_agent",
      tool_input: { agent_type: "worker", message: "[bee-tier: generation] do the thing" },
    });
    record(
      "P3 marker-anchored spawn allowed [synthetic-through-installed-hook]",
      p3.status === 0,
      `bee-model-guard.mjs exit ${p3.status} (expected 0); stderr: ${String(p3.stderr || "").slice(0, 200)}`,
    );

    // ---- P4 (synthetic-through-installed-hook): update_plan triggers state-sync ----
    const statePath = path.join(fixtureRoot, ".bee", "state.json");
    const stateBefore = JSON.parse(fs.readFileSync(statePath, "utf8"));
    const p4 = invokeInstalledHook(fixtureRoot, "bee-state-sync.mjs", {
      hook_event_name: "PostToolUse",
      cwd: fixtureRoot,
      tool_name: "update_plan",
      tool_input: { plan: [{ step: "canary probe", status: "in_progress" }] },
    });
    const stateAfter = JSON.parse(fs.readFileSync(statePath, "utf8"));
    const synced =
      typeof stateAfter.last_activity === "string" &&
      stateAfter.last_activity !== stateBefore.last_activity &&
      !Number.isNaN(Date.parse(stateAfter.last_activity)) &&
      stateAfter.cells &&
      typeof stateAfter.cells.open === "number";
    record(
      "P4 update_plan triggers state-sync [synthetic-through-installed-hook]",
      p4.status === 0 && synced,
      synced
        ? `.bee/state.json last_activity -> ${stateAfter.last_activity}, cells -> ${JSON.stringify(stateAfter.cells)}`
        : `bee-state-sync.mjs exit ${p4.status}; last_activity before=${stateBefore.last_activity} after=${stateAfter.last_activity}`,
    );

    // ---- P5 (synthetic-through-installed-hook): pre-Gate-3 source write blocked ----
    const p5 = invokeInstalledHook(fixtureRoot, "bee-write-guard.mjs", {
      hook_event_name: "PreToolUse",
      cwd: fixtureRoot,
      tool_name: "Write",
      tool_input: { file_path: path.join(fixtureRoot, "src", "example.js"), content: "// canary" },
    });
    record(
      "P5 pre-Gate-3 source write blocked [synthetic-through-installed-hook]",
      p5.status === 2,
      `bee-write-guard.mjs exit ${p5.status} (expected 2; fixture phase idle, execution gate unapproved); stderr: ${String(
        p5.stderr || "",
      ).slice(0, 200)}`,
    );
  } finally {
    fs.rmSync(fixtureRoot, { recursive: true, force: true });
    fs.rmSync(codexHome, { recursive: true, force: true });
  }
}

async function main() {
  const args = process.argv.slice(2);

  // --probe-selftest: offline invariant check for the probe leg below (R4)
  // — never invokes `codex`, so it runs (and must pass) even when no codex
  // binary is on PATH. This is the cell's verify command.
  if (args.includes("--probe-selftest")) {
    const { ok, detail } = runProbeSelftest();
    console.log(`${ok ? "PASS" : "FAIL"} probe-selftest: ${detail}`);
    return ok ? 0 : 1;
  }

  // --probe: the real native-override probe leg (V1/V3). Requires codex;
  // absent binary degrades to the same skip as the default P1-P5 suite.
  if (args.includes("--probe")) {
    if (!codexOnPath()) {
      // Loud, machine-greppable skip marker (hardening-8): a bare "skipped"
      // sentence reads as silence once this suite runs alongside dozens of
      // others under run_verify.mjs — CANARY_SKIP is the one line every
      // caller (a human scanning logs, or run_verify's own summary below)
      // can grep for to tell "ran nothing" apart from "ran and passed".
      // Exit code stays 0: an absent codex binary is an environment fact,
      // never a failure (see file header + canary.yml).
      console.log("CANARY_SKIP reason=no-codex-binary mode=probe");
      console.log("canary --probe: skipped (no codex binary)");
      return 0;
    }
    return runProbeMode();
  }

  if (!codexOnPath()) {
    console.log("CANARY_SKIP reason=no-codex-binary mode=default");
    console.log("canary: skipped (no codex binary)");
    return 0;
  }

  await runProbes();

  const failed = results.filter((r) => !r.ok);
  if (failed.length > 0) {
    console.error(`canary: FAIL — ${failed.map((f) => f.name).join("; ")}`);
    return 1;
  }
  console.log("canary: all probes green");
  return 0;
}

process.exitCode = await main();
