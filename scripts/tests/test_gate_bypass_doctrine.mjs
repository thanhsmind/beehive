#!/usr/bin/env node
// Machine-check: the LEVEL-AWARE gate-bypass rule (bee-hive "Gates" +
// routing-and-contracts.md §Gate bypass mode) must survive on the canonical
// bypass surfaces, and no live gate surface may carry the stale `normal`-only
// phrasing that contradicts the `full`/`total` levels.
//
// Why this exists (crit-pattern 20260714 — "the invariant you leave in prose WILL
// be bypassed; mechanize it"): the level-aware rule was correct in the canonical
// contract, but the operative gate steps once dropped it and carried stale
// "high-risk => bypass does not apply" text; a runtime following the step
// literally stopped at Gate 1/2 even under `total` autopilot. This test fails
// closed if a bypass surface drops the level carve-out or a gate surface
// re-introduces the stale unconditional floor.
//
// Post-refocus anchors (P4 consolidation, commit 93b95d2b): bee-exploring/
// bee-qualifying/bee-context-locking/bee-briefing → bee-shaping; bee-scribing/
// bee-compounding → bee-capturing; bee-bypass-gate → bee-hive "Gates";
// bee-executing → bee-swarming "Execute". The doctrine below is unchanged —
// only the surfaces that carry it moved.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(scriptPath), '..', '..');

// Each bypass/gate surface + the tokens proving it honors the level-aware rule.
// P4 consolidation (93b95d2b): the per-skill Gate 1/Gate 2 bypass carve-outs
// (formerly pinned in bee-exploring and bee-planning) are consolidated into
// bee-hive's Gates section and routing-and-contracts §Gate bypass mode — the
// level tokens are owed THERE now. The live gate-presenting skills
// (bee-shaping Gate 1, bee-planning Gate 2) no longer restate bypass, so they
// owe no level token — but the stale floor phrasing stays banned on them.
const GATE_SKILLS = [
  { file: 'skills/bee-hive/SKILL.md', gate: 'Gates', tokens: ['gate_bypass', 'full', 'total'] },
  { file: 'skills/bee-hive/references/routing-and-contracts.md', gate: 'Gate bypass mode', tokens: ['gate_bypass', 'full', 'total'] },
  { file: 'skills/bee-hive/references/go-mode.md', gate: 'go mode', tokens: ['full'] },
  { file: 'skills/bee-shaping/SKILL.md', gate: 'Gate 1', tokens: [] },
  { file: 'skills/bee-planning/SKILL.md', gate: 'Gate 2', tokens: [] },
];

// Phrases that assert an UNCONDITIONAL high-risk stop — true only under `normal`,
// false under `full`/`total`. Banned on every live gate surface.
const BANNED_PHRASES = [
  'safety floor is absolute',
  'bypass does not apply', // the stale normal-only exclusion
];

let failed = 0;
const fail = (msg) => {
  failed += 1;
  console.log(`FAIL  ${msg}`);
};
const ok = (msg) => console.log(`ok    ${msg}`);

for (const { file, gate, tokens } of GATE_SKILLS) {
  const abs = path.join(REPO_ROOT, file);
  let text;
  try {
    text = fs.readFileSync(abs, 'utf8');
  } catch {
    fail(`${file} (${gate}): unreadable — a gate/bypass surface must exist`);
    continue;
  }
  let fileFailed = false;
  for (const token of tokens) {
    if (!text.includes(token)) {
      fail(`${file} (${gate}): missing required level-aware token "${token}" — the bypass surface must state the full/total floor-lift`);
      fileFailed = true;
    }
  }
  for (const banned of BANNED_PHRASES) {
    if (text.includes(banned)) {
      fail(`${file} (${gate}): carries stale phrase "${banned}" — contradicts full/total (which lift the high-risk floor)`);
      fileFailed = true;
    }
  }
  if (!fileFailed) ok(`${file} (${gate}): level-aware bypass doctrine intact, no stale floor phrasing`);
}

