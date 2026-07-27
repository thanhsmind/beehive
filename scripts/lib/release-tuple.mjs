// release-tuple.mjs — the single registry of WHERE the release version lives.
//
// The bee release version is physically duplicated across five "tuple"
// components: the packages/bee/lib source constant, its byte-identical .bee/bin
// runtime mirror, the two plugin manifests (Claude + Codex) that external
// plugin systems read as raw JSON and therefore cannot import a JS const from,
// and the Rust port's own const (a compiled binary cannot import the mjs one
// either). One canonical value, five physical homes.
//
// This module is the ONE place those locations are enumerated. It is
// side-effect free (no top-level checks, no process.exit) so both the checker
// (scripts/test_release_tuple.mjs) and the writer (scripts/bump_version.mjs)
// can import the same registry — WHERE the version lives is defined once, and a
// release bumps every member from a single command instead of hand-editing
// each file. DIST-05 (SPEC), decisions ed0b2920 and cba8b832.

import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
// scripts/lib/ -> repo root is two levels up.
export const REPO_ROOT = path.join(__dirname, "..", "..");

/**
 * The release-version tuple: every physical home of the version string.
 * `kind` selects how the version is read/written:
 *   - "js-const": the `export const BEE_VERSION = '...'` line in a state.mjs
 *   - "json-version": the top-level `.version` string in a plugin manifest
 */
export const COMPONENTS = [
  {
    name: "packages/bee/lib/state.mjs (BEE_VERSION)",
    path: path.join(REPO_ROOT, "packages", "bee", "lib", "state.mjs"),
    kind: "js-const",
  },
  {
    name: ".bee/bin/lib/state.mjs (BEE_VERSION)",
    path: path.join(REPO_ROOT, ".bee", "bin", "lib", "state.mjs"),
    kind: "js-const",
  },
  {
    name: ".claude-plugin/plugin.json (.version)",
    path: path.join(REPO_ROOT, ".claude-plugin", "plugin.json"),
    kind: "json-version",
  },
  {
    name: ".codex-plugin/plugin.json (.version)",
    path: path.join(REPO_ROOT, ".codex-plugin", "plugin.json"),
    kind: "json-version",
  },
  {
    // The Rust port carries its own copy of the constant (its
    // `compute_runtime_drift` compares onboarding's recorded version against
    // it, exactly as the mjs reader does). It used to be hand-synced, so every
    // bump left it a release behind and the mjs/Rust parity suite went red on
    // the version-drift case. It is a tuple member like any other.
    name: "crates/bee-core/src/state.rs (BEE_VERSION)",
    path: path.join(REPO_ROOT, "crates", "bee-core", "src", "state.rs"),
    kind: "rust-const",
  },
];

export const BEE_VERSION_RE = /export\s+const\s+BEE_VERSION\s*=\s*['"]([^'"]+)['"]/;
export const RUST_BEE_VERSION_RE = /pub\s+const\s+BEE_VERSION\s*:\s*&str\s*=\s*"([^"]+)"/;

/**
 * Extracts BEE_VERSION from a state.mjs source string. Returns null if not
 * found (caller treats that as a read failure, not a silent pass).
 */
export function extractJsConstVersion(source) {
  const match = source.match(BEE_VERSION_RE);
  return match ? match[1] : null;
}

/**
 * Reads a single component's version given its {path, kind}. Returns the
 * version string, or throws with a message naming the component.
 */
export function readComponentVersion(component, readFileFn = fs.readFileSync) {
  let raw;
  try {
    raw = readFileFn(component.path, "utf8");
  } catch (error) {
    throw new Error(`could not read ${component.name} at ${component.path}: ${error.message}`);
  }

  if (component.kind === "js-const") {
    const version = extractJsConstVersion(raw);
    if (!version) {
      throw new Error(`${component.name}: no BEE_VERSION export found in ${component.path}`);
    }
    return version;
  }

  if (component.kind === "rust-const") {
    const match = raw.match(RUST_BEE_VERSION_RE);
    if (!match) {
      throw new Error(`${component.name}: no BEE_VERSION const found in ${component.path}`);
    }
    return match[1];
  }

  if (component.kind === "json-version") {
    let parsed;
    try {
      parsed = JSON.parse(raw);
    } catch (error) {
      throw new Error(`${component.name}: could not parse JSON at ${component.path}: ${error.message}`);
    }
    if (typeof parsed.version !== "string" || !parsed.version) {
      throw new Error(`${component.name}: no top-level .version string in ${component.path}`);
    }
    return parsed.version;
  }

  throw new Error(`${component.name}: unknown component kind "${component.kind}"`);
}

/**
 * Writes `version` into a single component, preserving everything else in the
 * file. For "js-const" only the BEE_VERSION line is rewritten; for
 * "json-version" only the `.version` field changes (2-space indent, trailing
 * newline — matching how the manifests are stored). Returns the previous
 * version string. Throws (naming the component) if the file cannot be read or
 * has no version marker to replace.
 */
export function writeComponentVersion(component, version, io = fs) {
  let raw;
  try {
    raw = io.readFileSync(component.path, "utf8");
  } catch (error) {
    throw new Error(`could not read ${component.name} at ${component.path}: ${error.message}`);
  }

  if (component.kind === "js-const") {
    const match = raw.match(BEE_VERSION_RE);
    if (!match) {
      throw new Error(`${component.name}: no BEE_VERSION export to rewrite in ${component.path}`);
    }
    const previous = match[1];
    const next = raw.replace(BEE_VERSION_RE, `export const BEE_VERSION = '${version}'`);
    io.writeFileSync(component.path, next, "utf8");
    return previous;
  }

  if (component.kind === "rust-const") {
    const match = raw.match(RUST_BEE_VERSION_RE);
    if (!match) {
      throw new Error(`${component.name}: no BEE_VERSION const to rewrite in ${component.path}`);
    }
    const previous = match[1];
    const next = raw.replace(RUST_BEE_VERSION_RE, `pub const BEE_VERSION: &str = "${version}"`);
    io.writeFileSync(component.path, next, "utf8");
    return previous;
  }

  if (component.kind === "json-version") {
    let parsed;
    try {
      parsed = JSON.parse(raw);
    } catch (error) {
      throw new Error(`${component.name}: could not parse JSON at ${component.path}: ${error.message}`);
    }
    if (typeof parsed.version !== "string" || !parsed.version) {
      throw new Error(`${component.name}: no top-level .version string in ${component.path}`);
    }
    const previous = parsed.version;
    parsed.version = version;
    io.writeFileSync(component.path, `${JSON.stringify(parsed, null, 2)}\n`, "utf8");
    return previous;
  }

  throw new Error(`${component.name}: unknown component kind "${component.kind}"`);
}

/**
 * Reads every component's current version. Returns { name, version } entries in
 * COMPONENTS order. Throws on the first unreadable component.
 */
export function readComponents(components = COMPONENTS, readFileFn = fs.readFileSync) {
  return components.map((component) => ({
    name: component.name,
    version: readComponentVersion(component, readFileFn),
  }));
}

/**
 * Given a list of {name, version} entries, checks they are all equal.
 * Returns { ok, desynced } — desynced is the list of entries that do not match
 * the first value, empty when all equal.
 */
export function checkTupleEquality(entries) {
  if (entries.length === 0) {
    return { ok: true, desynced: [] };
  }
  const baseline = entries[0].version;
  const desynced = entries.filter((entry) => entry.version !== baseline);
  return { ok: desynced.length === 0, desynced };
}
