---
artifact_contract: bee-plan/v1
feature: pi-version-floor
lane: small
---

# pi-version-floor — plan

Mode: `small` — 0 risk flags, 3 product files, one direct task.
Why this is the least workflow that protects the work: comments only, no
behavior change, and the existing belt-parity tests already cover the file.

## Summary

bee's Pi belt never states which Pi versions it supports. Two comments name
`0.84.3` as the version whose docs were read, and a reader has been taking
those as the supported version. They are provenance, not support range, and
the supported floor is actually 0.84.4.

This adds the missing statement and leaves every provenance citation as
written.

## Requirements

- The floor is 0.84.4, because the belt registers `ui_prompt_start` and
  `ui_prompt_end` and Pi added both in 0.84.4.
- Provenance comments — the ones naming which Pi docs or binary were read —
  are not rewritten. pi-stage-dispatch fixed that rule in its own plan.
- The built-in tool registry comment keeps its 0.84.3 provenance and gains the
  fact that the list was re-verified unchanged at 0.85.1.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | The belt registers both `ui_prompt_*` events | read | `.pi/extensions/bee-guard.ts:2179,2201` | `pi.on("ui_prompt_start", …)` and `pi.on("ui_prompt_end", …)` |
| 2 | Pi added those two events in 0.84.4 | read | Pi 0.85.1 `CHANGELOG.md`, 0.84.4 section (2026-08-28) | "Added `ui_prompt_start` and `ui_prompt_end` extension events" (issue 8355); both are absent from 0.84.3 `docs/extensions.md` |
| 3 | Rewriting provenance comments is ruled out by a prior plan | read | `docs/history/pi-stage-dispatch/plan.md:207` | "leave comments that name which Pi docs or binary were read unchanged" |
| 4 | The eight built-in names are unchanged across the range | ran | Pi 0.84.3 and 0.85.1 `docs/settings.md` | `rg -n 'Available built-ins are'` returns the identical sentence and the identical eight names at `:223` (0.84.3) and `:228` (0.85.1) |
| 5 | The belt is byte-compared by the doctor at compile time | read | `packages/bee-rs/crates/bee/src/doctor.rs:47` | `include_str!("../../../../../.pi/extensions/bee-guard.ts")` |
| 6 | `.pi/extensions` is a shipped release-manifest root | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:83` | `.pi/extensions` listed among the inventoried roots |
| 7 | The contract test derives the tool list from the source, so a comment edit cannot move it | read | `packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs:115-118` | `const MARKER: &str = "const PI_BUILTIN_TOOLS = [";` — parsed "rather than restated here" |

## Discovery

`rg -n 'marker_header'`-style sweeps are not needed here; the whole change is
comment text. The one real question was whether to rewrite the `0.84.3`
labels, and claim 3 answers it: a prior feature deliberately preserved them.
The floor statement is therefore an addition, not a relabel.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pvf-1 | State the Pi version range the belt supports | the belt plus two contract tests | — | A reader of the belt learns it needs Pi 0.84.4 or newer, instead of inferring 0.84.3 from a provenance note | the declared release suite green |

```json
[
  {
    "id": "pvf-1",
    "feature": "pi-version-floor",
    "title": "State the Pi version range the belt supports",
    "lane": "small",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      ".pi/extensions/bee-guard.ts",
      "packages/bee-rs/crates/bee/tests/pi_plugin_contracts.rs",
      "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs"
    ],
    "read_first": [
      "docs/history/pi-version-floor/plan.md",
      "docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md",
      ".pi/extensions/bee-guard.ts"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Add a titled comment block to the belt header, directly above the imports, stating the supported Pi range: FLOOR 0.84.4 because the belt registers ui_prompt_start and ui_prompt_end and Pi added both in 0.84.4; CEILING none proven, with 0.85.1 named as the newest version a bee workflow was driven on. Say in that block that the provenance comments elsewhere in the file record which docs were READ and are a different fact, so nobody rewrites them later. Point at docs/knowledge/areas/hook-runtime/pi-version-pin-and-capability-audit.md for the per-version dispositions. Then extend the PI_BUILTIN_TOOLS comment so it keeps its 0.84.3 provenance and adds that the same eight names were re-verified unchanged in Pi 0.85.1's docs/settings.md. In the two contract tests, extend the comment and the assertion message the same way: keep the 0.84.3 wording, add the re-verification, so the next reader does not file the label as stale. Comments and assertion strings only — change no code path, no assertion condition, and no provenance citation.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "The belt header states a floor of 0.84.4 and names the two events that set it",
        "Every existing comment naming which Pi docs or binary were read survives byte-identical except for appended re-verification text",
        "No assertion condition changes; only comments and assertion message strings"
      ],
      "artifacts": [
        {"path": ".pi/extensions/bee-guard.ts", "substantive": "carries a supported-range block above the imports; no TODO"}
      ],
      "key_links": [
        "the header block points at the capability-audit concept by path"
      ],
      "prohibitions": [
        "No rewrite of the 0.84.3 provenance citations at :19 and :317",
        "No change to PI_BUILTIN_TOOLS itself",
        "No behavior change anywhere in the belt"
      ]
    },
    "trace": {
      "worker": null, "outcome": null, "files_changed": [],
      "deviations": [], "friction": null, "capped_at": null,
      "behavior_change": false
    }
  }
]
```

## Test matrix

| Scenario | Pass when |
|---|---|
| Existing belt parity and contract tests | green, unchanged — the tool list derives from the source, so comment text cannot move it |
| `bee doctor` belt byte-compare | green — the embedded copy and the file move together at compile time |
| Release manifest | refreshed, because `.pi/extensions` is a shipped root |

## Open Questions

(none)

## Out of scope

The other three findings filed alongside this one: the codex concept's dead
pointers, the dead `paths` block in two prompt templates, and the two herding
flags missing from the porcelain help.
