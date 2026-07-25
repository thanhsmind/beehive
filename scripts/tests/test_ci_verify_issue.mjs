#!/usr/bin/env node
// test_ci_verify_issue.mjs — proves scripts/ci_verify_issue.mjs (cov-3,
// ci-owned-verify D2): deterministic title/body, the dedupe decision
// (comment on an existing open match, never duplicate-create), the
// `verify-red` label constant, and the shapes of every `gh` call the
// orchestration issues — all offline, via an injected exec function. No
// network, no real `gh` binary is invoked anywhere in this suite.

import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const MODULE_PATH = path.join(__dirname, "..", "ci_verify_issue.mjs");

const {
  buildIssueTitle,
  buildIssueBody,
  decideAction,
  tailLines,
  fileVerifyIssue,
  VERIFY_RED_LABEL,
} = await import(pathToFileURL(MODULE_PATH).href);

let passed = 0;
let failed = 0;
async function check(name, fn) {
  try {
    await fn();
    passed += 1;
    console.log(`PASS ${name}`);
  } catch (error) {
    failed += 1;
    console.error(`FAIL ${name}: ${error.stack ?? error.message}`);
  }
}

// ── title determinism ──────────────────────────────────────────────────
await check("buildIssueTitle is deterministic and carries workflow + branch", async () => {
  const a = buildIssueTitle("CI", "main");
  const b = buildIssueTitle("CI", "main");
  assert.equal(a, b, "same inputs must produce byte-identical titles");
  assert.match(a, /CI/, "title must carry the workflow name");
  assert.match(a, /main/, "title must carry the branch name");
});

await check("buildIssueTitle distinguishes branches (no cross-branch dedupe)", async () => {
  const main = buildIssueTitle("CI", "main");
  const feature = buildIssueTitle("CI", "wt/verify-scoping");
  assert.notEqual(main, feature, "different branches must not collide on the same title");
});

// ── body determinism + content ─────────────────────────────────────────
await check("buildIssueBody is deterministic and embeds tail, run URL, and sha", async () => {
  const a = buildIssueBody("boom: suite X failed", "https://example.invalid/run/1", "deadbeef");
  const b = buildIssueBody("boom: suite X failed", "https://example.invalid/run/1", "deadbeef");
  assert.equal(a, b, "same inputs must produce byte-identical bodies");
  assert.match(a, /boom: suite X failed/, "body must embed the failure tail");
  assert.match(a, /https:\/\/example\.invalid\/run\/1/, "body must embed the run URL");
  assert.match(a, /deadbeef/, "body must embed the commit sha");
  assert.match(a, new RegExp(VERIFY_RED_LABEL), "body should reference the label it was filed under");
});

await check("buildIssueBody tolerates an empty tail without throwing", async () => {
  const body = buildIssueBody("", "https://example.invalid/run/1", "deadbeef");
  assert.match(body, /no output captured/, "an empty tail must fall back to an explicit placeholder, not blank");
});

// ── tailLines ───────────────────────────────────────────────────────────
await check("tailLines keeps only the last N lines", async () => {
  const text = Array.from({ length: 10 }, (_, i) => `line-${i}`).join("\n");
  const result = tailLines(text, 3);
  assert.equal(result, "line-7\nline-8\nline-9");
});

await check("tailLines returns the whole text when under the limit", async () => {
  const text = "a\nb\nc";
  assert.equal(tailLines(text, 200), text);
});

// ── decideAction: THE HEADLINE dedupe decision ──────────────────────────
// This is the red-first assertion: a naive "always create" implementation
// (or one that picks an arbitrary/unsorted match) fails this exact case.
await check("decideAction: no matching open issue -> create (never comment blind)", async () => {
  const decision = decideAction([]);
  assert.deepEqual(decision, { action: "create" });
});

await check("decideAction: one matching open issue -> comment on it, never a duplicate create", async () => {
  const decision = decideAction([{ number: 42, title: "verify-red: CI failing on main" }]);
  assert.deepEqual(decision, { action: "comment", issueNumber: 42 });
});

await check("decideAction: multiple matches (a prior dedupe miss) converge on the lowest (original) issue number", async () => {
  const decision = decideAction([
    { number: 99, title: "verify-red: CI failing on main" },
    { number: 7, title: "verify-red: CI failing on main" },
    { number: 55, title: "verify-red: CI failing on main" },
  ]);
  assert.deepEqual(decision, { action: "comment", issueNumber: 7 });
});

await check("decideAction: non-array input degrades safely to create", async () => {
  assert.deepEqual(decideAction(undefined), { action: "create" });
  assert.deepEqual(decideAction(null), { action: "create" });
});

// ── fileVerifyIssue: gh call shapes, via a recording injected exec ──────

function makeRecordingExec(script) {
  // script: array of handlers keyed by the gh subcommand (args[0]); each
  // handler receives the full args array and returns/throws what the real
  // `gh` CLI would for that call, letting each test simulate a scenario
  // (label already exists, no open issues, one open issue, ...) without
  // ever shelling out.
  const calls = [];
  const exec = async (args) => {
    calls.push(args);
    const handler = script[args[0]];
    if (!handler) throw new Error(`unscripted gh call: ${args.join(" ")}`);
    return handler(args);
  };
  return { exec, calls };
}

