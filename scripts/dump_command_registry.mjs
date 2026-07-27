// dump_command_registry.mjs — file-based bridge that snapshots
// `.bee/bin/lib/command-registry.mjs`'s `COMMAND_REGISTRY` (116 defs, 19
// group prefixes at scout time) to `.bee/cache/command-registry.json` for
// bee-core's Rust registry loader (`crates/bee-core/src/registry.rs`,
// rust-port-8, CONTEXT.md D3/D7).
//
// Promoted from the proven spike at `.bee/spikes/rust-port/dump_registry.mjs`
// (same import shape: a tracked FILE statically importing COMMAND_REGISTRY,
// never a `node -e` inline eval — the internals-reach guard denies those,
// not file-based imports; see `.bee/bin/lib/guards.mjs`'s
// `checkBinLibImportBashCommand`).
//
// `command-registry.mjs` is FROZEN for the duration of the rust-port
// feature (D1) — this script only reads it, never edits it. The embedded
// `source_sha256` is the staleness anchor bee-core's loader compares
// against a fresh hash of the live file: a dump run before the LAST edit to
// command-registry.mjs must never be silently treated as current.
//
// Delivery note (decision 2026-07-26, cell rust-port-8): at final flip this
// JSON becomes a release/onboard-generated managed artifact — this script's
// output shape is final, only where it gets invoked from moves. Until then
// it is a manually-run dev/CI step; the output never gets committed
// (`.bee/cache/` is gitignored, both by the managed BEE:START/BEE:END fence
// and, redundantly and deliberately, by a standalone line this cell adds
// outside that fence).

import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(here, '..');
const sourcePath = path.join(repoRoot, '.bee', 'bin', 'lib', 'command-registry.mjs');
const outDir = path.join(repoRoot, '.bee', 'cache');
const outPath = path.join(outDir, 'command-registry.json');

const { COMMAND_REGISTRY } = await import(`file://${sourcePath}`);

const sourceBytes = fs.readFileSync(sourcePath);
const sourceSha256 = crypto.createHash('sha256').update(sourceBytes).digest('hex');

const dump = {
  source_sha256: sourceSha256,
  generated_at: new Date().toISOString(),
  entries: COMMAND_REGISTRY,
};

fs.mkdirSync(outDir, { recursive: true });
fs.writeFileSync(outPath, `${JSON.stringify(dump, null, 2)}\n`);

console.log(
  JSON.stringify({
    ok: true,
    entries: COMMAND_REGISTRY.length,
    bytes: fs.statSync(outPath).size,
    out: path.relative(repoRoot, outPath),
    source_sha256: sourceSha256,
  }),
);
