#!/usr/bin/env node
// Release manifest generator + checker (DIST-01/DIST-03/D-03, decision ed0b2920).
//
// Enumerates the release-identity file set for the bee distribution:
//   - packages/bee/** (excl. any hooks/ subtree) -> role "package_payload"
//   - .bee/bin/lib/*.mjs                    -> role "runtime_lib"
//   - expertise/*.md                        -> role "expertise_guide"
//   - .bee/expertise/*.md (vendored copy)   -> role "runtime_expertise"
//   - skills/** and hooks/**                 -> canonical plugin package
//   - both plugin manifests + marketplace   -> plugin metadata
//   - both installers + distribution engine -> migration machinery
//
// The lib directories are enumerated via fs.readdirSync — never a hand-kept
// list (crit-pattern 20260714: hand-kept file lists silently drift from the
// real tree). The two plugin.json files are individually named because they
// are not part of an enumerable lib directory.
//
// Subcommands:
//   --write     regenerate docs/history/codex-harness-hardening/release-manifest.json
//   --check     recompute the manifest from the current tree and compare it
//               against the stored one; prints each mismatch; exit 1 on any
//               path-set/hash/mode diff, exit 0 when identical
//   --selftest  proves the comparison logic actually bites: takes the real
//               (unmutated) manifest as a baseline, mutates ONE covered
//               file's content in a temp copy (never the real tree), and
//               asserts compareManifests() flags exactly that file. Exit 0
//               if the bite is proven, exit 1 if not.

import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createHash } from "node:crypto";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..");
const MANIFEST_PATH = path.join(
  REPO_ROOT,
  "docs",
  "history",
  "codex-harness-hardening",
  "release-manifest.json",
);

const RUNTIME_LIB_DIR = path.join(REPO_ROOT, ".bee", "bin", "lib");
// expertise-vendoring: the craft-guide layer. expertise/ is the SOURCE
// (authored guides); .bee/expertise/ is this repo's own vendored copy,
// produced by self-onboard exactly like .bee/bin/lib above — both enumerated
// via readdir, never hand-listed (crit-pattern 20260714).
const EXPERTISE_SOURCE_DIR = path.join(REPO_ROOT, "expertise");
const RUNTIME_EXPERTISE_DIR = path.join(REPO_ROOT, ".bee", "expertise");
// D9/cnr2-12: the committed per-runtime plugin skill-route trees (render at
// packages/bee/scripts/onboard_bee.mjs::renderSkillBytes). Distinct roles
// from "plugin_skill" (the canonical skills/ tree, still hashed unchanged for
// package integrity) so managedSkillNames() — which parses "skills/bee-*"
// paths — never sees these ".claude-plugin/skills/..." / ".codex-plugin/
// skills/..." paths.
const PLUGIN_SKILL_RENDER_ROOTS = [
  { dir: path.join(REPO_ROOT, ".claude-plugin", "skills"), role: "plugin_skill_claude_render" },
  { dir: path.join(REPO_ROOT, ".codex-plugin", "skills"), role: "plugin_skill_codex_render" },
];
const NAMED_PLUGIN_MANIFESTS = [
  path.join(REPO_ROOT, ".claude-plugin", "plugin.json"),
  path.join(REPO_ROOT, ".codex-plugin", "plugin.json"),
];
const PLUGIN_MARKETPLACE = path.join(REPO_ROOT, ".claude-plugin", "marketplace.json");
const DISTRIBUTION_TOOLS = [
  path.join(REPO_ROOT, "scripts", "install.sh"),
  path.join(REPO_ROOT, "scripts", "install.ps1"),
];
const DISTRIBUTION_TESTS = [
  path.join(REPO_ROOT, "scripts", "tests", "test_verify_manifest.mjs"),
  path.join(REPO_ROOT, "scripts", "tests", "test_release_tuple.mjs"),
];

const SCHEMA_VERSION = 1;

