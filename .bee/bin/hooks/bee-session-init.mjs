#!/usr/bin/env node
// bee-session-init: SessionStart (startup|resume|clear|compact).
// Prints the bee session preamble (status, gates, HANDOFF surfacing, patterns,
// decisions) built by the target repo's own .bee/bin/lib/inject.mjs.
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs,
// cell codex-parity-3, decision D2): stdin is normalized before any property
// access and root discovery lives inside the fail-open boundary.
// SessionStart stdout stays plain developer context on both hosts.
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).
//
// fresh-session-handoff fsh-10 (D1, D4): this is the ONLY place a
// planned-next handoff is ever adopted — buildSessionPreamble stays a PURE
// renderer (PURITY PIN, panel W2); the mutation lives here.
//   1. Register/refresh the acting session record from payload.session_id
//      (createSession-or-heartbeat via the repo's vendored claims.mjs).
//      Fail-open: a registration failure never blocks the preamble.
//   2. EVENT-SCOPE PIN (panel W1): the adopt+start-now path runs ONLY when
//      payload.source is "clear" or "startup" (the fresh-session boundaries
//      D1 names) — on "resume"/"compact" this hook NEVER calls adoptHandoff,
//      a planned-next handoff stays on disk and renders as a pending-wait
//      block. A "startup" whose handoff.writer_session equals the acting
//      session is ALSO refused without adopting: that is not a fresh-session
//      boundary, just the same session starting up again.
//   3. When the source qualifies, adoptHandoff(sessionId)'s typed outcome is
//      passed into buildSessionPreamble, never mutated further here.
import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl } from "./adapter.mjs";

const HOOK_NAME = "session-init";
const ADOPT_SOURCES = new Set(["clear", "startup"]);

// compaction-hardening cz-6 (D6/D8): the ONE source whose orientation trims to
// the compact capsule. `startup`, `clear` and `resume` keep today's full
// preamble, byte-identical — `resume` deliberately included, because a resume
// is a fresh human-initiated return where startup orientation is exactly what
// is wanted; only `compact` is the mid-work interruption.
//
// Deliberately NOT ANCHOR_LEAD_SOURCES below, which also carries `resume`:
// leading with the anchor and trimming the body are two different questions
// with two different answers, and collapsing them would trim `resume` too.
const CAPSULE_SOURCE = "compact";

// intent-anchor ia-1 (D4): the sources that mean "this same task is
// continuing after its context was compressed". On those, and ONLY those, the
// emitted block LEADS with the intent anchor and the phase/workflow detail
// comes after it. The ordering IS the fix — the whole defect is that a
// compacted session re-anchors on bee's own bookkeeping (which is on disk and
// comes back intact) instead of on the objective (which is not).
//
// This set is deliberately DISJOINT from ADOPT_SOURCES above and changes
// nothing about handoff adoption: a compacted session still never auto-adopts
// a planned-next handoff. The anchor is a prefix, never a router.
const ANCHOR_LEAD_SOURCES = new Set(["compact", "resume"]);

// Read + render the anchor for a compact/resume start. Returns "" for every
// other source, for a missing/corrupt anchor, for a repo vendored before this
// shipped, and for any failure at all — D5: with no anchor, this hook's
// output is byte-identical to what it printed before the feature existed.
async function intentLeadBlock(root, eventSource, sessionId) {
  if (!ANCHOR_LEAD_SOURCES.has(eventSource)) {
    return "";
  }
  try {
    const intent = await import(libModuleUrl(root, "intent.mjs"));
    const anchor = intent.readIntent(root, { sessionId });
    if (!anchor) {
      return "";
    }
    return intent.resumeBlock(anchor) || "";
  } catch {
    // fail-open: the preamble is orientation, never a place to fail a session.
    return "";
  }
}

