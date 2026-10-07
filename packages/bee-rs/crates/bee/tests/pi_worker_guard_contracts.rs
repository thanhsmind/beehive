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
const registeredTools = [];
const pi = {{
  on(event, handler) {{
    if (!handlers.has(event)) handlers.set(event, []);
    handlers.get(event).push(handler);
  }},
  registerTool(tool) {{
    registeredTools.push(tool);
  }},
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
    cmd.env_remove("BEE_SUPERVISOR_ALLOWED");
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
fn test_index_registers_no_dispatch_tools_for_paseo_worker() {
    run_extension_test(
        "test_index_registers_no_dispatch_tools_for_paseo_worker",
        &[("BEE_HERDING_WORKER", "1"), ("PASEO_AGENT_ID", "agent-1")],
        r#"
assert.equal(registeredTools.length, 1);
assert.equal(registeredTools[0].name, "verdict");
"#,
    );
}

#[test]
fn test_index_registers_dispatch_tools_without_paseo_worker() {
    run_extension_test(
        "test_index_registers_dispatch_tools_without_paseo_worker",
        &[],
        r#"
const toolNames = registeredTools.map(t => t.name);
assert.ok(toolNames.includes("verdict"));
assert.ok(toolNames.includes("bee_dispatch"));
assert.ok(toolNames.includes("bee_advisor"));
assert.ok(toolNames.includes("bee_steer"));
"#,
    );
}

#[cfg(unix)]
#[test]
fn test_belt_blocks_paseo_worker_shell_call_when_stub_denies() {
    run_extension_test(
        "test_belt_blocks_paseo_worker_shell_call_when_stub_denies",
        &[("BEE_HERDING_WORKER", "1"), ("PASEO_AGENT_ID", "agent-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pwg-deny-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
fs.writeFileSync(path.join(tmpDir, ".bee", "onboarding.json"), "{}");
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh
if [ "$1" = "hook" ] && [ "$2" = "--help" ]; then
  echo "hooks:"
  echo "  activity"
  echo "  worker-guard"
  exit 0
fi
if [ "$1" = "hook" ] && [ "$2" = "worker-guard" ]; then
  echo "bee worker-guard denied this shell command: git push is not allowed for a Paseo worker. FIX: stop and report blocked; the leader does this step." >&2
  exit 2
fi
exit 0
`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-1" };
const res = await fire("tool_call", { toolName: "bash", input: { command: "git push" } }, ctx);
assert.ok(res);
assert.equal(res.block, true);
assert.ok(res.reason.includes("worker-guard denied"));
"#,
    );
}

#[cfg(unix)]
#[test]
fn test_belt_allows_paseo_worker_shell_call_when_stub_allows() {
    run_extension_test(
        "test_belt_allows_paseo_worker_shell_call_when_stub_allows",
        &[("BEE_HERDING_WORKER", "1"), ("PASEO_AGENT_ID", "agent-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pwg-allow-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
fs.writeFileSync(path.join(tmpDir, ".bee", "onboarding.json"), "{}");
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh
if [ "$1" = "hook" ] && [ "$2" = "--help" ]; then
  echo "hooks:"
  echo "  activity"
  echo "  worker-guard"
  exit 0
fi
exit 0
`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-1" };
const res = await fire("tool_call", { toolName: "bash", input: { command: "cargo test" } }, ctx);
assert.equal(res, undefined);
"#,
    );
}

#[test]
fn test_belt_blocks_when_binary_is_missing() {
    run_extension_test(
        "test_belt_blocks_when_binary_is_missing",
        &[("BEE_HERDING_WORKER", "1"), ("PASEO_AGENT_ID", "agent-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pwg-missing-"));
fs.mkdirSync(path.join(tmpDir, ".bee"), { recursive: true });
fs.writeFileSync(path.join(tmpDir, ".bee", "onboarding.json"), "{}");

const ctx = { cwd: tmpDir, sessionId: "sess-1" };
const res = await fire("tool_call", { toolName: "bash", input: { command: "cargo test" } }, ctx);
assert.ok(res);
assert.equal(res.block, true);
assert.ok(res.reason.includes("could not find the bee binary"));
"#,
    );
}

#[test]
fn test_belt_blocks_when_stub_hook_help_lacks_worker_guard() {
    run_extension_test(
        "test_belt_blocks_when_stub_hook_help_lacks_worker_guard",
        &[("BEE_HERDING_WORKER", "1"), ("PASEO_AGENT_ID", "agent-1")],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pwg-lacks-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
fs.writeFileSync(path.join(tmpDir, ".bee", "onboarding.json"), "{}");
const stubBee = path.join(beeBinDir, "bee");
fs.writeFileSync(stubBee, `#!/bin/sh
if [ "$1" = "hook" ] && [ "$2" = "--help" ]; then
  echo "hooks:"
  echo "  activity"
  echo "  write-guard"
  exit 0
fi
exit 0
`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-1" };
const res = await fire("tool_call", { toolName: "bash", input: { command: "cargo test" } }, ctx);
assert.ok(res);
assert.equal(res.block, true);
assert.ok(res.reason.includes("does not list worker-guard"));
"#,
    );
}

#[test]
fn test_belt_does_not_call_stub_when_no_variable_set() {
    run_extension_test(
        "test_belt_does_not_call_stub_when_no_variable_set",
        &[],
        r#"
const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "pwg-noenv-"));
const beeBinDir = path.join(tmpDir, ".bee", "bin");
fs.mkdirSync(beeBinDir, { recursive: true });
fs.writeFileSync(path.join(tmpDir, ".bee", "onboarding.json"), "{}");
const stubBee = path.join(beeBinDir, "bee");
const callsLog = path.join(tmpDir, "worker-guard-calls.log");
fs.writeFileSync(stubBee, `#!/bin/sh
if [ "$1" = "hook" ] && [ "$2" = "worker-guard" ]; then
  echo "$*" >> "${callsLog}"
  exit 0
fi
exit 0
`);
fs.chmodSync(stubBee, 0o755);

const ctx = { cwd: tmpDir, sessionId: "sess-1" };
await fire("tool_call", { toolName: "bash", input: { command: "cargo test" } }, ctx);
assert.equal(fs.existsSync(callsLog), false);
"#,
    );
}
