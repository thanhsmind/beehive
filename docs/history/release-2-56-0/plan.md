---
artifact_contract: bee-plan/v2
mode: tiny
---

# Plan: Release 2.56.0

## Summary

Publish 2.56.0 from `main`. One command does the whole release; this plan exists
so the deployment stage has an approved role plan to authorize against.

Mode: `tiny` — 0 risk flags: none
Why this is the least workflow that protects the work: no source changes, one
sanctioned script, and a permit that is validated before a single byte moves.

## Requirements (from CONTEXT.md)

- D1 (`11dcf251`): the release runs only through `scripts/release.sh` under an
  authorized deployment dispatch.
- D2 (`5d56be49`): the release is done only when that script prints its final
  `OK`.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The release refuses without a deployment permit | ran | `scripts/release.sh 2.56.0` on 2026-10-18 | `release  FAIL: release authorization required — BEE_DISPATCH_ID is unset (dispatch with deployment stage under deploy role required); nothing was changed` |
| 2 | A deployment stage needs an approved v2 role plan, as the 2.55.0 lane had | read | docs/history/release-2-55-0/plan.md:80 | `## Role assignments` |
| 3 | The script checks the permit before it moves anything | read | scripts/release.sh:116 | `"$BEE_BIN" dispatch authorize --id "$BEE_DISPATCH_ID" --release-version "$AUTH_VERSION" \` |
| 4 | The version moves from the released 2.55.0 | ran | `.bee/bin/bee --version` | `bee 2.55.0` |
| 5 | The work this release ships is capped and closed | read | git log v2.55.0..HEAD | `efa5c544c Record the open-issues-sweep close and its promote proposals` |

## Cells (current slice)

```json
[
  {
    "id": "rel560-1",
    "feature": "release-2-56-0",
    "title": "Publish release 2.56.0 through the sanctioned script",
    "lane": "tiny",
    "role": "deploy",
    "deps": [],
    "decisions": [],
    "files": [".claude-plugin/plugin.json", ".codex-plugin/plugin.json", "docs/history/codex-harness-hardening/release-manifest.json"],
    "read_first": ["docs/history/release-2-56-0/CONTEXT.md", "scripts/release.sh"],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run `scripts/release.sh 2.56.0` with BEE_DISPATCH_ID set to an authorized deployment dispatch. The script bumps both plugin manifests, runs the regen chain (render-skill-trees, onboard --apply, release-manifest --write), runs the declared test suite before tagging, makes the release commit, tags v2.56.0, pushes main and the tag, waits for the release-binaries workflow and verifies the published assets. Never walk those steps by hand. Re-running at a version already committed is idempotent — it resumes the tail. The regen obligation on .claude-plugin/plugin.json is discharged INSIDE the script's own regen chain, which is why the release manifest rides this cell's files.",
    "verify": "bash scripts/release.sh 2.56.0 && bee dev release-manifest --check",
    "must_haves": {
      "truths": [
        "both plugin manifests report 2.56.0",
        "tag v2.56.0 exists locally and on the remote",
        "the GitHub release for v2.56.0 carries the binaries and SHA256SUMS",
        "the script printed its final OK line"
      ],
      "artifacts": [
        {"path": ".claude-plugin/plugin.json", "substantive": "version 2.56.0"},
        {"path": ".codex-plugin/plugin.json", "substantive": "version 2.56.0"}
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
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage":"deployment","classification":"required","role":"deploy","reason":"This lane exists to publish 2.56.0; the deploy role owns the release permit."},
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
