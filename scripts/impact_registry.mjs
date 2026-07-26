#!/usr/bin/env node
// impact_registry.mjs — cov-1 (ci-owned-verify CONTEXT.md D3): a committed
// file -> suite relatedness registry, DERIVED from run_verify.mjs's own
// SUITES discovery, never a hand-authored list. The dev loop (D4,
// `run_verify.mjs --impacted`/`--impacted-from-git`) and any harness-side
// connection lookup resolve "which suites does this file affect" through
// this registry's answers.
//
// For every runnable run_verify.mjs discovers (imported live from its
// exported SUITES — discovery itself is never reimplemented here), this
// walks the runnable's transitive FILE closure via four edge types,
// regex-level (no AST dependency), and inverts the result to file -> suites:
//
//   1. static relative ESM imports/re-exports (`import ... from "./x.mjs"`,
//      `import "./x.mjs"`, `export ... from "./x.mjs"`), BFS-expanded.
//   2. dynamic `import(pathToFileURL(<expr>).href)` — <expr> is resolved
//      through a small `const NAME = path.join(...)`-style variable tracker
//      (also handles `path.dirname(fileURLToPath(import.meta.url))` and
//      `__dirname`/`__filename`), plus plain literal `import("./x.mjs")`.
//   3. spawn/exec argv literals — `spawn(cmd, [TOKEN, ...])` /
//      `spawnSync(...)` / `execFile(...)` / `execFileSync(...)` — any argv
//      token that resolves (via the same variable tracker) to a real repo
//      file becomes an edge. This is how a suite that spawns `.bee/bin/
//      bee.mjs` as a child process (e.g. `scripts/tests/test_worktree_cli.mjs`)
//      inherits bee.mjs's own closure: bee.mjs becomes reachable, and BFS
//      then walks bee.mjs's own edges (all four types) from there.
//   4. `runModuleWorker(<modulePath>, ...)` first-arg targets
//      (scripts/lib/run-module-worker.mjs's pattern) — modulePath resolved
//      the same way, including `path.join(REPO_ROOT, ".bee", "bin",
//      "bee.mjs")`-style segment reconstruction. test_conformance.mjs,
//      test_heartbeat_touch.mjs, test_dispatch_prepare.mjs and
//      test_compact_*.mjs reach bee.mjs and the hooks ONLY this way.
//
// KNOWN BLIND SPOT (plan-check P3, by design — regex-level scanning, no AST):
// this never sees `readFileSync`'d fixture paths, env-pointed paths
// (BEE_*_PATH-style overrides), or any target reached only through a
// function CALL indirection (e.g. `runModuleWorker(beeModulePath(), ...)`,
// where `beeModulePath` is a helper function rather than a `const` bound to
// a `path.join(...)` chain) or a destructured/dynamically-built argv. Such
// edges are silently absent from the registry rather than guessed at —
// under-connecting is a missed impacted-suite (caught by the full CI run);
// over-connecting would defeat the entire point of scoping.
//
// Verbs:
//   --write            recompute the registry and write it to
//                       scripts/impact-registry.json (deterministic: sorted
//                       keys, sorted suite arrays, no timestamps).
//   --check            recompute in memory and byte-compare against the
//                       committed file; exit 1 with a regen-command message
//                       on any drift (missing file counts as drift).
//   --query <file...>  print the union of impacted suite labels for the
//                       given repo-relative (or absolute/cwd-relative) file
//                       paths, one per line, sorted, to stdout. Exit 0
//                       always, even for zero matches — but any input file
//                       absent from the registry gets a loud `UNMAPPED:`
//                       note on stderr; it is never silently dropped.
//   --level 1           (--query only) narrow the union to DIRECT edges only
//                       — files the suite's own source references — instead
//                       of the full transitive closure (the default). A file
//                       reachable only transitively is still not UNMAPPED;
//                       it just contributes no suites at this depth.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..");
const RUN_VERIFY_PATH = path.join(__dirname, "run_verify.mjs");
const REGISTRY_PATH = path.join(__dirname, "impact-registry.json");
const REGISTRY_PATH_REL = "scripts/impact-registry.json";

const EDGE_SCAN_EXTENSIONS = new Set([".mjs", ".js", ".cjs"]);
const RESOLVE_EXTS = ["", ".mjs", ".js", ".cjs"];

// ─── path helpers ───────────────────────────────────────────────────────────

