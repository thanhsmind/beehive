#!/usr/bin/env node
// Proves validateModelsConfig (ao-2ai-1) loudly flags malformed/prompt-less/
// unsafe cli-tier models config where today normalizeTierValue silently
// reverts to the seeded default (a malformed cli value -> normalizeTierValue
// returns undefined -> normalizeModels never overwrites -> the seeded
// 'sonnet' stays, no error). Plain node asserts, no framework, matching the
// style of the other scripts/tests/test_*.mjs checks.

import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {
  validateModelsConfig,
  validateAgentFilesDrift,
  UNSAFE_CLI_FLAGS,
  ADVICE_CLASS_WRITABLE_TOKENS,
} from '../../.bee/bin/lib/state.mjs';

const results = [];
function record(desc, passed, detail) {
  results.push({ desc, passed });
  console.log((passed ? 'PASS ' : 'FAIL ') + desc + (passed ? '' : ` -- ${detail}`));
}

function hasCode(problems, code) {
  return problems.some((p) => p.code === code);
}

// ── valid configs pass clean ────────────────────────────────────────────────

{
  const problems = validateModelsConfig({
    models: { claude: { extraction: 'haiku', generation: 'sonnet', review: 'opus' } },
  });
  record('valid plain model-name tier passes (no problems)', problems.length === 0, JSON.stringify(problems));
}

{
  const problems = validateModelsConfig({
    models: { claude: { generation: { model: 'sonnet', effort: 'medium' } } },
  });
  record('valid {model, effort} tier shape passes (no problems)', problems.length === 0, JSON.stringify(problems));
}

{
  const problems = validateModelsConfig({
    models: {
      claude: {
        generation: { kind: 'cli', command: 'codex exec --json -m gpt-5.3-codex -s read-only', promptVia: 'stdin' },
      },
    },
  });
  record('valid cli tier (declared transport, safe command) passes', problems.length === 0, JSON.stringify(problems));
}

{
  // no models key at all, or an empty object — nothing configured, not an error.
  const p1 = validateModelsConfig({});
  const p2 = validateModelsConfig({ hooks: {} });
  record('config with no `models` key at all produces no problems', p1.length === 0 && p2.length === 0, JSON.stringify([p1, p2]));
}

// ── malformed cli (missing kind:'cli' or non-empty command) ────────────────

{
  const problems = validateModelsConfig({
    models: { claude: { generation: { command: 'codex exec' } } }, // missing kind:'cli'
  });
  record(
    'cli-shaped value missing kind:"cli" is flagged cli-malformed',
    hasCode(problems, 'cli-malformed'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: { claude: { generation: { kind: 'cli', command: '' } } }, // empty command
  });
  record(
    'cli value with an empty command string is flagged cli-malformed',
    hasCode(problems, 'cli-malformed'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: { claude: { generation: { kind: 'cli' } } }, // missing command entirely
  });
  record(
    'cli value missing command entirely is flagged cli-malformed',
    hasCode(problems, 'cli-malformed'),
    JSON.stringify(problems),
  );
}

// ── prompt-less cli (no declared prompt transport) ──────────────────────────

{
  const problems = validateModelsConfig({
    models: {
      claude: {
        // Trailing "-" is a shell stdin convention, NOT a declared transport —
        // the validator must never sniff it; promptVia is absent here.
        generation: { kind: 'cli', command: 'codex exec -m gpt-5 -s read-only -' },
      },
    },
  });
  record(
    'cli value with no promptVia is flagged cli-prompt-transport-missing (never sniffed from a trailing "-")',
    hasCode(problems, 'cli-prompt-transport-missing') && !hasCode(problems, 'cli-malformed'),
    JSON.stringify(problems),
  );
}

// ── each unsafe alias is flagged individually ───────────────────────────────

for (const flag of UNSAFE_CLI_FLAGS) {
  const problems = validateModelsConfig({
    models: {
      claude: {
        generation: { kind: 'cli', command: `some-cli exec ${flag} --other-flag`, promptVia: 'stdin' },
      },
    },
  });
  record(
    `unsafe alias "${flag}" is flagged cli-unsafe-flag`,
    problems.some((p) => p.code === 'cli-unsafe-flag' && p.flag === flag),
    JSON.stringify(problems),
  );
}

{
  // A command carrying more than one known-bad alias reports each of them.
  const twoFlags = [UNSAFE_CLI_FLAGS[0], UNSAFE_CLI_FLAGS[1]];
  const problems = validateModelsConfig({
    models: {
      codex: {
        review: { kind: 'cli', command: `some-cli exec ${twoFlags[0]} ${twoFlags[1]}`, promptVia: 'stdin' },
      },
    },
  });
  const unsafe = problems.filter((p) => p.code === 'cli-unsafe-flag');
  record(
    'a command with two known-bad aliases reports both, one row each',
    unsafe.length === 2 && twoFlags.every((f) => unsafe.some((p) => p.flag === f)),
    JSON.stringify(problems),
  );
}

