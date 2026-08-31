# Reflection Becomes Lesson — Plan

**Feature:** reflection-becomes-lesson
**Lane:** high-risk (4 flags: data-model, public-contracts,
covered-contract-change, multi-domain; 8 product files)
**Decisions:** `docs/history/reflection-becomes-lesson/CONTEXT.md`
D1 `db562f26` · D2 `a240362a` · D3 `0872f328` · D4 `c556c959`
· D5 `20fe96d3` · D6 `7760339d`
**Plan-step hat wave:** five seats, 2026-08-31. Findings folded in
below; the wave rewrote the cell split, the scoping of the door, and
three evidence lines.

## The problem in one paragraph

`bee mailbox reflect` shipped on 2026-08-30 with doctrine as its only
trigger, and the doctrine line was never written. The verb is
unreachable: nothing in `skills/`, `AGENTS.md`, `CLAUDE.md`, the
host-repo block, or the worker prompt names it, and zero reflection
entries exist anywhere in `.bee/human-mailbox/`. LR4 (`bb73e821`) also
held reflections out of the lesson miner, so even a written one would
stop at the letter. The chain the user asked for — mistake → written
reflection → mined lesson — is broken at both links.

**What the evidence does and does not prove.** The zero proves the verb
is *unreachable*, not that a written doctrine line was tried and failed:
no doctrine line was ever written, and the measurement window is one day
(letter-reflection shipped 2026-08-30). An earlier draft of this plan
said "zero across 30 filed letters in the month since" — that counted 26
per-run entry stores as letters and overstated the window. The store
holds 4 letters and 26 entry stores. The corrected claim is the one
above, and it is enough: an orphaned verb stays at zero forever.

## Discovery — what the reality touch and the wave changed

1. **The departure rule this design mirrors is armed-gated.**
   `handlers_close.rs:250-253` runs `departure_door` only
   `if mailbox::armed(root)`. Copying the mirror wholesale would exempt
   every attended session — the user's own case. The mirror is taken for
   SHAPE, never for the arming gate.
2. **Nothing asserts today's exclusion.** `trouble_lines`
   (`mailbox_digest.rs:710-728`) reads only `SECTION_BROKEN` and
   `item.departure`; it ignores reflections by omission and no test
   asserts that. D3 replaces no existing proof — the `proof-weakening`
   flag was dropped from the route on this evidence.
3. **A run-scoped door would have refused every close.** Reflections are
   stored per run (`mailbox.rs:2771-2772`, `run_id` falling back to
   `UNATTRIBUTED_RUN` at `mailbox.rs:910-915`), and `bee close` resolves
   its *own* session's run (`close.rs:3084-3087`). Under the default
   `uat_stop: "close"` the closing session is the attended orchestrator,
   often a different run from the workers' — so a run-scoped door sees
   no reflections, refuses every close, and the only available answer is
   the clean-run flag. That reproduces exactly the silence D1 exists to
   end. **D5 below is the fix.**
4. **The two "parallel" cells were not disjoint.** rbl-1's collection
   lane needs the worker prompt to name a `mistakes` field, and rbl-3
   edits that same paragraph (`packages/bee/prompts/worker-cell.md:44,47`).
   They are folded into one cell.
5. **The enforced-answer precedent produces signal, not ritual.** Across
   the 155 stored entries, 115 caps carry a departure answer and 65 of
   them carry a substantive departure object — 57%, with zero hollow
   plan-followed objects. Enforcement demonstrably gets written.

## Decisions taken inside the agent's discretion

- **D5 (`20fe96d3`) — the answer is recorded per cell, and the close door walks the
  feature's capped cells.** The cap is the one writer and it writes to
  two sinks from one set of values, so they cannot disagree: the cell
  trace gains a mistakes answer beside the existing
  `trace.plan_followed` (`handlers_close.rs:511`), and the run's mailbox
  gains the reflection entry that renders in the letter. `bee close`
  refuses by reading the feature's capped cells — the walk it already
  performs at `close.rs:3036-3051` — never the closing session's run
  file. This keeps D1's door exactly where D1 put it, makes the answer
  survive a resumed, compacted, or handed-over session, and matches the
  user's own grain: per task, not per night.
