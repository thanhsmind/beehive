use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR too shallow: {}", env!("CARGO_MANIFEST_DIR")))
        .to_path_buf()
}

fn node_typescript_probe() -> Result<(), String> {
    let version_out = Command::new("node").arg("--version").output();
    let version = match version_out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return Err("`node` not found on PATH".to_string()),
    };
    let dir = tempfile::tempdir().map_err(|e| format!("could not create a tempdir for the node/TS probe: {e}"))?;
    let probe = dir.path().join("probe.ts");
    std::fs::write(&probe, "const x: number = 1\nconsole.log(x)\n").map_err(|e| e.to_string())?;
    let out = Command::new("node")
        .arg(&probe)
        .output()
        .map_err(|e| format!("failed to spawn `node {}`: {e}", probe.display()))?;
    if out.status.success() && String::from_utf8_lossy(&out.stdout).trim() == "1" {
        Ok(())
    } else {
        Err(format!(
            "`node` ({version}) cannot run a minimal .ts file directly — stderr: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

const ALLOW_SKIP_ENV: &str = "BEE_PI_SUITE_ALLOW_SKIP";

fn env_allows_skip() -> bool {
    std::env::var_os(ALLOW_SKIP_ENV).is_some()
}

macro_rules! node_or_skip {
    ($test_name:expr) => {
        if let Err(reason) = node_typescript_probe() {
            let allow = env_allows_skip();
            eprintln!("{} (env-limited: {reason}) — {}", if allow { "SKIP" } else { "FAIL" }, $test_name);
            if allow {
                return;
            }
            panic!(
                "{}: a `node` capable of stripping TypeScript natively is required ({reason}). Set {ALLOW_SKIP_ENV}=1 to explicitly accept a degraded run.",
                $test_name
            );
        }
    };
}

fn run_js_test(name: &str, code: &str) {
    node_or_skip!(name);
    let module_path = repo_root().join(".pi/extensions/bee-guard/paseo-heartbeat.ts");
    let script = format!(
        r#"
import {{ pathToFileURL }} from "node:url";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import assert from "node:assert/strict";

const mod = await import(pathToFileURL({:?}).href);
{}
"#,
        module_path.to_str().expect("valid utf-8 path"),
        code
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("runner.mjs");
    std::fs::write(&script_path, script).expect("write runner.mjs");

    let output = Command::new("node")
        .arg(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn node");

    if !output.status.success() {
        panic!(
            "Test {} failed (exit code {:?}):\nstdout:\n{}\nstderr:\n{}",
            name,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn run_extension_test(name: &str, env_vars: &[(&str, &str)], code: &str) {
    node_or_skip!(name);
    let extension_path = repo_root().join(".pi/extensions/bee-guard/index.ts");
    let script = format!(
        r#"
import {{ pathToFileURL }} from "node:url";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import assert from "node:assert/strict";

const handlers = new Map();
const messages = [];
const pi = {{
  on(event, handler) {{
    if (!handlers.has(event)) handlers.set(event, []);
    handlers.get(event).push(handler);
  }},
  registerTool() {{}},
  registerCommand() {{}},
  getActiveTools() {{ return []; }},
  setActiveTools() {{}},
  getAllTools() {{ return []; }},
  sendUserMessage: async (text, options) => {{
    messages.push({{ text: String(text), options: options ?? null }});
  }},
}};

const extMod = await import(pathToFileURL({:?}).href);
await extMod.default(pi);

const fire = async (event, eventArg, ctx) => {{
  const list = handlers.get(event) ?? [];
  let out = undefined;
  for (const fn of list) {{
    const r = await fn(eventArg, ctx);
    if (r !== undefined && r !== null) out = r;
  }}
  return out;
}};

{}
"#,
        extension_path.to_str().expect("valid utf-8 path"),
        code
    );
    let dir = tempfile::tempdir().expect("tempdir");
    let script_path = dir.path().join("runner.mjs");
    std::fs::write(&script_path, script).expect("write runner.mjs");

    let mut cmd = Command::new("node");
    cmd.arg(&script_path);
    cmd.env_remove("BEE_HERDING_WORKER");
    cmd.env_remove("PASEO_AGENT_ID");
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    let output = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn node");

    if !output.status.success() {
        panic!(
            "Test {} failed (exit code {:?}):\nstdout:\n{}\nstderr:\n{}",
            name,
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_is_paseo_leader() {
    run_js_test(
        "test_is_paseo_leader",
        r#"
const { isPaseoLeader } = mod;
assert.equal(isPaseoLeader({ PASEO_AGENT_ID: "agent-1" }), true);
assert.equal(isPaseoLeader({ PASEO_AGENT_ID: "agent-1", BEE_HERDING_WORKER: "" }), true);
assert.equal(isPaseoLeader({ PASEO_AGENT_ID: "agent-1", BEE_HERDING_WORKER: "1" }), false);
assert.equal(isPaseoLeader({ PASEO_AGENT_ID: "" }), false);
assert.equal(isPaseoLeader({}), false);
assert.equal(isPaseoLeader({ PASEO_AGENT_ID: "agent-1", BEE_HERDING_WORKER: "worker-1" }), false);
"#,
    );
}

#[test]
fn test_is_heartbeat_prompt() {
    run_js_test(
        "test_is_heartbeat_prompt",
        r#"
const { isHeartbeatPrompt } = mod;
assert.equal(isHeartbeatPrompt('<paseo-system>Schedule "bee-leader" fired (id=x, run=1). bee heartbeat</paseo-system>'), true);
assert.equal(isHeartbeatPrompt('  \n <paseo-system>Schedule "bee-leader" fired</paseo-system>'), true);
assert.equal(isHeartbeatPrompt('Schedule "bee-leader" fired'), false);
assert.equal(isHeartbeatPrompt('<paseo-system>Schedule "other" fired</paseo-system>'), false);
assert.equal(isHeartbeatPrompt('hello'), false);
assert.equal(isHeartbeatPrompt(null), false);
assert.equal(isHeartbeatPrompt(undefined), false);
assert.equal(isHeartbeatPrompt(123), false);
"#,
    );
}

#[test]
fn test_constants_and_read_paseo_settings() {
    run_js_test(
        "test_constants_and_read_paseo_settings",
        r#"
const { HEARTBEAT_NAME, DEFAULT_CRON, STALE_MARKER_MS, readPaseoSettings } = mod;
assert.equal(HEARTBEAT_NAME, "bee-leader");
assert.equal(DEFAULT_CRON, "*/30 * * * *");
assert.equal(STALE_MARKER_MS, 120000);

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-test-settings-"));
const defaultSettings = readPaseoSettings(tmpDir);
assert.deepEqual(defaultSettings, { command: "paseo", cron: "*/30 * * * *", broker_tick_secs: 30 });

const beeDir = path.join(tmpDir, ".bee");
fs.mkdirSync(beeDir, { recursive: true });
fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      command: "custom-paseo",
      heartbeat_cron: "0 * * * *",
      broker_tick_secs: 45
    }
  }
}));
const customSettings = readPaseoSettings(tmpDir);
assert.deepEqual(customSettings, { command: "custom-paseo", cron: "0 * * * *", broker_tick_secs: 45 });

fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      broker_tick_secs: -10
    }
  }
}));
const negativeSettings = readPaseoSettings(tmpDir);
assert.equal(negativeSettings.broker_tick_secs, 30);

fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      broker_tick_secs: "not-a-number"
    }
  }
}));
const stringSettings = readPaseoSettings(tmpDir);
assert.equal(stringSettings.broker_tick_secs, 30);

