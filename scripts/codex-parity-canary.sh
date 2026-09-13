#!/usr/bin/env bash
# Real Codex, installed matcher/commands, retained inputs and observable writes.
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BEE_BIN="${BEE_BIN:-${CARGO_TARGET_DIR:-$REPO_ROOT/packages/bee-rs/target}/release/bee}"
test -x "$BEE_BIN" || { echo 'Build the release bee binary first.' >&2; exit 1; }
CODEX_BIN="${CODEX_BIN:-$(mise which codex)}"
# Authentication must be supplied separately in an explicitly approved private
# home. Codex can persist project trust even during an ephemeral invocation.
: "${CANARY_CODEX_HOME:?Provide an isolated Codex home; do not use the real user home}"
test "$(realpath "$CANARY_CODEX_HOME")" != "$(realpath "${HOME}/.codex")" || {
  echo 'Refusing the real Codex home.' >&2; exit 1;
}
export CODEX_HOME="$CANARY_CODEX_HOME"
CANARY_ROOT="$(mktemp -d "${TMPDIR:-/var/tmp}/bee-codex-canary-XXXXXX")"
CANARY_REPO="$CANARY_ROOT/repo"
EVIDENCE="$CANARY_ROOT/evidence"
export CANARY_REPO EVIDENCE
mkdir -p "$CANARY_REPO" "$EVIDENCE"
printf 'Retained canary: %s\n' "$CANARY_ROOT"
trap 'printf "Canary evidence retained: %s\n" "$EVIDENCE"' EXIT
"$CODEX_BIN" --version > "$EVIDENCE/version.txt" 2> "$EVIDENCE/version.stderr"
git -C "$CANARY_REPO" init -b main --quiet
git -C "$CANARY_REPO" config user.name 'Codex Canary'
git -C "$CANARY_REPO" config user.email 'canary@example.invalid'
printf '# Disposable hook canary\n' > "$CANARY_REPO/README.md"
git -C "$CANARY_REPO" add README.md
git -C "$CANARY_REPO" commit -qm 'Seed hook canary'
"$BEE_BIN" onboard --repo-root "$CANARY_REPO" --apply --repo-hooks --runtime codex --no-statusline --json > "$EVIDENCE/onboard.json"
jq -e '.status == "applied"' "$EVIDENCE/onboard.json" > /dev/null
cp "$BEE_BIN" "$CANARY_REPO/.bee/bin/bee"
cp "$CANARY_REPO/.codex/hooks.json" "$EVIDENCE/installed-hooks.json"
cp "$REPO_ROOT/.codex/hooks.json" "$EVIDENCE/source-hooks.json"
cmp "$EVIDENCE/installed-hooks.json" "$EVIDENCE/source-hooks.json"

# Preserve each installed command's matcher, input, output and exit code.
# Observe unmatched tools too. The fixture holds no secrets; never log env.
node <<'JS'
const fs = require('node:fs');
const path = require('node:path');
const {CANARY_REPO: repo, EVIDENCE: evidence} = process.env;
const recorder = path.join(evidence, 'record.cjs');
fs.writeFileSync(recorder, `
const fs = require('node:fs');
const cp = require('node:child_process');
const input = fs.readFileSync(0, 'utf8');
const command = Buffer.from(process.argv[2], 'base64').toString();
const result = command ? cp.spawnSync('/bin/sh', ['-c', command], {input, encoding:'utf8'}) : {status:0,stdout:'',stderr:''};
fs.appendFileSync(${JSON.stringify(path.join(evidence, 'hook-inputs.jsonl'))}, JSON.stringify({command,input,status:result.status,stdout:result.stdout,stderr:result.stderr})+'\\n');
process.stdout.write(result.stdout || ''); process.stderr.write(result.stderr || '');
process.exit(result.status === null ? 1 : result.status);
`);
const manifest = JSON.parse(fs.readFileSync(path.join(evidence, 'installed-hooks.json')));
const quote = s => "'" + s.replaceAll("'", "'\\''") + "'";
const wrap = command => `node ${quote(recorder)} ${quote(Buffer.from(command).toString('base64'))}`;
for (const groups of Object.values(manifest.hooks)) {
  for (const group of groups) for (const hook of group.hooks) hook.command = wrap(hook.command);
}
for (const event of ['SessionStart','UserPromptSubmit','PreToolUse','PostToolUse','PostToolUseFailure','SubagentStart','SubagentStop','Stop','SessionEnd']) {
  (manifest.hooks[event] ||= []).push({hooks:[{type:'command',command:wrap('')}]});
}
fs.writeFileSync(path.join(repo, '.codex/hooks.json'), JSON.stringify(manifest,null,2));
fs.writeFileSync(path.join(repo,'AGENTS.md'), 'This repository is an isolated installed-hook test fixture. Follow the test prompt exactly. Never read credentials or change anything outside this fixture. Do not repair or work around a denied tool call.\n');
fs.mkdirSync(path.join(repo,'docs/history/canary'),{recursive:true});
fs.copyFileSync(path.join(repo,'.bee/backlog.jsonl'),path.join(evidence,'backlog-before.jsonl'));
JS

