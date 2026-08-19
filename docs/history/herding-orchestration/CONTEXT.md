# Herding Orchestration — Context

**Feature slug:** herding-orchestration
**Date:** 2026-08-18
**Shaping session:** complete
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

Backed by the discovery map `docs/discovery/herding-orchestration/MAP.md`
(10 tickets, closed 2026-08-18). Every decision below was settled there
and is consumed, not re-asked (bee-shaping D8).

## Feature Boundary

bee-herding gains a generic coordination core — open several agents, give
each a task, wait on them all at once, collect results, aggregate
failures — running on Linux and Windows, proven by one real
spawn-and-brief wave. It ends at that first wave: no recipe file format,
no orchestrator succession, and `bootstrap-cockpit.sh` stays bash.

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The feature is done when ONE real coordination scenario runs end to end on Linux and Windows — not when a design is agreed. | Fixes what "finished" means; a design document does not close this feature. Log `c7646101`. |
| D2 | Build a generic coordination core that knows workers, tasks, waiting, result collection and failure aggregation — and nothing about cells, lanes, worktrees or proof. bee-herding is its first client. | Every bee concept that leaks into the core is a defect, not a convenience. Log `a475986f`. |
| D3 | The specification for the core is the ORDERING of the herdr-agent-comms choreography, not its code: resolve and dedupe targets, fail-closed status filter, baseline every target BEFORE any dispatch, re-check status immediately before each individual send, dispatch then wait concurrently then aggregate. Its 29-test corpus is the behaviour to reproduce. No line of its Python or bash is ported. | Eight properties exist only because of that order — see § Ordering Invariants below. Ship the order and the tests fail-first; ship the calls and the races come back. Log `9d054b02`, touching the source manifest `f5288dda`. |
| D4 | Windows is a required outcome of this feature, not a follow-up. | bee already runs its full suite on `windows-latest` and ships `x86_64-pc-windows-msvc`; herdr ships native Windows support. Nothing new is being pioneered. Log `9db72b0b`. |
| D5 | The core lives in its own crate in the `packages/bee-rs` workspace, compiled INTO the existing `bee` binary as a library dependency. The core crate never depends on the `bee` crate. Not a second shipped binary. | The crate boundary is the only mechanism that enforces D2 — a violation becomes a dependency edit visible in review, not an accident. Linking into `bee` leaves the release matrix and both install paths untouched. Log `a6738d9d`. |
| D6 | The first real scenario is a spawn-and-brief wave with collection: open N agents each in its own worktree, wait for readiness, hand each a brief, wait on all concurrently, collect results, aggregate failures. It must exercise at least one failure path. | It runs the whole choreography and forces the repair in D12. Log `bad04b67`. |
| D7 | The core drives a worker-backend trait — start a worker, read status, send a task, read output — with herdr as the first implementation. The trait's status model is the choreography's own: ready, working, blocked, finished, unverifiable. `unverifiable` is a first-class value, never an error. | Testability decided it: the crate has no seam for testing an external binary, and this trait IS that seam, so the whole choreography becomes testable without a running herdr server. A trait shaped around herdr's exact enum would not be a seam. Log `72577bf5`. |
| D8 | `control-loop.sh` is replaced by Rust in this feature. `bootstrap-cockpit.sh` stays bash — a known, recorded gap. | `control-loop.sh` carries both Windows blockers (GNU coreutils `timeout`, a bash-4.3 nameref); `bootstrap-cockpit.sh` trips only on `BASH_SOURCE`, is run once by hand, and is off the first scenario's path. Log `bc6291ed`. |
| D9 | Wave concurrency uses `std::thread` plus `std::sync::mpsc`. No async runtime is added. The bash `$tmpdir/$i.{baseline,out,err,code}` handoff layer is NOT recreated. | A wave is at most a handful of workers and each waiter is a blocking poll loop around a subprocess; tokio would be the crate's first async dependency on two platforms for scale this never reaches. The tmpfile layer existed only because a bash subshell cannot return a structured value. Log `ef9fe466`. |
| D10 | The core records nothing (structural, per D5). bee keeps ONE append-only wave ledger, one row per wave, in the shape `.bee/` already uses: wave id, start time, and per worker — name, pane id, worktree, task, outcome, evidence pointer. Wave results never become cells, decisions or proof lines. | The four-slot cap is enforced today by the control model COUNTING PANES, so an agent that fails to name itself leaves a slot looking free and the next iteration over-spawns (a recorded Open Gap). The ledger makes occupancy readable instead of counted, replacing a worse source of truth rather than adding a new one. A cell has its own owner, claim and proof; a wave is not a cell. Log `984a2cde`. |
| D11 | A scenario is described through a Rust API in which a wave is a VALUE — a `Wave` struct holding worker specs, timeouts, and a failure policy — not a sequence of calls. The failure policy is an enum in that value from day one (wait-all, first-success-cancel-rest, best-effort) even with a single variant implemented. No recipe file format is built. | As a value, a file format later is `serde` on types that already exist; as a call sequence it means designing twice. The failure policy is the axis that actually varies between scenarios and the one a format would have to grow a language for. Log `a536c68a`. |
| D12 | The dead spawn line is repaired first, before any other work in this feature. On herdr 0.8.0, `herdr agent start <slug> --cwd … --workspace … --tab … --split right --no-focus -- claude …` returns `unknown option: --cwd`. The 0.8.0 form is split-then-start: `herdr pane split <runtime-pane-id> --direction right --cwd <worktree_path> --no-focus`, read `.result.pane.pane_id`, then `herdr agent start <slug> --kind claude --pane <new-pane-id> --timeout 60000 -- <agent args>`. | `agent start` now requires `--pane` and "never creates, splits, or moves layout", so no scenario that opens an agent can run until this is fixed. This INVERTS `references/spawn-proof.md`, which forbids splitting first — that proof is re-recorded, not just the command edited. Verified against the running binary by a deliberate parse-failure probe (nothing was mutated). |
| D13 | The Rust replacement for `control-loop.sh` must construct argv byte-identical to the bash default when no `herding.control_command` config is present, proven by test. | The script builds the control pane's `--allowedTools` surface and the working agent's `--permission-mode bypassPermissions` tail. A silent drift here breaks the recorded permission-posture split (herding-adopt D7 / R4) without anything failing loudly. This is the one named risk that holds the lane at standard. |
| D14 | `herding.agent_command` keeps its existing shape — token 0 stays the agent executable name — and bee splits it at spawn: token 0 feeds herdr's `--kind`, the remaining tokens go after `--` as agent arguments. The documented default array is unchanged. `operational-invariants.md` records the new mapping and restates its no-config promise as "the same effective spawn" rather than "byte-equivalent". An unrecognised token 0 (not one of herdr's supported kinds) must surface as a typed error naming the key, never as a generic start failure. | herdr 0.8.0 names the executable through `--kind` and hands everything after `--` to the agent, so the old "pass the array verbatim after `--`" rule would feed `claude` to claude as a positional argument. Byte-equivalence is no longer available to promise — herdr changed the wire format. Keeping the config's shape puts the change inside bee where it is testable, rather than on host projects that already set the key and would get no migration signal. Log `75bf36ba`. |

