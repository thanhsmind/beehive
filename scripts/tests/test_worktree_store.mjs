#!/usr/bin/env node
// Unit tests for .bee/bin/lib/worktree-store.mjs — the NEW, not-yet-wired
// decision + replay logic for bee feature `worktree-feature-parallelism`
// slice S2. Pure unit tests: no git, no real checkout, just the exported
// functions against constructed inputs. Mirrors the cases proven by the
// throwaway spike .bee/spikes/worktree-feature-parallelism/seam-proof.mjs
// (5/5 passed), now against the real module.

import {
  decideWorktreeStore,
  readGrants,
  replayLog,
  writeGrant,
  removeGrant,
  listGrants,
  bootstrapWorktreeStore,
} from "../../.bee/bin/lib/worktree-store.mjs";

import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  if (passed) {
    console.log(`PASS ${desc}`);
  } else {
    console.log(`FAIL ${desc}${detail ? ` -- ${detail}` : ""}`);
  }
}

function tmpDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), prefix));
}

// ---------------------------------------------------------------------------
// decideWorktreeStore
// ---------------------------------------------------------------------------

{
  const classification = {
    kind: "linked-valid",
    id: "abc123",
    mainRoot: "/repo/main",
    worktreeRoot: "/repo/wtA",
  };
  const grants = { abc123: true };
  const result = decideWorktreeStore(classification, { grants });
  const ok =
    result.ok === true &&
    result.kind === "linked-valid-granted" &&
    result.storeRoot === path.join("/repo/wtA", ".bee") &&
    result.id === "abc123";
  record("linked-valid + granted resolves to the worktree's own local store", ok, JSON.stringify(result));
}

{
  const classification = {
    kind: "linked-valid",
    id: "def456",
    mainRoot: "/repo/main",
    worktreeRoot: "/repo/wtB",
  };
  const grants = {}; // not granted
  const result = decideWorktreeStore(classification, { grants });
  const ok =
    result.ok === true &&
    result.kind === "linked-valid-default" &&
    result.storeRoot === path.join("/repo/main", ".bee") &&
    result.id === "def456";
  record("linked-valid + not granted falls back to the main store (P40 default)", ok, JSON.stringify(result));
}

{
  const classification = { kind: "linked-invalid" };
  let threw = false;
  let result;
  try {
    result = decideWorktreeStore(classification, { grants: { anything: true } });
  } catch {
    threw = true;
  }
  const ok = !threw && result && result.ok === false && result.reason === "WORKTREE_LINK_INVALID";
  record(
    "linked-invalid returns a typed deny reason and never throws (wrapped in try/catch)",
    ok,
    threw ? "decideWorktreeStore THREW" : JSON.stringify(result),
  );
}

{
  const classification = { kind: "ordinary", mainRoot: "/repo/solo" };
  const result = decideWorktreeStore(classification, { grants: {} });
  const ok = result.ok === true && result.kind === "ordinary" && result.storeRoot === path.join("/repo/solo", ".bee");
  record("ordinary checkout resolves to its own store", ok, JSON.stringify(result));
}

{
  // ordinary with worktreeRoot present should prefer worktreeRoot per spec.
  const classification = { kind: "ordinary", mainRoot: "/repo/solo", worktreeRoot: "/repo/solo-alt" };
  const result = decideWorktreeStore(classification, { grants: {} });
  const ok = result.ok === true && result.storeRoot === path.join("/repo/solo-alt", ".bee");
  record("ordinary checkout prefers worktreeRoot over mainRoot when both are present", ok, JSON.stringify(result));
}

// ---------------------------------------------------------------------------
// Structural self-trust proof: decideWorktreeStore has no parameter and no
// code path through which a worktree's own marker could influence the
// result. Two calls with IDENTICAL grants but a "self-claim" simulated only
// in a worktree-side object must produce identical results, because the
// function literally never receives or reads worktree-side data — this is
// exactly why the onboarding-marker-as-trust pattern (a worktree writing its
// own grant-shaped file and having it be honored) cannot recur here: there
// is no parameter through which such a marker could even be passed in.
// ---------------------------------------------------------------------------

