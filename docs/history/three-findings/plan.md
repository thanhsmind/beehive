---
artifact_contract: bee-plan/v1
feature: three-findings
lane: standard
---

# three-findings — plan

Mode: `standard` — 2 risk flags (public-contracts, covered-contract-change),
10 product files.
Why this is the least workflow that protects the work: two of the three fixes
touch a shipped contract surface (the prompt templates and the CLI help
registry), and one deliberately moves a value an existing test pins.

## Summary

Three independent fixes from the pi-capability-audit lane's findings. They
share no files, so they run as three parallel cells.

## Requirements (from CONTEXT.md)

- D1: delete the dead `paths` block; do not wire a supplier.
- D2: correct the codex concept's two dead pointers, keep its live one.
- D3: add the two herding flags to the registry and bump the pinned count with
  its reason in the ledger.
- D4: the stale `Paths:` sentence in the product description moves with D1.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|---|---|---|---|
| 1 | Nothing supplies `paths` in any source or test | ran | `packages/bee-rs` | `rg '"paths"' packages/bee-rs --glob '*.rs'` returns only lease dirs, knowledge anchors and unrelated flag parsing; the non-cell vars slice at `verbs/drivers/prepare.rs:1080-1084` is `brief`, `expertise`, `purpose`, `original_request`, `seat` |
| 2 | The block was introduced as a cosmetic fix, with no supplier | read | commit `58be9dfec` (2026-09-07) | Message: "The gather/advisor/reviewer prompts shipped a literal `\"<caller fills in the exact files/paths to read>\"` placeholder in the rendered payload; it is now a conditional block." Its stat touches only the four prompt files and their `.bee/bin/prompts/` twins — no `.rs` file |
| 3 | The goldens were already re-cut for the empty render | read | commit `bc232b8fe` (2026-09-07) | "That line became a conditional block in the prompt templates, so a dispatch supplying no paths renders none of it." |
| 4 | A falsy block leaves zero residue, so deletion changes no rendered bytes | read | `packages/bee-rs/crates/bee/src/devtools/prompts.rs` | Corpus case `("head\n{{#if f}}\nbody\n{{/if}}\ntail", &[("f", "")], Ok("head\ntail"))` |
| 5 | `PROBED_CODEX_VERSION` does not exist | ran | `packages/bee-rs/crates/bee/src` | `rg 'PROBED' packages/bee-rs/crates/bee/src` returns nothing |
| 6 | `scripts/canary_codex.mjs` does not exist | ran | repo root | `fd -t f 'canary_codex*'` returns nothing |
| 7 | `.bee/cells/i54-closeout-8.json` does not exist | ran | `.bee` | `fd -t f 'i54*' .` returns nothing |
| 8 | The concept's report pointer DOES exist and stays | ran | `docs/history/i54-closeout/reports/validation-canary.md` | `ls` returns the file |
| 9 | The surviving version mechanism is the attestation record | read | `packages/bee-rs/crates/bee/src/doctor.rs:46,772-784` | `const ATTEST_REL: &str = ".bee/doctor-attest.json";` and the three-leg match yielding `unprobed_version` / `version_changed` |
| 10 | Distinct flag names today total exactly 208, matching the pin | ran | `packages/bee-rs/crates/bee/src/generated/registry_payload.json` | A count over every command's `parameters.properties` returns 208; `catalog.rs:788` reads `const PINNED_FLAG_COUNT: usize = 208;` |
| 11 | Neither `seat` nor `inbox-session` appears anywhere in the registry | ran | same payload | Both lookups return "on commands = []", so both are new spellings and the count moves to 210 |
| 12 | `herding.run` really does parse both flags | read | `packages/bee-rs/crates/bee/src/herding/run.rs:369-373` | `"--seat" => { … }` and `"--inbox-session" => { … }` |
| 13 | `packages/bee` is a manifest root and `packages/bee-rs` is not | read | `packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:74-88` | `INVENTORY_ROOTS` lists `"packages/bee"`; no `packages/bee-rs` entry |
| 14 | The product-description sentence is stale | read | `docs/product-description/delegation/dispatch.md:47` | "the Agent tool, with that payload, its own task text filled into the prompt's `Paths:` line" |

## Discovery

The one question worth asking was why a dead block existed at all. Git history
answered it: a prompt-wording audit replaced placeholder prose with a
conditional and stopped there. Nothing was ever wired, so nothing is being
taken away. The deletion test agrees — remove the block and no behavior
reappears anywhere.

## Approach

Three cells, no shared files, run in parallel.

Waves: all three in one wave. No serial edge — thf-1 owns the prompt templates
and the manifest, thf-2 owns one knowledge page, thf-3 owns the registry
payload and `catalog.rs`.

