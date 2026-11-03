#!/bin/bash
CONFIG="$1"
MODEL="$2"
LABEL="$3"
S=/tmp/claude-1000/-home-thanhsmind-Projects-goglbe-beehive/db52a73c-18e5-4fa4-838b-1563220e1b7a/scratchpad
OUT=$S/measure/$LABEL
SB=$OUT/repo
if [ "$CONFIG" = "old" ]; then
  BIN=$S/old-target/release/bee
  TREE=$S/old-tree
else
  BIN=/home/thanhsmind/.cache/cargo-target/release/bee
  TREE=/home/thanhsmind/Projects/goglbe/beehive
fi
rm -rf "$OUT"
mkdir -p "$SB"
cd "$SB" || exit 1
git init -q -b main
git config user.email measure@example.invalid
git config user.name measure
printf '# measure sandbox\n\nA tiny repo for one measured run.\n' > README.md
git add -A && git commit -qm "seed the sandbox"
(cd "$TREE" && "$BIN" onboard --repo-root "$SB" --apply --json > "$OUT/onboard.json" 2>&1)
mkdir -p .bee/bin
cp "$BIN" .bee/bin/bee
python3 -c 'import json;p=".bee/config.json";d=json.load(open(p));d["gate_bypass"]="full";open(p,"w").write(json.dumps(d,indent=2)+"\n")'
git add -A && git commit -qm "onboard bee, gate bypass full" -q
TASK='Add a shell script scripts/hello.sh that prints "hello from bee" and make it executable. This repo uses the bee workflow: follow AGENTS.md exactly, from orient to a merged and capped cell. Do not ask questions; the gate bypass is on.'
START=$(date +%s)
timeout 1500 pi -p --model "$MODEL" --mode json --session-dir "$OUT/sessions" "$TASK" > "$OUT/stdout.jsonl" 2> "$OUT/stderr.txt"
echo "pi_exit=$? seconds=$(( $(date +%s) - START ))" > "$OUT/run.txt"
{
  echo "main_has_script=$( git -C "$SB" show main:scripts/hello.sh >/dev/null 2>&1 && echo yes || echo no )"
  echo "main_script_exec=$( git -C "$SB" ls-tree main scripts/hello.sh 2>/dev/null | grep -q '^100755' && echo yes || echo no )"
  echo "commits=$(git -C "$SB" rev-list --count main)"
  echo "worktrees=$(git -C "$SB" worktree list | wc -l)"
} >> "$OUT/run.txt"
"$BIN" cells list --json > "$OUT/cells.json" 2>/dev/null
cat "$OUT/run.txt"
