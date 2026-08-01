#!/usr/bin/env node
'use strict';

/**
 * classify-lane.mjs — decide whether a PBI is safe for an unattended
 * dispatcher to pick up (D6, agent-pane-orchestration; retargeted to the
 * fold by backlog-unification D6/bu-4).
 *
 * SUPERSEDED (R6a). The live path is the bee binary: `.bee/bin/bee herding classify-lane <PBI-ID>`.
 * This file is retained ONLY as the fixture the Node suites
 * packages/bee/tests/test_herding.mjs still execute; nothing in the
 * instruction layer names it any more, and no agent should run it. It is
 * deleted together with the Node engine and those suites in R6.
 *
 * Historical usage: `<node> classify-lane <PBI-ID>`.
 *
 * Reads the PBI's current record from the fold — `bee backlog pbi list
 * --json` (event-sourced records in .bee/backlog.jsonl, never
 * docs/backlog.md, which is only the generated view) — never a markdown
 * table row.
 *
 * Emits exactly one JSON object on stdout:
 *   {pbi, lane, hard_gate_flags[], lane_safe, reason}
 *
 * This answers only "is this PBI's lane safe for an unattended agent" — one
 * input to D1's four-condition dispatchable set, never the whole of it.
 *
 * Fail-closed contract: anything the rules cannot classify with confidence
 * (no matching id, an unreachable fold, empty/unparseable title+cos text, or
 * a signal the rules do not cover) returns lane_safe:false with lane
 * "high-risk" and a reason naming why confidence failed. An unclassifiable
 * PBI must never come back safe.
 */

import { spawnSync } from 'node:child_process';

const STATUS_VALUES = new Set(['proposed', 'in-flight', 'parked', 'done', 'declined']);

// Mode-gate risk flags, from bee-planning SKILL.md's "Mode Gate" section:
//   auth · authorization · data model · audit/security · external systems ·
//   public contracts · cross-platform · changes behavior an existing test
//   asserts · the change requires weakening/deleting/replacing existing
//   proof · multi-domain
// The six hard-gate flags (decisive per D6 / SKILL.md "4+ flags or any
// hard-gate flag"): auth, authorization, data loss, audit/security,
// external provider, validation removal. "data loss" broadens "data model"
// and "validation removal" broadens "weakening/deleting/replacing existing
// proof" — same category, D6's wording for the decisive subset.
const FLAG_RULES = [
  {
    id: 'auth',
    label: 'auth',
    hardGate: true,
    pattern: /\bauthentication\b|\bauth\b(?!ori)|đăng nhập|xác thực/i,
  },
  {
    id: 'authorization',
    label: 'authorization',
    hardGate: true,
    pattern: /\bauthoriz(?:e|ation|ed|ing)\b|\bauthz\b|phân quyền|ủy quyền|uỷ quyền|quyền truy cập/i,
  },
  {
    id: 'data-loss',
    label: 'data model / data loss',
    hardGate: true,
    pattern: /\bdata loss\b|\bdata model\b|drop table|xóa dữ liệu|xoá dữ liệu|mất dữ liệu|schema (?:change|migration)|mô hình dữ liệu/i,
  },
  {
    id: 'audit-security',
    label: 'audit/security',
    hardGate: true,
    pattern: /\baudit\b|\bsecurity\b|bảo mật|an ninh|lỗ hổng|vulnerab/i,
  },
  {
    id: 'external-provider',
    label: 'external systems / external provider',
    hardGate: true,
    pattern: /external (?:provider|system|service|api)|third[- ]party|bên ngoài|nhà cung cấp|dịch vụ ngoài/i,
  },
  {
    id: 'public-contracts',
    label: 'public contracts',
    hardGate: false,
    pattern: /public contract|breaking change|api contract|hợp đồng công khai/i,
  },
  {
    id: 'cross-platform',
    label: 'cross-platform',
    hardGate: false,
    pattern: /cross-platform|đa nền tảng|windows[^\n]{0,40}(?:macos|linux)|macos[^\n]{0,40}(?:windows|linux)/i,
  },
  {
    id: 'test-behavior',
    label: 'changes behavior an existing test asserts',
    hardGate: false,
    pattern: /existing test|covered contract|kiểm thử hiện có|test hiện có/i,
  },
  {
    id: 'validation-removal',
    label: 'weakening/deleting/replacing existing proof (validation removal)',
    hardGate: true,
    pattern: /\bweaken(?:ing)?\b|remove validation|skip validation|bỏ qua kiểm tra|gỡ bỏ (?:kiểm tra|validation)|xoá test|xóa test|xoá proof|xóa proof/i,
  },
  {
    id: 'multi-domain',
    label: 'multi-domain',
    hardGate: false,
    pattern: /multi-domain|multiple domains|nhiều domain|đa lĩnh vực/i,
  },
];

