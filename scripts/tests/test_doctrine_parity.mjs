#!/usr/bin/env node
// Doctrine parity gate — compaction-hardening D22/D25.
//
// Two things are guarded here, and both exist because prose drifted away from
// the tree while every suite stayed green:
//
//   (1) THE RETIRED SENTENCE. The guardrails paragraph used to say hooks
//       enforce on Claude Code while "on Codex you must honor them yourself".
//       Both runtimes have shipped hooks from one shared catalog for several
//       releases; what is actually unverified is whether an installed Codex
//       CLI *executes* them. cz-1 replaced that sentence. This gate asserts it
//       never comes back — in `**/AGENTS.block.md` AND in the merged root
//       `AGENTS.md`, which the glob does not match and which is the file every
//       session actually loads (D22: a gate globbing only the template would
//       go green with the retired sentence live in the file that matters).
//
//   (2) THE FOUR COUNTS. Four hook counts exist and all four are true of
//       different things (D25):
//         9  hook scripts           — glob of hooks/bee-*.mjs
//         8  Codex lifecycle events — Object.keys(json.hooks) of .codex/hooks.json
//         7  Claude lifecycle events— Object.keys(json.hooks) of hooks/claude-hooks.json
//         6  config toggles         — Object.keys(config.hooks) of .bee/config.json
//       All four are DERIVED at check time, never hand-written here. A prose
//       claim of "six hooks" is TRUE when it means toggles and FALSE when it
//       means scripts, so the classification rule below is deliberately narrow:
//       it checks only claims whose noun names one quantity, and REPORTS the
//       ones it cannot tell apart instead of guessing. A gate that guesses
//       would drive a future worker to falsify a correct document.
//
// crit-pattern 20260722 (a coverage gate derives its ground truth) and
// crit-pattern 20260723 (a scan scope set from assumption passes green while
// hiding the bug): the scan set, the file set, and all four counts are derived;
// every exclusion below is stated with the measurement that justifies it.
//
// Usage:
//   node scripts/tests/test_doctrine_parity.mjs             # self-test, then real check
//   node scripts/tests/test_doctrine_parity.mjs --selftest  # self-test only

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..", "..");
const NAME = "test_doctrine_parity";

// ─── the retired doctrine sentence ────────────────────────────────────────
// Both halves are retired: the first asserts hooks are a Claude-Code-only
// mechanism, the second tells Codex sessions they have no hooks at all. Either
// half reappearing is the regression. Matched case-insensitively against
// whitespace-normalised text, so re-wrapping does not evade it.
const RETIRED_SENTENCES = [
  "On Claude Code these are enforced mechanically by hooks",
  "on Codex you must honor them yourself",
];

// ─── scan-set exclusions (the only hand-written part of the scope) ────────
// docs/history/** is the per-feature archive: CONTEXT.md files quote the
// retired sentence verbatim as the thing being retired, and old plans record
// counts that were true when written. Freezing history is the point of that
// tree, so a doctrine gate must not police it. Every other tracked markdown
// file is in scope, derived from `git ls-files`.
const SCAN_EXCLUSION_PREFIXES = ["docs/history/"];

// ─── count derivation ─────────────────────────────────────────────────────

/**
 * Reads the `hooks` MAP out of a hook catalog and returns its lifecycle-event
 * key count. Object.keys() on the raw parsed file yields ['hooks'] — length 1
 * — which is the exact derivation bug D22 records; hence the explicit shape
 * check rather than a bare Object.keys(json).
 */
function lifecycleEventCount(file) {
  const json = JSON.parse(fs.readFileSync(file, "utf8"));
  const map = json.hooks;
  if (!map || typeof map !== "object" || Array.isArray(map)) {
    throw new Error(`${file}: expected a top-level "hooks" object mapping lifecycle events to entries`);
  }
  const keys = Object.keys(map);
  if (keys.length === 0) throw new Error(`${file}: "hooks" map is empty — derivation would assert nothing`);
  if (keys.length === 1 && keys[0] === "hooks") {
    throw new Error(`${file}: derived the wrapper, not the event map (see D22)`);
  }
  return keys.length;
}