// ── advice-class (advisor/review) write-granting sandbox tokens (ao-2b-2/AO8) ──
// A second, narrower blocklist layered on top of UNSAFE_CLI_FLAGS above:
// advisor/review must run read-only; generation/extraction are untouched.

for (const token of ADVICE_CLASS_WRITABLE_TOKENS) {
  const problems = validateModelsConfig({
    models: {
      claude: {
        advisor: { kind: 'cli', command: `codex exec -m gpt-5 ${token} -`, promptVia: 'stdin' },
      },
    },
  });
  record(
    `advice-class token "${token}" on advisor is flagged cli-advice-slot-writable`,
    problems.some((p) => p.code === 'cli-advice-slot-writable' && p.flag === token),
    JSON.stringify(problems),
  );
}

{
  // review slot (codex runtime), bare "-s workspace-write" form.
  const problems = validateModelsConfig({
    models: {
      codex: {
        review: { kind: 'cli', command: 'codex exec -m gpt-5.5 -s workspace-write -', promptVia: 'stdin' },
      },
    },
  });
  record(
    'advice-class token "-s workspace-write" on review is flagged cli-advice-slot-writable',
    problems.some((p) => p.code === 'cli-advice-slot-writable' && p.flag === '-s workspace-write'),
    JSON.stringify(problems),
  );
}

{
  // generation is NOT advice-class — the unique discriminator (W3): the same
  // token untouched by the new code, since workspace-write is not on
  // UNSAFE_CLI_FLAGS either, so a clean pass here proves the slot-scoping,
  // not just the absence of one code among several already firing.
  const problems = validateModelsConfig({
    models: {
      claude: {
        generation: { kind: 'cli', command: 'codex exec -m gpt-5 -s workspace-write -', promptVia: 'stdin' },
      },
    },
  });
  record(
    'generation with "-s workspace-write" is NOT flagged cli-advice-slot-writable (not advice-class)',
    !hasCode(problems, 'cli-advice-slot-writable'),
    JSON.stringify(problems),
  );
}

{
  // danger-full-access on an advice slot reports BOTH the universal
  // unsafe-flag code (UNSAFE_CLI_FLAGS' "-s danger-full-access") and the new
  // advice-class code (bare "danger-full-access" token).
  const problems = validateModelsConfig({
    models: {
      claude: {
        advisor: { kind: 'cli', command: 'codex exec -m gpt-5 -s danger-full-access -', promptVia: 'stdin' },
      },
    },
  });
  record(
    'advisor with "-s danger-full-access" reports both cli-unsafe-flag AND cli-advice-slot-writable',
    hasCode(problems, 'cli-unsafe-flag') && hasCode(problems, 'cli-advice-slot-writable'),
    JSON.stringify(problems),
  );
}

{
  // clean read-only advisor passes with zero problems.
  const problems = validateModelsConfig({
    models: {
      claude: {
        advisor: { kind: 'cli', command: 'codex exec -m gpt-5.6-sol -s read-only -', promptVia: 'stdin' },
      },
    },
  });
  record('clean "-s read-only" advisor passes (no problems)', problems.length === 0, JSON.stringify(problems));
}

// ── native V2 model-override shapes (D2, codex-native-transport cnt-1) ──────
// Detected BEFORE the looksLikeCli check (ADVISOR-R2 Δ1): {kind:'native'} would
// otherwise be mis-flagged cli-malformed, and the composite (no top-level kind)
// mis-flagged model-shape-malformed. Accept + reject rows for both.

{
  const problems = validateModelsConfig({
    models: { codex: { generation: { kind: 'native', model: 'gpt-5.5', effort: 'high', fork_turns: 'none', agent_type: 'worker' } } },
  });
  record('valid native override {kind:"native",model,effort,fork_turns:"none"} passes clean', problems.length === 0, JSON.stringify(problems));
}

{
  const problems = validateModelsConfig({
    models: {
      codex: {
        advisor: {
          primary: { kind: 'native', model: 'gpt-5.5', effort: 'high' },
          fallback: { kind: 'cli', command: 'codex exec -m gpt-5.5 -s read-only -', promptVia: 'stdin' },
          fallback_policy: 'explicit-only',
        },
      },
    },
  });
  record('valid explicit-only composite {primary(native),fallback(cli),fallback_policy} passes clean', problems.length === 0, JSON.stringify(problems));
}