function toRepoRelative(absPath) {
  const rel = path.relative(REPO_ROOT, absPath);
  return rel.split(path.sep).join("/");
}

function toAbs(repoRelPath) {
  return path.join(REPO_ROOT, ...repoRelPath.split("/"));
}

function existsAsFile(absPath) {
  try {
    return fs.statSync(absPath).isFile();
  } catch {
    return false;
  }
}

function resolveModuleFile(absNoExt) {
  for (const ext of RESOLVE_EXTS) {
    const candidate = absNoExt + ext;
    if (existsAsFile(candidate)) return candidate;
  }
  for (const idx of ["index.mjs", "index.js"]) {
    const candidate = path.join(absNoExt, idx);
    if (existsAsFile(candidate)) return candidate;
  }
  return absNoExt; // best-effort; filtered out downstream if it never exists
}

// ─── tiny expression resolver (regex-level, not an AST) ────────────────────
// Resolves a handful of shapes commonly used across this repo's suites to
// build repo-relative file targets from a chain of `const` assignments:
//   fileURLToPath(import.meta.url)
//   path.dirname(fileURLToPath(import.meta.url))
//   path.dirname(<expr>)
//   path.join(<args>) / path.resolve(<args>)
//   a bare identifier already tracked in `vars`, or __dirname/__filename
//   a bare quoted relative literal ("./x.mjs") resolved against fileDir
// Anything else resolves to `undefined` (a documented blind spot, never a
// guess).

function splitTopLevelArgs(s) {
  const args = [];
  let depth = 0;
  let quote = null;
  let cur = "";
  for (let i = 0; i < s.length; i += 1) {
    const c = s[i];
    if (quote) {
      cur += c;
      if (c === "\\") {
        i += 1;
        if (i < s.length) cur += s[i];
        continue;
      }
      if (c === quote) quote = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      quote = c;
      cur += c;
      continue;
    }
    if (c === "(" || c === "[" || c === "{") {
      depth += 1;
      cur += c;
      continue;
    }
    if (c === ")" || c === "]" || c === "}") {
      depth -= 1;
      cur += c;
      continue;
    }
    if (c === "," && depth === 0) {
      args.push(cur);
      cur = "";
      continue;
    }
    cur += c;
  }
  if (cur.trim() !== "") args.push(cur);
  return args;
}