/** The four counts, every one measured from the tree at check time. */
function deriveCounts(root) {
  const scripts = fs
    .readdirSync(path.join(root, "hooks"))
    .filter((n) => /^bee-.*\.mjs$/.test(n));
  if (scripts.length === 0) throw new Error("hooks/bee-*.mjs matched nothing — derivation would assert nothing");

  const configHooks = JSON.parse(fs.readFileSync(path.join(root, ".bee", "config.json"), "utf8")).hooks;
  if (!configHooks || typeof configHooks !== "object" || Array.isArray(configHooks)) {
    throw new Error('.bee/config.json: expected a "hooks" object of per-hook toggles');
  }
  const toggles = Object.keys(configHooks).length;
  if (toggles === 0) throw new Error(".bee/config.json: hooks toggle map is empty");

  return {
    scripts: scripts.length,
    codexEvents: lifecycleEventCount(path.join(root, ".codex", "hooks.json")),
    claudeEvents: lifecycleEventCount(path.join(root, "hooks", "claude-hooks.json")),
    toggles,
  };
}

// ─── scan-set derivation ──────────────────────────────────────────────────

/**
 * Every tracked markdown file, minus the stated exclusions. -z because the
 * repo genuinely contains tracked paths with spaces in them (docs/REFs/...),
 * which a newline-split of `git ls-files` mangles into nonexistent paths.
 */
function deriveScanSet(root) {
  const raw = execFileSync("git", ["ls-files", "-z", "*.md"], {
    cwd: root,
    maxBuffer: 64 * 1024 * 1024,
  }).toString("utf8");
  return raw
    .split("\0")
    .filter(Boolean)
    .filter((p) => !SCAN_EXCLUSION_PREFIXES.some((prefix) => p.startsWith(prefix)))
    .sort();
}

/**
 * The doctrine file set: every AGENTS.block.md projection PLUS the merged root
 * AGENTS.md, named explicitly because the AGENTS.block.md glob does not match
 * it and it is the file every session loads (D22).
 */
function doctrineFileSet(scanSet) {
  const blocks = scanSet.filter((p) => /(^|\/)AGENTS\.block\.md$/.test(p));
  const rootAgents = scanSet.filter((p) => p === "AGENTS.md");
  return { blocks, rootAgents, all: [...rootAgents, ...blocks] };
}

// ─── retired-sentence check ───────────────────────────────────────────────

const normalise = (s) => s.replace(/\s+/g, " ").trim().toLowerCase();

function findRetiredSentences({ root, files }) {
  const hits = [];
  for (const rel of files) {
    const text = fs.readFileSync(path.join(root, rel), "utf8");
    const lines = text.split("\n");
    const flatFile = normalise(text);
    for (const sentence of RETIRED_SENTENCES) {
      const needle = normalise(sentence);
      if (!flatFile.includes(needle)) continue;
      const lineNo = lines.findIndex((l) => normalise(l).includes(needle));
      hits.push({ file: rel, line: lineNo === -1 ? null : lineNo + 1, sentence });
    }
  }
  return hits;
}

// ─── count-claim classification ───────────────────────────────────────────
//
// THE RULE, stated narrowly on purpose:
//
//   A numeric claim is CHECKED only when the number is directly adjacent to a
//   noun that names exactly one of the four measured quantities:
//       "<n> script(s)" / "<n> hook script(s)"   -> hook scripts
//       "<n> toggle(s)"                          -> config toggles
//       "<n> lifecycle event(s)" / "<n> event(s)"-> lifecycle events
//
//   A claim of "<n> hook(s)" is NEVER checked. The bare word names no single
//   quantity, and the tree proves it is genuinely polysemous: measured live
//   uses include claudekit's "16-hook sprawl" (a competitor's count),
//   "exactly one hook source", "3 hooks take store lock" (a lock protocol),
//   and the TRUE six-toggle claim at docs/06-runtime-integration.md:121.
//   Checking bare "hook" against any of the four counts would flag every one
//   of those correct sentences. Instead they are REPORTED as unchecked, with
//   the four counts named — saying so rather than guessing.
//
//   Lifecycle-event claims resolve to a runtime by the markers present on the
//   line: only Codex markers -> 8; only Claude markers -> 7; both or neither
//   -> the reading is ambiguous, so BOTH counts are admissible. The admissible
//   set widens exactly as far as the prose is ambiguous and never further; a
//   claim fails only when it is false under every reading the prose permits.

