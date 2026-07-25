// judge.mjs — D5 (self-correcting-loop): one structured verdict schema for
// every semantic judge (the risk-scaled goal-check judge D4 dispatches, and
// any future review-tier judge that reuses the same shape), validated in ONE
// place so no caller hand-rolls its own acceptance check. Zero I/O, pure
// functions only — matching the CONTEXT D5 prohibition "no dispatching logic
// in lib (validation only)" and dispatch-guard.mjs's own deriveEconomics
// contract ("a pure function — zero I/O, zero fs/log access").
//
// cells.mjs's recordJudgeVerdict is the sole mutator that turns a validated
// verdict into a trace.semantic_judge entry; this module never touches a
// cell, a file, or the claim/reservation stores.

import { PINNED_MODEL_STATUS } from './dispatch-guard.mjs';

export const JUDGE_VERDICT_SCHEMA = 'judge-verdict/1';
export const JUDGE_VERDICTS = ['PASS', 'NEEDS_REVISION'];
export const CHECK_STATUSES = ['PASS', 'FAIL'];
export const JUDGE_FIXABILITIES = ['automatic', 'authority'];
export const JUDGE_CONFIDENCES = ['low', 'medium', 'high'];
export const MODEL_INDEPENDENCE_VALUES = ['confirmed', 'same-model', 'unverified'];

function isNonEmptyString(value) {
  return typeof value === 'string' && value.trim().length > 0;
}

// validateJudgeVerdict(obj) -> {ok, errors[]} — TYPED, NEVER THROWS (D5 truth
// "the validator never throws into guard paths"). A judge that returns free
// prose (a non-object, or a string) is a failed judge run per D5 — reported
// as a validation error here, not a JS exception a caller must catch.
//
// Schema judge-verdict/1: {schema, verdict, checks[], failure_signature?,
// fixability, confidence}. `checks[]` entries are {id, status, evidence},
// evidence required non-empty (an unevidenced check is not a check).
// `failure_signature` is required exactly when any check FAILs — its
// presence with an all-PASS verdict is tolerated (harmless), its absence
// with any FAIL is a validation error.
export function validateJudgeVerdict(obj) {
  const errors = [];
  if (!obj || typeof obj !== 'object' || Array.isArray(obj)) {
    errors.push(
      'verdict must be a JSON object per schema "judge-verdict/1" (got free-form/non-object output) — a judge that returns free prose is a failed judge run, not a valid verdict.',
    );
    return { ok: false, errors };
  }
  if (obj.schema !== JUDGE_VERDICT_SCHEMA) {
    errors.push(`schema must be "${JUDGE_VERDICT_SCHEMA}", got ${JSON.stringify(obj.schema)}.`);
  }
  if (!JUDGE_VERDICTS.includes(obj.verdict)) {
    errors.push(`verdict must be one of ${JUDGE_VERDICTS.join('|')}, got ${JSON.stringify(obj.verdict)}.`);
  }
  let anyFail = false;
  if (!Array.isArray(obj.checks) || obj.checks.length === 0) {
    errors.push('checks must be a non-empty array.');
  } else {
    obj.checks.forEach((entry, i) => {
      if (!entry || typeof entry !== 'object' || Array.isArray(entry)) {
        errors.push(`checks[${i}] must be a JSON object.`);
        return;
      }
      if (!isNonEmptyString(entry.id)) errors.push(`checks[${i}].id must be a non-empty string.`);
      if (!CHECK_STATUSES.includes(entry.status)) {
        errors.push(`checks[${i}].status must be one of ${CHECK_STATUSES.join('|')}, got ${JSON.stringify(entry.status)}.`);
      } else if (entry.status === 'FAIL') {
        anyFail = true;
      }
      if (!isNonEmptyString(entry.evidence)) errors.push(`checks[${i}].evidence must be a non-empty string.`);
    });
    // hardening-3: cross-check the top-level verdict against the check
    // statuses it summarizes — the verdict is a claim ABOUT the checks, not
    // an independent field a judge can set inconsistently. Evaluated only
    // once `checks` is known to be a valid non-empty array (the branch
    // above already refused a missing/empty/non-array `checks`).
    if (obj.verdict === 'PASS' && anyFail) {
      errors.push('verdict must not be PASS when any check has status FAIL — a PASS verdict must not carry a FAIL check.');
    }
    if (obj.verdict === 'NEEDS_REVISION' && !anyFail) {
      errors.push('verdict NEEDS_REVISION requires at least one check with status FAIL — got no FAIL check among the checks.');
    }
  }
  if (!JUDGE_FIXABILITIES.includes(obj.fixability)) {
    errors.push(`fixability must be one of ${JUDGE_FIXABILITIES.join('|')}, got ${JSON.stringify(obj.fixability)}.`);
  }
  if (!JUDGE_CONFIDENCES.includes(obj.confidence)) {
    errors.push(`confidence must be one of ${JUDGE_CONFIDENCES.join('|')}, got ${JSON.stringify(obj.confidence)}.`);
  }
  if (anyFail && !isNonEmptyString(obj.failure_signature)) {
    errors.push('failure_signature is required (non-empty string) when any check has status FAIL.');
  } else if (
    obj.failure_signature !== undefined &&
    obj.failure_signature !== null &&
    !isNonEmptyString(obj.failure_signature)
  ) {
    errors.push('failure_signature, when present, must be a non-empty string.');
  }
  return { ok: errors.length === 0, errors };
}

// deriveModelIndependence(builderModel, builderStatus, judgeModel, judgeStatus)
// Reuses dispatch-guard.mjs's `effective_model_status` vocabulary for the two
// status args (PINNED_MODEL_STATUS) instead of a second hand-rolled 'pinned'
// literal that could silently drift from deriveEconomics' actual output —
// the SAME "one vocabulary" principle deriveEconomics' own module comment
// documents for transport/enforcement.
//
//   both pinned AND both named AND names differ -> 'confirmed'
//   both pinned AND both named AND names equal  -> 'same-model'
//   anything else (either side unpinned, or either name missing) -> 'unverified'
//
// Never 'confirmed' without two PINNED, differing, named models (D5 must-have
// truth) — a single missing name or an unpinned side always degrades to
// 'unverified', never a guess.
export function deriveModelIndependence(builderModel, builderStatus, judgeModel, judgeStatus) {
  const bothPinned = builderStatus === PINNED_MODEL_STATUS && judgeStatus === PINNED_MODEL_STATUS;
  const bothNamed = isNonEmptyString(builderModel) && isNonEmptyString(judgeModel);
  if (!bothPinned || !bothNamed) return 'unverified';
  return builderModel === judgeModel ? 'same-model' : 'confirmed';
}