await check("fileVerifyIssue: no open verify-red issue -> creates a labeled issue, tolerating a pre-existing label", async () => {
  const { exec, calls } = makeRecordingExec({
    label: () => {
      throw new Error("HTTP 422: Label \"verify-red\" already exists");
    },
    issue: (args) => {
      if (args[1] === "list") return { stdout: "[]" };
      if (args[1] === "create") return { stdout: "https://example.invalid/issues/1" };
      throw new Error(`unscripted issue subcommand: ${args.join(" ")}`);
    },
  });

  const decision = await fileVerifyIssue({
    workflow: "CI",
    branch: "main",
    failTail: "suite X failed",
    runUrl: "https://example.invalid/run/1",
    sha: "deadbeef",
    exec,
  });

  assert.deepEqual(decision, { action: "create" });

  const labelCall = calls.find((c) => c[0] === "label");
  assert.ok(labelCall, "must attempt label creation");
  assert.deepEqual(labelCall.slice(0, 3), ["label", "create", VERIFY_RED_LABEL]);

  const listCall = calls.find((c) => c[0] === "issue" && c[1] === "list");
  assert.ok(listCall.includes("--label") && listCall.includes(VERIFY_RED_LABEL), "list must filter by the verify-red label");
  assert.ok(listCall.includes("--state") && listCall.includes("open"), "list must scope to open issues only");

  const createCall = calls.find((c) => c[0] === "issue" && c[1] === "create");
  assert.ok(createCall, "a create call must be issued when nothing matched");
  assert.ok(createCall.includes("--label") && createCall.includes(VERIFY_RED_LABEL), "created issue must carry the verify-red label");
  const titleIdx = createCall.indexOf("--title");
  assert.equal(createCall[titleIdx + 1], buildIssueTitle("CI", "main"));
});

await check("fileVerifyIssue: an existing matching open issue -> comments, never calls issue create", async () => {
  const existingTitle = buildIssueTitle("CI", "main");
  const { exec, calls } = makeRecordingExec({
    label: () => ({ stdout: "" }),
    issue: (args) => {
      if (args[1] === "list") {
        return { stdout: JSON.stringify([{ number: 17, title: existingTitle }]) };
      }
      if (args[1] === "comment") return { stdout: "" };
      if (args[1] === "create") throw new Error("must not be called — dedupe should comment instead");
      throw new Error(`unscripted issue subcommand: ${args.join(" ")}`);
    },
  });

  const decision = await fileVerifyIssue({
    workflow: "CI",
    branch: "main",
    failTail: "suite X failed again",
    runUrl: "https://example.invalid/run/2",
    sha: "cafef00d",
    exec,
  });

  assert.deepEqual(decision, { action: "comment", issueNumber: 17 });
  const createCall = calls.find((c) => c[0] === "issue" && c[1] === "create");
  assert.equal(createCall, undefined, "dedupe must prevent any issue-create call");
  const commentCall = calls.find((c) => c[0] === "issue" && c[1] === "comment");
  assert.ok(commentCall, "must comment on the matched issue");
  assert.equal(commentCall[2], "17", "must target the matched issue number");
});

await check("fileVerifyIssue: an open issue for a DIFFERENT workflow/branch is not treated as a match", async () => {
  const { exec, calls } = makeRecordingExec({
    label: () => ({ stdout: "" }),
    issue: (args) => {
      if (args[1] === "list") {
        return { stdout: JSON.stringify([{ number: 3, title: buildIssueTitle("CI", "some-other-branch") }]) };
      }
      if (args[1] === "create") return { stdout: "" };
      throw new Error(`unscripted issue subcommand: ${args.join(" ")}`);
    },
  });

  const decision = await fileVerifyIssue({
    workflow: "CI",
    branch: "main",
    failTail: "x",
    runUrl: "https://example.invalid/run/3",
    sha: "abc123",
    exec,
  });

  assert.deepEqual(decision, { action: "create" }, "an issue for a different branch must not dedupe this one");
  assert.ok(calls.some((c) => c[0] === "issue" && c[1] === "create"));
});

await check("fileVerifyIssue: an unparseable issue-list response degrades to create rather than throwing", async () => {
  const { exec } = makeRecordingExec({
    label: () => ({ stdout: "" }),
    issue: (args) => {
      if (args[1] === "list") return { stdout: "not json" };
      if (args[1] === "create") return { stdout: "" };
      throw new Error(`unscripted issue subcommand: ${args.join(" ")}`);
    },
  });

  const decision = await fileVerifyIssue({
    workflow: "CI",
    branch: "main",
    failTail: "x",
    runUrl: "https://example.invalid/run/4",
    sha: "abc123",
    exec,
  });
  assert.deepEqual(decision, { action: "create" });
});

console.log(`\n${passed} passed, ${failed} failed`);
if (failed > 0) process.exit(1);
