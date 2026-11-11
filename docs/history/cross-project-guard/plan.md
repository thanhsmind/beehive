---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Work on another project goes to its leader

## Summary

A bee session can no longer read or search another project's code. When
it tries, the guard stops the call and tells it to hand the work to that
project's leader through waggledance. The leader doctrine says the same
thing, so the rule reaches every project on its next bee refresh.

Mode: `standard` — 2 risk flags: public-contracts, multi-domain
Why this is the least workflow that protects the work: one new guard rule
changes what every session in every project may read, so it needs its own
tests and a doctrine line, but no new subsystem.

## Requirements (from CONTEXT.md)

- D1: another project = outside this root and its worktrees, nearest `.git` or `.bee/onboarding.json` root is a different repo; a folder whose direct child is such a root counts; no-repo paths stay allowed.
- D2: refuse Read, Glob, Grep on such a path, and Bash whose cwd or any absolute or `~` path token is such a path; leaders and workers, every phase.
- D3: the refusal names the root and the waggledance remedy, and the allowlist route for research.
- D4: `guards.cross_project_allow`, Claude `permissions.additionalDirectories`, and harness surfaces stay allowed; `guards.cross_project_allow` is governed.
- D5: doctrine rule `agents-other-project-to-its-leader`.

## Load-bearing claims

Labels: `read` = opened at that line; `ran` = executed. Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Read/Glob/Grep take the read branch of the write-guard | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:206 | `let is_read_tool = matches!(tool_name.as_str(), "Read" \| "Glob" \| "Grep");` |
| 2 | An outside-root read path is dropped before any check runs | read | packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs:330 | `if rel.is_empty() \|\| rel == "." \|\| rel.starts_with("..") \|\| np_is_absolute(&rel) {` |
| 3 | Bash reaches the same hook as a shell tool | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:205 | `let is_shell = matches!(tool_name.as_str(), "Bash" \| "exec");` |
| 4 | The Claude wiring already sends Read, Glob, Grep and Bash to write-guard | read | .claude/settings.json:65 | `"matcher": "Edit\|Write\|MultiEdit\|Bash\|Read\|Glob\|Grep\|AskUserQuestion",` |
| 5 | A governed-key refusal already exists for `bee config set` | read | packages/bee-rs/crates/bee/src/verbs/config.rs:111 | `adds a governed config key the config write guard protects` |
| 6 | The doctrine template carries rule markers | read | packages/bee/LEADER.md:255 | `<!-- rule: agents-feedback-changes-framework -->` |
| 7 | The user's Claude settings grant extra directories the guard must not refuse | read | /home/thanhsmind/.claude/settings.json:33 | `"additionalDirectories": [` |
| 8 | The hook keeps its per-repo off switch | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:227 | `if !hook_enabled(&store_root_pb, HOOK_NAME) {` |

## Discovery

A read-only gather (job-1794385768182-3069995-1) mapped the PreToolUse
wiring, the write-guard branches and the config knobs. The finding: the
read branch only checks in-repo paths, and the Bash branch only looks at
write targets, so nothing stops a session reading another repo. The
skill hub (`~/skillstation-hub`) is a git repo reached through
`permissions.additionalDirectories`, which is why D4 allows those entries.

## Approach

Recommended path: one new check in the write-guard (D1-D4), called before
the read and write branches, plus the governed key in the config guard;
a separate doctrine cell (D5). Both cite cc1d4957.

Rejected: a memory note (the owner said it fails); doctrine only (prose is
what already failed); a new hook name (the write-guard already receives
every tool this needs, claim 4); an agent-set env escape (the agent could
turn the mechanism off).

Risk map:
- Bash token scan refuses a legitimate in-repo command — MEDIUM — Lands in: cpg-1 — proof: tests that in-repo, `/tmp`, worktree-sibling and additionalDirectories paths stay allowed.
- Skill hub paths refused — MEDIUM — Lands in: cpg-1 — proof: a fixture with `permissions.additionalDirectories` pointing into another repo is allowed.
- Doctrine rule drifts from the index — LOW — Lands in: cpg-2 — proof: `bee knowledge check` and the doctrine parity tests.

