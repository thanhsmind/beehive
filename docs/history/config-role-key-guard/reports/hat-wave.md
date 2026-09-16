# Hat wave — config-role-key-guard

Plan-step wave, three default seats, one pass. Draft under critique:
`docs/history/config-role-key-guard/plan.md` (revision before this record).

Seats dispatched through `bee dispatch prepare --runtime claude --kind advisor`
— `general-purpose` at the `fable` advisor tier, the payload the door returned.
All three returned inside the ten-minute ceiling. No seat dropped.

## Synthesis — what changed the plan

Five findings moved the shape. The leader accepted all five; nothing was
overruled.

### 1. The guard's own off-switch was writable (`hat-facts-gaps`)

`hooks.write-guard: false` is an ordinary config key, read at
`hooks/write_guard/main.rs:88` through `state.rs:194-200`. An agent that wants a
different model never has to beat the arm — it switches the guard off first.

This is the finding that would have made the whole feature theatre. The
projection now covers the `hooks` subtree, and that part takes no add/change
carve-out: any change refuses, in either direction.

### 2. The arm would never have fired on Pi (`hat-user-impact`)

Pi's edit tool reaches the hook as `MultiEdit` carrying an `edits[]` array, not
as `Edit` with top-level strings (`hooks/write_guard/tests.rs:1518-1523`). The
draft wired `Write` and `Edit` only — so it would have missed the exact runtime
in the motivating incident, while the user believed a guard existed.

The call site now covers all five write surfaces. `apply_patch` and Bash cannot
be projected, so they refuse outright and name `Edit`/`Write` as the way in —
a redirect, not a hole.

### 3. The model lives one layer further up (`hat-user-impact`)

`team` names an agent; `herding.agents.<name>` holds that agent's argv, and the
argv is where `--model` is:
`"pi-opencode-free": ["pi","-a","--model","opencode/x-preview-f-free:high"]`.
An agent that leaves the team table alone and rewrites the argv gets the
identical outcome. `herding.agents` and `herding.agent_command` are in the
projection.

### 4. bee's own refusals instruct the write the arm would refuse (`hat-facts-gaps`)

`verbs/drivers/prepare.rs:212` and `:1976` tell the agent, in their FIX text, to
add a `team.<runtime>.<slot>` entry to `.bee/config.json`. An arm that refuses
adds contradicts bee's own instructions.

Settled as the add/change boundary: an ADD passes, a CHANGE and a REMOVAL
refuse. The residual gap — an agent can still pick the model for a role that had
no slot — is recorded in the plan's `## Recorded gaps` and filed to the backlog
rather than folded in.

### 5. The refusal must name the change (`hat-user-impact`)

The draft refused "when the two projections differ" and said nothing about
naming the differing key. The seat's litmus: a refusal the agent can turn into
one line the user acts on is protection; a bare wall is friction — especially
with no `bee team set` to point at. The arm now reports the first differing
address and both values.

## SMALLER PATH verdict (`hat-alternatives`)

**NO CHEAPER SHAPE.** The seat weighed three alternatives the draft had not:

- Dispatch-time re-read with a session-start baseline — fails D1's letter (the
  edit lands; only a later dispatch complains), and needs a stored baseline plus
  a compare plus a way to tell the user's edit from the agent's.
- A real `bee team set` verb plus the existing `direct_edit_verb` mechanism —
  `direct_edit_verb` is whole-file, so it fails D2, and it is a new verb AND
  still the same key-scoped arm.
- An onboard-time checksum of the team subtree — fails D1 the same way as the
  first, and adds an onboard write, a dispatch compare and a re-attest path.

It also confirmed the draft's own two rejections (path-only deny, doctrine-only
line) were real alternatives honestly knocked down, not straw men.

## Claims-table audit (`hat-facts-gaps`)

Three rows failed the verbatim rule — evidence paraphrased or reflowed onto one
line rather than quoted as the file holds it (rows for `guards.rs`, `tests.rs`,
`checks.rs`). Substance held in all three; every row was requoted from the
bytes. Two rows were missing a line anchor. One prose claim — "the model guard
watches `Agent`/`Task` and nothing else" — was wrong (it also watches Codex
`spawn_agent`, `model_guard.rs:59,65`) and was cut from the Summary rather than
promoted to a row, since the plan does not rest on it.

## Cell split

The seat called the original `crkg-1`/`crkg-2`/`crkg-3` split "one cell
pretending to be three": the red-first test cannot go green until the wiring
lands, and a red proof refuses a cap. Collapsed to one cell with red-first
inside it.

## Unresolved truth-table rows carried into the shape

The `hat-facts-gaps` Truth Table Test named nine input combinations the draft
left unstated. All nine are answered in the revised `## Shape`: null vs absent,
old absent, old unparseable, table removed, role added, role removed, `team`
and `models` both present, neither present, bare string vs object, and `Edit`
with `replace_all`.
