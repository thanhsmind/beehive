---
type: bee.area
title: Hook Runtime — the request shapes the write guard can read
description: "How the write guard decides a batch file-change request target by target, how it reads a shell request past its first command and through its wrappers, how it shape-checks a workflow command against the published catalog, which command forms it still recognises, how it repairs a mechanically fixable question request instead of refusing it, and why an intercepted-but-unreadable request is denied rather than waved through."
timestamp: 2026-08-05
bee:
  id: hook-runtime-write-guard-request-shapes
  lifecycle: active
  areas: [hook-runtime]
  required_context: [areas/hook-runtime/overview.md]
  decisions: ["codex-runtime-parity D1, D2", "bbc6bcea (shim-retire D3: dual command-shape recognition, retired form transitional)", "ask-guard-autofix D1/D2 (fixable question violations repaired + announced, deny wins, 2026-07-23)", "d4182ff1 (blanket-staging-guard: git add -A/-u and git commit -a count as broad writes, 2026-07-26)", "5bd08e53 (ask-guard verdict correction: a repaired question escalates with \"ask\", an advisory reservation notice carries no verdict at all, 2026-08-03)", "761515d4 (guard-parser-depth Gate 2: close the compound-command and shell-wrapper bypasses in one parse every guard consumer shares, depth-bounded with truncation marked — cell gpd-1, 2026-08-05)", "js-parity-cleanup D3 as corrected by jp-9 (docs/history/js-parity-cleanup/CONTEXT.md, 2026-08-04 — display caps count characters, but the question-heading limit keeps the platform validator's own counting unit; judge finding bc2e2d44)", "38e323d6 (no-code-comments D1: no comment in a code file, in any language; three exceptions only)", "80ec3cd1 (no-code-comments D2: the ratchet — a committed per-file baseline that only ever goes down)", "0add770d (no-code-comments D2b, touches D2: the baseline is seeded from the per-file maximum over main and every unmerged wt/* branch)", "c2477366 (no-code-comments D3: the write guard refuses a write that adds a comment line to a code file, naming file, line and remedy)", "0ee8248d (no-code-comments D4: the why has two homes — a docs/knowledge concept or a bee decisions log entry — and no third)", "002a935d (no-code-comments D5: the rule lands in packages/bee/AGENTS.block.md and packages/bee/prompts/worker-cell.md)", "69326d15 (no-code-comments D6: removing the existing comments is separate, batched grooming work)", "e3bf4a57 (no-code-comments D7: one config key no_code_comments, default false; this repo sets it true, hosts opt in)"]
  sources: ["codex-runtime-parity repo-fallback capture 2026-07-12 — cells codex-parity-6a, 6b", "dispatcher-unify du-2 (2026-07-12, flushed capture stub 9e68432b)", "shim-retire D3 transition guard (cell shim-retire-3, 2026-07-14)", "ask-guard-autofix cell ag-1 (2026-07-23, commit 52dad26)", "blanket-staging-guard cell bsg-1 (2026-07-26, commit b240110)", "docs/specs/hook-runtime.md#B3", "docs/specs/hook-runtime.md#B3a", "docs/specs/hook-runtime.md#R3", "docs/specs/hook-runtime.md#R14a", "docs/specs/hook-runtime.md#E1", "docs/specs/hook-runtime.md#P6", "docs/specs/hook-runtime.md#P7", "guard-parser-depth cell gpd-1 (trace .bee/cells/archive/guard-parser-depth/gpd-1.json, commit 98888896, plan docs/history/guard-parser-depth/plan.md, capped 2026-08-05)", "js-parity-cleanup cell jp-9 (heading guard restored to the platform's counting unit, the ASCII-only repair and its fall-through named; trace .bee/cells/archive/js-parity-cleanup/jp-9.json, 2026-08-04 — full suite 1006 passed, 0 failed)"]
  authoritative_for: "hook-runtime: write-guard request-shape recognition and per-target decisions"
---

# Hook Runtime — the request shapes the write guard can read

Before the guard can decide whether a write is allowed, it has to understand what
was actually requested. Three request shapes reach it — a batch file-change
envelope, a shell invocation of a workflow verb, and the two command forms the
vendored surface has used over time — and the discipline is the same in all
three: a request the guard intercepted but cannot read is denied, while an event
it never saw at all fails open and says so.