fs.writeFileSync(path.join(beeDir, "config.json"), "{ invalid json");
const fallbackSettings = readPaseoSettings(tmpDir);
assert.deepEqual(fallbackSettings, { command: "paseo", cron: "*/30 * * * *", broker_tick_secs: 30 });
"#,
    );
}

#[test]
fn test_heartbeat_marker_path() {
    run_js_test(
        "test_heartbeat_marker_path",
        r#"
const { heartbeatMarkerPath } = mod;
const p = heartbeatMarkerPath("/test/root", "my-agent");
assert.equal(p, path.join("/test/root", ".bee", "runtime", "paseo-heartbeat", "my-agent.json"));
"#,
    );
}

#[test]
fn test_ensure_heartbeat_concurrency_and_failure() {
    run_js_test(
        "test_ensure_heartbeat_concurrency_and_failure",
        r#"
const { ensureHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-test-ensure-"));
const agentId = "agent-conc";
const settings = { command: "paseo", cron: "*/5 * * * *" };

let callCount = 0;
const stubRun = async (cmd, args) => {
  callCount++;
  assert.equal(cmd, "paseo");
  assert.deepEqual(args, ["heartbeat", "create", "--cron", "*/5 * * * *", "--name", "bee-leader", "--json", "bee heartbeat"]);
  return JSON.stringify({ id: "sched-123" });
};

const [res1, res2] = await Promise.all([
  ensureHeartbeat(tmpDir, agentId, settings, stubRun),
  ensureHeartbeat(tmpDir, agentId, settings, stubRun),
]);

assert.equal(callCount, 1);
const createdOne = res1.created ? res1 : res2;
const failedOne = res1.created ? res2 : res1;
assert.equal(createdOne.created, true);
assert.equal(createdOne.id, "sched-123");
assert.equal(failedOne.created, false);
assert.equal(failedOne.reason, "exists");

const markerPath = heartbeatMarkerPath(tmpDir, agentId);
assert.equal(fs.existsSync(markerPath), true);
const markerContent = JSON.parse(fs.readFileSync(markerPath, "utf8"));
assert.equal(markerContent.agent_id, agentId);
assert.equal(markerContent.schedule_id, "sched-123");
assert.equal(markerContent.cron, "*/5 * * * *");
assert.ok(markerContent.created_at);

const failAgentId = "agent-fail";
const failRun = async () => {
  throw new Error("spawn error");
};
const failRes = await ensureHeartbeat(tmpDir, failAgentId, settings, failRun);
assert.equal(failRes.created, false);
assert.ok(failRes.reason.includes("spawn error"));
const failMarker = heartbeatMarkerPath(tmpDir, failAgentId);
assert.equal(fs.existsSync(failMarker), false);

const altAgentId = "agent-alt";
const altRun = async () => {
  return "prefix text\n" + JSON.stringify({ Id: "sched-456" }) + "\nsuffix text";
};
const altRes = await ensureHeartbeat(tmpDir, altAgentId, settings, altRun);
assert.equal(altRes.created, true);
assert.equal(altRes.id, "sched-456");
"#,
    );
}

#[test]
fn test_parse_tick() {
    run_js_test(
        "test_parse_tick",
        r#"
const { parseTick } = mod;
const tick1 = parseTick(JSON.stringify({ claimed: 2, notices_sent: 1 }));
assert.deepEqual(tick1, { claimed: 2, notices_sent: 1, news: true });

const tick2 = parseTick(JSON.stringify({ claimed: 0, notices_sent: 0 }));
assert.deepEqual(tick2, { claimed: 0, notices_sent: 0, news: false });

const tick3 = parseTick(JSON.stringify({}));
assert.deepEqual(tick3, { claimed: 0, notices_sent: 0, news: false });

const tick4 = parseTick("not json");
assert.equal(tick4, null);

const tick5 = parseTick("some logs\n" + JSON.stringify({ claimed: 5 }) + "\nend logs");
assert.deepEqual(tick5, { claimed: 5, notices_sent: 0, news: true });
"#,
    );
}

#[test]
fn test_heartbeat_text() {
    run_js_test(
        "test_heartbeat_text",
        r#"
const { heartbeatText } = mod;
const textWithNews = heartbeatText({ claimed: 2, notices_sent: 1, news: true });
assert.equal(textWithNews, "bee heartbeat: the broker routed 2 question(s) and sent 1 notice(s). Read them with bee orient and act.");

const textNoNews = heartbeatText({ claimed: 0, notices_sent: 0, news: false });
assert.equal(textNoNews, "bee heartbeat: no news. Reply with the single word ok and do nothing else.");

const textNull = heartbeatText(null);
assert.equal(textNull, "bee heartbeat: no news. Reply with the single word ok and do nothing else.");
"#,
    );
}

#[test]
fn test_leader_session_start_calls_stub_paseo_once() {
    run_extension_test(
        "test_leader_session_start_calls_stub_paseo_once",
        &[("PASEO_AGENT_ID", "agent-leader-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-leader-start-"));
const stubPaseo = path.join(tmpDir, "stub-paseo.sh");
const callsLog = path.join(tmpDir, "paseo-calls.log");
fs.writeFileSync(stubPaseo, `#!/bin/sh\nprintf '%s\n' "$*" >> "${callsLog}"\nprintf '{"id":"sched-hb-42"}'\n`);
fs.chmodSync(stubPaseo, 0o755);

const beeDir = path.join(tmpDir, ".bee");
fs.mkdirSync(beeDir, { recursive: true });
fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      command: stubPaseo
    }
  }
}));

