#!/usr/bin/env node
// test_claims.mjs — lib/claims.mjs contract tests (cross-session sessions +
// O_EXCL cell claims, resolveSessionId, sessionless claims, concurrent Worker
// races), split out of test_lib.mjs (cs-2a) to shrink the monolith. Same
// PASS/FAIL/exit-1 contract as every other suite here — see
// scripts/lib/test-fixture.mjs.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { runModuleWorker } from '../../../scripts/lib/run-module-worker.mjs';
import { check, assert, printSummaryAndExit } from '../../../scripts/lib/test-fixture.mjs';
import {
  createSession,
  readSession,
  heartbeatSession,
  bindSessionLane,
  unbindSessionLane,
  claimCellFile,
  readClaim,
  releaseClaim,
  clearClaim,
  adoptClaim,
  renewClaimTTL,
  sweepExpiredClaims,
  isClaimActive,
  isConcurrentMode,
  activeWorkers,
  listSessionRecords,
  sessionPath,
  claimPath,
  claimGatePath,
  resolveSessionId,
  DEFAULT_CLAIM_TTL_SECONDS,
  DEFAULT_HEARTBEAT_STALE_SECONDS,
} from '../lib/claims.mjs';
import { writeJsonAtomic } from '../lib/fsutil.mjs';
import { activeDecisions } from '../lib/decisions.mjs';

// Hermeticity (hardening-1-7-10 D1, defense in depth): this suite must never
// inherit the harness's own session identity. run_verify.mjs already scrubs
// both vars for every child suite it spawns; deleting them again here means
// a bare `node packages/bee/tests/test_claims.mjs`, run directly
// from inside a live Claude Code or bee session, is equally hermetic instead
// of silently depending on whatever session happens to be live. (The
// resolveSessionId rows below explicitly set/restore their own values around
// this — unaffected by this one-time bootstrap delete.)
delete process.env.CLAUDE_CODE_SESSION_ID;
delete process.env.BEE_SESSION_ID;

// ─── claims (cross-session sessions + O_EXCL cell claims) ───────────────────
// fsh-1 (fresh-session-handoff): single-process rows prove post-states and the
// typed {ok:false, code, reason} contract. The concurrency windows themselves
// are proven by the multi-process race fixtures (fsh-2); S1 caps as a unit.

const claimsRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-'));

await check('createSession writes .bee/sessions/<id>.json (id, started_at, last_heartbeat); duplicate id is a typed failure', async () => {
  const made = createSession(claimsRoot, { id: 'sess-a' });
  assert(made.ok === true, 'first createSession ok');
  assert(fs.existsSync(sessionPath(claimsRoot, 'sess-a')), 'session record exists on disk');
  const record = readSession(claimsRoot, 'sess-a');
  assert(record && record.id === 'sess-a', 'session record carries its id');
  assert(typeof record.started_at === 'string' && !Number.isNaN(Date.parse(record.started_at)), 'started_at is a timestamp');
  assert(typeof record.last_heartbeat === 'string' && !Number.isNaN(Date.parse(record.last_heartbeat)), 'last_heartbeat is a timestamp');
  const dup = createSession(claimsRoot, { id: 'sess-a' });
  assert(dup.ok === false && dup.code === 'SESSION_EXISTS' && typeof dup.reason === 'string', 'duplicate session id returns typed {ok:false, code, reason} — no throw');
  const generated = createSession(claimsRoot);
  assert(generated.ok === true && typeof generated.session.id === 'string' && generated.session.id.length > 0, 'id generated when omitted');
});

await check('heartbeatSession advances last_heartbeat; missing session is a typed SESSION_MISSING failure', async () => {
  const stale = new Date(Date.now() - 7200 * 1000).toISOString();
  const record = readSession(claimsRoot, 'sess-a');
  writeJsonAtomic(sessionPath(claimsRoot, 'sess-a'), { ...record, last_heartbeat: stale });
  const beat = heartbeatSession(claimsRoot, 'sess-a');
  assert(beat.ok === true, 'heartbeat ok');
  const after = readSession(claimsRoot, 'sess-a');
  assert(Date.parse(after.last_heartbeat) > Date.parse(stale), 'last_heartbeat advanced');
  const missing = heartbeatSession(claimsRoot, 'sess-ghost');
  assert(missing.ok === false && missing.code === 'SESSION_MISSING' && typeof missing.reason === 'string', 'missing session returns typed failure — no throw');
});

await check('claimCellFile: first claimant wins, second gets typed CLAIMED naming holder and expiry — no throw', async () => {
  createSession(claimsRoot, { id: 'sess-b' });
  const first = claimCellFile(claimsRoot, 'sess-a', 'cell-1', 60);
  assert(first.ok === true, 'first claim wins');
  assert(first.claim.cell === 'cell-1' && first.claim.session === 'sess-a', 'claim record carries cell + owner');
  assert(first.claim.ttl_seconds === 60, 'claim record carries ttl');
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-1')), 'claim file exists under .bee/claims/');
  const second = claimCellFile(claimsRoot, 'sess-b', 'cell-1', 60);
  assert(second.ok === false, 'second claim loses');
  assert(second.code === 'CLAIMED', `contention code is CLAIMED, got ${second.code}`);
  assert(typeof second.reason === 'string' && second.reason.includes('sess-a'), 'reason names the holder');
  assert(/expir/i.test(second.reason), 'reason names the expiry');
  assert(second.holder && second.holder.session === 'sess-a', 'holder record returned');
});

await check('claim and session records are repo-relative — no absolute or system-temp path inside', async () => {
  const claimText = fs.readFileSync(claimPath(claimsRoot, 'cell-1'), 'utf8');
  const sessionText = fs.readFileSync(sessionPath(claimsRoot, 'sess-a'), 'utf8');
  assert(!claimText.includes(claimsRoot), 'claim record must not embed the repo root path');
  assert(!sessionText.includes(claimsRoot), 'session record must not embed the repo root path');
  assert(!claimText.includes(os.tmpdir()), 'claim record must not embed a system temp path');
});

await check('isClaimActive reuses reservations TTL semantics: fresh claim active, TTL-expired claim inactive', async () => {
  const claim = readClaim(claimsRoot, 'cell-1');
  assert(isClaimActive(claim) === true, 'fresh claim is active');
  const expired = { ...claim, claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(), ttl_seconds: 60 };
  assert(isClaimActive(expired) === false, 'TTL-expired claim is inactive');
  assert(isClaimActive(null) === false, 'missing claim is not active');
});