**`R14a` is a disambiguated id.** This rule shipped as `R14` and shared that id
with the gate-bypass block-verdict rule in
[`advisories-and-turn-control.md`](advisories-and-turn-control.md); the collision made
one of the two permanently unmeasurable by the coverage gate. The two are
genuinely different rules, so neither was dropped: this one — the id no other
document ever cited — was renumbered `R14a` in the source before the migration
pin was captured, and the pointer stub's anchor map records both readings.

## Behaviors & Operations

**B3 — Batch file-change requests are guarded per target.** When the runtime
announces a batch file-change request (the patch-style tool), the write guard
parses every add/update/delete/move target and runs each one through the same
gate, direct-edit, and reservation decisions that govern single writes.
- All targets provable → each target decided on its own; one denied target
  denies the request with a corrective message.
- Request intercepted but targets NOT provable (no parsable change lines, a
  blank path, a target resolving outside the project) → **deny** with a
  corrective message. An intercepted-but-unreadable batch is never waved
  through.
- The outer event itself malformed (no batch envelope present at all) →
  fail-open, logged: the guard cannot know a write was intended.
- Containment recognizes the worktree-companion mount (PR #61, cell mp61-1,
  2026-07-24): a path under the recorded `commands.worktree_companion_mount`
  symlink — the mount `bee worktree new --with-companion` creates for a nested
  repo's own worktree — resolves to its companion-relative form instead of
  being denied as an out-of-worktree escape. Every other out-of-worktree
  target keeps today's denial.

**B3a — Workflow-command requests are shape-checked against the published
catalog.** When a shell request invokes a workflow verb, the guard resolves the
command against the catalog of record — including verbs whose full name is
three words deep (group, sub-group, action) — and validates the required
parameters and value shapes before the command runs. A malformed invocation is
denied with the command, the missing or wrong field, and the corrective shape;
a well-formed one proceeds untouched. Deep verbs previously escaped this check
unvalidated (a silent fail-open); they no longer do.

**B3c — Asking a command for its own help is never shape-checked.** A request
that asks a workflow command to explain itself passes the shape check
untouched, whatever parameters that command requires. Help is how a caller
learns the shape, so the check that teaches the shape may not stand in front of
it — and the denial text itself points the caller at the command's help, which
would otherwise be denied in turn. The exemption is read from the request as
the catalog parses it, not from the raw words: a help word that a preceding
parameter consumes as its own value is a value, not a help request, and the
shape check still applies in full.

**B3b — A shell request is read past its first command and through its
wrappers.** The guard no longer judges a shell request by its opening command
alone. Every command in a compound request — the pieces joined by the
sequencing, conditional, and pipe separators — is read, so a guarded command
hidden behind a harmless one is still seen. On top of that, a command handed to
a shell for interpretation (a shell name invoked with the read-a-command-string
option, or the built-in evaluate verb) has its payload re-read as commands in
its own right, recursively, so wrapping a guarded command in a shell no longer
hides it. Three limits keep the deeper reading from over-reaching:
- Only a wrapper's own payload is re-read. A quoted span that is merely an
  argument — a message, a literal string — stays one opaque word, so a
  guarded verb *named inside* a commit message or an echo is never mistaken
  for an invocation.
- A wrapper's payload is fenced. Commands unwrapped from inside a payload can
  never join with the text on either side of the wrapper to form a command
  that was never written.
- The re-reading is depth-bounded. A nesting deeper than the bound stops and
  marks the reading truncated rather than recursing without end; the marker
  travels with the reading so a consumer can treat a truncated reading as
  unproven rather than as clean.
All three write-guard consumers that reason about a shell request — the
reservation-target extraction, the guarded-command checks, and the
request-shape detectors — read through this same deeper parse, so a wrapper
closes for all of them at once or for none (cell gpd-1, 2026-08-05).