const ctx = { cwd: tmpDir, sessionId: "sess-leader-1" };
await fire("session_start", { reason: "new" }, ctx);

const deadline = Date.now() + 2000;
const markerPath = path.join(tmpDir, ".bee", "runtime", "paseo-heartbeat", "agent-leader-1.json");
let marker = {};
while (Date.now() < deadline) {
  if (fs.existsSync(callsLog) && fs.existsSync(markerPath)) {
    try {
      const parsed = JSON.parse(fs.readFileSync(markerPath, "utf8"));
      if (parsed.schedule_id) {
        marker = parsed;
        break;
      }
    } catch {}
  }
  await new Promise((r) => setTimeout(r, 20));
}

assert.ok(fs.existsSync(callsLog), "stub paseo must have been called");
const lines1 = fs.readFileSync(callsLog, "utf8").trim().split("\n");
assert.equal(lines1.length, 1);
assert.equal(lines1[0], "heartbeat create --cron */30 * * * * --name bee-leader --json bee heartbeat");

assert.ok(fs.existsSync(markerPath), "marker file must exist");
assert.equal(marker.schedule_id, "sched-hb-42");

await fire("session_start", { reason: "new" }, ctx);
await new Promise((r) => setTimeout(r, 100));
const lines2 = fs.readFileSync(callsLog, "utf8").trim().split("\n");
assert.equal(lines2.length, 1, "stub paseo must only be called once");
"#,
    );
}

