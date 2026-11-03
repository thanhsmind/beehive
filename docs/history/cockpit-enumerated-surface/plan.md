---
artifact_contract: bee-plan/v1
mode: high-risk
---

# Plan: Cockpit Enumerated Surface

Mode: `high-risk` — 3 risk flags: audit-security (hard gate), public-contracts,
covered-contract-change.

Why this is the least workflow that protects the work: the code change is one
function returning different strings, but it is the privilege boundary of two
autonomous agents that run with `bypassPermissions` inside a host repository.
The consult then proved the shape needed splitting — the net goes in before the
strings move.

## Requirements (from CONTEXT.md)

- **D1** — enumerated surface, never a CLI wildcard; neither role read-only.
- **D2** — the two surfaces DIFFER: dispatch cannot merge or gate; merge cannot
  create worktrees or start agents.
- **D3** — every entry traces to a real call site; prose is not a call site.
- **D4** — a forbidden-token test guards each surface.
- **D5** — `herding enable`/`disable` excluded from both, and named forbidden.

## Load-bearing claims

Labels: `read` = opened that file at that line and saw those bytes; `ran` =
executed and hold the output. Evidence is a verbatim byte substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Both cockpit panes hold the whole-CLI wildcard today | read | `packages/bee-rs/crates/bee/src/herding/control_loop.rs:277-286` | `Bash(.bee/bin/bee:*)` |
| 2 | A locked owner decision already mandates the enumerated surface | read | `.bee/decisions.jsonl` herding-adopt D7-FINAL | `the two CONTROL panes are narrowed to an enumerated command surface, never to "read-only"` |
| 3 | The enable verbs are owner-typed only, and nothing enforces it | read | `docs/knowledge/areas/bee-herding/overview.md:11` | `CLI verbs stay owner-typed only, never called by bee automation` |
| 4 | Verb-level granularity is expressible and already shipped | read | `control_loop.rs:299-303` | `Bash(.bee/bin/bee state session list:*),` / `Bash(.bee/bin/bee herding pane list:*),` |
| 5 | bee's own test comment documents this defect | read | `control_loop.rs:946-948` | `and not the whole-CLI wildcard the` / `cockpit roles carry.` |
| 6 | **My first merge list was wrong**: its `worktree new` and `agent-start` hits are the role-boundary PROHIBITION, not call sites | read | `skills/bee-herding/references/role-merge.md:12-13` | `About to run \`bee worktree new\`` / `or \`bee herding agent-start\`? Stop — that is dispatch's job.` |
| 7 | The merge pane's real durable-stop write is a compound `mkdir` + `touch`, so both need entries or the red marker never lands | read | `skills/bee-herding/references/role-merge.md:177` | `mkdir -p .bee/tmp && touch .bee/tmp/bee-herding.red.<slug>` |
| 8 | A WIDE `Bash(touch:*)` would make D5 false — the enable marker is itself a touch | read | `skills/bee-herding/references/role-dispatch.md:224` | `a \`touch\` of \`<main-root>/.bee/tmp/bee-herding.enable\`` |
| 9 | The dispatch role's own verbs all trace to call sites; none is prose | ran | `rg -o 'bee [a-z][a-z-]*( +[a-z][a-z-]*)?' skills/bee-herding/references/role-dispatch.md \| sort \| uniq -c` | `36 herding pane` |

## Discovery

The plan-step consult overturned the first shape. Three findings changed it.

**The same false positive I had just written a decision about, I then committed
myself** (claim 6). D3 exists because `bee gate` and `cells finish` in
`role-merge.md` are prose; I caught those by reading, then trusted a raw count
for the rest and put `worktree new` and `herding agent-start` on the merge
surface. Their only appearance is the line telling merge *not* to run them. A
merge pane carrying them can dispatch — the exact boundary D2 is for. Recorded
as a mistake at the moment it surfaced, not composed at the end.

**Two honesty limits the tests must state rather than paper over.** The
allowlist grammar is plain string-prefix matching, and in `git -C <path> <verb>`
the mutable path precedes the verb — so "dispatch may not write with git" is
**not mechanically enforceable** this pass, and the test file must say so
instead of shipping an untested claim. Separately, the multiplexer entry
(`Bash(herdr:*)` / `Bash(tmux:*)`) is a raw write surface wider than much of
what is being removed, but dropping it collides with a locked transport
decision — it stays this pass, named, not silently kept.