// The body under the anchor: the compact capsule on `source=compact`, today's
// full session preamble on every other source (D6/D8).
//
// THIN CALLER, BY DECISION (D3). Nothing here decides what a capsule contains,
// what a compaction record holds, or how the counts work — all of that lives in
// lib/compaction.mjs, reachable without any hook at all through
// `bee.mjs state compact-capsule` and `state compact-log`. This function knows
// exactly two things: which source trims, and that `handoffOutcome` — already
// computed by main() and already handed to buildSessionPreamble — must be
// carried into the capsule too (D27). Dropping it there is invisible to every
// suite in this feature and to the full verify, while a compacted session
// holding a planned-next handoff silently loses the `- Adoption not applied:`
// line that says WHY it is being told to wait.
async function sessionBody(root, eventSource, { sessionId, handoffOutcome }) {
  const inject = await import(libModuleUrl(root, "inject.mjs"));
  if (eventSource === CAPSULE_SOURCE) {
    try {
      const compaction = await import(libModuleUrl(root, "compaction.mjs"));
      // D5: SessionStart(compact) IS the `resume` event — the other half of
      // the PreCompact record, and the one that proves the session came back.
      // The append is fail-open inside the module (D4); it never affects what
      // renders below.
      compaction.appendCompactionRecord(root, { event: "resume", sessionId });
      return String(compaction.buildCompactCapsule(root, { sessionId, handoffOutcome }));
    } catch (error) {
      // fail-open: a repo vendored before compaction.mjs shipped still gets
      // its orientation. Orientation is never a place to fail a session.
      logCrash(root, HOOK_NAME, error, "session-init-capsule");
    }
  }
  return String(inject.buildSessionPreamble(root, { sessionId, handoffOutcome }));
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

    const sessionId =
      typeof ctx.payload.session_id === "string" && ctx.payload.session_id.trim()
        ? ctx.payload.session_id.trim()
        : null;
    const eventSource =
      typeof ctx.payload.source === "string" && ctx.payload.source.trim()
        ? ctx.payload.source.trim()
        : "";

    if (sessionId) {
      try {
        const claims = await import(libModuleUrl(root, "claims.mjs"));
        // msn-19 (CONTEXT.md D2/D3): sessions/claims are control-plane —
        // resolveContext(root) gives BOTH the re-root target (controlRoot,
        // same fix adapter.mjs's own ctx.controlRoot documents as "wiring
        // the individual hook consumers... is out of [that] cell's file
        // scope — a later cell wires them to [this]", i.e. here) and the
        // workspaceId to stamp onto the session record — one call instead of
        // two, since resolveContext already computes both from the same
        // git-worktree classification. Fail-open: an unresolvable context
        // (state.resolveContext's own "give up" case) falls back to `root`
        // itself and a null workspaceId, never a throw — this whole block is
        // already inside the try/catch that treats registration/heartbeat
        // failures as non-fatal to the preamble.
        const context = state.resolveContext(root);
        const sessionRoot = context.controlRoot || root;
        const workspaceId = typeof context.workspaceId === "string" ? context.workspaceId : undefined;
        // hardening-1-7-10 D5 (Codex session bridge): persist the real hook
        // payload's transcript_path onto the session record at creation time
        // (createSession itself omits it when absent/blank) so recovery.mjs
        // can resolve THIS session's transcript from the stored path directly
        // instead of guessing via Claude's encoded-layout math — the piece
        // that makes Codex transcript resolution real rather than a
        // relabeled Claude layout.
        const transcriptPath =
          typeof ctx.payload.transcript_path === "string" ? ctx.payload.transcript_path : undefined;
        const created = claims.createSession(sessionRoot, {
          id: sessionId,
          transcript_path: transcriptPath,
          workspace_id: workspaceId,
        });
        if (!created.ok) {
          claims.heartbeatSession(sessionRoot, sessionId);
        }
        // msn-19: "Main checkout auto-registered lazily (first touch)" — the
        // session-init hook is the natural first-touch point for BOTH the
        // main checkout and a linked worktree (a worktree created via
        // `createFeatureWorktree` already registered itself; this call is
        // then an idempotent no-op — see workspace-store.mjs's own header
        // for why registerWorkspace never errors or overwrites on an
        // existing record). Best-effort: never blocks the preamble.
        try {
          const workspaceStore = await import(libModuleUrl(root, "workspace-store.mjs"));
          if (workspaceId) {
            await workspaceStore.registerWorkspace(sessionRoot, {
              id: workspaceId,
              type: context.worktreeId ? "worktree" : "main",
              root: context.workspaceRoot || root,
            });
          }
        } catch (error) {
          logCrash(root, HOOK_NAME, error, ctx.source);
        }
      } catch (error) {
        // fail-open: registration/heartbeat never blocks the preamble.
        logCrash(root, HOOK_NAME, error, ctx.source);
      }
    }

    let handoffOutcome = null;
    if (sessionId) {
      const handoff = state.readHandoff(root);
      if (handoff && handoff.kind === "planned-next") {
        if (!ADOPT_SOURCES.has(eventSource)) {
          handoffOutcome = {
            ok: false,
            code: "WRONG_SOURCE",
            reason: `a planned-next handoff never auto-adopts on source "${eventSource || "unknown"}" — only "clear"/"startup" qualify (D1).`,
          };
        } else if (eventSource === "startup" && handoff.writer_session === sessionId) {
          handoffOutcome = {
            ok: false,
            code: "SAME_SESSION_STARTUP",
            reason:
              "the acting session is the same session that wrote this handoff — not a fresh-session boundary, never self-adopted.",
          };
        } else {
          try {
            handoffOutcome = state.adoptHandoff(root, sessionId);
          } catch (error) {
            // fail-open: an adoption crash never blocks the preamble; render
            // as if adoption were never attempted (today's wait block).
            logCrash(root, HOOK_NAME, error, ctx.source);
            handoffOutcome = null;
          }
        }
      }
    }

    const body = await sessionBody(root, eventSource, { sessionId, handoffOutcome });
    // intent-anchor ia-1 (D4/D5): PREFIX ONLY, and UNCHANGED by cz-6 (D19).
    // The anchor is prepended ahead of the body on a compact/resume start, and
    // with no anchor the emitted string is the body itself. cz-6 changed which
    // body renders on `compact` (D6) and changed nothing here: the anchor is
    // still rendered exactly once, by this hook, and the capsule never renders
    // it — under capsule ownership it would print twice and `startsWith` would
    // hold either way.
    const anchorBlock = await intentLeadBlock(root, eventSource, sessionId);
    const output = anchorBlock ? `${anchorBlock}\n\n${body}` : body;
    if (output && output.trim()) {
      process.stdout.write(output);
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
  return 0;
}

process.exitCode = await main();
