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
const { HEARTBEAT_NAME, DEFAULT_CRON, readPaseoSettings } = mod;
assert.equal(HEARTBEAT_NAME, "bee-leader");
assert.equal(DEFAULT_CRON, "*/5 * * * *");

const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "phb-test-settings-"));
const defaultSettings = readPaseoSettings(tmpDir);
assert.deepEqual(defaultSettings, { command: "paseo", cron: "*/5 * * * *" });

const beeDir = path.join(tmpDir, ".bee");
fs.mkdirSync(beeDir, { recursive: true });
fs.writeFileSync(path.join(beeDir, "config.json"), JSON.stringify({
  herding: {
    paseo: {
      command: "custom-paseo",
      heartbeat_cron: "0 * * * *"
    }
  }
}));
const customSettings = readPaseoSettings(tmpDir);
assert.deepEqual(customSettings, { command: "custom-paseo", cron: "0 * * * *" });

fs.writeFileSync(path.join(beeDir, "config.json"), "{ invalid json");
const fallbackSettings = readPaseoSettings(tmpDir);
assert.deepEqual(fallbackSettings, { command: "paseo", cron: "*/5 * * * *" });
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
