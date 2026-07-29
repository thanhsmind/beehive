#!/usr/bin/env node
// Machine-check: every gate-presenting skill's OPERATIVE gate step must apply the
// LEVEL-AWARE gate-bypass rule (routing-and-contracts.md §Gate bypass), and no
// live gate surface may carry the stale `normal`-only phrasing that contradicts
// the `full`/`total` levels.
//
// Why this exists (crit-pattern 20260714 — "the invariant you leave in prose WILL
// be bypassed; mechanize it"): the level-aware rule was correct in the canonical
// contract, but bee-exploring (Gate 1) and bee-planning (Gate 2) never applied it
// and bee-validating (Gate 3) + go-mode carried stale "high-risk => bypass does
// not apply" text. A Codex runtime following the step literally therefore stopped
// at Gate 1/2 even under `total` autopilot. This test fails closed if a gate skill
// drops the carve-out or re-introduces the stale unconditional floor.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const REPO_ROOT = path.join(path.dirname(scriptPath), '..', '..');

// Each gate-presenting surface + the tokens proving it honors the level-aware
// rule. The OPERATIVE gate steps (exploring/planning/validating) must read the
// active level (`gate_bypass_level`) AND state the full/total floor-lift; go-mode
// is a REFERENCE that points to the contract, so it only owes the floor-lift
// statement, not the level-read call.
const GATE_SKILLS = [
  { file: 'skills/bee-exploring/SKILL.md', gate: 'Gate 1', tokens: ['gate_bypass_level', 'full'] },
  { file: 'skills/bee-planning/SKILL.md', gate: 'Gate 2', tokens: ['gate_bypass_level', 'full'] },
  { file: 'skills/bee-hive/references/go-mode.md', gate: 'go mode', tokens: ['full'] },
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
    fail(`${file} (${gate}): unreadable — a gate-presenting surface must exist`);
    continue;
  }
  let fileFailed = false;
  for (const token of tokens) {
    if (!text.includes(token)) {
      fail(`${file} (${gate}): missing required level-aware token "${token}" — the gate step must honor the full/total floor-lift`);
      fileFailed = true;
    }
  }
  for (const banned of BANNED_PHRASES) {
    if (text.includes(banned)) {
      fail(`${file} (${gate}): carries stale phrase "${banned}" — contradicts full/total (which lift the high-risk floor)`);
      fileFailed = true;
    }
  }
  if (!fileFailed) ok(`${file} (${gate}): level-aware bypass carve-out present, no stale floor phrasing`);
}

// Information-vs-approval refinement (decision a93994d3): bee-exploring's
// Socratic step must keep asking for genuine INFORMATION under full/total while
// suppressing mere APPROVALS. Assert the distinguishing litmus survives.
{
  const exploringAbs = path.join(REPO_ROOT, 'skills/bee-exploring/SKILL.md');
  let exploringText = '';
  try {
    exploringText = fs.readFileSync(exploringAbs, 'utf8');
  } catch {
    fail('skills/bee-exploring/SKILL.md: unreadable — the info-vs-approval refinement lives here');
  }
  if (!exploringText.includes('confident best answer')) {
    fail('skills/bee-exploring/SKILL.md: missing the info-vs-approval litmus ("confident best answer") — under full/total, approval questions are suppressed but genuine information questions must still be asked (decision a93994d3)');
  } else {
    ok('skills/bee-exploring/SKILL.md: info-vs-approval refinement present (asks for information, not approval, under bypass)');
  }
}