#[test]
fn test_worker_session_start_makes_no_call() {
    run_extension_test(
        "test_worker_session_start_makes_no_call",
        &[("PASEO_AGENT_ID", "agent-worker-1"), ("BEE_HERDING_WORKER", "1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-worker-start-"));
const stubPaseo = path.join(tmpDir, "stub-paseo.sh");
const callsLog = path.join(tmpDir, "paseo-calls.log");
fs.writeFileSync(stubPaseo, `#!/bin/sh\nprintf '%s\n' "$*" >> "${callsLog}"\nprintf '{"id":"sched-hb-42"}'\n`);
fs.chmodSync(stubPaseo, 0o755);

const beeDir = path.join(tmpDir, ".bee");
fs.mkdirSync(beeDir, { recursive: true });
fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      command: stubPaseo
    }
  }
}));

const ctx = { cwd: tmpDir, sessionId: "sess-worker-1" };
await fire("session_start", { reason: "new" }, ctx);
await new Promise((r) => setTimeout(r, 100));
assert.equal(fs.existsSync(callsLog), false, "stub paseo must not be called in worker env");
"#,
    );
}

#[test]
fn test_heartbeat_input_no_news_returns_transform_ok() {
    run_extension_test(
        "test_heartbeat_input_no_news_returns_transform_ok",
        &[("PASEO_AGENT_ID", "agent-leader-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-input-nonews-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh\nprintf '{"claimed":0,"notices_sent":0}'\n`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-leader-1" };
const res = await fire("input", { text: '<paseo-system>Schedule "bee-leader" fired (id=x, run=1). bee heartbeat</paseo-system>' }, ctx);
assert.deepEqual(res, {
  action: "transform",
  text: "bee heartbeat: no news. Reply with the single word ok and do nothing else."
});
"#,
    );
}

#[test]
fn test_heartbeat_input_with_news_returns_transform_news() {
    run_extension_test(
        "test_heartbeat_input_with_news_returns_transform_news",
        &[("PASEO_AGENT_ID", "agent-leader-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-input-news-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh\nprintf '{"claimed":1,"notices_sent":0}'\n`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-leader-1" };
const res = await fire("input", { text: '<paseo-system>Schedule "bee-leader" fired (id=x, run=1). bee heartbeat</paseo-system>' }, ctx);
assert.deepEqual(res, {
  action: "transform",
  text: "bee heartbeat: the broker routed 1 question(s) and sent 0 notice(s). Read them with bee orient and act."
});
"#,
    );
}

#[test]
fn test_normal_input_returns_continue() {
    run_extension_test(
        "test_normal_input_returns_continue",
        &[("PASEO_AGENT_ID", "agent-leader-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-input-normal-"));
const ctx = { cwd: tmpDir, sessionId: "sess-leader-1" };
const res = await fire("input", { text: "hello please help me write some code" }, ctx);
assert.deepEqual(res, { action: "continue" });
"#,
    );
}

#[test]
fn test_quiet_heartbeat_settle_with_block_verdict_injects_no_nudge() {
    run_extension_test(
        "test_quiet_heartbeat_settle_with_block_verdict_injects_no_nudge",
        &[("PASEO_AGENT_ID", "agent-leader-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-settle-nudge-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh
if [ "$1" = "herding" ] && [ "$2" = "broker" ] && [ "$3" = "tick" ]; then
  printf '{"claimed":0,"notices_sent":0}'
  exit 0
fi
if [ "$1" = "hook" ] && [ "$2" = "session-close" ]; then
  printf '{"decision":"block","reason":"continuation nudge reason"}'
  exit 0
fi
exit 0
`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-leader-1" };
await fire("input", { text: '<paseo-system>Schedule "bee-leader" fired (id=x, run=1). bee heartbeat</paseo-system>' }, ctx);
await fire("agent_settled", {}, ctx);
assert.equal(messages.length, 0, "quiet heartbeat settle must inject no continuation nudge");

await fire("agent_settled", {}, ctx);
assert.equal(messages.length, 1, "subsequent settle with block verdict must inject continuation nudge");
assert.equal(messages[0].text, "continuation nudge reason");
"#,
    );
}

#[test]
fn test_empty_marker_200s_ago_retried_and_run_called_once() {
    run_js_test(
        "test_empty_marker_200s_ago_retried_and_run_called_once",
        r#"
const { ensureHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-stale-marker-"));
const agentId = "agent-stale-200s";
const settings = { command: "paseo", cron: "*/5 * * * *" };
const marker = heartbeatMarkerPath(tmpDir, agentId);
fs.mkdirSync(path.dirname(marker), { recursive: true });
fs.writeFileSync(marker, "{}");
const past = new Date(Date.now() - 200000);
fs.utimesSync(marker, past, past);

let calls = 0;
const stubRun = async () => {
  calls++;
  return JSON.stringify({ id: "sched-retry-1" });
};

const res = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(calls, 1);
assert.equal(res.created, true);
assert.equal(res.id, "sched-retry-1");
const updated = JSON.parse(fs.readFileSync(marker, "utf8"));
assert.equal(updated.schedule_id, "sched-retry-1");
"#,
    );
}

#[test]
fn test_empty_marker_10s_old_returns_exists_with_no_call() {
    run_js_test(
        "test_empty_marker_10s_old_returns_exists_with_no_call",
        r#"
const { ensureHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-fresh-marker-"));
const agentId = "agent-fresh-10s";
const settings = { command: "paseo", cron: "*/5 * * * *" };
const marker = heartbeatMarkerPath(tmpDir, agentId);
fs.mkdirSync(path.dirname(marker), { recursive: true });
fs.writeFileSync(marker, "{}");
const recent = new Date(Date.now() - 10000);
fs.utimesSync(marker, recent, recent);

let calls = 0;
const stubRun = async () => {
  calls++;
  return JSON.stringify({ id: "sched-should-not-call" });
};

const res = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(calls, 0);
assert.equal(res.created, false);
assert.equal(res.reason, "exists");
"#,
    );
}

#[test]
fn test_marker_with_schedule_id_200s_old_returns_exists_with_no_call() {
    run_js_test(
        "test_marker_with_schedule_id_200s_old_returns_exists_with_no_call",
        r#"
const { ensureHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-sched-200s-"));
const agentId = "agent-sched-200s";
const settings = { command: "paseo", cron: "*/5 * * * *" };
const marker = heartbeatMarkerPath(tmpDir, agentId);
fs.mkdirSync(path.dirname(marker), { recursive: true });
fs.writeFileSync(marker, JSON.stringify({
  agent_id: agentId,
  schedule_id: "sched-existing",
  cron: "*/5 * * * *",
  created_at: new Date(Date.now() - 200000).toISOString()
}));
const past = new Date(Date.now() - 200000);
fs.utimesSync(marker, past, past);

let calls = 0;
const stubRun = async () => {
  calls++;
  return JSON.stringify({ id: "sched-should-not-call" });
};

const res = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(calls, 0);
assert.equal(res.created, false);
assert.equal(res.reason, "exists");
"#,
    );
}

#[test]
fn test_failed_create_reason_contains_herding_paseo_command() {
    run_js_test(
        "test_failed_create_reason_contains_herding_paseo_command",
        r#"
const { ensureHeartbeat } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-fail-reason-"));
const agentId = "agent-fail-reason";
const settings = { command: "paseo", cron: "*/5 * * * *" };
const stubRun = async () => {
  throw new Error("CLI process hung and died");
};

const res = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(res.created, false);
assert.ok(res.reason.includes("herding.paseo.command"));
assert.ok(res.reason.endsWith(" — FIX: set herding.paseo.command to the npm @getpaseo/cli paseo binary"));
"#,
    );
}

#[test]
fn test_ensure_heartbeat_recreates_on_changed_cron_and_exists_on_same_cron() {
    run_js_test(
        "test_ensure_heartbeat_recreates_on_changed_cron_and_exists_on_same_cron",
        r#"
const { ensureHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-cron-change-"));
const agentId = "agent-cron-change";
const marker = heartbeatMarkerPath(tmpDir, agentId);
fs.mkdirSync(path.dirname(marker), { recursive: true });
fs.writeFileSync(marker, JSON.stringify({
  agent_id: agentId,
  schedule_id: "sched-old",
  cron: "*/5 * * * *",
  created_at: new Date().toISOString()
}));

const calls = [];
const stubRun = async (cmd, args) => {
  calls.push({ cmd, args });
  if (args[0] === "heartbeat" && args[1] === "delete") {
    return "";
  }
  if (args[0] === "heartbeat" && args[1] === "create") {
    return JSON.stringify({ id: "sched-new" });
  }
  throw new Error("unexpected call");
};

const settings = { command: "paseo", cron: "*/30 * * * *", broker_tick_secs: 30 };
const res = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(res.created, true);
assert.equal(res.id, "sched-new");
assert.equal(calls.length, 2);
assert.deepEqual(calls[0].args, ["heartbeat", "delete", "sched-old"]);
assert.deepEqual(calls[1].args, ["heartbeat", "create", "--cron", "*/30 * * * *", "--name", "bee-leader", "--json", "bee heartbeat"]);

const updatedMarker = JSON.parse(fs.readFileSync(marker, "utf8"));
assert.equal(updatedMarker.schedule_id, "sched-new");
assert.equal(updatedMarker.cron, "*/30 * * * *");

const resSame = await ensureHeartbeat(tmpDir, agentId, settings, stubRun);
assert.equal(resSame.created, false);
assert.equal(resSame.reason, "exists");
assert.equal(calls.length, 2);
"#,
    );
}

#[test]
fn test_delete_heartbeat_calls_delete_and_removes_marker_or_records_failure() {
    run_js_test(
        "test_delete_heartbeat_calls_delete_and_removes_marker_or_records_failure",
        r#"
const { deleteHeartbeat, heartbeatMarkerPath } = mod;
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-del-hb-"));
const agentId = "agent-del-ok";
const marker = heartbeatMarkerPath(tmpDir, agentId);
fs.mkdirSync(path.dirname(marker), { recursive: true });
fs.writeFileSync(marker, JSON.stringify({
  agent_id: agentId,
  schedule_id: "sched-del-123",
  cron: "*/30 * * * *"
}));

let deletedId = null;
const stubRunOk = async (cmd, args) => {
  assert.equal(cmd, "paseo");
  assert.deepEqual(args, ["heartbeat", "delete", "sched-del-123"]);
  deletedId = args[2];
  return "";
};
const settings = { command: "paseo", cron: "*/30 * * * *", broker_tick_secs: 30 };
await deleteHeartbeat(tmpDir, agentId, settings, stubRunOk);
assert.equal(deletedId, "sched-del-123");
assert.equal(fs.existsSync(marker), false);

const agentIdFail = "agent-del-fail";
const markerFail = heartbeatMarkerPath(tmpDir, agentIdFail);
fs.writeFileSync(markerFail, JSON.stringify({
  agent_id: agentIdFail,
  schedule_id: "sched-fail-456",
  cron: "*/30 * * * *"
}));
const stubRunFail = async () => {
  throw new Error("daemon unreachable");
};
await deleteHeartbeat(tmpDir, agentIdFail, settings, stubRunFail);
assert.equal(fs.existsSync(markerFail), true);
const failedMarker = JSON.parse(fs.readFileSync(markerFail, "utf8"));
assert.equal(failedMarker.schedule_id, "sched-fail-456");
assert.ok(failedMarker.delete_failed);
assert.ok(failedMarker.delete_failed.includes("daemon unreachable"));
"#,
    );
}

#[test]
fn test_broker_timer_sends_once_on_news_nothing_on_no_news_steers_when_busy_and_skips_when_in_flight() {
    let state_path = repo_root().join(".pi/extensions/bee-guard/state.ts");
    let code = format!(
        r#"
const {{ startBrokerTimer, stopBrokerTimer, setTickRunning }} = mod;
const messages = [];
const pi = {{
  sendUserMessage: async (text, options) => {{
    messages.push({{ text: String(text), options: options ?? null }});
  }}
}};
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-broker-timer-"));
let tickCalls = 0;
let tickResponse = JSON.stringify({{ claimed: 0, notices_sent: 0 }});
let tickDelayMs = 0;

const stubRun = async (cmd, args) => {{
  tickCalls++;
  if (tickDelayMs > 0) {{
    await new Promise((r) => setTimeout(r, tickDelayMs));
  }}
  return tickResponse;
}};

const settings = {{ command: "paseo", cron: "*/30 * * * *", broker_tick_secs: 1 }};
startBrokerTimer(pi, tmpDir, settings, {{ run: stubRun }});

await new Promise((r) => setTimeout(r, 1100));
assert.ok(tickCalls >= 1);
assert.equal(messages.length, 0);

tickResponse = JSON.stringify({{ claimed: 1, notices_sent: 0 }});
await new Promise((r) => setTimeout(r, 1100));
assert.equal(messages.length, 1);
assert.equal(messages[0].options, null);

const stateMod = await import(pathToFileURL({:?}).href);
stateMod.state.selfBusy = true;
await new Promise((r) => setTimeout(r, 1100));
assert.equal(messages.length, 2);
assert.deepEqual(messages[1].options, {{ deliverAs: "steer" }});
stateMod.state.selfBusy = false;

setTickRunning(true);
const callsBeforeInFlight = tickCalls;
await new Promise((r) => setTimeout(r, 1100));
assert.equal(tickCalls, callsBeforeInFlight);
setTickRunning(false);

tickDelayMs = 1500;
const callsBeforeSlow = tickCalls;
await new Promise((r) => setTimeout(r, 1100));
const firstCallInFlight = tickCalls;
assert.equal(firstCallInFlight, callsBeforeSlow + 1);
await new Promise((r) => setTimeout(r, 500));
assert.equal(tickCalls, firstCallInFlight);
await new Promise((r) => setTimeout(r, 1000));
tickDelayMs = 0;

stopBrokerTimer();
const callsAfterStop = tickCalls;
await new Promise((r) => setTimeout(r, 1100));
assert.equal(tickCalls, callsAfterStop);
"#,
        state_path.to_str().expect("valid utf-8 path")
    );
    run_js_test(
        "test_broker_timer_sends_once_on_news_nothing_on_no_news_steers_when_busy_and_skips_when_in_flight",
        &code,
    );
}

#[test]
fn test_leader_shutdown_deletes_heartbeat_and_restart_creates_one() {
    run_extension_test(
        "test_leader_shutdown_deletes_heartbeat_and_restart_creates_one",
        &[("PASEO_AGENT_ID", "agent-leader-lifecycle")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-leader-lifecycle-"));
const stubPaseo = path.join(tmpDir, "stub-paseo.sh");
const callsLog = path.join(tmpDir, "paseo-calls.log");
fs.writeFileSync(stubPaseo, `#!/bin/sh\nprintf '%s\n' "$*" >> "${callsLog}"\nif [ "$1" = "heartbeat" ] && [ "$2" = "create" ]; then\n  printf '{"id":"sched-lifecycle-1"}'\nfi\nexit 0\n`);
fs.chmodSync(stubPaseo, 0o755);

const beeDir = path.join(tmpDir, ".bee");
fs.mkdirSync(beeDir, { recursive: true });
fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      command: stubPaseo
    }
  }
}));

const ctx = { cwd: tmpDir, sessionId: "sess-leader-lifecycle" };
await fire("session_start", { reason: "new" }, ctx);

const deadline = Date.now() + 2000;
const markerPath = path.join(tmpDir, ".bee", "runtime", "paseo-heartbeat", "agent-leader-lifecycle.json");
let marker = {};
while (Date.now() < deadline) {
  if (fs.existsSync(markerPath)) {
    try {
      const parsed = JSON.parse(fs.readFileSync(markerPath, "utf8"));
      if (parsed.schedule_id) {
        marker = parsed;
        break;
      }
    } catch {}
  }
  await new Promise((r) => setTimeout(r, 20));
}
assert.equal(marker.schedule_id, "sched-lifecycle-1");

await fire("session_shutdown", { reason: "new" }, ctx);
assert.equal(fs.existsSync(markerPath), false);
const callsAfterShutdown = fs.readFileSync(callsLog, "utf8").trim().split("\n");
assert.equal(callsAfterShutdown.length, 2);
assert.equal(callsAfterShutdown[1], "heartbeat delete sched-lifecycle-1");

await fire("session_start", { reason: "new" }, ctx);
const deadline2 = Date.now() + 2000;
let marker2 = {};
while (Date.now() < deadline2) {
  if (fs.existsSync(markerPath)) {
    try {
      const parsed = JSON.parse(fs.readFileSync(markerPath, "utf8"));
      if (parsed.schedule_id) {
        marker2 = parsed;
        break;
      }
    } catch {}
  }
  await new Promise((r) => setTimeout(r, 20));
}
assert.equal(marker2.schedule_id, "sched-lifecycle-1");
const callsAfterRestart = fs.readFileSync(callsLog, "utf8").trim().split("\n");
assert.equal(callsAfterRestart.length, 3);
assert.equal(callsAfterRestart[2], "heartbeat create --cron */30 * * * * --name bee-leader --json bee heartbeat");
"#,
    );
}
