# Port notes — pstack playbooks into bee

One section per ported playbook: each pstack tool swapped for its bee
counterpart, and each step dropped because bee has no counterpart (D2). This
file is the only place a pstack-only tool is named; no ported file names one. Source: pstack poteto-mode at commit `b9ddc83`, MIT, copyright 2026
Lauren Tan; the notice ships as `skills/bee-planning/playbooks/LICENSE.pstack`.

## bug-fix

Source `playbooks/bug-fix.md`, merged with the bee `bugfix` class playbook
(D3) into `skills/bee-planning/playbooks/bugfix/bug-fix.md`.

Tool swaps:

- The control skill (Non-negotiables) became the project's `verify-app` skill,
  or the real interface when the project has none.
- The `how` skill and the `why` skill became `bee-how` and `bee-why`, run as
  two parallel gathers through `bee dispatch prepare --kind gather`.
- Delegating the investigation and the fix to subagents became
  `bee dispatch prepare`; the fix goes out as a cell
  (`bee dispatch prepare --kind cell`) on the code role's configured model,
  replacing the named default bug-fix model.
- The `sequence-verifiable-units` principle skill became the red-before-green
  home pointer (`bee-swarming/references/worker-details.md:33-35`) in step 2,
  plus the commit-order rule in step 6. Restored (ppb-37): step 6 names
  `bee-principle-sequence-verifiable-units`.
- The pointer to the same discipline for Perf became `playbooks/perf.md`.

Dropped steps:

<!-- bee:not-a-deferral: a record or description of work already done, never a promise to act -->
- The Cursor `/loop` command for a long or stubborn hunt: bee has no
  counterpart for a looping command inside a session.
- The `architect` and `interrogate` fan-out before the fix: neither skill
  exists in bee yet (D5 brings them in a later slice). Step 4 keeps the
  contract rule: write down what the fix keeps when it crosses a function
  boundary. Restored (ppb-24): step 4 runs `bee-architect` first and
  `bee-interrogate` over a contested plan, and step 3 confirms the mechanism
  before that fan-out.
- The `tdd` skill reference in the commit-order step: no bee counterpart yet
  (D5, a later slice). The cadence stays as the cheap-test-path rule.
  Restored (ppb-24): step 6 points at `bee-tdd`.
- Step 6, run Opening a PR: bee opens a draft PR on the first cap and the
  `uat` acceptance at `bee close` stays the owner's, so no playbook step opens
  the PR.
- The closing Reply instruction to paste the failing-then-passing output
  verbatim: it is the Proof line, which the cap enforces.
<!-- /bee:not-a-deferral -->

Kept from the bee `bugfix` playbook (D3): the red-before-green home pointer,
the crash-site principle, the same-surface re-run with the inconclusive-run
rule, the event-order `statechart` split, the formal verify rerun, the
revert of every refuted edit, the short path for a known one-line cause, the
always-on signal sentence, the Proof line and the Proof floor line.
