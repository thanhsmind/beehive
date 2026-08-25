# Review r3 — the post-freeze delta (user-invoked, 2026-08-25)

**Why this review exists.** The r2 panel froze its diff at `7b14a72f` —
**before** the four commits that fixed its own P1/P2 findings landed
(`06e95097`, `00fd53b6`, `c2ef2f9f`, `a7338b24`), and before the hand-resolved
merge of the one-line generated registry payload (`3fde4c85`). None of that had
independent eyes until now. Scope: those five commits at HEAD, verification of
the five filed P2s, and a fresh hole-hunt over the shipped role surface.

**Panel.** Two independent review-tier readers, dispatched through
`bee dispatch prepare`, neither shown session history, plus live probes of the
installed binary from the orchestrator. Every finding below was re-verified
against the code at its cited anchors before filing.

---

## The four fixes: all four are real closes, each pinned by a named test

- **06e95097 (P1-A, escalation bypass via `role: "ceiling"`)** — closed.
  Every other reader of the escalation word was swept; none reads it off a
  declared role. Pinned by
  `the_ration_and_the_dispatch_agree_on_which_cells_are_escalated`
  (`drivers/tests.rs:1963`), which fails on revert.
- **00fd53b6 (P1-B, backfill scan outside its lock)** — closed. The re-read
  loop covers both the active and archive paths under the same
  `cells-archive` lock every other writer takes; `write_cell` is the only
  writer of an active cell file. Pinned by three tests
  (`cells/tests.rs:8347/:8381/:8414`), each failing on revert.
- **c2ef2f9f (unconfigured advisor admitted; `code` walked past the two-agent
  check)** — closed. Exactly two legality doors remain and both ask
  `known_role_named`; the FIX list is derived from the same set, so the
  refusal cannot advertise a name it would refuse. Pinned at
  `model_guard.rs:2482/:2392` and the alias sweep at `:1614-1646`.
- **a7338b24 (null slot counted as configured)** — closed for the exact
  spelling; `role_is_declarable` is the single null-vs-absent decision.
  Pinned at `model_guard.rs:2525` and `prepare.rs:2640/:2695`.

**The registry hand-merge (`3fde4c85`)** — clean: 174 commands, 174 unique,
`escalate`/`backfill-roles` present, `cells tier` gone.

## Spec axis

### P2 · `bee cells escalate --off` is a silent no-op on a legacy `tier:"ceiling"` cell
Found independently by both reviewers; corroborated facets. `--off` removes
only the `escalate` key (`handlers_close.rs:1186-1190`); `cell_is_escalated`
(`validate.rs:108-113`) still answers true off the legacy string, and nothing
in the tree ever clears `tier`. The disarm reports success while the cell
keeps burning the ration, keeps dispatching on the session model, and the
preamble keeps showing it escalated. This hits exactly the 20 live cells the
D9 backfill converted. Second facet: a later `backfill-roles` run re-derives
the flag from the tier and re-arms a disarmed cell. The agreement test cannot
catch it — ration and dispatch agree on the wrong answer.

### P2 · `dispatch.prepare`'s registry description still teaches the retired cost-tier model
`commands[135].description` says kind `cell` "resolves the generation tier";
it resolves the cell-role-headed list since the split. The `role` parameter's
own text is accurate; the top-level prose was never rewritten — and this is
the surface agents read through `bee --help --json`.

### Verified still open (filed by r2, unchanged at HEAD)
- `cell_role_list("code")` → `["code","code","generation"]` — double warn,
  first copy names the name that just failed (`prepare.rs:98`).
- `host_opted_into_roles` null boundary unpinned — no null-valued fixture
  exists, so narrowing `.is_some()` still leaves the suite green
  (`models.rs:93-97`, fixtures `tests.rs:6023-6074`).
- `commands.test` still lacks `--no-fail-fast`.

### Downgraded
- The workers.rs refusal: garble fixed, `--tier` deliberate and recorded;
  residual is only that it calls a role an "invalid tier". P2 → P3.

## Standards axis

### P3 · A configured `models.<rt>.ceiling` key makes bee prepare a payload its own guard refuses
Introduced by the P1-A fix. Decision 0015 forbids the key; nothing enforces
it, and with it present a `role:"ceiling"` cell resolves to marker + model
param — the exact pair the guard's own test denies. Related authoring gap:
`role:"ceiling"` is still accepted at `cells add` with no nudge toward
`bee cells escalate`.

### P3 · A mis-cased `"Advisor"` key splits the doors the null fix unified
`role_is_declarable` matches the advisor arm by exact name while
`known_role_named` matches case-insensitively and `resolve_advisor` reads the
exact lowercase key — one-question-two-answers, reopened by a typo'd key.

### P3 · A role-keyed fallback chain for a fell-through role is silently dead
The chain is keyed on the resolved marker role, not the asked one, so
`fallbackChains["test"]` on a host not configuring `test` is accepted, stored,
and can never fire — with no signal to the operator.

### P3 · backfill counts mix pre-lock and post-lock provenance
Writes correct; two of five counts and the doc claim describe the unlocked
scan.

## What held, verified fresh

Role resolution live on main (`--role extraction` → `bee-extract`/`sonnet`);
the unconfigured explicit `--role` refusal with its typed FIX; the
role-required refusal at `cells add` with the full teach line; no
store-reaching bypass of the role requirement through any CLI door; the D11
error gate serialized on every chained payload in both directions; an
escalated cell's session-model pin unreachable by any chain; full suite green
but for the two `node`-missing environmental failures.

---

6 finding(s) — P1 0, P2 2, P3 4 · axis: spec 2, standards 4.