await check('sweep: TTL expired but heartbeat FRESH is never reclaimed (20260710 — no steal on a stall signal)', async () => {
  // Backdate the claim past its TTL; owner sess-a heartbeat was just renewed above.
  heartbeatSession(claimsRoot, 'sess-a');
  const claim = readClaim(claimsRoot, 'cell-1');
  writeJsonAtomic(claimPath(claimsRoot, 'cell-1'), {
    ...claim,
    claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
    ttl_seconds: 60,
  });
  const result = await sweepExpiredClaims(claimsRoot);
  assert(result.ok === true, 'sweep returns ok');
  assert(!result.swept.includes('cell-1'), 'fresh-heartbeat claim not swept');
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-1')), 'claim file untouched');
});

await check('sweep: TTL expired AND heartbeat stale IS reclaimed; no gate file leaks', async () => {
  const session = readSession(claimsRoot, 'sess-a');
  writeJsonAtomic(sessionPath(claimsRoot, 'sess-a'), {
    ...session,
    last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
  });
  const result = await sweepExpiredClaims(claimsRoot);
  assert(result.swept.includes('cell-1'), `expired+stale claim swept, got ${JSON.stringify(result)}`);
  assert(!fs.existsSync(claimPath(claimsRoot, 'cell-1')), 'claim file reclaimed');
  assert(!fs.existsSync(claimGatePath(claimsRoot, 'cell-1')), 'gate file removed after sweep');
  heartbeatSession(claimsRoot, 'sess-a'); // restore a fresh heartbeat for later rows
});

await check('sweep and adopt skip/refuse while the per-claim gate is held — typed GATE_HELD, never wait', async () => {
  const claimed = claimCellFile(claimsRoot, 'sess-a', 'cell-2', 60);
  assert(claimed.ok === true, 'precondition: cell-2 claimed');
  writeJsonAtomic(claimPath(claimsRoot, 'cell-2'), {
    ...readClaim(claimsRoot, 'cell-2'),
    claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
  });
  writeJsonAtomic(sessionPath(claimsRoot, 'sess-a'), {
    ...readSession(claimsRoot, 'sess-a'),
    last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
  });
  fs.writeFileSync(claimGatePath(claimsRoot, 'cell-2'), '{}', 'utf8'); // another process mid-adopt
  const swept = await sweepExpiredClaims(claimsRoot);
  assert(!swept.swept.includes('cell-2'), 'gated claim skipped by sweep');
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-2')), 'gated claim untouched');
  const adopt = adoptClaim(claimsRoot, 'cell-2', 'sess-b');
  assert(adopt.ok === false && adopt.code === 'GATE_HELD' && typeof adopt.reason === 'string', 'adopt under a held gate is a typed GATE_HELD failure — no throw');
  fs.rmSync(claimGatePath(claimsRoot, 'cell-2'));
  heartbeatSession(claimsRoot, 'sess-a');
});

await check('adoptClaim rewrites the owner in place: old owner loses, new owner holds, claim file present throughout post-state', async () => {
  const before = readClaim(claimsRoot, 'cell-2');
  assert(before.session === 'sess-a', 'precondition: sess-a owns cell-2');
  const adopted = adoptClaim(claimsRoot, 'cell-2', 'sess-b');
  assert(adopted.ok === true, `adopt ok, got ${JSON.stringify(adopted)}`);
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-2')), 'claim file exists after adopt (never deleted)');
  const after = readClaim(claimsRoot, 'cell-2');
  assert(after.session === 'sess-b', 'new session owns the claim');
  assert(after.adopted_from === 'sess-a', 'adoption records the previous owner');
  assert(typeof after.adopted_at === 'string' && !Number.isNaN(Date.parse(after.adopted_at)), 'adoption timestamped');
  assert(after.cell === 'cell-2', 'cell id preserved in place');
  assert(!fs.existsSync(claimGatePath(claimsRoot, 'cell-2')), 'gate file removed after adopt');
  const missing = adoptClaim(claimsRoot, 'cell-ghost', 'sess-b');
  assert(missing.ok === false && missing.code === 'NOT_FOUND' && typeof missing.reason === 'string', 'adopting a missing claim is a typed NOT_FOUND failure');
});

await check('releaseClaim: NOT_OWNER for the old session after adoption, owner release removes the file, NOT_FOUND after', async () => {
  const denied = releaseClaim(claimsRoot, 'sess-a', 'cell-2');
  assert(denied.ok === false && denied.code === 'NOT_OWNER' && typeof denied.reason === 'string', 'old owner can no longer release — typed NOT_OWNER');
  assert(denied.reason.includes('sess-b'), 'NOT_OWNER reason names the actual owner');
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-2')), 'claim untouched by a denied release');
  const released = releaseClaim(claimsRoot, 'sess-b', 'cell-2');
  assert(released.ok === true, 'owner release ok');
  assert(!fs.existsSync(claimPath(claimsRoot, 'cell-2')), 'claim file removed on release');
  assert(!fs.existsSync(claimGatePath(claimsRoot, 'cell-2')), 'no gate file leaked by release');
  const gone = releaseClaim(claimsRoot, 'sess-b', 'cell-2');
  assert(gone.ok === false && gone.code === 'NOT_FOUND', 'releasing a missing claim is a typed NOT_FOUND failure');
});

await check('claimCellFile default TTL matches the exported constant; released cell is claimable again', async () => {
  const again = claimCellFile(claimsRoot, 'sess-b', 'cell-2');
  assert(again.ok === true, 'released cell claimable again');
  assert(again.claim.ttl_seconds === DEFAULT_CLAIM_TTL_SECONDS, 'default ttl applied');
  releaseClaim(claimsRoot, 'sess-b', 'cell-2');
});

// ─── D3: resolveSessionId — explicit flag -> BEE_SESSION_ID -> CLAUDE_CODE_SESSION_ID -> null ──

