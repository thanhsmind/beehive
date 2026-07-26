// commands_detect.mjs — propose host-project standard commands from manifests.
// Detection is propose-only (decision D3): candidates are surfaced for user
// confirmation; this module never writes .bee/config.json. It reads named
// manifest files only — never .env or other secret-shaped paths.

import fs from 'node:fs';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import { COMMAND_KEYS } from './state.mjs';

function readText(file) {
  try {
    return fs.readFileSync(file, 'utf8');
  } catch {
    return null;
  }
}

function readManifestJson(file) {
  const text = readText(file);
  if (text === null) return null;
  try {
    return JSON.parse(text);
  } catch {
    return null;
  }
}

/** Script names defined in a package.json/composer.json `scripts` object. */
function scriptNames(manifest) {
  const scripts =
    manifest && typeof manifest === 'object' && !Array.isArray(manifest) ? manifest.scripts : null;
  if (!scripts || typeof scripts !== 'object' || Array.isArray(scripts)) return [];
  return Object.keys(scripts).filter((name) => {
    const value = scripts[name];
    if (typeof value === 'string') return value.trim().length > 0;
    return Array.isArray(value) && value.length > 0; // composer allows script arrays
  });
}

/** Target names declared at column 0 of a Makefile (never recipe bodies). */
function makefileTargets(text) {
  const targets = [];
  for (const line of text.split(/\r?\n/)) {
    if (/^[\t ]/.test(line)) continue; // recipe/continuation line
    const match = /^([A-Za-z0-9._-]+)\s*:(?!=)/.exec(line);
    if (match && !match[1].startsWith('.')) targets.push(match[1]);
  }
  return targets;
}

// Explicit sources: key = exact-name match of a script/target against
// COMMAND_KEYS; value is the invocable command, never the recipe body.
function packageJsonCandidates(root) {
  const names = scriptNames(readManifestJson(path.join(root, 'package.json')));
  return COMMAND_KEYS.filter((key) => names.includes(key)).map((key) => ({
    key,
    value: key === 'test' ? 'npm test' : `npm run ${key}`,
    source: 'package.json',
  }));
}

function makefileCandidates(root) {
  const text = readText(path.join(root, 'Makefile'));
  if (text === null) return [];
  const targets = makefileTargets(text);
  return COMMAND_KEYS.filter((key) => targets.includes(key)).map((key) => ({
    key,
    value: `make ${key}`,
    source: 'Makefile',
  }));
}

function composerCandidates(root) {
  const names = scriptNames(readManifestJson(path.join(root, 'composer.json')));
  return COMMAND_KEYS.filter((key) => names.includes(key)).map((key) => ({
    key,
    value: `composer ${key}`,
    source: 'composer.json',
  }));
}

function markerIfExists(root, name) {
  return fs.existsSync(path.join(root, name)) ? name : null;
}

/** Lexicographically first *.csproj filename at the repo root, or null. */
function firstCsproj(root) {
  let entries;
  try {
    entries = fs.readdirSync(root, { withFileTypes: true });
  } catch {
    return null;
  }
  const matches = entries
    .filter((entry) => entry.isFile() && entry.name.toLowerCase().endsWith('.csproj'))
    .map((entry) => entry.name)
    .sort();
  return matches.length > 0 ? matches[0] : null;
}

// Ecosystem conventions fire only when no explicit script/target matched the
// key. The candidate carries the marker file as source so the user can see it
// is a convention to confirm, not a recorded fact (propose-only per D3).
const CONVENTION_SOURCES = [
  { key: 'test', value: 'pytest', marker: (root) => markerIfExists(root, 'pyproject.toml') },
  { key: 'test', value: 'dotnet test', marker: firstCsproj },
  { key: 'test', value: 'go test ./...', marker: (root) => markerIfExists(root, 'go.mod') },
];

/**
 * Scan a repo root's manifests and return command candidates for the
 * COMMAND_KEYS: [{ key, value, source }], at most one per key. Priority when
 * sources conflict: package.json, Makefile, composer.json, then conventions
 * (pyproject.toml, *.csproj, go.mod). Pure read — proposes, never records.
 */
export function detectCommands(root) {
  const resolved = path.resolve(root || process.cwd());
  const byKey = new Map();

  const explicit = [
    ...packageJsonCandidates(resolved),
    ...makefileCandidates(resolved),
    ...composerCandidates(resolved),
  ];
  for (const candidate of explicit) {
    if (!byKey.has(candidate.key)) byKey.set(candidate.key, candidate);
  }

  for (const convention of CONVENTION_SOURCES) {
    if (byKey.has(convention.key)) continue;
    const marker = convention.marker(resolved);
    if (marker) byKey.set(convention.key, { key: convention.key, value: convention.value, source: marker });
  }

  return COMMAND_KEYS.filter((key) => byKey.has(key)).map((key) => byKey.get(key));
}

// Guarded CLI entry: `node commands_detect.mjs [root]` prints the JSON
// candidate list; importing the module stays side-effect-free.
const invokedDirectly = (() => {
  if (!process.argv[1]) return false;
  const entry = pathToFileURL(path.resolve(process.argv[1])).href;
  return process.platform === 'win32'
    ? entry.toLowerCase() === import.meta.url.toLowerCase()
    : entry === import.meta.url;
})();

if (invokedDirectly) {
  const target = process.argv[2] || process.cwd();
  process.stdout.write(`${JSON.stringify(detectCommands(target), null, 2)}\n`);
}
