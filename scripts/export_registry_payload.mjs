#!/usr/bin/env node
// export_registry_payload.mjs — writes the EXACT manifest-hash payload string
// (bee.mjs computeManifestHash: JSON.stringify({schema_version, commands}))
// to packages/bee-rs/crates/bee/src/generated/registry_payload.json, where
// the Rust binary embeds it via include_str!. Both runtimes then hash the
// same bytes, so .bee/cache/manifest-hash.json never flip-flops between them.
//
// Sync chain: re-run this after ANY command-registry.mjs change, then rebuild
// bee-rs. The Rust test `embedded_registry_payload_is_fresh` fails loudly
// when this step is forgotten.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { SCHEMA_VERSION, COMMAND_REGISTRY } from '../packages/bee/lib/command-registry.mjs';

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const out = path.join(repoRoot, 'packages', 'bee-rs', 'crates', 'bee', 'src', 'generated', 'registry_payload.json');

const payload = JSON.stringify({ schema_version: SCHEMA_VERSION, commands: COMMAND_REGISTRY });
fs.mkdirSync(path.dirname(out), { recursive: true });
fs.writeFileSync(out, payload, 'utf8');
console.log(`registry payload -> ${path.relative(repoRoot, out)} (${payload.length} bytes)`);
