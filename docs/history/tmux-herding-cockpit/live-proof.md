# tmux transport — live proof (2026-08-23)

Phase 3 of `docs/history/tmux-herding-transport/plan.md`: an owner-run round
trip on a real tmux server, driven from a detached session named `beeproof`
(tmux 3.4, WSL2). Every command below was typed into a tmux pane (`send-keys`,
two calls) and read back with `capture-pane`; the binary is the phase-2 build
installed at `.bee/bin/bee`. `.bee/config.json` carried
`herding.transport: "tmux"` for the run and was reverted afterwards.

## A — probe, current pane, list (pane %0)

```
$ echo TMUX=$TMUX PANE=$TMUX_PANE
TMUX=/tmp/tmux-1000/default,152819,0 PANE=%0
$ .bee/bin/bee herding status --json
… "transport":{"ready":true,"reason":"TMUX and TMUX_PANE=%0 are set","pane_id":"%0","kind":"tmux"}}
$ .bee/bin/bee herding pane current
{"ok":true,"transport":"tmux","result":{"pane_id":"%0","tab_id":"@0","workspace_id":"beeproof"}}
$ .bee/bin/bee herding pane list --with-status
{"ok":true,"transport":"tmux","result":{"panes":[{"pane_id":"%0","label":"DESKTOP-ThanhsMind","tab_id":"@0",
 "cwd":"/home/thanhsmind/projects/goglbe/beehive","command":".bee/bin/bee",
 "foreground_cwd":"/home/thanhsmind/projects/goglbe/beehive","agent_status":"idle","agent_session":null}]}}
```

Observed: tmux's default pane title is the host name, so an unrenamed pane
reads `label: "DESKTOP-ThanhsMind"`. `pane-id --label` only matches after a
`pane rename`, which is what bootstrap and the roles do.

## B — split, rename, label lookup, send, read, layout, tabs, close

```
$ bee herding pane split %0 --direction right --ratio 0.6 --cwd …/beehive
{"ok":true,"transport":"tmux","result":{"pane_id":"%2"}}
$ bee herding pane rename %2 worker-a            → {"ok":true,…,"result":{}}
$ bee herding pane-id --label worker-a           → {"ok":true,…,"result":{"pane_id":"%2"}}
$ bee herding pane send-text %2 'echo hello-from-bee-$((6*7))'   → ok
$ bee herding pane read %2 --lines 5
{… "text":"…$ echo hello-from-bee-$((6*7))\nhello-from-bee-42\n…$"}
$ bee herding pane layout --pane %0
{… "panes":[{"pane_id":"%0","width":119,"height":50,"x":0,"y":0},{"pane_id":"%2","width":80,"height":50,"x":120,"y":0}]}
$ bee herding pane tab-list
{… "tabs":[{"tab_id":"@0","label":"bash"},{"tab_id":"@1","label":"bash"}]}
$ bee herding pane close %2                      → ok; list-panes shows %0 only
```

Observed: with `--ratio 0.6` (the share the parent keeps) the parent kept
119 of 200 columns and the child got 80 — the ratio semantics match herdr.

## C — run dry-run and bootstrap dry-run (pane %1)

```
$ bee herding run --task "say hello" --dry-run --json
{"job_id":"job-…","outcome":"dry_run",…,"transport":"tmux","job_path":"…/.bee/mailbox/job-…/job.json"}
$ bash skills/bee-herding/scripts/bootstrap-cockpit.sh --dry-run --main-root .
./.bee/bin/bee herding pane-id --label dispatch --main-root .
./.bee/bin/bee herding pane current --main-root .
./.bee/bin/bee herding pane split <cockpit_chat_pane> --direction right --cwd . --main-root .
./.bee/bin/bee herding pane split <cockpit_dispatch_pane> --direction down --cwd . --main-root .
./.bee/bin/bee herding pane tab-create --cwd . --label runtime --main-root .
./.bee/bin/bee herding pane run <cockpit_dispatch_pane> "'./.bee/bin/bee' herding control-loop --role dispatch --main-root '.'" --main-root .
# (no merge loop started - D11: merge is a single-shot owner gesture, run in the merge pane on request)
```

## D — real `bee herding run` (pane %1 → worker pane %3)

First attempt, phase-2 binary:

```
$ bee herding run --task "Write the result file … status done …" --idle-timeout 240 --ceiling 480 --json
{"job_id":"job-1787452964461","outcome":"spawn_failed","pane_id":"%3",…,
 "error":"agent never reported ready within 60s (ready-wait) — pane kept for inspection — pane %3 … shows what looks like an unanswered prompt: \"❯\" …"}
```

The worker DID start: pane %3 showed Claude Code v2.1.241 idle at its `❯`
prompt with "bypass permissions on", and two captures 3 s apart were
byte-identical (`diff` → STABLE); `pane list --with-status` read `%3` as
`idle`. The ready gate still timed out. Cause (`run.rs:68`
`POLL_INTERVAL = 200 ms`, ready loop at ~2046): the gate calls
`agent_wait(job, 200)` repeatedly, and `RealTmux::agent_wait` restarted its
3 × 2000 ms stability window on every call, so no call could ever answer
`idle`. Fixed by `tmux-ready-wait` D1 (stability state kept per pane across
calls). Second attempt, fixed binary (commit 827373d):

```
$ bee herding run --task "Write the result file … status done and summary hello-from-tmux …" --idle-timeout 240 --ceiling 480 --json
{"job_id":"job-1787453627146","outcome":"done","pane_id":"%4","closed_pane":true,"dry_run":false,
 "summary":"hello-from-tmux","files_changed":[],"proof":"task requested only writing the result file …"}
$ cat .bee/mailbox/job-1787453627146/ack-1.json
{"nickname":"job-1787453627146","job_id":"job-1787453627146","round":1,"agent":"claude-code","received_at":"2026-08-23T02:54:04Z"}
$ cat .bee/mailbox/job-1787453627146/result-1.json
{"status":"done","summary":"hello-from-tmux","files_changed":[],"proof":"…"}
```

The worker pane %4 was split off the caller's pane, Claude Code booted,
the ready gate passed, the one-line pointer was typed in, the worker wrote
`ack-1.json` then `result-1.json` through the mailbox, and the pane was
closed on the `done` outcome. The whole round trip ran with no human at the
keyboard and no herdr process.

Takeaways:
1. A settle-aware wait that a caller polls with a short timeout must keep its
   own state between calls — herdr does this server-side; tmux has no server
   state, so the backend must.
2. The give-up diagnosis reads `❯` as "an unanswered prompt". On Claude Code
   that glyph is the idle input prompt, not a question; the remedy text is
   misleading on tmux and names `herdr pane read` — worth a follow-up.