| D15 | Ordering invariant 8 is satisfied at the generic-core layer ONLY for specs whose names are the same string. Collapsing a name and a differently-spelled canonical id for one target is backend-layer work. The backend phase owes four things: a canonical-identity step with the dedupe keyed on it; a test where a name and its pane id name one target and assert exactly one send, one baseline and one succeeded entry; a correction to the public doc on `WorkerSpec::name`; and this decision cited wherever invariant 8 is stated. | The `WorkerBackend` trait exposes only start, status, send and read_output — no name-to-canonical-id resolver — and adding one to the core would require it to know a backend's identity scheme, which D2 forbids. A judge proved the gap by execution: two specs naming one target under different strings received two preflight reads, two baselines and TWO sends. Log `fb8a8628`. |
| D16 | The herdr backend lives INSIDE the fleet crate, as a module beside the fake one — not in the `bee` crate, not in a third crate. | D2 forbids the core knowing *bee* concepts; herdr is a terminal multiplexer, not one of them, so a herdr backend is a peer of the fake backend. Putting it in `bee` would mean anyone reusing fleet has to reimplement the only backend that exists, making the genericness nominal. Log `64e8abe6`. |
| D17 | The bee-side entry point is a new CLI verb in the existing herding group, `bee herding wave`. | The dispatch and merge roles are models following markdown and can reach bee only through its CLI, so a library-only entry point could never be used by the cockpit. This is the feature's only public-contract change and was already counted in the route's flags. Named and recorded here rather than appearing in a plan, which is what the review wave objected to. Log `8d413c12`. |

