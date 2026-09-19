# Pi native stage driver — hat wave synthesis

Plan-step wave, run 2026-09-19 on runtime `claude` against
`docs/history/pi-native-stage-driver/plan.md` at `995fe0bd5`.

Seats dispatched: 5 (high-risk). **Returned: 5** — `hat-facts-gaps`,
`hat-risks`, `hat-alternatives`, `hat-value`, `hat-user-impact`. None dropped.

Correction on this record: an earlier revision of this file named `hat-risks`
dropped for missing the 10-minute budget. That was wrong. It returned in 305 s,
well inside the ceiling; it was simply the last to land, and the leader wrote
the synthesis before it arrived. The seat is fully counted below, and it carries
the wave's most serious findings.

Note on the budget: the door issued the two pane seats `--ceiling 1800`, not 600.
The 600 s hat clamp at `prepare.rs:2479-2484` is guarded by `runtime == "pi"`, so a
`claude` wave runs to the configured herding ceiling. Filed P2. The leader held
the 10-minute rule by hand.

**Verdict: the plan is NOT gate-ready.** Three blockers, one wrong row, three
weak rows, and one alternative shape that is cheaper and needs no locked
decision superseded. Every finding below was re-verified by the leader against
the code before it was accepted — the seat's claim alone was not enough.

---

## Accepted blockers

### B1 — The cell's proof cannot prove the cell (from `hat-facts-gaps` G2)

`pnsd-2`'s `verify` is `cargo test … --test pi_plugin_contracts`. That suite
derives `pi.on(` events, `registerCommand(` names and `PI_BUILTIN_TOOLS`. It
does **not** derive registered tools at all.

Leader verification: `rg -n 'registerTool' packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs`
returns exactly one hit, line 35, and it is inside a comment. So the suite goes
green whether or not the new tool exists, and the cell's instruction to "write
the contract test rows first and watch them fail" is unsatisfiable. `pnsd-4`
carries the same verify string while claiming `green:live`.

Consequence: `pnsd-2` and `pnsd-4` have no honest proof line as written.

### B2 — The native arm skips three Pi-only branches (from `hat-facts-gaps` G1)

Leader verification: `rg -n 'runtime == "pi"' …/prepare.rs` returns five sites —
`:2184`, `:2237`, `:2464`, `:2480`, `:2524`. The refusal the plan targets is
`:2237`. But `--seat` is appended at `:2464`, the 600 s hat clamp is applied at
`:2480`, and `detached_delivery` is added at `:2524` — all three inside the
**herding payload branch**.

A native arm added at `:2237` reaches none of them. D10's ceiling, seat naming
and drop-and-name would have to be rebuilt from scratch on the new path, and no
cell, must-have or test row says so. The D9 byte-equality test guards only
claude and codex; nothing guards Pi's own three branches.

### B3 — D3's "is a pi binary" rule is under-specified and the config defeats it (from `hat-facts-gaps` G3)

Leader verification of `.bee/config.json` `herding.agents`: two shapes coexist.

```
pi-gpt-5.6-luna        ARRAY   argv0='pi'
agy-flash              DICT    argv0='agy'     (argv nested under "argv")
```

`pnsd-3`'s action says "the first argv element being `pi`" without naming the
shape. `agents[name][0]` is correct only by luck today and misclassifies any pi
agent written in object form. The native arm must also lift the model out of
that argv (`["pi","-a","--model","openai-codex/gpt-5.6-luna:high"]`) — no row,
no must-have.

---

### B4 — A belt-spawned child runs with full LEADER posture (from `hat-risks` P1-B)

Leader verification: `herding/run.rs:2499` puts `BEE_HERDING_WORKER=1` into the
pane environment, and `hooks/mod.rs:91` plus `:145-146` make **every** hook
invocation exit 0 under that marker before stdin is read — every name except
`activity`.