Waves: cpg-1 and cpg-2 run in parallel (no shared files); cpg-3 waits for both.

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "d4e696dbde7989d6388dacce3fd7057cb3a6935b46024b79698596a46f6f41d3",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"cpg-1 writes the Rust guard and its tests."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"cpg-2 writes the doctrine rule, its index row and the config reference."},
    {"stage":"writing","classification":"required","role":"writer","reason":"cpg-3 consolidates knowledge and writes delivery.md."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"Release runs on the owner's word through scripts/release.sh."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"test-and-live-proof","classification":"not-applicable","role":"test","reason":"Each cell writes its own red-first tests."},
    {"stage":"read-only-gather","classification":"not-applicable","role":"read","reason":"The gather already ran during planning."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"No narrow lookup is left."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Independent review stays user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"No hard-gate flag; standard with 5 or fewer product files runs the plan check inline."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape, no competing designs."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"design-ui","classification":"not-applicable","role":"design","reason":"No rendered UI."},
    {"stage":"hat-facts-gaps","classification":"not-applicable","role":"hat-facts-gaps","reason":"Standard, 4 product files, no hard-gate flag: inline plan check."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Not high-risk."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Not high-risk."},
    {"stage":"hat-alternatives","classification":"not-applicable","role":"hat-alternatives","reason":"Inline SMALLER PATH check below."},
    {"stage":"hat-user-impact","classification":"not-applicable","role":"hat-user-impact","reason":"Inline plan check."}
  ]
}
```

## Shape

One slice, three cells. Playbook: `skills/bee-planning/playbooks/feature.md` (cited, not copied).

SMALLER PATH: doctrine-only is cheaper but breaks the owner's ask for a mechanism; dropping the additionalDirectories allowance is cheaper but refuses the skill hub in every project. PASS.

Inline plan check — STRUCTURE: D1-D4 land in cpg-1, D5 in cpg-2; deps are a DAG; every cell has files, read_first, action, must_haves and verify. No BLOCKER. CELLS: cold pickup checked for each cell; no CRITICAL.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| cpg-1 | Refuse reads and commands that touch another project's tree | write_guard/main.rs, write_guard/cross_project.rs (new), write_guard/mod.rs, write_guard/tests.rs, verbs/config.rs, write-guard-request-shapes.md | — | `Read /other/repo/x` and `cd /other/repo && rg` are refused with the waggledance remedy; in-repo and skill-hub reads still pass | `cargo test … --bin bee -- cross_project` |
| cpg-2 | Add the other-project-to-its-leader doctrine rule | packages/bee/LEADER.md, .bee/LEADER.md (regen), router-triage doc, docs/config-reference.md | — | The leader doctrine and rule index carry `agents-other-project-to-its-leader`; the config reference names `guards.cross_project_allow` | `bee knowledge check` and doctrine parity tests |
| cpg-4 | Check relative .. path tokens in Bash against the cross-project rule | cross_project.rs, main.rs, tests.rs, write-guard-request-shapes.md | cpg-1 | `cat ../../<other repo>/x` is refused like an absolute path | `cargo test … --bin bee -- cross_project` |
| cpg-3 | Consolidate this feature's knowledge and write its delivery | docs/knowledge/**, docs/history/cross-project-guard/delivery.md | cpg-1, cpg-2, cpg-4 | delivery.md lists the changed concepts | `.bee/bin/bee knowledge check` |

```json
[
  {
    "id": "cpg-1",
    "feature": "cross-project-guard",
    "title": "Refuse reads and commands that touch another project's tree",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [
      "cc1d4957"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/mod.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/config.rs",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "read_first": [
      "docs/history/cross-project-guard/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/verbs/config.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract cc1d4957 (docs/history/cross-project-guard/CONTEXT.md D1-D4). Add write_guard/cross_project.rs with a check that, given this project's roots, the hook cwd and a candidate path, returns the other project's root when the path belongs to another project per D1: resolve the path (absolute, cwd-relative, or ~ expanded from HOME); it is this project when it is under the store/main root or under a directory whose .git resolves to the same git common dir (this repo's worktrees); otherwise walk up from the path to the nearest ancestor holding .git or .bee/onboarding.json, never counting HOME or / as a root; a different root means another project; a directory that is in no repo but has a direct child holding .git or .bee/onboarding.json (not this project) also counts. Allowed despite a match (D4): a path under any entry of guards.cross_project_allow in .bee/config.json (absolute or ~ prefixes), under any permissions.additionalDirectories entry of ~/.claude/settings.json, <root>/.claude/settings.json or <root>/.claude/settings.local.json, or a harness-allowlisted target (hook_local.rs harness_allowlisted_target). In main.rs run the check after the hook_enabled switch and before the AskUserQuestion/read/write branches, for every phase: Read uses file_path; Glob and Grep use path, and an absolute Glob pattern prefix; Bash/exec checks the hook cwd and every token from tokenize_deep(fence_heredocs(command)) that starts with / or ~ (strip a leading key= of --flag=value tokens). The first hit denies with exit 2 and a message that starts with 'bee cross-project guard:', names the path and the other project root, says not to read or work it from this session, and gives the remedy: waggledance_ask_state for that project, then waggledance_dispatch with leader: true; if the user asked to research that repo from here, ask the user to add the path to guards.cross_project_allow in .bee/config.json. Make guards.cross_project_allow a governed key: extend config_governed_change_deny so bee config set and an agent Edit/Write that adds or changes it are refused like the other governed keys. Red first in write_guard/tests.rs with test names containing cross_project: Read of a file in a sibling fixture repo is refused with the remedy; Grep path at a folder whose child is a repo is refused; Bash 'cat <sibling>/x' and 'cd <sibling> && ls' are refused; Bash with hook cwd inside the sibling is refused; in-repo Read, /tmp Read, a sibling worktree of this repo, a path under guards.cross_project_allow, and a path under a fixture permissions.additionalDirectories entry are all allowed; the check runs while phase is idle; bee config set guards.cross_project_allow is refused. Watch them fail before the fix. Knowledge delta in the same commit: add a cross-project section to docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md citing docs/history/cross-project-guard/CONTEXT.md D1-D4 (not the raw decision id). No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cross_project",
    "must_haves": {
      "truths": [
        "A Read, Glob, Grep or Bash call that touches another repo's tree is refused with exit 2 and the waggledance remedy",
        "In-repo paths, this repo's worktrees, no-repo paths, guards.cross_project_allow entries and Claude additionalDirectories entries stay allowed",
        "The refusal fires in every phase, idle included, for leaders and workers",
        "bee config set cannot add guards.cross_project_allow"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
          "substantive": "the D1 ownership check and the D4 allowances"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
          "substantive": "red-first cross_project tests for every D1-D4 case"
        }
      ],
      "key_links": [
        "main.rs calls the cross-project check before the read and write branches",
        "config_governed_change_deny protects guards.cross_project_allow"
      ],
      "prohibitions": [
        "Never weaken an existing read, privacy or write check",
        "Never let an environment variable turn the check off",
        "No code comments"
      ],
      "open_choices": [
        "Cache the walked roots per hook call"
      ]
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
    "id": "cpg-2",
    "feature": "cross-project-guard",
    "title": "Add the other-project-to-its-leader doctrine rule",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": [
      "cc1d4957"
    ],
    "files": [
      "packages/bee/LEADER.md",
      ".bee/LEADER.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/config-reference.md"
    ],
    "read_first": [
      "docs/history/cross-project-guard/CONTEXT.md",
      "packages/bee/LEADER.md",
      "docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md",
      "docs/config-reference.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract cc1d4957 (docs/history/cross-project-guard/CONTEXT.md D5, D3, D4). In packages/bee/LEADER.md, section Work in parallel, coordinate through the store, add a marked rule block <!-- rule: agents-other-project-to-its-leader --> ... <!-- /rule -->: work that belongs to another project goes to that project's leader through waggledance — waggledance_ask_state for that project, then waggledance_dispatch with leader: true (follow_up_of when its leader already runs the same work); the session never reads, searches or explores that project's code, and the write-guard refuses it; a research read of another repo needs the user to add it to guards.cross_project_allow. Regenerate the vendored copy with the repo's regen command (bee dev regen) so .bee/LEADER.md matches. Add the rule to the rule index in the router-triage doc beside agents-feedback-changes-framework, same shape: home, spoken line citing cross-project-guard D5, applied_at. Add guards.cross_project_allow to docs/config-reference.md: list of absolute or ~ path prefixes, governed (user edits only), with Claude permissions.additionalDirectories also allowed. ASD-STE100 style, no bee jargon beyond rule ids. One commit.",
    "verify": ".bee/bin/bee knowledge check && cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- doctrine",
    "must_haves": {
      "truths": [
        "packages/bee/LEADER.md and .bee/LEADER.md carry rule agents-other-project-to-its-leader",
        "The rule index lists it with home, spoken line and applied_at",
        "docs/config-reference.md documents guards.cross_project_allow as governed"
      ],
      "artifacts": [
        {
          "path": "packages/bee/LEADER.md",
          "substantive": "the marked rule block"
        }
      ],
      "key_links": [
        "the rule names the write-guard refusal and the waggledance remedy"
      ],
      "prohibitions": [
        "No hand edit of .bee/LEADER.md beyond the regen output"
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
  },
  {
    "id": "cpg-4",
    "feature": "cross-project-guard",
    "title": "Check relative .. path tokens in Bash against the cross-project rule",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [
      "cpg-1"
    ],
    "decisions": [
      "cc1d4957",
      "118414b3",
      "a239cdf4"
    ],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "read_first": [
      "docs/history/cross-project-guard/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/cross_project.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contracts 118414b3 and a239cdf4. Today the Bash cross-project scan in main.rs and cross_project.rs checks only tokens starting with / or ~, so 'cat ../../anphabe/a-blog/package.json' run from the beehive worktree exits 0. Extend the scan: a Bash token that is a relative path containing a .. segment (also the value of a --flag=value token) is resolved against the hook cwd and checked by the same rule; walk the command's cd tokens in order so a relative token after a relative cd (cd ../y && cat z) resolves against the cd target; a cd to an absolute target resets the base. Relative tokens without .. stay unchecked, as today. Red first in write_guard/tests.rs, names containing cross_project: 'cat ../../<sibling>/x' refused; 'cd ../<sibling-parent> && cat <sibling>/x' refused; 'cat ../<this-repo-dir>/AGENTS.md' resolving into this repo is allowed; 'cat src/../AGENTS.md' allowed. Watch them fail first. Update the cross-project section of write-guard-request-shapes.md (cite CONTEXT.md D2). No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cross_project",
    "must_haves": {
      "truths": [
        "A Bash relative token with .. that resolves into another project is refused with the cross-project message",
        "A relative .. token that resolves inside this repo stays allowed"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
          "substantive": "red-first cross_project tests for .. tokens"
        }
      ],
      "key_links": [
        "main.rs hands .. tokens and the cd chain to the cross-project check"
      ],
      "prohibitions": [
        "Never weaken an existing check",
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
    "id": "cpg-3",
    "feature": "cross-project-guard",
    "title": "Consolidate this feature's knowledge and write its delivery",
    "lane": "standard",
    "role": "writer",
    "status": "open",
    "deps": [
      "cpg-1",
      "cpg-2",
      "cpg-4"
    ],
    "decisions": [
      "cc1d4957"
    ],
    "files": [
      "docs/knowledge/**",
      "docs/history/cross-project-guard/delivery.md"
    ],
    "read_first": [
      "docs/history/cross-project-guard/CONTEXT.md",
      "docs/history/cross-project-guard/plan.md"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Run the writer consolidation step on this feature: run `bee knowledge consolidate --feature cross-project-guard`. Dedupe and trim the deltas the cells wrote, anchor each concept to the code it describes, never restate code, and write docs/history/cross-project-guard/delivery.md listing the concepts changed and why, one line each. Run `.bee/bin/bee knowledge check` green. One commit.",
    "verify": ".bee/bin/bee knowledge check",
    "must_haves": {
      "truths": [
        "area concepts describe the feature's changes, anchored to code",
        "docs/history/cross-project-guard/delivery.md lists every concept the feature changed"
      ],
      "artifacts": [
        {
          "path": "docs/history/cross-project-guard/delivery.md",
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

Amendment (after the first approval, decision a239cdf4): the leader probe of cpg-1 found `cat ../../anphabe/a-blog/package.json` exits 0; cpg-4 closes that gap. cpg-3 still runs last.

## Test matrix

- Happy: Read of `<sibling repo>/src/x` refused, message names the sibling root and `waggledance_dispatch`. Pass when exit 2 and stderr starts `bee cross-project guard:`.
- Edge: sibling worktree of this repo, `/tmp/x`, a `guards.cross_project_allow` path, an additionalDirectories path. Pass when each is allowed (no denial).
- Edge: Grep at a folder of repos; Bash `cd <sibling> && ls`; Bash cwd inside the sibling. Pass when each is refused.
- Error: `bee config set --key guards.cross_project_allow --value '["/x"]'`. Pass when refused, file unchanged.
- Main vs head: Read of a sibling repo file is allowed on main and refused on head.

## Open Questions

(none)

## Out of scope

- Non-Claude runtimes whose hook wiring does not send Read/Glob/Grep to write-guard; they get the doctrine rule only.
- Release: runs on the owner's word through scripts/release.sh.