{
  const problems = validateModelsConfig({
    models: { codex: { generation: { kind: 'native' } } }, // no model
  });
  record(
    'native override without a model is flagged native-model-missing (not cli-malformed)',
    hasCode(problems, 'native-model-missing') && !hasCode(problems, 'cli-malformed'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: { codex: { generation: { kind: 'native', model: 'gpt-5.5', fork_turns: 'full' } } }, // full-history fork rejects overrides (E2)
  });
  record(
    'native override with fork_turns other than "none" is flagged native-fork-turns-unknown',
    hasCode(problems, 'native-fork-turns-unknown'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: {
      codex: {
        advisor: {
          primary: { kind: 'native', model: 'gpt-5.5' },
          fallback: { kind: 'cli', command: 'codex exec -m gpt-5.5 -s read-only -' },
        },
      },
    }, // missing fallback_policy
  });
  record(
    'composite without fallback_policy is flagged composite-fallback-policy-missing (silent native->cli fallback forbidden, D1)',
    hasCode(problems, 'composite-fallback-policy-missing'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: {
      codex: {
        advisor: {
          primary: { model: 'gpt-5.5' }, // not a native override (no kind:'native')
          fallback: { kind: 'cli', command: 'x' },
          fallback_policy: 'explicit-only',
        },
      },
    },
  });
  record(
    'composite whose primary is not a native override is flagged composite-primary-malformed',
    hasCode(problems, 'composite-primary-malformed'),
    JSON.stringify(problems),
  );
}

{
  const problems = validateModelsConfig({
    models: {
      codex: {
        advisor: {
          primary: { kind: 'native', model: 'gpt-5.5' },
          fallback: { kind: 'cli' }, // fallback missing command
          fallback_policy: 'explicit-only',
        },
      },
    },
  });
  record(
    'composite whose cli fallback is malformed is flagged composite-fallback-malformed',
    hasCode(problems, 'composite-fallback-malformed'),
    JSON.stringify(problems),
  );
}

// ── malformed / null / wrong-type input never throws ───────────────────────

{
  let threw = false;
  let problems = [];
  try {
    problems = validateModelsConfig(null);
  } catch {
    threw = true;
  }
  record('validateModelsConfig(null) does not throw and reports a malformed-input row', !threw && hasCode(problems, 'config-malformed'), JSON.stringify(problems));
}

{
  // `undefined` is the "no config file on disk at all" case (a fresh repo) —
  // distinct from `null`: it must NOT be reported as a problem, or every
  // fresh repo without .bee/config.json would warn on every `bee status`.
  let threw = false;
  let problems = [];
  try {
    problems = validateModelsConfig(undefined);
  } catch {
    threw = true;
  }
  record('validateModelsConfig(undefined) does not throw and reports nothing (no config file yet is normal)', !threw && problems.length === 0, JSON.stringify(problems));
}

for (const bad of ['a string', 42, true, ['array', 'config'], () => {}]) {
  let threw = false;
  let problems = [];
  try {
    problems = validateModelsConfig(bad);
  } catch {
    threw = true;
  }
  record(
    `validateModelsConfig(${JSON.stringify(String(bad))}) (wrong type) does not throw and reports a malformed-input row`,
    !threw && hasCode(problems, 'config-malformed'),
    JSON.stringify(problems),
  );
}

{
  // `models` present but the wrong shape (not an object) — still no throw.
  let threw = false;
  let problems = [];
  try {
    problems = validateModelsConfig({ models: 'not-an-object' });
  } catch {
    threw = true;
  }
  record(
    'a `models` key of the wrong type does not throw and reports a malformed-input row',
    !threw && hasCode(problems, 'config-malformed'),
    JSON.stringify(problems),
  );
}

{
  // a runtime value of the wrong shape (not an object) — still no throw.
  let threw = false;
  let problems = [];
  try {
    problems = validateModelsConfig({ models: { claude: 'not-an-object' } });
  } catch {
    threw = true;
  }
  record(
    'a runtime value of the wrong type does not throw and reports runtime-malformed',
    !threw && hasCode(problems, 'runtime-malformed'),
    JSON.stringify(problems),
  );
}

// ── validateAgentFilesDrift (ao-3b-2, AO12) ─────────────────────────────────
// The pure validateModelsConfig above never touches disk; drift-checking a
// RENDERED .claude/agents/bee-*.md against live config needs root, so it is
// a separate helper (AO12 purity split). Fixtures below write agent files
// under a tmp root's .claude/agents/ directly — no onboarding render needed.

function mkFixtureRoot(prefix) {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), prefix));
  fs.mkdirSync(path.join(root, '.claude', 'agents'), { recursive: true });
  return root;
}