**B23 — Blanket staging reads as a broad write, not as zero targets.** When a
shell request's targets are extracted for the reservation guard, `git add
-A`/`--all`/`-u`/`--update` and `git commit -a`/`--all` (combined short
clusters like `-am` included) set the broad-write marker even though they name
no path — they stage or fold in *every* changed file, which on a shared
checkout can sweep another session's in-progress work into the commit. The
broad-write marker resolves to the `**` target, so the existing reservation
flow blocks exactly when another session holds a reservation and stays a no-op
for a single session. Explicit-path `git add`, plain `git commit`/`-m`, and
`--amend` (matched by exact token, never substring) are untouched (cell bsg-1,
2026-07-26).

**B22 — A malformed question-to-the-human request is repaired when the repair
is mechanical, refused when it is not.** When the runtime announces the
ask-the-human tool, the guard shape-checks the request before the platform's
own opaque validation can reject it. Trigger: any question request. What
happens: a violation whose repair is deterministic and meaning-preserving — a
chip-label heading over the 12-unit limit (B28) — is FIXED, not refused: the
heading is rewritten (first 11 characters, right-trimmed, plus an ellipsis) on
a copy of the request, and the question proceeds with the rewritten input; the
platform is told, in the approval itself, exactly what was changed, and the
human sees a one-line note of the rewrite. A violation with no mechanical
repair — question count outside 1–4, option count outside 2–4, an option
missing its label or description — refuses with the specific correction, and a
refusal always wins over any repair collected in the same request: the mixed
case refuses. Odd shapes still fail open. What each actor observes: the asker's
question reaches the human instead of dying on a label-length technicality;
the original request object is never mutated — the rewrite rides a replacement
copy (ask-guard-autofix D1/D2, cell ag-1, 2026-07-23).

**B22a — The repair escalates the question to the human; it never pre-approves
it.** The repaired request is announced to the platform as an *escalation*
(`permissionDecision: "ask"`), not as an approval. The distinction is the whole
behavior: for the ask-the-human tool the platform's approval prompt IS the
question the human answers, so an approval verdict answers the prompt away —
the tool then returns with no selection and the asker falls back to its own
default, swallowing the very question the repair existed to save. The
escalation verdict carries the rewritten request with it, and forces the prompt
even where the permission mode would otherwise skip it. Observed and fixed
2026-08-03: a 13-character heading tripped the repair, and the human never saw
the question.

**B28 — The heading limit is counted the way the platform counts it, and the
repair runs only where it is provably safe (js-parity-cleanup D3 as corrected by
jp-9, 2026-08-04).** Trigger: shape-checking a question's chip-label heading.
What happens: the length is measured in the platform's own unit — the unit its
external validator uses, in which a character outside the basic range counts as
two — because a guard that measures a limit differently from the validator it
exists to satisfy will wave through exactly the requests that validator then
rejects. Every other length cap in this runtime counts characters instead; this
one is deliberately the exception, and it says so where it is written. The
repair is narrower than the check: a heading that is plain ASCII is rewritten
and the question proceeds, while a heading carrying any character outside that
range is not rewritten at all — truncating a paired character in the middle is
not a meaning-preserving repair, and our agreement with the platform on where
such a pair may be cut has never been proven. What each actor observes: an
over-long ASCII heading is repaired and announced exactly as B22 describes; an
over-long non-ASCII heading falls through to the guard's unported branch, which
is an open gap named below rather than a working path.

## Business Rules

- R3 — An intercepted batch change with unprovable targets is denied, not
  fail-opened (codex-runtime-parity D2, strengthening).

- R3a — Depth of reading is a property of the request, not of a consumer. Every
  guard decision that reads a shell request reads the same compound-and-wrapper
  parse; a reading that hit the depth bound is marked truncated and is treated
  as unproven, never as clean (cell gpd-1, 2026-08-05).

- R14a — The write guard's command-shape recognition accepts both the unified
  dispatcher form (group + verb) and the retired per-command helper form. The
  retired form is a transition affordance for hosts whose vendored tools predate
  the unified surface — it is slated for removal once hosts have upgraded (a
  debt item tracks it), and its recognition never revives the deleted scripts
  themselves (decision bbc6bcea, D3).

- R22 — A question-request violation is repaired only when the repair is
  deterministic and meaning-preserving; everything else refuses with the
  specific correction, and a refusal always beats a repair found in the same
  request. The repair is announced — to the platform in the escalation, and to
  the human as a one-line note — never applied silently (ask-guard-autofix
  D1/D2, 2026-07-23; verdict corrected 2026-08-03, see R23).

- R23 — No advisory verdict the guard emits may carry a permission approval.
  A repaired question escalates to the human; a soft reservation notice emits
  its warning as context and nothing else. The guard's only permission verdict
  is a refusal — anything short of a refusal leaves the host's ordinary
  permission flow exactly as it found it. An approval verdict attached to an
  advisory buys the guarded call more permission than it had, and on the
  ask-the-human tool it destroys the answer outright (2026-08-03).

- R27 — The chip-label heading limit is measured in the platform validator's own
  counting unit, not in characters — the one deliberate exception to this
  runtime's char-based length caps — and the automatic repair is restricted to
  plain-ASCII headings, because no truncation of a paired character has been
  proven to match the platform's own (js-parity-cleanup D3 as corrected by jp-9,
  2026-08-04).

## Edge Cases Settled

- A change line with a whitespace-only path counts as unprovable → deny (found
  and pinned during matrix construction).

- A program whose own name merely ends in a shell name's letters is not a
  wrapper: only a real shell name, invoked with the read-a-command-string
  option, opens its payload for re-reading (cell gpd-1).


**A shell request the guard cannot resolve statically is refused, not
resolved.** Three shapes reach the guard as unreadable and are denied rather
than executed and checked afterwards (proven 2026-09-01 while building
`.claude/skills/verify-bee`, and hit again 2026-09-08 during a capture drain):

- **An unexpanded `$VAR` in a write target or in command position.** The guard
  sees `$SP/out.sh`, cannot canonicalise it, and refuses by name — its remedy
  line says to expand the variable first or pass a plain in-worktree path.
  A literal dollar in a filename must be escaped or quoted so the shell never
  expands it.
- **A write after a `cd` in the same compound command.** The target is resolved
  against the request's own worktree root, not against the directory the
  compound command would have moved to, so the write is refused instead of
  landing somewhere the guard never checked.
- **A redirect to an absolute path outside the worktree.**

The consequence for tooling: a harness that drives bee against an out-of-tree
sandbox ships as an executable script invoked by its literal absolute path,
never as inline compound shell.

## The comment arm and the comment ratchet

No comment may be written in code in this repository. Two layers hold the rule:
the write guard refuses the write for a hooked agent, and a ratchet test in the
declared suite goes red when any file's comment count rises above its committed
baseline (38e323d6, 80ec3cd1, c2477366). The why a comment used to carry has two
homes and no third — a `docs/knowledge` concept whose Pointers name the file, or
a `bee decisions log` entry (0ee8248d) — and the rule is stated where agents read
it, as the marked rule `agents-no-code-comments` (002a935d).

**The switch.** One config key, `no_code_comments` (boolean, default false),
turns the arm on (e3bf4a57). The arm reads it through the guard's own merged
config read (`read_config`, `write_guard/store.rs`), so `.bee/config.local.json`
overlays `.bee/config.json` exactly as every other key. A host that onboards bee
gets the hook code and the doctrine line and refuses nothing until it sets the
key — `scripts/` exists in most hosts, and an always-on arm would refuse a host's
own code the moment it installed bee. The store the key is read FROM follows the
worktree grant (`hooks/adapter.rs`): a GRANTED feature worktree reads its own
`.bee/config.json`, so a branch that predates the key is never refused; an
UNGRANTED worktree reads main's store, so main's key decides for it.

**The code roots.** `under_code_root` owns the path list and nothing else
repeats it: `packages/bee-rs/crates/<any>/src/**`, `packages/bee/hooks/**`,
`packages/bee/lib/**`, `scripts/**`, `.bee/verify/**`. A target outside those
roots is passed over, and so is every `.md`, `.json` and `.toml` inside them —
the file must also read as code: extension `.rs`, `.sh`, `.bash` or `.py`, or an
extension-less file whose first line is a `#!` naming sh, bash or python
(`code_lang`).

**What counts as a comment line.** For Rust: a line whose first non-blank
characters are `//` — `///` and `//!` doc comments included, because a doc
comment carries "why" in code just as a plain one does — or `/*`, plus every
line inside the block it opens. For shell and python: a line whose first
non-blank character is `#`. Three exceptions are never counted and never
refused: a `#!` line (on ANY line, because a shell heredoc body carries one
too), a Rust comment whose text opens with `SAFETY:` on an unsafe block, and a
license header at the top of a file — a leading comment run holding `Copyright`,
`SPDX-License-Identifier` or `Licensed under`.

**The request shapes the arm can read.** The arm runs per target, after the
config guard and before the worktree-first guard, and it skips the `**`
broad-write sentinel. It reads:

- `Write` — `tool_input.content` is the whole proposed file.
- `Edit` and `MultiEdit` — `reconstruct_target_text` (`write_guard/main.rs`)
  rebuilds the WHOLE proposed file from disk plus the replacement, so line
  numbers, block-comment state, the shebang and the license rule all see the
  full file rather than a fragment.
- A `Bash` heredoc whose body is redirected into a code path — `heredoc_writes`
  (`write_guard/guards.rs`) pairs each body with its `>` or `>>` target, and an
  append is judged against the file it would extend.
- `apply_patch` — `apply_patch_added_lines` (`write_guard/detectors.rs`) returns
  the added lines per `*** Add File:` / `*** Update File:` target, and the arm
  judges those added lines alone.

It CANNOT read a Bash write with no readable body — a `sed -i`, a `cp`, a
redirect from a command's output. That shape is allowed through, by design: the
arm refuses only what it can prove, and the D2 ratchet catches whatever reaches
the tree anyway.

The comparison is old-versus-new as a multiset of trimmed comment text
(`added_comment_lines`), so moving a comment, deleting one, or reindenting one
passes; only a comment that was not there before refuses. The refusal names the
file and the line, quotes the first 80 characters of the offending line, says
that `///` and `//!` count too, and gives the remedy in the same message: the
two homes for the why, the owning concept for a public item's description, and
`bee backlog add` for a workaround, cited from the concept and never from the
code (`comment_guard_denial`, `write_guard/main.rs`). It reaches the host as
exit 2 on stderr like every other write-guard deny.

**The known limitation.** The comment predicate is a line predicate, not a
parser: a line that STARTS with `//` or `#` inside a string literal counts as a
comment. One such line exists today
(`hooks/session_close/html.rs`, a `// expand a project row…` line inside an
embedded script string). A false red there is fixed by rewriting the string —
never by an allowlist.

**The ratchet.** `.bee/comment-baseline.json` holds `{"files": {path: count}}`,
keys `/`-separated on every platform, written sorted from a `BTreeMap` so the
bytes are identical across runs. `bee dev comment-baseline --check` reports every
file above its baseline and fails; `--write` lowers and drops entries and REFUSES
to raise one, naming the file. Both are source-checkout-only verbs. The seed is
taken once from the per-file MAXIMUM over main and every unmerged `wt/*` branch
(0add770d), so a sibling feature that merges later cannot push a file above its
baseline and turn CI red; after those merges, `--write` lowers every entry that
now sits above the tree. A count that rose from a merge is fixed by deleting the
merged comment lines, never by raising the entry. The declared suite carries the
fence as the bin unit test
`devtools::comment_baseline::tests::every_code_file_is_at_or_below_its_comment_baseline`,
and the end-to-end refusal as
`a_comment_guard_deny_reaches_the_host_as_exit_two_on_stderr`
(`tests/hook_contracts.rs`). The baseline and its test are this repository's own
and never ship to a host.

<!-- bee:not-a-deferral: D6 records the batched cleanup as separate grooming work; nothing here promises a date -->
The comment lines that already exist stay for now: removing them is separate,
batched grooming work, module by module, and each batch moves every un-homed why
into `docs/knowledge` or the decision log before it deletes the comments and
lowers the baseline (69326d15).
<!-- /bee:not-a-deferral -->

## Open Gaps

- An over-long heading containing any non-ASCII character reaches a branch that
  hands the request off to a runtime that no longer exists, so nothing repairs
  it and nothing refuses it in this guard's own terms. The safe half is
  correct — such a heading is never truncated blindly — but the fall-through is
  a dead delegation signal, not a decision. Closing it means either proving a
  paired-character truncation matches the platform's, or turning the branch into
  an explicit refusal that tells the asker to shorten the heading itself
  (js-parity-cleanup D6 scoped the live delegation signals out and filed them;
  this is one of them).

## Pointers (implementation)

- Batch guard: `packages/bee/hooks/bee-write-guard.mjs` (`extractApplyPatchTargets`).

- CLI-shape guard incl. 3-token verb resolution: `packages/bee/hooks/bee-write-guard.mjs`
  against the `command-registry.mjs` catalog. Evidence: `.bee/cells/du-2.json`,
  `docs/history/dispatcher-unify/`.

- Help exemption: `check_cli_shape` in
  `packages/bee-rs/crates/bee/src/hooks/cli_shape.rs` breaks out of the segment
  when `parse_cli_flags` produced a `help` key, before `validate`. No registry
  entry declares a `help` property, so the key can only be a help request.
  Test: `asking_a_subcommand_for_its_help_reaches_the_help_surface`
  (same file). Provenance: cell `chsg-1`, commit 8dd2e846.

- Deep command reading: `tokenize_deep` / `expand_wrappers` /
  `is_wrapper_shell_name` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs`, consumed by
  `find_git_invocations` (`paths.rs`), `checks.rs`, and `detectors.rs`. Tests:
  `write_guard/tests.rs` (`sh_bash_eval_wrapper_around_a_git_verb_is_now_refused`,
  `nested_wrapper_still_refuses`, `tokenize_deep_never_expands_a_quoted_span_that_is_not_a_wrapper_payload`,
  `tokenize_deep_bounds_recursion_and_flags_truncation`). Provenance:
  `.bee/cells/archive/guard-parser-depth/gpd-1.json`, commit 98888896. HEREDOC bodies are fenced BEFORE
  that deep read (`fence_heredocs`, ahead of `tokenize_deep` in
  `extract_bash_targets`): `<<`/`<<-` with quoted, unquoted, and dash
  terminators, several heredocs per line in operator order, unterminated
  bodies failing safe as skipped content — heredoc CONTENT can never become
  an extraction target, while the operator's neighboring real redirects keep
  byte-identical extraction and `<<<` here-strings are untouched
  (guard-heredoc-fencing cell ghf-1, 2026-08-11; friction row 570's residue —
  a body word had been denied as a write target repeatedly, including a
  live cells-add refusal over the word "it").

- Question-schema guard + auto-fix: `check_ask_user_question` in
  `packages/bee-rs/crates/bee/src/hooks/write_guard/detectors.rs`
  (`AskResult::{Allow, Deny, Fixed}`); verdict emission in the same module's
  `main.rs` (`fixed_ask` branch — stdout JSON `hookSpecificOutput` with
  `permissionDecision: "ask"` + `updatedInput`, exit 0; deny path unchanged,
  exit 2 + stderr). The advisory reservation branch below it emits
  `additionalContext` only, no verdict. Tests: `write_guard/tests.rs`
  (`ask_long_header_is_auto_fixed`, `intent_reservation_allows_with_warning`).
  Provenance: `.bee/cells/ag-1.json`, commit 52dad26 (Node original).
- Heading counting unit and the ASCII-only repair (B28/R27):
  `textutil::utf16_len` (`packages/bee-rs/crates/bee/src/textutil.rs:51`) called
  at `hooks/write_guard/detectors.rs:159` — the crate's only caller of that
  helper, with the reason written beside the call; the repair's ASCII guard and
  the delegation fall-through are at `detectors.rs:201-207`, the rewrite itself
  at `detectors.rs:207-215`. Full text-measurement rule:
  `areas/rust-runtime/text-measurement-and-the-two-counting-units.md`. Evidence:
  trace `.bee/cells/archive/js-parity-cleanup/jp-9.json` (full suite 1006 passed, 0 failed, 2026-08-04);
  the finding that produced it is judge decision bc2e2d44.

- Comment arm and ratchet: the code-root, code-file and comment-line predicates
  plus the added-lines diff live in `packages/bee-rs/crates/bee/src/comments.rs`;
  the verb, the seed, the lower-only rule and the ratchet test live in
  `packages/bee-rs/crates/bee/src/devtools/comment_baseline.rs`; the committed
  counts live in `.bee/comment-baseline.json`. The arm itself is the
  `no_code_comments` branch of `run_native_with_roots` with
  `first_added_comment` / `comment_guard_denial`
  (`packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs`), fed by
  `reconstruct_target_text` (same file), `heredoc_writes`
  (`write_guard/guards.rs`) and `apply_patch_added_lines`
  (`write_guard/detectors.rs`). Tests: the `comment_guard_*` cases in
  `write_guard/tests.rs`, `a_comment_guard_deny_reaches_the_host_as_exit_two_on_stderr`
  (`tests/hook_contracts.rs`), and the ratchet in `comment_baseline.rs`'s own
  test module. Provenance: feature `no-code-comments`, cells ncc-1 through ncc-4
  (`docs/history/no-code-comments/CONTEXT.md`).