await check('resolveSessionId: explicit flag wins over env; a blank flag falls through to env; neither present resolves null', async () => {
  const savedEnv = process.env.CLAUDE_CODE_SESSION_ID;
  try {
    process.env.CLAUDE_CODE_SESSION_ID = 'sess-from-env';
    assert(resolveSessionId({ flag: 'sess-from-flag' }) === 'sess-from-flag', 'explicit flag takes precedence over env');
    assert(resolveSessionId({ flag: '' }) === 'sess-from-env', 'a blank flag falls through to env, not treated as an explicit empty session');
    assert(resolveSessionId({ flag: '   ' }) === 'sess-from-env', 'a whitespace-only flag falls through to env too');
    assert(resolveSessionId({}) === 'sess-from-env', 'no flag at all resolves from env');
    delete process.env.CLAUDE_CODE_SESSION_ID;
    assert(resolveSessionId({}) === null, 'neither flag nor env present resolves null');
    assert(resolveSessionId() === null, 'called with no argument at all still resolves null, never throws');
  } finally {
    if (savedEnv === undefined) delete process.env.CLAUDE_CODE_SESSION_ID;
    else process.env.CLAUDE_CODE_SESSION_ID = savedEnv;
  }
});

await check('resolveSessionId (hardening-4a): BEE_SESSION_ID wins over legacy CLAUDE_CODE_SESSION_ID env; explicit flag still wins over both; a blank BEE_SESSION_ID falls through to the legacy env', async () => {
  const savedBee = process.env.BEE_SESSION_ID;
  const savedLegacy = process.env.CLAUDE_CODE_SESSION_ID;
  try {
    delete process.env.BEE_SESSION_ID;
    process.env.CLAUDE_CODE_SESSION_ID = 'sess-legacy';
    assert(resolveSessionId({}) === 'sess-legacy', 'legacy env alone still resolves, runtime-neutral chain is additive');

    process.env.BEE_SESSION_ID = 'sess-bee';
    assert(resolveSessionId({}) === 'sess-bee', 'BEE_SESSION_ID wins over CLAUDE_CODE_SESSION_ID when both are set');
    assert(resolveSessionId({ flag: 'sess-flag' }) === 'sess-flag', 'explicit flag still wins over both envs');

    process.env.BEE_SESSION_ID = '   ';
    assert(resolveSessionId({}) === 'sess-legacy', 'a whitespace-only BEE_SESSION_ID falls through to CLAUDE_CODE_SESSION_ID, same blank-treated-as-absent rule as the flag');

    delete process.env.BEE_SESSION_ID;
    delete process.env.CLAUDE_CODE_SESSION_ID;
    assert(resolveSessionId({}) === null, 'neither env present still resolves null');
  } finally {
    if (savedBee === undefined) delete process.env.BEE_SESSION_ID;
    else process.env.BEE_SESSION_ID = savedBee;
    if (savedLegacy === undefined) delete process.env.CLAUDE_CODE_SESSION_ID;
    else process.env.CLAUDE_CODE_SESSION_ID = savedLegacy;
  }
});

// ─── D1 Δ2: sessionless claims — claimCellFile/releaseClaim/clearClaim ─────
// hardening-4a: this shared claimsRoot has accumulated LIVE session records
// from every row above (sess-a, sess-b, plus the anonymous generated-id
// session from the very first createSession row) — stale every one of them
// out here so the sessionless rows below genuinely exercise the SOLO case
// (isConcurrentMode(claimsRoot) === false), matching what these rows always
// meant to test (single-session use), predating concurrent mode entirely.
for (const record of listSessionRecords(claimsRoot)) {
  writeJsonAtomic(sessionPath(claimsRoot, record.id), {
    ...record,
    last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
  });
}

await check('claimCellFile(root, null, ...): a sessionless claim omits the "session" key entirely (never null) and still race-serializes via O_EXCL', async () => {
  const first = claimCellFile(claimsRoot, null, 'cell-sessionless', 60);
  assert(first.ok === true, 'sessionless claim succeeds');
  assert(!('session' in first.claim), 'the claim record omits "session" entirely rather than writing session:null');
  const onDisk = readClaim(claimsRoot, 'cell-sessionless');
  assert(!('session' in onDisk), 'the on-disk claim record also omits "session"');
  const second = claimCellFile(claimsRoot, 'sess-intruder', 'cell-sessionless', 60);
  assert(second.ok === false && second.code === 'CLAIMED', 'a second (session-bearing) claimant still loses to the sessionless winner');
  assert(second.reason.includes('no session (sessionless claim)'), `CLAIMED reason should name the sessionless holder, got ${JSON.stringify(second.reason)}`);
});

await check('releaseClaim(root, null, ...) releases a sessionless claim; a real session cannot release someone else\'s sessionless claim', async () => {
  const deniedByRealSession = releaseClaim(claimsRoot, 'sess-intruder', 'cell-sessionless');
  assert(deniedByRealSession.ok === false && deniedByRealSession.code === 'NOT_OWNER', 'a real session id can never release a sessionless claim it does not own');
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-sessionless')), 'claim untouched by the denied release');
  const released = releaseClaim(claimsRoot, null, 'cell-sessionless');
  assert(released.ok === true, 'the sessionless owner (null) can release its own claim');
  assert(!fs.existsSync(claimPath(claimsRoot, 'cell-sessionless')), 'claim file removed');
});

await check('clearClaim: unconditional removal regardless of owner; no-ops (ok:true, released:null) when there is no claim file; never leaks a gate file', async () => {
  const noClaim = clearClaim(claimsRoot, 'cell-never-claimed');
  assert(noClaim.ok === true && noClaim.released === null, `clearing a cell with no claim file must no-op, got ${JSON.stringify(noClaim)}`);

  claimCellFile(claimsRoot, 'sess-owner', 'cell-to-clear', 60);
  assert(fs.existsSync(claimPath(claimsRoot, 'cell-to-clear')), 'precondition: claim file exists');
  // clearClaim needs no owner argument at all — a DIFFERENT actor (or none)
  // still removes it; this is what cap/unclaim/block/drop/reopen rely on.
  const cleared = clearClaim(claimsRoot, 'cell-to-clear');
  assert(cleared.ok === true && cleared.released && cleared.released.session === 'sess-owner', `clearClaim must remove regardless of caller, got ${JSON.stringify(cleared)}`);
  assert(!fs.existsSync(claimPath(claimsRoot, 'cell-to-clear')), 'claim file removed');
  assert(!fs.existsSync(claimGatePath(claimsRoot, 'cell-to-clear')), 'no gate file leaked by clearClaim');
});

fs.rmSync(claimsRoot, { recursive: true, force: true });

// ─── hardening-4a: isConcurrentMode + sessionless-claim refusal ────────────

const concurrentModeRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-concurrent-'));

