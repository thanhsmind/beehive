# Delivery — deploy-verify-gate

Feature: every project drives its product before a deploy goes live. Rules are CONTEXT.md D1-D7 and C1.

## Concepts changed

- `docs/knowledge/areas/onboarding/host-project-artifacts.md`: new section on the product drive step, `--skip-product-drive`, the 0, 1, 2 exit codes and the onboarding notice (D1, D2, D3, D5, D6, D7), because the area had nothing on bee-deploying's record and script.
- `docs/knowledge/areas/verify-pipeline/overview.md`: every host deploy drives `drive --full` with its exit-code contract (D4, D7), beside the release smoke, because a deploy script needs an exit code it can trust.
- `skills/bee-deploying/references/deploy.sh.template` and `DEPLOY.template.md`: carry the drive step before the first live step (D1, D2, D7).
- `skills/bee-deploying/SKILL.md`: replay adds the step to older records and reads the control script instead of running it (D3, D7).
- `skills/bee-verifying/SKILL.md` and `skills/bee-verify-upkeep/SKILL.md`: fix `drive --full` and its exit codes as the contract, and have upkeep add `drive --fast` and `drive --full` (D4, D7).
- `onboard/notices.rs` and `onboard/templates.rs`: one propose-only notice when an existing deploy file lacks `.bee/verify/verify-app/` (D5, D6, C1).

## Known limits

- A host deploy script written before this feature gains the step only when bee-deploying replays it.
- A host control script without `drive --full` stops the deploy until bee-verify-upkeep adds it.
- Deploys triggered by CI, deploys with no script, and deploys recorded outside `docs/DEPLOY.md` and `scripts/deploy.sh` are not gated.
- beehive's own control-bee has no drive entry point and its release uses the smoke.
