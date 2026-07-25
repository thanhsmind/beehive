#!/usr/bin/env node
// bee-session-close: Stop + PreCompact.
// The "hive door open" check: if the session ends mid-phase with no
// .bee/HANDOFF.json, warn with claimed-but-uncapped cells and active
// reservations, plus the decision/capture/capture-queue nudges. Never blocks;
// always exits 0.
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs,
// cell codex-parity-3, decision D2). Stop and PreCompact are advisory events:
// all messages for one invocation are collected and emitted as ONE parseable
// JSON systemMessage (Codex ignores plain PreCompact stdout and requires JSON
// for non-empty Stop stdout; multiple raw writes would not parse). Never
// decision:"block" — that would loop the main turn instead of advising.
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).

import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl, emitHookOutput, encodeBlock } from "./adapter.mjs";

const HOOK_NAME = "session-close";

// The one phase → pending-gate mapping the mechanical bypass net acts on. Gate 1
// (exploring) is DELIBERATELY absent: under `total`, genuine information questions
// still stop for the human (routing-and-contracts.md, decision a93994d3) — only
// approval gates are mechanized. planning → Gate 2 (shape); validating → Gate 3
// (execution).
const PHASE_GATE = Object.freeze({ planning: "shape", validating: "execution" });

// Which bypass levels cover a pending gate for a given lane/mode. `full`/`total`
// lifted the high-risk floor, so they cover every lane; `normal` covers only the
// non-hard-gate lanes (mode already encodes the hard-gate floor — a hard-gate
// change is classified high-risk, never tiny/small/standard).
const NORMAL_COVERED_MODES = Object.freeze(["tiny", "small", "standard"]);
function levelCoversGate(level, mode) {
  if (level === "total" || level === "full") return true;
  if (level === "normal") return NORMAL_COVERED_MODES.includes(mode);
  return false; // off
}

// Repository-harness lesson: review the session for an unrecorded decision
// before it ends. When source files changed with no bee flow active and no
// recent decision logged, nudge once (deduped) — never block.
const NUDGE_ALLOWED = /^(\.bee\/|docs\/|plans\/|AGENTS\.md$)/;
const DECISION_RECENT_MS = 6 * 3600 * 1000;

// fresh-session-handoff fsh-6 (D4): a bound session's PHASE comes from its
// lane via resolvePipeline; no session_id, an unbound session, or an
// unresolvable binding all fall back to the default record — this hook is
// advisory only and must never let a lane-resolution gap block the "hive
// door open" warning (fail-open, matching the file's own documented
// discipline).
function getSessionId(payload) {
  return typeof payload.session_id === "string" && payload.session_id.trim()
    ? payload.session_id.trim()
    : null;
}

async function maybeDecisionNudge(root) {
  try {
    const { execSync } = await import("node:child_process");
    const out = execSync("git status --porcelain", {
      cwd: root,
      timeout: 3000,
      stdio: ["ignore", "pipe", "ignore"],
    }).toString();
    const changed = out
      .split("\n")
      .map((line) => line.slice(3).trim().replace(/^"|"$/g, ""))
      .filter(Boolean)
      .filter((p) => !NUDGE_ALLOWED.test(p));
    if (changed.length === 0) {
      return null;
    }
    const decisionsLib = await import(libModuleUrl(root, "decisions.mjs"));
    const injectLib = await import(libModuleUrl(root, "inject.mjs"));
    const recent = decisionsLib.activeDecisions(root, { recent: 1 });
    const lastTs = recent[0] && recent[0].date ? Date.parse(recent[0].date) : 0;
    if (lastTs && Date.now() - lastTs < DECISION_RECENT_MS) {
      return null;
    }
    const hash = changed.sort().join("|");
    if (!injectLib.shouldInject(root, "decision-nudge", hash)) {
      return null;
    }
    injectLib.markInjected(root, "decision-nudge", hash);
    return (
      `bee decision review: ${changed.length} source file(s) changed with no bee flow active ` +
      "and no recent decision logged. Before finishing, ask the user: is there a durable " +
      'decision or convention here worth recording? If yes: node .bee/bin/bee.mjs decisions log ' +
      '--decision "..." --rationale "..." (or a dated learning in docs/history/learnings/). ' +
      "If not, carry on."
    );
  } catch {
    // fail-open: no git, no lib, no problem
    return null;
  }
}