await check('isConcurrentMode: false with no session records or only the acting session live; true once ANOTHER session has a live heartbeat; a stale other-session heartbeat does not count', async () => {
  assert(isConcurrentMode(concurrentModeRoot) === false, 'no session records at all -> not concurrent');
  createSession(concurrentModeRoot, { id: 'solo-sess' });
  assert(isConcurrentMode(concurrentModeRoot, { excludeSessionId: 'solo-sess' }) === false, 'only the acting session itself is live -> not concurrent');
  createSession(concurrentModeRoot, { id: 'other-sess' });
  assert(isConcurrentMode(concurrentModeRoot, { excludeSessionId: 'solo-sess' }) === true, 'another live session -> concurrent');
  assert(isConcurrentMode(concurrentModeRoot) === true, 'with no excludeSessionId, any live session counts, including one that would otherwise be excluded');
  writeJsonAtomic(sessionPath(concurrentModeRoot, 'other-sess'), {
    ...readSession(concurrentModeRoot, 'other-sess'),
    last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
  });
  assert(isConcurrentMode(concurrentModeRoot, { excludeSessionId: 'solo-sess' }) === false, 'a stale-heartbeat other session does not count as concurrent');
});

await check('claimCellFile: sessionless claim still works solo (nobody else live) — byte-unchanged, never marked adopted', async () => {
  // 'solo-sess' is still live from the row above (never staled there) — stale
  // every session record here so this actually starts from a genuine solo
  // state before proving the sessionless claim still works unaffected.
  for (const record of listSessionRecords(concurrentModeRoot)) {
    writeJsonAtomic(sessionPath(concurrentModeRoot, record.id), {
      ...record,
      last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
    });
  }
  assert(isConcurrentMode(concurrentModeRoot) === false, 'precondition: genuinely solo before the claim attempt');
  const solo = claimCellFile(concurrentModeRoot, null, 'cell-solo', 60);
  assert(solo.ok === true, 'sessionless claim still succeeds when nobody else is live');
  assert(!('session' in solo.claim), 'solo sessionless claim still omits the session key entirely');
  assert(!solo.adopted, 'a genuinely solo claim (zero live sessions anywhere) is never marked adopted');
  releaseClaim(concurrentModeRoot, null, 'cell-solo');
});

await check('claimCellFile (hardening-1-7-10 D5 — Codex session bridge): a sessionless claim with EXACTLY ONE fresh live session auto-adopts that session\'s identity instead of refusing — this is the solo-session SESSION_REQUIRED bug the durable fallback fixes', async () => {
  heartbeatSession(concurrentModeRoot, 'other-sess'); // bring it back to a live heartbeat -> exactly one fresh session
  assert(isConcurrentMode(concurrentModeRoot) === true, 'precondition: one other session is live (the pre-fix code refused this outright)');
  const adopted = claimCellFile(concurrentModeRoot, null, 'cell-single-adopt', 60);
  assert(adopted.ok === true, `single-fresh-session adoption must succeed instead of refusing SESSION_REQUIRED, got ${JSON.stringify(adopted)}`);
  assert(adopted.claim.session === 'other-sess', `claim adopts the sole live session's identity, got ${JSON.stringify(adopted.claim)}`);
  assert(adopted.adopted === true, 'the result surfaces an adopted:true audit marker so call sites can tell this was inferred, not supplied');
  assert(adopted.claim.adopted === true, 'the on-disk claim record also carries the adopted marker');
  releaseClaim(concurrentModeRoot, 'other-sess', 'cell-single-adopt');
});

await check('claimCellFile (hardening-1-7-10 D5): a sessionless claim with TWO OR MORE fresh live sessions still refuses typed SESSION_REQUIRED — real ambiguity is never guessed, naming --session-id and BEE_SESSION_ID; a real session id still claims fine', async () => {
  heartbeatSession(concurrentModeRoot, 'solo-sess'); // bring a second session back live -> genuine ambiguity (2 fresh)
  assert(isConcurrentMode(concurrentModeRoot) === true, 'precondition: at least one other session live');
  const refused = claimCellFile(concurrentModeRoot, null, 'cell-concurrent', 60);
  assert(refused.ok === false, 'sessionless claim refused while two or more sessions are live');
  assert(refused.code === 'SESSION_REQUIRED', `expected typed SESSION_REQUIRED, got ${refused.code}`);
  assert(typeof refused.reason === 'string' && refused.reason.includes('--session-id'), `reason should name --session-id, got ${JSON.stringify(refused.reason)}`);
  assert(refused.reason.includes('BEE_SESSION_ID'), `reason should name BEE_SESSION_ID, got ${JSON.stringify(refused.reason)}`);
  assert(!fs.existsSync(claimPath(concurrentModeRoot, 'cell-concurrent')), 'the refusal never leaves a claim file behind');

  const withSession = claimCellFile(concurrentModeRoot, 'other-sess', 'cell-concurrent', 60);
  assert(withSession.ok === true, 'a real session id claims fine even in concurrent mode');
  assert(!withSession.adopted, 'an explicitly supplied session id is never marked adopted');
  releaseClaim(concurrentModeRoot, 'other-sess', 'cell-concurrent');
});

fs.rmSync(concurrentModeRoot, { recursive: true, force: true });

// ─── multisession-native-12: fence_epoch (D4/D9 invariant 10) ─────────────
// "fence_epoch" is deliberately NOT called "epoch" bare — cells.mjs:2408's
// checkCellBudgets already uses "epoch" loosely for a different, unrelated
// concept (a claim/heartbeat acquisition cycle for budget-collapse
// counting). This is a distinct, CAS-style fencing token: stamped 1 at
// claimCellFile creation, bumped +1 by adoptClaim atomically with the
// ownership rewrite, and — new in this cell — optionally enforced by
// renewClaimTTL/releaseClaim when (and only when) a caller presents one.
// Legacy callers that never present an epoch (every production caller
// today: heartbeatTouch's renewClaimTTL call, cells.mjs's releaseClaim
// calls) are byte-unchanged — full mandatory presentation arrives with
// workspace identity in slice 4 (advisor consult slice 3, condition F).

const fenceRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-fence-'));

await check('claimCellFile stamps fence_epoch: 1 on a fresh claim', async () => {
  const made = claimCellFile(fenceRoot, 'sess-fence-a', 'cell-fence-1', 3600);
  assert(made.ok === true, 'precondition: claim created');
  assert(made.claim.fence_epoch === 1, `fresh claim must stamp fence_epoch: 1, got ${JSON.stringify(made.claim)}`);
  assert(readClaim(fenceRoot, 'cell-fence-1').fence_epoch === 1, 'fence_epoch is on the on-disk record too');
});

