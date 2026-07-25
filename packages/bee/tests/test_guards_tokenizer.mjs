// guards.test.mjs — first automated test harness for guards.mjs (D4).
//
// Covers cell fix-write-guard-symlink-e4:
//   - D9: glued-separator mis-tokenization (any of `;`, `&&`, `&`, `|`, `||`
//     glued directly to an adjacent token must split into its own token,
//     exactly like the already-working spaced form).
//   - D8: the CLI-owned direct-edit check distinguishes `git add` (staging
//     already-written content — allow) from an actual content mutation
//     (still deny), including a mixed chained command.
//   - D12: `.bee/companion-session.json` joins DIRECT_EDIT_DENY.
//
// Every case below is written red-before-green: run against the pre-fix
// guards.mjs, the "GLUED" cases fail (garbled paths, extra literal tokens)
// and the D8/D12 cases fail (false-positive denial / missing denial). After
// the fix, all pass.

import assert from 'node:assert/strict';
import { test } from 'node:test';
import { checkWrite, extractBashTargets, tokenize } from '../lib/guards.mjs';
import { tokenizeCommand } from '../../../.bee/bin/hooks/tokenize-command.mjs';

const SWARMING_STATE = { phase: 'swarming', approved_gates: { execution: true } };

function verdict(relPath) {
  return checkWrite('.', SWARMING_STATE, relPath, 'test-agent', {});
}

// ─── D9: glued separator tokenization, all four+ forms ────────────────────

const SEPARATOR_FORMS = [
  { label: 'semicolon', sep: ';' },
  { label: 'doubled-ampersand', sep: '&&' },
  { label: 'single-ampersand', sep: '&' },
  { label: 'single-pipe', sep: '|' },
  { label: 'doubled-pipe', sep: '||' }, // same SEPARATORS set, same fix — free regression coverage
];

for (const { label, sep } of SEPARATOR_FORMS) {
  test(`D9: GLUED ${label} separator between chained git-add targets is split correctly`, () => {
    const command = `git add a.txt${sep}git add b.txt`;
    const { paths } = extractBashTargets(command);
    assert.deepEqual(
      paths,
      ['a.txt', 'b.txt'],
      `glued "${sep}" must not glue onto the adjacent token or leak command-verb tokens into paths`,
    );
  });

  test(`D9 regression: already-working SPACED ${label} separator stays correct`, () => {
    const command = `git add a.txt ${sep} git add b.txt`;
    const { paths } = extractBashTargets(command);
    assert.deepEqual(paths, ['a.txt', 'b.txt']);
  });
}

test('D9: fd-duplication redirects (2>&1, 1>&2) are still not treated as file writes', () => {
  assert.deepEqual(extractBashTargets('echo hi 2>&1').paths, []);
  assert.deepEqual(extractBashTargets('echo hi 1>&2').paths, []);
});

test('D9: quoted arguments containing a separator character are not split', () => {
  const { paths } = extractBashTargets("rm 'a&b.txt'");
  assert.deepEqual(paths, ['a&b.txt']);
});

// ─── D9 follow-up (independent review): adjacent-quote concatenation and
// backslash-escaped separators, found while verifying the tokenizer fix ────

test('D9: adjacent quoted segments with no space merge into one word (bash word-splitting), so a DIRECT_EDIT_DENY path cannot be split across quotes to evade containment', () => {
  const { paths } = extractBashTargets(`echo bad > '.bee/state'".json"`);
  assert.deepEqual(paths, ['.bee/state.json']);
  const v = verdict(paths[0]);
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
});

test('D9: a backslash-escaped separator inside an unquoted token stays part of that token (real filename, not a command boundary)', () => {
  const { paths } = extractBashTargets('rm a\\;b.txt');
  assert.deepEqual(paths, ['a;b.txt']);
});

// ─── D9 follow-up (2nd independent review): tokenizer equivalence ─────────
// guards.mjs's tokenize() and bee-write-guard.mjs's tokenizeCommand() (moved
// to tokenize-command.mjs so it's importable here) are meant to be the same
// algorithm, hand-synced rather than shared via import (see both files'
// comments for why). Nothing else catches drift between them — this is that
// catch. On failure: re-apply whatever changed in guards.mjs's tokenize() to
// tokenize-command.mjs (or vice versa) until this passes again.