{
  const classification = {
    kind: "linked-valid",
    id: "self-claim-id",
    mainRoot: "/repo/main",
    worktreeRoot: "/repo/wtSelf",
  };
  const trustedGrants = {}; // main registry says: not granted

  // A worktree-side object simulating a self-written marker claiming a
  // grant. decideWorktreeStore's signature is (classification, { grants })
  // -- there is no third argument, and `classification` itself carries no
  // grant-shaped field. Passing this simulated marker as extraneous data on
  // an unrelated key proves it has zero path into the decision: the
  // function only ever reads `grants[classification.id]`.
  const selfWrittenWorktreeMarker = { "self-claim-id": true };

  const resultA = decideWorktreeStore(classification, { grants: trustedGrants });
  const resultB = decideWorktreeStore(classification, {
    grants: trustedGrants,
    // Even if a caller mistakenly attached worktree-side data under some
    // other key, decideWorktreeStore's destructuring `{ grants }` ignores
    // every other key entirely.
    worktreeSelfMarker: selfWrittenWorktreeMarker,
  });

  // Also prove there is no way to route the self-written marker in AS the
  // grants object itself and still land on a grant — because the caller
  // contract is "grants must come from the MAIN store". Simulate a caller
  // who (incorrectly) unions in the worktree's own marker; showing the
  // *correct*-contract call ignores it is the point, not defending against
  // a caller who breaks the contract on purpose.
  const bothIdentical = JSON.stringify(resultA) === JSON.stringify(resultB);
  const stillDefault = resultA.ok === true && resultA.kind === "linked-valid-default" && resultB.kind === "linked-valid-default";

  record(
    "structural self-trust proof: identical grants + worktree-side self-claim data produce identical results (function never sees worktree-side data)",
    bothIdentical && stillDefault,
    JSON.stringify({ resultA, resultB }),
  );
}