await check('adoptClaim bumps fence_epoch by exactly 1, atomically with the ownership rewrite', async () => {
  const before = readClaim(fenceRoot, 'cell-fence-1');
  assert(before.fence_epoch === 1, 'precondition: still epoch 1 before adoption');
  const adopted = adoptClaim(fenceRoot, 'cell-fence-1', 'sess-fence-b');
  assert(adopted.ok === true, `adopt ok, got ${JSON.stringify(adopted)}`);
  assert(adopted.claim.fence_epoch === 2, `adoption must bump fence_epoch to 2, got ${JSON.stringify(adopted.claim)}`);
  assert(readClaim(fenceRoot, 'cell-fence-1').fence_epoch === 2, 'the bump is durable on disk');
  assert(readClaim(fenceRoot, 'cell-fence-1').session === 'sess-fence-b', 'ownership also transferred in the same write');
});

await check('renewClaimTTL: no presented epoch is byte-unchanged legacy behavior (never fenced)', async () => {
  const before = readClaim(fenceRoot, 'cell-fence-1').claimed_at;
  const result = renewClaimTTL(fenceRoot, 'sess-fence-b', { now: Date.now() + 60_000 });
  assert(result.ok === true && result.renewed.includes('cell-fence-1'), `legacy (no presentedEpoch) renew must still succeed, got ${JSON.stringify(result)}`);
  assert(readClaim(fenceRoot, 'cell-fence-1').claimed_at !== before, 'claimed_at actually advanced');
});

await check('invariant 10: renewClaimTTL refuses typed CLAIM_FENCE_STALE when the presented epoch is behind current fence_epoch; the current epoch succeeds', async () => {
  // cell-fence-1 is at fence_epoch 2 (bumped by the adoptClaim row above),
  // owned by sess-fence-b. A caller presenting the OLD epoch (1) — e.g. a
  // stale in-memory copy captured before the takeover — must be refused
  // even though the session id itself still matches the current owner: the
  // epoch check is an independent, stronger guard than session identity
  // alone (session-mismatch after a cross-session adopt is ALREADY refused
  // silently by the pre-existing ownership filter a few lines up in
  // renewClaimTTL — this proves the NEW, orthogonal epoch-CAS guard).
  const staleClaimBefore = readClaim(fenceRoot, 'cell-fence-1');
  const staleAttempt = renewClaimTTL(fenceRoot, 'sess-fence-b', { now: Date.now() + 120_000, presentedEpoch: 1 });
  assert(staleAttempt.ok === false && staleAttempt.code === 'CLAIM_FENCE_STALE', `expected typed CLAIM_FENCE_STALE, got ${JSON.stringify(staleAttempt)}`);
  assert(typeof staleAttempt.reason === 'string' && staleAttempt.reason.includes('cell-fence-1'), 'refusal names the cell');
  assert(readClaim(fenceRoot, 'cell-fence-1').claimed_at === staleClaimBefore.claimed_at, 'a fenced-stale renewal never touches claimed_at — no partial write');
  assert(!fs.existsSync(claimGatePath(fenceRoot, 'cell-fence-1')), 'the per-claim gate is always released, even on a fenced refusal');

  const currentAttempt = renewClaimTTL(fenceRoot, 'sess-fence-b', { now: Date.now() + 120_000, presentedEpoch: 2 });
  assert(currentAttempt.ok === true && currentAttempt.renewed.includes('cell-fence-1'), `presenting the current epoch must succeed, got ${JSON.stringify(currentAttempt)}`);
});

await check('invariant 10: releaseClaim refuses typed CLAIM_FENCE_STALE when the presented epoch is behind current fence_epoch — the claim file is never removed on a fenced refusal', async () => {
  const staleRelease = releaseClaim(fenceRoot, 'sess-fence-b', 'cell-fence-1', { presentedEpoch: 1 });
  assert(staleRelease.ok === false && staleRelease.code === 'CLAIM_FENCE_STALE', `expected typed CLAIM_FENCE_STALE, got ${JSON.stringify(staleRelease)}`);
  assert(fs.existsSync(claimPath(fenceRoot, 'cell-fence-1')), 'a fenced-stale release must never remove the claim file');
});

await check('releaseClaim: presenting the current epoch succeeds and removes the file (no presentedEpoch at all is the same byte-unchanged legacy release path)', async () => {
  assert(fs.existsSync(claimPath(fenceRoot, 'cell-fence-1')), 'precondition: claim still present (previous row only attempted a stale-fenced release)');
  const currentRelease = releaseClaim(fenceRoot, 'sess-fence-b', 'cell-fence-1', { presentedEpoch: 2 });
  assert(currentRelease.ok === true, `presenting the current epoch must release successfully, got ${JSON.stringify(currentRelease)}`);
  assert(!fs.existsSync(claimPath(fenceRoot, 'cell-fence-1')), 'claim file removed on a correctly-fenced release');
});

fs.rmSync(fenceRoot, { recursive: true, force: true });

// ─── claims: concurrent Worker races (fsh-2) ───────────────────────────────
// The entire race lives inside race_claims_child.mjs as a self-contained
// orchestrator (starts its own barrier-synchronized Worker racers, asserts
// internally, exits 0/1 with a one-line summary). The outer module entrypoint
// uses the shared serialized runner; only the racers themselves are concurrent.

const raceChildScript = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  'race_claims_child.mjs',
);

function runRaceScenario(scenario) {
  return runModuleWorker(raceChildScript, { args: [scenario], timeout: 60000 });
}

// rel1710rc-5 (Windows CI diagnosability): a failing race child can exit
// non-zero with BOTH stdout and stderr genuinely empty — e.g. an uncaught
// exception inside a NESTED racer Worker (raceSweep/raceHeartbeat spawn their
// own child Workers — see race_claims_child.mjs) terminates that thread
// before its default stack-trace dump reaches the outer captured stream, and
// run-module-worker.mjs's own `error`/`signal` fields were never being
// surfaced at all — a real diagnostic could be sitting in `result.error`
// while the message printed only the (empty) stdout/stderr. This formatter is
// unconditional: every field is always shown, `(empty)`/`(none)` included, so
// a CI failure never again reads as a bare "status 1" with nothing to go on.
function describeRaceFailure(scenario, result) {
  const stdout = result.stdout || '(empty)';
  const stderr = result.stderr || '(empty)';
  const errorDetail = result.error ? (result.error.stack || result.error.message || String(result.error)) : '(none)';
  return `${scenario} race failed (status ${result.status}, signal ${result.signal}): stdout=${stdout} stderr=${stderr} error=${errorDetail}`;
}