// Decision 0003 capture nudge: a settled outcome must reach the state layer in
// the same session it settled. When the newest active decision is more recent
// than every docs/specs/*.md update AND every docs/knowledge/**/*.md concept
// (okf-foundation D34 — the bundle is the state layer's new home as areas
// migrate), warn (deduped) that something settled was never captured — invoke
// bee-scribing capture before closing. Never blocks.
async function maybeCaptureNudge(root) {
  try {
    // docs/specs/ and the bundle are both PRODUCT doc trees — resolve against
    // the product root so the nudge reads the real ones under the repo-divorce
    // topology, not the empty workshop-side copies (GitHub #14).
    const stateLib = await import(libModuleUrl(root, "state.mjs"));
    const productRoot = stateLib.resolveProductRoot(root);
    const specsDir = path.join(productRoot, "docs", "specs");
    // okf-foundation D34: knowledge migrates out of docs/specs/ into the bundle
    // area by area (legacy specs become pointer stubs whose mtime never moves
    // again). A capture that lands as a bundle concept under docs/knowledge/
    // must count as "captured", or this nudge fires forever once knowledge
    // moves. The bundle nests (areas/<slug>/, work/<id>/), so it is walked
    // recursively; the retired tree keeps its historical flat scan.
    const knowledgeDir = path.join(productRoot, "docs", "knowledge");
    if (!fs.existsSync(specsDir) && !fs.existsSync(knowledgeDir)) {
      return null;
    }
    const decisionsLib = await import(libModuleUrl(root, "decisions.mjs"));
    const injectLib = await import(libModuleUrl(root, "inject.mjs"));
    const recent = decisionsLib.activeDecisions(root, { recent: 1 });
    const lastDecision = recent[0];
    const decisionTs = lastDecision && lastDecision.date ? Date.parse(lastDecision.date) : 0;
    if (!decisionTs) {
      return null;
    }
    const newestMd = (dir, recursive) => {
      let newest = 0;
      if (!fs.existsSync(dir)) {
        return newest;
      }
      const stack = [dir];
      while (stack.length > 0) {
        const current = stack.pop();
        for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
          const abs = path.join(current, entry.name);
          if (entry.isDirectory()) {
            if (recursive) {
              stack.push(abs);
            }
            continue;
          }
          if (!entry.isFile() || !entry.name.endsWith(".md")) {
            continue;
          }
          const mtime = fs.statSync(abs).mtimeMs;
          if (mtime > newest) {
            newest = mtime;
          }
        }
      }
      return newest;
    };
    const newestSpec = Math.max(newestMd(specsDir, false), newestMd(knowledgeDir, true));
    if (decisionTs <= newestSpec) {
      return null;
    }
    const hash = String(lastDecision.id || lastDecision.date);
    if (!injectLib.shouldInject(root, "capture-nudge", hash)) {
      return null;
    }
    injectLib.markInjected(root, "capture-nudge", hash);
    // f4-6 (D3): the nudge NAMES THE RESOLVED TARGET. The logic above has been
    // bundle-aware since okf-foundation D34 — it maxes the retired tree against
    // the bundle — so in a migrated repo this fires precisely BECAUSE the
    // bundle is stale, and the old unconditional wording then sent the agent to
    // the wrong tree: "merge it into the touched area's spec" under a tree that
    // `scripts/okf_specs_fence.mjs` fails the chain for accepting new content.
    // An agent obeying the nudge would have been stopped by another guard. This
    // is the session-CLOSE twin of the session-INIT fix in f4-3 (inject.mjs's
    // scribing-debt nudge), and it routes on the same ONE predicate. The
    // no-bundle branch is byte-identical to what shipped before.
    const knowledgeLib = await import(libModuleUrl(root, "knowledge.mjs"));
    if (knowledgeLib.bundleMode(root)) {
      return (
        "bee capture nudge (decision 0003): the newest decision is more recent than every " +
        "concept in the knowledge bundle (docs/knowledge/) — a settled outcome may exist only " +
        "in the decision log and the chat. Before finishing, invoke bee-scribing capture to " +
        "author it as a concept in the touched area's bundle folder (or confirm no area is affected)."
      );
    }
    return ( // no-bundle branch: today's wording, byte for byte.
      "bee capture nudge (decision 0003): the newest decision is more recent than every " +
      "area spec under docs/specs/ — a settled outcome may exist only in the decision log " + // no-bundle branch
      "and the chat. Before finishing, invoke bee-scribing capture to merge it into the " +
      "touched area's spec (or confirm no spec is affected)."
    );
  } catch {
    // fail-open: no specs, no bundle, no lib, no problem
    return null;
  }
}