- **D6 (`7760339d`) — the collapse is measured, not assumed.** A reflexive clean-run
  answer on every close would regress to today's zero. The counts that
  detect it are already computable: the feedback digest reports the
  clean-run-to-reflection ratio, the same way the 57% figure above was
  computed for departures. A number nobody can see is a defense nobody
  has.

## Shape

Two cells over two slices.

### Slice 1 — the answer is demanded, recorded, and instructed

**rbl-1 — the mistakes answer, end to end** (D1, D2, D4, D5).

- **The store.** A new entry kind carries D2's explicit clean-run
  answer, built the one way the store builds entries, beside
  `KIND_REFLECTION` (`mailbox.rs:599,653`). It is excluded from Done and
  from the subject by the same predicate family `is_reflection` uses
  (`mailbox.rs:1765-1767,1833`) — never by string matching — and its
  `what` must fail `check_subject`, so an older or reverted binary
  re-composing the letter can never elect it as the subject.
- **The cap collects.** A `mistakes` array in the worker result and a
  cap flag, each line in the two required parts, parsed through the
  existing `read_reflection` door (`mailbox.rs:717-738`) so the two-part
  rule keeps one home. The cap adds **no refusal** — the busiest door's
  behaviour is unchanged, and its blast radius stays at zero.
- **The close enforces.** `bee close` refuses when any capped cell of
  the feature carries no mistakes answer, naming the cells. **Not
  armed-gated**: a feature close files its letter unconditionally,
  attended sessions included (`close.rs:2970,3075-3077`), so a close may
  always demand the answer. The refusal copies the scribing-debt door's
  shape (`close.rs:2429-2446`) and runs with the other blocking doors,
  before any write — a refused close appends nothing and files nothing.
  Its text is the one drafted by the wave's user-impact seat: it names
  what the letter cannot say, offers the reflect command first and the
  clean-run flag second, and ends on the human reason.
- **The three doctrine homes** (D4): `AGENTS.md` "Care for the session"
  (line 248), `packages/bee/AGENTS.block.md` (281 lines, the host-repo
  block), and `packages/bee/prompts/worker-cell.md` (the cap bullet at
  line 44 and the result-form paragraph at line 47). The wording is part
  of the cell, not an afterthought: each line carries LR2's
  moment-of-notice rule — write it when you notice, never composed from
  memory at the end — and demands a concrete noun in the what-went-wrong
  part: a file, a command, or an observable.

### Slice 2 — the answer becomes a lesson

**rbl-2 — reflections join the trouble sources** (D3, D6).

- `trouble_lines` (`mailbox_digest.rs:710-728`) gains the letter's
  reflection items beside the broken bullets and the three
  `MINED_DEPARTURE_KINDS` (`mailbox_digest.rs:631-635`).
- **The tokenized text is the reflection item's `what` field, never the
  rendered bullet.** The rendered bullet is `<what> — better: <better>`
  (`mailbox.rs:1767-1774`); tokenizing it would make the same mistake
  with a differently worded counterfactual fail to match, and re-parsing
  rendered prose is what the digest's own doc forbids. `compose_letter`
  maps every entry to an item, so `item.what` is available structurally.
- It reuses `normalize_shape` and `shape_token`
  (`mailbox_digest.rs:659-675,680-685`) and the existing spent-token
  dedup — no second derivation. The clean-run answer is excluded.
- D6's ratio line lands here, beside the miner that would otherwise be
  the only witness.

## The honest ceiling

`normalize_shape` lowercases and collapses digit runs; it does no
stemming and no fuzzy matching, by design. Two honest reflections about
the same underlying mistake, written in different runs about different
files, will not hash to the same token — so **auto-mined lessons will be
rare**. What ships immediately and visibly is the letter: every run's
mistakes become readable, and they become the raw material a human or a
later feature can distill. Saying this now is the point; the user's
complaint returning in a month as "still no lessons" is the failure this
paragraph exists to prevent.

