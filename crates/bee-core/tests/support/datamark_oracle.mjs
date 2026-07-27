// datamark_oracle.mjs — file-based node driver for
// crates/bee-core/tests/datamark_oracle.rs (rpl-3). It executes the REAL,
// FROZEN `.bee/bin/lib/decisions.mjs` and `.bee/bin/lib/capture.mjs` so the
// Rust content-safety port is diffed against the runtime that actually
// ships, never against this author's reading of those files.
//
// A tracked FILE deliberately, never `node -e`: the internals-reach guard
// denies inline evals importing bin/lib.
//
// Every path this script writes to is a per-invocation temp directory under
// the OS temp dir (or a caller-supplied one), never the repo's live `.bee/`
// store. `logDecision` and `addCaptureStub` both APPEND to a store, so this
// driver must never be pointed at a real root.
//
// Transport is JSON on stdin and stdout in both directions: it escapes
// control characters, U+FEFF, CRLF and astral pairs losslessly, which a
// raw-text pipe would not.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));

function findRepoRoot(startDir) {
  let dir = startDir;
  for (let i = 0; i < 12; i++) {
    if (fs.existsSync(path.join(dir, '.bee', 'bin', 'lib', 'decisions.mjs'))) return dir;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  throw new Error(`datamark_oracle: could not locate .bee/bin/lib/decisions.mjs above ${startDir}`);
}

const repoRoot = findRepoRoot(here);
const libDir = path.join(repoRoot, '.bee', 'bin', 'lib');
const { datamark, logDecision, SECRET_CONTENT_PATTERNS, INJECTION_PATTERNS } = await import(
  `file://${path.join(libDir, 'decisions.mjs')}`
);
const { addCaptureStub, flushCaptureStub, captureQueuePath } = await import(
  `file://${path.join(libDir, 'capture.mjs')}`
);
const { KIND_ALIASES, NORMALIZED_KINDS } = await import(`file://${path.join(libDir, 'feedback.mjs')}`);

let scratchSeq = 0;
function freshRoot() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), `bee-datamark-oracle-${process.pid}-${scratchSeq++}-`));
  fs.mkdirSync(path.join(dir, '.bee'), { recursive: true });
  return dir;
}

function readStdin() {
  return new Promise((resolve, reject) => {
    let data = '';
    process.stdin.setEncoding('utf8');
    process.stdin.on('data', (chunk) => {
      data += chunk;
    });
    process.stdin.on('end', () => resolve(data));
    process.stdin.on('error', reject);
  });
}

function lastLine(file) {
  const text = fs.readFileSync(file, 'utf8');
  const lines = text.split('\n').filter((l) => l !== '');
  return lines[lines.length - 1] ?? null;
}

// ─── ops ───────────────────────────────────────────────────────────────────

/// `datamark(text)` over every corpus entry.
function opDatamark(payload) {
  return payload.map((text) => datamark(text));
}

/// The DECISION refusal surface. The corpus text goes into the named field;
/// `logDecision` runs for real (lock, classification, append) in a throwaway
/// root, so the message is the runtime's, not a reconstruction.
///
/// `decision` and `rationale` are pre-validated as non-empty BEFORE
/// `assertSafe` runs, so a caller sending blank text under those names is
/// asking for `logDecision`'s own required-field message; the Rust side
/// mirrors that by only routing non-blank entries through them.
function opDecisionRefusal(payload) {
  const { field, values } = payload;
  return values.map((text) => {
    const root = freshRoot();
    const args = { decision: 'a safe settled decision', rationale: 'a safe recorded rationale' };
    args[field] = text;
    try {
      logDecision(root, args);
      return { ok: true, message: null };
    } catch (err) {
      return { ok: false, message: String(err && err.message) };
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}

/// The CAPTURE write path end to end: the same function the CLI calls, so
/// the diff covers the refusal texts AND the appended line's exact bytes
/// and key order.
function opCaptureAdd(payload) {
  return payload.map((fields) => {
    const root = freshRoot();
    try {
      addCaptureStub(root, {
        outcome: fields.outcome ?? null,
        dids: fields.dids ?? null,
        area: fields.area ?? null,
        files: fields.files ?? null,
        lane: fields.lane ?? null,
        source: fields.source ?? null,
      });
      return { ok: true, message: null, line: lastLine(captureQueuePath(root)) };
    } catch (err) {
      return { ok: false, message: String(err && err.message), line: null };
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}

/// `flushCaptureStub` over a caller-seeded queue.
function opCaptureFlush(payload) {
  return payload.map((testCase) => {
    const root = freshRoot();
    const queue = captureQueuePath(root);
    fs.mkdirSync(path.dirname(queue), { recursive: true });
    fs.writeFileSync(queue, testCase.seed.map((l) => `${l}\n`).join(''), 'utf8');
    try {
      flushCaptureStub(root, testCase.id, { into: testCase.into ?? null });
      return { ok: true, message: null, line: lastLine(queue) };
    } catch (err) {
      return { ok: false, message: String(err && err.message), line: null };
    } finally {
      fs.rmSync(root, { recursive: true, force: true });
    }
  });
}

/// The shared kind vocabulary, plus the ONE expression built on top of it:
/// `bee.mjs:3695` `backlogAllowedTypes()`. That function is not exported, so
/// its single line is reproduced here VERBATIM — the constants it reads are
/// the real ones, which is the part that can drift.
function opVocabulary() {
  return {
    kind_aliases: Object.entries(KIND_ALIASES),
    normalized_kinds: [...NORMALIZED_KINDS],
    backlog_allowed_types: [...new Set([...Object.keys(KIND_ALIASES), ...NORMALIZED_KINDS])].sort(),
  };
}

/// The pattern tables' own `String(pattern)` spellings — the exact text the
/// refusal messages interpolate through `${pattern}`.
function opPatternSources() {
  return {
    secret: SECRET_CONTENT_PATTERNS.map(String),
    injection: INJECTION_PATTERNS.map(String),
  };
}

async function main() {
  const op = process.argv[2];
  const raw = await readStdin();
  const payload = raw.trim() ? JSON.parse(raw) : null;
  let out;
  switch (op) {
    case 'datamark':
      out = opDatamark(payload);
      break;
    case 'decision-refusal':
      out = opDecisionRefusal(payload);
      break;
    case 'capture-add':
      out = opCaptureAdd(payload);
      break;
    case 'capture-flush':
      out = opCaptureFlush(payload);
      break;
    case 'vocabulary':
      out = opVocabulary();
      break;
    case 'pattern-sources':
      out = opPatternSources();
      break;
    default:
      console.error(
        'usage: datamark_oracle.mjs <datamark|decision-refusal|capture-add|capture-flush|vocabulary|pattern-sources>',
      );
      process.exit(2);
  }
  process.stdout.write(JSON.stringify(out));
}

await main();
