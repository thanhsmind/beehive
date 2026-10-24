---
artifact_contract: bee-plan/v2
mode: high-risk
---

# Plan: design-os in bee — Slice 4 (installers)

## Summary

Slices 1–3 merged into main at `7c03297`. A check of the installers found that
a fresh install still cannot deliver design-os: neither installer can pass a
pack pick, the installer hides onboarding's offer to pick, and the plugin-first
install skips every skill copy, so its projects never get a design pack.

This slice closes those three gaps. `install.sh` and `install.ps1` gain
`--design-packs`, the installers print onboarding's notices after a successful
install, and plugin-first onboarding still copies the picked design packs (and
only those) into the project. After the merge, main is pushed so the clone the
installer makes carries design-os.

Mode: `high-risk` — 4 flags: external-systems, public-contracts, multi-domain,
covered-contract-change.
Why this is the least workflow that protects the work: both installers write into
every new host, on two operating systems.

## Requirements

- **D4** (`572bc11e`) — the user picks packs at install; a pack not picked is
  never written into the host.
- **Owner decision** `1dd050b7` (2026-10-24) — installers carry the pick end to
  end: `--design-packs` on both installers, notices shown after apply,
  plugin-first still installs the picked packs, then main is pushed.

## Load-bearing claims

Labels: `read` = the file was opened at the named line; `ran` = the command was
executed and its output kept; `guessed` = neither, and no such row may survive
the gate.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | Plugin-first onboarding turns off every skill copy | read | `packages/bee-rs/crates/bee/src/onboard/mod.rs:419` | `sync_skills: !args.plugin_source,` |
| 2 | With the copy off, the plan holds no skill targets at all, design packs included | read | `packages/bee-rs/crates/bee/src/onboard/plan.rs:1242` | `let skill_sync = if opts.sync_skills {` |
| 3 | install.sh collects the flags it forwards to onboard in one array | read | `scripts/install.sh:474` | `ONBOARD_FLAGS+=("--no-statusline")` |
| 4 | install.sh captures the apply output and prints it only on failure | read | `scripts/install.sh:743` | `printf '%s\n' "$APPLY_OUTPUT" >&2` |
| 5 | install.ps1 collects its onboard flags in one array | read | `scripts/install.ps1:545` | `if ($NoStatusline) { $onboardFlags += '--no-statusline' }` |
| 6 | An installer contract test drives both installers end to end | read | `packages/bee-rs/crates/bee/tests/installer_contracts.rs:371` | `fn both_installers_onboard_from_the_source_and_vendor_before_applying() {` |
| 7 | local main is ahead of the remote the installer clones | ran | `git log origin/main --oneline -1` | `69570bb2b Release 2.59.0` |

## Discovery

The leader read both installers and the onboard switch (claims 1–6) after the
owner asked whether `install.sh` installs design-os fully. Claim 7: the
installer clones `main` from the remote, which still stands at the 2.59.0
release.

## Approach

**Recommended path.**

- **Plugin-first copies picked packs (dob-7).** When the skill copy is off
  (plugin-first), onboarding still plans and applies the design source only:
  `bee-design` plus each picked `bee-design-<pack>` into the repo skill homes,
  and prunes un-picked `bee-design*` dirs. No other skill is copied or removed in
  plugin-first. The router's onboard usage line names `--design-packs`.
- **Installers pass the pick and show notices (dob-8).** `install.sh` gains
  `--design-packs <list|none>` appended to `ONBOARD_FLAGS` (claim 3);
  `install.ps1` gains `-DesignPacks <list|none>` appended to `$onboardFlags`
  (claim 5). After a successful apply, each installer prints onboard's notices
  (the design offer among them) instead of dropping them (claim 4). README and
  INSTALL document the flag.
- **Push (leader, after merge).** Push main so the installer's clone carries
  design-os (claim 7). No release tag in this slice.

**Rejected alternatives.**

