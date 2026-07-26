// capture.mjs — the capture queue (decision 0017): durable-now, elaborate-later.
// A settlement in tiny/small/standard lanes appends a one-line stub here in the
// same turn (the decision log remains the anchor); the expensive BA-grade spec
// merge happens at a flush point (wrap-up, PreCompact, or next session).
// High-risk lane never queues — it syncs inline (decision 0017).
// Append-only JSONL: `stub` records + `flush` records; pending = stubs − flushed.

import path from 'node:path';
import crypto from 'node:crypto';
import { appendJsonl, readJsonl } from './fsutil.mjs';
import { SECRET_CONTENT_PATTERNS, INJECTION_PATTERNS } from './decisions.mjs';

export function captureQueuePath(root) {
  return path.join(root, '.bee', 'capture-queue.jsonl');
}

function assertSafeContent(field, value) {
  if (typeof value !== 'string' || !value) return;
  for (const pattern of SECRET_CONTENT_PATTERNS) {
    if (pattern.test(value)) {
      throw new Error(
        `Capture stub rejected: field "${field}" matches a secret pattern (${pattern}). Never queue credentials — describe the outcome without the secret.`,
      );
    }
  }
  for (const pattern of INJECTION_PATTERNS) {
    if (pattern.test(value)) {
      throw new Error(
        `Capture stub rejected: field "${field}" contains instruction-like content (${pattern}). Stub text must be data, not instructions.`,
      );
    }
  }
}

function normalizeList(value) {
  if (Array.isArray(value)) {
    return value.map((v) => String(v).trim()).filter(Boolean);
  }
  if (typeof value === 'string' && value.trim()) {
    return value
      .split(',')
      .map((v) => v.trim())
      .filter(Boolean);
  }
  return [];
}

/**
 * Append a capture stub — the same-turn durability record for a settlement
 * whose full spec merge is deferred to flush. `outcome` is required; `dids`
 * (decision ids), `area`, `files`, `lane` are optional context for the
 * flusher. `source` is optional provenance (e.g. "mined" for a transcript-
 * recovery candidate settlement, transcript-recovery D6) — additive only: a
 * stub created without it stays byte-shape-identical to today's stubs, so
 * the key is omitted rather than defaulted to null.
 */
export function addCaptureStub(root, { outcome, dids = null, area = null, files = null, lane = null, source = null }) {
  if (typeof outcome !== 'string' || !outcome.trim()) {
    throw new Error('addCaptureStub: outcome text is required.');
  }
  if (lane === 'high-risk') {
    throw new Error(
      'addCaptureStub: high-risk settlements never queue — run the full bee-scribing sync inline (decision 0017).',
    );
  }
  const stub = {
    kind: 'stub',
    id: crypto.randomUUID(),
    at: new Date().toISOString(),
    outcome: outcome.trim(),
    dids: normalizeList(dids),
    area: typeof area === 'string' && area.trim() ? area.trim() : null,
    files: normalizeList(files),
    lane: typeof lane === 'string' && lane.trim() ? lane.trim() : null,
  };
  const normalizedSource = typeof source === 'string' && source.trim() ? source.trim() : null;
  if (normalizedSource) stub.source = normalizedSource;
  assertSafeContent('outcome', stub.outcome);
  if (stub.area) assertSafeContent('area', stub.area);
  appendJsonl(captureQueuePath(root), stub);
  return stub;
}

/** Pending stubs (not yet flushed), oldest first. */
export function pendingCaptureStubs(root) {
  const events = readJsonl(captureQueuePath(root));
  const flushed = new Set();
  const stubs = [];
  for (const event of events) {
    if (!event || typeof event !== 'object') continue;
    if (event.kind === 'flush' && event.id) flushed.add(event.id);
    else if (event.kind === 'stub' && event.id) stubs.push(event);
  }
  return stubs
    .filter((stub) => !flushed.has(stub.id))
    .sort((a, b) => String(a.at).localeCompare(String(b.at)));
}

/** Convenience summary for status surfaces. */
export function captureQueue(root) {
  const stubs = pendingCaptureStubs(root);
  return { count: stubs.length, stubs };
}

/**
 * Mark a stub flushed (its content merged into a spec by bee-scribing).
 * `into` names where it landed (e.g. "docs/specs/<area>.md").
 */
export function flushCaptureStub(root, id, { into = null } = {}) {
  if (typeof id !== 'string' || !id.trim()) {
    throw new Error('flushCaptureStub: stub id is required.');
  }
  const pending = pendingCaptureStubs(root);
  const stub = pending.find((s) => s.id === id.trim());
  if (!stub) {
    throw new Error(`flushCaptureStub: no pending stub with id "${id}".`);
  }
  const record = {
    kind: 'flush',
    id: stub.id,
    at: new Date().toISOString(),
    into: typeof into === 'string' && into.trim() ? into.trim() : null,
  };
  appendJsonl(captureQueuePath(root), record);
  return record;
}
