# Every project drives its product before a deploy goes live — Context

**Feature slug:** deploy-verify-gate
**Date:** 2026-11-13
**Shaping session:** complete (owner: "running verify should become a mechanism, so deploying any other project will always do it")
**Scope:** Standard
**Domain types:** RUN

## Feature Boundary

bee-deploying writes each host project's deploy record (`docs/DEPLOY.md`)
and script (`scripts/deploy.sh`) from two templates. Neither runs the
project's own verify skill, so a deploy can go live without the product
ever being driven. After this feature, every record and script bee writes
drives the product's full verify before the first live step and stops on
red; replay adds the step to records written before; the verify contract
promises an exit code a script can read; and `bee onboard` tells a project
whose deploy files lack the step at its next install or upgrade. The
feature ends there: it edits no host project's files itself, adds no
refusal to bee's hooks, and does not change beehive's own release, which
`release-smoke` already covers.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | The deploy record and script that bee-deploying writes carry one `SAFE` step, before the first `LIVE` step, that runs the project's full product drive: `bash` on the one file matching `.bee/verify/verify-app/control-*` with the arguments `drive --full`. A non-zero exit stops the deploy before anything touches a live environment. This step is apart from the existing `## Verify` section, which checks the live target after the deploy. | The templates are where every host's deploy script comes from. |
| D2 | With no file matching `.bee/verify/verify-app/control-*`, the deploy script stops and names bee-verifying as the way to build one. A per-run `--skip-verify` flag lets the run go on and prints that the product was not driven. The flag is the user's choice for that run, never the agent's, the same rule as `--yes`. | A silent skip is the gap this feature closes. |
| D3 | bee-deploying Branch A checks that the record and script carry the drive step before it runs them. When either lacks it, the skill adds it to both in the same turn and checks that the control script answers `drive --full`. Branch B writes it from the templates. | Existing records predate the rule; replay is where the agent meets them. |
| D4 | The bee-verifying contract fixes the full drive's entry point as `drive --full` on the generated control script. It prints one `ok` or `FAIL` line per mapped feature and exits non-zero when any feature fails, zero only when every one passes. | A deploy script needs an exit code it can trust. |
| D5 | `bee onboard` reports one notice when a host project has `docs/DEPLOY.md` or `scripts/deploy.sh` and neither file contains the text `.bee/verify/verify-app/`. The notice says the deploy drives no product before it goes live, and to add the drive step through bee-deploying. Silent when neither file exists or either one names the path. Propose-only. | Onboard runs on every install and upgrade, so the rule reaches every project without an agent remembering it. |
| D6 | Refines D5: no special case for the bee repo. It has neither file; its release script owns its smoke. | A special case would be dead code. |
| C1 | Contract deploy-verify-notice: a predicate over the repo root reads `docs/DEPLOY.md` and `scripts/deploy.sh` only; the notice text is a constant in `onboard/templates.rs` and is emitted beside the verification offer. Proving tests carry `deploy_verify` in their names: a record without the path fires; a script with the path is silent; no deploy file is silent; a project with only `scripts/release.sh` is silent; every other notice is unchanged. | |

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/onboarding/host-project-artifacts.md | suspect | Owns what a host project receives from onboarding, including bee-deploying's record and the notices. |
| docs/knowledge/areas/verify-pipeline/overview.md | suspect | Owns the verify driver and the release smoke this feature mirrors for hosts. |

### Code

| Path | What it decides |
|------|-----------------|
| `skills/bee-deploying/SKILL.md` | Replay and Learn branches. |
| `skills/bee-deploying/references/deploy.sh.template` | The host deploy script's steps and flags. |
| `skills/bee-deploying/references/DEPLOY.template.md` | The host deploy record's sections. |
| `skills/bee-verifying/SKILL.md` | The control script contract (section 5). |
| `packages/bee-rs/crates/bee/src/onboard/notices.rs` | The verification notices onboarding prints. |
| `packages/bee-rs/crates/bee/src/onboard/templates.rs` | The notice texts. |

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable.