const TOKENIZER_EQUIVALENCE_CORPUS = [
  '', ';', ';;;', '&|&|', '|||', '&&&',
  'git add a.txt;git add b.txt', 'git add a.txt && git add b.txt',
  'echo hi 2>&1', 'echo hi 1>&2', "rm 'a&b.txt'",
  `echo bad > '.bee/state'".json"`, 'rm a\\;b.txt',
  'echo "never closes', 'rm file\\', 'rm \\"file',
  '"a""b""c"', `"it's here"`, 'cmd\ta\nb\rc',
  'a'.repeat(50) + ';'.repeat(20) + 'b'.repeat(50),
];

test('D9: guards.mjs tokenize() and the hook\'s tokenizeCommand() (tokenize-command.mjs) stay in sync', () => {
  for (const command of TOKENIZER_EQUIVALENCE_CORPUS) {
    assert.deepEqual(
      tokenizeCommand(command),
      tokenize(command),
      `tokenizer drift on: ${JSON.stringify(command)}`,
    );
  }
});

// ─── D8: CLI-owned git add/commit vs content mutation ─────────────────────

test('D8: a chained git-add-then-git-commit targeting a CLI-owned file succeeds (no direct-edit target)', () => {
  const { paths } = extractBashTargets('git add .bee/backlog.jsonl && git commit -m "stage"');
  assert.deepEqual(paths, [], 'staging/committing a CLI-owned file must not surface as a direct-edit target');
});

test('D8: git add of a CLI-owned file alone is exempt (not a target)', () => {
  const { paths } = extractBashTargets('git add .bee/backlog.jsonl');
  assert.deepEqual(paths, []);
});

test('D8: a mixed chained command (mutate + stage the same CLI-owned file) still denies, citing the mutating segment', () => {
  const { paths } = extractBashTargets('echo bad >> .bee/backlog.jsonl && git add .bee/backlog.jsonl');
  assert.deepEqual(paths, ['.bee/backlog.jsonl'], 'only the mutating segment\'s target should surface');
  const v = verdict(paths[0]);
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
});

test('D8: a single (non-chained) content-mutating Bash command on a CLI-owned file still denies', () => {
  const { paths } = extractBashTargets('echo bad > .bee/backlog.jsonl');
  assert.deepEqual(paths, ['.bee/backlog.jsonl']);
  const v = verdict(paths[0]);
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
});

test('D8: a direct Edit-tool-shaped write (bare checkWrite call) on a CLI-owned file still denies exactly as today', () => {
  const v = verdict('.bee/backlog.jsonl');
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
});

test('D8: git mv/git rm of a CLI-owned file are NOT exempted (only git add/commit are)', () => {
  const mv = extractBashTargets('git mv .bee/backlog.jsonl .bee/backlog2.jsonl');
  assert.ok(mv.paths.includes('.bee/backlog.jsonl'));
  const rm = extractBashTargets('git rm .bee/backlog.jsonl');
  assert.deepEqual(rm.paths, ['.bee/backlog.jsonl']);
  assert.equal(verdict(rm.paths[0]).allow, false);
});

test('D8: git add of a non-CLI-owned file is completely unaffected (still a target)', () => {
  const { paths } = extractBashTargets('git add src/foo.txt');
  assert.deepEqual(paths, ['src/foo.txt']);
  assert.equal(verdict(paths[0]).allow, true);
});

// ─── D12: .bee/companion-session.json joins DIRECT_EDIT_DENY ──────────────

test('D12: a direct Edit/Write/Bash-redirect attempt on .bee/companion-session.json is denied as CLI-owned', () => {
  const v = verdict('.bee/companion-session.json');
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
  assert.match(v.reason, /companion-session\.json/);
});

test('D12: .bee/state.json keeps its existing, unchanged direct-edit protection', () => {
  const v = verdict('.bee/state.json');
  assert.equal(v.allow, false);
  assert.equal(v.kind, 'direct-edit');
});
