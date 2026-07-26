// bee-codex-subagent-audit: Codex SubagentStart + SubagentStop.
//
// Codex exposes SubagentStart only after the native subagent has started. This
// handler therefore records bounded audit evidence; it is never a pre-spawn
// authorization or denial surface. The same handler closes the audit at stop.

import {
  appendHookLog,
  logCoverageGap,
  logCrash,
  readHookContext,
} from "./adapter.mjs";

const HOOK_NAME = "codex-subagent-audit";
const FIELD_LIMITS = Object.freeze({
  session_id: 120,
  agent_id: 120,
  agent_name: 120,
  agent_type: 80,
});

function boundedAuditFields(payload) {
  const out = {};
  for (const [field, limit] of Object.entries(FIELD_LIMITS)) {
    const value = payload && payload[field];
    if (typeof value !== "string" || !value.trim()) continue;
    const normalized = value.trim();
    out[field] = normalized.length <= limit ? normalized : `${normalized.slice(0, limit)}...`;
  }
  return out;
}

async function main() {
  let ctx = null;
  try {
    ctx = await readHookContext(HOOK_NAME);
    if (!ctx.root) return;

    const lifecycle = ctx.event === "SubagentStart"
      ? "start"
      : ctx.event === "SubagentStop"
        ? "stop"
        : null;
    if (!lifecycle) {
      logCoverageGap(
        ctx.root,
        HOOK_NAME,
        "unsupported-subagent-event",
        "expected SubagentStart or SubagentStop; audit record omitted",
        ctx.source,
      );
      return;
    }

    appendHookLog(ctx.root, {
      ts: new Date().toISOString(),
      hook: HOOK_NAME,
      event: "subagent-audit",
      lifecycle,
      authority: "audit-only",
      ...(lifecycle === "start" ? { timing: "post-start" } : {}),
      ...(ctx.source ? { source: ctx.source } : {}),
      ...boundedAuditFields(ctx.payload),
    });
  } catch (error) {
    logCrash(ctx && ctx.root, HOOK_NAME, error, ctx && ctx.source);
  }
}

await main();
