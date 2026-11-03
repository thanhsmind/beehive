---
name: bee-deploying
description: "Learn, record and replay how one project deploys or releases: find the project's deploy record and run it, stopping before every step that touches a live environment, or, with no record yet, read the CI, container, package and platform files, ask only the gaps, walk one deploy with the user and write docs/DEPLOY.md, scripts/deploy.sh and a pointer in CLAUDE.md or AGENTS.md. Use when the user says deploy, release, ship, or asks to record how this project deploys. Not for the bee repo's own release, which scripts/release.sh owns, and not for a server a project-specific deploy skill already owns."
---

# bee-deploying

One project, one way to deploy, written down once. The skill has two
branches. Pick the branch first, every time.

## Step 0: Find the record

Look, in this order, from the project root:

1. `docs/DEPLOY.md`
2. A `## Deploy` or `## Release` section in `CLAUDE.md` or `AGENTS.md`
   that names a script.
3. `scripts/deploy.sh` or `scripts/release.sh`

Found one: go to **Branch A (Replay)**. Found none: go to **Branch B (Learn)**.

A project-specific deploy skill or release script beats this skill.
Point at it and follow it; never write a second path beside it. The
record still owns *which* target, tag and environment; the other skill
or script owns *how* the command runs.

## Branch A: Replay

1. Read the record fully and note the date in its header. When a CI
   workflow, `Dockerfile`, `docker-compose*.yml`, `Makefile`,
   `package.json` or platform file (`fly.toml`, `vercel.json`,
   `Procfile`) changed after that date, say so in one line and check
   the drifted step before you run it.
2. Run the **Preconditions**. A red precondition stops the deploy:
   report it, never patch around it.
3. Run the **Steps** in order. Before each `LIVE` step, show the exact
   command and wait for the user's yes. One yes covers one command,
   never the next.
4. Run **Verify** and report its fresh output.
5. When reality disagreed with the record (a step failed, or the user
   typed something the record did not say), fix the record in the same
   turn.

## Branch B: Learn

Goal: one working deploy, then a record good enough that Branch A needs
no memory of this session.

### B1. Read before you ask

| File | Tells you |
|---|---|
| `.github/workflows/*`, `.gitlab-ci.yml`, `Jenkinsfile` | Deploy on push or tag? Which branch or tag pattern? |
| `Dockerfile`, `docker-compose*.yml` | Image build, registry, service names |
| `package.json` scripts, `Makefile`, `justfile`, `Taskfile.yml` | Named build, test and deploy targets |
| `fly.toml`, `vercel.json`, `netlify.toml`, `render.yaml`, `app.yaml`, `serverless.yml`, `wrangler.toml` | Hosted platform and its CLI |
| `.env.example`, `.env.*.example` | Required variables, names only |
| `README*`, `docs/*deploy*`, `docs/*release*` | Human steps already written |
| `scripts/*` | Half-written deploy scripts |
| `git tag --sort=-v:refname`, top 5 | Tag format and release cadence |

Never open `.env`, `*.pem`, `*.key`, `credentials*` or `secrets*`. Name
them as required inputs instead.

### B2. Ask only the gaps

One round of questions, only for what the files did not answer:

- Target: where does it go (host, platform, server alias)?
- Environments: staging and production, or production only?
- Trigger: tag, branch push, manual command, or a button in a UI?
- Secrets: which variables or key files, and where they live. The
  place, never the value.
- Manual steps: clicks, a CI wait, a migration, a cache clear.
- Rollback: how to undo. "Redeploy the previous tag" is a valid answer.
- Done check: URL, health endpoint, version string or log line.

### B3. Walk one deploy together

Show each command before it runs. Every step that touches a live
environment is `LIVE` and waits for the user's yes for that exact
command. Record each step as it succeeds, with its real output shape. A
failed step is fixed or marked manual, never skipped in silence.

When the user wants the record and no deploy now, build it from B1 and
B2 and mark it `UNTESTED` in the header. Branch A then asks harder until
a real run removes that mark.

### B4. Write the record

1. `docs/DEPLOY.md` from `references/DEPLOY.template.md`: header with
   date and `TESTED` or `UNTESTED`; Target, Preconditions, Steps (each
   `SAFE` or `LIVE`), Verify, Rollback, Gotchas.
2. `scripts/deploy.sh` from `references/deploy.sh.template`, for the
   steps that are safe to automate: `set -euo pipefail`; environment and
   version as arguments; refuse a dirty tree unless `--allow-dirty`;
   each `LIVE` command prints and asks `Continue? [y/N]` unless `--yes`.
   Nothing safe to automate: write no script and say why in the record.
3. A short section appended to `CLAUDE.md`, or `AGENTS.md` when that is
   the project's file:

   ```markdown
   ## Deploy

   When asked to deploy or release, follow `docs/DEPLOY.md`
   (script: `scripts/deploy.sh <env> <version>`). Every `LIVE` step
   waits for an explicit yes.
   ```

Report the three paths. Do not commit them unless the user asks.

## Hard rules

- A `LIVE` step never runs without a fresh yes from the user for that
  exact command. `--yes` on the script is the user's choice, never the
  agent's.
- Never read, print or copy a secret value. Names only.
- Never deploy a dirty working tree or an unpushed commit unless the
  user says so for that run.
- The record is the single source of truth. When reality and the
  record disagree, fix the record in the same turn.
- Report the whole result: what ran, what was skipped, what is
  `UNTESTED`.
- Close with a capture line: a settled deploy fact goes to
  `bee decisions log` when the project runs bee, otherwise to a Gotchas
  line in the record.
