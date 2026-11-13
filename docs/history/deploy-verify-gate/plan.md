---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Every project drives its product before a deploy goes live

## Summary

The deploy script and record that bee writes for a project gain one step:
before anything goes live, the script runs the project's full product
check and stops if it fails. A project with no product check stops too,
unless the person running the deploy says to skip it for that run. When
bee is installed or upgraded in a project whose deploy files lack the
step, bee says so. The product-check skill promises the exit code the
deploy script reads.

Mode: `standard` — 2 risk flags: public contracts (templates and a skill contract every host uses), multi-domain (onboarding code, two skills).
Why this is the least workflow that protects the work: the change sets a rule for every project's deploy, so it needs a plan check, but each part is small.

## Requirements (from CONTEXT.md)

- D1: a SAFE `drive --full` step before the first LIVE step in the deploy record and script.
- D2: no verify skill stops the deploy; `--skip-product-drive` (D7) is the user's per-run escape.
- D3: replay adds a missing step to old records and scripts.
- D4: `drive --full` and its exit-code contract in bee-verifying.
- D5, D6, C1: the onboarding notice and its tests.
- D7: the plan-check rulings — notice when any existing deploy file lacks the path; `--skip-product-drive` never overrides red; exit codes 0, 1, 2; upkeep adds missing entry points; project-owned release scripts are asked about, not edited.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The deploy template has a test step and then a deploy step, so the drive fits between | read | skills/bee-deploying/references/deploy.sh.template:41 | `echo "== test"` |
| 2 | The deploy step follows | read | skills/bee-deploying/references/deploy.sh.template:44 | `echo "== deploy $ENV_NAME $VERSION"` |
| 3 | The template parses its flags in one place | read | skills/bee-deploying/references/deploy.sh.template:12 | `YES=0; ALLOW_DIRTY=0` |
| 4 | The record template has a Steps table and a separate post-deploy Verify section | read | skills/bee-deploying/references/DEPLOY.template.md:40 | `## Verify` |
| 5 | Replay is a numbered branch the check can join | read | skills/bee-deploying/SKILL.md:27 | `## Branch A: Replay` |
| 6 | The verify contract names the full drive without a literal entry point | read | skills/bee-verifying/SKILL.md:151 | `- **Full drive** — every feature in the map, top to bottom.` |
| 7 | Onboarding's verification notices are pushed in one arm the new notice can follow | read | packages/bee-rs/crates/bee/src/onboard/notices.rs:409 | `notices.push(VERIFICATION_UPKEEP_POINTER.to_string());` |
| 8 | The bee repo has neither deploy file, so the notice needs no special case | ran | ls docs/DEPLOY.md scripts/deploy.sh | `ls: cannot access 'docs/DEPLOY.md': No such file or directory` |
| 9 | An existing control script has no `drive --full` entry point | ran | bash .bee/verify/verify-app/control-bee drive --full | `control-bee drive --full exit=1` |
| 10 | bee-verify-upkeep's pass is a numbered list a step can join | read | skills/bee-verify-upkeep/SKILL.md:56 | `## Pass` |

## Discovery

The leader read bee-deploying's skill and both templates in full, section 5
of bee-verifying, and the verification arm of `stale_advisor_notices`. The
release smoke of `release-smoke` is the beehive-only counterpart; this plan
does not touch it.

## Approach

Three cells: the onboarding notice (code, D5), the skill and template edits
(docs, D1-D4), then consolidation. Rejected: a new `bee verify` verb (a new
command surface when the templates already own the call); a hook refusal on
host deploy scripts (too strong for an owner-written file); warning instead
of stopping when no verify skill exists (the rule would be optional).

Risk map:
- An existing onboarding notice changes — MEDIUM — the one fact: the new notice is pushed only when the predicate is true — Lands in: dvg-1 — proof: the existing verification notice tests pass unchanged, plus a no-deploy-file silence test.
- A host deploy blocked by a control script without `drive --full` — MEDIUM — the one fact: replay checks the entry point before running and points to upkeep — Lands in: dvg-2 — proof: the Branch A text read at the leader check; named in delivery limits.

