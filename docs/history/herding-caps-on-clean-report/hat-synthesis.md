# Herding caps on a clean report — hat wave synthesis

Plan-step wave, 2026-09-19, against `plan.md` as first drafted (the runner
auto-cap). Seats dispatched: **3** — `hat-facts-gaps`, `hat-risks`,
`hat-alternatives`. All three returned.

Deviation, recorded in the plan's role assignments before the wave ran: 3 seats
rather than the 5 a high-risk lane asks for. `hat-value` and `hat-user-impact`
were marked not-applicable because the owner had already chosen the behavior
against a stated alternative with the measured failure rate in front of them, and
the only user-visible change was one output line whose wording was Agent's
Discretion. Nothing those seats would weigh was still open.

**Verdict: the drafted plan was not buildable, and the wave found why.** The
feature is re-shaped, not abandoned: the fix moves from the runner to the
instruction. Every finding below was re-verified by the leader against the source
before it was accepted.

---

## The finding that re-shaped the feature

**An execution worker is handed two different finish instructions in one task
text, and the first one does not run.**

Leader verification, both files opened at the line:

- `packages/bee/agents/bee-build.md.tmpl:15` — *"then `bee cells finish --report`"*. No `--id`, no `.bee/bin/` prefix. `--id` is required, so this command refuses.
- `packages/bee/prompts/worker-cell.md:49` — *"Finish with: `.bee/bin/bee cells finish --id {{cell_id}} --outcome "<one line>" --files <a,b> --report '<json>'`"*. Correct.

Both reach one worker: `prepare.rs:1125` loads `worker-cell` as the task text,
and the agent contract renders beside it. The broken instruction comes FIRST; the
correct one is roughly two hundred words later, mid-bullet, behind test-scope,
red-refusal, CI and mistakes prose.

This is a one-fact-one-home violation in the text every execution worker reads,
and it is a far better supported cause for five consecutive misses than worker
unreliability.

**Correction owed on the leader's own record.** The backlog row filed at the last
feature's close reads *"workers commit and test but never cap"*, which blames the
worker. The evidence says the worker was told to run a command that does not run.

**Correction to the seat.** `hat-alternatives` also argued these five were
non-interactive `pi --mode json -p --no-session` children with no loop left to run
a follow-up command. Not accepted: those cells dispatched as `--agent "agy-flash"`
with no `--no-pane` flag, so they ran as pane workers and could have run a second
command. The unrunnable instruction stands as the explanation; the unreachable
runtime does not.

---

## Why the drafted auto-cap could not be built honestly

### B1 — The runner cannot produce the worker's bytes (from `hat-facts-gaps` G1, `hat-risks` P1)

Leader verification: `herding/mailbox.rs:466` — `MailboxResult` carries `round`,
`status`, `summary`, `files_changed`, `proof`, `options`, `leaning`. The parser at
`:633-658` reads `status`, `summary`, `files_changed`, `proof` and nothing else.

A legal cap needs the report keys `outcome`, `commit`, `files`, `tests`,
`deviations` (`verbs/cells/finish_support.rs:57`), with `outcome` and `commit`
non-empty, and reads `mistakes` (`handlers_close.rs:888`).

So the runner would have to author `commit`, `deviations` and `mistakes` about a
worker it only observed. That is precisely the new trust D5 said was not being
extended, and it makes D1's "byte-identical to the one the worker should have
made" false.

### B2 — It would move the manual step, not remove it

A cap recorded with no `mistakes` answer lands as `Unanswered`, and `bee close`
counts exactly that as blocking debt. The leader hit that refusal by hand two
hours earlier on another feature. Auto-capping would have traded capping by hand
for unblocking closes by hand — with the cell already capped, which is harder to
undo than capping was.

### B3 — The no-pane runner never produces a valid proof line

Leader verification: `herding/run.rs:3036-3046` synthesizes
`proof` as `"child process exited 0 (tokens: …)"`. That is not a three-segment
proof line, so under the drafted design every child-runner cell job would have
printed a loud refusal by design — and a real refusal would hide among them. The
no-pane runner shipped from this session's previous feature, so the two would have
collided immediately.

### B4 — Accepted, smaller, and still true if the auto-cap is ever built

- Gate on `status: done`, not proof shape alone: `MailboxStatus::Blocked` is a well-formed completion and a blocked worker can still return a valid proof line.
- `cap_cell_from_flags` does not release reservations; `finish_cap_and_release` (`handlers_close.rs:1207`) is what `cells finish` actually is.
- A worker that DID cap makes the runner's call return `already capped` — that must read as already-capped, never as a refusal.
- The proof line's command segment must byte-equal the cell's `verify` (`handlers_close.rs:252-261`), so "the worker's own bytes" were never free bytes.
- Nothing records WHO capped. Auto-capping would delete the only evidence that a worker does not self-cap. `trace.capped_by` is the cheap fix, and belongs with the auto-cap if it is ever built.

### Claims-table corrections accepted

| Row | Verdict | Correction |
|---|---|---|
| 1 | OK, one byte wrong | the flag is `--cell-id`, not `--cell` |
| 3 | OK as quoted, misleading as a claim | "a callable cap exists" hid roughly nine refusal preconditions the plan never named |
| 7 | WEAK | labelled `ran`, but the archived cells now read `capped` after the manual caps, so it is not reproducible today; and one of the seven was capped by the leader, not a dispatched worker, so "every dispatched worker" was 5 of 6, not 7 |

---

## What the wave did NOT change

`hat-alternatives` judged the SMALLER PATH reasoning sound and the live-drive cell
worth keeping. That holds in the new shape: the whole value is that a mechanical
step fires, and only a real dispatch shows it.

## Outcome

Decision `9d2347e4` is superseded by `7152ebab`: fix the instruction, do not build
the runner auto-cap. The auto-cap route is recorded here with its blockers so a
later attempt starts from B1-B4 rather than rediscovering them.
