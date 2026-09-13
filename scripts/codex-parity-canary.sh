#!/usr/bin/env bash
# Real Codex, installed matcher/commands, retained inputs and observable writes.
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Capture host environment and custom settings before any overrides
HOST_REAL_HOME="$(realpath "${HOME:-/nonexistent}" 2>/dev/null || echo "${HOME:-/nonexistent}")"
HOST_REAL_CODEX="$(realpath "${HOST_REAL_HOME}/.codex" 2>/dev/null || echo "${HOST_REAL_HOME}/.codex")"
HOST_XDG_CONFIG="$(realpath "${XDG_CONFIG_HOME:-$HOST_REAL_HOME/.config}" 2>/dev/null || echo "${XDG_CONFIG_HOME:-$HOST_REAL_HOME/.config}")"
HOST_XDG_DATA="$(realpath "${XDG_DATA_HOME:-$HOST_REAL_HOME/.local/share}" 2>/dev/null || echo "${XDG_DATA_HOME:-$HOST_REAL_HOME/.local/share}")"
HOST_XDG_STATE="$(realpath "${XDG_STATE_HOME:-$HOST_REAL_HOME/.local/state}" 2>/dev/null || echo "${XDG_STATE_HOME:-$HOST_REAL_HOME/.local/state}")"
HOST_XDG_CACHE="$(realpath "${XDG_CACHE_HOME:-$HOST_REAL_HOME/.cache}" 2>/dev/null || echo "${XDG_CACHE_HOME:-$HOST_REAL_HOME/.cache}")"

# Do not silently fall back from an explicitly supplied invalid BEE_BIN to a different binary
if [[ -n "${BEE_BIN:-}" ]]; then
  test -f "$BEE_BIN" && test -x "$BEE_BIN" || {
    echo "Specified BEE_BIN is not an executable file: $BEE_BIN" >&2
    exit 1
  }
else
  BEE_BIN="${CARGO_TARGET_DIR:-$REPO_ROOT/packages/bee-rs/target}/release/bee"
  if [[ ! -x "$BEE_BIN" && -x "$REPO_ROOT/.bee/bin/bee" ]]; then
    BEE_BIN="$REPO_ROOT/.bee/bin/bee"
  fi
  test -x "$BEE_BIN" || { echo 'Build the release bee binary first.' >&2; exit 1; }
fi

# Require explicit direct executable; reject discovery, mise invocation, and wrappers.
test -n "${CODEX_BIN:-}" || {
  echo 'Provide an explicit direct executable in CODEX_BIN; discovery or PATH wrapper is refused.' >&2
  exit 1
}