**The remedy has to ship with the refusal.** A too-narrow surface stops the
dispatch pane in an unattended run, and bee has paid for that three times. The
working 3am remedy needs no rebuild: a literal allowlist in
`herding.control_command`. It is proven by an existing substitution test and
gets its own test here.

## Approach

Two cells, net first — the consult's own recommendation and bee's
arm-the-remedy-first pattern.

**Cell 1 — the net.** A prefix-matcher simulation in the test module, plus a
doc-scrape completeness test: extract every fenced command from both role
prompts, normalise the bare `bee ` spelling, and assert each is admitted by its
role's CANDIDATE surface, carried as a fixture. Production strings do not move.
Whatever the scrape cannot admit is a recorded finding, never a deleted case.

**Cell 2 — the flip.** Move the candidate surfaces into `allowed_tools_for`,
add per-role forbidden-token constants, and add the config-override remedy test.
With cell 1 green this is a small change under a proven net.

Rejected: one cell doing both (the surface and its guard would land together
with nothing having tested the guard against the old world first); narrowing
`git -C` by interpolating the root (half-closes the hole, breaks the
`&'static str` shape and the byte-exact argv tests); dropping the multiplexer
wildcard (collides with tmux-herding-cockpit D1, earns its own decision).

Corrections folded into the candidate surfaces: merge loses `worktree new` and
`herding agent-start`; merge gains `ls`, `mkdir -p .bee/tmp`, and a
red-marker-scoped `touch`; both gain `.bee/bin/bee --help` so a denied verb's
remedy is reachable; `backlog pbi` narrows to `backlog pbi list`.

Risk map: cell 1 LOW (tests only). Cell 2 MEDIUM — a missing verb stops the
pane, mitigated by cell 1's scrape and the override remedy.

## Test matrix

| Dimension | Probe | Cell |
|---|---|---|
| Completeness | every fenced command in each role prompt is admitted by that role's surface | 1 |
| Matcher honesty | the simulation is documented as an approximation of the runtime rule | 1 |
| Prohibition, dispatch | no `worktree merge`, `state gate`, `herding enable\|disable`; and dispatch carries no `touch` at all | 2 |
| Prohibition, merge | no `worktree new`, `herding agent-start`, `state gate`, `herding enable\|disable`, `cells finish` | 2 |
| Prohibition, both | neither surface contains `Bash(.bee/bin/bee:*)` | 2 |
| The enable marker | `touch .bee/tmp/bee-herding.enable` is refused by BOTH surfaces — the probe that fails if `touch` is left wide | 2 |
| Stated limit | a test names, in prose, that `git -C:*` leaves dispatch's git write unenforced | 2 |
| Remedy | a literal allowlist in `herding.control_command` reaches the spawned argv | 2 |
| Both transports | every assertion holds for `Herdr` and `Tmux` | 1, 2 |
| Observer untouched | supervisor surface, constant and test byte-unchanged | 2 |

Before any unattended interval run: one supervised `--role dispatch --once`
through a real spawn, and one `--role merge --once` over a staged finished
worktree and a staged red merge. A dry run stops before the spawn and proves
nothing about the spawn-path entries.

## Open Questions

- `(cd <path> && bee orient --json)` (role-dispatch.md:156) matches no prefix
  entry, and does not match today's wildcard either. Either the panes already
  eat a denial here or the runtime admits compound commands some way neither
  the consult nor I have proven. Cell 1's scrape surfaces it as a finding;
  resolving it may belong to a role-prompt change, which this feature's
  boundary excludes.
- The bare `bee` versus `.bee/bin/bee` spelling in the role docs has the same
  shape and the same answer: record, do not silently widen to `Bash(bee:*)`.

## Out of scope

- Closing the `git -C` width — needs a read-only bee verb wrapping the worktree
  reads; recorded as the real fix.
- Dropping the multiplexer wildcard — needs its own decision.
- The working agent panes' posture: an owner-accepted risk, untouched.
