#!/usr/bin/env node
// Census: stale Codex spawn syntax must not survive on any live skills surface
// (SPEC docs/history/codex-harness-hardening/SPEC.md §9.1 ORCH-01, finding E-06).
// Codex's collaboration API dropped spawn_agent(agent_type=..., fork_context=...)
// and the "re-spawn to continue" pattern; any surviving prose describing that
// contract would mislead a Codex-runtime worker into calling an API that no
// longer exists. This is a red-now census: it is expected to fail today
// (skills/bee-swarming/references/swarming-reference.md:17,20,22 still carry
// the old syntax) and will go green once a later slice rewrites that doc.
//
// Modeled on the "retired auto-review-trigger phrasing" census
// (packages/bee/tests/test_lib.mjs, review-od-7): scan only live
// operative prose, exclude build logs and decision archaeology, and never
// widen the scan into files this script itself lives outside of.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptPath = fileURLToPath(import.meta.url);
const repoRoot = path.dirname(path.dirname(scriptPath)); // scripts/ -> repo root

// ─── scan set: skills/**/SKILL.md + skills/**/references/**/*.md only ──────
// Explicit scope fence (SPEC §9.1 ORCH-01 / cell must_haves):
//   - *-LOG.md excluded (e.g. skills/bee-swarming/CREATION-LOG.md — a build
//     log recording what was done, not operative guidance a worker reads).
//   - docs/history/** and docs/decisions/** excluded — decision archaeology,
//     never live instructions.
//   - the SPEC itself (docs/history/codex-harness-hardening/SPEC.md) is
//     covered by the docs/history/** exclusion above, and is excluded again
//     explicitly here for clarity since it is the very document naming the
//     stale tokens as a finding, not a place the tokens should be censused.
const skillsRoot = path.join(repoRoot, "skills");

function walkMarkdown(dir, out) {
  if (!fs.existsSync(dir)) return out;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      walkMarkdown(full, out);
    } else if (entry.isFile() && entry.name.endsWith(".md") && !entry.name.endsWith("-LOG.md")) {
      out.push(full);
    }
  }
  return out;
}

function collectScanFiles() {
  const files = [];
  if (!fs.existsSync(skillsRoot)) return files;
  for (const entry of fs.readdirSync(skillsRoot, { withFileTypes: true })) {
    if (!entry.isDirectory()) continue;
    const skillDir = path.join(skillsRoot, entry.name);

    const skillMd = path.join(skillDir, "SKILL.md");
    if (fs.existsSync(skillMd) && !path.basename(skillMd).endsWith("-LOG.md")) {
      files.push(skillMd);
    }

    const referencesDir = path.join(skillDir, "references");
    walkMarkdown(referencesDir, files);
  }
  return files;
}

// ─── stale-token matchers ───────────────────────────────────────────────────
// Two literal substrings named directly in the cell contract, plus a pattern
// for the re-spawn-as-continuation phrasing (e.g. "Re-`spawn_agent` with
// enriched context") which is prose, not a fixed literal.
const LITERAL_TOKENS = ["spawn_agent(agent_type", "fork_context"];
const RESPAWN_CONTINUATION = /re-`?spawn_agent`?\s+with\s+(?:enriched\s+)?context/i;

function findViolationsInFile(file) {
  const text = fs.readFileSync(file, "utf8");
  const lines = text.split("\n");
  const hits = [];

  lines.forEach((line, idx) => {
    const lineNo = idx + 1;
    for (const token of LITERAL_TOKENS) {
      if (line.includes(token)) {
        hits.push({ line: lineNo, token });
      }
    }
    const respawnMatch = line.match(RESPAWN_CONTINUATION);
    if (respawnMatch) {
      hits.push({ line: lineNo, token: respawnMatch[0] });
    }
  });

  return hits;
}

function main() {
  const scanFiles = collectScanFiles();
  const violations = [];

  for (const file of scanFiles) {
    const rel = path.relative(repoRoot, file);
    for (const hit of findViolationsInFile(file)) {
      violations.push({ file: rel, line: hit.line, token: hit.token });
    }
  }

  if (violations.length === 0) {
    console.log("census_stale_spawn_syntax: clean — no stale Codex spawn syntax found.");
    process.exit(0);
  }

  for (const v of violations) {
    // Exact sentinel prefix: a crash stack trace must never emit this line,
    // so callers can grep for it as unambiguous proof of a real hit.
    console.log(`CENSUS-VIOLATION ${v.file}:${v.line} ${v.token}`);
  }
  console.log(`census_stale_spawn_syntax: ${violations.length} violation(s) found.`);
  process.exit(1);
}

main();
