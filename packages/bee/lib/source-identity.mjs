// source-identity.mjs — SRC-01..06: classify the running launcher's source
// identity, so onboarding and `bee status` share ONE detector (DIST-04) instead
// of each guessing from the nearest path (SRC-01).
//
// PURE: only read probes (existsSync / realpathSync / readFileSync). It never
// mutates anything and never throws — any probe failure, unparseable manifest,
// or ambiguity resolves to `unknown` (fail-closed, SRC-04). Classification is a
// report/corroboration surface; onboarding's authoritative-source refusal
// (identityOk) remains the decision-maker.

import fs from 'node:fs';
import path from 'node:path';

// The render provenance sidecar (D9): a skills root carrying it is a rendered
// per-runtime PROJECTION, refused as an onboarding source for any target.
export const RENDER_SIDECAR = '.bee-render.json';

export const SOURCE_KINDS = [
  'source_checkout', // the canonical dev working checkout (plugin.json + .git)
  'rendered_projection', // a skills root produced by the runtime renderer (has RENDER_SIDECAR) — never a source (D9)
  'project_projection', // a host repo's vendored .agents/skills or .claude/skills copy
  'plugin_package', // an installed manifested snapshot (plugin.json, no .git) — never a global/plugin authority (SRC-03)
  'legacy_global', // the legacy global ~/.claude/skills root — reported/migrated, never an implicit source (SRC-06)
  'unknown', // missing/unparseable manifest or ambiguous — fail closed before mutation (SRC-04)
];

function realpathOrNull(p) {
  try {
    return fs.realpathSync(p);
  } catch {
    return null;
  }
}

function existsSafe(p) {
  try {
    return fs.existsSync(p);
  } catch {
    return false;
  }
}

/**
 * Classify the source identity of a running launcher.
 * @param {{ hiveDir: string, homeDir?: string }} input
 *   hiveDir — the running launcher's `.../bee-hive` directory.
 *   homeDir — the user's home dir (for the legacy global-root check); optional.
 * @returns {{ kind: string, root: string|null, markers: object }}
 */
export function classifySource({ hiveDir, homeDir } = {}) {
  if (!hiveDir || typeof hiveDir !== 'string') {
    return { kind: 'unknown', root: null, markers: { reason: 'no hiveDir' } };
  }
  const sourceRoot = path.dirname(hiveDir); // .../skills (or .agents/skills, .claude/skills)
  const pluginRoot = path.dirname(sourceRoot); // the package / repo root
  const markers = { source_root: sourceRoot, plugin_root: pluginRoot };

  // (0) rendered_projection FIRST — a skills root carrying the render sidecar is
  // per-runtime renderer OUTPUT, never a source. Checked ahead of every other
  // kind so a rendered .claude/.agents root (which would otherwise read as
  // project_projection) is refused as a source for ANY target (D9 provenance).
  if (existsSafe(path.join(sourceRoot, RENDER_SIDECAR))) {
    return { kind: 'rendered_projection', root: pluginRoot, markers: { ...markers, render_sidecar: true } };
  }

  // (1) legacy_global FIRST — the global ~/.claude/skills root also has a
  // `.claude` grandparent, so it would collide with project_projection below;
  // the realpath match to the true global root disambiguates it.
  if (homeDir && typeof homeDir === 'string') {
    const globalRoot = path.join(homeDir, '.claude', 'skills');
    const rp = realpathOrNull(sourceRoot);
    const rpGlobal = realpathOrNull(globalRoot);
    if (rp && rpGlobal && rp === rpGlobal) {
      return { kind: 'legacy_global', root: pluginRoot, markers: { ...markers, global_root: globalRoot } };
    }
  }

  // (2) project_projection — launcher under a host's .agents/skills or .claude/skills.
  const projectionParent = path.basename(pluginRoot);
  if (projectionParent === '.agents' || projectionParent === '.claude') {
    return { kind: 'project_projection', root: pluginRoot, markers: { ...markers, projection_parent: projectionParent } };
  }

  // (3)/(4) a manifested package: .claude-plugin/plugin.json at the package root.
  const pluginManifest = path.join(pluginRoot, '.claude-plugin', 'plugin.json');
  if (existsSafe(pluginManifest)) {
    // SRC-04: an unparseable manifest is `unknown`, never a usable source.
    try {
      JSON.parse(fs.readFileSync(pluginManifest, 'utf8'));
    } catch {
      return { kind: 'unknown', root: pluginRoot, markers: { ...markers, reason: 'plugin.json unparseable' } };
    }
    if (existsSafe(path.join(pluginRoot, '.git'))) {
      return { kind: 'source_checkout', root: pluginRoot, markers: { ...markers, plugin_manifest: true, git: true } };
    }
    // plugin.json without .git — a distributed snapshot. SRC-03: it may source
    // the same repo's runtime + projection, but is NEVER a global/plugin-target
    // authority.
    return {
      kind: 'plugin_package',
      root: pluginRoot,
      markers: { ...markers, plugin_manifest: true, git: false, can_target_global: false },
    };
  }

  // (5) unknown — no manifest, not a projection, not the global root: fail closed.
  return { kind: 'unknown', root: pluginRoot, markers: { ...markers, reason: 'no plugin manifest' } };
}