const NUMBER_WORDS = {
  one: 1, two: 2, three: 3, four: 4, five: 5, six: 6, seven: 7, eight: 8,
  nine: 9, ten: 10, eleven: 11, twelve: 12, thirteen: 13, fourteen: 14,
  fifteen: 15, sixteen: 16, seventeen: 17, eighteen: 18, nineteen: 19,
  twenty: 20,
};
// Longest-first so "sixteen" is never chewed down to "six".
const NUMBER_ALT = [
  "\\d{1,3}",
  ...Object.keys(NUMBER_WORDS).sort((a, b) => b.length - a.length),
].join("|");
const GAP = "[\\s\\u00a0-]+";
const CHECKED_NOUNS = "hook scripts?|lifecycle events?|scripts?|toggles?|events?";
const CLAIM_RE = new RegExp(`\\b(${NUMBER_ALT})${GAP}(${CHECKED_NOUNS})\\b`, "gi");
const BARE_HOOK_RE = new RegExp(`\\b(${NUMBER_ALT})${GAP}(hooks?)\\b`, "gi");
// Near-miss probe: a number separated from a measured noun by EXACTLY ONE word
// — "6 lifecycle hooks", "the six core hooks", "three later scripts". Reported,
// never failed: one intervening word usually means the phrase names a subset or
// a different thing ("the three later scripts", "9 thin hooks"), so the gate
// declines to adjudicate but shows its work. The one-word window is what keeps
// this useful; a looser character window buried these few real cases under
// ~140 dates, rule numbers and version strings (measured).
const NEAR_MISS_RE = new RegExp(
  `\\b(${NUMBER_ALT})${GAP}\\w+${GAP}(${CHECKED_NOUNS}|hooks?)\\b(?!-)`,
  "gi",
);

const CODEX_MARKER = /\.codex\/hooks\.json|\bcodex\b/i;
const CLAUDE_MARKER = /claude-hooks\.json|\bclaude code\b/i;

const parseNumber = (raw) => {
  const t = raw.toLowerCase();
  return /^\d+$/.test(t) ? Number(t) : NUMBER_WORDS[t];
};

/**
 * Classifies one adjacency match into { quantity, admissible[] } — the set of
 * derived counts under which the claim is true. Every noun CLAIM_RE can match
 * has a branch here; an unmatched noun means the rule and the pattern have
 * drifted apart, which is a bug to surface rather than a claim to wave through.
 */
function classify(noun, line, counts) {
  const n = noun.toLowerCase();
  if (/script/.test(n)) return { quantity: "hook scripts", admissible: [counts.scripts] };
  if (/toggle/.test(n)) return { quantity: "config toggles", admissible: [counts.toggles] };
  if (/event/.test(n)) {
    const codex = CODEX_MARKER.test(line);
    const claude = CLAUDE_MARKER.test(line);
    if (codex && !claude) return { quantity: "Codex lifecycle events", admissible: [counts.codexEvents] };
    if (claude && !codex) return { quantity: "Claude lifecycle events", admissible: [counts.claudeEvents] };
    return {
      quantity: "lifecycle events (runtime not resolved by this line)",
      admissible: [counts.codexEvents, counts.claudeEvents],
    };
  }
  throw new Error(`classify: no branch for matched noun "${noun}" — CLAIM_RE and the rule have drifted`);
}