// Decision 0017: capture stubs queued mid-flow must not die with the context.
// On Stop the warning is deduped (same pending set warns once per interval);
// on PreCompact it always fires — compaction is the point where an unflushed
// queue would silently outlive the conversation that explains it.
async function maybeCaptureQueueNudge(root, { force = false } = {}) {
  try {
    const captureLib = await import(libModuleUrl(root, "capture.mjs"));
    const injectLib = await import(libModuleUrl(root, "inject.mjs"));
    const pending = captureLib.pendingCaptureStubs(root);
    if (pending.length === 0) {
      return null;
    }
    const hash = pending.map((stub) => stub.id).sort().join("|");
    if (!force) {
      if (!injectLib.shouldInject(root, "capture-queue-nudge", hash)) {
        return null;
      }
      injectLib.markInjected(root, "capture-queue-nudge", hash);
    }
    return (
      `bee capture queue (decision 0017): ${pending.length} settlement stub(s) are queued and ` +
      "unflushed. Flush them now via bee-scribing (drain oldest-first, merge each into its " +
      "area spec) — or they must survive into the next session's preamble, never be dropped."
    );
  } catch {
    // fail-open: no lib, no problem
    return null;
  }
}

// intent-anchor ia-1 (D3): the PreCompact re-assertion. Compaction is the
// exact moment the user's request is at greatest risk — it lives only in the
// conversation, which is what a summary compresses first, while every piece of
// bee's own scaffolding is on disk and comes back at full strength. So on the
// COMPACTION event only, the anchor block is pushed into the preserved context,
// labelled top and bottom so a summarizer cannot treat it as ordinary prose.
//
// It stays ADVISORY, deliberately and permanently: this hook's PreCompact
// output is emitted through emitHookOutput as a `systemMessage`, never
// encodeBlock — the B2/R14 contract (docs/knowledge/areas/hook-runtime/
// advisories-and-turn-control.md) forbids a turn-control verdict on
// compaction, and re-asserting an objective is advice, not steering.
//
// D5: with no anchor (or a missing/older vendored lib, or any failure at all)
// this returns null and every byte this hook emits is what it emitted before
// the feature shipped.
async function maybeIntentAnchor(root, sessionId) {
  try {
    const intent = await import(libModuleUrl(root, "intent.mjs"));
    const anchor = intent.readIntent(root, { sessionId });
    if (!anchor) {
      return null;
    }
    const block = intent.precompactBlock(anchor);
    return block && block.trim() ? block : null;
  } catch {
    // fail-open: no lib (a repo vendored before this shipped), no anchor, no problem
    return null;
  }
}

// compaction-hardening cz-7 (D4/D5/D9): the compaction record, and the D9
// survival advisory rendered off the record this compaction just wrote.
//
// THIN CALLER, BY DECISION (D3). Nothing here decides the record's shape, the
// counting rule, or the threshold — all of that lives in lib/compaction.mjs and
// is reachable without any hook at all through `bee.mjs state compact-log`.
// This function knows exactly two things: that a PreCompact is a `precompact`
// event, and that the returned record's `cell_compact_count` is what the
// advisory is rendered from.
//
// The COUNT IS TAKEN FROM THE RECORD THIS COMPACTION WROTE (D9), not from a
// separate read: the threshold is evaluated against the compaction NOW
// beginning, which is exactly what makes the warning first appear on a unit's
// SECOND compaction rather than its first.
//
// It stays ADVISORY, like every other part on this event: the string is
// collected into `parts` and emitted through emitHookOutput as a systemMessage,
// never encodeBlock — the B2/R14 contract documented at :234-238 above. D9 says
// the warning "never blocks", and on this event a blocking design is
// unimplementable rather than merely undesirable.
async function maybeCompactionRecord(root, sessionId) {
  try {
    const compaction = await import(libModuleUrl(root, "compaction.mjs"));
    const record = compaction.appendCompactionRecord(root, { event: "precompact", sessionId });
    const warning = compaction.survivalWarning(record.cell_compact_count);
    if (!warning) {
      return null;
    }
    return `⚠ bee compaction watch (D9): ${record.cell ? `cell ${record.cell} — ` : ""}${warning}.`;
  } catch {
    // fail-open: a repo vendored before compaction.mjs shipped, or any failure
    // at all, emits exactly what this hook emitted before the feature existed.
    return null;
  }
}

