// guard-lite.mjs — the SINGLE home of the static read-policy constants
// (exec-speed cell es-3, CONTEXT D3). Deliberately dependency-free (no other
// lib imports, no I/O) so the write-guard hook's allow-only fast path for
// Read/Glob/Grep can consume these WITHOUT importing state.mjs/guards.mjs.
// guards.mjs imports the secret/scout policy back from here and re-exports it
// under its existing names — policy bytes stay identical, defined once.
//
// Do NOT add logic here: this module is constants only. Any change to a
// pattern below changes BOTH the fast path and the full guard at once, which
// is exactly the point.

/** File-path patterns that must never be read without asking the human. */
export const SECRET_PATTERNS = [
  /(^|[\\/])\.env(\.[A-Za-z0-9._-]+)?$/i,
  /\.pem$/i,
  /\.key$/i,
  /(^|[\\/])id_rsa[^\\/]*$/i,
  /\.p12$/i,
  /(^|[\\/])credentials[^\\/]*$/i,
  /(^|[\\/])secrets\.[^\\/]+$/i,
];

/** Directories agents should never scout through. */
export const SCOUT_DIRS = [
  'node_modules/',
  'dist/',
  'build/',
  '.git/objects',
  'vendor/',
  'coverage/',
  '.next/',
  '__pycache__/',
];

// Read-size guard thresholds (router-cost D1/D2/D3/D4; moved here from
// bee-write-guard.mjs by es-3 so the fast path and the full path share one
// definition). The line threshold default is overridable per machine via
// `guards.max_read_lines` in .bee/config(.local).json.
export const DEFAULT_MAX_READ_LINES = 800;
// Files larger than this are never measured — counting lines would mean
// reading the whole thing into memory just to decide whether to deny reading
// the whole thing into context. Fail-open (allow) instead.
export const READ_SIZE_GUARD_CAP_BYTES = 25 * 1024 * 1024;