- Rendering design packs into the plugin skill trees for plugin-first users —
  every plugin host would carry every pack, breaking D4.
- A separate design installer — a second install door.

**SMALLER PATH check.** Each of the three gaps blocks a fresh install on its own;
dropping any one leaves design-os unreachable for some install path. PASS.

**Risk map.**

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| Plugin-first skill gate | HIGH — a leak would copy every bee skill into plugin hosts | dob-7 | a plugin-first apply with a pick writes only `bee-design*` dirs; with no pick writes none |
| Installer flag parsing | MEDIUM — two shells, two OSes | dob-8 | installer contract test passes the flag through on install.sh; install.ps1 parsed by its existing contract path |
| Notice printing | LOW | dob-8 | a successful install prints the design offer for a host with UI files |

Waves: dob-7 and dob-8 run in parallel — disjoint files (onboard Rust vs
installer scripts and their contract test).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "30ff876890293b1cea96673b722ea95a7b27780259af61e6c3b2be09a0c2b112",
  "stages": [
    {"stage": "plan-hat-wave", "classification": "not-applicable", "role": "advisor", "reason": "The wave is once per feature and ran at slice 1 (docs/history/design-os-in-bee/reports/hat-wave.md). Re-stamped, not re-run."},
    {"stage": "hat-facts-gaps", "classification": "not-applicable", "role": "hat-facts-gaps", "reason": "See plan-hat-wave."},
    {"stage": "hat-risks", "classification": "not-applicable", "role": "hat-risks", "reason": "See plan-hat-wave."},
    {"stage": "hat-value", "classification": "not-applicable", "role": "hat-value", "reason": "See plan-hat-wave."},
    {"stage": "hat-alternatives", "classification": "not-applicable", "role": "hat-alternatives", "reason": "See plan-hat-wave."},
    {"stage": "hat-user-impact", "classification": "not-applicable", "role": "hat-user-impact", "reason": "See plan-hat-wave."},
    {"stage": "research", "classification": "not-applicable", "role": "read", "reason": "Every claim was read or run by the leader."},
    {"stage": "extraction", "classification": "conditional", "role": "extraction", "condition": "A narrow fact is needed from a known file during execution.", "reason": "Cheap narrow lookups."},
    {"stage": "gather", "classification": "conditional", "role": "read", "condition": "A multi-file hunt is needed that the leader does not hold.", "reason": "Mechanical reads delegate down-tier."},
    {"stage": "cell-execution", "classification": "required", "role": "test", "reason": "Both cells are red-first changes; the code role's herding worker went idle twice on this feature (decision db4d5464)."},
    {"stage": "code", "classification": "not-applicable", "role": "code", "reason": "See cell-execution."},
    {"stage": "docs", "classification": "not-applicable", "role": "docs", "reason": "README and INSTALL lines ride dob-8."},
    {"stage": "review", "classification": "conditional", "role": "review", "condition": "The slice judge runs.", "reason": "Goal-check judge for behavior_change cells."},
    {"stage": "plan", "classification": "not-applicable", "role": "plan", "reason": "This is the last slice."},
    {"stage": "supervisor", "classification": "not-applicable", "role": "supervisor", "reason": "Attended session."},
    {"stage": "deploy", "classification": "not-applicable", "role": "deploy", "reason": "A plain push of main, no release."},
    {"stage": "generation", "classification": "not-applicable", "role": "generation", "reason": "Every writing stage resolves to a named role above."},
    {"stage": "blind-lane-1", "classification": "not-applicable", "role": "lane-1", "reason": "No open design shape."},
    {"stage": "blind-lane-2", "classification": "not-applicable", "role": "lane-2", "reason": "See blind-lane-1."},
    {"stage": "blind-lane-3", "classification": "not-applicable", "role": "lane-3", "reason": "See blind-lane-1."}
  ]
}
```

## Shape

| Epic | Capability / risk area | Why it exists | Slices | Proof needed |
|---|---|---|---|---|
| Taste, role, pick, gate | Content, dispatch, onboarding, cap | D1–D6 | 1–3 — **merged** `7c03297` | done |
| A fresh install delivers the pick | Installers, plugin-first | D4, `1dd050b7` | 4 (this plan) | installer contract tests; plugin-first onboard tests |

**Current slice to prepare: slice 4 (last).**

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| dob-7 | Install picked design packs on plugin-first onboarding too | onboard mod, plan, skills, tests, router | — | a plugin-first install with `--design-packs web` puts `bee-design` and `bee-design-web` in the project and no other skill | the declared test command, red first |
| dob-8 | Pass the design pick through both installers and show onboarding's notices | install.sh, install.ps1, installer contract test, README, INSTALL, manifest | — | `install.sh --design-packs web` installs the web pack; a host with UI files sees the design offer at the end of install | release-manifest check, then the declared test command, red first |

```json
[
  {
    "id": "dob-7",
    "feature": "design-os-in-bee",
    "title": "Install picked design packs on plugin-first onboarding too",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["572bc11e-0249-4ef8-97ef-a2dc996d4a9b", "1dd050b7-ac07-4227-a39c-7a8d807fd283", "053a14a4-7bac-4462-9b0b-733b80481aa8"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "packages/bee-rs/crates/bee/src/router.rs"
    ],
    "read_first": [
      "docs/history/design-os-in-bee/plan.md",
      "docs/knowledge/areas/onboarding/onboarding-design-packs-the-design-role-and-the-design-gate.md",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/skills.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST: write the failing tests in onboard/tests.rs before the code and watch each fail for its own reason.\n\nPlugin-first onboarding sets `sync_skills: !args.plugin_source,` (mod.rs:419), and plan.rs `let skill_sync = if opts.sync_skills {` (:1242) then plans no skill targets at all, so a plugin-first host never receives its picked design packs (owner decision 1dd050b7). Make plugin-first onboarding still plan and apply the DESIGN source only: when sync_skills is false and a pick exists, compute the repo skill-home targets for `bee-design` plus each picked `bee-design-<pack>` from the design root (the same SkillSources pick logic dob-3 added), and prune un-picked `bee-design*` dirs; in plugin-first no other skill is copied, refreshed or removed. With no pick (absent or \"none\"), plugin-first writes and removes nothing design-related except pruning `bee-design*` dirs when the pick is \"none\". Also add `--design-packs <list|none>` to the onboard usage line in router.rs (around the onboard entry, ~:99-101).\n\nTests: plugin-first + --design-packs web --apply writes exactly bee-design and bee-design-web into the repo skill homes and no bee-hive or other bee- skill; plugin-first with no pick writes no skill dir; plugin-first switching web to none removes the design dirs; repo-copy behavior is unchanged (existing dob-3 tests stay green).",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "each new test fails first for its own reason",
        "a plugin-first apply with a pick writes exactly bee-design and the picked packs and no other skill",
        "a plugin-first apply with no pick writes no skill dir",
        "repo-copy onboarding behaves as before"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "design-only skill targets when the general skill copy is off"}
      ],
      "key_links": [
        "plugin-first design targets reuse the dob-3 pick logic"
      ],
      "prohibitions": [
        "Do not copy, refresh or remove any non-design skill in plugin-first",
        "Do not render design packs into the plugin skill trees",
        "Do not touch the installer scripts in this cell"
      ]
    },
    "behavior_change": true
  },
  {
    "id": "dob-8",
    "feature": "design-os-in-bee",
    "title": "Pass the design pick through both installers and show onboarding's notices",
    "lane": "high-risk",
    "role": "test",
    "deps": [],
    "decisions": ["572bc11e-0249-4ef8-97ef-a2dc996d4a9b", "1dd050b7-ac07-4227-a39c-7a8d807fd283", "053a14a4-7bac-4462-9b0b-733b80481aa8"],
    "files": [
      "scripts/install.sh",
      "scripts/install.ps1",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs",
      "README.md",
      "INSTALL.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/design-os-in-bee/plan.md",
      "scripts/install.sh",
      "scripts/install.ps1",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "RED FIRST: extend packages/bee-rs/crates/bee/tests/installer_contracts.rs (model: `fn both_installers_onboard_from_the_source_and_vendor_before_applying() {` at :371) before changing the scripts, and watch each new assertion fail for its own reason.\n\n(1) install.sh: add `--design-packs <list|none>` to argument parsing and the usage text; when given, append `--design-packs <value>` to ONBOARD_FLAGS beside `ONBOARD_FLAGS+=(\"--no-statusline\")` (:474) so the plan, apply and recheck calls all carry it. (2) install.ps1: add a `-DesignPacks <list|none>` parameter and append `--design-packs <value>` to `$onboardFlags` beside `if ($NoStatusline) { $onboardFlags += '--no-statusline' }` (:545). (3) Notices: after a successful onboard apply, each installer prints onboard's notices (the plan JSON's notices array, or the apply output's `notice:` lines) to the user, so the design offer and other offers are seen; today install.sh keeps `$APPLY_OUTPUT` and prints it only on failure (`printf '%s\\n' \"$APPLY_OUTPUT\" >&2` at :743). (4) README.md and INSTALL.md: document the flag in the installer options where the other onboard pass-through flags are listed. (5) scripts/install.sh and install.ps1 are release inventory roots: rewrite the release manifest with `bee dev release-manifest --write`. No comments added to the scripts beyond what already exists (repo rule: no new comments).\n\nTests: install.sh with --design-packs web onto a sandbox target forwards the flag (the target's .bee/config.json records design.packs [\"web\"] and its skill homes hold bee-design and bee-design-web); a target with an index.html and no pick sees the design offer text in the installer output; install.ps1 carries -DesignPacks into its onboard flags (string-level contract check if pwsh is absent, as the existing ps1 contracts do).",
    "verify": ".bee/bin/bee dev release-manifest --check && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --no-fail-fast --manifest-path packages/bee-rs/Cargo.toml",
    "must_haves": {
      "truths": [
        "each new assertion fails first for its own reason",
        "install.sh --design-packs web onto a sandbox target installs bee-design and bee-design-web and records the pick",
        "a target with UI files and no pick shows the design offer in installer output",
        "install.ps1 forwards -DesignPacks to onboard",
        "README and INSTALL document the flag"
      ],
      "artifacts": [
        {"path": "scripts/install.sh", "substantive": "--design-packs parsing, forwarding, and notice printing after apply"},
        {"path": "scripts/install.ps1", "substantive": "-DesignPacks parameter forwarded to onboard, notices printed"}
      ],
      "key_links": [
        "the flag reaches every onboard call the installer makes (plan, apply, recheck)"
      ],
      "prohibitions": [
        "Do not change onboard Rust code in this cell",
        "Do not add new comments to the scripts"
      ]
    },
    "behavior_change": true
  }
]
```

## Verify

dob-7: the declared `commands.test`. dob-8: `bee dev release-manifest --check`,
then the declared `commands.test` (the installers are inventory roots).

## Test matrix

| Case | Kind | Pass when |
|---|---|---|
| Plugin-first with pick | happy | only `bee-design*` dirs written |
| Plugin-first without pick | edge | no skill dir written |
| Repo-copy unchanged | regression | dob-3 tests green |
| install.sh flag | happy | target records `design.packs` and holds the web pack |
| Offer shown | happy | installer output carries the design offer |
| install.ps1 flag | edge | `-DesignPacks` forwarded |

## Open Questions

- Upstream `THIRD_PARTY_LICENSES.md` says MIT while `LICENSE` is Shield.

## Out of scope

- A release tag; this slice ends with a push of main.
