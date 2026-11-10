# Plan: Absolute write targets after a cd

## Summary

Today the Bash write guard refuses any write target that comes after a `cd`
in the same compound command, even an absolute path. After this, an absolute
target after a `cd` goes through the normal checks. A relative target after a
`cd` is still refused.

Mode: `standard` — 0 risk flags; guard source under `hooks/` owes an independent judge read.
Why this is the least workflow that protects the work: one guard rule narrows
for one path shape; every other check still runs on that path.

Class playbook: bugfix — `skills/bee-planning/references/planning-reference.md` ("Playbooks").

## Requirements (from CONTEXT.md)

- D1: absolute targets after a `cd` are not cd-opaque → gcd-1.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The extractor marks every target after the first cd as opaque | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:1050 | `let cd_opaque = match cd_marker {` |
| 2 | The marker is set at the first cd token | read | packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs:860 | `cd_marker = Some(paths.len());` |
| 3 | An opaque target is refused before its path is resolved | read | packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs:388 | `if targets.cd_opaque.get(idx).copied().unwrap_or(false) {` |

## Discovery

Read the write-guard-request-shapes concept. Its `cd` bullet gives the reason
for the rule: the guard resolves a target against the request's root, not the
directory the `cd` moved to. That reason does not hold for an absolute path.

## Approach

- In `extract_bash_targets`, a target is cd-opaque only when it comes after
  the `cd` marker AND `Path::new(target).is_absolute()` is false.
- Red first: a test where `cd /tmp && touch <absolute in-root path>` is
  allowed, and `cd /tmp && touch <absolute .bee state file>` is refused with
  the direct-edit wording, not the cd wording. Both fail before the fix.

Rejected: drop the `cd` rule (relative targets really resolve against the
wrong directory); track the `cd` destination (its argument can be a variable
or `-`, which the extractor cannot see).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| Write guard Bash surface | MEDIUM | gcd-1 | the absolute targets still meet containment and direct-edit checks |
| Relative targets after cd | LOW | gcd-1 | the existing cd-opaque tests stay green |

## Shape

One slice, one cell.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| gcd-1 | Check absolute write targets after a cd instead of refusing them | write_guard/guards.rs, write_guard/tests.rs, write-guard-request-shapes.md | — | `cd X && touch /abs/in-root/file` is allowed; a relative target after cd is still refused | `cargo test … --bin bee -- cd_` |

```json
[
  {
    "id": "gcd-1",
    "feature": "guard-cd-absolute-targets",
    "title": "Check absolute write targets after a cd instead of refusing them",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": ["d199ac87"],
    "files": [
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md"
    ],
    "read_first": [
      "docs/history/guard-cd-absolute-targets/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
      "packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Keep contract d199ac87 (guard-cd-absolute-targets). In write_guard/guards.rs extract_bash_targets, mark a target cd-opaque only when its index is at or after the cd marker AND std::path::Path::new(target).is_absolute() is false; an absolute target after a cd is then resolved and checked by main.rs exactly like a target with no cd. Targets before a cd and relative targets after a cd keep their current behavior; a ~ path stays relative here. Red first in write_guard/tests.rs: extend or add extractor tests named with cd_ so the filter runs them (an absolute target after cd is not opaque, a relative one is, mixed commands mark per target), and fixture tests named with cd_ through the existing bash()/expect_done helpers: cd /tmp && touch <fixture root absolute path to an ordinary in-root file> is allowed; cd /tmp && touch <fixture root absolute path>/.bee/state.json is refused with the direct-edit wording and never the cd wording; cd /tmp && touch out.txt is still refused with the cd wording. Watch the new tests fail before the fix. Knowledge delta in the same commit: in docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md change the cd bullet to say the refusal applies to relative targets only and an absolute target after a cd is checked normally, citing docs/history/guard-cd-absolute-targets/CONTEXT.md D1 (not the raw decision id). No code comments. One commit.",
    "verify": "cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- cd_",
    "must_haves": {
      "truths": [
        "An absolute write target after a cd is resolved and checked like a target with no cd",
        "A relative write target after a cd is still refused with the cd wording",
        "An absolute target after a cd that hits a guarded bee state file is still refused"
      ],
      "artifacts": [
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs",
          "substantive": "cd_opaque excludes absolute targets"
        },
        {
          "path": "packages/bee-rs/crates/bee/src/hooks/write_guard/tests.rs",
          "substantive": "red-first tests for absolute and relative targets after cd"
        }
      ],
      "key_links": [
        "main.rs refuses only targets that guards.rs marks cd_opaque"
      ],
      "prohibitions": [
        "Never skip containment or direct-edit checks for an absolute target",
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
  }
]
```
