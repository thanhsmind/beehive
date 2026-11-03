# host-packaging-gaps — plan

Route: class `feature` · lane `standard` · flags `public-contracts`,
`cross-platform`, `multi-domain` · product files 12.
Class playbook: `bee-planning/references/planning-reference.md`
("Class playbooks" → feature).

Revision 2 — rewritten after the plan-step hat wave and the user's
added ask to fix the herdr gap (D5). What changed is under `## Hat wave`.

## Summary

bee installs on another project, but five things stop it from working
fully there. After this change:

- `bee doctor --runtime pi` reports READY on a normal project, also in
  a plain terminal with no herdr or tmux, when every Pi role runs a
  plain Pi process.
- A new project can start Pi workers at once. Onboarding writes a Pi
  role table that runs the plain `pi` binary with the user's own Pi
  model. An older project gets the same table on its next onboard run,
  once, only where it is missing, and onboard says so.
- `--runtime pi` is accepted by `bee onboard`, `install.sh` and
  `install.ps1`.
- Each release also ships macOS (Apple silicon and Intel) and ARM Linux
  binaries. The installer runs a downloaded binary once before it trusts
  it, and builds from source if that run fails.

Mode: `standard` — 3 risk flags: public-contracts, cross-platform,
multi-domain.
Why this is the least workflow that protects the work: four cells split
by file so all four run at once, each with its own proof, and one live
install run over all of them at the end.

## Requirements (from CONTEXT.md)

Store ids: D1 `cd9b1a60`, D2 `298b1edb`, D3 `331bc33e`, D4 `320fcad4`,
D5 `974a2285`. They extend, and do not replace, `d2fd8c63` (prebuilt
binaries preferred by both installers) and `4a6e38be` (Pi routing lives
in the existing config homes; `team.pi` is that home).

- D1: Pi freshness in a host compares the binary with
  `.bee/onboarding.json` `bee_version`; the row stays mandatory and
  fail-closed; the source-checkout path does not change.
- D2: onboarding writes `team.pi` plus a plain `pi` herding agent; an
  existing config gains them only where absent; no existing value moves.
- D3: `pi` is a valid `--runtime` in `bee onboard`, `install.sh`,
  `install.ps1`, and the docs.
- D4: the release adds `aarch64-apple-darwin`, `x86_64-apple-darwin`,
  `aarch64-unknown-linux-gnu` on native runners; `install.sh` maps them
  and checks sums on stock macOS; `release.sh` expects the new count.
- D5: the Pi `herding_transport` row is ok when every `team.pi` slot is
  a no-pane Pi agent; any pane-transport Pi slot, or no `team.pi`,
  keeps today's HERDR/TMUX check.

Two named narrowings, recorded here so they are not silent:

- D2: a config with neither a `team` nor a legacy `models` table gets
  no Pi table (the `{"host_shell":"posix"}` fixture at
  `onboard/tests.rs` is that shape; adding a whole role table where the
  user has none is a larger change than D2 asks for). Such a host
  still gets the refusal's own FIX text.
- D2: the Pi table is offered once. A user who deletes it afterwards
  is not overridden on the next onboard.

## Load-bearing claims