/** repo-relative POSIX path for an absolute path under REPO_ROOT. */
function relPosix(absPath) {
  return path.relative(REPO_ROOT, absPath).split(path.sep).join("/");
}

function sha256File(absPath) {
  const data = fs.readFileSync(absPath);
  return createHash("sha256").update(data).digest("hex");
}

function modeOctal(absPath) {
  const mode = fs.statSync(absPath).mode & 0o777;
  return mode.toString(8).padStart(3, "0");
}

function buildRecord(absPath, role, packagePath = null) {
  const record = {
    path: relPosix(absPath),
    sha256: sha256File(absPath),
    mode: modeOctal(absPath),
    role,
  };
  if (packagePath) record.packagePath = packagePath;
  return record;
}

// `excludeTopDirNames` skips an immediate child directory by name (never
// nested matches) - used to carve the hooks/ subtree out of the packages/bee
// payload walk once cell 2 lands it at packages/bee/hooks/ (the dedicated
// plugin_hook walk below stays the single source for that content; D5).
function enumerateTree(dirAbsPath, role, { excludeTopDirNames = [] } = {}) {
  if (!fs.existsSync(dirAbsPath)) throw new Error(`release_manifest: expected directory missing: ${dirAbsPath}`);
  const records = [];
  const walk = (current) => {
    for (const entry of fs.readdirSync(current, { withFileTypes: true })) {
      if (current === dirAbsPath && entry.isDirectory() && excludeTopDirNames.includes(entry.name)) {
        continue;
      }
      const absPath = path.join(current, entry.name);
      if (entry.isSymbolicLink()) throw new Error(`release_manifest: symlink forbidden in package inventory: ${absPath}`);
      if (entry.isDirectory()) walk(absPath);
      else if (entry.isFile()) records.push(buildRecord(absPath, role, relPosix(absPath)));
      else throw new Error(`release_manifest: unsupported package entry: ${absPath}`);
    }
  };
  walk(dirAbsPath);
  return records.sort((a, b) => a.path.localeCompare(b.path));
}

/** Enumerate files with `ext` directly inside dirAbsPath (no recursion), sorted. */
function enumerateFlatDir(dirAbsPath, role, ext) {
  if (!fs.existsSync(dirAbsPath)) {
    throw new Error(`release_manifest: expected directory missing: ${dirAbsPath}`);
  }
  return fs
    .readdirSync(dirAbsPath, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(ext))
    .map((entry) => buildRecord(path.join(dirAbsPath, entry.name), role))
    .sort((a, b) => a.path.localeCompare(b.path));
}

/** Enumerate *.mjs files directly inside dirAbsPath (no recursion), sorted. */
function enumerateMjsDir(dirAbsPath, role) {
  return enumerateFlatDir(dirAbsPath, role, ".mjs");
}

/**
 * Build the current manifest (array of records) by re-reading the real repo
 * tree. Read-only — never mutates anything on disk.
 */
function buildCurrentRecords() {
  const records = [
    ...enumerateMjsDir(RUNTIME_LIB_DIR, "runtime_lib"),
    ...enumerateFlatDir(EXPERTISE_SOURCE_DIR, "expertise_guide", ".md"),
    ...enumerateFlatDir(RUNTIME_EXPERTISE_DIR, "runtime_expertise", ".md"),
    ...enumerateTree(path.join(REPO_ROOT, "packages", "bee"), "package_payload", {
      excludeTopDirNames: ["hooks"],
    }),
    ...enumerateTree(path.join(REPO_ROOT, "skills"), "plugin_skill"),
    ...PLUGIN_SKILL_RENDER_ROOTS.flatMap(({ dir, role }) => enumerateTree(dir, role)),
    ...enumerateTree(path.join(REPO_ROOT, "packages", "bee", "hooks"), "plugin_hook"),
    ...NAMED_PLUGIN_MANIFESTS.map((absPath) => {
      if (!fs.existsSync(absPath)) {
        throw new Error(`release_manifest: expected plugin manifest missing: ${absPath}`);
      }
      return buildRecord(absPath, "plugin_manifest", relPosix(absPath));
    }),
    buildRecord(PLUGIN_MARKETPLACE, "plugin_marketplace", relPosix(PLUGIN_MARKETPLACE)),
    ...DISTRIBUTION_TOOLS.map((absPath) => buildRecord(absPath, "distribution_tool")),
    ...DISTRIBUTION_TESTS.map((absPath) => buildRecord(absPath, "distribution_test")),
  ];
  records.sort((a, b) => a.path.localeCompare(b.path));
  const duplicates = records.filter((record, index) => index > 0 && record.path === records[index - 1].path);
  if (duplicates.length) throw new Error(`release_manifest: duplicate inventory path(s): ${duplicates.map((record) => record.path).join(", ")}`);
  return records;
}

