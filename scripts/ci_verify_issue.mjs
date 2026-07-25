#!/usr/bin/env node
// ci_verify_issue.mjs — cov-3 (ci-owned-verify CONTEXT.md D2): when CI's
// verify chain (scripts/verify_all.mjs, run by .github/workflows/ci.yml)
// goes red, this files (or, deduped, comments on) a `verify-red` GitHub
// issue carrying the failed-suite tail.
//
// Split deliberately into a pure decision core (title/body construction +
// the create-vs-comment dedupe decision) and a thin CLI shell that reads
// GitHub Actions env vars and drives `gh` through an INJECTABLE exec
// function — so the decision logic is unit-testable offline, with no
// network and no real `gh` binary required (scripts/tests/test_ci_verify_issue.mjs).
//
// Pure core:
//   buildIssueTitle(workflow, branch)        -> deterministic issue title
//   buildIssueBody(failTail, runUrl, sha)     -> deterministic issue body
//   decideAction(openIssues)                  -> { action: "create" }
//                                                 | { action: "comment", issueNumber }
//
// Dedupe rule (D2): an OPEN issue whose title exactly matches
// buildIssueTitle(workflow, branch) already exists for this workflow+branch
// -> comment on it, never create a duplicate. No matching open issue ->
// create one, labeled `verify-red`.
//
// CLI mode (invoked by ci.yml on failure):
//   env GITHUB_WORKFLOW, GITHUB_REF_NAME, GITHUB_SHA required;
//   GITHUB_RUN_URL used if set, else constructed from GITHUB_SERVER_URL +
//   GITHUB_REPOSITORY + GITHUB_RUN_ID; VERIFY_TAIL_FILE points at the
//   captured verify-chain output (tail is what lands in the issue body).
//   `gh` itself must be authenticated via GITHUB_TOKEN in the environment
//   (ci.yml sets it) — this script never touches the token directly, `gh`
//   reads it itself.

import fs from "node:fs";
import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";

export const VERIFY_RED_LABEL = "verify-red";
const LABEL_COLOR = "b60205"; // GitHub's stock "bug" red
const TAIL_LINE_COUNT = 200;

// ─── pure decision core ─────────────────────────────────────────────────

export function buildIssueTitle(workflow, branch) {
  return `verify-red: ${workflow} failing on ${branch}`;
}

export function buildIssueBody(failTail, runUrl, sha) {
  const tail = (failTail ?? "").trim() || "(no output captured)";
  return [
    "CI's verify chain is red.",
    "",
    `- Commit: \`${sha}\``,
    `- Run: ${runUrl}`,
    "",
    "<details><summary>Failure tail</summary>",
    "",
    "```",
    tail,
    "```",
    "",
    "</details>",
    "",
    `_Filed automatically by \`scripts/ci_verify_issue.mjs\` — labeled \`${VERIFY_RED_LABEL}\`._`,
  ].join("\n");
}

// openIssues: array of { number, title } for OPEN issues already narrowed
// to this workflow+branch's exact title by the caller (CLI layer does the
// `gh issue list` + title filter; this function makes no gh/network call).
export function decideAction(openIssues) {
  if (!Array.isArray(openIssues) || openIssues.length === 0) {
    return { action: "create" };
  }
  // Multiple matches would themselves be a dedupe failure from an earlier
  // run; deterministically pick the lowest issue number (the original) so
  // repeated red runs always converge on the same thread.
  const sorted = [...openIssues].sort((a, b) => a.number - b.number);
  return { action: "comment", issueNumber: sorted[0].number };
}

// Keep only the last N lines — GitHub issue bodies have a size limit and a
// full suite's failure output can run to thousands of lines; the tail is
// what a human needs to triage which suite broke.
export function tailLines(text, n = TAIL_LINE_COUNT) {
  const lines = (text ?? "").split("\n");
  if (lines.length <= n) return lines.join("\n");
  return lines.slice(lines.length - n).join("\n");
}

// ─── gh-driving orchestration (exec injected — no direct gh/network call) ──
//
// `exec` signature: (args: string[]) => { stdout: string } — throws (or
// rejects) on a non-zero `gh` exit, matching child_process's execFileSync
// contract so the real CLI exec function needs no adapter.

export async function fileVerifyIssue({ workflow, branch, failTail, runUrl, sha, exec }) {
  const title = buildIssueTitle(workflow, branch);
  const body = buildIssueBody(failTail, runUrl, sha);

  // Tolerate-existing: label creation races/duplicates across concurrent
  // runs or prior runs are expected and not fatal — only a missing label
  // on the first-ever run matters, and `gh issue create --label` fails
  // outright if the label doesn't exist yet.
  try {
    await exec(["label", "create", VERIFY_RED_LABEL, "--color", LABEL_COLOR, "--description", "CI verify chain is red"]);
  } catch {
    // already exists (or any other create hiccup) — non-fatal, proceed.
  }

  const listResult = await exec([
    "issue", "list",
    "--state", "open",
    "--label", VERIFY_RED_LABEL,
    "--json", "number,title",
  ]);
  let allOpen = [];
  try {
    const parsed = JSON.parse(listResult?.stdout ?? "[]");
    if (Array.isArray(parsed)) allOpen = parsed;
  } catch {
    allOpen = []; // unparseable listing degrades to "no known match" -> create
  }
  const matching = allOpen.filter((issue) => issue.title === title);

  const decision = decideAction(matching);
  if (decision.action === "create") {
    await exec(["issue", "create", "--title", title, "--body", body, "--label", VERIFY_RED_LABEL]);
  } else {
    await exec(["issue", "comment", String(decision.issueNumber), "--body", body]);
  }
  return decision;
}

// ─── CLI shell ──────────────────────────────────────────────────────────

function realExec(args) {
  const stdout = execFileSync("gh", args, { encoding: "utf8" });
  return { stdout };
}

function buildRunUrl(env) {
  if (env.GITHUB_RUN_URL) return env.GITHUB_RUN_URL;
  if (env.GITHUB_SERVER_URL && env.GITHUB_REPOSITORY && env.GITHUB_RUN_ID) {
    return `${env.GITHUB_SERVER_URL}/${env.GITHUB_REPOSITORY}/actions/runs/${env.GITHUB_RUN_ID}`;
  }
  return "(run URL unavailable)";
}

function readTail(env) {
  const tailFile = env.VERIFY_TAIL_FILE;
  if (!tailFile) return "(VERIFY_TAIL_FILE not set — no output captured)";
  try {
    return tailLines(fs.readFileSync(tailFile, "utf8"));
  } catch (err) {
    return `(failed to read VERIFY_TAIL_FILE ${tailFile}: ${err.message})`;
  }
}

async function mainCli() {
  const env = process.env;
  const workflow = env.GITHUB_WORKFLOW || "CI";
  const branch = env.GITHUB_REF_NAME || "unknown";
  const sha = env.GITHUB_SHA || "unknown";
  const runUrl = buildRunUrl(env);
  const failTail = readTail(env);

  const decision = await fileVerifyIssue({
    workflow,
    branch,
    failTail,
    runUrl,
    sha,
    exec: realExec,
  });

  if (decision.action === "create") {
    console.log(`ci_verify_issue: filed a new ${VERIFY_RED_LABEL} issue for ${workflow} @ ${branch}`);
  } else {
    console.log(`ci_verify_issue: commented on existing #${decision.issueNumber} (${workflow} @ ${branch})`);
  }
}

const isMain = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url;
if (isMain) {
  mainCli().catch((err) => {
    console.error(`ci_verify_issue: ${err.stack ?? err.message}`);
    process.exit(1);
  });
}