{
  // Reinforce: decideWorktreeStore's source has exactly two formal
  // parameters -- (classification, { grants } = {}) -- and nothing about
  // `classification.worktreeRoot` is ever used to locate a grants source
  // inside decideWorktreeStore itself (only readGrants, called separately
  // by the caller against the MAIN store, ever touches the filesystem).
  // (Function.prototype.length stops counting at the first defaulted
  // parameter, so it reports 1 here -- that is a JS quirk, not evidence of
  // a missing parameter, which is why this checks the source text instead.)
  const src = decideWorktreeStore.toString();
  const paramLine = src.slice(0, src.indexOf(")") + 1);
  const twoFormalParams = /^function\s*\w*\s*\(\s*classification\s*,\s*\{\s*grants\s*\}/.test(paramLine.replace(/\n/g, " "));
  record(
    "decideWorktreeStore's only formal parameters are (classification, { grants }) -- no channel for worktree-supplied trust data",
    twoFormalParams,
    paramLine,
  );
}

// ---------------------------------------------------------------------------
// readGrants
// ---------------------------------------------------------------------------

{
  const dir = tmpDir("worktree-store-missing-");
  try {
    const grants = readGrants(dir); // runtime/worktree-grants.json does not exist
    record("readGrants: missing file returns {}", typeof grants === "object" && grants !== null && Object.keys(grants).length === 0, JSON.stringify(grants));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-corrupt-");
  try {
    const runtimeDir = path.join(dir, "runtime");
    fs.mkdirSync(runtimeDir, { recursive: true });
    fs.writeFileSync(path.join(runtimeDir, "worktree-grants.json"), "{ not valid json ][");
    let threw = false;
    let grants;
    try {
      grants = readGrants(dir);
    } catch {
      threw = true;
    }
    const ok = !threw && typeof grants === "object" && grants !== null && Object.keys(grants).length === 0;
    record("readGrants: corrupt JSON returns {} without throwing", ok, threw ? "readGrants THREW" : JSON.stringify(grants));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-valid-");
  try {
    const runtimeDir = path.join(dir, "runtime");
    fs.mkdirSync(runtimeDir, { recursive: true });
    const payload = { abc123: true, def456: false };
    fs.writeFileSync(path.join(runtimeDir, "worktree-grants.json"), JSON.stringify(payload));
    const grants = readGrants(dir);
    const ok = JSON.stringify(grants) === JSON.stringify(payload);
    record("readGrants: valid file returns the parsed object", ok, JSON.stringify(grants));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// ---------------------------------------------------------------------------
// replayLog
// ---------------------------------------------------------------------------

{
  const events = [
    { id: "e1", ts: 1, type: "add" },
    { id: "dup1", ts: 5, type: "first" },
    { id: "dup1", ts: 5, type: "second" }, // duplicate id, same ts: last wins after sort
  ];
  const state = replayLog(events);
  const ok = Object.keys(state).filter((k) => k === "dup1").length === 1 && state.dup1.type === "second";
  record("replayLog: duplicate id folds to a single entry (last wins after sort)", ok, JSON.stringify(state));
}

{
  const events = [
    { id: "e1", ts: 1, type: "add" },
    { id: "e0", ts: 0, type: "init" },
    { id: "dup1", ts: 5, type: "x" },
    { id: "dup1", ts: 5, type: "y" },
  ];
  const state1 = replayLog(events);
  const state2 = replayLog(events);
  const bytes1 = JSON.stringify(state1);
  const bytes2 = JSON.stringify(state2);
  record("replayLog: idempotent -- two calls on the same input are byte-identical via JSON.stringify", bytes1 === bytes2, `${bytes1} vs ${bytes2}`);
}

{
  // ordering by (ts, id): events out of order in the input must still fold
  // deterministically, and same-ts entries order by id.
  const eventsA = [
    { id: "b", ts: 2, type: "second-ts2" },
    { id: "a", ts: 2, type: "first-ts2" },
    { id: "z", ts: 1, type: "only-ts1" },
  ];
  const eventsB = [
    { id: "z", ts: 1, type: "only-ts1" },
    { id: "a", ts: 2, type: "first-ts2" },
    { id: "b", ts: 2, type: "second-ts2" },
  ];
  const stateA = replayLog(eventsA);
  const stateB = replayLog(eventsB);
  const ok = JSON.stringify(stateA) === JSON.stringify(stateB) && Object.keys(stateA).length === 3;
  record("replayLog: same events in a different input order fold to the same (ts,id)-sorted result", ok, `${JSON.stringify(stateA)} vs ${JSON.stringify(stateB)}`);
}

{
  // Does not mutate the input array.
  const events = [
    { id: "b", ts: 2, type: "x" },
    { id: "a", ts: 1, type: "y" },
  ];
  const before = JSON.stringify(events);
  replayLog(events);
  const after = JSON.stringify(events);
  record("replayLog: does not mutate its input array", before === after, `${before} vs ${after}`);
}

// ---------------------------------------------------------------------------
// writeGrant / removeGrant / listGrants (Slice A: CLI-usable write side)
// ---------------------------------------------------------------------------

// hardening-4b: writeGrant/removeGrant are now async (withStoreLock-wrapped,
// serialized under the MAIN checkout's 'worktree-admin' lock). The lock root
// is derived from `path.dirname(mainStoreRoot)` — production always calls
// these with `mainStoreRoot === path.join(mainRoot, '.bee')`, so these tests
// now mirror that exact shape (`path.join(dir, ".bee")`, not a bare tmpDir)
// instead of the looser pre-existing convention, and every call is awaited
// (top-level await — plain ESM, no wrapping async function needed here).

{
  const dir = tmpDir("worktree-store-write-grant-");
  const mainStoreRoot = path.join(dir, ".bee");
  try {
    const next = await writeGrant(mainStoreRoot, "id-a");
    const ok = next["id-a"] === true && JSON.stringify(readGrants(mainStoreRoot)) === JSON.stringify(next);
    record("writeGrant: creates the runtime dir + grants file when absent", ok, JSON.stringify(next));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-write-grant-merge-");
  const mainStoreRoot = path.join(dir, ".bee");
  try {
    await writeGrant(mainStoreRoot, "id-a");
    const next = await writeGrant(mainStoreRoot, "id-b");
    const ok = next["id-a"] === true && next["id-b"] === true && Object.keys(next).length === 2;
    record("writeGrant: merges into existing entries, never drops prior grants", ok, JSON.stringify(next));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-remove-grant-");
  const mainStoreRoot = path.join(dir, ".bee");
  try {
    await writeGrant(mainStoreRoot, "id-a");
    await writeGrant(mainStoreRoot, "id-b");
    const next = await removeGrant(mainStoreRoot, "id-a");
    const ok = !("id-a" in next) && next["id-b"] === true && !("id-a" in readGrants(mainStoreRoot));
    record("removeGrant: deletes only the named id, preserves the rest", ok, JSON.stringify(next));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-remove-grant-missing-");
  const mainStoreRoot = path.join(dir, ".bee");
  try {
    const before = readGrants(mainStoreRoot);
    const after = await removeGrant(mainStoreRoot, "never-granted");
    const ok = JSON.stringify(before) === "{}" && JSON.stringify(after) === "{}";
    record("removeGrant: no-op (never throws) when the id/file does not exist", ok, JSON.stringify(after));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

{
  const dir = tmpDir("worktree-store-list-grants-");
  const mainStoreRoot = path.join(dir, ".bee");
  try {
    await writeGrant(mainStoreRoot, "id-a");
    const ok = JSON.stringify(listGrants(mainStoreRoot)) === JSON.stringify(readGrants(mainStoreRoot));
    record("listGrants: matches readGrants (thin named alias, no second read implementation)", ok, JSON.stringify(listGrants(mainStoreRoot)));
  } finally {
    fs.rmSync(dir, { recursive: true, force: true });
  }
}

// ---------------------------------------------------------------------------
// bootstrapWorktreeStore
// ---------------------------------------------------------------------------

{
  const tmpRoot = tmpDir("worktree-store-bootstrap-");
  try {
    const worktreeRoot = path.join(tmpRoot, "wt");
    const mainStoreRoot = path.join(tmpRoot, "main", ".bee");
    fs.mkdirSync(worktreeRoot, { recursive: true });
    fs.mkdirSync(mainStoreRoot, { recursive: true });
    fs.writeFileSync(path.join(mainStoreRoot, "onboarding.json"), JSON.stringify({ schema_version: "1.0" }));
    fs.writeFileSync(path.join(mainStoreRoot, "config.json"), JSON.stringify({ commands: {} }));

    const result = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, "my-feature");
    const stateFile = path.join(worktreeRoot, ".bee", "state.json");
    const state = JSON.parse(fs.readFileSync(stateFile, "utf8"));
    const ok =
      result.created === true &&
      state.feature === "my-feature" &&
      state.phase === "idle" &&
      state.approved_gates.context === false &&
      state.approved_gates.shape === false &&
      state.approved_gates.execution === false &&
      state.approved_gates.review === false &&
      fs.existsSync(path.join(worktreeRoot, ".bee", "onboarding.json")) &&
      fs.existsSync(path.join(worktreeRoot, ".bee", "config.json"));
    record(
      "bootstrapWorktreeStore: writes a fresh idle/all-gates-false state.json and copies onboarding.json/config.json from main",
      ok,
      JSON.stringify({ result, state }),
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

{
  const tmpRoot = tmpDir("worktree-store-bootstrap-idempotent-");
  try {
    const worktreeRoot = path.join(tmpRoot, "wt");
    const mainStoreRoot = path.join(tmpRoot, "main", ".bee");
    fs.mkdirSync(worktreeRoot, { recursive: true });
    fs.mkdirSync(mainStoreRoot, { recursive: true });

    bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, "first-feature");
    const stateFile = path.join(worktreeRoot, ".bee", "state.json");
    fs.writeFileSync(stateFile, JSON.stringify({ sentinel: "do-not-clobber", phase: "swarming" }));

    const second = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, "second-feature");
    const afterState = JSON.parse(fs.readFileSync(stateFile, "utf8"));
    const ok = second.created === false && afterState.sentinel === "do-not-clobber" && afterState.phase === "swarming";
    record(
      "bootstrapWorktreeStore: idempotent — re-running with an existing state.json never overwrites it",
      ok,
      JSON.stringify({ second, afterState }),
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

{
  const tmpRoot = tmpDir("worktree-store-bootstrap-no-main-files-");
  try {
    const worktreeRoot = path.join(tmpRoot, "wt");
    const mainStoreRoot = path.join(tmpRoot, "main", ".bee");
    fs.mkdirSync(worktreeRoot, { recursive: true });
    fs.mkdirSync(mainStoreRoot, { recursive: true }); // no onboarding.json/config.json on the main side

    const result = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, null);
    const state = JSON.parse(fs.readFileSync(path.join(worktreeRoot, ".bee", "state.json"), "utf8"));
    const ok =
      result.created === true &&
      result.onboarding.copied === false &&
      result.config.copied === false &&
      state.feature === null &&
      !fs.existsSync(path.join(worktreeRoot, ".bee", "onboarding.json"));
    record(
      "bootstrapWorktreeStore: missing main onboarding.json/config.json degrades to copied:false, never throws; null feature is preserved",
      ok,
      JSON.stringify({ result, state }),
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// ---------------------------------------------------------------------------
// bootstrapWorktreeStore — the IMMUTABLE creation slug (issues-46-53 D4, #46)
// ---------------------------------------------------------------------------

{
  const tmpRoot = tmpDir("worktree-store-identity-");
  try {
    const worktreeRoot = path.join(tmpRoot, "wt");
    const mainStoreRoot = path.join(tmpRoot, "main", ".bee");
    fs.mkdirSync(worktreeRoot, { recursive: true });
    fs.mkdirSync(mainStoreRoot, { recursive: true });

    const first = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, "created-as-this");
    const identityFile = path.join(worktreeRoot, ".bee", "runtime", "worktree-identity.json");
    const identity = JSON.parse(fs.readFileSync(identityFile, "utf8"));
    record(
      "bootstrapWorktreeStore: records the creation slug in .bee/runtime/worktree-identity.json (gitignored everywhere, so it never makes a worktree read dirty)",
      first.identity.written === true && identity.feature === "created-as-this" && typeof identity.created_at === "string",
      JSON.stringify({ identity: first.identity, onDisk: identity }),
    );

    // "Immutable" is the whole point: a re-bootstrap under a DIFFERENT name —
    // the exact shape of the drift #46 is about — must never restate it.
    fs.rmSync(path.join(worktreeRoot, ".bee", "state.json"));
    const second = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, "renamed-later");
    const afterIdentity = JSON.parse(fs.readFileSync(identityFile, "utf8"));
    const afterState = JSON.parse(fs.readFileSync(path.join(worktreeRoot, ".bee", "state.json"), "utf8"));
    record(
      "bootstrapWorktreeStore: the creation slug is write-if-absent — re-bootstrapping under a new name rewrites state.feature but NEVER the immutable record",
      second.identity.written === false && afterIdentity.feature === "created-as-this" && afterState.feature === "renamed-later",
      JSON.stringify({ second: second.identity, afterIdentity, stateFeature: afterState.feature }),
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

{
  const tmpRoot = tmpDir("worktree-store-identity-null-");
  try {
    const worktreeRoot = path.join(tmpRoot, "wt");
    const mainStoreRoot = path.join(tmpRoot, "main", ".bee");
    fs.mkdirSync(worktreeRoot, { recursive: true });
    fs.mkdirSync(mainStoreRoot, { recursive: true });

    const result = bootstrapWorktreeStore(worktreeRoot, mainStoreRoot, null);
    const identityFile = path.join(worktreeRoot, ".bee", "runtime", "worktree-identity.json");
    record(
      "bootstrapWorktreeStore: a null feature records no creation slug and never throws — the reader falls back to state.feature exactly as before",
      result.created === true && result.identity.written === false && !fs.existsSync(identityFile),
      JSON.stringify(result.identity),
    );
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// ---------------------------------------------------------------------------
// summary
// ---------------------------------------------------------------------------

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length === 0 ? 0 : 7);