A child spawned by the belt carries no such marker. So inside it `session-init`
runs for real and registers a live acting session, one per seat; the belt maps a
fresh session's reason to `"clear"` (`bee-guard.ts:553-556`), which is an ADOPT
source, so several hat seats can race to adopt the leader's carried handoff
claim. `session-close` runs too, and a block verdict calls `pi.sendUserMessage`
inside a `-p` child, opening a second turn against the ceiling. Evidence A3
already proves `session-init` ran in the probe child.

The plan never picks a worker posture. The herding path picked one years ago.

### B5 — The belt tool contradicts the belt's own recorded exclusion (from `hat-risks` P1-A)

Leader verification, `.pi/extensions/bee-guard.ts:52-59`:

> `model-guard is a NAMED EXCLUSION on this belt — n/a — Pi has NO native
> subagent surface: no Agent tool, no Task tool, no subagent_type parameter
> anywhere in its built-in tool registry (store decision 7f9c8518) … Every
> worker dispatch from a Pi session routes through the herding transport
> instead, which is a bee CLI call (\`bee herding run\`) and therefore already
> covered by write-guard on the \`bash\` tool. … the exclusion is asserted BY
> NAME in the belt parity test.`

And `tests/pi_plugin_contracts.rs:1804-1810` enforces it: any tool routed to a
hook other than `write-guard` is a gap, with the message *"the Pi belt's only
BLOCKING destination is write-guard (model-guard is a NAMED EXCLUSION — Pi has
no subagent surface, store 7f9c8518)"*.

`pnsd-2` makes that premise false. A registered spawner whose `model`, `tools`
and `cwd` come from the model is exactly what model-guard exists to stop, and
D8 forbids the leader picking a model. So the belt-hosted shape owes: a
model-guard route on the Pi belt, a rewrite of that assertion, plus the posture,
env, seat, ceiling and drain machinery of B2 and B4.

The same paragraph states why the alternative shape is already safe: a
`bee herding run` call **is** a bash CLI call, and is therefore already covered.

### B6 — Environment inheritance makes a child act as its parent (from `hat-risks` P2-D)

Pi's own precedent spawns with no `env` override (evidence § B1), so the child
inherits everything, including the session identity variables bee reads first.
A child's own `bee` CLI calls would then act as the PARENT session. Herding
builds a per-agent environment explicitly (`run.rs:2482-2500`); the plan says
nothing. Accepted into the rework either way.

Also accepted from `hat-risks`: test-matrix row 9 was never exercised — the probe
ran with `--tools read` and an explicit "Do not use any tool", so a denied write
inside a child is still unproven (this is the same defect as claims row 1); and
with no agent name a native writer reaches `Allow` by default
(`write_guard/checks.rs:638-643`), so two native writers on one file collide
silently.

## Accepted corrections to the claims table

| Row | Verdict | Correction owed |
|---|---|---|
| 1 | **WEAK — the leader's inference was wrong** | The bytes end `The guard did NOT run on it`. They prove the extension **loaded**; they do not prove it **enforces**. `plan.md` and test-matrix row 9 both rest on enforcement. Needs its own deny-path run, or the claim must be narrowed to "loads". |
| 9 | WEAK | A second `pi_requires_herding_refusal` call sits at `prepare.rs:2184-2185`. "One place" is true of the refusal, false of the payload. |
| 10a | WEAK — wrong anchor | The byte-compare is at `doctor.rs:305`, not `:310-312` (those are the two detail strings). Conclusion holds. |
| 12 | **WRONG — fabricated quote on a `ran` row** | The "verbatim" string was the leader's own hand-composed summary, not bytes from `bee team show --runtime pi --json`, which prints `model` and `transport` on separate JSON lines. It also silently dropped the `deploy` role. Conclusion holds; the evidence must be re-quoted from real bytes. Reflection recorded. |
| 13c | **WRONG** | `get_approved_preview_packet` has **two** return sites, and the lane-file branch at `:633-639` compares no feature at all. "The fix has one home" is false — the cell's own action already says "two return sites", so the row contradicts its own cell. |