// compaction-hardening cz-7 (D10/D11): the anchor nudge. An anchor that is
// never written protects nothing, so the two orientation surfaces that see a
// session mid-work — this one and UserPromptSubmit — say so when work is active
// and no anchor exists.
//
// THE DEDUP IS THE CALLER'S, BOTH HALVES OF IT. `anchorMissing` is a pure
// predicate: it never reads shouldInject and never calls markInjected. So on
// the FORCED path (PreCompact) this function must skip BOTH — skipping only
// shouldInject would mark the cache and let a compaction swallow the deduped
// surface's next nudge. That is exactly maybeCaptureQueueNudge's shape at
// :201-225 above, where `force` skips the gate and the mark together.
//
// D11: forced on PreCompact deliberately — that is the last moment the
// objective can still be captured verbatim, and a 30-minute throttle must not
// swallow it.
async function maybeAnchorNudge(root, sessionId, { force = false } = {}) {
  try {
    const compaction = await import(libModuleUrl(root, "compaction.mjs"));
    const nudge = compaction.anchorMissing(root, { sessionId });
    if (!nudge) {
      return null;
    }
    if (!force) {
      const injectLib = await import(libModuleUrl(root, "inject.mjs"));
      if (!injectLib.shouldInject(root, nudge.key, nudge.hash)) {
        return null;
      }
      injectLib.markInjected(root, nudge.key, nudge.hash);
    }
    return nudge.message;
  } catch {
    // fail-open: no lib, no predicate, no problem
    return null;
  }
}

// maybePerfRefresh — keep the global performance data + matrix current WITHOUT
// the operator doing anything. Writes the just-ended session's rollup into the
// persistent log (performance.jsonl, upsert by session id — so Stop+PreCompact
// double-fire never duplicates), then redraws the HTML matrix by READING that
// log. Cheap: only the one current transcript is parsed; no cross-project scan.
// Wholly best-effort and fail-open: any failure is swallowed, never touching the
// exit code (critical-patterns 20260714 — a fail-open host must never let this
// throw).
async function maybePerfRefresh(root, sessionId) {
  try {
    const perf = await import(libModuleUrl(root, "perf.mjs"));
    const projectsRoot = perf.claudeProjectsRoot();
    const transcript = perf.resolveTranscript(projectsRoot, root, sessionId ? { sessionId } : {});
    if (transcript) {
      const rollup = perf.rollupTranscript(transcript);
      if (rollup) {
        perf.upsertSessionRecords([perf.sessionRecord(rollup)]);
      }
    }
    if (perf.readSessionRecords().length > 0) {
      perf.writeReport(perf.buildMatrixFromLog());
    }
  } catch (error) {
    try {
      logCrash(root, HOOK_NAME, error, "perf-refresh");
    } catch {
      // swallow: perf refresh is never allowed to affect the hook outcome.
    }
  }
}