Two known limits, stated rather than hidden:

- A run that caps cells and never closes a feature still ends without an
  answer. D1's door is the close door; this class of run is out of its
  reach by construction.
- The close door cannot detect a false clean-run answer over three
  red-then-fixed cells. D6's ratio is the only witness, and it is a
  witness, not a guard.

## Why this size

Dropping the doctrine work leaves the verb orphaned in host repos and in
every dispatched worker — the exact failure being repaired, and no home
derives from another (`packages/bee-rs/crates/bee/tests/pointer_integrity.rs:35` lists `AGENTS.md` and
`packages/bee/AGENTS.block.md` as separate sources with no parity sync;
the worker prompt reaches a third reader class). Dropping rbl-2 leaves
the user's stated ask unmet. Dropping the door leaves the chain resting
on a doctrine line, and D1 is that door.

## SMALLER PATH check

*Is there a cheaper shape that still honors every locked decision?*
**PASS, after one redraft.** The wave's alternatives seat found the
three-cell split false — rbl-3 shared `worker-cell.md` with rbl-1 — and
this plan folds them. It also tested and rejected two further
reductions: one merged cell across `mailbox.rs` and `mailbox_digest.rs`
(rejected — the miner widening deserves its own revert, separate from
the close-door hazard, and red-first is cleaner against an unmodified
`trouble_lines`), and riding `PLAN_FOLLOWED` instead of a new kind
(rejected — D2 locks the encoding, and `plan_followed` persists only as
a per-cell trace flag, never as a mailbox entry). Doctrine-only is not a
candidate: an unnamed verb is unreachable by construction.

## Cost if the shape is wrong