// Lane-ceremony-v3 doctrine (D1/D3/D4/D5/D8): bee-planning must carry the NEW
// plan-freeze + intake-first + lane-shape wording and must NOT carry the retired
// in-place plan enrichment. Prose-only invariants get bypassed unless mechanized
// (crit-pattern 20260714); these pins keep the rewrite from silently reverting to
// the shrunken-feature-plan model this feature removes.
{
  const planningAbs = path.join(REPO_ROOT, 'skills/bee-planning/SKILL.md');
  let planningText = '';
  try {
    planningText = fs.readFileSync(planningAbs, 'utf8');
  } catch {
    fail('skills/bee-planning/SKILL.md: unreadable — lane-ceremony-v3 planning doctrine lives here');
  }

  // (a) D1: the retired in-place enrichment instruction must be gone.
  if (planningText.includes('Enrich the **same**')) {
    fail('skills/bee-planning/SKILL.md (D1): still carries the retired in-place enrichment "Enrich the **same**" — plan.md is frozen at Gate 2, the enrichment step is removed');
  } else {
    ok('skills/bee-planning/SKILL.md (D1): retired in-place enrichment instruction absent');
  }

  // (b)-(f) Present-wording pins: each new lane invariant must be stated in text.
  const REQUIRED_PLANNING = [
    { token: 'frozen at Gate 2', d: 'D1', why: 'plan.md content is immutable once approved_gates.shape is set' },
    { token: 'approval stamp', d: 'D1', why: 'the only permitted post-approval plan.md write is an approval stamp' },
    { token: 'intake classification', d: 'D8', why: 'cheap intake classification runs before the lane-scaled bootstrap' },
    { token: 'request + one cell', d: 'D3', why: 'tiny lane shape = request + one cell, no plan.md' },
    { token: 'scoping synthesis', d: 'D4', why: 'small lane default = a logged scoping synthesis + 1-3 cells' },
    { token: 'plan.md is opt-in', d: 'D4', why: 'plan.md is opt-in for small, never written by default' },
    { token: 'never persist-then-preview', d: 'D5', why: 'draft cells are previewed before the merged gate; persisted only after approval' },
  ];
  for (const { token, d, why } of REQUIRED_PLANNING) {
    if (!planningText.includes(token)) {
      fail(`skills/bee-planning/SKILL.md (${d}): missing required lane-doctrine wording "${token}" — ${why}`);
    } else {
      ok(`skills/bee-planning/SKILL.md (${d}): "${token}" present`);
    }
  }
}