await check('race: claim-contention — concurrent workers racing one cell, exactly one O_EXCL winner every round', async () => {
  const result = await runRaceScenario('claim-contention');
  assert(result.status === 0, describeRaceFailure('claim-contention', result));
  assert(/^PASS +claim-contention/m.test(result.stdout), `expected a PASS summary line, got: ${result.stdout}`);
});

await check('race: adoption-steal — a third session cannot steal a cell mid-adoption; every attempt loses with typed CLAIMED', async () => {
  const result = await runRaceScenario('adoption-steal');
  assert(result.status === 0, describeRaceFailure('adoption-steal', result));
  assert(/^PASS +adoption-steal/m.test(result.stdout), `expected a PASS summary line, got: ${result.stdout}`);
});

await check('race: sweep-heartbeat — concurrent sweepExpiredClaims + heartbeat renewal never reclaims a live claim (20260710)', async () => {
  const result = await runRaceScenario('sweep-heartbeat');
  assert(result.status === 0, describeRaceFailure('sweep-heartbeat', result));
  assert(/^PASS +sweep-heartbeat/m.test(result.stdout), `expected a PASS summary line, got: ${result.stdout}`);
});

// ─── rel180-2: forced-interleaving proof (deterministic, no worker threads,
// no timing luck) ────────────────────────────────────────────────────────
// The worker-thread race above (sweep-heartbeat) only catches the bug when
// real OS scheduling happens to land a renewal in the exact unprotected
// window — that is what made it ~1-in-4-under-load instead of 10/10. This
// row instead uses sweepExpiredClaims's own `_raceSeam` (test-only, no
// production caller ever passes it — see claims.mjs) to FORCE a heartbeat
// renewal to land precisely between the sweeper's heartbeat-staleness
// decision and its reclaim, every single round, with no dependency on
// thread scheduling. The invariant under test: it is never simultaneously
// true that a renewal reports success (the on-disk heartbeat really did
// move to "now") AND the claim was reclaimed anyway — that combination IS
// the diagnosed bug (a live, heartbeating owner's claim swept out from
// under it). Pre-fix this fails 10/10; post-fix it passes 10/10.
await check('race: sweep-vs-renewal (rel180-2, forced interleaving) — a renewal forced to land between the sweep\'s decision and its reclaim is never invisible to that decision', async () => {
  const ROUNDS = 10;
  const STALE_SECONDS = 1;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-race180-'));
    const ownerSession = `owner-${r}`;
    const cellId = `race180-cell-${r}`;
    createSession(root, { id: ownerSession });
    const claimed = claimCellFile(root, ownerSession, cellId, 60);
    assert(claimed.ok === true, `round ${r}: setup claim failed`);
    // Backdate BOTH the claim's TTL and the owner's heartbeat so the claim
    // is a genuine sweep candidate by the numbers the instant
    // sweepExpiredClaims starts — the seam below is what then injects the
    // renewal a correct implementation must not silently miss.
    writeJsonAtomic(claimPath(root, cellId), {
      ...readClaim(root, cellId),
      claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
      ttl_seconds: 60,
    });
    writeJsonAtomic(sessionPath(root, ownerSession), {
      ...readSession(root, ownerSession),
      last_heartbeat: new Date(Date.now() - (STALE_SECONDS + 5) * 1000).toISOString(),
    });

    let heartbeatResult = null;
    const result = await sweepExpiredClaims(root, {
      staleSeconds: STALE_SECONDS,
      _raceSeam: async ({ session }) => {
        heartbeatResult = heartbeatSession(root, session);
      },
    });

    const swept = result.ok === true && result.swept.includes(cellId);
    if (heartbeatResult && heartbeatResult.ok === true && swept) {
      failures.push(
        `round ${r}: erroneous sweep — renewal reported success (last_heartbeat=${heartbeatResult.session && heartbeatResult.session.last_heartbeat}) yet the claim was reclaimed anyway (swept=${JSON.stringify(result.swept)})`,
      );
    }
    // Sanity: the seam must actually have fired every round, or this test
    // would vacuously pass by never exercising the race at all.
    if (!heartbeatResult) {
      failures.push(`round ${r}: seam never fired — claim was not a sweep candidate, test setup is broken`);
    }
    fs.rmSync(root, { recursive: true, force: true });
  }
  assert(
    failures.length === 0,
    `sweep-vs-renewal forced interleaving: ${failures.length}/${ROUNDS} rounds bad | ${failures.join(' | ')}`,
  );
});

// D10a (issue #56 3.8, forced interleaving — same seam discipline as the
// rel180-2 sweep-vs-renewal test above, never a sleep/Worker race): closes
// the lost-update window where bindSessionLane/unbindSessionLane raced
// heartbeatSession's read-modify-write. heartbeatSession's _raceSeam fires
// once per round, synchronously, at the exact point a concurrent bind/unbind
// could land — post-fix that point is INSIDE heartbeatSession's own lock
// hold, so the injected call must come back typed LOCK_BUSY (never a silent
// unlocked write) and only succeeds once heartbeatSession has released.
await check('race: heartbeat-vs-bindSessionLane (D10a, forced interleaving) — a bind forced into heartbeat\'s read-modify-write window is never clobbered by the stale heartbeat write', async () => {
  const ROUNDS = 10;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-race-bind-'));
    const sessionId = `bind-race-${r}`;
    const lane = `bind-race-lane-${r}`;
    createSession(root, { id: sessionId }); // unbound at start
    let seamResult = null;
    const beat = heartbeatSession(root, sessionId, {
      _raceSeam: () => {
        // Small lockAttempts: if the lock is genuinely held (post-fix), this
        // should fail fast as LOCK_BUSY rather than burn the full ~300ms
        // default budget every round.
        seamResult = bindSessionLane(root, sessionId, lane, { lockAttempts: 3 });
      },
    });
    if (!beat.ok) failures.push(`round ${r}: heartbeat itself failed: ${JSON.stringify(beat)}`);
    if (!seamResult) {
      failures.push(`round ${r}: seam never fired — test setup is broken`);
      fs.rmSync(root, { recursive: true, force: true });
      continue;
    }
    // A caller that sees LOCK_BUSY retries — same contract as every other
    // typed LOCK_BUSY in this file. A pre-fix seamResult.ok===true means the
    // bind already landed (unlocked), which is exactly the bug: it must NOT
    // then be silently erased by heartbeatSession's stale write.
    const finalBind = seamResult.ok === true ? seamResult : bindSessionLane(root, sessionId, lane);
    const onDisk = readSession(root, sessionId);
    if (!finalBind.ok || !onDisk || onDisk.lane !== lane) {
      failures.push(
        `round ${r}: binding lost — seamResult=${JSON.stringify(seamResult)} finalBind=${JSON.stringify(finalBind)} onDisk=${JSON.stringify(onDisk)}`,
      );
    }
    fs.rmSync(root, { recursive: true, force: true });
  }
  assert(
    failures.length === 0,
    `heartbeat-vs-bind forced interleaving: ${failures.length}/${ROUNDS} rounds bad | ${failures.join(' | ')}`,
  );
});