// GitHub #18 — the mechanical gate-bypass net. Honoring `gate_bypass` was
// 100% prose-dependent: the level-aware rule lives in the planning/validating
// skills and is machine-guarded green by test_gate_bypass_doctrine.mjs, but
// NOTHING caught the model when it skipped the "check gate_bypass_level first"
// step and stopped at Gate 2/3 anyway. This IS crit-pattern 20260714 — "the
// invariant you leave in prose WILL be bypassed; mechanize it": the doctrine
// test mechanized the prose, this mechanizes the runtime.
//
// When the session tries to STOP mid-planning/validating with a gate the active
// bypass level should have auto-approved, return decision:"block" (via
// encodeBlock) so the turn CONTINUES, carrying an instruction to auto-approve
// and proceed. Deliberately narrow so it can only convert an illegitimate
// gate-stop into a continue, never trap a session:
//   - Stop event ONLY (ctx.event==="Stop" exactly; never PreCompact, never an
//     empty/missing event — those must stay advisory).
//   - phase ∈ {planning, validating} with that phase's gate still pending.
//   - the active level covers that gate for the lane (levelCoversGate).
//   - loop-guard: block ONCE per sessionId:phase:gate:level (inject dedup); an
//     immediate re-stop at the same gate degrades to advisory — never loops.
// Returns the block reason string when it fires, else null. Fail-open: any throw
// is swallowed by the caller's try/catch → advisory path / exit 0.
async function maybeBypassBlock(root, ctx) {
  // Stop-event only. An empty/missing event (wrapper also serves PreCompact)
  // must never block — we cannot prove it is a Stop, so fail safe to advisory.
  if (!ctx || ctx.event !== "Stop") {
    return null;
  }
  const stateLib = await import(libModuleUrl(root, "state.mjs"));
  const level = stateLib.bypassLevel(root);
  if (level === "off") {
    return null;
  }
  const sessionId = getSessionId(ctx.payload);
  const pipeline = stateLib.resolvePipeline(root, sessionId ? { sessionId } : {});
  const record = pipeline.ok ? pipeline.record : stateLib.readState(root);
  const phase = record.phase || "idle";
  const gate = PHASE_GATE[phase];
  if (!gate) {
    return null; // not a mechanized-gate phase (exploring/Gate 1 excluded on purpose)
  }
  const mode = record.mode || null;
  if (!levelCoversGate(level, mode)) {
    return null; // e.g. normal + high-risk lane still stops for the human
  }
  const approved = record.approved_gates && record.approved_gates[gate] === true;
  if (approved) {
    return null; // gate already passed — nothing to force
  }

  // Loop-guard: one forced block per (session, phase, gate, level). A same-key
  // re-stop is deduped → null → the advisory path runs instead of looping.
  const injectLib = await import(libModuleUrl(root, "inject.mjs"));
  const key = "bypass-stop-net";
  const hash = `${sessionId || "nosession"}:${phase}:${gate}:${level}`;
  if (!injectLib.shouldInject(root, key, hash)) {
    return null;
  }
  injectLib.markInjected(root, key, hash);

  const gateNo = gate === "shape" ? "2" : "3";
  // AO3/AO13: `state gate --name execution --approved true` refuses for
  // high-risk work without a non-stale advisor_ref (.bee/bin/bee.mjs
  // handleStateGate). Pre-precondition this instruction only named "set the
  // gate" and would steer the agent straight into that throw uninformed
  // (H3 / P3 friction). Prose only — the net's firing conditions, loop-guard,
  // and verdict shape are untouched; the sentence is empty (byte-identical
  // instruction) for every non-high-risk-execution case.
  const consultSentence =
    gate === "execution" && mode === "high-risk"
      ? 'High-risk execution requires a live advisor consult first: resolve the advisor from config (models.<runtime>.advisor), run it read-only with the evidence bundle on stdin, then record it via node .bee/bin/bee.mjs state advisor-ref record --advisor "<identity>" --digest-file <path> (the gate throws without a non-stale advisor_ref, per AO3/AO13) — do this BEFORE setting the gate. '
      : "";
  return (
    `⚡ GATE BYPASS (${level}): you are stopping mid-${phase} with Gate ${gateNo} ` +
    `(${gate}) still pending, but bypass level "${level}" requires auto-approval at ` +
    `this lane — do NOT ask the human. ${consultSentence}Set the gate yourself now: ` +
    `node .bee/bin/bee.mjs state gate --name ${gate} --approved true ; log a one-line ` +
    `audit decision (node .bee/bin/bee.mjs decisions log --decision "auto-approved Gate ` +
    `${gateNo} (bypass): <choice>" --rationale "<why>"); post the short "⚡ auto-approved ` +
    `Gate ${gateNo} (bypass)" line; then CONTINUE to the next phase. Do not re-emit the ` +
    `gate question. (If you genuinely need information only the human holds — not a ` +
    `rubber-stamp — ask that specific question instead; this net blocks once, then steps ` +
    `aside.)`
  );
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

  // Best-effort, fail-open: record this session + refresh the performance matrix.
  // Deliberately NOT in the advisory try/parts block — it emits nothing and must
  // never influence the warning path or the return value.
  await maybePerfRefresh(root, getSessionId(ctx.payload));

  const parts = [];
  try {
    const stateLib = await import(libModuleUrl(root, "state.mjs"));
    if (!stateLib.hookEnabled(root, HOOK_NAME)) {
      return 0;
    }
    // GitHub #18: the mechanical bypass net takes precedence over every advisory.
    // When it fires we FORCE the turn to continue (auto-approve + proceed), so the
    // "hive door open" warning is moot — emit only the block and return.
    const blockReason = await maybeBypassBlock(root, ctx);
    if (blockReason) {
      process.stdout.write(encodeBlock(blockReason));
      return 0;
    }
    // intent-anchor ia-1 (D3): FIRST in `parts` on the compaction event, so
    // the objective leads the advisory the summary sees. Compaction only —
    // Stop already ends the turn, and the anchor's job is to survive a
    // summary, not to close one.
    if (ctx.event === "PreCompact") {
      const sessionId = getSessionId(ctx.payload);
      const anchorMsg = await maybeIntentAnchor(root, sessionId);
      if (anchorMsg) {
        parts.push(anchorMsg);
      }
      // cz-7 (D10/D11): the anchor's ABSENCE leads the advisory in exactly the
      // place its presence would. The two are mutually exclusive by
      // construction — anchorMissing fires only when readIntent returns null,
      // which is the one case maybeIntentAnchor returns null — so this position
      // costs nothing and keeps the objective (or the demand to write one)
      // first, ahead of everything the summary sees. FORCED: never deduped,
      // and the cache is left untouched (D11).
      const nudgeMsg = await maybeAnchorNudge(root, sessionId, { force: true });
      if (nudgeMsg) {
        parts.push(nudgeMsg);
      }
      // cz-7 (D4/D5/D9): the record, then the advisory rendered off it.
      const survivalMsg = await maybeCompactionRecord(root, sessionId);
      if (survivalMsg) {
        parts.push(survivalMsg);
      }
    }
    const queueMsg = await maybeCaptureQueueNudge(root, {
      force: ctx.event === "PreCompact",
    });
    if (queueMsg) {
      parts.push(queueMsg);
    }
    const captureMsg = await maybeCaptureNudge(root);
    if (captureMsg) {
      parts.push(captureMsg);
    }
    const state = stateLib.readState(root);
    const pipeline = stateLib.resolvePipeline(root, { sessionId: getSessionId(ctx.payload) });
    const phase = (pipeline.ok ? pipeline.record.phase : state.phase) || "idle";
    if (phase === "idle" || phase === "compounding-complete") {
      const decisionMsg = await maybeDecisionNudge(root);
      if (decisionMsg) {
        parts.push(decisionMsg);
      }
    } else if (!stateLib.readHandoff(root)) {
      const cellsLib = await import(libModuleUrl(root, "cells.mjs"));
      const reservationsLib = await import(libModuleUrl(root, "reservations.mjs"));
      const claimed = cellsLib.listCells(root, { status: "claimed" });
      const active = reservationsLib.listReservations(root, { activeOnly: true });

      const lines = [
        `bee session-close warning: session is ending mid-phase (phase: ${phase}) ` +
          "with no .bee/HANDOFF.json. You are about to leave the hive door open.",
      ];
      if (claimed.length > 0) {
        lines.push(
          `Claimed-but-uncapped cells: ${claimed
            .map((cell) => `${cell.id}${cell.trace && cell.trace.worker ? ` (${cell.trace.worker})` : ""}`)
            .join(", ")}.`,
        );
      }
      if (active.length > 0) {
        lines.push(
          `Active reservations: ${active
            .map((r) => `${r.agent} -> ${r.path}${r.cell ? ` (cell ${r.cell})` : ""}`)
            .join("; ")}.`,
        );
      }
      lines.push(
        "Either finish and cap the work, or write .bee/HANDOFF.json and release " +
          "reservations so the next session can resume cleanly.",
      );
      parts.push(lines.join("\n"));
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    // fall through: advisory parts already collected are still emitted below,
    // matching the old behavior where earlier nudges had already printed.
  }
  if (parts.length > 0) {
    // This wrapper is only wired to Stop and PreCompact — both advisory —
    // so a payload missing hook_event_name still encodes as an advisory.
    emitHookOutput(ctx, parts.join("\n"), { defaultEvent: "Stop" });
  }
  return 0;
}

process.exitCode = await main();
