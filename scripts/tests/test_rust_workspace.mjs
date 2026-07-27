#!/usr/bin/env node
// rust-port-6 (D6): CI-full-run visibility for the Rust `crates/` workspace
// (queen-bee, bee-core, queen-bench, bee-parity — built and green as of cells
// rust-port-1..5). Before this suite existed, nothing in run_verify's chain
// ever touched the Rust estate, so a Rust regression could ship invisibly
// between Slice 0 and Slice 7. This wrapper closes that gap the cheap way:
// spawn cargo, mirror its exit status, never invent a substitute check.
//
// SCOPE HONESTY (advisor note 4, validation W4): this gives CI FULL-RUN
// visibility ONLY. scripts/impact_registry.mjs has zero edges into crates/ —
// Rust files are invisible to the impacted-run graph — so a local
// `node scripts/run_verify.mjs --impacted-from-git` run (commands.test) will
// NOT select this suite until the D6 dependency graph lands in Slice 7.
// Local impacted-green is NOT Rust-green in the interim. Reach it with a full
// `node scripts/run_verify.mjs` run, or scope directly with
// `node scripts/run_verify.mjs --only test_rust_workspace`.
//
// This is a NEW test file, not a change to frozen mjs mechanics (D1):
// scripts/run_verify.mjs itself is untouched — discovery is automatic via its
// scripts/tests `test_*.mjs` glob (cs-4, contention-split).

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, "..", "..");
const MANIFEST_PATH = path.join(REPO_ROOT, "crates", "Cargo.toml");

function runCargo(args, label) {
  const result = spawnSync("cargo", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
    stdio: "inherit",
  });

  // ENOENT (or any spawn-level error) means cargo itself could not be
  // launched — most commonly a missing Rust toolchain on PATH. That is a
  // configuration gap the CI runner must fix (install the toolchain), never
  // a condition this suite quietly reports as green.
  if (result.error) {
    console.error(`FAIL test_rust_workspace: could not spawn cargo for ${label} (${result.error.code || result.error.message})`);
    console.error("      FIX: install the Rust toolchain (cargo) on PATH — https://rustup.rs — this suite never silent-skips a missing toolchain.");
    process.exit(1);
  }

  if (result.signal) {
    console.error(`FAIL test_rust_workspace: ${label} was killed by signal ${result.signal}`);
    process.exit(1);
  }

  if (result.status !== 0) {
    console.error(`FAIL test_rust_workspace: ${label} exited ${result.status}`);
    process.exit(result.status ?? 1);
  }

  console.log(`PASS test_rust_workspace: ${label} exited 0`);
}

runCargo(["build", "--release", "--manifest-path", MANIFEST_PATH], "cargo build --release (crates/ workspace)");
runCargo(["test", "--manifest-path", MANIFEST_PATH], "cargo test (crates/ workspace)");

console.log("PASS test_rust_workspace: crates/ workspace build+test green (queen-bee, bee-core, queen-bench, bee-parity)");