`.bee/bin` is covered by the onboarding managed-hash ledger as well as the
release manifest, so thf-1 owes both regen doors and carries both in its own
verify. The ids are `thf-*` rather than `tf-*` because cell ids are global and
the dropped `two-flows` feature already holds `tf-1` and `tf-2`.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| thf-1 | Delete the dead paths block from the dispatch prompts | 3 templates, 3 vendored twins, 1 product-description page, the manifest | — | Nothing changes in any prompt a worker receives; the templates stop carrying a block that could never fire | the declared suite plus the manifest check |
| thf-2 | Point the codex probe concept at the mechanism that exists | 1 knowledge page | — | A reader following the concept's pointers lands on real files | knowledge check and index freshness |
| thf-3 | Publish the two herding flags in the command registry | the registry payload and catalog.rs | — | `bee herding run --help` lists `--seat` and `--inbox-session` | the declared suite, including the pinned-count test |

```json
[
  {
    "id": "thf-1",
    "feature": "three-findings",
    "title": "Delete the dead paths block from the dispatch prompts",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee/prompts/gather.md",
      "packages/bee/prompts/reviewer.md",
      "packages/bee/prompts/advisor.md",
      ".bee/bin/prompts/gather.md",
      ".bee/bin/prompts/reviewer.md",
      ".bee/bin/prompts/advisor.md",
      "docs/product-description/delegation/dispatch.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/three-findings/CONTEXT.md",
      "packages/bee/prompts/gather.md",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Delete the `{{#if paths}}` … `{{/if}}` block, including its `Paths:` line and the `{{paths}}` placeholder, from packages/bee/prompts/gather.md, reviewer.md and advisor.md. Per D1 it is dead surface: nothing supplies the var (prepare.rs:1080-1084 passes brief, expertise, purpose, original_request, seat and nothing else), so the block already renders to nothing on every dispatch and its removal changes zero rendered bytes. Delete the block only — leave every other line of all three templates byte-identical, and do NOT add a supplier. Then sync the three .bee/bin/prompts/ twins so both on-disk copies stay byte-identical, because verbs/drivers/prompt.rs prompt_skew checks BOTH and refuses a dispatch on either mismatch; .bee/bin is covered by the onboarding managed-hash ledger, so do that sync by running `bee onboard --repo-root . --apply` rather than by hand. Then fix the one stale sentence at docs/product-description/delegation/dispatch.md:47, which still says the agent's task text is 'filled into the prompt's `Paths:` line' — that stopped being true at commit 58be9dfec; rewrite it to describe what the payload actually carries, without inventing a new mechanism. Finally run the regen chain (bee dev regen, or at minimum bee dev release-manifest --write) because packages/bee is a release-manifest inventory root.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json",
    "must_haves": {
      "truths": [
        "No `{{#if paths}}` block and no `{{paths}}` placeholder remains in any of the three templates",
        "Every other line of the three templates is byte-identical to before",
        "packages/bee/prompts and .bee/bin/prompts hold byte-identical copies of all three files",
        "No supplier for a paths var is added anywhere",
        "The product-description sentence no longer claims a `Paths:` line exists"
      ],
      "artifacts": [
        {"path": "packages/bee/prompts/gather.md", "substantive": "no paths block; the digest-contract line and the expertise block survive unchanged"},
        {"path": ".bee/bin/prompts/gather.md", "substantive": "byte-identical to packages/bee/prompts/gather.md"}
      ],
      "key_links": [
        "both on-disk prompt copies agree, so prompt_skew stays None"
      ],
      "prohibitions": [
        "No new template var and no supplier in prepare.rs",
        "No edit to the grammar corpus in devtools/prompts.rs",
        "No change to any other block in the three templates"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "thf-2",
    "feature": "three-findings",
    "title": "Point the codex probe concept at the mechanism that exists",
    "lane": "standard",
    "role": "docs",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "docs/knowledge/areas/hook-runtime/codex-capability-probe-version-pin-and-re-probe-evidence.md"
    ],
    "read_first": [
      "docs/history/three-findings/CONTEXT.md",
      "docs/knowledge/areas/hook-runtime/codex-capability-probe-version-pin-and-re-probe-evidence.md",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D2, correct this concept's dead pointers. Three things it names do not exist at this commit: the constant PROBED_CODEX_VERSION 'in the bee binary' (rg 'PROBED' over packages/bee-rs/crates/bee/src returns nothing), the canary scripts/canary_codex.mjs (fd finds nothing), and the cell trace .bee/cells/i54-closeout-8.json (fd finds nothing). One thing it names DOES exist and must stay cited: docs/history/i54-closeout/reports/validation-canary.md. Replace the dead ones with the mechanism that survived the Rust port: the attestation record .bee/doctor-attest.json (doctor.rs:46 ATTEST_REL), written by `bee doctor attest --runtime codex`, whose three legs are the hooks-file hash, the codex version and the repo identity, and whose version leg yields reason `unprobed_version` when the live CLI does not answer and `version_changed` when it answers differently (doctor.rs:772-784). Keep the concept's RULE intact — a pin moves only on live evidence — because that rule is this page's reason to exist and it is unchanged; only its implementation pointers and its frontmatter sources were wrong. Update the frontmatter sources list to drop the dead cell-trace entry. Do not restate anything the Pi sibling page owns.",
    "verify": ".bee/bin/bee knowledge check --json && .bee/bin/bee knowledge index --check --json",
    "must_haves": {
      "truths": [
        "Every path and symbol the page names exists at this commit",
        "The evidence-gated pin rule is unchanged in substance",
        "validation-canary.md stays cited"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/hook-runtime/codex-capability-probe-version-pin-and-re-probe-evidence.md", "substantive": "Pointers section names .bee/doctor-attest.json and the attest verb; no PROBED_CODEX_VERSION, no canary_codex.mjs"}
      ],
      "key_links": [
        "the page still reads as the home of the evidence-gated pin rule that the Pi audit page cites"
      ],
      "prohibitions": [
        "No change to the rule the page owns",
        "No new claim about Pi — that belongs to the sibling page"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": false}
  },
  {
    "id": "thf-3",
    "feature": "three-findings",
    "title": "Publish the two herding flags in the command registry",
    "lane": "standard",
    "role": "code",
    "status": "open",
    "deps": [],
    "decisions": [],
    "files": [
      "packages/bee-rs/crates/bee/src/generated/registry_payload.json",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "read_first": [
      "docs/history/three-findings/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/catalog.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D3. `bee herding run` parses --seat and --inbox-session (herding/run.rs:369-373) and bee dispatch prepare emits both for runtime pi, but neither appears in the command registry, so neither shows in `bee herding run --help`. Add both to the herding.run entry's parameters.properties in packages/bee-rs/crates/bee/src/generated/registry_payload.json, as string flags, each with a description written from the parser and its call sites — --seat names the seat this detached run belongs to and rides the result envelope and the result-inbox marker; --inbox-session names the orchestrator session token that makes the run detached, writing the .bee/result-inbox marker the Pi drain reads. Match the surrounding entries' wording style. This payload is checked in and no bee dev verb regenerates it, so edit it directly and keep the file's existing single-line JSON formatting. Both names are new spellings nowhere else in the registry, so the distinct flag-name count moves 208 to 210: bump PINNED_FLAG_COUNT at catalog.rs:788 and add one ledger comment in the same style as the entries above it, recording the +2 and stating that the check for an existing flag meaning the same thing was made and found none.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "`bee herding run --help` lists both flags with their descriptions",
        "The pinned flag-count test passes at 210",
        "The ledger comment records the +2 and the reuse check"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/generated/registry_payload.json", "substantive": "herding.run carries seat and inbox-session; file stays valid JSON in its existing format"},
        {"path": "packages/bee-rs/crates/bee/src/catalog.rs", "substantive": "PINNED_FLAG_COUNT is 210 with a ledger comment"}
      ],
      "key_links": [
        "the descriptions match what herding/run.rs actually does with each flag"
      ],
      "prohibitions": [
        "No new flag beyond these two",
        "No rename of any existing flag",
        "No change to herding/run.rs parsing"
      ]
    },
    "trace": {"worker": null, "outcome": null, "files_changed": [], "deviations": [], "friction": null, "capped_at": null, "behavior_change": true}
  }
]
```

## Test matrix

| Scenario | Pass when |
|---|---|
| tf-1 happy path — any non-cell dispatch | rendered body byte-identical to before the deletion |
| tf-1 edge — both on-disk prompt copies | `prompt_skew` returns `None` for all four names |
| tf-1 regression — the advisor golden | `ADVISOR_BODY_WITHOUT_A_BRIEF` still matches, unchanged |
| tf-2 — knowledge bundle | `bee knowledge check` errors 0, profile_errors 0; index drift false |
| tf-2 — every named path | exists at this commit |
| tf-3 happy path | `bee herding run --help` prints both flags |
| tf-3 pinned count | the catalog test passes at 210, and fails if either flag is dropped |
| tf-3 error path | the payload still parses — a malformed edit breaks every command, not just this one |

## Open Questions

(none)

## Out of scope

Wiring a `paths` supplier; the stale "generated by `bee dev`" comment in
`registry.rs`.
