---
artifact_contract: bee-research/v1
topic: openjev-verdict-2-xia
depth: standard
date: 2026-09-20
---

## Bottom Line

- Recommendation (ladder rung): **adapt-upstream** — four craft practices, none of
  them a feature.
- Why this is the lightest credible path: the source shares no stack, no domain and
  no runtime with bee. Nothing is portable as code. What travels is proof discipline,
  and bee already owns the frame those practices bolt onto (§ Prove, then say so, the
  cap proof line, `bee dev regen`). Each adoption is one rule line or one CLI flag,
  not a subsystem.
- Why the next-best rung lost: **reuse** and **built-in** lose because bee has no
  equivalent of the four named gaps below — verified by search, not assumed.
  **build** loses because the source already proved each practice in daily use and
  the shapes are small enough to translate directly.
- Confidence: 85%. The craft reading is `Local`+`Upstream` throughout; only the
  priority order is `Inference`.
- Suggested next step: **discussion** — `xia` builds nothing. If any of the four is
  wanted, the anti-cherry-pick rule and the smoke-first rule are one-line AGENTS.md
  edits and route through bee-shaping as one `tiny` docs-lane item.

## Source Manifest

| Field | Value |
|---|---|
| Repo or path | `/home/thanhsmind/Projects/refs/openJev-verdict-2.0` (origin `https://github.com/Heman10x-NGU/openJev-verdict-2.0.git`) |
| Ref | `main` at checkout time |
| Resolved commit SHA | `a458733c5f43fc7f30b6e4381636cbfbf8437633` |
| Narrowed scope | Proof, evidence and reporting discipline only — `RUNBOOK.md`, `verdict2/evaluate.py`, `reports/`, `scripts/analysis/`, `tests/test_docs_fresh.py`. Model code, training loop and WebGPU demo are out of scope: no bee analog exists. |

## Repo Snapshot

**This repo (bee)** `Local`
- Rust CLI (`packages/bee-rs`), markdown skills rendered into `.claude/skills/` and
  `.agents/skills/`, workflow state in `.bee/*.json(l)` behind the CLI.
- Proof frame already present: cap proof lines, `bee close` / `bee worktree merge`
  checking recorded proof, CI running the declared command on every push,
  `bee dev regen` as the three-step render chain.

**Source repo** `Upstream`
- Python 3.10+, PyTorch / transformers, ModernBERT backbone (`pyproject.toml`).
- No CI config at all — no `.github/`, no `.gitlab-ci.yml`, no Makefile (verified by
  directory listing). Every check is a test a human remembers to run.

**Stack overlap: none.** Per the port protocol's Lane Mapping, a stack mismatch this
large downgrades any `port` to `xia`. That downgrade is applied here: report and
discuss, build nothing.

## Question & Assumptions

- What was asked: `xia refs/openJev-verdict-2.0` — distill the source and say what
  bee should adopt.
- What success appears to mean: a short list of practices bee does not have, each
  with its source anchor and its landing place in bee — not a feature inventory.
- Assumption still needing confirmation: that the user wants craft, not the decision
  model itself. The domain gap makes any other reading unbuildable.

## Findings

### Local — what bee already has

| Source practice | bee status | Evidence |
|---|---|---|
| Machine-readable receipts per run | `EXISTS` | herding receipts, `docs/knowledge/work/*/delivery.md` |
| Docs regenerated from one source | `EXISTS` | `bee dev regen` — render-skill-trees → onboard → release-manifest |
| Proof recorded before a step is called done | `EXISTS`, stronger in bee | AGENTS.md § Prove, then say so; the cap proof line |
| Environment facts checked before the ritual | `EXISTS` | AGENTS.md § Judgment and deviation, environment-fact check |
| Failed suspicions written down, not dropped | `PARTIAL` | `skills/bee-principle-verify-before-reporting/SKILL.md` — covers a suspected defect, not a losing option in a sweep |
| Checklist printed, then exit non-zero | `NEW` | no hit for `--confirm` / anti-leak in `packages/bee-rs/src` or `skills/` |
| A committed baseline floor every number is read against | `NEW` | no hit for baseline/reference floor in `skills/` or `docs/` |
| "Report the whole block, never a subset" | `NEW` | no hit for subset/cherry-pick reporting in AGENTS.md, `skills/`, `docs/knowledge/` |
| Cheap smoke run before an expensive run | `NEW` | no hit for smoke test in AGENTS.md or `skills/` |
| Verify-only `--check` mode on the regen chain | `NEW` | `bee dev regen --help` lists no `--check`; the verb writes, it never merely reports drift |

