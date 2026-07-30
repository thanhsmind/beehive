#!/usr/bin/env node
// bee-state-sync: PostToolUse (update_plan|TaskCreate|TaskUpdate|TodoWrite) + SubagentStop + Stop.
// Refreshes cell status counts and last_activity into .bee/state.json so state
// stays fresh as a side effect of working. Always silent — it never emits
// stdout, so the Codex SubagentStop/Stop JSON-output requirement is satisfied
// by silence (cell codex-parity-3, decision D2).
// Input/root/logging go through the shared runtime adapter (hooks/adapter.mjs):
// stdin is normalized before any property access and root discovery lives
// inside the fail-open boundary.
// Fail-open: any miss or crash -> exit 0 (crash logged to .bee/logs/hooks.jsonl).

import fs from "node:fs";
import path from "node:path";
import crypto from "node:crypto";
import { readHookContext, logCrash, libModuleUrl } from "./adapter.mjs";
// exec-speed D5 (es-5): fsutil.mjs is import-light (fs/path/crypto only, no
// cells.mjs/lock.mjs/state-projection.mjs) — safe to import unconditionally
// on the fast path below without pulling in the heavy store machinery this
// cell exists to skip.
import { readJson, writeJsonAtomic } from "../lib/fsutil.mjs";

const HOOK_NAME = "state-sync";

// exec-speed D5 (es-5) — cheap stat-only debounce for the listCells()
// scan + rebuildStateProjection below. Mirrors the EXACT file set
// lib/cells.mjs's listCells(root, {}) scans (top-level .bee/cells/*.json;
// any directory entry, including `archive`, is explicitly skipped — same
// discipline listCells itself documents) but never reads/parses a single
// cell's JSON: stat only. A stamp is {count, newestMtimeMs, namesHash};
// namesHash also catches a same-count add+delete swap that mtime/count
// alone would miss.
function cellsDirPath(root) {
  return path.join(root, ".bee", "cells");
}

function stateSyncStampPath(root) {
  return path.join(root, ".bee", "logs", "state-sync.stamp.json");
}

function computeCellsStamp(root) {
  const dir = cellsDirPath(root);
  let entries;
  try {
    entries = fs.readdirSync(dir, { withFileTypes: true });
  } catch {
    entries = [];
  }
  const names = [];
  let newestMtimeMs = 0;
  for (const entry of entries) {
    if (entry.isDirectory()) continue; // skip `archive` (or any dir), same as listCells
    if (!entry.name.endsWith(".json")) continue;
    names.push(entry.name);
    try {
      const stat = fs.statSync(path.join(dir, entry.name));
      if (stat.mtimeMs > newestMtimeMs) newestMtimeMs = stat.mtimeMs;
    } catch {
      // vanished between readdir and stat — best-effort; the next run's own
      // readdir will simply omit it, which already changes count/namesHash
      // and forces the full path then.
    }
  }
  names.sort();
  const namesHash = crypto.createHash("sha256").update(names.join("\n")).digest("hex");
  return { count: names.length, newestMtimeMs, namesHash };
}

function stampsEqual(a, b) {
  return (
    a &&
    b &&
    typeof a.count === "number" &&
    typeof b.newestMtimeMs === "number" &&
    typeof b.namesHash === "string" &&
    a.count === b.count &&
    a.newestMtimeMs === b.newestMtimeMs &&
    a.namesHash === b.namesHash
  );
}