Waves: serial, dvg-1 → dvg-2 → dvg-3: one worktree, one shared build; dvg-2's proof builds the crate for parity tests.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {
      "stage": "implementation",
      "classification": "required",
      "role": "code",
      "reason": "dvg-1 writes the onboarding notice and its tests."
    },
    {
      "stage": "docs",
      "classification": "required",
      "role": "docs",
      "reason": "dvg-2 edits two skills and two templates and runs the regen chain."
    },
    {
      "stage": "writing",
      "classification": "required",
      "role": "writer",
      "reason": "dvg-3 consolidates knowledge."
    },
    {
      "stage": "slice-judge",
      "classification": "required",
      "role": "review",
      "reason": "Standard lane: one read-only checklist judge covers the behavior-change cell dvg-1. Independent review stays user-invoked."
    },
    {
      "stage": "hat-facts-gaps",
      "classification": "required",
      "role": "hat-facts-gaps",
      "reason": "Six product files: the three default seats run the plan check."
    },
    {
      "stage": "hat-alternatives",
      "classification": "required",
      "role": "hat-alternatives",
      "reason": "Default seat; cites the SMALLER PATH check below."
    },
    {
      "stage": "hat-user-impact",
      "classification": "required",
      "role": "hat-user-impact",
      "reason": "Default seat; host owners and agents meet the notice and the deploy stop."
    },
    {
      "stage": "hat-risks",
      "classification": "not-applicable",
      "role": "hat-risks",
      "reason": "Not high-risk; no hard-gate flag."
    },
    {
      "stage": "hat-value",
      "classification": "not-applicable",
      "role": "hat-value",
      "reason": "Not high-risk."
    },
    {
      "stage": "deployment",
      "classification": "not-applicable",
      "role": "deploy",
      "reason": "A release runs through scripts/release.sh only when the owner asks."
    },
    {
      "stage": "planning",
      "classification": "not-applicable",
      "role": "plan",
      "reason": "The leader wrote this plan."
    },
    {
      "stage": "test-and-live-proof",
      "classification": "not-applicable",
      "role": "test",
      "reason": "Each cell writes its own red-first tests."
    },
    {
      "stage": "read-only-gather",
      "classification": "not-applicable",
      "role": "read",
      "reason": "The leader read the files during planning."
    },
    {
      "stage": "fact-extraction",
      "classification": "not-applicable",
      "role": "extraction",
      "reason": "No narrow lookup is left."
    },
    {
      "stage": "generation-fallback",
      "classification": "not-applicable",
      "role": "generation",
      "reason": "Every job has a specific role."
    },
    {
      "stage": "generic-advisor",
      "classification": "not-applicable",
      "role": "advisor",
      "reason": "The hat wave is the consult."
    },
    {
      "stage": "supervision",
      "classification": "not-applicable",
      "role": "supervisor",
      "reason": "No unattended loop."
    },
    {
      "stage": "blind-lane-1",
      "classification": "not-applicable",
      "role": "lane-1",
      "reason": "One shape."
    },
    {
      "stage": "blind-lane-2",
      "classification": "not-applicable",
      "role": "lane-2",
      "reason": "One shape."
    },
    {
      "stage": "blind-lane-3",
      "classification": "not-applicable",
      "role": "lane-3",
      "reason": "One shape."
    },
    {
      "stage": "design-ui",
      "classification": "not-applicable",
      "role": "design",
      "reason": "No rendered UI."
    }
  ]
}
```

## Shape

Epic map, one slice. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied); step 6 does not apply.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Reach every project | the onboarding notice | D5 | 1 | `deploy_verify` tests |
| The deploy step | templates and skills | D1-D4 | 1 | parity, pointer and spelling tests; template syntax check |

SMALLER PATH: templates only, with no notice, is cheaper, but projects with deploy files written before today would never learn the rule, which is exactly the owner's "always". A new verb is costlier. PASS.

PLAN CHECK (three seats, one round). STRUCTURE — BLOCKER, closed: dvg-3 pointed at a section of host-project-artifacts.md that does not exist (it now adds one). WARNINGS applied (D7): control scripts without `drive --full` had no owner — bee-verify-upkeep now adds the entry points, the template gives exit 2 its own message (claim 9); the notice matches D3's both; the retired-key exception is dropped; the flag is renamed and never overrides red; project-owned release scripts are asked about, not edited; the replay check is a read, not a run; dvg-1 writes no docs; dvg-2 carries an executable proof of its stop branches and is judged as a behaviour change. Dismissed: removing the dvg-1 → dvg-2 edge (it is the shared-build serial edge, named under Waves); adding `drive` to beehive's own control-bee (it has no deploy record; named in limits). CELLS (reviewed: 3) — CRITICAL: none after the fixes. MINOR: a name-filtered proof passes on zero matches; dvg-1 must quote its matched count.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dvg-1 | Tell a project at onboarding when its deploy drives no product first | onboard/notices.rs, onboard/templates.rs | — | `bee onboard` in a project with a deploy script and no product drive prints one notice naming bee-deploying | `deploy_verify` tests |
| dvg-2 | Make every bee-written deploy drive the product before going live | bee-deploying SKILL.md and both templates, bee-verifying and bee-verify-upkeep SKILL.md, rendered copies | dvg-1 | a new deploy script runs the product drive before the first live step and stops on red, with its own message for a missing entry point | parity and spelling tests, manifest check, four scratch runs of the template |
| dvg-3 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, delivery.md | dvg-1, dvg-2 | onboarding and verify-pipeline concepts describe the step | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "dvg-1",
    "feature": "deploy-verify-gate",
    "title": "Tell a project at onboarding when its deploy drives no product first",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "69dcdaba-8eb3-495d-a855-57d117c97b07",
      "5cdeb901-4dda-4615-bd48-d2fd45641265",
      "1c535c1d-f5b8-4d6d-96b6-4987dff0fdc8",
      "a946357a-cf1a-4d8f-b490-a2bfbec57f01"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs"
    ],
    "read_first": [
      "docs/history/deploy-verify-gate/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs"
    ],
    "affects_skills": [],
    "affects_specs": [
      "docs/knowledge/areas/onboarding/host-project-artifacts.md"
    ],
    "action": "Implement docs/history/deploy-verify-gate/CONTEXT.md D5 as refined by D6, and contract C1. Anchors in onboard/notices.rs: `pub fn stale_advisor_notices(repo_root: &Path) -> Vec<String> {` and, inside it, the verification arm that pushes `VERIFICATION_UPKEEP_POINTER.to_string()` or one of the two verification offers; in onboard/templates.rs: `pub const VERIFICATION_UPKEEP_POINTER` (the voice to follow). Add a predicate over the repo root that reads only docs/DEPLOY.md and scripts/deploy.sh (a missing or unreadable file counts as absent, never a panic) and is true when at least one of them exists and any one that exists lacks the text `.bee/verify/verify-app/` (D7). When it is true, push one notice, a new constant in templates.rs, right after the whole verification arm and before the design offer, whichever branch of that arm ran, the retired-verify-key branch included (D7: it is a separate fact). The notice says, in plain words: this project's deploy goes live without driving the product first; before the first live step, its deploy record and script should run the full product drive of its verify skill and stop on a failure; use the bee-deploying skill to add that step; when the project has no verify skill yet, the bee-verifying skill builds one. It writes nothing and refuses nothing. Every other notice stays byte-identical. Red first, with test names that contain deploy_verify: docs/DEPLOY.md without the path fires once; scripts/deploy.sh that names `.bee/verify/verify-app/control-x` while docs/DEPLOY.md is absent is silent; docs/DEPLOY.md naming the path while scripts/deploy.sh lacks it fires; no deploy file is silent; a project with only scripts/release.sh is silent; the retired verify key present with a deploy file lacking the path fires; and the existing verification notice tests still pass unchanged. The knowledge delta is left to dvg-3; this cell writes no docs. No code comments. The proof run must show the new tests ran: quote the count of tests the filter matched, which must be above zero.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- deploy_verify && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- every_code_file_is_at_or_below_its_comment_baseline",
    "must_haves": {
      "truths": [
        "A project with docs/DEPLOY.md or scripts/deploy.sh where any existing one lacks .bee/verify/verify-app/ gets one notice that its deploy drives no product first",
        "A project whose every existing deploy file names .bee/verify/verify-app/ gets no such notice",
        "A project with no deploy file, or only scripts/release.sh, gets no such notice",
        "Every other onboarding notice is unchanged"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/onboard/notices.rs",
          "substantive": "the deploy-verify predicate, the notice push and deploy_verify tests"
        }
      ],
      "key_links": [
        "stale_advisor_notices pushes the deploy-verify notice beside the verification arm"
      ],
      "prohibitions": [
        "No write to any host file",
        "No refusal and no change to the hook layer",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "dvg-2",
    "feature": "deploy-verify-gate",
    "title": "Make every bee-written deploy drive the product before going live",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [
      "dvg-1"
    ],
    "decisions": [
      "f04c9c97-4bbf-4d7a-beea-7708bafb1611",
      "809acb09-fe5f-47a0-be2f-45f378e8257a",
      "c1e421b5-7e67-4042-8b5b-18139c83656d",
      "c81ae967-9017-4c83-8728-4734871a5910",
      "a946357a-cf1a-4d8f-b490-a2bfbec57f01"
    ],
    "files": [
      "skills/bee-deploying/SKILL.md",
      "skills/bee-deploying/references/deploy.sh.template",
      "skills/bee-deploying/references/DEPLOY.template.md",
      "skills/bee-verifying/SKILL.md",
      "skills/bee-verify-upkeep/SKILL.md",
      ".claude/skills/*",
      ".agents/skills/*",
      ".opencode/skills/*",
      ".claude-plugin/skills/*",
      ".codex-plugin/skills/*",
      ".bee/onboarding.json",
      "skills-lock.json",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/deploy-verify-gate/CONTEXT.md",
      "skills/bee-deploying/SKILL.md",
      "skills/bee-deploying/references/deploy.sh.template",
      "skills/bee-deploying/references/DEPLOY.template.md",
      "skills/bee-verifying/SKILL.md",
      "skills/bee-verify-upkeep/SKILL.md"
    ],
    "affects_skills": [
      "skills/bee-deploying/SKILL.md",
      "skills/bee-verifying/SKILL.md",
      "skills/bee-verify-upkeep/SKILL.md"
    ],
    "affects_specs": [],
    "action": "Implement docs/history/deploy-verify-gate/CONTEXT.md D1, D2, D3 and D4, as refined by D7, in three skills and two templates. Load the bee-technical-writing skill first. (1) references/deploy.sh.template: add a `--skip-product-drive` flag beside `--yes` and `--allow-dirty` (anchor: `YES=0; ALLOW_DIRTY=0`) and to the usage line (D7 b). Add a step between the test step (anchor: `echo \"== test\"`) and the deploy step (anchor: `echo \"== deploy $ENV_NAME $VERSION\"`): it finds the files matching `.bee/verify/verify-app/control-*`; exactly one: run it with bash and the arguments `drive --full`; exit 0 goes on; exit 2 stops with a message that the control script has no `drive --full` entry point and that the bee-verify-upkeep skill adds it; any other non-zero exit stops with a message that the product drive went red and nothing went live (the flag never overrides a red drive); none: stop with a message that the project has no verify skill and that the bee-verifying skill builds one, unless --skip-product-drive was passed, in which case print that the product was not driven and go on; more than one: stop and name them. Keep `set -euo pipefail` semantics, write no comments in the template, and prove it in a scratch copy under .bee/tmp with its angle-bracket placeholders replaced by harmless commands: bash -n, then four runs against fake control scripts that exit 0, 1 and 2 and against no control script (with and without the flag), each showing the expected go-on or stop. Record those runs in the cap. (2) references/DEPLOY.template.md: in `## Steps`, add a SAFE row before the first LIVE row that runs `bash .bee/verify/verify-app/control-<app> drive --full` and must show every feature ok; add one sentence under the table that this row is the product drive and the `## Verify` section is the check of the live target after the deploy. (3) SKILL.md of bee-deploying: in Branch A, add a step before the Preconditions step: check that the record and the script carry the product drive step before the first LIVE step; when either lacks it, add it to both in the same turn from the templates and report the edit; check, by reading the control script (never by a full run), that it answers `drive --full`, and when it does not, point to bee-verify-upkeep before running anything LIVE; when the project has no verify skill, say so and point to bee-verifying. When the deploy is owned by a project release script or deploy skill (Step 0), do not edit it: tell the user the drive step is missing and ask (D7 e). In B4, say the record and script carry the drive step. In Hard rules, add that `--skip-product-drive` is the user's choice for one run, never the agent's, the same as `--yes`, and that it never overrides a red drive. Renumber steps where needed. (4) SKILL.md of bee-verifying, section 5: give the full drive its literal entry point `drive --full` beside the fast one, and state the exit-code contract: one ok or FAIL line per mapped feature; exit 0 when every one passes, 1 when any fails, 2 when the entry point is unknown; a deploy script reads that exit code before it goes live (D7 c). (4b) SKILL.md of bee-verify-upkeep: in its Pass, add one step that checks the control script answers `drive --fast` and `drive --full` with that exit-code contract, and adds a missing one, proven by running it (D7 d). (5) Run `.bee/bin/bee dev regen` so the rendered copies and the release manifest match the sources; never hand-edit a rendered copy. Cite CONTEXT.md D-numbers in prose, never raw decision ids. One commit.",
    "verify": ".bee/bin/bee dev release-manifest --check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test instruction_laws && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- no_shipped_command_spelling every_code_file_is_at_or_below_its_comment_baseline",
    "must_haves": {
      "truths": [
        "deploy.sh.template runs the one .bee/verify/verify-app/control-* file with drive --full before the deploy step and stops on a non-zero exit",
        "deploy.sh.template stops when no verify control script exists unless --skip-product-drive was passed, and a red drive always stops",
        "DEPLOY.template.md has a SAFE product drive row before the first LIVE row",
        "bee-deploying Branch A adds a missing drive step to the record and the script before running them",
        "bee-deploying Hard rules say --skip-product-drive is the user's choice for one run",
        "bee-verifying section 5 names drive --full and its 0, 1, 2 exit-code contract",
        "The rendered skill copies match their sources after the regen chain",
        "deploy.sh.template gives exit 2 of the drive its own message pointing to bee-verify-upkeep",
        "bee-verify-upkeep checks and adds the drive --fast and drive --full entry points"
      ],
      "artifacts": [
        {
          "path": "skills/bee-deploying/references/deploy.sh.template",
          "substantive": "the product drive step and the --skip-product-drive flag"
        },
        {
          "path": "skills/bee-deploying/SKILL.md",
          "substantive": "the replay check, the B4 line and the hard rule"
        }
      ],
      "key_links": [
        "the deploy template calls drive --full, the entry point bee-verifying section 5 fixes"
      ],
      "prohibitions": [
        "No hand edit of a rendered skill copy",
        "No change to the post-deploy Verify section's meaning",
        "No comments in the template"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": true
    }
  },
  {
    "id": "dvg-3",
    "feature": "deploy-verify-gate",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "dvg-1",
      "dvg-2"
    ],
    "decisions": [
      "f04c9c97-4bbf-4d7a-beea-7708bafb1611",
      "809acb09-fe5f-47a0-be2f-45f378e8257a",
      "c1e421b5-7e67-4042-8b5b-18139c83656d",
      "c81ae967-9017-4c83-8728-4734871a5910",
      "69dcdaba-8eb3-495d-a855-57d117c97b07",
      "1c535c1d-f5b8-4d6d-96b6-4987dff0fdc8",
      "a946357a-cf1a-4d8f-b490-a2bfbec57f01"
    ],
    "files": [
      "docs/knowledge/**",
      "docs/history/deploy-verify-gate/delivery.md"
    ],
    "read_first": [
      "docs/history/deploy-verify-gate/CONTEXT.md",
      "docs/history/deploy-verify-gate/plan.md",
      "docs/knowledge/areas/onboarding/host-project-artifacts.md",
      "docs/knowledge/areas/verify-pipeline/overview.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature deploy-verify-gate`. docs/knowledge/areas/onboarding/host-project-artifacts.md has no section on bee-deploying's record and script today: add one short section, beside the section on static checks, that states the product drive step, the --skip-product-drive rule, the 0, 1, 2 exit codes the template reads, and the onboarding notice (D1, D2, D3, D5, D6, D7), anchored to the templates and to onboard/notices.rs. In docs/knowledge/areas/verify-pipeline/overview.md, state that every host deploy drives the product through drive --full with its exit-code contract (D4), beside the release smoke. Dedupe and trim, anchor each concept to the files it describes, never restate code, and write docs/history/deploy-verify-gate/delivery.md listing the concepts changed and why, one line each, plus a Known limits section: a host deploy script written before this feature gains the step only when bee-deploying replays it; a host control script without drive --full stops the deploy with exit 2 until bee-verify-upkeep adds it; deploys triggered by CI, deploys with no script, and deploys recorded outside docs/DEPLOY.md and scripts/deploy.sh are not gated; beehive's own control-bee has no drive entry point and its release uses the smoke. Cite CONTEXT.md D-numbers, never raw decision ids. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "area and pattern concepts describe the feature's changes, anchored to code",
        "docs/history/deploy-verify-gate/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {
          "path": "docs/history/deploy-verify-gate/delivery.md",
          "substantive": "the feature's delivery record"
        }
      ],
      "key_links": [
        "the delivery lands in the same merge as the code"
      ],
      "prohibitions": [
        "No restating of code in area concepts",
        "No code comments"
      ],
      "open_choices": []
    },
    "trace": {
      "worker": null,
      "outcome": null,
      "files_changed": [],
      "deviations": [],
      "friction": null,
      "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Case | Probe | Pass when |
|---|---|---|
| Happy path, D5 | docs/DEPLOY.md without the verify path | one deploy-verify notice |
| Edge, D5 | scripts/deploy.sh naming `.bee/verify/verify-app/control-x` | no deploy-verify notice |
| Edge, D6 | only scripts/release.sh | no deploy-verify notice |
| Error path, D5 | unreadable deploy file | no panic, treated as absent |
| D1/D2 | template with placeholders replaced, `bash -n` | parses |
| D4 | read section 5 | `drive --full` and the exit-code rule are stated |

## Open Questions

(none)

## Out of scope

- Editing any host project's existing deploy files from bee's code.
- Adding `drive` entry points to beehive's own control-bee.
- Gating deploys triggered by CI or recorded outside the two deploy paths.
- A refusal in bee's hooks.
- beehive's own release, covered by release-smoke.
