# Bug triage

A consolidated list of the defects and inconsistencies the feature documents raised in their bodies and "Open questions and verification" sections. Each entry is read from the beehive repo at commit `6b0ae488` and its tests; entries confirmed against the built binary during drafting carry a **Status** line. The list exists so the bee team can decide, item by item, whether to fix, to document as intended, or to leave.

## Summary

Roughly sixty items were raised across the thirty-four documents; they merge to twenty-four entries — six high, fourteen medium, four low. The two largest clusters share one root each: the retirement of the Node delegation path left many semantic refusals falling through to a generic (and sometimes false) message, in one case reporting failure for work that succeeded (B-01); and the port left a tail of advertised-but-unbuilt commands with live documentation still pointing at them, including one shipped skill that cannot run (B-04). The high entries have one thing in common: the product silently does something different from what its own text promises.

| ID | Title | Severity | Area | Decision needed |
| --- | --- | --- | --- | --- |
| B-01 | Retired-delegation refusals collapse into a false generic message; one path reports failure after the write succeeded | high | invocation-wide | fix |
| B-02 | The secret guard can be walked around with Bash, and never checks paths outside the repo | high | privacy | fix |
| B-03 | A boolean flag spelled `--flag=true` parses as false, turning read-only spellings into writes | high | invocation-wide | fix |
| B-04 | Advertised commands are not built, and shipped docs and skills still tell the agent to run them | high | command surface | product call |
| B-05 | The review evidence preflight records `passed: true` unconditionally | high | reviews | fix |
| B-06 | The mailbox letter promises five sections; the code can only ever print two | high | mailbox | fix |
| B-07 | The bypass banners overstate what stops: UAT and secret reads and the review P1 door never read the bypass level | medium | gates | fix |
| B-08 | `reviews record` can lose a concurrent finding | medium | reviews | fix |
| B-09 | The "expired but never released" warning fires on every live reservation | medium | status | fix |
| B-10 | Performance-log writer diverges from its recorded rules (branch always null, cache gate gone, lost-row race) | medium | perf | fix |
| B-11 | `bee tmp sweep` cannot reach a granted worktree's scratch, which the write guard steers writes into | medium | maintenance | fix |
| B-12 | Store files the workflow trusts are hand-writable: `test-results.json` (read by the red-base check) and `decisions.jsonl` | medium | store | product call |
| B-13 | Two refusals name `--session-id` on a command whose flag is `--session` | medium | reservations | fix |
| B-14 | A typed `dispatch prepare` refusal exits 0; only the payload's `ok: false` signals it | medium | dispatch | product call |
| B-15 | The frontier definition diverges: the skill says unreserved, the CLI reads only `claimed-by:` | medium | discovery | product call |
| B-16 | A released session no longer blocks writes but still blocks a sibling's `start-feature` | medium | sessions | fix |
| B-17 | Stale help, registry, and knowledge text misleads the agent | medium | docs drift | fix |
| B-18 | The blocking hooks are not uniformly panic-safe | medium | failure | fix |
| B-19 | The concurrent-worker git guard's prescribed recipe is refused by the idle intake gate | medium | guards | fix |
| B-20 | `bee mailbox mark` resolves the worktree root while letters are filed at the control root | medium | mailbox | fix |
| B-21 | There is no `bee --version` | low | invocation | product call |
| B-22 | `discovery list` prints only the first line of a hard-wrapped Destination | low | discovery | fix |
| B-23 | Small copy and rendering slips (grouped) | low | various | fix |
| B-24 | Small dead or inert surfaces (grouped) | low | various | product call |

## High

### B-01: Retired-delegation refusals collapse into a false generic message; one path reports failure after the write succeeded