function resolveJoinArg(argRaw, vars, fileAbsPath, fileDir) {
  const arg = argRaw.trim();
  const lit = arg.match(/^["'`]([^"'`]*)["'`]$/);
  if (lit) return lit[1]; // a literal path.join segment, used as-is
  return resolveExprToAbs(arg, vars, fileAbsPath, fileDir);
}

function resolveExprToAbs(exprRaw, vars, fileAbsPath, fileDir) {
  const expr = (exprRaw || "").trim();
  if (!expr) return undefined;

  if (expr === "__dirname") return fileDir;
  if (expr === "__filename") return fileAbsPath;
  if (vars.has(expr)) return vars.get(expr);

  if (/^fileURLToPath\(\s*import\.meta\.url\s*\)$/.test(expr)) return fileAbsPath;
  if (/^path\.dirname\(\s*fileURLToPath\(\s*import\.meta\.url\s*\)\s*\)$/.test(expr)) return fileDir;

  let m = expr.match(/^path\.dirname\(([\s\S]*)\)$/);
  if (m) {
    const inner = resolveExprToAbs(m[1], vars, fileAbsPath, fileDir);
    return inner ? path.dirname(inner) : undefined;
  }

  m = expr.match(/^path\.(join|resolve)\(([\s\S]*)\)$/);
  if (m) {
    const argList = splitTopLevelArgs(m[2])
      .map((a) => a.trim())
      .filter(Boolean);
    const parts = [];
    for (const a of argList) {
      if (a.startsWith("...")) return undefined; // spread — unresolvable
      const part = resolveJoinArg(a, vars, fileAbsPath, fileDir);
      if (part === undefined) return undefined;
      parts.push(part);
    }
    if (parts.length === 0) return undefined;
    try {
      return m[1] === "join" ? path.join(...parts) : path.resolve(...parts);
    } catch {
      return undefined;
    }
  }

  // a bare quoted relative literal used standalone (not inside path.join)
  m = expr.match(/^["'`]([^"'`]*)["'`]$/);
  if (m) {
    const lit = m[1];
    if (lit.startsWith(".") || lit.startsWith("/")) {
      return path.resolve(fileDir, lit);
    }
    return undefined;
  }

  return undefined;
}

const CONST_ASSIGN_RE = /\b(?:const|let)\s+(\w+)\s*=\s*([^;]+);/g;

function extractVars(source, fileAbsPath, fileDir) {
  const vars = new Map();
  CONST_ASSIGN_RE.lastIndex = 0;
  let m;
  while ((m = CONST_ASSIGN_RE.exec(source))) {
    const resolved = resolveExprToAbs(m[2], vars, fileAbsPath, fileDir);
    if (resolved !== undefined) vars.set(m[1], resolved);
  }
  return vars;
}

// ─── balanced call-argument extraction ─────────────────────────────────────
// Regex alone can't safely capture a call's full, paren-balanced argument
// list (nested calls/objects/arrays inside it), so this does one linear
// bracket/quote-depth scan per call site found by a `<callee>(` anchor.

function findMatchingParen(source, openIdx) {
  let depth = 0;
  let quote = null;
  for (let i = openIdx; i < source.length; i += 1) {
    const c = source[i];
    if (quote) {
      if (c === "\\") {
        i += 1;
        continue;
      }
      if (c === quote) quote = null;
      continue;
    }
    if (c === '"' || c === "'" || c === "`") {
      quote = c;
      continue;
    }
    if (c === "(") depth += 1;
    else if (c === ")") {
      depth -= 1;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function extractCallArgsList(source, calleeOpenRe) {
  const results = [];
  calleeOpenRe.lastIndex = 0;
  let m;
  while ((m = calleeOpenRe.exec(source))) {
    const openIdx = calleeOpenRe.lastIndex - 1;
    if (source[openIdx] !== "(") continue;
    const closeIdx = findMatchingParen(source, openIdx);
    if (closeIdx === -1) continue;
    results.push(source.slice(openIdx + 1, closeIdx));
    calleeOpenRe.lastIndex = closeIdx + 1;
  }
  return results;
}

// ─── per-file edge extraction (the four edge types) ────────────────────────

const IMPORT_STMT_RE = /\bimport\s[^;]*?;/g;
const EXPORT_STMT_RE = /\bexport\s[^;]*?;/g;
const IMPORT_CALL_RE = /\bimport\(/g;
const SPAWN_CALL_RE = /\b(?:spawn|spawnSync|execFile|execFileSync)\(/g;
const RUN_MODULE_WORKER_RE = /\brunModuleWorker\(/g;

function staticImportEdges(source, fileDir, edges) {
  for (const re of [IMPORT_STMT_RE, EXPORT_STMT_RE]) {
    re.lastIndex = 0;
    let m;
    while ((m = re.exec(source))) {
      const stmt = m[0];
      let specifier = null;
      const fromMatch = stmt.match(/\bfrom\s+["']([^"']+)["']/);
      if (fromMatch) {
        specifier = fromMatch[1];
      } else {
        const bareMatch = stmt.match(/^import\s+["']([^"']+)["']/);
        if (bareMatch) specifier = bareMatch[1];
      }
      if (!specifier) continue;
      if (!(specifier.startsWith(".") || specifier.startsWith("/"))) continue; // skip bare/node: specifiers
      edges.add(resolveModuleFile(path.resolve(fileDir, specifier)));
    }
  }
}

function dynamicImportEdges(source, vars, fileAbsPath, fileDir, edges) {
  for (const argsStr of extractCallArgsList(source, IMPORT_CALL_RE)) {
    const trimmed = argsStr.trim();
    const ptfu = trimmed.match(/^pathToFileURL\(([\s\S]*)\)\.href$/);
    if (ptfu) {
      const resolved = resolveExprToAbs(ptfu[1], vars, fileAbsPath, fileDir);
      if (resolved) edges.add(resolveModuleFile(resolved));
      continue;
    }
    const lit = trimmed.match(/^["'`]([^"'`]*)["'`]$/);
    if (lit) {
      const specifier = lit[1];
      if (specifier.startsWith(".") || specifier.startsWith("/")) {
        edges.add(resolveModuleFile(path.resolve(fileDir, specifier)));
      }
      continue;
    }
    const resolved = resolveExprToAbs(trimmed, vars, fileAbsPath, fileDir);
    if (resolved) edges.add(resolveModuleFile(resolved));
  }
}

function spawnArgvEdges(source, vars, fileAbsPath, fileDir, edges) {
  for (const argsStr of extractCallArgsList(source, SPAWN_CALL_RE)) {
    const parts = splitTopLevelArgs(argsStr).map((p) => p.trim());
    const arrPart = parts.find((p) => p.startsWith("["));
    if (!arrPart) continue;
    const inner = arrPart.replace(/^\[/, "").replace(/\]$/, "");
    for (const tokRaw of splitTopLevelArgs(inner)) {
      const tok = tokRaw.trim();
      if (!tok || tok.startsWith("...")) continue;
      const resolved = resolveExprToAbs(tok, vars, fileAbsPath, fileDir);
      if (resolved) edges.add(resolveModuleFile(resolved));
    }
  }
}

function runModuleWorkerEdges(source, vars, fileAbsPath, fileDir, edges) {
  for (const argsStr of extractCallArgsList(source, RUN_MODULE_WORKER_RE)) {
    const parts = splitTopLevelArgs(argsStr);
    if (parts.length === 0) continue;
    const resolved = resolveExprToAbs(parts[0], vars, fileAbsPath, fileDir);
    if (resolved) edges.add(resolveModuleFile(resolved));
  }
}

const edgeCache = new Map();

function getEdges(repoRelPath) {
  if (edgeCache.has(repoRelPath)) return edgeCache.get(repoRelPath);
  const absPath = toAbs(repoRelPath);
  const rawEdges = new Set();
  const ext = path.extname(repoRelPath);
  if (EDGE_SCAN_EXTENSIONS.has(ext) && existsAsFile(absPath)) {
    let source = "";
    try {
      source = fs.readFileSync(absPath, "utf8");
    } catch {
      source = "";
    }
    const fileDir = path.dirname(absPath);
    const vars = extractVars(source, absPath, fileDir);
    staticImportEdges(source, fileDir, rawEdges);
    dynamicImportEdges(source, vars, absPath, fileDir, rawEdges);
    spawnArgvEdges(source, vars, absPath, fileDir, rawEdges);
    runModuleWorkerEdges(source, vars, absPath, fileDir, rawEdges);
  }
  const relEdges = new Set();
  for (const abs of rawEdges) {
    if (!existsAsFile(abs)) continue;
    const rel = toRepoRelative(abs);
    if (!rel || rel.startsWith("..")) continue;
    relEdges.add(rel);
  }
  edgeCache.set(repoRelPath, relEdges);
  return relEdges;
}

function closureFor(entryRelPath) {
  const visited = new Set();
  const queue = [entryRelPath];
  while (queue.length > 0) {
    const cur = queue.shift();
    if (visited.has(cur)) continue;
    visited.add(cur);
    for (const child of getEdges(cur)) {
      if (!visited.has(child)) queue.push(child);
    }
  }
  return visited;
}

// ─── registry build / (de)serialize / query ────────────────────────────────

function suiteLabel(entry) {
  return [entry[0], ...entry.slice(1)].join(" ");
}

// Per-file edge DEPTH (impacted-level1 D1): 'direct' = the suite's own
// source references the file — any of the four edge types read straight off
// the suite's entry file (getEdges(entry[0])), plus the entry file itself
// (a suite's own path is always direct for itself). 'transitive' = reached
// only through a spawned/worker entrypoint's inherited closure — e.g. a
// suite spawns .bee/bin/bee.mjs (bee.mjs is direct for that suite) and
// bee.mjs's own static imports (state.mjs, etc.) come along for the ride
// (transitive for that suite, since the suite's own source never mentions
// them). `all` is exactly the full BFS closure this registry always
// computed (unchanged semantics); `direct` is the depth-1 subset of it —
// direct is always a subset of all, never the other way around.
export async function buildRegistry() {
  edgeCache.clear();
  const { SUITES } = await import(pathToFileURL(RUN_VERIFY_PATH).href);
  const fileToAll = new Map();
  const fileToDirect = new Map();
  for (const entry of SUITES) {
    const label = suiteLabel(entry);
    const closure = closureFor(entry[0]);
    for (const f of closure) {
      if (!fileToAll.has(f)) fileToAll.set(f, new Set());
      fileToAll.get(f).add(label);
    }
    const directSet = new Set([entry[0], ...getEdges(entry[0])]);
    for (const f of directSet) {
      if (!fileToDirect.has(f)) fileToDirect.set(f, new Set());
      fileToDirect.get(f).add(label);
    }
  }
  const files = {};
  const allKeys = new Set([...fileToAll.keys(), ...fileToDirect.keys()]);
  for (const key of [...allKeys].sort()) {
    files[key] = {
      direct: [...(fileToDirect.get(key) ?? [])].sort(),
      all: [...(fileToAll.get(key) ?? [])].sort(),
    };
  }
  return { version: 2, files };
}

export function serializeRegistry(registry) {
  return `${JSON.stringify(registry, null, 2)}\n`;
}

export function normalizeQueryPath(inputPath) {
  const abs = path.isAbsolute(inputPath) ? inputPath : path.resolve(process.cwd(), inputPath);
  return toRepoRelative(abs);
}

// `level: 1` narrows the per-file suite list to `direct` edges only; the
// default (no `level`, or any other value) uses `all` — the full closure, as
// this always behaved. A file's "known to the registry at all" status is
// judged by `all` regardless of `level`: a file that's reachable only
// transitively (all non-empty, direct empty) is NOT unmapped at level 1 — it
// legitimately contributes zero suites at that depth, which is a different,
// louder thing than "no suite relates to this file at all" (the caller is
// expected to surface that distinction, e.g. run_verify.mjs's
// direct=0-but-transitive>0 note).
export function queryRegistry(registry, files, { level } = {}) {
  const mapped = new Set();
  const unmapped = [];
  for (const f of files) {
    const rel = normalizeQueryPath(f);
    const entry = registry.files[rel];
    const allSuites = entry ? entry.all : undefined;
    if (!allSuites || allSuites.length === 0) {
      unmapped.push(rel);
      continue;
    }
    const suites = level === 1 ? entry.direct : allSuites;
    for (const s of suites) mapped.add(s);
  }
  return { mappedSuites: [...mapped].sort(), unmapped };
}

// ─── CLI ────────────────────────────────────────────────────────────────────

async function main() {
  const [mode, ...rest] = process.argv.slice(2);

  if (mode === "--write") {
    const registry = await buildRegistry();
    const json = serializeRegistry(registry);
    fs.writeFileSync(REGISTRY_PATH, json);
    console.log(
      `impact_registry --write: wrote ${REGISTRY_PATH_REL} (${Object.keys(registry.files).length} files)`,
    );
    process.exit(0);
  }

  if (mode === "--check") {
    const registry = await buildRegistry();
    const expected = serializeRegistry(registry);
    let actual = null;
    try {
      actual = fs.readFileSync(REGISTRY_PATH, "utf8");
    } catch {
      actual = null;
    }
    if (actual === expected) {
      console.log(`impact_registry --check: ${REGISTRY_PATH_REL} is up to date`);
      process.exit(0);
    }
    console.error(
      actual === null
        ? `impact_registry --check: ${REGISTRY_PATH_REL} is missing.`
        : `impact_registry --check: ${REGISTRY_PATH_REL} is STALE (drift detected).`,
    );
    console.error("FIX: node scripts/impact_registry.mjs --write");
    process.exit(1);
  }

  if (mode === "--query") {
    let level;
    const queryFiles = [];
    for (let i = 0; i < rest.length; i += 1) {
      if (rest[i] === "--level") {
        const value = rest[i + 1];
        if (value !== "1") {
          console.error(`usage: node scripts/impact_registry.mjs --query <file...> [--level 1] (got --level ${JSON.stringify(value)})`);
          process.exit(1);
        }
        level = 1;
        i += 1;
        continue;
      }
      queryFiles.push(rest[i]);
    }
    if (queryFiles.length === 0) {
      console.error("usage: node scripts/impact_registry.mjs --query <file...> [--level 1]");
      process.exit(1);
    }
    let registry;
    try {
      registry = JSON.parse(fs.readFileSync(REGISTRY_PATH, "utf8"));
    } catch (error) {
      console.error(
        `impact_registry --query: could not read/parse ${REGISTRY_PATH_REL} (${error.message}). Run --write first.`,
      );
      process.exit(1);
    }
    const { mappedSuites, unmapped } = queryRegistry(registry, queryFiles, { level });
    for (const u of unmapped) {
      console.error(`UNMAPPED: ${u} (no known suite relates to this file — full verify still covers it)`);
    }
    for (const s of mappedSuites) console.log(s);
    process.exit(0);
  }

  console.error("usage: node scripts/impact_registry.mjs --write | --check | --query <file...> [--level 1]");
  process.exit(2);
}

const isMain = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url;
if (isMain) {
  main();
}