function emit(result) {
  const { pbi, lane, hard_gate_flags, lane_safe, reason } = result;
  process.stdout.write(`${JSON.stringify({ pbi, lane, hard_gate_flags, lane_safe, reason })}\n`);
}

function unclassifiable(pbi, reason) {
  emit({ pbi, lane: 'high-risk', hard_gate_flags: [], lane_safe: false, reason });
}

function parseArgs(argv) {
  const args = argv.slice(2);
  let pbi = null;
  let beeCmd = '.bee/bin/bee';
  const positional = [];
  for (let i = 0; i < args.length; i += 1) {
    if (args[i] === '--bee-cmd') {
      beeCmd = args[i + 1];
      i += 1;
    } else {
      positional.push(args[i]);
    }
  }
  if (positional.length > 0) {
    [pbi] = positional;
  }
  return { pbi, beeCmd };
}

// Read the PBI's current record from the fold — `bee backlog pbi list
// --json` (event-sourced records in .bee/backlog.jsonl, last-event-wins per
// field) — never docs/backlog.md, which is only the generated view and can
// lag the fold between renders. Returns null when the fold has no matching
// id, throws when the command itself cannot be run or its output cannot be
// parsed as JSON (both are caller-classified fail-closed).
function findPbi(beeCmd, pbi) {
  const [cmd, ...cmdArgs] = beeCmd.split(/\s+/);
  const result = spawnSync(cmd, [...cmdArgs, 'backlog', 'pbi', 'list', '--json'], {
    encoding: 'utf8',
  });
  if (result.error) {
    throw new Error(`spawn "${beeCmd}" failed: ${result.error.message}`);
  }
  if (result.status !== 0) {
    throw new Error(`"${beeCmd} backlog pbi list --json" exited ${result.status}: ${(result.stderr || '').trim()}`);
  }
  let records;
  try {
    records = JSON.parse(result.stdout);
  } catch (err) {
    throw new Error(`could not parse "${beeCmd} backlog pbi list --json" output as JSON: ${err.message}`);
  }
  if (!Array.isArray(records)) {
    throw new Error('"backlog pbi list --json" did not return an array');
  }
  return records.find((r) => r && r.id === pbi) || null;
}

function classify(pbi, record) {
  const title = typeof record.title === 'string' ? record.title : '';
  const cos = typeof record.cos === 'string' ? record.cos : '';
  const text = [title, cos].filter((part) => part.length > 0).join(' ').trim();
  if (!text) {
    unclassifiable(pbi, `record for ${pbi} has empty or unparseable text (no title or cos found)`);
    return;
  }
  if (record.status !== undefined && record.status !== null && !STATUS_VALUES.has(record.status)) {
    unclassifiable(pbi, `record for ${pbi} has out-of-enum status "${record.status}"`);
    return;
  }

  const matched = FLAG_RULES.filter((rule) => rule.pattern.test(text));
  const hardGateMatches = matched.filter((rule) => rule.hardGate);

  if (hardGateMatches.length > 0) {
    const labels = hardGateMatches.map((rule) => rule.label);
    emit({
      pbi,
      lane: 'high-risk',
      hard_gate_flags: labels,
      lane_safe: false,
      reason: `hard-gate flag matched: ${labels.join(', ')}`,
    });
    return;
  }

  if (matched.length >= 4) {
    const labels = matched.map((rule) => rule.label);
    emit({
      pbi,
      lane: 'high-risk',
      hard_gate_flags: [],
      lane_safe: false,
      reason: `${matched.length} mode-gate risk flags matched (4+ classifies high-risk): ${labels.join(', ')}`,
    });
    return;
  }

  // 0-1 flags -> tiny/small; 2-3 -> standard (bee-planning Mode Gate). The
  // tiny/small split further depends on a product-file count this script
  // has no visibility into from backlog text alone, so 0-1 flags reports
  // the safer (larger) of the two, "small".
  const lane = matched.length >= 2 ? 'standard' : 'small';
  const reason = matched.length === 0
    ? 'no mode-gate risk flags matched in title/cos text'
    : `${matched.length} mode-gate risk flag(s) matched, below the high-risk threshold: ${matched.map((rule) => rule.label).join(', ')}`;
  emit({ pbi, lane, hard_gate_flags: [], lane_safe: true, reason });
}

function main() {
  const { pbi, beeCmd } = parseArgs(process.argv);
  if (!pbi) {
    unclassifiable('', 'no PBI id provided on the command line');
    return;
  }

  let record;
  try {
    record = findPbi(beeCmd, pbi);
  } catch (err) {
    unclassifiable(pbi, `cannot read PBI fold via "${beeCmd} backlog pbi list --json": ${err.message}`);
    return;
  }

  if (!record) {
    unclassifiable(pbi, `no matching PBI found for ${pbi} in the fold`);
    return;
  }

  classify(pbi, record);
}

main();
