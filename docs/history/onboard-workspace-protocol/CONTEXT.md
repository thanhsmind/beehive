# onboard-workspace-protocol — locked decisions

The owner wants `WORKSPACE_PROTOCOL.md` finished at every `bee onboard`, and
wants the agent told how to write a good one.

- **D1** (store `a4c2c38d`). `bee onboard`, dry-run and `--apply` alike,
  reports a `workspace_protocol` object (`state`, `unedited`, `issues`,
  `action_needed`). It adds one notice when the file is missing, invalid,
  unreadable, or still byte-equal to the fresh `bee protocol init` render.
  Onboard never writes the file. Rejected: onboard `--apply` writes the
  render, because a valid generic file then hides that nobody finished it.
- **D2** (store `0687dd2a`). The bee-hive onboarding procedure runs after
  every onboard with `action_needed: true`. It says where to look for each
  key, how to pick `PROJECT_CRITICALITY`, what a good line looks like, and a
  self-check before commit.
- **D3** (store `6474eb2d`, tag `contract:onboard-workspace-protocol-report`).
  The payload shape and the notice text that the onboard tests check.
