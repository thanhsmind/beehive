#!/usr/bin/env node
// test_bypass_matrix.mjs — consistency guard for the bypass-level matrix
// hand-maintained in TWO places (README.md and skills/bee-bypass-gate/SKILL.md),
// per i54-closeout D6: "a verify suite parses the off/normal/full/total
// matrices ... and fails on semantic drift. A canonical generator is
// deliberately NOT built this round." (Seam #54: "two sources of truth" with
// no guard against drift.)
//
// Each matrix expresses, per level (off/normal/full/total), whether FOUR
// things auto-approve: (1) Gates 1-3, (2) high-risk/hard-gate work,
// (3) Gate 4 UAT & P1 findings, (4) secret-file reads. The two files use
// DIFFERENT table shapes (README: one column per flag; SKILL.md: a combined
// "Auto-approves" prose column + a combined "Still stops" prose column), so
// this parses each shape into the same {level -> {gates13, highRisk, gate4P1,
// secret}} semantic tuple and diffs tuple-by-tuple. Cosmetic differences
// (bold markers, wording, column layout) must NOT trip this — only an actual
// disagreement about what a level does.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const README_PATH = path.join(REPO_ROOT, "README.md");
const SKILL_PATH = path.join(REPO_ROOT, "skills/bee-bypass-gate/SKILL.md");

const LEVELS = ["off", "normal", "full", "total"];
const FLAGS = ["gates13", "highRisk", "gate4P1", "secret"];

let failed = 0;
const fail = (msg) => {
  failed += 1;
  console.log(`FAIL  ${msg}`);
};
const ok = (msg) => console.log(`ok    ${msg}`);

// ── row extraction (shape-agnostic) ─────────────────────────────────────────
// Matches a markdown table row whose first cell is a backticked level name,
// e.g. `| \`normal\` | ... |`, and returns the remaining cells (split on the
// table's own `|` delimiters, trimmed). Both files use exactly this shape for
// their level rows and use `·` (not `|`) as an in-cell separator, so a naive
// split on `|` is safe here.
function extractLevelRows(text) {
  const rows = new Map();
  const lineRe = /^\|\s*`(off|normal|full|total)`\s*\|(.*)\|\s*$/;
  for (const line of text.split("\n")) {
    const m = line.match(lineRe);
    if (!m) continue;
    const [, level, rest] = m;
    const cells = rest.split("|").map((c) => c.trim());
    rows.set(level, { raw: line.trim(), cells });
  }
  return rows;
}