Also accepted, minor: the belt has **12** `pi.on` handlers, not five
(`bee-guard.ts:2073, 2098, 2136, 2183, 2201, 2223, 2248, 2305, 2403, 2407, 2414, 2434`);
`pnsd-2`'s prohibition and CONTEXT.md's code-context section both undercount, which
lets a worker rationalize editing the seven nobody named. And `pnsd-3` lists
`config.rs` in the preview table but not in its JSON `files`, so a `config.rs`
edit would run unreserved and the "parallel, no overlap" judgment was made on the
wrong set.

---

## The alternative shape (from `hat-alternatives`)

**Herding minus tmux.** Put the child spawn in Rust, inside `bee herding run`:
same job id, same inbox marker, same `--seat`, same mailbox report, same drain —
but spawn `pi --mode json -p …` as a child instead of splitting a tmux pane. The
door's payload shape does not change at all; only the runner beneath it does.

Why it is cheaper, checked against B1-B3:
- It deletes the plan's only HIGH risk — no new tool in the guard belt, so no new
  `mapToolCall` row, no `doctor` byte-compare rebuild gate, and **B1 stops
  existing** (there is no belt surface needing a contract fixture that the suite
  cannot derive).
- **B2 stops existing**: `--seat`, the 600 s clamp and the drain are already on
  that path, shipped and `green:live` by `pi-stage-dispatch`.
- B3 remains, and must be solved either way.

Locked decisions it needs superseded: **none**. D2 requires a child `pi`
subprocess; it does not say who spawns it. D8's one door holds. D1 and D3 hold.

It conflicts with one thing only: the owner's explicit choice on 2026-09-18 to
"grow bee's own Pi extension", made before this evidence existed. That is a
decision, so it returns to the owner rather than being changed here.

The belt-hosted tool earns its keep only if the leader must receive the full text
**inside its turn** without a file read. Nothing in CONTEXT.md or plan.md states
that requirement today.

---

## Value and user-impact findings

From `hat-value`:
- `team.pi` routes `code`, `read`, `test`, `docs`, `extraction`, `generation`,
  `supervisor` and `lane-3` to `agy-flash` — not a `pi` binary. Under D3 those
  roles stay on panes, so **execution cells get no benefit**. The native path
  helps advisor, review, plan, lane-1/2 and hat seats only.
- `pnsd-2` alone changes nothing a user can see; value needs `pnsd-3` with it.
  The plan already groups them, so this confirms the slice rather than changing it.

<!-- bee:not-a-deferral: The flagged phrase is "deferred loading", the name of a Pi runtime feature, quoted verbatim from the installed host's docs/extensions.md. A technical term, not a deferral of work. -->
From `hat-user-impact`, on D4's hard tool gate:
- Removing tools is not purely additive, so Pi drops deferred loading and the
  provider's cached prompt prefix **may** be invalidated. Leader verification at
  the host's `docs/extensions.md`, § "Fallback behavior": *"Pi also uses this safe
  fallback when the active set is not purely additive… Tool removals therefore
  work, but they do not use deferred loading."* and *"For the best cache behavior,
  keep the loader tool active for the whole session and add tools instead of
  replacing the active set."*
- The model is never told **why** a tool vanished, so it apologizes, hallucinates,
  or falls back to `bash` redirection — which then trips the write-guard. The
  failure is not legible to the user.
- D5's warning on `agent_settled` via `ctx.ui.notify` is an ephemeral toast, and
  in `-p`/JSON mode `ctx.hasUI` is false and notify is a no-op. The warning must
  land in the visible transcript instead.
- Remedy accepted into the rework: name the re-open command explicitly, announce
  it where the narrowing happens, and put D5's warning in the transcript.
<!-- /bee:not-a-deferral -->


---

## What happens next

1. The shape question goes back to the owner — it contradicts their own recorded choice.
2. D4's hard gate goes back with it: the owner chose it to stop wasted turns, and
   the cost is cache fallback on every stage change.
3. Whichever shape wins, B3 and the five claims-table corrections are owed, and
   `hat-risks` owes a fresh pass on the trust boundary before the gate.