function writeManifestFile(records) {
  const manifest = { schemaVersion: SCHEMA_VERSION, files: records };
  fs.mkdirSync(path.dirname(MANIFEST_PATH), { recursive: true });
  fs.writeFileSync(MANIFEST_PATH, `${JSON.stringify(manifest, null, 2)}\n`, "utf8");
  return manifest;
}

function readStoredManifest() {
  if (!fs.existsSync(MANIFEST_PATH)) {
    throw new Error(`release_manifest: stored manifest missing: ${MANIFEST_PATH} (run --write first)`);
  }
  const parsed = JSON.parse(fs.readFileSync(MANIFEST_PATH, "utf8"));
  if (!parsed || !Array.isArray(parsed.files)) {
    throw new Error(`release_manifest: stored manifest malformed: ${MANIFEST_PATH}`);
  }
  return parsed.files;
}

/**
 * Compare two record arrays (stored vs current). Returns:
 *   { ok, missing, added, changed }
 * - missing: paths present in stored but not current
 * - added: paths present in current but not stored
 * - changed: [{ path, reasons: ["sha256"|"mode"|"role"] }] for paths in both
 *   whose sha256/mode/role differ
 * ok === true iff missing/added/changed are all empty.
 */
function compareManifests(stored, current) {
  const storedByPath = new Map(stored.map((r) => [r.path, r]));
  const currentByPath = new Map(current.map((r) => [r.path, r]));

  const missing = [...storedByPath.keys()].filter((p) => !currentByPath.has(p)).sort();
  const added = [...currentByPath.keys()].filter((p) => !storedByPath.has(p)).sort();

  const changed = [];
  for (const [p, storedRecord] of storedByPath) {
    const currentRecord = currentByPath.get(p);
    if (!currentRecord) continue;
    const reasons = [];
    if (storedRecord.sha256 !== currentRecord.sha256) reasons.push("sha256");
    if (storedRecord.mode !== currentRecord.mode) reasons.push("mode");
    if (storedRecord.role !== currentRecord.role) reasons.push("role");
    if ((storedRecord.packagePath ?? null) !== (currentRecord.packagePath ?? null)) reasons.push("packagePath");
    if (reasons.length > 0) changed.push({ path: p, reasons });
  }
  changed.sort((a, b) => a.path.localeCompare(b.path));

  const ok = missing.length === 0 && added.length === 0 && changed.length === 0;
  return { ok, missing, added, changed };
}

function printDiff(diffResult) {
  for (const p of diffResult.missing) {
    console.error(`MISMATCH missing (in stored manifest, absent from current tree): ${p}`);
  }
  for (const p of diffResult.added) {
    console.error(`MISMATCH added (in current tree, absent from stored manifest): ${p}`);
  }
  for (const c of diffResult.changed) {
    console.error(`MISMATCH ${c.path}: ${c.reasons.join(", ")} differ`);
  }
}