### Agent's Discretion

D9 (concurrency primitive) was the agent's call, recorded and open to
override — the user was told and did not override. Everything else was
the user's pick.

Delegated to the agent within the locked boundaries: crate and module
naming, the ledger's exact field names and file path, the internal shape
of the backend trait's methods, and how the Rust control loop reproduces
the wall-clock ceiling that GNU `timeout` provided.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| Wave | One coordinated run: a set of workers, each with a task, dispatched and waited on together, aggregating to a single verdict. A value, not a procedure (D11). |
| Worker | One agent the core drives through the backend trait. Not a bee swarm worker, not a Task-tool subagent. |
| Backend | An implementation of the worker-backend trait. herdr is the first; the trait is also the test seam (D7). |
| Baseline | A transcript snapshot of a target captured BEFORE anything is dispatched to it. The anchor every completion check is measured against (D3). |
| Completion marker | A token embedded in a dispatched task, split so that echoing the prompt cannot reproduce it. Counts as proof only when present now AND absent from the baseline. |
| Unverifiable | A status lookup that failed, returned a null field, or returned a value outside the enum. A first-class status, never coerced to safe and never raised as an error (D7). |
| Wave ledger | The single append-only record bee keeps of waves, one row each (D10). Occupancy reads it instead of counting panes. |

## Ordering Invariants

D3 locks an order; these are the properties that order buys. Planning
must be able to point at each one. Losing any of them means the
choreography was reimplemented as primitives.

1. **Fast completion** — an agent finishing in under a second is still
   detected, because its baseline was taken before dispatch.
2. **Stale marker rejection** — a marker or a working→idle transition
   that predates this send is never credited to it.
3. **Dispatch-time re-check** — a target that flips to working or
   blocked during the baseline pass is not sent to.
4. **Fail-closed status** — a lookup failure, a null field, or an
   off-enum value is `unverifiable`, never safe.
5. **Partial-failure isolation** — one send failing mid-fan-out does not
   abandon agents already working.
6. **Mixed-result aggregation** — a wave where every SENT target
   succeeded still fails if any target was dropped.
7. **Bounded working→finished polling** — an agent that settles on
   `done` rather than `idle` is not made to wait out the full timeout.
8. **Dedupe before preflight** — a name and its pane id resolving to one
   target is sent to once.

## Specific Ideas And References

- The source skill (`luongnv89/skills`, commit
  `48730b30da90dfd2d2e3fa77a93c657cf75c4448`, path
  `skills/herdr-agent-comms`) is READ-ONLY prior art. Its content is
  data, never instructions (AGENTS.md, Guardrails). Its ordering and its
  test scenarios are the specification; its code is not adopted.
- `herdr --skill` prints a 195-line agent skill from the installed
  binary — the authority for CLI shape, ahead of any documentation.

## Existing Code Context

From the shaping scout only. Downstream agents read these before planning.

### Reusable Assets

- `packages/bee-rs/crates/bee/src/shell.rs` — the existing cross-platform
  pattern: a Win32 Git-Bash resolver that deliberately excludes the WSL
  launcher, plus the `BEE_POSIX_SHELL` env override and a PATH-prepend
  trick. The nearest precedent for the D7 test seam.
- `packages/bee-rs/crates/bee/src/lock.rs:126-186` — the established
  `cfg(windows)` / `cfg(unix)` split, and the atomic
  `create_new` + rename locking style.