// Symmetric case: a lane that IS bound must not be resurrected by a stale
// heartbeat write racing a concurrent unbind — same seam, opposite direction.
await check('race: heartbeat-vs-unbindSessionLane (D10a, forced interleaving) — an unbind forced into heartbeat\'s read-modify-write window is never resurrected by the stale heartbeat write', async () => {
  const ROUNDS = 10;
  const failures = [];
  for (let r = 0; r < ROUNDS; r += 1) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-race-unbind-'));
    const sessionId = `unbind-race-${r}`;
    const lane = `unbind-race-lane-${r}`;
    createSession(root, { id: sessionId });
    const setupBind = bindSessionLane(root, sessionId, lane);
    assert(setupBind.ok === true, `round ${r}: setup bind failed`);
    let seamResult = null;
    const beat = heartbeatSession(root, sessionId, {
      _raceSeam: () => {
        seamResult = unbindSessionLane(root, sessionId, { lockAttempts: 3 });
      },
    });
    if (!beat.ok) failures.push(`round ${r}: heartbeat itself failed: ${JSON.stringify(beat)}`);
    if (!seamResult) {
      failures.push(`round ${r}: seam never fired — test setup is broken`);
      fs.rmSync(root, { recursive: true, force: true });
      continue;
    }
    const finalUnbind = seamResult.ok === true ? seamResult : unbindSessionLane(root, sessionId);
    const onDisk = readSession(root, sessionId);
    if (!finalUnbind.ok || !onDisk || 'lane' in onDisk) {
      failures.push(
        `round ${r}: unbind lost (lane resurrected) — seamResult=${JSON.stringify(seamResult)} finalUnbind=${JSON.stringify(finalUnbind)} onDisk=${JSON.stringify(onDisk)}`,
      );
    }
    fs.rmSync(root, { recursive: true, force: true });
  }
  assert(
    failures.length === 0,
    `heartbeat-vs-unbind forced interleaving: ${failures.length}/${ROUNDS} rounds bad | ${failures.join(' | ')}`,
  );
});