// Information-vs-approval refinement (decision a93994d3): the Socratic step
// (bee-shaping, Explore) must keep asking for genuine INFORMATION under
// full/total while suppressing mere APPROVALS. The distinguishing litmus now
// lives in routing-and-contracts §Gate bypass mode (P4 consolidation 93b95d2b
// moved it out of the exploring skill body).
{
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  let routingText = '';
  try {
    routingText = fs.readFileSync(routingAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/routing-and-contracts.md: unreadable — the info-vs-approval refinement lives here');
  }
  if (!routingText.includes('confident best answer')) {
    fail('skills/bee-hive/references/routing-and-contracts.md: missing the info-vs-approval litmus ("confident best answer") — under full/total, approval questions are suppressed but genuine information questions must still be asked (decision a93994d3)');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md: info-vs-approval refinement present (asks for information, not approval, under bypass)');
  }
}

// Lane-ceremony-v3 doctrine (D1/D3/D4/D5/D8): bee-planning must carry the
// plan-freeze + intake-first + lane-shape doctrine and must NOT carry the
// retired in-place plan enrichment. Prose-only invariants get bypassed unless
// mechanized (crit-pattern 20260714); these pins keep the rewrite from silently
// reverting to the shrunken-feature-plan model. The wording moved in the
// refocus rewrite — each pin now targets the surface's current phrasing.
{
  const planningAbs = path.join(REPO_ROOT, 'skills/bee-planning/SKILL.md');
  const planningRefAbs = path.join(REPO_ROOT, 'skills/bee-planning/references/planning-reference.md');
  let planningText = '';
  let planningRefText = '';
  try {
    planningText = fs.readFileSync(planningAbs, 'utf8');
  } catch {
    fail('skills/bee-planning/SKILL.md: unreadable — lane-ceremony-v3 planning doctrine lives here');
  }
  try {
    planningRefText = fs.readFileSync(planningRefAbs, 'utf8');
  } catch {
    fail('skills/bee-planning/references/planning-reference.md: unreadable — the plan-freeze/approval-stamp detail lives here');
  }

  // (a) D1: the retired in-place enrichment instruction must be gone.
  if (planningText.includes('Enrich the **same**')) {
    fail('skills/bee-planning/SKILL.md (D1): still carries the retired in-place enrichment "Enrich the **same**" — plan.md is frozen at Gate 2, the enrichment step is removed');
  } else {
    ok('skills/bee-planning/SKILL.md (D1): retired in-place enrichment instruction absent');
  }

  // (b)-(f) Present-wording pins: each lane invariant, at its current phrasing.
  // Prose wraps mid-sentence (including inside blockquotes), so match against
  // whitespace-normalized text with blockquote markers stripped.
  const planningFlat = planningText.replace(/^>\s?/gm, '').replace(/\s+/g, ' ');
  const REQUIRED_PLANNING = [
    { token: 'a stamp may follow, a content edit may not', d: 'D1', why: 'plan.md content is immutable once approved_gates.shape is set (formerly "frozen at Gate 2")' },
    { token: 'Classify before reading deeply', d: 'D8', why: 'cheap intake classification runs before any deep reading (formerly "intake classification")' },
    { token: 'one cell — the cell is the micro-plan', d: 'D3', why: 'tiny lane shape = one cell, no plan.md (formerly "request + one cell")' },
    { token: '`plan.md` only on request', d: 'D4', why: 'plan.md is opt-in for small, never written by default (formerly "plan.md is opt-in")' },
    { token: 'scoping synthesis', d: 'D4', why: 'small lane default = a logged scoping synthesis + 1-3 cells' },
    { token: 'never persist-then-preview', d: 'D5', why: 'draft cells are previewed before the merged gate; persisted only after approval' },
  ];
  for (const { token, d, why } of REQUIRED_PLANNING) {
    if (!planningFlat.includes(token)) {
      fail(`skills/bee-planning/SKILL.md (${d}): missing required lane-doctrine wording "${token}" — ${why}`);
    } else {
      ok(`skills/bee-planning/SKILL.md (${d}): "${token}" present`);
    }
  }

  // D1: the approval-stamp rule (the only permitted post-approval plan.md
  // write) moved into the planning reference in the refocus rewrite.
  if (!planningRefText.includes('approval stamp')) {
    fail('skills/bee-planning/references/planning-reference.md (D1): missing "approval stamp" — the only permitted post-approval plan.md write is an approval stamp');
  } else {
    ok('skills/bee-planning/references/planning-reference.md (D1): "approval stamp" present');
  }
}

// Lane-ceremony-v3 doctrine (D3/D4/D5/D6/D7): the surfaces restating the lane
// doctrine must agree — never the old shrunken-feature-plan /
// unconditional-plan-caps wording. P4 consolidation (93b95d2b): the risk-flag
// list and file-cap rule live in bee-planning's Route section now; the lane
// ceremony table lives in routing-and-contracts §Lane ceremony in full (the
// slim bee-hive router carries neither).
{
  const planningAbs = path.join(REPO_ROOT, 'skills/bee-planning/SKILL.md');
  const hiveAbs = path.join(REPO_ROOT, 'skills/bee-hive/SKILL.md');
  const goModeAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/go-mode.md');
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  let planningText = '';
  let hiveText = '';
  let goModeText = '';
  let routingText = '';
  try {
    planningText = fs.readFileSync(planningAbs, 'utf8');
  } catch {
    fail('skills/bee-planning/SKILL.md: unreadable — the risk-flag list lives here');
  }
  try {
    hiveText = fs.readFileSync(hiveAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/SKILL.md: unreadable — the router must exist');
  }
  try {
    goModeText = fs.readFileSync(goModeAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/go-mode.md: unreadable — lane-ceremony-v3 go-mode doctrine lives here');
  }
  try {
    routingText = fs.readFileSync(routingAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/routing-and-contracts.md: unreadable — lane-ceremony-v3 routing doctrine lives here');
  }

  // (a) D7: old narrow-flag wordings must be gone from every surface that
  // states flags today (router, planning, routing reference).
  const OLD_FLAG_PHRASES = ['existing covered behavior', 'weak proof around the area'];
  for (const [label, text] of [
    ['skills/bee-hive/SKILL.md', hiveText],
    ['skills/bee-planning/SKILL.md', planningText],
    ['skills/bee-hive/references/routing-and-contracts.md', routingText],
  ]) {
    for (const phrase of OLD_FLAG_PHRASES) {
      if (text.includes(phrase)) {
        fail(`${label} (D7): still carries the retired flag wording "${phrase}" — narrowed per D7`);
      } else {
        ok(`${label} (D7): retired flag wording "${phrase}" absent`);
      }
    }
  }

  // (b) D7: narrowed flag wordings present on the flag list's current home,
  // bee-planning's Route section (the blockquoted list wraps across lines —
  // strip blockquote markers, then normalize whitespace).
  const planningFlat = planningText.replace(/^>\s?/gm, '').replace(/\s+/g, ' ');
  const NEW_FLAG_TOKENS = [
    'changes behavior an existing test asserts',
    'weakening, deleting, or replacing existing proof',
  ];
  for (const token of NEW_FLAG_TOKENS) {
    if (!planningFlat.includes(token)) {
      fail(`skills/bee-planning/SKILL.md (D7): missing narrowed flag wording "${token}"`);
    } else {
      ok(`skills/bee-planning/SKILL.md (D7): narrowed flag wording "${token}" present`);
    }
  }

  // (c) D6: product-files-only carve-out present beside the file caps
  // (moved to bee-planning's Route section).
  if (!planningText.includes('product files only')) {
    fail('skills/bee-planning/SKILL.md (D6): missing "product files only" carve-out — lane caps must count product files only');
  } else {
    ok('skills/bee-planning/SKILL.md (D6): "product files only" carve-out present');
  }

  // (d) D3/D4: the lane ceremony table (now in routing-and-contracts) states
  // tiny has no plan.md, small's plan.md is opt-in.
  const LANE_TOKENS = [
    { token: 'cell is the micro-plan', d: 'D3' },
    { token: 'plan.md is opt-in', d: 'D4' },
    { token: 'logged scoping synthesis', d: 'D4' },
  ];
  for (const { token, d } of LANE_TOKENS) {
    if (!routingText.includes(token)) {
      fail(`skills/bee-hive/references/routing-and-contracts.md (${d}): missing lane-table wording "${token}"`);
    } else {
      ok(`skills/bee-hive/references/routing-and-contracts.md (${d}): "${token}" present`);
    }
  }

  // (e) D5: go-mode's fast-path line describes preview-then-merged-gate, and
  // AO14's dispatched execution worker — not plan.md-first / solo-in-session.
  if (!goModeText.includes('previewed before persist')) {
    fail('skills/bee-hive/references/go-mode.md (D5): fast-path line missing "previewed before persist"');
  } else {
    ok('skills/bee-hive/references/go-mode.md (D5): fast-path line describes preview-before-persist');
  }
  if (goModeText.includes('solo in-session execution')) {
    fail('skills/bee-hive/references/go-mode.md (AO14): still carries retired "solo in-session execution" wording — tiny/small execute via one dispatched execution worker');
  } else {
    ok('skills/bee-hive/references/go-mode.md (AO14): retired "solo in-session execution" wording absent');
  }
  if (!goModeText.includes('one dispatched execution worker')) {
    fail('skills/bee-hive/references/go-mode.md (AO14): fast-path line missing "one dispatched execution worker"');
  } else {
    ok('skills/bee-hive/references/go-mode.md (AO14): "one dispatched execution worker" present');
  }

  // (f) D1: STEP 2/3 and the Gate 2 revise line no longer restate the retired
  // requirements-only -> implementation-ready mutation.
  if (goModeText.includes('plan.md enriched to implementation-ready')) {
    fail('skills/bee-hive/references/go-mode.md (D1): STEP 3 still carries the retired "plan.md enriched to implementation-ready" wording');
  } else {
    ok('skills/bee-hive/references/go-mode.md (D1): retired "plan.md enriched to implementation-ready" wording absent');
  }
  if (goModeText.includes('still `requirements-only`')) {
    fail('skills/bee-hive/references/go-mode.md (D1): Gate 2 revise line still carries the retired "still requirements-only" wording');
  } else {
    ok('skills/bee-hive/references/go-mode.md (D1): retired "still requirements-only" wording absent');
  }

  // (g) D1/D2: routing-and-contracts.md Chaining Contract / working-files tree
  // no longer state plan.md as unconditional or requirements-only -> implementation-ready.
  if (routingText.includes('requirements-only → implementation-ready')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (D1): Chaining Contract still carries the retired "requirements-only → implementation-ready" arrow');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (D1): retired "requirements-only → implementation-ready" arrow absent');
  }
  if (!routingText.includes('frozen at Gate 2')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (D1): missing "frozen at Gate 2" wording');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (D1): "frozen at Gate 2" wording present');
  }
}

// Lane-ceremony-v3 doctrine (D9), narrowed by validation-diet D1: the chain
// surfaces after the P4 consolidation (93b95d2b) — briefing lives in
// bee-shaping ("Brief"), the worker contract in bee-swarming ("Execute") and
// its reference — must gate-in on the frozen plan.md + current-slice cells,
// and the brief's drift rule (now stated in routing-and-contracts' Chaining
// Contract) must fire on cell changes only, since D1 freezes plan.md content
// after Gate 2 (D9) — the plan itself can no longer drift.
{
  const shapingAbs = path.join(REPO_ROOT, 'skills/bee-shaping/SKILL.md');
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  const swarmingAbs = path.join(REPO_ROOT, 'skills/bee-swarming/SKILL.md');
  const swarmingRefAbs = path.join(REPO_ROOT, 'skills/bee-swarming/references/swarming-reference.md');
  let shapingText = '';
  let routingText = '';
  let swarmingText = '';
  let swarmingRefText = '';
  try {
    shapingText = fs.readFileSync(shapingAbs, 'utf8');
  } catch {
    fail('skills/bee-shaping/SKILL.md: unreadable — the Brief move (successor of bee-briefing) lives here');
  }
  try {
    routingText = fs.readFileSync(routingAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/routing-and-contracts.md: unreadable — the brief drift rule lives in its Chaining Contract');
  }
  try {
    swarmingText = fs.readFileSync(swarmingAbs, 'utf8');
  } catch {
    fail('skills/bee-swarming/SKILL.md: unreadable — lane-ceremony-v3 swarming doctrine lives here');
  }
  try {
    swarmingRefText = fs.readFileSync(swarmingRefAbs, 'utf8');
  } catch {
    fail('skills/bee-swarming/references/swarming-reference.md: unreadable — the single-execution-worker contract lives here');
  }

  // (a) The Brief surface itself must exist inside bee-shaping (93b95d2b).
  if (!shapingText.includes('## Brief')) {
    fail('skills/bee-shaping/SKILL.md: missing the "## Brief" section — the implement-plan render (successor of bee-briefing) must live here');
  } else {
    ok('skills/bee-shaping/SKILL.md: "## Brief" section present (bee-briefing successor)');
  }

  // (b) D9: the brief's drift rule fires on cell changes only — the plan can no
  // longer drift after Gate 2 approval (D1). Stated in the Chaining Contract's
  // shaping (Brief) row.
  if (!routingText.includes('cell changes only')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (D9): brief drift rule missing "cell changes only" — since D1 freezes plan.md, drift now fires only when cells change');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (D9): brief drift rule fires on "cell changes only"');
  }

  // (c) D3/D4: the worker-prompt contract covers the tiny/small no-plan case
  // — cite the cell as the work spec when the lane has no plan.md (moved into
  // the swarming reference's spawn/isolation contract).
  if (!swarmingRefText.includes('cite the cell')) {
    fail('skills/bee-swarming/references/swarming-reference.md (D3/D4): worker-prompt contract missing "cite the cell" — tiny/small lanes have no plan.md, so the prompt must cite the cell as the work spec');
  } else {
    ok('skills/bee-swarming/references/swarming-reference.md (D3/D4): worker-prompt contract cites the cell for the no-plan case');
  }

  // (d) D2: next-slice completion wording names the next batch of cells, not a
  // plan-document slice — planning shapes the next batch, never reopens the
  // frozen plan (formerly "next batch of cells").
  if (!swarmingText.includes('shapes the next batch, never reopens it')) {
    fail("skills/bee-swarming/SKILL.md (D2): completion wording missing \"shapes the next batch, never reopens it\" — the current slice is the feature's open cells, not a plan section");
  } else {
    ok('skills/bee-swarming/SKILL.md (D2): completion wording names the next batch of cells, plan stays frozen');
  }

  // AO14 single-worker + orchestrator-never-implements rules must survive:
  // the single-worker contract in the swarming reference, the
  // orchestrator-never-implements rule in the skill body (now phrased
  // "you do not implement").
  if (!swarmingRefText.includes('one dispatched execution worker')) {
    fail('skills/bee-swarming/references/swarming-reference.md (AO14): missing required survival wording "one dispatched execution worker"');
  } else {
    ok('skills/bee-swarming/references/swarming-reference.md (AO14): "one dispatched execution worker" present');
  }
  if (!swarmingText.includes('you do not implement')) {
    fail('skills/bee-swarming/SKILL.md (AO14): missing required survival wording "you do not implement" — the orchestrator never implements cells itself');
  } else {
    ok('skills/bee-swarming/SKILL.md (AO14): orchestrator-never-implements wording present');
  }
}

// Lane-ceremony-v3 doctrine (D6/D7/D10): README.md still restates the
// lane/plan doctrine and must match the canonical surfaces (D10 — "shipping
// contradictory doctrine surfaces is out of the question"). The AGENTS block
// (packages/bee/AGENTS.block.md) is a prose doctrine-section block after the
// P4 consolidation (93b95d2b): it no longer restates lane tables, the
// docs/history tree, or the briefing fan-out — those pins move to the surfaces
// that carry them now (README.md, routing-and-contracts.md), while the block's
// own surviving doctrine (lanes scale ceremony, never memory) stays pinned.
{
  const readmeAbs = path.join(REPO_ROOT, 'README.md');
  const agentsBlockAbs = path.join(REPO_ROOT, 'packages/bee/AGENTS.block.md');
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  let readmeText = '';
  let agentsBlockText = '';
  let routingText = '';
  try {
    readmeText = fs.readFileSync(readmeAbs, 'utf8');
  } catch {
    fail('README.md: unreadable — lane-ceremony-v3 README doctrine lives here');
  }
  try {
    agentsBlockText = fs.readFileSync(agentsBlockAbs, 'utf8');
  } catch {
    fail('packages/bee/AGENTS.block.md: unreadable — the agents-facing doctrine block lives here');
  }
  try {
    routingText = fs.readFileSync(routingAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/routing-and-contracts.md: unreadable — the canonical lane/plan doctrine lives here');
  }

  // (a) D7: old narrow-flag wordings must be gone from README.md.
  const OLD_FLAG_PHRASES_README = ['existing covered behavior', 'weak proof'];
  for (const phrase of OLD_FLAG_PHRASES_README) {
    if (readmeText.includes(phrase)) {
      fail(`README.md (D7): still carries the retired flag wording "${phrase}" — narrowed per D7`);
    } else {
      ok(`README.md (D7): retired flag wording "${phrase}" absent`);
    }
  }

  // (b) D7: new narrowed wordings present in README.md, verbatim as on the
  // canonical flag surface (bee-planning's Route section).
  const NEW_FLAG_TOKENS_README = [
    'changes behavior an existing test asserts',
    'weakening, deleting, or replacing existing proof',
  ];
  for (const token of NEW_FLAG_TOKENS_README) {
    if (!readmeText.includes(token)) {
      fail(`README.md (D7): missing narrowed flag wording "${token}"`);
    } else {
      ok(`README.md (D7): narrowed flag wording "${token}" present`);
    }
  }

  // (c) D6: product-files-only carve-out present in README.md's lane section.
  if (!readmeText.includes('product files only')) {
    fail('README.md (D6): missing "product files only" carve-out — lane caps must count product files only');
  } else {
    ok('README.md (D6): "product files only" carve-out present');
  }

  // (d) D3/D4: README's lane table states tiny has no plan.md (D3, "the cell
  // is the micro-plan"), small's plan.md is opt-in (D4) — matching the
  // canonical lane table in routing-and-contracts.md verbatim.
  const LANE_TOKENS_README = [
    { token: 'cell is the micro-plan', d: 'D3' },
    { token: 'plan.md is opt-in', d: 'D4' },
  ];
  for (const { token, d } of LANE_TOKENS_README) {
    if (!readmeText.includes(token)) {
      fail(`README.md (${d}): missing lane-table wording "${token}"`);
    } else {
      ok(`README.md (${d}): "${token}" present`);
    }
  }

  // (e) D9/D10 fan-out: bee-shaping's Brief is on-demand for standard,
  // mandatory for high-risk — the old unconditional "(small+)"/"(bigger
  // work)" wording must stay gone. The two live restatements are README.md
  // and routing-and-contracts' Chaining Contract (the AGENTS block dropped
  // its chain line in the 93b95d2b prose rewrite).
  if (readmeText.includes('implement plan (bigger work)')) {
    fail('README.md (D9/D10): still carries the retired unconditional "(bigger work)" briefing fan-out wording');
  } else {
    ok('README.md (D9/D10): retired "(bigger work)" briefing fan-out wording absent');
  }
  if (agentsBlockText.includes('implement-plan.md (small+)')) {
    fail('packages/bee/AGENTS.block.md (D9/D10): still carries the retired unconditional "(small+)" briefing fan-out wording');
  } else {
    ok('packages/bee/AGENTS.block.md (D9/D10): retired "(small+)" briefing fan-out wording absent');
  }
  if (!readmeText.includes('standard: on-demand') || !readmeText.includes('high-risk: always')) {
    fail('README.md (D9/D10): missing the real briefing fan-out wording ("standard: on-demand" / "high-risk: always")');
  } else {
    ok('README.md (D9/D10): real briefing fan-out wording present');
  }
  if (!routingText.includes('high-risk` always, `standard` on-demand')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (D9/D10): Chaining Contract missing the real briefing fan-out wording ("`high-risk` always, `standard` on-demand")');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (D9/D10): real briefing fan-out wording present in the Chaining Contract');
  }

  // (f) D1/D3/D4/D10: the docs/history tree note must state plan.md's freeze +
  // per-lane conditionality, never the old unconditional "always: CONTEXT.md,
  // plan.md, reports/" line. The tree moved from the AGENTS block to
  // routing-and-contracts' File Quick Reference in the 93b95d2b prose rewrite.
  if (agentsBlockText.includes('always: CONTEXT.md, plan.md, reports/')) {
    fail('packages/bee/AGENTS.block.md (D1/D3/D4): still states plan.md as unconditionally always-present');
  } else {
    ok('packages/bee/AGENTS.block.md (D1/D3/D4): no unconditional plan.md tree note');
  }
  const TREE_TOKENS = [
    { token: 'frozen at Gate 2', why: 'plan.md freezes at Gate 2 (D1)' },
    { token: 'small opt-in', why: 'plan.md is opt-in for small (D4)' },
    { token: 'tiny/spike none', why: 'tiny/spike lanes have no plan.md (D3)' },
  ];
  for (const { token, why } of TREE_TOKENS) {
    if (!routingText.includes(token)) {
      fail(`skills/bee-hive/references/routing-and-contracts.md: File Quick Reference tree note missing "${token}" — ${why}`);
    } else {
      ok(`skills/bee-hive/references/routing-and-contracts.md: File Quick Reference tree note carries "${token}"`);
    }
  }

  // (g) D3/D4: the "Lanes scale ceremony, never memory" rule (formerly AGENTS
  // block critical rule 8) survives in the prose block, and the capture
  // obligation stays legible even when a lane produced no plan.md at all —
  // routing-and-contracts' Capture discipline states it as "tiny lanes
  // included" (the explicit "(D3/D4)" parenthetical retired in 93b95d2b).
  if (!agentsBlockText.includes('Lanes scale ceremony, never memory')) {
    fail('packages/bee/AGENTS.block.md: missing "Lanes scale ceremony, never memory" — the lanes-scale-ceremony rule must survive the prose rewrite');
  } else {
    ok('packages/bee/AGENTS.block.md: "Lanes scale ceremony, never memory" present');
  }
  if (!routingText.includes('tiny lanes included')) {
    fail('skills/bee-hive/references/routing-and-contracts.md: Capture discipline missing "tiny lanes included" — the capture obligation holds whether or not the lane produced a plan.md');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md: Capture discipline covers no-plan lanes ("tiny lanes included")');
  }
}

// Lane-plan-unconditional doctrine (lpu-1): the concurrency law must state the
// lane decision as a step taken BEFORE EVERY feature start, never conditioned
// on another feature already being live. Substance (disjoint-paths test, the
// --paths refusal, the worktree-only-when-needed carve-out) is unchanged; only
// the trigger moved from conditional to unconditional. After the P4
// consolidation (93b95d2b) the law has ONE canonical home —
// routing-and-contracts' "LANES, FIRST-CLASS" paragraph; the slim bee-hive
// router no longer restates it, so the router owes only the absence of the
// retired busy-precondition wording.
// (Removed per 93b95d2b: the second-restatement presence check on
// skills/bee-hive/SKILL.md's Routing list — that restatement was consolidated
// into routing-and-contracts, whose checks below carry the doctrine.)
{
  const hiveAbs = path.join(REPO_ROOT, 'skills/bee-hive/SKILL.md');
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  let hiveText = '';
  let routingText = '';
  try {
    hiveText = fs.readFileSync(hiveAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/SKILL.md: unreadable — lane-plan-unconditional doctrine lives here');
  }
  try {
    routingText = fs.readFileSync(routingAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/references/routing-and-contracts.md: unreadable — lane-plan-unconditional doctrine lives here');
  }

  // (a) lpu-1: the retired busy-precondition wording must be gone from
  // skills/bee-hive/SKILL.md.
  if (hiveText.includes('busy + disjoint paths')) {
    fail('skills/bee-hive/SKILL.md (lpu-1): still carries the retired busy-precondition wording "busy + disjoint paths" — the lane decision fires before every feature start, not only once something is already busy');
  } else {
    ok('skills/bee-hive/SKILL.md (lpu-1): retired busy-precondition wording "busy + disjoint paths" absent');
  }

  // (b) lpu-1: the retired live-feature precondition wording must be gone from
  // routing-and-contracts.md's LANES, FIRST-CLASS paragraph.
  if (routingText.includes('when new feature work is ready while another feature is live')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (lpu-1): still carries the retired precondition "when new feature work is ready while another feature is live" — the lane decision must not require another feature to already be live');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (lpu-1): retired live-feature precondition wording absent');
  }

  // (c) lpu-1: the unconditional trigger + explicit either-way statement
  // must be present in the LANES, FIRST-CLASS paragraph.
  const LPU_ROUTING_TOKENS = ['before every feature start', 'whether or not another feature is already live'];
  for (const token of LPU_ROUTING_TOKENS) {
    if (!routingText.includes(token)) {
      fail(`skills/bee-hive/references/routing-and-contracts.md (lpu-1): missing unconditional wording "${token}"`);
    } else {
      ok(`skills/bee-hive/references/routing-and-contracts.md (lpu-1): unconditional wording "${token}" present`);
    }
  }
}

// Sentinel: prove the checker bites. A synthetic gate surface missing the tokens
// (and carrying a banned phrase) MUST be flagged by the same predicates.
const sentinelBad = 'Present Gate X, then verbatim ask. The safety floor is absolute.';
const sentinelMissingToken = !['gate_bypass', 'full', 'total'].every((t) => sentinelBad.includes(t));
const sentinelHasBanned = BANNED_PHRASES.some((b) => sentinelBad.includes(b));
if (!sentinelMissingToken || !sentinelHasBanned) {
  fail('sentinel: the checker does not bite a non-compliant gate surface (fail-open) — the guard is useless');
} else {
  ok('sentinel: a non-compliant gate surface is correctly flagged (checker bites)');
}

console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} - gate-bypass doctrine: ${failed} failure(s)`);
process.exit(failed > 0 ? 1 : 0);