// Strip markdown emphasis/code markers and lowercase, for tolerant keyword matching.
function normalize(cell) {
  return cell.replace(/\*\*/g, "").replace(/`/g, "").toLowerCase().trim();
}

// ── README shape: one dedicated column per flag ─────────────────────────────
// Columns after the level: [Gates 1-3, High-risk/hard-gate, Gate 4 UAT & P1, Secret reads]
function classifyReadmeCell(cellText) {
  const t = normalize(cellText);
  if (t.includes("auto")) return true; // "auto", "auto (normal lanes)", "auto — nothing stops"
  if (t.includes("approve")) return false; // "you approve"
  if (t.includes("stop")) return false; // "stops"
  if (t.includes("ask")) return false; // "asks"
  return null; // unrecognized cell text — treated as a parse failure by the caller
}

function parseReadmeMatrix(text) {
  const rows = extractLevelRows(text);
  const result = new Map();
  for (const level of LEVELS) {
    const row = rows.get(level);
    if (!row) continue;
    const [gates13Cell, highRiskCell, gate4P1Cell, secretCell] = row.cells;
    const tuple = {
      gates13: classifyReadmeCell(gates13Cell ?? ""),
      highRisk: classifyReadmeCell(highRiskCell ?? ""),
      gate4P1: classifyReadmeCell(gate4P1Cell ?? ""),
      secret: classifyReadmeCell(secretCell ?? ""),
    };
    result.set(level, { tuple, raw: row.raw });
  }
  return result;
}

// ── SKILL.md shape: one combined "Auto-approves" column + one combined
// "Still stops for the human" column, in prose. Determine each flag by
// keyword presence, resolving each flag against WHICHEVER column names it
// (auto-approves -> true, stops -> false); a flag named in neither column
// falls back to the row-level "nothing"/"everything" shortcut.
const FLAG_KEYWORDS = {
  gates13: ["gates 1-3", "gates1-3"],
  highRisk: ["high-risk", "hard-gate"],
  gate4P1: ["gate 4", "uat", "p1"],
  secret: ["secret"],
};

function classifySkillRow(approvesCell, stopsCell) {
  const approvesRaw = normalize(approvesCell);
  const stopsRaw = normalize(stopsCell);

  // Row-level shortcuts: "nothing" auto-approved / "every[thing]" stops both
  // collapse to all-false; "everything" auto-approved collapses to all-true.
  if (approvesRaw === "nothing" || /\bevery\b/.test(stopsRaw)) {
    return { gates13: false, highRisk: false, gate4P1: false, secret: false };
  }
  if (approvesRaw.includes("everything")) {
    return { gates13: true, highRisk: true, gate4P1: true, secret: true };
  }

  // Strip negated qualifiers ("non-hard-gate work") so they don't register as
  // a positive mention of the flag they negate.
  const approves = approvesRaw.replace(/non-[\w-]+/g, "");
  // Strip flag mentions that are themselves scoped BY another flag ("high-risk/
  // hard-gate Gates 1-3" = the hard-gate subset of Gates 1-3 stops — that is
  // evidence for highRisk, not a standalone claim about Gates 1-3 in general).
  const stops = stopsRaw.replace(/(high-risk\/hard-gate|high-risk|hard-gate)\s+gates\s*1-3/g, "$1");

  const tuple = {};
  for (const flag of FLAGS) {
    const keywords = FLAG_KEYWORDS[flag];
    const inApproves = keywords.some((k) => approves.includes(k));
    const inStops = keywords.some((k) => stops.includes(k));
    if (inApproves && !inStops) tuple[flag] = true;
    else if (inStops && !inApproves) tuple[flag] = false;
    else tuple[flag] = null; // ambiguous or unmentioned — parse failure
  }
  return tuple;
}

function parseSkillMatrix(text) {
  const rows = extractLevelRows(text);
  const result = new Map();
  for (const level of LEVELS) {
    const row = rows.get(level);
    if (!row) continue;
    // cells: [gate_bypass value, Auto-approves, Still stops for the human]
    const [, approvesCell, stopsCell] = row.cells;
    const tuple = classifySkillRow(approvesCell ?? "", stopsCell ?? "");
    result.set(level, { tuple, raw: row.raw });
  }
  return result;
}

function tupleLabel(tuple) {
  return FLAGS.map((f) => `${f}=${tuple[f]}`).join(" ");
}

function diffMatrices(a, aName, b, bName) {
  let totalFailed = 0;
  for (const level of LEVELS) {
    let levelFailed = 0;
    const aRow = a.get(level);
    const bRow = b.get(level);
    if (!aRow) {
      fail(`${aName}: missing a "${level}" row in its bypass matrix`);
      levelFailed += 1;
    }
    if (!bRow) {
      fail(`${bName}: missing a "${level}" row in its bypass matrix`);
      levelFailed += 1;
    }
    if (aRow && bRow) {
      for (const flag of FLAGS) {
        const av = aRow.tuple[flag];
        const bv = bRow.tuple[flag];
        if (av === null) {
          fail(`${aName}: level "${level}" — could not classify flag "${flag}" from row: ${aRow.raw}`);
          levelFailed += 1;
          continue;
        }
        if (bv === null) {
          fail(`${bName}: level "${level}" — could not classify flag "${flag}" from row: ${bRow.raw}`);
          levelFailed += 1;
          continue;
        }
        if (av !== bv) {
          fail(
            `bypass matrix drift at level "${level}", flag "${flag}": ` +
              `${aName} reads ${flag}=${av} (row: ${aRow.raw}) but ${bName} reads ${flag}=${bv} (row: ${bRow.raw})`,
          );
          levelFailed += 1;
        }
      }
    }
    if (levelFailed === 0 && aRow) {
      ok(`level "${level}" agrees across both matrices: ${tupleLabel(aRow.tuple)}`);
    }
    totalFailed += levelFailed;
  }
  return totalFailed;
}

// ── load real files ──────────────────────────────────────────────────────────
let readmeText = "";
let skillText = "";
try {
  readmeText = fs.readFileSync(README_PATH, "utf8");
} catch {
  fail(`${README_PATH}: unreadable — the bypass matrix lives here`);
}
try {
  skillText = fs.readFileSync(SKILL_PATH, "utf8");
} catch {
  fail(`${SKILL_PATH}: unreadable — the bypass matrix lives here`);
}

if (readmeText && skillText) {
  const readmeMatrix = parseReadmeMatrix(readmeText);
  const skillMatrix = parseSkillMatrix(skillText);
  diffMatrices(readmeMatrix, "README.md", skillMatrix, "skills/bee-bypass-gate/SKILL.md");
}

// ── sentinel: prove the differ bites, and prove cosmetic noise is tolerated ──
// (a) A genuinely drifted pair (README says "normal" auto-approves high-risk;
// SKILL.md says it still stops) must be flagged.
{
  const goodReadme = parseReadmeMatrix(
    "| Level | Gates 1–3 | High-risk / hard-gate | Gate 4 UAT & P1 | Secret reads |\n" +
      "|---|---|---|---|---|\n" +
      "| `normal` | auto (normal lanes) | **stops** | **stops** | **asks** |\n",
  );
  const driftedSkill = parseSkillMatrix(
    "| Level | `gate_bypass` | Auto-approves | Still stops for the human |\n" +
      "|---|---|---|---|\n" +
      "| `normal` | `true` | Gates 1-3 for `tiny`/`small`/`standard` work, high-risk/hard-gate included | secret reads · Gate 4 UAT/P1 |\n",
  );
  // diffMatricesSilently never touches the real `failed` counter or console —
  // it's the pure comparison core, safe to run against synthetic fixtures.
  const realDiff = diffMatricesSilently(goodReadme, "sentinel-README", driftedSkill, "sentinel-SKILL");
  const sentinelCaught = realDiff.some((m) => m.includes('flag "highRisk"'));
  if (!sentinelCaught) {
    fail("sentinel: the differ does NOT catch a genuine high-risk-flag divergence — the guard is fail-open");
  } else {
    ok("sentinel: a genuine high-risk-flag divergence between matrices is correctly caught");
  }
}

// (b) Cosmetic-only differences (extra whitespace, different bold placement,
// synonym wording) between two representations of the SAME semantics must NOT
// be flagged.
{
  const readmeCosmetic = parseReadmeMatrix(
    "| Level | Gates 1–3 | High-risk / hard-gate | Gate 4 UAT & P1 | Secret reads |\n" +
      "|---|---|---|---|---|\n" +
      "|  `full`  |  auto  |  **auto**  |  **stops**  |  **asks**  |\n",
  );
  const skillCosmetic = parseSkillMatrix(
    "| Level | `gate_bypass` | Auto-approves | Still stops for the human |\n" +
      "|---|---|---|---|\n" +
      '| `full` | `"full"` | **all** Gates 1-3 at every lane, high-risk/hard-gate included | secret-file reads · a review P1 finding |\n',
  );
  const cosmeticDiffs = diffMatricesSilently(readmeCosmetic, "cosmetic-README", skillCosmetic, "cosmetic-SKILL");
  if (cosmeticDiffs.length > 0) {
    fail(`sentinel: cosmetic-only formatting differences were incorrectly flagged as drift: ${cosmeticDiffs.join(" | ")}`);
  } else {
    ok("sentinel: cosmetic-only formatting differences are correctly tolerated (no false positive)");
  }
}

// Silent variant of diffMatrices used only by the sentinels above, so their
// synthetic fixtures never affect the real pass/fail count or console noise
// beyond a single summary line.
function diffMatricesSilently(a, aName, b, bName) {
  const messages = [];
  for (const level of LEVELS) {
    const aRow = a.get(level);
    const bRow = b.get(level);
    if (!aRow || !bRow) continue;
    for (const flag of FLAGS) {
      const av = aRow.tuple[flag];
      const bv = bRow.tuple[flag];
      if (av === null || bv === null) continue;
      if (av !== bv) {
        messages.push(`level "${level}" flag "${flag}": ${aName}=${av} vs ${bName}=${bv}`);
      }
    }
  }
  return messages;
}

console.log(`\n${failed === 0 ? "PASS" : "FAIL"} - bypass matrix consistency: ${failed} failure(s)`);
process.exit(failed > 0 ? 1 : 0);
