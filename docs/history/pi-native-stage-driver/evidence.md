# Pi native stage driver — planning evidence

Verbatim bytes behind the `## Load-bearing claims` table in `plan.md`.
Two sources sit outside this repo — the installed Pi host, and a child process
this session ran — so their output is copied here and tracked, instead of being
cited at a path a reviewer cannot open.

Captured 2026-09-18/19 by the planning session, from
`/home/thanhsmind/Projects/goglbe/beehive--wt--pi-native-stage-driver`.
Pi host: `/home/thanhsmind/.local/share/mise/installs/pi/0.85.1/pi` (`pi` 0.85.1, via mise).

---

## A — A native child loads bee's belt

### A1 — the command that was run

```
timeout 240 pi --mode json -p --no-session --tools read \
  "Do not use any tool. Answer from your injected context only, in under 40 words: \
   does your context contain a bee session preamble naming a bee phase, feature or gate? \
   If yes quote the phase and feature verbatim. If no, say NONE."
```

Exit code `0`. Working directory was the feature worktree. Raw transcript kept
at `.bee/spikes/pi-native-stage-driver/child-stdout.jsonl` (62 lines) and
`child-stderr.log`; that directory is gitignored, so the load-bearing lines are
reproduced below.

### A2 — the belt ran inside the child (stderr)

This line is emitted by `.pi/extensions/bee-guard.ts`'s advisory wrapper. Nothing
else in the child writes it, so its presence proves the extension loaded:

```
bee: hook prompt-context could not decide this payload — allowing the operation (fail-open).          The guard did NOT run on it.
```

Note the second half: one `prompt-context` call could not decide and failed open.
That is the belt's documented ADVISORY policy working, not a failure — but it
means a native worker must not be assumed to carry a decided prompt context.

### A3 — bee's preamble reached the child (stdout, `message_end`)

```
"text":"Yes.\n\nPhase: `idle` | Mode: `none`  \nFeature: `pi-native-stage-driver`  \nGates: `none pending (no active work)`"
```

The child named this feature without being told it. The preamble is injected.

### A4 — stdout is JSONL behind non-JSON noise

First two lines of the transcript, in order:

```
mise ~/.config/mise/config.toml tools: pi@0.85.1
{"type":"session","version":3,"id":"01a0b519-dfae-754f-9de0-d5aeb8b634ee","timestamp":"2026-09-18T15:19:26.895Z","cwd":"/home/thanhsmind/Projects/goglbe/beehive--wt--pi-native-stage-driver"}
```

Line 1 is not JSON. A parser must skip unparseable lines rather than abort.
The session header carries an `id` and the `cwd` — both useful for naming a seat.

### A5 — the child settles and reports cost

From the final `message_end`, then the last line of the transcript:

```
"usage":{"input":13924,"output":114,"cacheRead":0,"cacheWrite":0,"reasoning":76,"totalTokens":14038,"cost":{"input":0.001233749944,"output":0.000020202168,"cacheRead":0,"cacheWrite":0,"total":0.001253952112}},"stopReason":"stop"
```

```
{"type":"agent_settled"}
```

A per-worker budget is therefore enforceable from the child's own output.

---

## B — Pi 0.85.1 host API

### B1 — the spawn shape (`examples/extensions/subagent/index.ts:300-307`)

```ts
	const args: string[] = ["--mode", "json", "-p", "--no-session"];
	const inheritsDispatchConfig = !agent.model;
	const model = agent.model ?? dispatchDefaults.model;
	if (model) args.push("--model", model);
	if (inheritsDispatchConfig && dispatchDefaults.thinkingLevel) {
		args.push("--thinking", dispatchDefaults.thinkingLevel);
	}
	if (agent.tools && agent.tools.length > 0) args.push("--tools", agent.tools.join(","));
```

Same file, `:344-351`:

```ts
		const exitCode = await new Promise<number>((resolve) => {
			const invocation = getPiInvocation(args);
			const proc = spawn(invocation.command, invocation.args, {
				cwd: cwd ?? defaultCwd,
				shell: false,
				stdio: ["ignore", "pipe", "pipe"],
			});
			let buffer = "";
```

No `--no-extensions` appears in that flag set, which is consistent with A2: the
child loads project extensions.

### B2 — `setActiveTools` narrows (`docs/extensions.md:1692-1694`)

```
const extensionTools = all.filter((t) => t.sourceInfo.source !== "builtin" && t.sourceInfo.source !== "sdk");
pi.setActiveTools([...new Set([...active, "my_custom_tool"])]); // Keep current tools and enable my_custom_tool
pi.setActiveTools(["read", "bash"]); // Switch to read-only
```

### B3 — the additive-only rule is scoped to a loader tool (`docs/extensions.md:2373-2375`)

```
1. Register every tool with `pi.registerTool()` so it appears in `pi.getAllTools()`.
2. Keep loader tools, such as `search_tools`, active and leave searchable tools inactive.
3. During loader execution, call `pi.setActiveTools([...currentTools, ...matchingTools])`. The change must be additive: do not remove currently active tools in the same call.
```

The constraint binds step 3 — a loader tool's own execution. An event handler is
not a loader tool, so narrowing from one is legal, and B2 shows the narrowing form.

### B4 — `agent_settled` is the settle event; `session_stop` does not exist

`docs/extensions.md:569`:

```
`agent_start` fires when a low-level agent run begins. `agent_end` fires when that run ends, but Pi may still auto-retry, auto-compact and retry, or continue with queued follow-up messages. Use `agent_settled` for status integrations that need to know Pi will not continue running automatically.
```

`grep -c 'session_stop'` over that same file returns `0`. The `refs/oh-my-pi`
mirror documents `session_stop` and `ctx.setInterval`; that mirror is a FORK and
does not describe the installed host.