Labels are `read` (file opened at that line) or `ran` (command run,
output held). Evidence is a verbatim substring of the anchored line.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The Pi freshness row is forced on even in a host repo | read | packages/bee-rs/crates/bee/src/doctor.rs:488 | `if !is_source_checkout && !is_pi {` |
| 2 | The Pi row list always pushes the freshness row | read | packages/bee-rs/crates/bee/src/doctor.rs:331 | `rows.push(pi_binary_freshness_row(root));` |
| 3 | A fresh host reports Pi BLOCKED on this row | ran | `bee doctor --runtime pi` in a repo installed by scripts/install.sh v2.43.0 | `.claude-plugin/plugin.json is missing or unreadable — cannot determine source release version` |
| 4 | An existing test pins "host with no manifest is unknown", and D1 keeps it green | read | packages/bee-rs/crates/bee/src/doctor/tests.rs:807 | `assert_eq!(freshness_no_plugin.1, None, "missing plugin manifest must report unknown");` |
| 5 | The host remedy text today tells the user to run cargo | read | packages/bee-rs/crates/bee/src/doctor.rs:483 | `const REMEDY: &str = "FIX: cargo build --release --manifest-path packages/bee-rs/Cargo.toml \` |
| 6 | The Pi row list always pushes the transport row | read | packages/bee-rs/crates/bee/src/doctor.rs:332 | `rows.push(pi_herding_transport_row_with_env(root, env));` |
| 7 | A Pi agent on runtime pi dispatches with no pane | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:2511 | `if runtime == "pi" && agent.as_deref().is_some_and(` |
| 8 | The no-pane runner needs no multiplexer | read | packages/bee-rs/crates/bee/src/herding/run.rs:4140 | `// no pane multiplexer is needed.` |
| 9 | Onboarding never touches an existing config today | read | packages/bee-rs/crates/bee/src/onboard/plan.rs:653 | `// 2. runtime files (create-if-missing only)` |
| 10 | The default config has a codex table but no pi table | read | packages/bee-rs/crates/bee/src/onboard/templates.rs:204 | `"codex": {` |
| 11 | A fresh host refuses every Pi dispatch | ran | `bee dispatch prepare --runtime pi --kind gather --role extraction --purpose test --json` in the fresh host | `"reason": "pi_requires_herding",` |
| 12 | A Pi agent is detected by its first argv token only | read | packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs:240 | `pub(crate) fn is_pi_agent(cfg: &Value, agent_name: &str) -> bool {` |
| 13 | The native Pi spawn rebuilds argv, so a bare `["pi"]` agent runs with Pi's default model | read | packages/bee-rs/crates/bee/src/herding/run.rs:2775 | `pub(crate) fn build_child_argv(agent_args: &[String], task: &str) -> Vec<String> {` |
| 14 | The dispatcher reads the deep-merged config pair, so the absence check must too | read | packages/bee-rs/crates/bee/src/state.rs:161 | `pub fn read_config_raw(root: &Path) -> Map<String, Value> {` |
| 15 | The team/models fold already exists to reuse | read | packages/bee-rs/crates/bee/src/verbs/drivers/models.rs:284 | `pub(crate) fn fold_team_key(map: &mut Map<String, Value>) -> bool {` |
| 16 | `bee onboard --runtime` accepts only three values | read | packages/bee-rs/crates/bee/src/onboard/mod.rs:217 | `if !["claude", "codex", "both"].contains(&args.runtime.as_str()) {` |
| 17 | install.sh rejects pi | read | scripts/install.sh:162 | `fail "--runtime must be claude, codex, or both" ;; esac` |
| 18 | install.ps1 rejects pi | read | scripts/install.ps1:37 | `[ValidateSet('claude', 'codex', 'both')]` |
| 19 | The plugin-distribution helper has no pi value | read | packages/bee-rs/crates/bee/src/devtools/plugin_distribution.rs:1060 | `Some(v @ ("claude"` |
| 20 | install.sh calls that helper in both modes | read | scripts/install.sh:698 | `"$BEE_BIN" dev plugin-distribution "${DIST_ARGS[@]}"` |
| 21 | Editing an installer makes the release manifest stale, so the cell must list and regen it | read | packages/bee-rs/crates/bee/src/devtools/release_manifest.rs:93 | `pub(crate) const MANIFEST_REL: &str = "docs/history/codex-harness-hardening/release-manifest.json";` |
| 22 | The release matrix has only two targets | read | .github/workflows/release-binaries.yml:45 | `target: x86_64-unknown-linux-gnu` |
| 23 | The version check runs node | read | .github/workflows/release-binaries.yml:83 | `got="$(node -e 'process.stdout.write(require("./.claude-plugin/plugin.json").version)')"` |
| 24 | A sed read of the manifest version already exists in the repo to reuse | read | scripts/install.sh:387 | `BEE_VERSION="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' \` |
| 25 | install.sh maps only Linux x86_64 and Windows | read | scripts/install.sh:193 | `Linux/x86_64)                          PREBUILT_ASSET="bee-x86_64-unknown-linux-gnu" ;;` |
| 26 | install.sh checks sums with `sha256sum`, absent on stock macOS | read | scripts/install.sh:282 | `sha256sum -c want.txt >/dev/null 2>&1 ); then` |
| 27 | release.sh expects at least two binaries | read | scripts/release.sh:345 | `[ "$BIN_COUNT" -ge 2 ]` |
| 28 | One failed row does not cancel the others | read | .github/workflows/release-binaries.yml:41 | `fail-fast: false` |
| 29 | The crate has no C or TLS dependency that blocks macOS or ARM builds | read | packages/bee-rs/Cargo.toml:13 | `serde_json = { version = "1", features = ["preserve_order"] }` |
| 30 | The repo is public, so the free ARM Linux runners are available | ran | `gh repo view thanhsmind/beehive --json visibility -q .visibility` | `PUBLIC` |

Row 29 note: the full dependency list (workspace `Cargo.toml:9-20` plus
`windows-sys`/`libc` per target) was read by a gather; the anchored line
also shows `preserve_order`, which D2's merge relies on to keep the
user's key order.

## Discovery

A fresh-host run (`scripts/install.sh -d <new repo> -y`, bee 2.43.0)
installed every file, reported Claude READY and Codex DEGRADED (normal),
and Pi BLOCKED. `bee dispatch prepare --runtime pi` refused. Two gathers
mapped the onboarding config path, the runtime flag, and the release
pipeline. `macos-15-intel` is GitHub's x86_64 macOS label until August
2027 (runner-images issue 13045). This session runs inside herdr
(`HERDR_ENV=1`), so every live Pi proof runs with the HERDR and TMUX
variables removed, or it would hide the D5 gap.

## Approach

- Cell A (hpg-1, D1 + D5, doctor only). Freshness: with no source
  checkout, read `.bee/onboarding.json` `bee_version` first and the
  plugin manifest only as a fallback. Host rows name the installer as
  the remedy, never cargo. Transport: read the merged config; when
  `team.pi` exists and every slot resolves to a Pi agent (the same
  `is_pi_agent` test dispatch uses), the row is ok and says no pane
  multiplexer is needed. Chesterton's fence: both rows were made
  mandatory on purpose (`5c61c85ed`); they stay mandatory and
  fail-closed, only what they judge changes.
- Cell B (hpg-2, D2 + the onboard half of D3, onboard Rust only).
  `default_config()` gains `team.pi` and `herding.agents.pi`; one new
  plan item adds them once to an existing config; `--runtime pi` is
  accepted.
- Cell C (hpg-4, the CI half of D4). Three matrix rows on native
  runners; the version read moves from node to the repo's own sed
  pattern; release.sh expects 5.
- Cell D (hpg-3, the installer half of D3 and D4, plus docs). Both
  installers accept pi, skip the plugin-distribution helper for it, and
  refuse plugin-first with pi; install.sh maps the new platforms, checks
  sums with `shasum` when `sha256sum` is absent, and runs the binary
  once before trusting it.

Rejected: cross-compiling macOS Intel on the arm runner (D4 says native;
the run-the-binary check is the point); a notice instead of the config
merge (old hosts would keep refusing); a built-in Pi fallback inside
dispatch (D2 says onboarding writes the table); python3 for the version
read (not proven on the Windows runner, where a missing `python3` would
stop all assets).

Risk map:

| Component | Risk | Lands in | Proof |
|---|---|---|---|
| doctor host path and transport row | LOW | hpg-1 | doctor tests, red-first |
| config merge into a user file | MEDIUM | hpg-2 | onboard tests: absent → added once; present → byte-identical; local overlay honored |
| installer pi flow | MEDIUM | hpg-3 | `bash -n`, installer contracts, live install with no herdr |
| new CI runners | MEDIUM | hpg-4 | YAML parse now; the real proof is the next release's `release-binaries` run, named as not yet run. `publish` needs all 5 rows green, so one slow row delays every asset — the same all-green rule release.sh already holds |
| ARM glibc floor | LOW | hpg-3, hpg-4 | ARM built on `ubuntu-22.04-arm` (glibc 2.35); the installer smoke run falls back to a source build on an older glibc |

Waves: one wave, all four cells at once. No two cells share a file.
The live install run at feature close needs all four capped.

## Hat wave

Three seats (`hat-facts-gaps`, `hat-alternatives`, `hat-user-impact`)
checked revision 1. What changed:

- Re-cut by file (alternatives F1): the old hpg-3 → hpg-2/hpg-4 edge is
  gone; all four cells run in one wave.
- Release manifest (facts B1): hpg-3 lists and regens
  `docs/history/codex-harness-hardening/release-manifest.json`.
- Version read (facts B2): sed, not python3.
- D1 order (facts W1): onboarding.json first in a host.
- plugin-first + pi refused (facts W3); the exact helper line named
  (facts W4); missed docs added (facts W5); hpg-2 verify gains
  `pi_plugin_contracts` and uses `jsjson::stringify_pretty` (facts W6);
  CI risk named (facts W7).
- Plain-terminal BLOCKED (user-impact F1): became D5 on the user's word.
- Host remedy (user-impact F2): installer, never cargo or onboard.
- Config rewrite (user-impact F3): merged-config check, indent and line
  ending kept, a notice, offered once.
- No default model (user-impact F4): installer banner and usage text
  say Pi needs a default model; a doctor row for it is out of scope.
- ARM glibc (user-impact F5): smoke run after the checksum, ARM on
  `ubuntu-22.04-arm`.
- `models` + `team` both present (alternatives F2): add under `team`.
- config-sample (alternatives F3): read, not written.

## Smaller path check

Is there a cheaper shape that still honors D1-D5? A notice instead of
the config merge leaves old hosts refusing Pi dispatch, which D2 exists
to close. Cross-compiling breaks D4's native rule. Folding cells saves
no dispatch now that all four run at once. PASS.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| hpg-1 | Judge Pi doctor rows by what a host really has | `doctor.rs`, `doctor/tests.rs`, health-checks knowledge page | — | `bee doctor --runtime pi` on a host reads READY in a plain terminal when the binary matches its install record | doctor tests green, red-first |
| hpg-2 | Give onboarded hosts a Pi role table and accept --runtime pi | `onboard/templates.rs`, `plan.rs`, `apply.rs`, `mod.rs`, `notices.rs`, `tests.rs`, onboarding product doc | — | `bee dispatch prepare --runtime pi` resolves in a new host and in an old one after one re-onboard | onboard + pi contract tests green |
| hpg-4 | Build macOS and ARM Linux binaries in the release | `release-binaries.yml`, `scripts/release.sh` | — | the next release lists five `bee-*` binaries | YAML parse, `bash -n` |
| hpg-3 | Install on pi, macOS and ARM Linux | `scripts/install.sh`, `scripts/install.ps1`, `INSTALL.md`, `README.md`, release manifest | — | `install.sh --runtime pi` works; a Mac or ARM box downloads a checked, runnable binary | `bash -n`, installer contracts, manifest check |

```json
[
  {
    "id": "hpg-1",
    "feature": "host-packaging-gaps",
    "title": "Judge Pi doctor rows by what a host really has",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["cd9b1a60-26fc-4508-95cd-d32e30809f7a", "974a2285-cfc4-4dea-b838-184d1386465f"],
    "files": [
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"
    ],
    "read_first": [
      "docs/history/host-packaging-gaps/CONTEXT.md",
      "docs/history/host-packaging-gaps/plan.md",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md"],
    "action": "Two Pi doctor rows in packages/bee-rs/crates/bee/src/doctor.rs judge a host by facts it does not have. Fix both; leave every claude/codex row and the source-checkout path byte-identical. (A) Freshness, per D1 (store cd9b1a60). In `fn binary_freshness_row_impl(root: &Path, is_pi: bool) -> Option<Row> {` the host path (`if !is_source_checkout && !is_pi {` then `let Some(source_version) = read_source_release_version(root) else {`) reads only `.claude-plugin/plugin.json`, which a host never has, so it is unknown and the Pi verdict is BLOCKED. When `is_source_checkout` is false: read the expected version from `<root>/.bee/onboarding.json` field `bee_version` FIRST (a real host record: `{\"schema_version\": \"1.0\", \"bee_version\": \"2.43.0\", ...}`; `bee_version` may be null — treat null or absent as missing), and fall back to the plugin manifest only if onboarding.json gives nothing; neither → unknown, as today. Add the reader beside `fn read_source_release_version(root: &Path) -> Option<String> {`. In the host case every row that carries a remedy (the too-old row, the mismatch row, the probe-failed unknown row) names the installer as the fix, never cargo and never `bee onboard --apply` (onboard does not replace .bee/bin/bee): e.g. `FIX: re-run the bee installer in this repo: curl -fsSL https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.sh | bash -s -- -y`. The mismatch detail names `.bee/onboarding.json` as the source in the host case. `const REMEDY: &str = \"FIX: cargo build --release ...` stays for the source checkout. (B) Transport, per D5 (store 974a2285). `rows.push(pi_herding_transport_row_with_env(root, env));` always demands HERDR or TMUX, yet a Pi agent on runtime pi dispatches with `--no-pane` (prepare.rs: `if runtime == \"pi\" && agent.as_deref().is_some_and(|name| is_pi_agent(&cfg, name)) {`) and that runner needs no multiplexer (herding/run.rs: `// no pane multiplexer is needed.`). In `fn pi_herding_transport_row_with_env(root: &Path, env: &dyn Fn(&str) -> Option<String>) -> Row {`, first read the merged config (`crate::state::read_config_raw(root)`, the same deep-merged view the dispatcher reads; fold team/models with `crate::verbs::drivers::fold_team_key` on a clone). If `team.pi` is an object with at least one slot and EVERY slot is `{\"kind\":\"herding\",\"agent\":<name>}` where `crate::verbs::drivers::prepare::is_pi_agent(&cfg, name)` (make it reachable if its visibility needs a nudge; do not copy its logic) is true, return ok with a detail like `every team.pi role runs a Pi process with --no-pane; no pane multiplexer is needed`. Otherwise (no team.pi, an empty one, or any slot that is not a no-pane Pi agent) keep today's probe unchanged. RED FIRST in doctor/tests.rs, using the `fn pi_repo(` fixture (its last parameter is the config text) and a written `.bee/onboarding.json`: (1) host, no plugin manifest, onboarding.json bee_version equal to the binary's version → freshness ok; (2) host, different bee_version → not_ok, detail names `.bee/onboarding.json` and the installer, not cargo; (3) host with both files disagreeing → onboarding.json wins; (4) config whose team.pi slots all use an agent `[\"pi\"]` and env with no HERDR/TMUX vars → transport ok and all six rows ok; (5) same but one slot uses a non-pi agent → transport not_ok with today's `HERDR_ENV is not set` text. Watch each fail for the reported reason, then fix. `pi_doctor_ready_case`, `pi_doctor_reports_missing_pane` (config None) and the assertion `\"missing plugin manifest must report unknown\"` must stay green unchanged. Then update docs/knowledge/areas/hook-runtime/health-checks-and-proof-surfaces.md: the Pi bullet's sentence 'Binary freshness checks the release version against `.claude-plugin/plugin.json` in both source and host repositories' becomes the host rule above, and its transport sentence gains the no-pane rule.",
    "must_haves": {
      "truths": [
        "a Pi host whose binary matches .bee/onboarding.json bee_version reports binary_freshness ok",
        "a Pi host whose binary differs reports not_ok naming .bee/onboarding.json and the installer",
        "a Pi host with neither file still reports unknown",
        "no host-path remedy mentions cargo",
        "a config whose every team.pi slot is a no-pane Pi agent reports herding_transport ok with no HERDR or TMUX variables",
        "a config with any non-Pi team.pi slot, or no team.pi, keeps the HERDR/TMUX check"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "onboarding.json reader for the host freshness path, host remedy text, no-pane branch in the Pi transport row"},
        {"path": "packages/bee-rs/crates/bee/src/doctor/tests.rs", "substantive": "five new Pi tests: freshness match, mismatch, precedence; transport all-pi, mixed"}
      ],
      "key_links": ["the transport row uses prepare.rs is_pi_agent, the same test dispatch uses to add --no-pane"],
      "prohibitions": [
        "No change to claude or codex doctor rows",
        "Do not drop either Pi row or make it optional",
        "Do not edit the existing no-manifest or missing-pane assertions"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee doctor"
  },
  {
    "id": "hpg-2",
    "feature": "host-packaging-gaps",
    "title": "Give onboarded hosts a Pi role table and accept --runtime pi",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["298b1edb-7519-4b13-8e9a-7b099df8915e", "331bc33e-0d7f-4c2f-9ab4-4889ee55cd1c"],
    "files": [
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/onboard/plan.rs",
      "packages/bee-rs/crates/bee/src/onboard/apply.rs",
      "packages/bee-rs/crates/bee/src/onboard/mod.rs",
      "packages/bee-rs/crates/bee/src/onboard/notices.rs",
      "packages/bee-rs/crates/bee/src/onboard/tests.rs",
      "docs/product-description/maintenance/onboarding.md"
    ],
    "read_first": [
      "docs/history/host-packaging-gaps/CONTEXT.md",
      "docs/history/host-packaging-gaps/plan.md",
      "packages/bee-rs/crates/bee/src/onboard/templates.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/prepare.rs",
      "packages/bee-rs/crates/bee/src/verbs/drivers/models.rs",
      ".bee/config-sample.json"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/product-description/maintenance/onboarding.md"],
    "action": "Per D2 (store 298b1edb) and the onboard half of D3 (store 331bc33e). (1) Default table. In packages/bee-rs/crates/bee/src/onboard/templates.rs `pub fn default_config() -> Value {`, add a `\"pi\"` table in `\"team\"` right after the `\"codex\": {` table: the SAME 18 role names as `team.claude`, same order, each `{ \"kind\": \"herding\", \"agent\": \"pi\", \"description\": <that claude role's description> }`. Add `\"pi\": [\"pi\"]` to `herding.agents` after `claude-haiku`. Do not change `agent_command` or `control_command`. Why a bare `[\"pi\"]`: prepare.rs `pub(crate) fn is_pi_agent(cfg: &Value, agent_name: &str) -> bool {` detects Pi by the first argv token, and herding/run.rs `pub(crate) fn build_child_argv(agent_args: &[String], task: &str) -> Vec<String> {` rebuilds argv copying only --model/--thinking/--tools, so bare `pi` runs with the user's own default model; bee picks no provider or model. Extend the pin tests in templates.rs (`default_config_keeps_literal_order_and_nulls`, `default_config_publishes_the_full_role_table`, `every_claude_role_has_a_description`) to pin the pi table: names equal to claude's, every slot kind herding agent pi with a non-empty description, herding.agents.pi == [\"pi\"]. (2) Existing hosts. `// 2. runtime files (create-if-missing only)` in plan.rs never touches an existing .bee/config.json. Add ONE plan item `add_pi_team` (target `.bee/config.json`), planned only when ALL hold: config.json exists and parses as an object; the MERGED config (`crate::state::read_config_raw(root)`, which deep-merges `.bee/config.local.json`) has no `team.pi` (and no `models.pi`); the tracked file has a `team` object or a legacy `models` object (when both exist, `team` wins, as `pub(crate) fn fold_team_key(map: &mut Map<String, Value>) -> bool {` does; when neither exists, plan nothing — a named narrowing in plan.md); and `.bee/onboarding.json` does not already record that the Pi table was offered. Apply it in apply.rs: insert the table (derive it from `default_config()`, never a second literal) under `team` (or `models`), and `herding.agents.pi` only if absent (create `herding`/`agents` objects only if absent). Never overwrite, reorder or delete an existing key. Write the file back keeping its detected indent width and line ending (CRLF stays CRLF) and a trailing newline; reuse `crate::jsjson::stringify_pretty` for the two-space case and re-indent only if the file used another width. Record the offer in `.bee/onboarding.json` (a field such as `pi_team_offered: true`, written with the rest of that record) so a later deletion is respected. Emit one onboard notice (notices.rs) naming the two added keys and the file. Unparseable config: plan nothing, write nothing. (3) Runtime value. packages/bee-rs/crates/bee/src/onboard/mod.rs `if ![\"claude\", \"codex\", \"both\"].contains(&args.runtime.as_str()) {` accepts `pi` too; the message becomes `--runtime must be claude, codex, pi, or both (got: {})`; update the module doc comment near the top of mod.rs that lists the values. `runtime_covers_codex` stays as is. Update the onboard test that pins `\"--runtime must be claude, codex, or both (got: rust)\"`. (4) docs/product-description/maintenance/onboarding.md quotes the old runtime message — update it, and add one line that onboarding writes a Pi role table and adds it once to an existing config. (5) .bee/config-sample.json already has a `\"pi\"` table; read it, do not write it. TESTS in onboard/tests.rs: fresh apply → team.pi with 18 herding slots and herding.agents.pi; existing config without team.pi → gains exactly those keys, every other key/value unchanged and in order, a 4-space file stays 4-space, a CRLF file stays CRLF, the notice appears; team.pi only in config.local.json → tracked file byte-identical; team.pi present → byte-identical; offered already recorded and team.pi deleted → byte-identical; unparseable → byte-identical, no error; a config with neither team nor models (the `{\"host_shell\":\"posix\"}` fixture) → no add_pi_team item; second run → no item; `--runtime pi` parses and a plan succeeds.",
    "must_haves": {
      "truths": [
        "a fresh onboard writes team.pi with the 18 claude role names, each kind herding agent pi, and herding.agents.pi = [\"pi\"]",
        "an existing config without team.pi (in the merged view) gains team.pi and herding.agents.pi once, keeping every other key, its order, indent width and line ending",
        "onboard prints a notice naming the added keys",
        "a deleted Pi table is not re-added after it was offered once",
        "bee onboard --runtime pi plans and applies with no error"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/onboard/templates.rs", "substantive": "team.pi and herding.agents.pi in default_config, pin tests extended"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/plan.rs", "substantive": "add_pi_team plan item with the merged-config, team/models and offered-once conditions"},
        {"path": "packages/bee-rs/crates/bee/src/onboard/apply.rs", "substantive": "add_pi_team apply arm that inserts absent keys only and keeps indent and line ending"}
      ],
      "key_links": ["apply.rs derives the pi table from default_config(), not a copied literal"],
      "prohibitions": [
        "No existing config value is changed, reordered or removed",
        "No provider or model is written into the default pi agent",
        "team.claude and team.codex defaults are unchanged",
        ".bee/config-sample.json is not edited"
      ]
    },
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee onboard && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test pi_plugin_contracts"
  },
  {
    "id": "hpg-4",
    "feature": "host-packaging-gaps",
    "title": "Build macOS and ARM Linux binaries in the release",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["320fcad4-af38-4ab4-a120-9509c30d2e86"],
    "files": [
      ".github/workflows/release-binaries.yml",
      "scripts/release.sh"
    ],
    "read_first": [
      "docs/history/host-packaging-gaps/CONTEXT.md",
      ".github/workflows/release-binaries.yml",
      "scripts/release.sh"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "Per D4 (store 320fcad4), the CI half. (1) .github/workflows/release-binaries.yml matrix (`target: x86_64-unknown-linux-gnu`, `target: x86_64-pc-windows-msvc`): add three rows in the same shape (os, target, built, asset): `macos-latest` / `aarch64-apple-darwin` / `bee` / `bee-aarch64-apple-darwin`; `macos-15-intel` / `x86_64-apple-darwin` / `bee` / `bee-x86_64-apple-darwin`; `ubuntu-22.04-arm` / `aarch64-unknown-linux-gnu` / `bee` / `bee-aarch64-unknown-linux-gnu` (22.04, not 24.04, so the binary needs glibc 2.35, not 2.39). Every row builds natively, so the 'Check the binary reports the tag's version' step still runs the built binary on all five; keep it for all. In that step replace `got=\"$(node -e 'process.stdout.write(require(\"./.claude-plugin/plugin.json\").version)')\"` with a sed read that needs no interpreter, the same pattern scripts/install.sh already uses: `sed -n 's/.*\"version\"[[:space:]]*:[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p' .claude-plugin/plugin.json | head -n 1` (it must work under Git Bash on windows-latest too). Leave the publish job (it already globs `bee-*` into SHA256SUMS and the upload) and `fail-fast: false` as they are. (2) scripts/release.sh `[ \"$BIN_COUNT\" -ge 2 ]`: expect >= 5 and update the message's number. Do not touch scripts/install.sh — another cell owns it.",
    "must_haves": {
      "truths": [
        "the release matrix builds five targets, each on a native runner, each version-checked by running the built binary",
        "the version check uses no node or python",
        "release.sh refuses a release with fewer than five bee-* binaries"
      ],
      "artifacts": [
        {"path": ".github/workflows/release-binaries.yml", "substantive": "five-row matrix, sed version read"},
        {"path": "scripts/release.sh", "substantive": "binary count of five"}
      ],
      "key_links": ["asset names are bee-aarch64-apple-darwin, bee-x86_64-apple-darwin, bee-aarch64-unknown-linux-gnu, exactly what scripts/install.sh maps to"],
      "prohibitions": [
        "No cross-compiled target that skips the run-the-binary check",
        "No change to the publish job's SHA256SUMS or upload logic",
        "No edit to scripts/install.sh"
      ]
    },
    "verify": "bash -n scripts/release.sh && python3 -c \"import yaml;d=yaml.safe_load(open('.github/workflows/release-binaries.yml'));print(len(d['jobs']['build']['strategy']['matrix']['include']))\""
  },
  {
    "id": "hpg-3",
    "feature": "host-packaging-gaps",
    "title": "Install on pi, macOS and ARM Linux",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["331bc33e-0d7f-4c2f-9ab4-4889ee55cd1c", "320fcad4-af38-4ab4-a120-9509c30d2e86"],
    "files": [
      "scripts/install.sh",
      "scripts/install.ps1",
      "INSTALL.md",
      "README.md",
      "docs/history/codex-harness-hardening/release-manifest.json"
    ],
    "read_first": [
      "docs/history/host-packaging-gaps/CONTEXT.md",
      "docs/history/host-packaging-gaps/plan.md",
      "scripts/install.sh",
      "scripts/install.ps1",
      "packages/bee-rs/crates/bee/src/devtools/plugin_distribution.rs",
      "packages/bee-rs/crates/bee/tests/installer_contracts.rs"
    ],
    "affects_skills": [],
    "affects_specs": [],
    "action": "The installer half of D3 (store 331bc33e) and D4 (store 320fcad4). Read scripts/install.sh fully first. (1) Runtime pi in install.sh. `case \"$RUNTIME\" in claude|codex|both) ;; *) fail \"--runtime must be claude, codex, or both\" ;; esac` accepts pi, message `--runtime must be claude, codex, pi, or both`; update the usage text at the `--runtime <which>` line. With RUNTIME=pi, `runtime_active` is false for claude and codex, so `probe_plugin_state` calls no CLI and `transition_plugin`/`rollback_plugin` skip — leave those. The one call that must not see pi is `\"$BEE_BIN\" dev plugin-distribution \"${DIST_ARGS[@]}\" || {` (and its plugin-first siblings with `--apply` and the recheck): plugin_distribution.rs `parse_runtime` has no pi value. Skip every plugin-distribution call when RUNTIME is pi, with one log line saying Pi has no plugin marketplace. Refuse `--distribution plugin-first` together with `--runtime pi` early, before any write, with a message that names repo-copy (plugin-first creates no repo skills or hooks, and Pi has no plugin). Pass `--runtime pi` to `bee onboard` as today's flag threading does. When RUNTIME is pi, the final banner names Pi and says Pi must have a default model set (bee's default Pi role table runs plain `pi`). (2) Same in scripts/install.ps1: `[ValidateSet('claude', 'codex', 'both')]` gains 'pi'; mirror the skips (the codex|both and claude|both CLI probes, the plugin loops, the plugin-distribution calls), the plugin-first refusal and the banner. Keep the file ASCII-only (a repo test guards non-ASCII bytes in scripts/*.ps1). (3) Platforms in install.sh. The case with `Linux/x86_64)                          PREBUILT_ASSET=\"bee-x86_64-unknown-linux-gnu\" ;;` gains `Linux/aarch64|Linux/arm64` → bee-aarch64-unknown-linux-gnu, `Darwin/arm64` → bee-aarch64-apple-darwin, `Darwin/x86_64` → bee-x86_64-apple-darwin. The checksum step (`sha256sum -c want.txt >/dev/null 2>&1 ); then`) uses `shasum -a 256 -c` when `sha256sum` is not on PATH; neither present → treat as a verify failure (source build, never an unverified binary). After the checksum passes, run the binary once (`\"$STATE_TMP_BIN/$PREBUILT_ASSET\" rs-info >/dev/null 2>&1`); if it fails (for example an old glibc on ARM Linux), log the reason and fall back to the source build. (4) Docs. INSTALL.md: the `--runtime` row names pi and says the Pi extension and skills are written for every runtime value and pi skips the Claude/Codex plugin steps; the platforms line (x86_64 Linux/Windows need nothing) adds macOS (Apple silicon and Intel) and ARM64 Linux (glibc 2.35+). README.md: the `bee onboard` synopsis `[--runtime claude|codex|both]` and the platform line name the same. (5) Release manifest: editing an installer makes `docs/history/codex-harness-hardening/release-manifest.json` stale. Run `.bee/bin/bee dev release-manifest --write` last and commit it with the rest. If `bee dev regen` is the documented chain in this repo, run it instead.",
    "must_haves": {
      "truths": [
        "install.sh --runtime pi reaches bee onboard --runtime pi and never calls plugin-distribution with pi",
        "install.sh and install.ps1 refuse plugin-first with pi before any write",
        "install.sh picks a prebuilt asset on Darwin arm64, Darwin x86_64 and Linux aarch64/arm64",
        "install.sh verifies the checksum with shasum when sha256sum is absent, and refuses an unverified binary",
        "install.sh falls back to a source build when the downloaded binary does not run",
        "install.ps1 accepts -Runtime pi and stays ASCII-only",
        "the release manifest matches the edited installers"
      ],
      "artifacts": [
        {"path": "scripts/install.sh", "substantive": "pi runtime, plugin-distribution skip, plugin-first refusal, platform map, shasum fallback, smoke run"},
        {"path": "scripts/install.ps1", "substantive": "pi runtime, skips and refusal"},
        {"path": "docs/history/codex-harness-hardening/release-manifest.json", "substantive": "regenerated installer hashes"}
      ],
      "key_links": ["install.sh asset names equal the release-binaries.yml asset names"],
      "prohibitions": [
        "No change to claude, codex or both behavior in either installer",
        "No new plugin marketplace for pi",
        "No non-ASCII byte in install.ps1",
        "No edit to .github/workflows/release-binaries.yml or scripts/release.sh"
      ]
    },
    "verify": "bash -n scripts/install.sh && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --test installer_contracts && .bee/bin/bee dev release-manifest --check"
  }
]
```

## Test matrix

| Dimension | Case | Pass when |
|---|---|---|
| happy | Pi host, onboarding.json version = binary version | `binary_freshness` ok |
| happy | all `team.pi` slots are `["pi"]`, no HERDR/TMUX | `herding_transport` ok, Pi doctor READY |
| happy | fresh onboard | config has `team.pi` (18 herding slots) and `herding.agents.pi` |
| happy | `install.sh --runtime pi` on a fresh repo, HERDR/TMUX unset | exit 0, Pi doctor READY |
| edge | Pi host with neither manifest nor onboarding.json | freshness unknown (unchanged) |
| edge | existing 4-space CRLF config without `team.pi` | gains two keys; indent, line ending, other keys and order unchanged |
| edge | `team.pi` only in `config.local.json` | tracked config byte-identical |
| edge | Pi table offered, then deleted | next onboard leaves it deleted |
| edge | macOS with `shasum` only | checksum verified, prebuilt used |
| error | Pi host, onboarding.json version differs | not_ok naming `.bee/onboarding.json` and the installer |
| error | one `team.pi` slot is not a Pi agent, no HERDR/TMUX | transport not_ok, `HERDR_ENV is not set` |
| error | `--distribution plugin-first --runtime pi` | refused before any write, names repo-copy |
| error | downloaded binary does not run | source build, reason logged |
| behavior-change | `bee doctor --runtime pi` in the same fresh host, HERDR/TMUX unset, main vs head | main: BLOCKED. head: READY |
| behavior-change | `bee dispatch prepare --runtime pi` in the same fresh host, main vs head | main: `pi_requires_herding`. head: a herding payload with `--no-pane` |

## Test scoping

Each cell records its scoped proof line. The feature closes on one live
run over the rebuilt binary, with HERDR and TMUX variables removed:
`scripts/install.sh --runtime pi` into a fresh repo, then Pi doctor and
a Pi dispatch prepare there (`green:live`). The new CI runners are
proven only by the next release's `release-binaries` run; that is named
at close as not yet run.

## Open questions

(none)

## Out of scope

- `bee doctor --runtime opencode`.
- A Windows ARM64 asset.
- A doctor row that checks Pi has a default model or auth.
- Changing existing Pi role values in any host.
