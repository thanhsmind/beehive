# orient-host-playbook — plan

Lane: small. Class: bugfix. Architect skipped: no new data shape — D1 adds
a path lookup to an existing packet field, D2 adds one advisory check beside
an existing one, D3 is one doc sentence.

## Throughput

- Blocking first steps: none.
- Independent workstreams: orient (ohp-1), the Stop check (ohp-2) and the
  skill text (ohp-3) touch disjoint files and run in parallel.
- Shared mutable state: none.
- Smallest safe decomposition: three implementer cells; ohp-1 and ohp-2 each
  get a tester cell (ohp-4, ohp-5); ohp-3 is docs and gets its parity check.

## Cells

| Cell | Role | Does | Decisions |
|---|---|---|---|
| ohp-1 | code | orient resolves real playbook paths and ends with a next line | D1 |
| ohp-2 | code | Stop hook warns on unnamed principles and an unopened playbook | D2 |
| ohp-3 | docs | Communication contract close carries a Principles line | D3 |
| ohp-4 | test | proves orient paths in host-shaped temp repos | D1 |
| ohp-5 | test | proves both Stop warnings on transcript fixtures | D2 |

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | orient prints a fixed skills/ playbook path | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:741 | packet.insert("playbook".into(), json!(format!("skills/bee-planning/playbooks/{class}.md"))); |
| 2 | situation lines use the same fixed root | read | packages/bee-rs/crates/bee/src/verbs/status_full/orient.rs:757 | "situation: skills/bee-planning/playbooks/{class}/{}.md — {}", |
| 3 | onboarding installs skills under .claude/skills | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:269 | let dir_rel = format!(".claude/skills/{id}"); |
| 4 | the Stop hook has a reply check to model on | read | packages/bee-rs/crates/bee/src/hooks/session_close/claims.rs:34 | pub(crate) fn unverified_agreement_warning(events: &[Value]) -> Option<String> { |
| 5 | that warning joins the Stop output parts | read | packages/bee-rs/crates/bee/src/hooks/session_close/mod.rs:178 | .then(\|\| perf_events.as_deref().and_then(unverified_agreement_warning)) |

## Proof

Each cell runs its filtered cargo tests or parity checks. The full suite
runs once with `bee test` before `bee worktree merge`.