function scanCountClaims({ root, files, counts }) {
  const failures = [];
  const unchecked = [];
  const nearMisses = [];
  let checked = 0;

  for (const rel of files) {
    const lines = fs.readFileSync(path.join(root, rel), "utf8").split("\n");
    lines.forEach((line, idx) => {
      // Scope narrowing, measured not assumed: a count-claim about bee's hook
      // skeleton names hooks on the same line — all 23 genuine claims in the
      // tree do. Dropping this filter admitted 9 more candidates, every one of
      // them a false positive about something else ("three event kinds" of
      // decisions.jsonl, "406-event backfill", "Two scripts" of the status-line
      // pair, claudekit's "~16 scripts"). Known limit, stated rather than
      // heuristically patched: a foreign system's count written on a
      // hook-mentioning line (e.g. "claudekit's 16 hook scripts") would be
      // read as a claim about bee.
      if (!/hook/i.test(line)) return;
      // A near miss starting where a checked claim or a bare-hook claim already
      // starts is the SAME text seen through a looser pattern ("8 lifecycle
      // events" is both). Report each phrase once, under the strictest reading
      // that matched it — otherwise the report calls a checked claim unchecked.
      const claimedStarts = new Set();

      for (const m of line.matchAll(CLAIM_RE)) {
        const value = parseNumber(m[1]);
        if (value === undefined) continue;
        claimedStarts.add(m.index);
        const verdict = classify(m[2], line, counts);
        checked += 1;
        if (!verdict.admissible.includes(value)) {
          failures.push({
            file: rel,
            line: idx + 1,
            text: line.trim(),
            stated: value,
            phrase: `${m[1]} ${m[2]}`,
            quantity: verdict.quantity,
            admissible: verdict.admissible,
          });
        }
      }

      // Bare "<n> hook(s)" is detected purely so it can be reported, never judged.
      for (const m of line.matchAll(BARE_HOOK_RE)) {
        const value = parseNumber(m[1]);
        if (value === undefined) continue;
        claimedStarts.add(m.index);
        unchecked.push({ file: rel, line: idx + 1, stated: value, phrase: `${m[1]} ${m[2]}` });
      }

      for (const m of line.matchAll(NEAR_MISS_RE)) {
        if (parseNumber(m[1]) === undefined || claimedStarts.has(m.index)) continue;
        nearMisses.push({ file: rel, line: idx + 1, phrase: m[0].replace(/\s+/g, " ") });
      }
    });
  }

  return { failures, unchecked, nearMisses, checked };
}

// ─── the checker, parameterised over a root + file set so it can be aimed at
//     fixture trees as well as this repo (that is what makes --selftest bite)
function checkDoctrine({ root, scanSet, counts }) {
  const doctrine = doctrineFileSet(scanSet);
  const retired = findRetiredSentences({ root, files: doctrine.all });
  const claims = scanCountClaims({ root, files: scanSet, counts });
  return {
    ok: retired.length === 0 && claims.failures.length === 0,
    doctrine,
    retired,
    ...claims,
  };
}

// ─── self-test: prove the checker bites, on fixture roots, never this tree ──

function writeFixture(root, rel, body) {
  const abs = path.join(root, rel);
  fs.mkdirSync(path.dirname(abs), { recursive: true });
  fs.writeFileSync(abs, body);
}