// Lane-ceremony-v3 doctrine (D3/D4/D5/D6/D7): the bee-hive surfaces (SKILL.md,
// go-mode.md, routing-and-contracts.md) must restate the SAME lane doctrine as
// the already-rewritten bee-planning/SKILL.md (lcv3-1) — never the old
// shrunken-feature-plan / unconditional-plan-caps wording. Consistency across
// restatements is the point of this cell.
{
  const hiveAbs = path.join(REPO_ROOT, 'skills/bee-hive/SKILL.md');
  const goModeAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/go-mode.md');
  const routingAbs = path.join(REPO_ROOT, 'skills/bee-hive/references/routing-and-contracts.md');
  let hiveText = '';
  let goModeText = '';
  let routingText = '';
  try {
    hiveText = fs.readFileSync(hiveAbs, 'utf8');
  } catch {
    fail('skills/bee-hive/SKILL.md: unreadable — lane-ceremony-v3 hive doctrine lives here');
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

  // (a) D7: old narrow-flag wordings must be gone from bee-hive/SKILL.md.
  const OLD_FLAG_PHRASES = ['existing covered behavior', 'weak proof around the area'];
  for (const phrase of OLD_FLAG_PHRASES) {
    if (hiveText.includes(phrase)) {
      fail(`skills/bee-hive/SKILL.md (D7): still carries the retired flag wording "${phrase}" — narrowed per D7`);
    } else {
      ok(`skills/bee-hive/SKILL.md (D7): retired flag wording "${phrase}" absent`);
    }
  }

  // (b) D7: new narrowed wordings present, matching bee-planning verbatim.
  const NEW_FLAG_TOKENS = [
    'changes behavior an existing test asserts',
    'weakening, deleting, or replacing existing proof',
  ];
  for (const token of NEW_FLAG_TOKENS) {
    if (!hiveText.includes(token)) {
      fail(`skills/bee-hive/SKILL.md (D7): missing narrowed flag wording "${token}"`);
    } else {
      ok(`skills/bee-hive/SKILL.md (D7): narrowed flag wording "${token}" present`);
    }
  }

  // (c) D6: product-files-only carve-out present.
  if (!hiveText.includes('product files only')) {
    fail('skills/bee-hive/SKILL.md (D6): missing "product files only" carve-out — lane caps must count product files only');
  } else {
    ok('skills/bee-hive/SKILL.md (D6): "product files only" carve-out present');
  }

  // (d) D3/D4: lane table states tiny has no plan.md, small's plan.md is opt-in.
  const LANE_TOKENS = [
    { token: 'cell is the micro-plan', d: 'D3' },
    { token: 'plan.md is opt-in', d: 'D4' },
    { token: 'logged scoping synthesis', d: 'D4' },
  ];
  for (const { token, d } of LANE_TOKENS) {
    if (!hiveText.includes(token)) {
      fail(`skills/bee-hive/SKILL.md (${d}): missing lane-table wording "${token}"`);
    } else {
      ok(`skills/bee-hive/SKILL.md (${d}): "${token}" present`);
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

// Lane-ceremony-v3 doctrine (D9), narrowed by validation-diet D1 (bee-validating
// is deleted outright, so its gate-in checks go with it): the remaining chain
// skills (bee-briefing, bee-swarming) must gate-in on the frozen plan.md +
// current-slice cells instead of the retired `artifact_readiness` field, and
// briefing's drift rule must fire on cell changes only, since D1 freezes
// plan.md content after Gate 2 (D9) — the plan itself can no longer drift.
{
  const briefingAbs = path.join(REPO_ROOT, 'skills/bee-briefing/SKILL.md');
  const swarmingAbs = path.join(REPO_ROOT, 'skills/bee-swarming/SKILL.md');
  let briefingText = '';
  let swarmingText = '';
  try {
    briefingText = fs.readFileSync(briefingAbs, 'utf8');
  } catch {
    fail('skills/bee-briefing/SKILL.md: unreadable — lane-ceremony-v3 briefing doctrine lives here');
  }
  try {
    swarmingText = fs.readFileSync(swarmingAbs, 'utf8');
  } catch {
    fail('skills/bee-swarming/SKILL.md: unreadable — lane-ceremony-v3 swarming doctrine lives here');
  }

  // (b) D9: briefing's drift rule fires on cell changes only — the plan can no
  // longer drift after Gate 2 approval (D1), so briefing's drift trigger narrows.
  if (!briefingText.includes('cell changes only')) {
    fail('skills/bee-briefing/SKILL.md (D9): drift rule missing "cell changes only" — since D1 freezes plan.md, drift now fires only when cells change');
  } else {
    ok('skills/bee-briefing/SKILL.md (D9): drift rule fires on "cell changes only"');
  }

  // (c) D3/D4: swarming's worker-prompt line covers the tiny/small no-plan case
  // — cite the cell as the work spec when the lane has no plan.md.
  if (!swarmingText.includes('cite the cell')) {
    fail('skills/bee-swarming/SKILL.md (D3/D4): worker-prompt line missing "cite the cell" — tiny/small lanes have no plan.md, so the prompt must cite the cell as the work spec');
  } else {
    ok('skills/bee-swarming/SKILL.md (D3/D4): worker-prompt line cites the cell for the no-plan case');
  }

  // (d) D2: next-slice completion wording names the next batch of cells, not a
  // plan-document slice — the current slice lives only in cells (D2).
  if (!swarmingText.includes('next batch of cells')) {
    fail("skills/bee-swarming/SKILL.md (D2): completion wording missing \"next batch of cells\" — the current slice is the feature's open cells, not a plan section");
  } else {
    ok('skills/bee-swarming/SKILL.md (D2): completion wording names the "next batch of cells"');
  }

  // AO14 single-worker + orchestrator-never-implements rules must survive.
  const AO14_TOKENS = ['one dispatched execution worker', 'never implement cells yourself'];
  for (const token of AO14_TOKENS) {
    if (!swarmingText.includes(token)) {
      fail(`skills/bee-swarming/SKILL.md (AO14): missing required survival wording "${token}"`);
    } else {
      ok(`skills/bee-swarming/SKILL.md (AO14): "${token}" present`);
    }
  }
}

// Lane-ceremony-v3 doctrine (D6/D7/D10): README.md and the AGENTS.block.md
// template are the last two surfaces that restate the lane/plan doctrine
// (D10 — "any README.md sections that restate the lane/plan doctrine ...
// updated in lockstep — shipping contradictory doctrine surfaces is out of
// the question"). Both must carry the SAME narrowed flag wording and lane
// shapes as the already-rewritten skills/bee-hive/SKILL.md (lcv3-2), never
// the retired unnarrowed flags or the unconditional "(small+)" briefing
// fan-out this feature removes.
{
  const readmeAbs = path.join(REPO_ROOT, 'README.md');
  const agentsBlockAbs = path.join(REPO_ROOT, 'packages/bee/AGENTS.block.md');
  let readmeText = '';
  let agentsBlockText = '';
  try {
    readmeText = fs.readFileSync(readmeAbs, 'utf8');
  } catch {
    fail('README.md: unreadable — lane-ceremony-v3 README doctrine lives here');
  }
  try {
    agentsBlockText = fs.readFileSync(agentsBlockAbs, 'utf8');
  } catch {
    fail('packages/bee/AGENTS.block.md: unreadable — lane-ceremony-v3 AGENTS block doctrine lives here');
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

  // (b) D7: new narrowed wordings present in README.md, verbatim as in
  // skills/bee-hive/SKILL.md.
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
  // already-rewritten skills/bee-hive/SKILL.md lane table verbatim.
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

  // (e) D9/D10 fan-out: bee-briefing is on-demand for standard, mandatory for
  // high-risk — the old unconditional "(small+)"/"(bigger work)" wording for
  // when a brief renders must be gone, replaced by the real fan-out, stated
  // identically in README.md and the AGENTS.block.md template.
  if (readmeText.includes('implement plan (bigger work)')) {
    fail('README.md (D9/D10): still carries the retired unconditional "(bigger work)" briefing fan-out wording');
  } else {
    ok('README.md (D9/D10): retired "(bigger work)" briefing fan-out wording absent');
  }
  if (agentsBlockText.includes('implement-plan.md (small+)')) {
    fail('packages/bee/AGENTS.block.md (D9/D10): chain line still carries the retired unconditional "(small+)" briefing fan-out wording');
  } else {
    ok('packages/bee/AGENTS.block.md (D9/D10): retired "(small+)" briefing fan-out wording absent');
  }
  for (const [label, text] of [['README.md', readmeText], ['packages/bee/AGENTS.block.md', agentsBlockText]]) {
    if (!text.includes('standard: on-demand') || !text.includes('high-risk: always')) {
      fail(`${label} (D9/D10): missing the real briefing fan-out wording ("standard: on-demand" / "high-risk: always")`);
    } else {
      ok(`${label} (D9/D10): real briefing fan-out wording present`);
    }
  }

  // (f) D1/D3/D4/D10: AGENTS.block.md's docs/history tree note must state
  // plan.md's freeze + per-lane conditionality, not the old unconditional
  // "always: CONTEXT.md, plan.md, reports/" line.
  if (agentsBlockText.includes('always: CONTEXT.md, plan.md, reports/')) {
    fail('packages/bee/AGENTS.block.md (D1/D3/D4): docs/history tree note still states plan.md as unconditionally always-present');
  } else {
    ok('packages/bee/AGENTS.block.md (D1/D3/D4): docs/history tree note no longer states plan.md as unconditional');
  }
  const TREE_TOKENS = ['frozen at Gate 2', 'plan.md is opt-in', 'cell is the micro-plan'];
  for (const token of TREE_TOKENS) {
    if (!agentsBlockText.includes(token)) {
      fail(`packages/bee/AGENTS.block.md: docs/history tree note missing "${token}"`);
    } else {
      ok(`packages/bee/AGENTS.block.md: docs/history tree note carries "${token}"`);
    }
  }

  // (g) D3/D4: AGENTS.block.md critical rule 8 ("Lanes scale ceremony, never
  // memory") must explicitly tie the tiny/small no-plan.md shapes to D3/D4,
  // so the scribing-capture obligation is legible even when a lane produced
  // no plan.md at all.
  if (!agentsBlockText.includes('whether or not the lane produced a `plan.md` (D3/D4)')) {
    fail('packages/bee/AGENTS.block.md: critical rule 8 missing the D3/D4 tie ("whether or not the lane produced a `plan.md` (D3/D4)")');
  } else {
    ok('packages/bee/AGENTS.block.md: critical rule 8 ties lanes-scale-ceremony to D3/D4');
  }
}

// Lane-plan-unconditional doctrine (lpu-1): the concurrency law's two
// restatements must state the lane decision as a step taken BEFORE EVERY
// feature start, never conditioned on another feature already being live —
// AGENTS.md rule 15's actual case (two independent ready features, nobody
// busy) never fired the old wording. Substance (disjoint-paths test, the
// --paths refusal, the worktree-only-when-needed carve-out) is unchanged;
// only the trigger moves from conditional to unconditional.
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
  // skills/bee-hive/SKILL.md's Routing list.
  if (hiveText.includes('busy + disjoint paths')) {
    fail('skills/bee-hive/SKILL.md (lpu-1): still carries the retired busy-precondition wording "busy + disjoint paths" — the lane decision fires before every feature start, not only once something is already busy');
  } else {
    ok('skills/bee-hive/SKILL.md (lpu-1): retired busy-precondition wording "busy + disjoint paths" absent');
  }

  // (b) lpu-1: the new unconditional trigger must be present in the Routing list.
  if (!hiveText.includes('before every feature start')) {
    fail('skills/bee-hive/SKILL.md (lpu-1): missing unconditional trigger "before every feature start" — the lane decision must be stated as a step taken before every feature start, not a reaction to detecting a busy session');
  } else {
    ok('skills/bee-hive/SKILL.md (lpu-1): unconditional trigger "before every feature start" present');
  }

  // (c) lpu-1: the retired live-feature precondition wording must be gone from
  // routing-and-contracts.md's LANES, FIRST-CLASS paragraph.
  if (routingText.includes('when new feature work is ready while another feature is live')) {
    fail('skills/bee-hive/references/routing-and-contracts.md (lpu-1): still carries the retired precondition "when new feature work is ready while another feature is live" — the lane decision must not require another feature to already be live');
  } else {
    ok('skills/bee-hive/references/routing-and-contracts.md (lpu-1): retired live-feature precondition wording absent');
  }

  // (d) lpu-1: the new unconditional trigger + explicit either-way statement
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
const sentinelMissingToken = !['gate_bypass_level', 'full'].every((t) => sentinelBad.includes(t));
const sentinelHasBanned = BANNED_PHRASES.some((b) => sentinelBad.includes(b));
if (!sentinelMissingToken || !sentinelHasBanned) {
  fail('sentinel: the checker does not bite a non-compliant gate surface (fail-open) — the guard is useless');
} else {
  ok('sentinel: a non-compliant gate surface is correctly flagged (checker bites)');
}

console.log(`\n${failed === 0 ? 'PASS' : 'FAIL'} - gate-bypass doctrine: ${failed} failure(s)`);
process.exit(failed > 0 ? 1 : 0);