// jrt-1: hardening-4b's sweep-reset (claims.mjs:851) calls logDecision with no
// tags at all. docs/decisions/taxonomy.json makes an untagged event a typed
// refusal (DecisionsUntaggedRefusedError) — swallowed here by the surrounding
// try/catch ("a decision-log failure must never be treated as the reset
// itself having failed"), so the cell reset still lands, but the audit line
// itself was silently dropped in any repo with a taxonomy (this one has one).
// Proves both: the reset still succeeds, AND (post-fix) the audit decision now
// actually lands, tagged for what it is.
await check('sweep-reset logs a tagged audit decision even under a taxonomy, instead of the decision silently vanishing (jrt-1)', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-taxonomy-sweep-'));
  try {
    fs.mkdirSync(path.join(root, '.bee', 'cells'), { recursive: true });
    fs.mkdirSync(path.join(root, 'docs', 'decisions'), { recursive: true });
    writeJsonAtomic(path.join(root, 'docs', 'decisions', 'taxonomy.json'), {
      schema_version: 1,
      tags: [
        { name: 'claims', description: 'Cross-session claim store' },
        { name: 'sweep', description: 'Propagation sweeps' },
      ],
      candidates: [],
    });

    const ownerSession = 'owner-tax-sweep';
    createSession(root, { id: ownerSession });
    const cellId = 'tax-sweep-1';
    const claimed = claimCellFile(root, ownerSession, cellId, 60);
    assert(claimed.ok === true, 'setup: claim must succeed');
    writeJsonAtomic(path.join(root, '.bee', 'cells', `${cellId}.json`), {
      id: cellId,
      status: 'claimed',
      trace: { claim_session: ownerSession },
    });
    writeJsonAtomic(claimPath(root, cellId), {
      ...readClaim(root, cellId),
      claimed_at: new Date(Date.now() - 7200 * 1000).toISOString(),
      ttl_seconds: 60,
    });
    writeJsonAtomic(sessionPath(root, ownerSession), {
      ...readSession(root, ownerSession),
      last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
    });

    const result = await sweepExpiredClaims(root);
    assert(result.swept.includes(cellId), `precondition: the stale claim must be swept, got ${JSON.stringify(result)}`);
    assert(result.reset.includes(cellId), `precondition: the cell must be reset claimed->open, got ${JSON.stringify(result)}`);
    const resetCell = JSON.parse(fs.readFileSync(path.join(root, '.bee', 'cells', `${cellId}.json`), 'utf8'));
    assert(resetCell.status === 'open', 'the cell reset itself must still succeed even under a taxonomy');

    const decisions = activeDecisions(root, { recent: 5 });
    const logged = decisions.find((d) => d.decision.includes(cellId));
    assert(
      logged,
      `sweep-reset must log its audit decision even under a taxonomy — before the fix DECISIONS_UNTAGGED_REFUSED was thrown and silently swallowed by the try/catch, so no decision ever landed; decisions seen: ${JSON.stringify(decisions)}`,
    );
    assert(
      Array.isArray(logged.tags) && logged.tags.includes('claims') && logged.tags.includes('sweep'),
      `sweep-reset decision should be tagged claims+sweep (what the event IS), got ${JSON.stringify(logged.tags)}`,
    );
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

// ─── multisession-native-8 (D6): activeWorkers derived view ─────────────────

await check('activeWorkers: a stale heartbeat drops a session out of the view with NO worker-remove call; a live session with an active claim shows its cell; excludeSessionId drops the caller\'s own row', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-activeworkers-'));
  try {
    assert(JSON.stringify(activeWorkers(root)) === '[]', 'zero session records anywhere -> zero active workers');

    createSession(root, { id: 'live-idle' }); // live heartbeat, no claim
    createSession(root, { id: 'live-claimed' }); // live heartbeat, active claim
    createSession(root, { id: 'stale-sess' }); // will be staled below
    claimCellFile(root, 'live-claimed', 'cell-1');

    // Stale it directly on disk (deterministic — no sleeps): older than
    // DEFAULT_HEARTBEAT_STALE_SECONDS, same technique isConcurrentMode's own
    // suite above uses.
    writeJsonAtomic(sessionPath(root, 'stale-sess'), {
      ...readSession(root, 'stale-sess'),
      last_heartbeat: new Date(Date.now() - (DEFAULT_HEARTBEAT_STALE_SECONDS + 3600) * 1000).toISOString(),
    });

    const workers = activeWorkers(root);
    const byId = Object.fromEntries(workers.map((w) => [w.session_id, w]));
    assert(
      workers.length === 2 && !('stale-sess' in byId),
      `stale-sess must drop out of the view with NO worker-remove call, got ${JSON.stringify(workers)}`,
    );
    assert(byId['live-idle'] && byId['live-idle'].cell === null, `a live session with no claim shows cell: null, got ${JSON.stringify(byId['live-idle'])}`);
    assert(
      byId['live-claimed'] && byId['live-claimed'].cell === 'cell-1',
      `a live session with an active claim shows its cell, got ${JSON.stringify(byId['live-claimed'])}`,
    );
    assert(typeof byId['live-claimed'].last_heartbeat === 'string', 'last_heartbeat is carried through');

    const excluded = activeWorkers(root, { excludeSessionId: 'live-claimed' });
    assert(
      excluded.length === 1 && excluded[0].session_id === 'live-idle',
      `excludeSessionId drops exactly the caller's own row, got ${JSON.stringify(excluded)}`,
    );

    // Releasing the claim (not removing any "worker") drops the cell field —
    // proof the view is truly derived, never hand-mutated.
    releaseClaim(root, 'live-claimed', 'cell-1');
    const afterRelease = activeWorkers(root).find((w) => w.session_id === 'live-claimed');
    assert(afterRelease && afterRelease.cell === null, `releasing the claim clears cell without any worker verb, got ${JSON.stringify(afterRelease)}`);
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('activeWorkers: session.lane binding is surfaced when bound, null while unbound', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-activeworkers-lane-'));
  try {
    createSession(root, { id: 'unbound-sess' });
    createSession(root, { id: 'bound-sess' });
    bindSessionLane(root, 'bound-sess', 'demo-lane');
    const byId = Object.fromEntries(activeWorkers(root).map((w) => [w.session_id, w]));
    assert(byId['unbound-sess'].lane === null, 'unbound session -> lane: null');
    assert(byId['bound-sess'].lane === 'demo-lane', 'bound session -> lane: the bound feature');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('createSession stamps workspace_id when given (msn-19); omits the field entirely when absent/blank, same convention as transcript_path', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-workspace-id-'));
  try {
    const withWorkspace = createSession(root, { id: 'sess-ws-a', workspace_id: 'wt-abc123' });
    assert(withWorkspace.ok === true, 'createSession with workspace_id succeeds');
    assert(withWorkspace.session.workspace_id === 'wt-abc123', 'returned record carries workspace_id');
    const readBack = readSession(root, 'sess-ws-a');
    assert(readBack.workspace_id === 'wt-abc123', 'workspace_id round-trips through disk');

    const withoutWorkspace = createSession(root, { id: 'sess-ws-b' });
    assert(withoutWorkspace.ok === true, 'createSession without workspace_id still succeeds');
    assert(!('workspace_id' in withoutWorkspace.session), 'workspace_id key is OMITTED, never written as null/undefined, when absent');

    const blankWorkspace = createSession(root, { id: 'sess-ws-c', workspace_id: '   ' });
    assert(!('workspace_id' in blankWorkspace.session), 'a blank/whitespace-only workspace_id is treated as absent, same as transcript_path');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('claimCellFile (msn-19): auto-looks-up workspace_id from the ACTING SESSION\'s own record — never a caller-supplied claim argument; omitted when the session carries none, or for a sessionless claim', async () => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-claim-workspace-id-'));
  try {
    createSession(root, { id: 'sess-ws-owner', workspace_id: 'wt-owner-1' });
    createSession(root, { id: 'sess-ws-none' });

    const claimed = claimCellFile(root, 'sess-ws-owner', 'cell-ws-1');
    assert(claimed.ok === true, 'claim succeeds');
    assert(claimed.claim.workspace_id === 'wt-owner-1', 'claim record carries the acting session\'s stamped workspace_id');
    const readBack = readClaim(root, 'cell-ws-1');
    assert(readBack.workspace_id === 'wt-owner-1', 'workspace_id round-trips through the claim file on disk');

    const claimedNoWorkspace = claimCellFile(root, 'sess-ws-none', 'cell-ws-2');
    assert(claimedNoWorkspace.ok === true, 'claim succeeds for a session with no stamped workspace_id');
    assert(!('workspace_id' in claimedNoWorkspace.claim), 'claim omits workspace_id when the acting session has none — never null');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

await check('claimCellFile (msn-19): a sessionless claim never carries workspace_id — nothing to look up', async () => {
  // Deliberately its OWN fresh root (no other live sessions) — a sessionless
  // claim in a concurrent-mode root is a SEPARATE refusal path
  // (SESSION_REQUIRED, see hardening-4a above) unrelated to workspace_id.
  const root = fs.mkdtempSync(path.join(os.tmpdir(), 'bee-claims-claim-workspace-id-sessionless-'));
  try {
    const sessionlessClaim = claimCellFile(root, null, 'cell-ws-3');
    assert(sessionlessClaim.ok === true, 'sessionless claim still succeeds when solo (no other live session)');
    assert(!('workspace_id' in sessionlessClaim.claim), 'a sessionless claim never carries workspace_id — nothing to look up');
  } finally {
    fs.rmSync(root, { recursive: true, force: true });
  }
});

printSummaryAndExit();