function runSelftest(counts) {
  const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-doctrine-parity-selftest-"));
  try {
    // Fixture A — the retired sentence re-introduced into the MERGED ROOT
    // AGENTS.md only, with every AGENTS.block.md projection left clean. This is
    // the D22 case: a gate globbing AGENTS.block.md alone reports green here.
    const rootA = path.join(tmpRoot, "a");
    writeFixture(
      rootA,
      "AGENTS.md",
      "## Guardrails (hook-equivalent rules)\n\nOn Claude Code these are enforced mechanically by hooks; on Codex you must honor them yourself.\n",
    );
    writeFixture(
      rootA,
      "packages/bee/AGENTS.block.md",
      "## Guardrails (hook-equivalent rules)\n\nBoth runtimes ship hooks rendered from one shared catalog.\n",
    );
    const scanA = ["AGENTS.md", "packages/bee/AGENTS.block.md"];
    const resultA = checkDoctrine({ root: rootA, scanSet: scanA, counts });

    const projectionHit = resultA.retired.some((h) => h.file !== "AGENTS.md");
    const rootHit = resultA.retired.some((h) => h.file === "AGENTS.md");
    if (resultA.ok || !rootHit || projectionHit || !resultA.doctrine.rootAgents.includes("AGENTS.md")) {
      console.error(`FAIL ${NAME} --selftest: retired sentence in the merged root AGENTS.md was not reported`);
      console.error(`      result: ${JSON.stringify({ ok: resultA.ok, retired: resultA.retired, doctrine: resultA.doctrine })}`);
      return 1;
    }

    // Fixture B — a prose count that disagrees with its derived count, beside a
    // TRUE six-toggle claim that must not be flagged. "six hook scripts" names
    // the script quantity explicitly, so it is false against the derived 9;
    // "six toggles" and "all six hooks default-on, each toggleable" are both
    // true statements about the 6 config toggles and must pass untouched.
    const rootB = path.join(tmpRoot, "b");
    writeFixture(rootB, "docs/wrong.md", "bee ships six hook scripts under hooks/.\n");
    writeFixture(
      rootB,
      "docs/true-six.md",
      "The hooks are config-gated in `.bee/config.json` (six toggles, each default-on).\n" +
        "Onboarding writes `config.json` (all six hooks default-on, each toggleable).\n",
    );
    writeFixture(rootB, "AGENTS.md", "## Guardrails (hook-equivalent rules)\n\nBoth runtimes ship hooks.\n");
    const scanB = ["AGENTS.md", "docs/true-six.md", "docs/wrong.md"];
    const resultB = checkDoctrine({ root: rootB, scanSet: scanB, counts });

    const bitWrong = resultB.failures.some(
      (f) => f.file === "docs/wrong.md" && f.stated === 6 && f.quantity === "hook scripts",
    );
    const sparedTrueSix = !resultB.failures.some((f) => f.file === "docs/true-six.md");
    const saidSoAboutBareHook = resultB.unchecked.some((u) => u.file === "docs/true-six.md" && u.stated === 6);
    if (resultB.ok || !bitWrong || !sparedTrueSix || !saidSoAboutBareHook) {
      console.error(`FAIL ${NAME} --selftest: count checker did not behave as specified`);
      console.error(`      bit the false "six hook scripts": ${bitWrong}`);
      console.error(`      spared the true six-toggle claims: ${sparedTrueSix}`);
      console.error(`      reported bare "six hooks" as unchecked: ${saidSoAboutBareHook}`);
      console.error(`      result: ${JSON.stringify({ ok: resultB.ok, failures: resultB.failures, unchecked: resultB.unchecked })}`);
      return 1;
    }

    console.log(
      `PASS ${NAME} --selftest: bites on the retired sentence in the merged root AGENTS.md ` +
        `(which the AGENTS.block.md glob does not match), and on "six hook scripts" against the derived ` +
        `${counts.scripts}, while sparing the true six-toggle claims and reporting bare "six hooks" as unchecked`,
    );
    return 0;
  } finally {
    fs.rmSync(tmpRoot, { recursive: true, force: true });
  }
}

// ─── main ─────────────────────────────────────────────────────────────────

