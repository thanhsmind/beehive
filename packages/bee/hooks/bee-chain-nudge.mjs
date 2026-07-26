#!/usr/bin/env node
// bee-chain-nudge: SubagentStop.
// Advances the bee chain mechanically: when a registered bee worker stops (or
// the phase is swarming) it nudges the orchestrator to collect the [STATUS],
// update the cell, and check reservations; when the phase is reviewing it
// nudges reviewer synthesis. Otherwise silent.
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs,
// cell codex-parity-3, decision D2). SubagentStop is an advisory event: any
// output is emitted as a parseable JSON systemMessage, never plain prose and
// never decision:"block" (which would continue the child instead of advising
// the parent — discovery.md "Current Codex Contracts").
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).

import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl, emitHookOutput } from "./adapter.mjs";

const HOOK_NAME = "chain-nudge";

function getAgentName(payload) {
  const candidates = [
    payload.agent_name,
    payload.agentName,
    payload.agent_nickname,
    payload.subagent_type,
    payload.agent_type,
  ];
  for (const value of candidates) {
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return "";
}

function workerName(entry) {
  if (typeof entry === "string") {
    return entry;
  }
  if (entry && typeof entry === "object") {
    // The state CLI (bee.mjs state worker add) registers workers under
    // `nickname` (discovery.md Proved Gaps) — match that first; the generic
    // name|agent|worker fallback stays for foreign/legacy entries.
    return entry.nickname || entry.name || entry.agent || entry.worker || "";
  }
  return "";
}

// fresh-session-handoff fsh-6 (D4): a bound session's PHASE comes from its
// lane via resolvePipeline; workers stay a global registry regardless (lane
// records carry no `workers` field — cell registration is not lane-scoped).
// No session_id, an unbound session, or an unresolvable binding all fall
// back to the default record — this hook is advisory only and must never
// let a lane-resolution gap block the nudge (fail-open, matching the file's
// own documented discipline).
function getSessionId(payload) {
  return typeof payload.session_id === "string" && payload.session_id.trim()
    ? payload.session_id.trim()
    : null;
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
    const stateLib = await import(libModuleUrl(root, "state.mjs"));
    if (!stateLib.hookEnabled(root, HOOK_NAME)) {
      return 0;
    }
    const state = stateLib.readState(root);
    const pipeline = stateLib.resolvePipeline(root, { sessionId: getSessionId(ctx.payload) });
    const phase = (pipeline.ok ? pipeline.record.phase : state.phase) || "idle";
    const agentName = getAgentName(ctx.payload);
    const workers = Array.isArray(state.workers) ? state.workers : [];
    const isRegisteredWorker =
      agentName !== "" && workers.some((entry) => workerName(entry) === agentName);

    let msg = null;
    if (phase === "reviewing") {
      msg =
        "bee chain-nudge: a review agent finished. Collect its findings report, " +
        "score severities independently (P1/P2/P3), and when all reviewers are done " +
        "synthesize findings (corroboration promotes one level; disagreements go " +
        "conservative), then present Gate 4.";
    } else if (isRegisteredWorker || phase === "swarming") {
      const who = agentName ? `Worker "${agentName}"` : "A bee worker";
      msg =
        `bee chain-nudge: ${who} returned - collect its [STATUS] token ` +
        "([DONE]/[BLOCKED]/[HANDOFF]/[NOOP]), update the cell " +
        "(node .bee/bin/bee.mjs cells), and check/release its reservations " +
        "(node .bee/bin/bee.mjs reservations list --active-only). " +
        "When the wave is clean, move to the next wave or the next chain step.";
      // Decision 0011: capture-mode spine — if behavior_change cells capped since
      // the last scribing run, nudge capture in-flight, not only at feature close.
      try {
        const cellsLib = await import(libModuleUrl(root, "cells.mjs"));
        const debt = cellsLib.scribingDebt(root);
        if (debt && debt.count > 0) {
          msg +=
            `\n⚠ Scribing debt: ${debt.count} behavior_change cell(s) capped since the last capture ` +
            `(${debt.cells.join(", ")}) — run bee-scribing capture now; don't wait for review (decision 0011).`;
        }
      } catch {
        // fail-open: the debt nudge is advisory, never a blocker
      }
    }
    // else: not a bee-managed subagent -> silent.
    if (msg) {
      // This wrapper is only wired to SubagentStop, so a payload missing
      // hook_event_name still encodes as an advisory.
      emitHookOutput(ctx, msg, { defaultEvent: "SubagentStop" });
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
  return 0;
}

process.exitCode = await main();