# Require canonical absolute path; reject bare commands and relative paths.
[[ "$CODEX_BIN" == /* ]] || {
  echo 'CODEX_BIN must be an absolute path, not a relative path or bare command resolved via PATH.' >&2
  exit 1
}

test -f "$CODEX_BIN" && test -x "$CODEX_BIN" || {
  echo "CODEX_BIN is not an executable file: $CODEX_BIN" >&2
  exit 1
}

CODEX_REAL="$(realpath "$CODEX_BIN")"
CODEX_BIN="$CODEX_REAL"

# Refuse wrapper scripts that invoke mise or delegate through shims.
# Avoid read-scanning entire large binary executables by checking header / limiting size.
if [[ "$(head -c 4 "$CODEX_REAL" 2>/dev/null || true)" != $'\x7fELF' ]]; then
  if head -c 65536 "$CODEX_REAL" 2>/dev/null | grep -Eq '(mise[[:space:]]+(use|x|exec|run)|\bmise\b|/mise/shims)'; then
    echo 'Refusing mise wrapper script in CODEX_BIN; provide the direct Codex executable.' >&2
    exit 1
  fi
fi

if [[ "$CODEX_REAL" == *"/mise/shims/"* ]]; then
  echo 'Refusing mise shim in CODEX_BIN; provide the direct Codex executable.' >&2
  exit 1
fi

# Replace or configure BEE_CODEX_PROBE_BIN directly so nested probes cannot invoke wrappers
export BEE_CODEX_PROBE_BIN="$CODEX_REAL"

# Validate isolated Codex home and reject unsafe homes, descendants, and aliases before probe launch
: "${CANARY_CODEX_HOME:?Provide an isolated Codex home; do not use the real user home}"
RESOLVED_CODEX_HOME="$(realpath -m "$CANARY_CODEX_HOME")"

is_sensitive_path() {
  local target="$1"

  # Reject the real user home itself
  if [[ "$target" == "$HOST_REAL_HOME" ]]; then
    return 0
  fi

  # Reject sensitive configuration and data directories and all their descendants
  local s
  local sensitive=(
    "$HOST_REAL_CODEX"
    "$HOST_REAL_HOME/.codex"
    "$HOST_REAL_HOME/.config"
    "$HOST_REAL_HOME/.local"
    "$HOST_REAL_HOME/.cache"
    "$HOST_XDG_CONFIG"
    "$HOST_XDG_DATA"
    "$HOST_XDG_STATE"
    "$HOST_XDG_CACHE"
  )
  for s in "${sensitive[@]}"; do
    [[ -n "$s" && "$s" != "/" ]] || continue
    if [[ "$target" == "$s" || "$target" == "$s/"* ]]; then
      return 0
    fi
  done
  if [[ "$target" == "/" || "$target" == "/root" || "$target" == "/root/"* || \
        "$target" == "/home" || "$target" == "/etc" || "$target" == "/etc/"* ]]; then
    return 0
  fi
  return 1
}

if is_sensitive_path "$RESOLVED_CODEX_HOME"; then
  echo "Refusing real user home, settings directory, or unsafe path as CANARY_CODEX_HOME: $CANARY_CODEX_HOME" >&2
  exit 1
fi

# Reject symlinks pointing into real settings
if [[ -L "$CANARY_CODEX_HOME" ]]; then
  TARGET_OF_LINK="$(realpath "$CANARY_CODEX_HOME" 2>/dev/null || true)"
  if is_sensitive_path "$TARGET_OF_LINK"; then
    echo "Refusing symlink to real home/settings in CANARY_CODEX_HOME: $CANARY_CODEX_HOME -> $TARGET_OF_LINK" >&2
    exit 1
  fi
fi

if [[ -d "$CANARY_CODEX_HOME" ]]; then
  while IFS= read -r -d '' link; do
    link_target="$(realpath "$link" 2>/dev/null || true)"
    # Allow legitimate Codex executable helper link pointing to the direct CODEX_REAL executable
    if [[ -n "$CODEX_REAL" && "$link_target" == "$CODEX_REAL" && -x "$link_target" ]]; then
      continue
    fi
    if is_sensitive_path "$link_target"; then
      echo "Refusing symlink into real user settings inside CANARY_CODEX_HOME: $link -> $link_target" >&2
      exit 1
    fi
  done < <(find "$CANARY_CODEX_HOME" -type l -print0 2>/dev/null)
fi

# Establish controlled PATH without host wrappers or mise shims
CODEX_DIR="$(dirname "$CODEX_REAL")"
SAFE_PATH=""
add_safe_path() {
  local dir="$1"
  [[ -d "$dir" ]] || return 0
  local canonical
  canonical="$(realpath "$dir" 2>/dev/null || true)"
  [[ -n "$canonical" ]] || return 0
  # Exclude mise shims
  if [[ "$canonical" == *"/mise/shims"* ]]; then
    return 0
  fi
  # If directory contains a codex executable that is not CODEX_REAL, exclude it
  if [[ -x "$canonical/codex" ]]; then
    local candidate
    candidate="$(realpath "$canonical/codex" 2>/dev/null || true)"
    if [[ "$candidate" != "$CODEX_REAL" ]]; then
      return 0
    fi
  fi
  # Avoid duplicate entries
  if [[ ":$SAFE_PATH:" != *":$canonical:"* ]]; then
    if [[ -z "$SAFE_PATH" ]]; then
      SAFE_PATH="$canonical"
    else
      SAFE_PATH="$SAFE_PATH:$canonical"
    fi
  fi
}

add_safe_path "$CODEX_DIR"
for tool in node git jq timeout cmp cp mktemp dirname realpath printf env; do
  tool_path="$(command -v "$tool" 2>/dev/null || true)"
  if [[ -n "$tool_path" ]]; then
    add_safe_path "$(dirname "$tool_path")"
  fi
done
for std in /usr/local/bin /usr/bin /bin /usr/local/sbin /usr/sbin /sbin; do
  add_safe_path "$std"
done
IFS=':' read -ra P_DIRS <<< "$PATH"
for p in "${P_DIRS[@]}"; do
  [[ -n "$p" ]] && add_safe_path "$p"
done
export PATH="$SAFE_PATH"

# Setup hermetic canary directories and isolate HOME, XDG, and CODEX_HOME
CANARY_ROOT="$(mktemp -d "${TMPDIR:-/var/tmp}/bee-codex-canary-XXXXXX")"
CANARY_HOME="$CANARY_ROOT/home"
CANARY_REPO="$CANARY_ROOT/repo"
EVIDENCE="$CANARY_ROOT/evidence"

mkdir -p "$CANARY_HOME/.config" "$CANARY_HOME/.local/share" "$CANARY_HOME/.local/state" "$CANARY_HOME/.cache"
mkdir -p "$CANARY_REPO" "$EVIDENCE"

export HOME="$CANARY_HOME"
export XDG_CONFIG_HOME="$CANARY_HOME/.config"
export XDG_DATA_HOME="$CANARY_HOME/.local/share"
export XDG_STATE_HOME="$CANARY_HOME/.local/state"
export XDG_CACHE_HOME="$CANARY_HOME/.cache"
export CODEX_HOME="$CANARY_CODEX_HOME"
export CANARY_REPO EVIDENCE

printf 'Retained canary: %s\n' "$CANARY_ROOT"
trap 'printf "Canary evidence retained: %s\n" "$EVIDENCE"' EXIT
"$CODEX_REAL" --version > "$EVIDENCE/version.txt" 2> "$EVIDENCE/version.stderr"
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
  timeout 180 "$CODEX_REAL" exec --ignore-user-config --ignore-rules --json \
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