- **Where the agent meets it:** any semantic mistake on several verbs: `capture add` with a missing, blank, high-risk-lane, or secret-shaped input; `decisions active --since 20260826`; `backlog propose` with a blank `--story`; several `dispatch prepare` and `knowledge` shapes; running some verbs from a linked worktree.
- **What happens / what was expected:** the answer is the catch-all `bee: unsupported argument shape …. Its required arguments are all present …` — which is false when the required flag is the thing missing, and names no real cause. Expected: the verb's own refusal naming the actual rule, as `capture flush` already does after its "CUTOVER FIX". Worst case: `decisions log` re-reads the store *after* its append; when that read falls outside the modeled region it prints the generic refusal and exits 1 — failure reported for a decision that is already durably on disk.
- **Reproduce:** `bee capture add --outcome "   "` → generic refusal; `bee capture add` → generic refusal claiming required arguments present; `bee decisions active --since notadate` → generic refusal.
- **Why (from the code):** `verbs/capture.rs:296-323` (`return None` paths whose comment says Node owns the error text), `verbs/decisions/mod.rs` (post-append active-set re-read), the same pattern across the groups; the catch-all text at `router.rs:320-326` assumes the registry's `required` list is truthful, and `registry_payload.json` marks `capture.add` as requiring nothing.
- **Severity:** high — a false claim in an error message, and one path that reports failure for a success.
- **Decision needed:** fix — give each surviving `return None` refusal a native message (the `capture flush` precedent), and make the catch-all stop asserting "required arguments are all present" when it cannot know.
- **Raised by:** [memory/capture.md](memory/capture.md#open-questions-and-verification), [foundations/invocation.md](foundations/invocation.md#open-questions-and-verification), [memory/decisions.md](memory/decisions.md#open-questions-and-verification), [memory/backlog.md](memory/backlog.md#open-questions-and-verification), [memory/knowledge.md](memory/knowledge.md#open-questions-and-verification), [delegation/dispatch.md](delegation/dispatch.md#open-questions-and-verification)
- **Status:** confirmed against the built binary, 2026-08-26, for the capture, decisions `--since`, and backlog `--story` cases.

### B-02: The secret guard can be walked around with Bash, and never checks paths outside the repo

- **Where the agent meets it:** reading a credential file.
- **What happens / what was expected:** `Read .env` is denied (exit 2, privacy marker); `Bash: cat .env` passes with exit 0; a Read of `/etc/ssl/private/x.pem` or `../outside/.env` passes. Expected: the same file shape denied through every tool, wherever it lives.
- **Reproduce:** in a hooked session, run `cat .env` via Bash; compare with a Read of the same file.
- **Why (from the code):** the secret patterns are applied to read-tool targets (`hooks/write_guard/guards.rs:168-234`); Bash target extraction feeds the write-side checks, not the secret check, and containment scoping skips out-of-repo paths for reads.
- **Severity:** high — the one guard that exists specifically for the human's secrets has a first-class bypass in the most common tool.
- **Decision needed:** fix — run the secret patterns over Bash-extracted read targets and over absolute paths; or document the boundary loudly if the narrower scope is intended.
- **Raised by:** [cross-cutting/privacy.md](cross-cutting/privacy.md#open-questions-and-verification)
- **Status:** confirmed live in a hooked session, 2026-08-26.

### B-03: A boolean flag spelled `--flag=true` parses as false, turning read-only spellings into writes

- **Where the agent meets it:** `bee knowledge index --check=true` renders (writes) all index files; `bee backlog render --write=true` silently writes nothing and exits 0 — each the exact opposite of the spelling's intent.
- **What happens / what was expected:** a value-form boolean passes shape validation but is not `=== true`, so it reads as false. Expected: `--flag=true` ≡ `--flag`, or a refusal naming the accepted spelling.
- **Reproduce:** `bee knowledge index --check=true` in a repo with a bundle; watch it take the write path.
- **Why (from the code):** the shared flag parser accepts the `=value` form for booleans but only the bare form sets true (parser comment in `verbs/backlog.rs`; observed live for `knowledge index`).
- **Severity:** high — a spelling any agent would write, silently inverting read/write intent in both directions. The behavior is not even uniform: `bee status --json=true` *refuses* (exit 1) while `--check=true` and `--write=true` are accepted and read as false, so an agent cannot learn one rule.
- **Decision needed:** fix — treat `=true`/`=false` literally or refuse the value form.
- **Raised by:** [memory/knowledge.md](memory/knowledge.md#open-questions-and-verification), [memory/backlog.md](memory/backlog.md#open-questions-and-verification)
- **Status:** confirmed live for `knowledge index --check=true` (write path taken; output happened to be byte-identical), 2026-08-26.

### B-04: Advertised commands are not built, and shipped docs and skills still tell the agent to run them

- **Where the agent meets it:** `bee config get/set/unset/validate`, all seven `bee perf` verbs, `bee recovery window`, `bee herding enable`/`disable`, the three `bee state compact-*` verbs, and the cross-repository arm of `feedback collect/rank` (non-empty `dogfood_repos`) all answer `bee: not built into this binary` (or the generic shape refusal). Meanwhile: `.bee/config-sample.json` tells the reader to run `bee config set`; `skills/bee-hive/references/scout-and-ticks.md` tells the agent to mine with `recovery window`; `skills/bee-evolving/SKILL.md` still invokes `node .bee/bin/bee.mjs feedback rank` on a retired runtime; `docs/knowledge/areas/feedback-digest/cross-repo-trust-boundary.md` documents the unbuilt arm as live.
- **What happens / what was expected:** an agent following bee's own text meets a refusal; `bee-evolving` cannot run at all.
- **Why (from the code):** registry entries carry `unavailable` for some but not all of these; the pointers were not updated when the Node runtime retired.
- **Severity:** high — bee's own instruction layer directs agents into dead ends; one shipped skill is broken.
- **Decision needed:** product call per group — build or retire each; either way, fix every pointer (the sample config, the two skills, the knowledge area).
- **Raised by:** [cross-cutting/configuration.md](cross-cutting/configuration.md#open-questions-and-verification), [observability/perf.md](observability/perf.md#open-questions-and-verification), [maintenance/recovery.md](maintenance/recovery.md#open-questions-and-verification), [memory/feedback.md](memory/feedback.md#open-questions-and-verification), [delegation/herding.md](delegation/herding.md#open-questions-and-verification)
- **Status:** confirmed live for the config and perf groups, 2026-08-26.

### B-05: The review evidence preflight records `passed: true` unconditionally

- **Where the agent meets it:** `bee reviews create`. The registry entry and the knowledge area (workflow-state R9) promise it "fails closed with zero files written on missing evidence".
- **What happens / what was expected:** the preflight only auto-excludes open/claimed cells and refuses an unresolvable id, then writes `"passed": true` regardless; nothing reads the field afterward. Expected: the promised evidence check, or an honest field.
- **Why (from the code):** `verbs/reviews.rs` — the preflight insertion sets `passed` to a literal `true`.
- **Severity:** high — a recorded "passed" that no check produced, in the flow whose whole point is independent verification.
- **Decision needed:** fix — implement the check or remove the field and correct the registry and R9.
- **Raised by:** [reviews/reviewing.md](reviews/reviewing.md#open-questions-and-verification)

### B-06: The mailbox letter promises five sections; the code can only ever print two

- **Where the agent meets it:** the letter a human reads after an unattended run.
- **What happens / what was expected:** a locked decision defines five body sections; `Broken or unfinished` has a constant (`KIND_BLOCKER`) with no call site — `bee cells block` appends no entry — and `needs_you` is set to an empty list at both wired stops, so `Needs your call` and `Next` can never render either. The very things an unattended run most needs to tell its human are the ones that cannot print.
- **Why (from the code):** `verbs/mailbox.rs` — `KIND_BLOCKER` unused; both composition sites pass empty `needs_you`.
- **Severity:** high — the human's one window into an unattended run silently omits blockers and open calls.
- **Decision needed:** fix — wire `cells block` (and the needs-you sources) to append entries, or re-lock the decision at two sections.
- **Raised by:** [memory/mailbox.md](memory/mailbox.md#open-questions-and-verification)

## Medium

### B-07: The bypass banners overstate what stops

- **What happens:** the `total` banner says "NO human checkpoint remains" and the secret-read line drops at `full`/`total`; but `set_gate.rs` refuses `--name uat --actor auto` at every level, the write guard never reads `gate_bypass` at all (secret reads deny at every level), and `reviews.rs` reads no bypass level (an open P1 refuses approval everywhere) though the `full`/`total` banners claim a P1 auto-proceeds.
- **Why:** banner text in `session_preamble/state.rs:130-137` versus the three enforcement sites.
- **Severity:** medium — the enforcement is safe; the promise is wrong, in the direction of overclaiming autonomy.
- **Decision needed:** fix the banners.
- **Raised by:** [foundations/gates.md](foundations/gates.md#open-questions-and-verification), [reviews/reviewing.md](reviews/reviewing.md#open-questions-and-verification), [cross-cutting/configuration.md](cross-cutting/configuration.md#open-questions-and-verification), [cross-cutting/privacy.md](cross-cutting/privacy.md#open-questions-and-verification)

### B-08: `reviews record` can lose a concurrent finding

- **What happens:** read, mutate, rewrite whole file — no store lock, no append. Two concurrent `--kind finding` calls can drop one. The only bee write path observed that neither locks nor appends.
- **Why:** `verbs/reviews.rs` record path.
- **Severity:** medium — multi-reviewer flows are exactly the design intent of the review pass.
- **Decision needed:** fix — take a named lock like every other record writer.
- **Raised by:** [reviews/reviewing.md](reviews/reviewing.md#open-questions-and-verification)

### B-09: The "expired but never released" warning fires on every live reservation

- **What happens:** `bee status` warns "N reservation(s) expired but never released" whenever any reservation exists. The port faithfully preserved a comparison that was always false in Node and is now always true: it counts every reservation whose `released_at` is null — which is all of them.
- **Why:** `status_full/build.rs:64-71`.
- **Severity:** medium — a standing false alarm trains agents to ignore the warning channel.
- **Decision needed:** fix — compare expiry against now.
- **Raised by:** [observability/status.md](observability/status.md#open-questions-and-verification)

### B-10: Performance-log writer diverges from its recorded rules

- **What happens:** `branch` is hard-coded null (R6 says every entry carries project and branch); the R10 scan-cache gate is gone (the HTML re-renders whenever the log is non-empty); and the read-filter-rewrite upsert has no lock, so two sessions closing at once can lose a row.
- **Why:** `hooks/session_close/perf.rs:444` and the rollup path.
- **Severity:** medium — telemetry only, but it is the cross-project comparison surface.
- **Decision needed:** fix, or re-record R6/R10 to match.
- **Raised by:** [observability/perf.md](observability/perf.md#open-questions-and-verification)

### B-11: `bee tmp sweep` cannot reach a granted worktree's scratch

- **What happens:** the scratch-shape guard steers ephemeral writes into `.bee/tmp/` on the promise the sweep clears them; the sweep resolves through the ordinary root door, refuses inside a granted worktree, and from main sees only main's roots — so a granted worktree's scratch accumulates forever.
- **Why:** `verbs/` tmp sweep root resolution versus `roots.rs`'s granted split.
- **Severity:** medium — a slow leak with no owner.
- **Decision needed:** fix — let the sweep run in (or reach into) granted worktrees.
- **Raised by:** [maintenance/recovery.md](maintenance/recovery.md#open-questions-and-verification)

### B-12: Store files the workflow trusts are hand-writable

- **What happens:** `.bee/logs/test-results.json` is exempt from the scratch-shape guard and absent from the direct-edit table, yet the red-base claim check trusts it — a fake green record skips the `--fix-first` discipline. `decisions.jsonl` — the durable decision record — is likewise not in the deny table, though every rendered surface has drift checks and the store has none.
- **Why:** the deny table at `hooks/write_guard/guards.rs:68-88` covers neither file.
- **Severity:** medium — integrity of two records other machinery reasons from.
- **Decision needed:** product call — guard them, or accept that logs and event stores are trust-the-agent surfaces and say so.
- **Raised by:** [maintenance/testing.md](maintenance/testing.md#open-questions-and-verification), [foundations/store.md](foundations/store.md#open-questions-and-verification), [memory/decisions.md](memory/decisions.md#open-questions-and-verification)

### B-13: Two refusals name `--session-id` on a command whose flag is `--session`

- **What happens:** the reservations `SESSION_REQUIRED` refusal and the `shared-disjoint` write-policy deny both print `bee reservations reserve … --session-id <id>`; that spelling gets a shape refusal. Following the remedy literally fails.
- **Why:** `verbs/reservations/reserve.rs:299`, `verbs/state_group/policy.rs:112-116`.
- **Severity:** medium — a deny's remedy is load-bearing by bee's own rules.
- **Decision needed:** fix the two strings.
- **Raised by:** [coordination/reservations.md](coordination/reservations.md#open-questions-and-verification)
- **Status:** flag mismatch confirmed against `bee reservations reserve --help`, 2026-08-26.

### B-14: A typed `dispatch prepare` refusal exits 0

- **What happens:** `claim_ownership`, `role_not_configured`, and the other typed refusals return `{"ok": false, …}` with exit 0, while a malformed call exits 1. Scripts and hooks branching on exit codes read a refused dispatch as success.
- **Why:** the typed-refusal emit path in the dispatch verbs.
- **Severity:** medium.
- **Decision needed:** product call — exit 1 on `ok: false`, or document the payload as the only signal.
- **Raised by:** [delegation/dispatch.md](delegation/dispatch.md#open-questions-and-verification)

### B-15: The frontier definition diverges between the skill and the CLI

- **What happens:** `bee-wayfinding` defines frontier tickets as open, unblocked, and *unreserved*; the CLI counts only the `claimed-by:` line and never consults the reservation store. A ticket reserved without the line still counts as frontier and can be double-taken.
- **Why:** `verbs/discovery.rs` ticket parsing.
- **Severity:** medium.
- **Decision needed:** product call — which definition is the contract.
- **Raised by:** [discovery/wayfinding.md](discovery/wayfinding.md#open-questions-and-verification)

### B-16: A released session no longer blocks writes but still blocks a sibling's `start-feature`

- **What happens:** the write guard reads a `closed`/`dead` owner as not live; `apply_write_policy`'s `is_owner_live` checks only heartbeat staleness and ignores `status`, so a just-released session unblocks writes but still refuses the sibling's isolation start for up to 15 minutes.
- **Why:** `verbs/state_group/policy.rs:155-160` versus `hooks/write_guard/checks.rs`.
- **Severity:** medium — the release verb's whole point is immediate hand-over.
- **Decision needed:** fix — make `is_owner_live` respect `released`/`closed`.
- **Raised by:** [coordination/sessions.md](coordination/sessions.md#open-questions-and-verification)

### B-17: Stale help, registry, and knowledge text misleads the agent

- **What happens**, one line each: the `finish` flow-spelling's registry entry lacks the `report`/`commit-pending`/`deviation` flags its implementation requires, and `--inline-reason` — named in a cap refusal's own remedy — is declared for no command at all; `bee finish --help` still claims it runs the declared tests and refuses on red, contradicting the no-door-runs-tests contract; `bee close --help` says tests is the one blocking door while the scribing-debt door also blocks; `state.session.bind`'s registry text denies a lane-existence check the code performs; `backlog counts --help` says counts come from `docs/backlog.md` (the fallback); `backlog add --help` describes an unbuilt `--queue-submit`; `knowledge report`'s registry omits the `evidence_ladder` it returns; `knowledge context`'s `unknown_work` remedy names only one of its four resolution rungs; the feedback area's overview names a nonexistent `bee feedback add`; `verify-pipeline/overview.md` names retired suite-running entry points; the onboarding registry claims the command installs the binary (the installer does); the preamble's drift line says "re-run onboarding" where a host repo without a source checkout meets `engine_not_found`.
- **Severity:** medium as a cluster — each is small; together they erode trust in the surfaces agents are told to rely on instead of guessing.
- **Decision needed:** fix — a text-only sweep.
- **Raised by:** [coordination/sessions.md](coordination/sessions.md#open-questions-and-verification), [memory/backlog.md](memory/backlog.md#open-questions-and-verification), [memory/knowledge.md](memory/knowledge.md#open-questions-and-verification), [memory/feedback.md](memory/feedback.md#open-questions-and-verification), [maintenance/testing.md](maintenance/testing.md#open-questions-and-verification), [maintenance/onboarding.md](maintenance/onboarding.md#open-questions-and-verification)

### B-18: The blocking hooks are not uniformly panic-safe

- **What happens:** the write guard wraps evaluation in `catch_unwind` ("a native panic is never a verdict"); the model guard — the other hook that can exit 2 — has no equivalent wrapper; `main()` has no top-level panic handler, so a verb panic would exit outside the documented 0/1/2/3 set. No reachable panic was found; the net is simply absent.
- **Why:** `hooks/write_guard/main.rs:20` versus `hooks/model_guard.rs:28`; `src/main.rs`.
- **Severity:** medium — a latent gap in the crash contract, not an observed failure.
- **Decision needed:** fix — extend the wrapper.
- **Raised by:** [cross-cutting/failure.md](cross-cutting/failure.md#open-questions-and-verification)

### B-19: The concurrent-worker git guard's prescribed recipe is refused by the idle intake gate

- **What happens:** at idle with multiple live workers, the guard's remedy names a temp-index recipe beginning `git read-tree`; the intake gate refuses `git read-tree` as an unmodeled mutation. The agent is stranded one step until it discovers the path-scoped `git commit -- <paths>` form both guards allow.
- **Why:** the remedy constant at `hooks/write_guard/paths.rs:365-372` versus the intake git classification at `checks.rs:712-737`.
- **Severity:** medium — both guards behave as designed; the composition dead-ends the named remedy.
- **Decision needed:** fix — model the recipe's read-only steps in the intake gate, or name the composed case in the remedy.
- **Raised by:** [foundations/guards.md](foundations/guards.md#open-questions-and-verification)
- **Status:** confirmed live in this repository, 2026-08-26.

### B-20: `bee mailbox mark` resolves a different root than the letter writers

- **What happens:** letters and entries land at the control root; `mark` resolves the worktree root, so from a linked worktree it reads an empty mailbox and can mark nothing.
- **Why:** `verbs/mailbox.rs:89-93` root door choice.
- **Severity:** medium.
- **Decision needed:** fix — one root door for the whole mailbox.
- **Raised by:** [memory/mailbox.md](memory/mailbox.md#open-questions-and-verification)

## Low

### B-21: There is no `bee --version`

- **What happens:** `bee --version` answers `bee: unknown command`. The version lives in the preamble and `status --json`.
- **Severity:** low. **Decision needed:** product call.
- **Raised by:** [foundations/invocation.md](foundations/invocation.md#open-questions-and-verification)
- **Status:** confirmed live, 2026-08-26.

### B-22: `discovery list` prints only the first line of a hard-wrapped Destination

- **What happens:** every hard-wrapped destination in beehive's own maps prints cut mid-sentence.
- **Why:** `verbs/discovery.rs` reads one line under `## Destination`.
- **Severity:** low. **Decision needed:** fix — join until the next heading.
- **Raised by:** [discovery/wayfinding.md](discovery/wayfinding.md#open-questions-and-verification)
- **Status:** confirmed live, 2026-08-26.

### B-23: Small copy and rendering slips

- The fail-open stderr line carries a run of ten spaces mid-sentence (`hooks/mod.rs:67`) — the line agents most often quote.
- Doctor's fourth row label overflows its 22-character alignment field; the report is visibly misaligned.
- The feedback digest prints its default path with a hardcoded backslash (`.bee\feedback-digest.json`) on every platform; the file lands correctly.
- `bee doctor` emits no timing line and no timings entry; every other served verb does. Some help forms log their command as `unknown` in the timing line.
- `bee onboard` prints its human report on stdout but a parse error as `{"error": …}` on stdout *without* `--json`, and skips the timing line — three contract deviations in one command.
- **Severity:** low, grouped. **Decision needed:** fix.
- **Raised by:** [cross-cutting/failure.md](cross-cutting/failure.md#open-questions-and-verification), [observability/status.md](observability/status.md#open-questions-and-verification), [memory/feedback.md](memory/feedback.md#open-questions-and-verification), [maintenance/onboarding.md](maintenance/onboarding.md#open-questions-and-verification), [foundations/invocation.md](foundations/invocation.md#open-questions-and-verification)

### B-24: Small dead or inert surfaces

- The feedback digest's `oversize` drop reason can never be produced; the cluster `key` (neutralization-stripped titles) still leaks into `--json`.
- The `hooks.codex-subagent-audit` config toggle is read by nothing.
- Skill frontmatter `missing_effect` is read by no code.
- `verbs/discovery.rs`'s header names a `type:` ticket key `parse_ticket` never reads.
- **Severity:** low, grouped. **Decision needed:** product call — delete or wire each.
- **Raised by:** [memory/feedback.md](memory/feedback.md#open-questions-and-verification), [cross-cutting/configuration.md](cross-cutting/configuration.md#open-questions-and-verification), [cross-cutting/skills-layer.md](cross-cutting/skills-layer.md#open-questions-and-verification), [discovery/wayfinding.md](discovery/wayfinding.md#open-questions-and-verification)