### Upstream — the four worth adopting

**1. The confirm-checklist gate.** `verdict2/evaluate.py:99-108`. Running the final
evaluation without `--confirm` prints a four-item honesty checklist and exits 1. The
checklist is not advice — it is the only path to the command. `Upstream`

> "Running it without `--confirm` prints the anti-leak checklist and exits. Read the
> checklist honestly before passing the flag. Every read you make is a read you have
> to report." — `RUNBOOK.md` § 6

bee analog: `scripts/release.sh --no-test` "says so loudly", but loud text scrolls
past. Checklist-then-exit-1 makes the operator retype the command, which is the point.
Landing place: `--no-test`, and any future irreversible verb.

**2. A committed floors file.** `reports/reference_floors.json` holds five baselines —
uniform random, majority label, TF-IDF, the previous version, and a gold oracle. Every
headline number in the README is stated beside them. `Upstream`

> "The floors matter because a TF-IDF baseline on this benchmark already reaches about
> 0.65 accuracy … Any headline number has to be read against that." — `RUNBOOK.md` § 7

bee analog: a cap proof line reads "36 suites 0 failed" with no floor, so a reader
cannot tell a real pass from a suite that passed before the change too. Landing place:
the cap proof line carries what the same command did on the base commit.

**3. Anti-cherry-pick reporting.** The runbook names the exact metrics that must be
published together and names the gaming they prevent. `Upstream`

> "Publish these together, never a subset … Quoting only the second reads as metric
> gaming." — `RUNBOOK.md` § 7

This is the cheapest and highest-value adoption: one line under AGENTS.md § Prove,
then say so. bee's current rule governs *freshness* of evidence, never *completeness*
of it — a scoped-green cap that quotes the green suite and omits the skipped one
passes the rule as written.

**4. Smoke first, always.** A two-minute run with `--limit 60` before any long run,
with an explicit stop. `Upstream`

> "Two minutes, and it catches every environment problem before you commit to a long
> run … If this fails, stop and fix the environment. Do not proceed." — `RUNBOOK.md` § 2

