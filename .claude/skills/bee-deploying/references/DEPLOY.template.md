# Deploy: <project name>

Recorded: <YYYY-MM-DD> · Status: TESTED | UNTESTED · Last verified by: <who>

## Target

- Platform or host: <fly.io | vercel | own server alias | k8s cluster>
- Environments: <staging, production>
- Trigger: <git tag vX.Y.Z | push to main | manual script | UI button>
- Deploy skill or release script, if any: <name | none>

## Preconditions

Run these first. Any red line stops the deploy.

| Command | Must show |
|---|---|
| `git status --porcelain` | nothing |
| `git fetch && git status -sb` | not behind origin |
| `<tool> --version` | <min version> |

Required inputs, names only, never values:

- `<ENV_VAR_NAME>`: lives in <where>
- `<key file>`: lives in <where>

## Steps

| # | Kind | Command or action | Expected result |
|---|------|-------------------|-----------------|
| 1 | SAFE | `<build command>` | build ok, artifact at <path> |
| 2 | SAFE | `<test command>` | green |
| 3 | LIVE | `<push or deploy command>` | <what the tool prints> |
| 4 | LIVE | <manual: wait for CI job X to go green> | link: <url> |
| 5 | LIVE | `<migration or cache clear>` | <output> |

`SAFE` runs without asking. `LIVE` touches a real environment and waits
for an explicit yes for that exact command.

## Verify

| Command | Must show |
|---|---|
| `curl -fsS <health url>` | 200, body contains <marker> |
| `<version check>` | <version> |

## Rollback

<how to undo; "redeploy the previous tag with step 3" is fine>

## Gotchas

- <thing that bit us once, and the fix>