// Fail-open by design (must_have): a stamp-write failure only costs one
// extra full-path run next time — it must never surface as a hook crash or
// block the store refresh that already succeeded.
function writeStateSyncStamp(root, stamp) {
  try {
    writeJsonAtomic(stateSyncStampPath(root), stamp);
  } catch {
    // see comment above
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
    const stateLib = await import(libModuleUrl(root, "state.mjs"));
    if (!stateLib.hookEnabled(root, HOOK_NAME)) {
      return 0;
    }

    // D5 — throttled heartbeat + claim/hold lease renewal, same contract as
    // bee-prompt-context.mjs: session id off the hook payload, own try/catch
    // so a throw here never blocks this hook's primary job (the state
    // counts/last_activity refresh below).
    //
    // multisession-native-21b: sessions/claims are control-plane (msn-18c,
    // hooks/adapter.mjs's own ctx.controlRoot comment) — heartbeatTouch's
    // `root` argument must resolve against ctx.controlRoot (always MAIN for
    // a linked worktree, grant-independent), never the bare `root` this
    // read from before. A granted worktree's own local `.bee` has no
    // record for this session at all, so the bare-root call silently
    // renewed nothing: MAIN's heartbeat went stale and a live write-owner
    // became reclaimable as dead (isOwnerLive / guard class c read that
    // stale heartbeat). `ctx.controlRoot` is byte-identical to `root` for
    // every ordinary/solo checkout (same fallback adapter.mjs's own
    // readHookContext already guarantees), so this is a no-op there.
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

          // hardening-1-7-10 (D3) — also refresh this session's CROSS-WORKTREE
          // ledger holds (worktree-holds.mjs's renewHolds), same try-once/
          // never-block posture as the local renewal above: a missed
          // renewal is skipped, never waited on, never escalated past this
          // hook's own fail-open try/catch. The ledger always lives in the
          // MAIN checkout's store (never this checkout's own `.bee/`), so
          // mainRoot is resolved the same way bee.mjs's private
          // resolveMainRoot does — ordinary checkout: mainRoot is `root`
          // itself; linked worktree (granted or not): resolveRoots' own
          // `mainRoot` field, independent of THIS checkout's own grant
          // state. Duplicated here rather than imported from bee.mjs
          // because bee.mjs is the CLI entry point, not a lib module a hook
          // imports.
          try {
            let mainRoot = root;
            const resolution = stateLib.resolveRoots(process.cwd());
            if (resolution.worktreeResolution === "linked-valid" && resolution.mainRoot) {
              mainRoot = resolution.mainRoot;
            }
            const holdsLib = await import(libModuleUrl(root, "worktree-holds.mjs"));
            await holdsLib.renewHolds(mainRoot, sessionId, { maxAttempts: 1 });
          } catch (error) {
            logCrash(root, HOOK_NAME, error, ctx.source);
          }
        }
      } catch (error) {
        logCrash(root, HOOK_NAME, error, ctx.source);
      }
    }

    // exec-speed D5 (es-5) — the debounce gate: unchanged AND valid stamp
    // means the cells store cannot have moved since the last full sync, so
    // the scan + rebuild below (and their imports of cells.mjs/lock.mjs/
    // state-projection.mjs) are skipped entirely. A missing or corrupt
    // stamp reads as "unknown", never "unchanged" — it falls through to the
    // full path below, same as any other tolerant reader in this codebase.
    const stamp = computeCellsStamp(root);
    const previousStamp = readJson(stateSyncStampPath(root), null);
    if (stampsEqual(stamp, previousStamp)) {
      return 0;
    }

    const cellsLib = await import(libModuleUrl(root, "cells.mjs"));

    const counts = { open: 0, claimed: 0, capped: 0, blocked: 0 };
    for (const cell of cellsLib.listCells(root, {})) {
      if (cell && typeof cell.status === "string" && counts[cell.status] !== undefined) {
        counts[cell.status] += 1;
      }
    }

    // D3-amended: this hook's own state read-modify-write is a store write
    // too — an unlocked hook write racing a locked CLI write would reintroduce
    // the exact lost-update D2 exists to kill. Same try-once/skip-on-busy
    // contract as the touch above: LOCK_BUSY skips this sync silently
    // (fail-open preserved), never waited on, never escalated to a crash log.
    //
    // multisession-native-7 (F8, binding): this write is now a FULL
    // idempotent rebuild via state-projection.mjs's rebuildStateProjection —
    // never a partial patch of just `cells`/`last_activity` on top of
    // whatever readState happens to return. rebuildStateProjection recomputes
    // every D1-owned field (phase/feature/mode/approved_gates/summary/
    // next_action) from the newest ACTIVE workflow record in the SAME write
    // as this hook's own cells/last_activity refresh — but ONLY while
    // state.json is itself idle (no live default feature); zero workflow
    // records anywhere (C1) OR a live non-idle default feature (C5 — the
    // default mutation path does not keep its workflow record in sync until
    // msn-10) both leave those D1 fields exactly as they already are, and
    // only refresh cells/last_activity — byte-identical to this hook's
    // pre-msn-7 behavior for every repo mid-feature on the default pipeline,
    // which in practice is most of the time this hook runs. This hook
    // reads/writes .bee/state.json ONLY through that function — it never
    // calls createWorkflow/updateWorkflow, so it never writes a workflow
    // record (must_have: "bee-state-sync never writes workflow records").
    const lockLib = await import(libModuleUrl(root, "lock.mjs"));
    const stateProjectionLib = await import(libModuleUrl(root, "state-projection.mjs"));
    try {
      await lockLib.withStoreLock(
        root,
        "state",
        () => {
          stateProjectionLib.rebuildStateProjection(root, {
            cellCounts: counts,
            lastActivity: new Date().toISOString(),
          });
        },
        { maxAttempts: 1 },
      );
      // exec-speed D5 (es-5): stamp only what actually got refreshed — a
      // LockBusyError below skips this, so a busy run leaves the stamp
      // stale on purpose and the NEXT invocation retries the full path
      // instead of falsely believing this run already synced.
      writeStateSyncStamp(root, stamp);
    } catch (error) {
      if (!(error instanceof lockLib.LockBusyError)) throw error;
    }
  } catch (error) {
    logCrash(root, HOOK_NAME, error, ctx.source);
    return 0;
  }
  return 0;
}

process.exitCode = await main();
