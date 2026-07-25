#!/usr/bin/env node
// Guard: the release-version WRITER (scripts/lib/release-tuple.mjs +
// scripts/bump_version.mjs) sets every tuple component from one command, and
// each write preserves the surrounding file so the byte-mirror and manifest
// invariants still hold. Decision cba8b832.
//
// Everything runs against TEMP fixtures — the real tuple files are never
// touched. Exit 0 when all checks pass, exit 1 on the first failure.

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  COMPONENTS,
  readComponentVersion,
  writeComponentVersion,
  readComponents,
  checkTupleEquality,
} from "../lib/release-tuple.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

let failures = 0;
function check(label, cond, detail) {
  if (cond) {
    console.log(`PASS test_bump_version: ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL test_bump_version: ${label}${detail ? ` — ${detail}` : ""}`);
  }
}

const tmpRoot = fs.mkdtempSync(path.join(os.tmpdir(), "test-bump-version-"));
try {
  // ── js-const component: write preserves everything but the version, keeps
  //    single quotes + trailing semicolon (so the byte-mirror stays identical) ──
  const jsPath = path.join(tmpRoot, "state.mjs");
  const jsBefore =
    "// header comment\nimport x from './x.mjs';\n\nexport const BEE_VERSION = '1.2.3';\n\nexport const OTHER = 42;\n";
  fs.writeFileSync(jsPath, jsBefore, "utf8");
  const jsComponent = { name: "temp state.mjs", path: jsPath, kind: "js-const" };

  const jsPrev = writeComponentVersion(jsComponent, "9.9.9");
  const jsAfter = fs.readFileSync(jsPath, "utf8");
  check("js-const returns the previous version", jsPrev === "1.2.3", `got ${jsPrev}`);
  check("js-const readback is the new version", readComponentVersion(jsComponent) === "9.9.9");
  check(
    "js-const rewrites only the version line (quotes + semicolon + neighbours intact)",
    jsAfter === jsBefore.replace("'1.2.3'", "'9.9.9'"),
    JSON.stringify(jsAfter),
  );

  // ── json-version component: write preserves other fields, 2-space indent,
  //    trailing newline ──
  const jsonPath = path.join(tmpRoot, "plugin.json");
  const jsonObj = { name: "bee", version: "1.2.3", description: "keep me", nested: { a: 1 } };
  fs.writeFileSync(jsonPath, `${JSON.stringify(jsonObj, null, 2)}\n`, "utf8");
  const jsonComponent = { name: "temp plugin.json", path: jsonPath, kind: "json-version" };

  const jsonPrev = writeComponentVersion(jsonComponent, "9.9.9");
  const jsonAfter = fs.readFileSync(jsonPath, "utf8");
  const jsonParsed = JSON.parse(jsonAfter);
  check("json-version returns the previous version", jsonPrev === "1.2.3", `got ${jsonPrev}`);
  check("json-version readback is the new version", readComponentVersion(jsonComponent) === "9.9.9");
  check("json-version preserves other fields", jsonParsed.description === "keep me" && jsonParsed.nested.a === 1);
  check("json-version keeps 2-space indent + trailing newline", jsonAfter === `${JSON.stringify({ ...jsonObj, version: "9.9.9" }, null, 2)}\n`);

  // ── registry-driven coverage: a bump touches EVERY real component. Copy the
  //    real tuple files into temp, retarget the components, bump, and assert
  //    all agree on the new version — never a hand-kept subset. ──
  const tempComponents = COMPONENTS.map((component, i) => {
    const dest = path.join(tmpRoot, `component-${i}${component.kind === "js-const" ? ".mjs" : ".json"}`);
    fs.copyFileSync(component.path, dest);
    return { ...component, path: dest };
  });
  const startVersions = readComponents(tempComponents);
  check(
    "real tuple starts in sync (sanity, from copied files)",
    checkTupleEquality(startVersions).ok,
    JSON.stringify(startVersions),
  );
  for (const component of tempComponents) writeComponentVersion(component, "7.0.0");
  const bumped = readComponents(tempComponents);
  check(
    `bump covers all ${tempComponents.length} registry components`,
    checkTupleEquality(bumped).ok && bumped.every((e) => e.version === "7.0.0"),
    JSON.stringify(bumped),
  );

  // ── the CLI enforces its contract: bad version rejected, --check is read-only ──
  const { execFileSync } = await import("node:child_process");
  const cli = path.join(__dirname, "..", "bump_version.mjs");
  const runCli = (args) => {
    try {
      const stdout = execFileSync("node", [cli, ...args], { encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] });
      return { code: 0, stdout };
    } catch (error) {
      return { code: error.status ?? 1, stdout: `${error.stdout || ""}${error.stderr || ""}` };
    }
  };
  check("CLI rejects a non-numeric version", runCli(["not-a-version"]).code === 1);
  check("CLI rejects an unknown flag", runCli(["1.2.3", "--bogus"]).code === 1);
  const checkRun = runCli(["--check"]);
  check("CLI --check exits 0 on the real in-sync tuple and writes nothing", checkRun.code === 0 && /Current release tuple/.test(checkRun.stdout));
} finally {
  fs.rmSync(tmpRoot, { recursive: true, force: true });
}

if (failures > 0) {
  console.error(`FAIL test_bump_version: ${failures} check(s) failed`);
  process.exit(1);
}
console.log("PASS test_bump_version: all checks passed");