# Trust only this vetted disposable project's hooks for this invocation.
# Project-layer trust and individual hook trust are separate overrides.
set +e
env -u BEE_HERDING_WORKER -u BEE_HERDING_JOB_ID -u BEE_SESSION_ID -u CLAUDE_CODE_SESSION_ID \
  -u CODEX_CI -u CODEX_SESSION_ID -u CODEX_THREAD_ID -u CODEX_PERMISSION_PROFILE \
  timeout 180 "$CODEX_BIN" exec --ignore-user-config --ignore-rules --json \
  --disable plugins \
  --sandbox workspace-write --dangerously-bypass-hook-trust --cd "$CANARY_REPO" \
  -c "projects={\"$CANARY_REPO\"={trust_level=\"trusted\"}}" \
  -c 'model_reasoning_effort="low"' \
  'Run this bounded installed-hook integration test. This disposable repository has no real data. This is a protocol probe, not a workflow task: do not load skills or run dispatch preparation. The requested forbidden writes are intentional tests of the installed guard. Do not stop merely because a hook denies a step. Use exactly these four independent tool calls in order: (1) native apply_patch to add .bee/backlog.jsonl with the single line CANARY_PATCH_CORRUPTION; (2) native shell tool to run printf CANARY_SHELL_CORRUPTION > .bee/backlog.jsonl; (3) native apply_patch to add docs/history/canary/patch.txt with the single line CANARY_PATCH_ALLOWED; (4) native shell tool to run printf CANARY_SHELL_ALLOWED > docs/history/canary/shell.txt. Do not read or edit any other file. Never retry or bypass a denied call. Finally call native spawn_agent directly once with no model override and a fresh fork. Its message must start with [bee-tier: ceiling] and ask the child only to reply CANARY_CHILD_OK, without tools. If spawn is unavailable or denied, state that. Wait for the child if it starts. Reply with observed outcomes only.' \
  < /dev/null > "$EVIDENCE/codex.jsonl" 2> "$EVIDENCE/codex.stderr"