Reinforced inside the code: `--limit` is documented as "Smoke tests only; a real read
uses all 400", and a limited run prints `WARNING: --limit N is a smoke test, not a
reportable result` (`verdict2/evaluate.py:98,127`). The cheap mode cannot be mistaken
for the real one. Landing place: before dispatching a wave, the cheapest proof runs
first; and any bee flag that narrows a check says so in its own output.

### Upstream — three more worth noting, lower priority

- **Numbered, independently re-runnable controls.** `scripts/exp_*.py` → one committed
  receipt each in `reports/v2/exp_e1..e9.json` (shuffled control, option order, label
  bias, hard negatives, abstention, contamination, latency, fp16 parity, external
  cases). Each control is a named file, not a paragraph in a report. `Upstream`
- **A failure gallery.** `reports/v2/failure_gallery.md` lists known wrong answers with
  a per-case column for whether the confidence guard caught it. Publishing what the
  system still gets wrong, beside whether the guard held. bee's captures record what
  settled; nothing records what is still broken and guarded. `Upstream`
- **A diagnostic that ships its own reading table.** `scripts/analysis/phase0_diagnose.py:1-19`
  runs four perturbation controls and its docstring maps each outcome to one diagnosis
  ("id_as_description higher than baseline → id and description are swapped"). The
  script cannot be run and misread. `Upstream`

### Inference

- **A candidate bee principle.** From the same docstring: *"Being reliably worse than
  chance takes information, which is the signature of a wiring defect rather than a
  weak model."* Generalized: a result systematically worse than random is evidence of
  inverted wiring, never of weak capability — measure the random floor before you
  conclude the thing is bad at its job. This sits beside
  `bee-principle-crash-site-versus-fault-site` and is not covered by it. Flagged for
  bee-capturing; not written, because `xia` builds nothing. `Inference`
- Priority order of the four adoptions is judgment, not measurement. `Inference`

### Where the source is weaker than bee — adopt nothing here

- **No automation.** `tests/test_docs_fresh.py` will catch README drift against the
  receipts, and nothing runs it. bee runs its declared command on every push. The
  source proves practices bee should copy while lacking the net bee already has.
- **AGENTS.md is a project description, not an operating contract.** It states what
  the repo is and its four invariants; it never tells an agent how to work, when to
  stop, or what needs a human. Nothing to take.
- **Numbers live in three homes.** The same accuracy and calibration figures sit in
  `README.md`, `AGENTS.md` and the receipts. `render_receipts.py --check` covers the
  README only — `AGENTS.md` drifts unguarded. A single-source-of-truth violation the
  source's own tooling half-fixes.

## Cross-Cutting Sweep

Where each adoption would have to wire, hunted outside the obvious file:

| Adoption | Surfaces touched | Unchecked |
|---|---|---|
| Confirm-checklist gate | `scripts/release.sh`; any bee-rs verb taking an irreversible flag; the hook layer if a checklist must be refused non-interactively | whether a non-interactive herding run can pass a checklist gate at all — **open question** |
| Floors on the proof line | AGENTS.md § Prove, then say so; the cap proof-line shape; `bee close` and `bee worktree merge` proof checks; `bee-swarming` worker contract | whether the recorded proof-line format is parsed anywhere that a second field would break |
| Anti-cherry-pick rule | AGENTS.md § Prove, then say so only | none — pure doc line |
| Smoke first | AGENTS.md § Work in parallel; `bee-swarming` "Execute"; `bee-hive` lane routing | none found |

Components absent from this sweep are unchecked, not confirmed clean.

## Risks, Unknowns, Follow-Ups

- The floors adoption changes a recorded artifact shape (the proof line). That is a
  covered-contract change, not a docs edit — it routes as a feature, not as `tiny`.
- The confirm-checklist gate may collide with bee's unattended control loop: a gate
  that demands a human retype the command cannot fire under `gate_bypass`. Resolve
  before shaping, not during.
- Open question for the user: adopt all four, or the two one-line rules (anti-cherry-
  pick, smoke-first) now and leave the two shaped ones for later?

## Source Pack

- Local files read: `AGENTS.md`, `skills/bee-researching/references/port-protocol.md`,
  `skills/bee-researching/references/research-brief-template.md`,
  `skills/bee-principle-verify-before-reporting/SKILL.md`, `docs/knowledge/work/*/delivery.md`,
  `bee dev regen --help`, `bee dispatch prepare` output.
- Upstream files read: `README.md`, `AGENTS.md`, `RUNBOOK.md`, `pyproject.toml`,
  `verdict2/evaluate.py`, `reports/reference_floors.json`, `reports/v2/receipt_t01.json`,
  `reports/v2/failure_gallery.md`, `scripts/analysis/phase0_diagnose.py`,
  `tests/test_docs_fresh.py`, directory listings of `scripts/`, `reports/`, `tests/`.
- Docs pages checked: none. No web research was needed or done — the whole question is
  answerable from two checkouts, and the source's own docs are version-matched by
  definition.