The close refusal sits on a door every feature passes. Three guards:
the refusal is answerable by the acting agent from the cells' own
records, not from its memory (D5); no script, hook, CI job, or the
herding control loop calls `bee close` — every caller is an agent
session or a human who can read the remedy; and all refusal doors run
before any write, so a refused close is idempotent and repeatable.
Reversal is one revert of one check. The asymmetry to say out loud: the
refusal carries no config off-switch, so in a host repo a wrong refusal
is undone by shipping a binary, not by editing a file. A config escape
valve was considered and rejected — the departure door it mirrors has
none, and a switch that turns the record off is a switch that will be
left off.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The verb exists and is reachable from the mailbox dispatch | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2658,2751` | `"reflect" => run_reflect(...)`; `fn run_reflect` |
| 2 | `KIND_REFLECTION`, its constructor and its two-part door exist | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:599,653,717-738` | `const KIND_REFLECTION`; `Entry::reflection`; `read_reflection` refusing a missing part |
| 3 | A reflection is excluded from Done and from the subject, by kind | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1765-1767,1833` | `else if entry.is_reflection()` with "Its OWN section, and never Done"; `.filter(\|e\| !e.is_reflection())` in the subject chooser |
| 4 | No doctrine file names the verb | ran | `rg -n "mailbox reflect" skills/ AGENTS.md CLAUDE.md packages/bee/` | zero matches |
| 5 | Zero reflection entries exist in the live store, which holds 4 letters and 26 entry stores | ran | `.bee/human-mailbox/` | `ls -1 .bee/human-mailbox/*.md \| wc -l` printed `4`; `ls -1 .bee/human-mailbox/entries/ \| wc -l` printed `26`; `rg -l '"kind":"reflection"' .bee/human-mailbox/` printed nothing and exited 1. Run in the main checkout — the store does not exist in this worktree |
| 6 | The miner's trouble sources are the broken section plus three departure kinds only | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:631-635,710-728` | `MINED_DEPARTURE_KINDS`; `fn trouble_lines` reading `SECTION_BROKEN` and `item.departure` and nothing else |
| 7 | One shape-token derivation exists and rbl-2 must reuse it | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:659-675,680-685` | `fn normalize_shape`; `fn shape_token` |
| 8 | No test asserts reflections are unmined — the exclusion is by omission | read | `packages/bee-rs/crates/bee/src/verbs/mailbox_digest.rs:710-728` | `fn trouble_lines` walks `section_lines(&letter.body, SECTION_BROKEN)` and `letter.items`' `departure` only; the word "reflection" appears nowhere in the file, its test module included |
| 9 | The close door has a typed refusal shape to copy, and the doors run before any mailbox write | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2429-2446,2955-2960,2970,3117` | scribing-debt door returning `Out::Emit(..., 1)` with `remedy:`/`next:`; the doors-before-tail comment; `record_feature_close_in_mailbox` at 2970; `file_close_letter` at 3117 |
| 10 | A feature close files its letter unconditionally, attended or not | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:2970,3075-3077` | the unconditional call on the non-dry-run tail, and its own doc line naming attended sessions |
| 11 | The departure rule is armed-gated, and the mirror must not copy that gate | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:250-253,725-777` | `if armed { departure_door(...) }`; the refusal ending "Silence and nothing-happened must not read alike" at 772 |
| 12 | The cap already persists its answer onto the cell trace — D5's precedent | read | `packages/bee-rs/crates/bee/src/verbs/cells/handlers_close.rs:419-511` | `lift_plan_followed`; `if plan_followed { … }` writing `trace.plan_followed` |
| 13 | The close door already walks the feature's capped cells | read | `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3036-3051` | `list_cells_including_archive(root, feature, Some("capped"))` |
| 14 | Reflections are stored per run, and close resolves its own run | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:2771-2772,910-915`; `packages/bee-rs/crates/bee/src/verbs/drivers/close.rs:3084-3087` | `run_reflect`'s resolver; `UNATTRIBUTED_RUN` fallback; `mailbox::run_id(...)` at close |
| 15 | `AGENTS.md` and the host-repo block are separate SOURCE files, with no parity sync between them | read | `packages/bee-rs/crates/bee/tests/pointer_integrity.rs:35` | both listed as sources in the same array; no test syncs one from the other |
| 18 | Doctrine home one exists where D4 names it | read | `AGENTS.md:248` | the `## Care for the session` heading |
| 19 | Doctrine home three is the cap bullet and the result-form paragraph | read | `packages/bee/prompts/worker-cell.md:44,47` | line 44 `- Finish with: .bee/bin/bee cells finish --id {{cell_id}} ...`; line 47 `Result form: ... \`deviations\` carries one line per departure from the plan, each in THREE parts ...` |
| 16 | The worker result JSON already carries the parallel `deviations` lane, and it works | ran | `packages/bee/prompts/worker-cell.md` | the three-part deviation rule in the prompt; 115 cap entries, 65 carrying a substantive departure object (57%), zero hollow plan-followed objects |
| 17 | The letter frontmatter is additive and unknown keys are ignored by readers | read | `packages/bee-rs/crates/bee/src/verbs/mailbox.rs:1255-1259,1149,1462` | the ADDITIVE contract comment; `Letter::from_frontmatter` reading known keys |

## Test scope

Related tests, not the full suite: the `mailbox` and `mailbox_digest`
module tests, the `verbs::drivers::close` and
`verbs::cells::handlers_close` tests, and the registry/flag-ratchet
walks any new flag trips (`catalog.rs` `PINNED_FLAG_COUNT`, the
`registry_contracts` target — letter-reflection tripped both, per its
recorded deviation). CI runs the declared command on push.

Cases the wave named that the tests must cover:

- A close whose feature has a capped cell with no answer is refused; the
  refusal names that cell and writes nothing.
- A close by a session whose *own* run holds no entries still passes when
  the cells carry answers — the D5 regression test.
- The clean-run entry never appears under Done, never becomes the
  subject, and is never mined.
- Red-first for rbl-2: the mining test fails against today's
  `trouble_lines`, is seen red, and then the source set widens.

## Open Questions

None. Every load-bearing claim is `read` or `ran`.