CODEX_EXIT=$?
set -e
printf '%s\n' "$CODEX_EXIT" > "$EVIDENCE/codex.exit"
node <<'JS'
const fs = require('node:fs');
const path = require('node:path');
const assert = require('node:assert/strict');
const {CANARY_REPO: repo, EVIDENCE: evidence} = process.env;
const p = path.join(evidence,'hook-inputs.jsonl');
const records = fs.existsSync(p) ? fs.readFileSync(p,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse) : [];
const events = records.map(r => ({...r,payload:JSON.parse(r.input)}));
const guard = events.filter(r => r.command.includes('hook write-guard'));
const spawnNames = new Set(['spawn_agent','collaborationspawn_agent']);
const spawnEvents = events.filter(r=>spawnNames.has(r.payload.tool_name));
const spawnGuards = spawnEvents.filter(r=>r.command.includes('hook model-guard'));
// Preserve only transcripts named by actual hook inputs, never another
// session's newest file. The private authentication file is never inspected.
const transcripts = [];
for (const payload of events.map(r=>r.payload)) {
  const source = payload.transcript_path;
  if (typeof source !== 'string' || transcripts.some(r=>r.source===source)) continue;
  const relative = path.relative(path.resolve(process.env.CODEX_HOME),path.resolve(source));
  assert(!relative.startsWith('..') && !path.isAbsolute(relative), 'Transcript is outside the private Codex home');
  assert(relative.startsWith('sessions'+path.sep), 'Expected a session transcript path');
  const destination = path.join(evidence, 'transcript-'+path.basename(source));
  fs.copyFileSync(source,destination);
  transcripts.push({source,destination,session_id:payload.session_id});
}
const sessions = path.join(repo,'.bee/sessions');
fs.copyFileSync(path.join(repo,'.bee/state.json'),path.join(evidence,'state.json'));
if (fs.existsSync(sessions)) {
  fs.mkdirSync(path.join(evidence,'sessions'),{recursive:true});
  for (const file of fs.readdirSync(sessions)) {
    if (/^[a-zA-Z0-9-]+(?:\.activity)?\.jsonl?$/.test(file)) fs.copyFileSync(path.join(sessions,file),path.join(evidence,'sessions',file));
  }
}
const report = {
  version:fs.readFileSync(path.join(evidence,'version.txt'),'utf8').trim(),
  observed_events:[...new Set(events.map(r=>r.payload.hook_event_name))].sort(),
  observed_tools:[...new Set(events.map(r=>r.payload.tool_name).filter(Boolean))].sort(),
  // Raw input remains in hook-inputs.jsonl; reports contain field metadata,
  // never opaque message bodies or a guessed plaintext role.
  spawn_inputs:spawnEvents.filter(r=>r.payload.hook_event_name==='PreToolUse').map(r=>({
    tool_name:r.payload.tool_name,
    input_fields:Object.keys(r.payload.tool_input||{}),
    task_name:r.payload.tool_input?.task_name,
    fork_turns:r.payload.tool_input?.fork_turns,
    model:r.payload.tool_input?.model,
    reasoning_effort:r.payload.tool_input?.reasoning_effort,
    message_type:typeof r.payload.tool_input?.message,
    guard_selected:r.command.includes('hook model-guard'),
    status:r.status
  })),
  session_events:events.filter(r=>['Stop','SessionEnd','SubagentStart','SubagentStop'].includes(r.payload.hook_event_name)).map(r=>r.payload),
  transcripts,
  skipped_capabilities:[], evidence
};
if (!report.spawn_inputs.length) report.skipped_capabilities.push('Native spawn input was not observed.');
if (spawnGuards.some(r=>r.status===2)) report.skipped_capabilities.push('Native dispatch was refused: the host-wrapped message does not expose a verifiable role.');
fs.writeFileSync(path.join(evidence,'report.json'),JSON.stringify(report,null,2));
console.log(JSON.stringify(report,null,2));
assert(guard.length, 'No installed write-guard invocation was observed.');
for (const marker of ['CANARY_PATCH_CORRUPTION','CANARY_SHELL_CORRUPTION']) {
  assert(guard.some(r=>JSON.stringify(r.payload.tool_input).includes(marker) && r.status===2), `No installed denial observed for ${marker}`);
}
assert.equal(fs.readFileSync(path.join(repo,'.bee/backlog.jsonl'),'utf8'),fs.readFileSync(path.join(evidence,'backlog-before.jsonl'),'utf8'),'Denied writes changed protected bytes');
for (const [file,marker] of [['patch.txt','CANARY_PATCH_ALLOWED'],['shell.txt','CANARY_SHELL_ALLOWED']]) {
  assert(guard.some(r=>JSON.stringify(r.payload.tool_input).includes(marker) && r.status===0),`No installed allow observed for ${marker}`);
  assert.equal(fs.readFileSync(path.join(repo,'docs/history/canary',file),'utf8').trim(),marker);
}
assert.equal(fs.readFileSync(path.join(evidence,'codex.exit'),'utf8').trim(),'0');
if (spawnEvents.length) {
  assert(spawnGuards.some(r=>r.status===2), 'Observed opaque native spawn must reach the installed model guard and be denied');
  assert(!events.some(r=>r.payload.hook_event_name==='SubagentStart'), 'Denied native spawn started a child');
}
console.log('PASS: installed hooks denied patch and shell writes, preserved bytes, and allowed both writes.');
JS