---

## C — The stale approved packet (claims 13a-13c)

### C1 — the door refuses

```
.bee/bin/bee dispatch prepare --runtime claude --kind advisor --json
→ {"ok":false,"type":"refused","reason":"stage_required","feature":"pi-native-stage-driver", ...}

.bee/bin/bee dispatch prepare --runtime claude --kind advisor --stage read-only-gather --json
→ {"ok":false,"type":"refused","reason":"stage_not_applicable","feature":"pi-native-stage-driver","stage":"read-only-gather", ...}
```

### C2 — the stored packet belongs to a different feature

Reading `.bee/state.json`:

```
packet.feature     : release-2-41-2
packet.previewed_at: 2026-09-18T02:02:35.752Z
state.feature      : pi-native-stage-driver
```

And that packet's role plan classifies the research stages away, because
`release-2-41-2` was a tiny release lane:

```
read-only-gather -> {"stage": "read-only-gather", "classification": "not-applicable", "role": "read", "reason": "Nothing to survey; the preconditions are the script's own."}
generic-advisor  -> {"stage": "generic-advisor", "classification": "not-applicable", "role": "advisor", "reason": "No high-risk consult is owed for a publish."}
hat-risks        -> {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "Tiny lane: no hat wave."}
```

### C3 — the lookup never compares the packet's own feature

`packages/bee-rs/crates/bee/src/verbs/state_group/plan_packets.rs:641-649`:

```rust
    let state_file = root.join(".bee").join("state.json");
    if let Ok(text) = std::fs::read_to_string(&state_file) {
        if let Ok(Value::Object(m)) = serde_json::from_str(&text) {
            if m.get("feature").map(js_disp).as_deref() == Some(feature) {
                if let Some(Value::Object(p)) = m.get("approved_cell_packet").or_else(|| m.get("gate_preview")) {
                    return Some(p.clone());
                }
            }
        }
    }
```

The guard is `state.feature == requested feature`. The packet's own `feature`
field is never read, so any packet left in `state.json` is inherited by whatever
feature is active next. `bee state start-feature` resets the four gates but does
not clear `approved_cell_packet` or `gate_preview`.

---

## D — The write guard ENFORCES inside a child, not only loads

Section A proved the belt *loaded* in a child. It did not prove enforcement: that
probe ran with `--tools read` and an explicit instruction not to use a tool, so no
write was ever attempted. Claim 1 was narrowed to "loads" for exactly that reason,
and the plan carried the enforcement question as open. This section closes it.

Run 2026-09-19 from the feature worktree, by the leader.

### D1 — picking a target that denial cannot be explained away

Gate 2 was approved by then, so ordinary source writes are legal and would prove
nothing. Probing the parent guard for a path denied regardless of phase:

```
.env                                exit=0
.env.local                          exit=0
secrets.json                        exit=0
id_rsa                              exit=0
.agents/skills/bee-hive/SKILL.md    exit=0
.claude/skills/bee-hive/SKILL.md    exit=0
.bee/state.json                     exit=2   bee direct-edit guard: ".bee/state.json" is CLI-owned — direct edits are blocked in every phase.
.bee/decisions.jsonl                exit=0
```

`.bee/state.json` is the one target whose refusal cannot be attributed to gate
state. (Worth noting separately: the secret-shaped paths above were all allowed
by the guard at this phase.)

### D2 — the child was refused, in the guard's own words

The child was given `--tools write` and told to replace that file. From its
transcript, the tool result:

```
"text": "bee direct-edit guard: \".bee/state.json\" is CLI-owned — direct edits are blocked in every phase. Hand-edited state files reintroduce schema drift (the exact class the CLI validates away). FIX: use bee state set --owner <selected pre-mutation phase>, or the dedicated state gate/worker/scribing-run verb instead of editing this file directly."}], "details": {}, "isError": true
```

and the child's own reply:

```
"text": "REFUSED bee direct-edit guard: \".bee/state.json\" is CLI-owned — direct edits are blocked in every phase. …"
```

### D3 — and the file was untouched

A second run asserted on content rather than on a hash:

```
### did the payload land?
  the child's payload is NOT in the file

### is it still valid bee state after?
  valid json, feature = pi-native-stage-driver | keys = 12

### what changed in the file, if anything
  identical once last_activity is ignored
```

**Conclusion, stated at its true scope: a BARE child `pi` process — one spawned
without bee's herding worker marker — inherits the write guard and is refused by
it. The spawn itself does not leak past the guard.**

That is NOT the same as saying a no-pane WORKER is guarded, and the difference
matters. A worker started through `bee herding run` carries
`BEE_HERDING_WORKER=1`, and `packages/bee-rs/crates/bee/src/hooks/mod.rs:144-155`
short-circuits every hook except `activity` under that marker, with the reason
stated in the code: *"the worker's posture is already fully-open (herding-adopt
D7), so it gets zero bee preamble, zero guards, zero nudges."* So a no-pane
worker has no write guard — and neither does a pane worker. That is existing,
deliberate herding posture, identical on both transports, which is exactly the
parity D11 requires. This feature does not change it in either direction.

The probe above therefore proves the narrow thing it can prove: spawning a child
does not by itself create an unguarded hole. What makes a worker unguarded is the
marker, not the transport.

### D4 — a false alarm worth recording

The first version of this probe judged the outcome with `sha256sum` on
`.bee/state.json` and printed *"FILE CHANGED — the guard did NOT enforce in the
child"*. That was wrong: bee writes `last_activity` into that same file on a
heartbeat, so the hash moved on its own while the child ran. A whole-file hash is
not a valid assertion against live state the system mutates itself. The D3 form —
is the attacker payload present, is the file still valid, is it identical once the
known-volatile field is excluded — is the one that answers the question.