function writeAgentFile(root, agentName, frontmatterBody) {
  fs.writeFileSync(
    path.join(root, '.claude', 'agents', `${agentName}.md`),
    `---\n${frontmatterBody}\n---\n\nBody text, not parsed by the drift check.\n`,
  );
}

{
  const root = mkFixtureRoot('bee-agent-drift-flagged-');
  writeAgentFile(root, 'bee-gather', 'name: bee-gather\nmodel: opus');
  const problems = validateAgentFilesDrift(root, { models: { claude: { generation: 'sonnet' } } });
  record(
    'a rendered agent file whose model no longer matches the configured tier is flagged agent-file-drift',
    problems.length === 1 && problems[0].code === 'agent-file-drift' && problems[0].agent === 'bee-gather',
    JSON.stringify(problems),
  );
}

{
  const root = mkFixtureRoot('bee-agent-drift-match-');
  writeAgentFile(root, 'bee-gather', 'name: bee-gather\nmodel: sonnet');
  writeAgentFile(root, 'bee-extract', 'name: bee-extract\nmodel: haiku');
  writeAgentFile(root, 'bee-review', 'name: bee-review\nmodel: opus');
  const problems = validateAgentFilesDrift(root, {
    models: { claude: { generation: 'sonnet', extraction: 'haiku', review: 'opus' } },
  });
  record(
    'rendered agent files whose model matches the configured tier report no problems',
    problems.length === 0,
    JSON.stringify(problems),
  );
}

{
  // No .claude/agents/*.md files at all — nothing rendered yet, or a
  // cli/null slot correctly skipped by onboarding (AO10/AO11). Clean.
  const root = mkFixtureRoot('bee-agent-drift-missing-');
  const problems = validateAgentFilesDrift(root, { models: { claude: { generation: 'sonnet' } } });
  record('missing agent files produce no problems (absent is clean)', problems.length === 0, JSON.stringify(problems));
}

{
  // A configured slot now cli-shaped (no model name) but a stale rendered
  // file still declares a model — flagged, never silently accepted.
  const root = mkFixtureRoot('bee-agent-drift-cli-');
  writeAgentFile(root, 'bee-gather', 'name: bee-gather\nmodel: sonnet');
  const problems = validateAgentFilesDrift(root, {
    models: { claude: { generation: { kind: 'cli', command: 'codex exec -m gpt-5.5 -s read-only -' } } },
  });
  record(
    'a stale agent file under a now cli-shaped slot is flagged agent-file-drift',
    problems.length === 1 && problems[0].code === 'agent-file-drift',
    JSON.stringify(problems),
  );
}

{
  // Malformed frontmatter (no parseable "model:" line) never throws — it is
  // reported as its own problem code.
  const root = mkFixtureRoot('bee-agent-drift-malformed-');
  fs.writeFileSync(
    path.join(root, '.claude', 'agents', 'bee-extract.md'),
    'not even frontmatter, just plain text\n',
  );
  let threw = false;
  let problems = [];
  try {
    problems = validateAgentFilesDrift(root, { models: { claude: { extraction: 'haiku' } } });
  } catch {
    threw = true;
  }
  record(
    'malformed agent-file frontmatter never throws and reports agent-file-malformed',
    !threw && problems.length === 1 && problems[0].code === 'agent-file-malformed',
    JSON.stringify(problems),
  );
}

{
  // review slot falls back to generation when explicitly null (decision
  // 0021), same as resolveTier — the drift check must mirror that fallback.
  const root = mkFixtureRoot('bee-agent-drift-review-fallback-');
  writeAgentFile(root, 'bee-review', 'name: bee-review\nmodel: sonnet');
  const problems = validateAgentFilesDrift(root, {
    models: { claude: { generation: 'sonnet', review: null } },
  });
  record(
    'a null review slot falls back to generation for the drift comparison (decision 0021)',
    problems.length === 0,
    JSON.stringify(problems),
  );
}

{
  // rawConfig undefined (no .bee/config.json on disk at all) never throws —
  // resolves against the seeded defaults exactly like a fresh repo.
  const root = mkFixtureRoot('bee-agent-drift-undefined-config-');
  writeAgentFile(root, 'bee-gather', 'name: bee-gather\nmodel: sonnet');
  let threw = false;
  let problems = [];
  try {
    problems = validateAgentFilesDrift(root, undefined);
  } catch {
    threw = true;
  }
  record(
    'validateAgentFilesDrift(root, undefined) does not throw and resolves against seeded defaults',
    !threw && problems.length === 0,
    JSON.stringify(problems),
  );
}

const failed = results.filter((r) => !r.passed);
console.log(`SUMMARY: ${results.length - failed.length}/${results.length} passed`);
process.exit(failed.length ? 7 : 0);
