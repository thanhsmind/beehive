---
date: 2026-08-14
feature: traceable-runs
categories: [failure, pattern]
severity: critical
tags: [capture, mutating-cli, workflow-record, worktree-first-guard, binary-freshness]
---

# Learning: A mutating CLI verb probed for its own documentation is still a mutating verb

**Category:** failure
**Severity:** critical
**Tags:** [capture, mutating-cli, workflow-record, default-record-target-resolution]
**Applicable-when:** investigating what a bee CLI verb accepts or how it behaves, in ANY checkout that has a real active feature (not a scratch fixture).

## What Happened

While capturing `bee state gate`'s new `--actor`/`--bypass-level`/`--reason`
flags (traceable-runs D2/trun-2) into the knowledge bundle, the capture agent
ran `.bee/bin/bee state gate --name execution --approved true --actor auto
--bypass-level normal --reason test --json` directly in the main checkout,
intending only to confirm the flags were accepted. `bee state gate` with no
`--lane` resolves to "the default `state.json` record for an unbound
session" — which, in this repo at that moment, WAS the real, still-active
`traceable-runs` workflow record (`.bee/runtime/workflows/wf-4605d9c6/`,
`feature: "traceable-runs"`, `phase: "swarming"`). The call overwrote the
`execution` gate's `actor`/`at`/`reason`/`bypass_level` fields — genuine
audit trail from earlier in the feature's real life — with the test values.
The agent caught this by re-reading the record file directly (fields that
should have stayed `null`, matching the neighboring `context`/`shape`
gates, now carried the probe's `auto`/`test`/`normal`), and repaired it by
hand-restoring the four fields to `null` (the value the `approved: true`,
`state: "approved"` neighbors already carried) — the only fix available,
because no `bee state gate` flag combination can write those fields back to
`null` (`--actor` always writes a string, defaulting to `"user"`, never
`null`).

## Root Cause

`bee state gate`'s target resolution has three tiers — explicit `--lane`,
the calling session's bound lane, or "the default record for an unbound
session" — and the third tier is not a scratch or sandboxed target: it is
whatever feature the repo's `.bee/state.json` currently names. A capture or
documentation pass that "just tries a command to see what it does" has no
signal, at the command line, that the default record is live production
state rather than an inert leftover. The same class of mistake would trigger
identically for `bee state set`, `bee cells cap`, or any other mutating verb
resolved the same way.

## Recommendation

Never invoke a mutating CLI verb to observe its *shape* or *acceptance* —
use `--help`, a `--show`/read-only counterpart, or `bee status --json`
instead. When a mutating call is genuinely necessary to observe an effect,
either target an isolated scratch fixture (a throwaway repo/worktree with
its own `.bee/`) or first capture the current field values so a mistaken
write can be repaired byte-for-byte rather than guessed back. Before
repairing, look at a structurally identical sibling record for the honest
"before" shape (here: the untouched `context`/`shape` gate entries proved
what `execution`'s fields looked like before the accidental write) — never
invent a plausible-looking default.

---

# Learning: The vendored `.bee/bin/bee` binary in a self-hosted checkout lags the source it dogfoods — already mechanized, no new record needed

**Category:** pattern
**Severity:** standard
**Tags:** [binary-freshness, self-hosting, doctor]
**Applicable-when:** confirming whether a just-merged CLI change (a new flag, a new verb) is actually reachable through `.bee/bin/bee` in THIS repo.

## What Happened

Testing `bee state gate`'s new flags and the new `bee deferred-queue` verb
(trun-2/trun-8/trun-9) against `.bee/bin/bee` in this checkout showed no
new flags in `--help` output and an `unknown_command` refusal for
`deferred-queue`, even though both landed on `main` (`635b78f3`) hours
earlier. `bee doctor --runtime claude --json` already names this exactly:
row `binary_freshness` reads `not_ok`, with the fix command
(`cargo build --release ... then copy target/release/bee to .bee/bin/bee`).
Rebuilding and copying was attempted but refused by the worktree-first
write guard (`packages/bee-rs/target/release/bee` read as a source-write
argument token inside the MAIN checkout for an active code-touching
feature holding no granted worktree) — correctly, per this repo's own
doctrine; the binary was left untouched, unpatched.

## Root Cause

A self-hosted checkout (this repo builds the same `bee` it runs on itself)
needs its own vendored binary rebuilt after every merge that touches the
CLI — `doctor-binary-freshness` (an earlier feature) already built the
mechanized check for exactly this drift. There is nothing new to promote
here: the detector exists and reported correctly; only the *fix path*
collided with an unrelated guard (worktree-first) in the specific case of
running the rebuild from main mid-feature.

## Recommendation

Trust `bee doctor`'s `binary_freshness` row over a manual `--help`
comparison. When its fix command is blocked by the worktree-first guard
(an active code-touching feature with no granted worktree in main), that is
the guard doing its job — name the blocker rather than working around it
(e.g. `.bee/config.json`'s `worktree_first: "off"`); the rebuild waits for
a worktree or for the feature to close.

---

# Learning: A shared helper extracted to reconcile N copies of a scan is only as complete as the census of those copies — already covered, extending the existing pattern

**Category:** pattern (recurrence of `pattern-20260807-fix-the-fan-out-not-the-reported-cell`, not a new record — see that file for the full rule)
**Severity:** critical
**Tags:** [refactor, census, shared-helper, judge-review, scribing-debt]
**Applicable-when:** consolidating N scattered copies of a check or scan
behind one shared function or rule.

## What Happened

trun-9 (traceable-runs D5) extracted `deferred_debt_cleared` in
`state_group/ledger.rs` to reconcile the scribing-debt scan against the new
deferred-work queue. The scan itself had six independent copies across four
files (`drivers/close.rs`; `hooks/session_preamble/store.rs` ×2;
`hooks/chain_nudge.rs`; `verbs/status_full/cells.rs` ×2). The cell's first
cap wired two of the six. A first judge round found two more (the session
preamble and chain-nudge copies) — the debt cleared at `bee close`'s door
but stayed loud in the session preamble and mid-session nudge. A second
judge round, after those two were fixed, found the last two
(`status_full/cells.rs`) — the exact copies `bee status --json` and
`bee orient`'s routing blocker read, so the debt cleared everywhere except
the one surface agents read on every routing turn. The cell's own doc
comment claimed "three" scans after round one and "five" after round two;
both counts were wrong, caught only by a fresh `rg` census in round three,
not by re-reading the comment.

## Root Cause

Same root cause `pattern-20260807-fix-the-fan-out-not-the-reported-cell`
already names: a rule (or, here, a shared helper) is only as good as the
grid it's checked against, and a fix scoped to the callers a first pass
happened to find is a scheduled recurrence, not a fix. This instance adds
one detail: the miscount survived inside the helper's *own doc comment*,
which asserted a specific total ("three", later "five") that read as
authoritative and was wrong both times — a stated count is not evidence of
a complete census, only a grep across the codebase for the shape being
replaced is.

## Recommendation

Not a new rule — apply `pattern-20260807-fix-the-fan-out-not-the-reported-cell`'s
existing rule ("name the grid before fixing; enumerate from source, not a
hand-written list") with this addition: when the grid is "every existing
copy of a scan/check", the source to enumerate from is a structural search
for the shape the copies share (here: every `capped_at`-vs-threshold
comparison in the crate), never a hand-maintained doc comment stating the
count — a comment claiming "N copies, all reconciled" is itself a claim to
verify, not evidence the census is complete.
