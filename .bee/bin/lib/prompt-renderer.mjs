// prompt-renderer.mjs — minimal template loader/renderer for the prompt files
// in ../prompts/ (prompt-files spec §1). Zero dependencies, and the prompts
// directory is resolved RELATIVE TO THIS MODULE, so the canonical checkout
// reads packages/bee/prompts/ and the vendored .bee/bin/lib copy reads
// .bee/bin/prompts/ — hosts always run the prompts matching their vendored
// engine.
//
// THE SPLIT: templates hold wording; code computes which {{#if name}} blocks
// appear and what fills the {{name}} placeholders. Substitution is plain —
// values (cell JSON, event lines) are inserted verbatim, never escaped, and a
// substituted value is never re-scanned for markers. Conditional blocks do
// not nest; a nested {{#if}} is a loud refusal, not a silent misrender.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const PROMPTS_DIR = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'prompts');
const cache = new Map(); // per-process template cache, keyed by name

/** Read prompts/<name>.md (cached). CRLF is normalized to LF and ONE trailing
 *  newline is stripped, so a checked-out template renders byte-identically to
 *  the historical join('\n') string builders it replaced. */
export function loadPrompt(name) {
  if (cache.has(name)) return cache.get(name);
  const raw = fs.readFileSync(path.join(PROMPTS_DIR, `${name}.md`), 'utf8');
  const template = raw.replace(/\r\n/g, '\n').replace(/\n$/, '');
  cache.set(name, template);
  return template;
}

/** Substitute {{name}} placeholders and process {{#if name}}…{{/if}} blocks.
 *  Each marker sits on its own line; the marker line AND the newline before it
 *  are consumed, so a dropped block leaves zero residue bytes and a kept block
 *  splices its inner lines in exactly. Truthy vars[name] keeps the inner
 *  content (with substitution); falsy drops the whole block. */
export function render(template, vars = {}) {
  const out = template.replace(/\n\{\{#if ([A-Za-z0-9_]+)\}\}([\s\S]*?)\n\{\{\/if\}\}/g, (_m, name, inner) => {
    if (inner.includes('{{#if ')) {
      throw new Error(`prompt-renderer: nested {{#if}} inside block "${name}" — nesting is not supported.`);
    }
    return vars[name] ? inner : '';
  });
  if (out.includes('{{#if ') || out.includes('{{/if}}')) {
    throw new Error('prompt-renderer: unmatched or malformed {{#if}}/{{/if}} marker in template.');
  }
  return out.replace(/\{\{([A-Za-z0-9_]+)\}\}/g, (_m, name) => {
    if (vars[name] === undefined || vars[name] === null) {
      throw new Error(`prompt-renderer: no value supplied for placeholder {{${name}}}.`);
    }
    return String(vars[name]);
  });
}