function runWrite() {
  const records = buildCurrentRecords();
  writeManifestFile(records);
  console.log(`WROTE ${relPosix(MANIFEST_PATH)}: ${records.length} file(s)`);
  return 0;
}

function runCheck() {
  const stored = readStoredManifest();
  const current = buildCurrentRecords();
  const diffResult = compareManifests(stored, current);
  if (diffResult.ok) {
    console.log(`release_manifest --check: ${current.length} file(s) match stored manifest`);
    return 0;
  }
  printDiff(diffResult);
  console.error(
    `release_manifest --check: FAIL (${diffResult.missing.length} missing, ${diffResult.added.length} added, ${diffResult.changed.length} changed)`,
  );
  return 1;
}

function runSelftest() {
  // Baseline: read the real, unmutated tree. Read-only.
  const baseline = buildCurrentRecords();
  if (baseline.length === 0) {
    console.error("FAIL release_manifest --selftest: baseline manifest is empty, cannot prove anything");
    return 1;
  }

  // Pick a covered file to bite on — prefer a package_payload/runtime_lib
  // record so the mutation exercises a real enumerated file, not a named one.
  const target =
    baseline.find((r) => r.role === "package_payload" || r.role === "runtime_lib") ?? baseline[0];

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), "release-manifest-selftest-"));
  let selftestOk = false;
  try {
    const realAbsPath = path.join(REPO_ROOT, target.path.split("/").join(path.sep));
    const tempCopyPath = path.join(tempDir, path.basename(target.path));

    // Copy the real file's content into the temp dir, then mutate the COPY.
    const originalContent = fs.readFileSync(realAbsPath);
    fs.writeFileSync(tempCopyPath, originalContent);
    fs.appendFileSync(tempCopyPath, "\n// release_manifest --selftest mutation marker\n");

    const mutatedHash = sha256File(tempCopyPath);
    if (mutatedHash === target.sha256) {
      console.error("FAIL release_manifest --selftest: mutation did not change the file's hash");
      return 1;
    }

    // "current" = baseline with ONLY the target record's sha256 swapped to
    // the mutated hash — models what --check would see if the real file had
    // been changed, without ever touching the real tree.
    const mutatedCurrent = baseline.map((r) =>
      r.path === target.path ? { ...r, sha256: mutatedHash } : { ...r },
    );

    const diffResult = compareManifests(baseline, mutatedCurrent);

    const flagged = diffResult.changed.find((c) => c.path === target.path);
    const bites =
      diffResult.ok === false &&
      diffResult.missing.length === 0 &&
      diffResult.added.length === 0 &&
      diffResult.changed.length === 1 &&
      flagged !== undefined &&
      flagged.reasons.includes("sha256");

    if (!bites) {
      console.error(
        `FAIL release_manifest --selftest: comparison logic did not flag mutated file ${target.path} as expected`,
      );
      console.error(`      diff result: ${JSON.stringify(diffResult)}`);
      return 1;
    }

    console.log(
      `PASS release_manifest --selftest: comparison logic correctly flags a mutated file (${target.path}) as sha256 mismatch, exit 1`,
    );
    selftestOk = true;
    return 0;
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
    if (!selftestOk) {
      // no-op: cleanup already ran; kept for clarity that temp dir never
      // leaks regardless of pass/fail.
    }
  }
}

function main() {
  const args = process.argv.slice(2);
  const hasFlag = (name) => args.includes(name);

  const flags = ["--write", "--check", "--selftest"].filter(hasFlag);
  if (flags.length !== 1) {
    console.error("usage: release_manifest.mjs (--write | --check | --selftest)");
    process.exit(1);
  }

  let exitCode;
  try {
    if (hasFlag("--write")) exitCode = runWrite();
    else if (hasFlag("--check")) exitCode = runCheck();
    else exitCode = runSelftest();
  } catch (error) {
    console.error(`FAIL release_manifest: ${error.message}`);
    process.exit(1);
  }
  process.exit(exitCode);
}

main();
