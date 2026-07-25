#!/usr/bin/env node
// bee-prompt-context: UserPromptSubmit.
// Injects a 1-3 line phase/mode/next-action/gate reminder, deduped via the
// injection cache (only when state changed or >30 min since last injection).
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs,
// cell codex-parity-3, decision D2): stdin is normalized before any property
// access and root discovery lives inside the fail-open boundary.
// UserPromptSubmit stdout stays plain developer context on both hosts.
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).

import fs from "node:fs";
import path from "node:path";
import { readHookContext, logCrash, libModuleUrl } from "./adapter.mjs";

const HOOK_NAME = "prompt-context";

// compaction-hardening cz-7 (D10/D11): the deduped half of the anchor nudge.
// D10 closes the gap the intent-anchor feature recorded against itself —
// nothing yet PROMPTED an agent to write an anchor, and an anchor that is never
// written protects nothing. This is the surface that asks for it during
// ordinary work; hooks/bee-session-close.mjs asks again, forced, at the one
// moment it is about to matter.
//
// THIN CALLER, BY DECISION (D3): the predicate, the message, the cache key and
// the dedup hash (`<sessionId>:<feature>:<cell>`) all come from
// lib/compaction.mjs and are reachable without any hook through
// `bee.mjs state compact-check`. This function owns only the dedup gate —
// `anchorMissing` is pure and performs none itself — and it uses the SAME
// shouldInject/markInjected cache every other nudge in this repo uses
// (30-minute interval), so a session is asked once per interval and not once
// per turn.
//
// Its own try/catch, separate from the reminder logic: a throw here must never
// cost the hook its primary job.
async function maybeAnchorNudge(root, sessionId, inject) {
  try {
    const compaction = await import(libModuleUrl(root, "compaction.mjs"));
    const nudge = compaction.anchorMissing(root, { sessionId });
    if (!nudge) {
      return null;
    }
    if (!inject.shouldInject(root, nudge.key, nudge.hash)) {
      return null;
    }
    inject.markInjected(root, nudge.key, nudge.hash);
    return nudge.message;
  } catch (error) {
    logCrash(root, HOOK_NAME, error, "anchor-nudge");
    return null;
  }
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

    // D5 — throttled heartbeat + claim/hold lease renewal. Session id comes
    // straight off the hook payload (bee-session-init.mjs:49-51 pattern),
    // never handed down. Wrapped in its OWN try/catch, separate from the
    // reminder logic below: a throw here must never block the hook's
    // primary job (printing the reminder) — the outer catch alone would
    // abort that too if this ran unguarded inside it.
    //
    // multisession-native-21b: same control-plane fix as bee-state-sync.mjs
    // — heartbeatTouch's `root` argument must resolve against
    // ctx.controlRoot (always MAIN for a linked worktree), never the bare
    // `root`. Byte-identical to before for every ordinary/solo checkout.
    const sessionId =
      typeof ctx.payload.session_id === "string" && ctx.payload.session_id.trim()
        ? ctx.payload.session_id.trim()
        : null;
    if (sessionId) {
      try {
        const claims = await import(libModuleUrl(root, "claims.mjs"));
        const touch = await claims.heartbeatTouch(ctx.controlRoot, sessionId);
        if (touch && touch.touched) {
          const reservations = await import(libModuleUrl(root, "reservations.mjs"));
          await reservations.renewHoldsBySession(root, sessionId, { lockOptions: { maxAttempts: 1 } });
        }
      } catch (error) {
        logCrash(root, HOOK_NAME, error, ctx.source);
      }
    }

    const inject = await import(libModuleUrl(root, "inject.mjs"));
    // P0 (codex-loop-p0): pass the resolved sessionId so a session bound to a
    // feature lane sees ITS OWN phase/gate/next-action, not the default state's.
    // buildPromptReminder already threads {sessionId} through resolvePipeline;
    // dropping it here made every bound session read the default record (usually
    // idle), which pulled it back toward hive — a top driver of the looping.
    // cz-7 (D10): the reminder and the nudge are INDEPENDENT surfaces with
    // independent dedup keys, so an empty or throttled reminder must not
    // swallow the nudge — hence parts, and no early return above it. With no
    // nudge to add, the bytes written are exactly the reminder's, as before.
    const parts = [];
    const reminder = inject.buildPromptReminder(root, { sessionId });
    if (reminder && reminder.text && String(reminder.text).trim()) {
      // codex-loop (advisor #54): the dedup key was the repo-global string
      // "prompt", so two sessions working the same checkout each invalidated the
      // other's last-injected hash — turning the 30-minute throttle into a
      // reminder on nearly every turn. Key it by the acting session (falling back
      // to the global key only when no session id is available, which preserves
      // today's single-session behaviour exactly).
      const injectKey = sessionId ? `prompt:${sessionId}` : "prompt";
      if (inject.shouldInject(root, injectKey, reminder.hash)) {
        parts.push(String(reminder.text));
        inject.markInjected(root, injectKey, reminder.hash);
      }
    }
    const nudge = await maybeAnchorNudge(root, sessionId, inject);
    if (nudge) {
      parts.push(nudge);
    }
    if (parts.length > 0) {
      // UserPromptSubmit stdout stays PLAIN developer context on both hosts
      // (adapter.mjs: it is a context event, never an advisory) — so this is a
      // direct write, never emitHookOutput's JSON envelope.
      process.stdout.write(parts.join("\n\n"));
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
  return 0;
}

process.exitCode = await main();
