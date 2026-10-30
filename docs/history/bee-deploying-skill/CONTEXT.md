# bee-deploying-skill — locked context

Request (verbatim): "skill này phải đi kèm với bee, cho option cài tích hợp
với skillstation hoặc cài vào project skill"

What the user wants: the deploy-runbook skill written on 2026-10-30 must
ship as part of bee, and `bee onboard` must offer two ways to install it:
into the project's own skill folders (the default every bee skill already
takes), or into the user's skillstation hub.

## Locked decisions

- D1 (store 24175c6f): `skills/bee-deploying/` ships with bee and is
  rendered into every onboarded project like the other bee skills. The
  per-project copy is the default install.
- D2 (store 6855564f): `bee onboard --skillstation` also publishes the
  rendered `bee-deploying` skill into the user's skillstation hub through
  the skillstation CLI (`skill commit <repo>/.claude/skills/bee-deploying
  --yes --json`, then `skill activate bee-deploying --json` and the bound
  confirm command it returns). One plan item
  `{action: publish_skillstation, skill, path}`, present only with the flag.
- D3 (store 464c2cca): a missing `skillstation` binary or a non-zero exit
  is a loud skipped entry `{skill, target: "skillstation", reason}` in
  `skipped_skills`; the onboard still reports `applied`. The hub publish
  set is the constant `HUB_SKILLS = ["bee-deploying"]`.
- D4 (store 7d06a37d): the skill has two branches, Replay and Learn,
  chosen by whether a deploy record exists; `LIVE` steps wait for the
  user's yes per command; secret values are never read; an existing deploy
  skill or release script is pointed at, never duplicated.

## Out of scope

- Publishing any other bee skill to the hub.
- Changing `scripts/install.sh` or `install.ps1` (they call `bee onboard`;
  a `--skillstation` pass-through is a follow-up backlog item).
- Retiring the hand-authored `deploy-runbook` skill already in the user's
  hub; the user deprecates it after `bee-deploying` lands there.