function main() {
  const selftestOnly = process.argv.includes("--selftest");

  let counts;
  try {
    counts = deriveCounts(REPO_ROOT);
  } catch (err) {
    console.error(`FAIL ${NAME}: count derivation failed — ${err.message}`);
    return 1;
  }
  console.log(
    `      derived counts: ${counts.scripts} hook scripts (hooks/bee-*.mjs), ` +
      `${counts.codexEvents} Codex lifecycle events (.codex/hooks.json), ` +
      `${counts.claudeEvents} Claude lifecycle events (hooks/claude-hooks.json), ` +
      `${counts.toggles} config toggles (.bee/config.json)`,
  );

  const selftestCode = runSelftest(counts);
  if (selftestCode !== 0) return selftestCode;
  if (selftestOnly) return 0;

  const scanSet = deriveScanSet(REPO_ROOT);
  if (scanSet.length === 0) {
    console.error(`FAIL ${NAME}: derived scan set is empty — the gate would assert nothing`);
    return 1;
  }

  const result = checkDoctrine({ root: REPO_ROOT, scanSet, counts });

  // The scan set is itself a derived quantity — a scope that silently lost the
  // root AGENTS.md is the exact failure D22 was written against, so prove it is
  // in the set rather than assuming the glob covered it.
  if (result.doctrine.rootAgents.length !== 1) {
    console.error(`FAIL ${NAME}: merged root AGENTS.md is not in the derived scan set — the doctrine check would be blind`);
    return 1;
  }
  if (result.doctrine.blocks.length === 0) {
    console.error(`FAIL ${NAME}: no AGENTS.block.md projection found in the derived scan set`);
    return 1;
  }

  let failed = false;

  if (result.retired.length > 0) {
    failed = true;
    console.error(`FAIL ${NAME}: the retired doctrine sentence is live again:`);
    for (const hit of result.retired) {
      console.error(`      ${hit.file}:${hit.line ?? "?"} — "${hit.sentence}"`);
    }
    console.error(`      Both runtimes ship hooks from one shared catalog; what is unverified is whether`);
    console.error(`      an installed Codex CLI executes them (D2). Do not restore the old wording.`);
  } else {
    console.log(
      `PASS ${NAME}: retired doctrine sentence absent from ${result.doctrine.all.length} doctrine file(s) ` +
        `(${result.doctrine.blocks.length} AGENTS.block.md projection(s) + the merged root AGENTS.md)`,
    );
  }

  if (result.failures.length > 0) {
    failed = true;
    console.error(`FAIL ${NAME}: prose count(s) disagree with the quantity they name:`);
    for (const f of result.failures) {
      console.error(`      ${f.file}:${f.line} — "${f.phrase}" claims ${f.stated} ${f.quantity}, derived ${f.admissible.join(" or ")}`);
      console.error(`        ${f.text.slice(0, 160)}`);
    }
    console.error(`      Four counts are live and all four are true of different things (D25):`);
    console.error(`      ${counts.scripts} scripts / ${counts.codexEvents} Codex events / ${counts.claudeEvents} Claude events / ${counts.toggles} toggles.`);
    console.error(`      Fix the number to match the quantity the sentence names — do not rename the quantity.`);
  } else {
    console.log(
      `PASS ${NAME}: ${result.checked} count claim(s) across ${scanSet.length} tracked markdown file(s) ` +
        `agree with their derived quantity`,
    );
  }

  // Saying so rather than guessing. These are NOT failures: the phrase names no
  // single one of the four quantities, so the gate refuses to adjudicate it.
  if (result.unchecked.length > 0) {
    console.log(`      not checked — bare "hook" names no single quantity (${result.unchecked.length}):`);
    for (const u of result.unchecked) {
      console.log(
        `        ${u.file}:${u.line} — "${u.phrase}" (could mean any of ` +
          `${counts.scripts}/${counts.codexEvents}/${counts.claudeEvents}/${counts.toggles})`,
      );
    }
  }
  if (result.nearMisses.length > 0) {
    console.log(`      not checked — number is not directly adjacent to a measured noun (${result.nearMisses.length}):`);
    for (const nm of result.nearMisses) {
      console.log(`        ${nm.file}:${nm.line} — "${nm.phrase}"`);
    }
  }

  return failed ? 1 : 0;
}

process.exit(main());