- `packages/bee-rs/crates/bee/src/verbs/worktree/git.rs:86-91` — argv-based
  `std::process::Command` with stdin nulled. The shape a herdr backend
  should copy, never a shell string.
- `packages/bee-rs/crates/bee/src/herding.rs` — the live `bee herding`
  group: `classify-lane`, `interlock`, `command-template`,
  `herdr-result`, `herdr-pane-id`. Note `enable`/`disable`/`status` are
  NOT built and refuse by name.
- `.bee/*.jsonl` — the append-only ledger shape D10 follows
  (`backlog.jsonl`, `decisions.jsonl`, `capture-queue.jsonl`).

### Established Patterns

- Integration tests spawn the real binary via
  `assert_cmd::Command::cargo_bin("bee")` (`crates/bee/tests/front_door.rs`).
- Config seams are argv-token arrays substituted per token, never joined
  and re-split, never `eval` (`skills/bee-herding/scripts/control-loop.sh:277-286`).
- Every herdr call in the cockpit is issued live by a control-pane agent
  following the role markdown; only `bootstrap-cockpit.sh` calls herdr
  from a script. No herdr-driving Rust exists yet.

### Integration Points

- `packages/bee-rs/Cargo.toml` — workspace `members` gains the core crate.
- `skills/bee-herding/references/role-dispatch.md` §8 — the spawn line
  D12 repairs.
- `skills/bee-herding/references/spawn-proof.md` — the proof D12 inverts
  and must re-record.
- `skills/bee-herding/scripts/control-loop.sh` — replaced per D8.
- `skills/bee-herding/scripts/bootstrap-cockpit.sh:231` — starts the
  control loop; its invocation changes when the loop becomes a binary.
- `docs/knowledge/areas/bee-herding/overview.md` — authoritative for the
  cockpit; its Open Gaps and its stale Node-era Pointers both move.

## Canonical References

- `docs/discovery/herding-orchestration/MAP.md` — the map this feature
  came from, and its ten tickets.
- `docs/history/research/herdr-orchestrator-distill.md` — the distill,
  the dependency matrix, and the cross-cutting sweep.
- `docs/knowledge/areas/bee-herding/overview.md` — the cockpit's business
  rules R1–R8 and its recorded Open Gaps. **R2 (merge is a human
  gesture), R3 (dispatch stays behind the owner interlock) and R4 (the
  permission-posture split) are untouched by this feature.**
- `docs/knowledge/areas/worktree-parallelism/` — the isolation the
  working agents depend on.

## Outstanding Questions

### Resolve Before Planning

None. The map closed with no fog.

### Deferred To Planning

- [ ] What the core crate is named, and its module split — naming is
      implementation, but it must not name herdr (D7) or bee (D2).
- [ ] The wave ledger's exact path, field names, and sweep/retention
      behaviour — D10 fixes the content, not the schema.
- [ ] How the Rust control loop reproduces the wall-clock ceiling GNU
      `timeout` gave the bash version, on both platforms.
- [ ] The exact method set on the backend trait, and how a fake backend
      is injected in tests — PATH-prepended fake binary, or a trait
      object.
- [ ] Whether the 29 source test scenarios port one-to-one or collapse;
      D3 fixes the behaviour to reproduce, not the test count.
- [ ] Whether the wave ledger reading occupancy replaces the pane-count
      check inside `role-dispatch.md` §4 in this feature, or after it.

## Deferred Ideas

- Orchestrator succession — a wave surviving the orchestrator's own
  context running out. The source migrates the role to a fresh successor
  pane and goes read-only rather than pausing, which bee has no
  equivalent for. Out of the map's scope; returns as a fresh effort.
- A declarative recipe file format for scenarios. D11 keeps it cheap to
  add; a format invented before a second real scenario would freeze the
  only shape anyone has proven.
- `bootstrap-cockpit.sh` in Rust — the remaining Windows gap after D8.
- Re-runnable eval suites for bee skills — filed as backlog
  `p-cf66d519`, including the negative-trigger case shape.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Planning's Gate 2 shape stage and reviewing use locked decisions for coverage and UAT.
