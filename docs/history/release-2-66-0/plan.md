---
artifact_contract: bee-plan/v2
mode: tiny
---

# Plan: Release 2.66.0

## Summary

Publish 2.66.0 from `main`. One command does the whole release; this plan exists
so the deployment stage has an approved role plan to authorize against.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: no source changes, one
sanctioned script, and a permit that is validated before a single byte moves.

## Requirements (from CONTEXT.md)

- D1 (`997faaba`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch, and is done only when that script prints its
  final `OK`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | A deployment stage needs an approved v2 role plan, as the 2.65.0 lane had | read | docs/history/release-2-65-0/plan.md:69 | `## Role assignments` |
| 2 | The script spends the permit through one guarded call | read | scripts/release.sh:122 | `"$BEE_BIN" dispatch authorize --id "$BEE_DISPATCH_ID" --release-version "$AUTH_VERSION" \` |
| 3 | The version moves from the released 2.65.0 | ran | `rg -n '"version"' .claude-plugin/plugin.json` | `"version": "2.65.0",` |
| 4 | The work this release ships is merged | ran | `git log --oneline v2.65.0..HEAD` | `d899c3c93 Merge worktree beehive--wt--ui-ux-pro-max-a11y-port (branch wt/ui-ux-pro-max-a11y-port) via bee worktree merge` |

## Cells (current slice)

```json
[
  {
    "id": "rel660-1",
    "feature": "release-2-66-0",
    "title": "Publish release 2.66.0 through the sanctioned script",
    "lane": "tiny",
    "role": "deploy",
    "deps": [],
    "decisions": [],
    "files": [".claude-plugin/plugin.json", ".codex-plugin/plugin.json", "docs/history/codex-harness-hardening/release-manifest.json"],
    "read_first": ["docs/history/release-2-66-0/CONTEXT.md", "scripts/release.sh"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run `scripts/release.sh 2.66.0` with BEE_DISPATCH_ID set to an authorized deployment dispatch. The script bumps both plugin manifests, runs the regen chain (render-skill-trees, onboard --apply, release-manifest --write), runs the declared test suite before tagging, makes the release commit, tags v2.66.0, pushes main and the tag, waits for the release-binaries workflow and verifies the published assets. Never walk those steps by hand. Re-running at a version already committed is idempotent — it resumes the tail. The regen obligation on .claude-plugin/plugin.json is discharged INSIDE the script's own regen chain, which is why the release manifest rides this cell's files.",
    "verify": "bash scripts/release.sh 2.66.0 && bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "both plugin manifests report 2.66.0",
        "tag v2.66.0 exists locally and on the remote",
        "the GitHub release for v2.66.0 carries the binaries and SHA256SUMS",
        "the script printed its final OK line"
      ],
      "artifacts": [
        {"path": ".claude-plugin/plugin.json", "substantive": "version 2.66.0"},
        {"path": ".codex-plugin/plugin.json", "substantive": "version 2.66.0"}
      ],
      "key_links": ["the pushed tag is what triggers release-binaries.yml"],
      "prohibitions": ["Do not hand-walk bump, tag or push", "Do not skip the test gate"]
    },
    "behavior_change": false
  }
]
```

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"deployment","classification":"required","role":"deploy","reason":"This lane exists to publish 2.66.0; the deploy role owns the release permit."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The plan is this one page and needs no planning dispatch."},
    {"stage":"implementation","classification":"not-applicable","role":"code","reason":"No source changes: the release publishes what main already holds."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"The script's own test gate runs the declared suite before tagging."},
    {"stage":"documentation-and-capture","classification":"not-applicable","role":"docs","reason":"No documentation changes ride this release."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"Nothing to survey; the preconditions are the script's own."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow fact lookup is needed."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job here has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked and was not requested."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No high-risk consult is owed for a publish."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"No competing designs; there is one sanctioned command."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"No competing designs."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"No competing designs."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI changes ride this release."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Tiny lane: no hat wave."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Tiny lane: no hat wave."}
  ]
}
```

## Proof

- The script's final `OK` line, with the tag pushed, `release-binaries` green and
  the published release carrying the binaries and `SHA256SUMS`.
