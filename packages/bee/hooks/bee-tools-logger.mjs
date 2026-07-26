#!/usr/bin/env node
// bee-tools-logger: PostToolUse, every tool (no matcher), BOTH runtimes.
// Passive measurement only — zero enforcement. Answers "how many tool calls,
// by whom" (advisor-and-orchestration CONTEXT.md "The Cost Model": nothing
// today measures the thing that costs money). This hook can NEVER deny, NEVER
// write a verdict to stderr, and NEVER emits a Codex decision:"block" — a
// logger that can block a tool is a defect by definition.
// Line schema (AO15, decision f1ca79b9): {ts, tool_name, agent_id, agent_type}
// — agent_id/agent_type are null on orchestrator calls (the fields are absent
// from the payload); duration/status are appended ONLY when the payload
// itself carries them. tool_input/tool_response bodies are NEVER logged
// (PII/secret discipline — attribution and names only, never content).
// Thin wrapper on the bee-prompt-context.mjs pattern: input/root/logging go
// through the shared runtime adapter (hooks/adapter.mjs) — stdin is
// normalized before any property access and root discovery lives inside the
// fail-open boundary.
// Fail-open: every throw -> logCrash + exit 0 (crash logged to
// .bee/logs/hooks.jsonl, visibly, never silently — fail-open must never
// become fail-silent).

import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl } from "./adapter.mjs";

const HOOK_NAME = "tools-logger";

function logToolCall(root, payload) {
  const logsDir = path.join(root, ".bee", "logs");
  fs.mkdirSync(logsDir, { recursive: true });
  const toolName = typeof payload.tool_name === "string" ? payload.tool_name : "";
  const entry = {
    ts: new Date().toISOString(),
    tool_name: toolName,
    agent_id: typeof payload.agent_id === "string" ? payload.agent_id : null,
    agent_type: typeof payload.agent_type === "string" ? payload.agent_type : null,
  };
  if (typeof payload.duration_ms === "number" && Number.isFinite(payload.duration_ms)) {
    entry.duration_ms = payload.duration_ms;
  }
  if (typeof payload.tool_status === "string" && payload.tool_status) {
    entry.status = payload.tool_status;
  }
  fs.appendFileSync(path.join(logsDir, "tools.jsonl"), `${JSON.stringify(entry)}\n`);
}

async function main() {
  const ctx = await readHookContext(HOOK_NAME);
  const root = ctx.root;
  if (!root) {
    return 0;
  }
  if (!fs.existsSync(path.join(root, ".bee", "bin", "lib", "state.mjs"))) {
    return 0;
  }

  try {
    const state = await import(libModuleUrl(root, "state.mjs"));
    if (!state.hookEnabled(root, HOOK_NAME)) {
      return 0;
    }
    logToolCall(root, ctx.payload);
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
  return 0;
}

process.exitCode = await main();
