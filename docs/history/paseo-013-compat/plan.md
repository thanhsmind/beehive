---
artifact_contract: bee-plan/v2
mode: standard
---

# Plan: Paseo 0.13 compatibility

## Summary

bee keeps working with Paseo 0.12 and newer. Workers start with the flag
Paseo now documents (`--no-wait`) instead of the deprecated `-d`. Every
version check in bee asks for Paseo 0.12.0 or newer. Steering a running
worker goes through one plain `paseo send --behavior steer` call for every
provider, so the Node helper script and its npm package lookup go away.
The Paseo channel doc says the same.

Mode: `standard` — 3 risk flags: external systems (the Paseo CLI), a
covered contract change (unit tests pin the `-d` argv, the 0.10.3 floor and
the steer runner), proof weakening (the steer runner tests are replaced by
fake-CLI tests).
Why this is the least workflow that protects the work: every change is an
argv or a version number behind the `PaseoCli` seam, and the fake-CLI unit
tests plus the live `paseo run --no-wait` and `paseo send --behavior steer`
runs held in Discovery prove both.

Playbook: `skills/bee-planning/playbooks/feature.md` — cited, not copied.

## Requirements (from CONTEXT.md)

- D1 (store `5477c172`): `bee herding run` starts a Paseo worker with `paseo run --no-wait --json ...`; `-d` is never passed.
- D2 (store `4236f83d`): `bee doctor` requires Paseo CLI 0.12.0 or newer; FIX lines name 0.12.0.
- D3 (store `ad803822`): `bee herding steer` on a Paseo job runs `paseo send` with `--behavior steer --no-wait` for every provider through the `PaseoCli` seam; `paseo_steer.mjs`, `paseo_steer.rs` (package-dir lookup, `SteerRunner`, `steerable`) and the pi mailbox steer file on Paseo jobs are removed; non-Paseo jobs keep the mailbox `steer-N.json` file.
- D4 (store `dc38fecc`): a `paseo send` error yields `paseo_steer_failed` with FIX `check the Paseo daemon`; `paseo_steer_unsupported` stays only for a job whose agent has no paseo config.
- D5 (store `5ccde9fb`): the 0.12.0 floor holds at every check — doctor AND the pre-spawn checks in `herding/run.rs` with their refusal texts; one number in the runtime.
- D6 (store `11719b08`): the steer argv places the text after a lone `--` separator.
- D7 (store `87defaa3`): Paseo owns the steer capability check; bee keeps no provider allow-list; a steer to an idle worker opens a new turn.

## Load-bearing claims

Labels: `read` (file opened at that line), `ran` (command run, output held). Evidence is a verbatim substring.

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | The run argv passes `-d` in one builder | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:219 | "-d".to_string(), |
| 2 | Three unit tests pin `-d` in the run argv | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:1239 | "-d", |
| 3 | Paseo 0.13.2 deprecates `-d` and still answers | ran | paseo run -d --json --provider pi --title beeprobe --wait-timeout 1s "say ok" | Warning: --background (-d) is deprecated. Use --no-wait instead. |
| 4 | `paseo run --no-wait --json` prints the agentId JSON the spawn parser reads | ran | paseo run --no-wait --json --provider pi --title beeprobe2 "wait 60 seconds, then say ok" | "agentId": "7ff0821b-b00c-42fe-8718-acfab12f0559", |
| 5 | `paseo send` 0.13.2 takes `--behavior steer` | ran | paseo send --help | --behavior <behavior>  Active-turn behavior (agent default: steer; otherwise: |
| 6 | Steer text starting with a dash is read as an option | ran | paseo send 7ff0821b-b00c-42fe-8718-acfab12f0559 --behavior steer --no-wait --json "-x starts with a dash" | error: unknown option '-x starts with a dash' |
| 7 | A lone `--` before the text makes the daemon accept it | ran | paseo send 7ff0821b-b00c-42fe-8718-acfab12f0559 --behavior steer --no-wait --json -- "-y after double dash" | "message": "Message queued for delivery after two quiet seconds" |
| 8 | `send_argv` already carries `--no-wait` | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:504 | "--no-wait".to_string(), |
| 9 | Doctor's floor tuple | read | packages/bee-rs/crates/bee/src/doctor.rs:1070 | } else if !crate::herding::paseo::version_at_least(&out, (0, 10, 3)) { |
| 10 | Doctor's ok detail names the floor | read | packages/bee-rs/crates/bee/src/doctor.rs:1267 | detail: "paseo daemon is reachable, version >= 0.10.3, and heartbeats ready".to_string(), |
| 11 | Doctor tests pin the 0.10.3 FIX text | read | packages/bee-rs/crates/bee/src/doctor/tests.rs:1302 | old_version_row.detail.contains("upgrade to 0.10.3 with npm @getpaseo/cli"), |
| 12 | The doctor old-version fixture is 0.10.2 | read | packages/bee-rs/crates/bee/src/doctor/tests.rs:1296 | version_out: Ok("0.10.2".to_string()), |
| 13 | The run verb has its own pre-spawn floor | read | packages/bee-rs/crates/bee/src/herding/run.rs:3637 | Ok(out) => paseo::version_at_least(&out, (0, 10, 3)), |
| 14 | The run verb repeats that floor on continue | read | packages/bee-rs/crates/bee/src/herding/run.rs:3900 | Ok(out) => paseo::version_at_least(&out, (0, 10, 3)), |
| 15 | The pre-spawn refusal text names 0.10.3 | read | packages/bee-rs/crates/bee/src/herding/run.rs:3643 | "paseo version 0.10.3 or newer required — FIX: install or update paseo (npm @getpaseo/cli 0.10.3+)".to_string(), |
| 16 | The daemon-down FIX text names 0.10.3 | read | packages/bee-rs/crates/bee/src/herding/run.rs:3713 | "start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)".to_string() |
| 17 | run.rs tests assert the 0.10.3 text | read | packages/bee-rs/crates/bee/src/herding/run.rs:12526 | assert!(msg.contains("0.10.3")); |
| 18 | The steer verb enters through the Node runner | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:791 | steer_job_with(main_root, job_id, text, &crate::herding::paseo_steer::RealSteerRunner) |
| 19 | The provider allow-list gates the Paseo path | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:933 | if !crate::herding::paseo_steer::steerable(&spec.provider) { |
| 20 | The steer tests fake the runner seam | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:1608 | impl crate::herding::paseo_steer::SteerRunner for FakeSteerRunner { |
| 21 | The interrupt verb builds the real CLI lazily with 5 s | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:170 | real_cli = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(Duration::from_secs(5)); |
| 22 | job_verbs.rs tests already own a recording fake CLI | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:1795 | struct FakePaseoCli { |
| 23 | The helper module is declared in one place | read | packages/bee-rs/crates/bee/src/herding.rs:126 | pub(crate) mod paseo_steer; |
| 24 | The mailbox steer file is written in job_verbs only | read | packages/bee-rs/crates/bee/src/herding/job_verbs.rs:986 | let tmp_file = mbox.join(format!("steer-{next_n}.json.tmp")); |
| 25 | The channel doc states the `-d` argv | read | docs/knowledge/areas/bee-herding/the-paseo-channel.md:85 | The runner executes `paseo run -d --json` with the following arguments: |
| 26 | The channel doc describes the Node helper | read | docs/knowledge/areas/bee-herding/the-paseo-channel.md:349 | This daemon helper requires Node.js and the npm package `@getpaseo/cli`. |
| 27 | The channel doc states the 0.10.3 floor | read | docs/knowledge/areas/bee-herding/the-paseo-channel.md:281 | First, Paseo 0.10.3 is the minimum required version. |
| 28 | The channel doc says `paseo send` cannot steer | read | docs/knowledge/areas/bee-herding/the-paseo-channel.md:285 | Third, the CLI command `paseo send` has no steer flag. |
| 29 | The CLI bee runs here is the bundled 0.13.2 | ran | paseo --version | 0.13.2 |

## Discovery

The leader read the Paseo changelog (0.12.0: `--background` deprecated for
`--no-wait`, `permit` exits non-zero after 15 s; 0.13.0: plugin API and
hosts, nothing bee calls), probed the installed CLI live (claims 3-7, 29),
and read the run, doctor and steer code paths. No file under
`herding/run.rs`, `mailbox.rs` or `wave.rs` reads `steer-N.json`
(`rg -n steer` on those three files returns nothing), so the pi steer file
on a Paseo job had no reader. The hat wave (three seats) found the second
floor in `run.rs` (claims 13-17), the `--` need (claims 6-7), the 5 s
lazy-CLI pattern (claim 21), the local fake (claim 22) and the doc
paragraph at claim 28; all are folded in below. Probe agents `beeprobe` and
`beeprobe2` were stopped and archived. Environment: `herding.paseo.command`
is unset here, so bee resolves the running daemon's bundled CLI (0.13.2);
the npm `@getpaseo/cli` on this machine is 0.10.3 and must not be configured
as the command after this change.

## Approach

One code cell and one docs cell, in parallel on disjoint files. p13-1
swaps the run flag, adds `send_steer_argv` beside `send_argv` in
`herding/paseo.rs`, raises every floor to 0.12.0 (doctor and run.rs), and
rewires `steer_job_with` onto `Option<&dyn PaseoCli>` the way `interrupt`
does, deleting the helper module and script (D1-D7). p13-2 updates the
channel doc from the locked decisions; its text does not depend on the
code landing, and the leader checks the match at close.

Rejected: keeping `-d` until Paseo removes it (every worker spawn then
fails at once on that day); keeping the Node helper as a fallback for old
CLIs (the floor names the CLI bee supports); routing pi steers through the
mailbox file as well (no reader exists on a Paseo job); two code cells
(the split saved no time and forced a cross-cell link on `paseo.rs`).

Risk map:

| Component | Risk | Lands in | Proof needed |
|---|---|---|---|
| A CLI below 0.12.0 passing a bee check and failing on `--no-wait` | MEDIUM | p13-1 | run.rs and doctor fixtures at 0.11.4 refuse; claims 13-16 name every floor |
| Steer text starting with `-` misread as an option | MEDIUM | p13-1 | argv test pins `--` before the text; claims 6-7 held live |
| Lost refusal for a job with no paseo config | LOW | p13-1 | new test for the no-config case (none exists today; the only `paseo_steer_unsupported` assert tests the provider list that goes away) |
| Doc drifts from code | LOW | p13-2 | leader compares the doc to `send_steer_argv` and the floor at close |

Waves: p13-1 ∥ p13-2 (disjoint files).

## Role assignments

```json
{
  "schema_version": "1.0",
  "runtime": "claude",
  "roster_sha256": "094d64e24e68c0b6fabaed19ae6619f3ac5e32bac9e9da2ab51728125c724d9b",
  "stages": [
    {"stage":"implementation","classification":"required","role":"code","reason":"p13-1 changes Rust argv builders, every version floor and the steer verb."},
    {"stage":"test-authoring","classification":"not-applicable","role":"test","reason":"Tests ride the code cell."},
    {"stage":"documentation-and-capture","classification":"required","role":"docs","reason":"p13-2 updates the Paseo channel concept."},
    {"stage":"planning","classification":"not-applicable","role":"plan","reason":"The leader wrote this plan."},
    {"stage":"deployment","classification":"not-applicable","role":"deploy","reason":"A release is a separate user ask."},
    {"stage":"read-only-gather","classification":"conditional","role":"read","condition":"a worker report needs a multi-file check","reason":"Discovery is done; gathers only on demand."},
    {"stage":"fact-extraction","classification":"not-applicable","role":"extraction","reason":"Discovery is done."},
    {"stage":"generation-fallback","classification":"not-applicable","role":"generation","reason":"Every job has a specific role."},
    {"stage":"independent-review","classification":"not-applicable","role":"review","reason":"Review is user-invoked."},
    {"stage":"generic-advisor","classification":"not-applicable","role":"advisor","reason":"The hat wave is the plan consult."},
    {"stage":"supervision","classification":"not-applicable","role":"supervisor","reason":"No unattended loop runs here."},
    {"stage":"blind-lane-1","classification":"not-applicable","role":"lane-1","reason":"One shape."},
    {"stage":"blind-lane-2","classification":"not-applicable","role":"lane-2","reason":"One shape."},
    {"stage":"blind-lane-3","classification":"not-applicable","role":"lane-3","reason":"One shape."},
    {"stage":"hat-facts-gaps","classification":"required","role":"hat-facts-gaps","reason":"Standard plan check — ran."},
    {"stage":"hat-risks","classification":"not-applicable","role":"hat-risks","reason":"Three seats on standard work."},
    {"stage":"hat-value","classification":"not-applicable","role":"hat-value","reason":"Three seats on standard work."},
    {"stage":"hat-alternatives","classification":"required","role":"hat-alternatives","reason":"Standard plan check — ran."},
    {"stage":"hat-user-impact","classification":"required","role":"hat-user-impact","reason":"Standard plan check — ran."},
    {"stage":"design","classification":"not-applicable","role":"design","reason":"No rendered UI."}
  ]
}
```

## Shape

Epic map. Outcome: bee on Paseo 0.12+ spawns and steers workers through
the CLI's documented flags only.

| Epic | Capability/Risk Area | Why It Exists | Slices | Proof Needed |
|---|---|---|---|---|
| Spawn + floor + steer | `--no-wait` argv, 0.12.0 at every check, `paseo send --behavior steer -- text` | `-d` is deprecated; the Node helper is redundant; the floors disagree | p13-1 | fake-CLI and fixture unit tests |
| Record | docs | the concept must match | p13-2 | pointer check |

One slice; every cell is current.

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| p13-1 | Spawn with --no-wait, require Paseo 0.12.0 everywhere, steer through paseo send | herding/paseo.rs, herding/run.rs, herding/job_verbs.rs, herding.rs, herding/tests.rs, doctor.rs, doctor/tests.rs, herding/paseo_steer.rs (deleted), herding/paseo_steer.mjs (deleted) | — | a worker spawn prints no deprecation warning; `bee doctor` and `bee herding run` name 0.12.0 on an old CLI; `bee herding steer` on any Paseo job sends one steer message and starts no Node process | cargo test herding:: and doctor |
| p13-2 | Document the --no-wait spawn, the 0.12.0 floor and the CLI steer | docs/knowledge/areas/bee-herding/the-paseo-channel.md | — | the concept names the new argv, floor and steer path | rg pointer check |

Paths under `herding/` and `doctor` are in `packages/bee-rs/crates/bee/src/`. The cell packet:

```json
[
  {
    "id": "p13-1",
    "feature": "paseo-013-compat",
    "title": "Spawn with --no-wait, require Paseo 0.12.0 everywhere, steer through paseo send",
    "lane": "standard",
    "role": "code",
    "deps": [],
    "decisions": ["5477c172", "4236f83d", "ad803822", "dc38fecc", "5ccde9fb", "11719b08", "87defaa3"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/run.rs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/herding.rs",
      "packages/bee-rs/crates/bee/src/herding/tests.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs",
      "packages/bee-rs/crates/bee/src/doctor/tests.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo_steer.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo_steer.mjs"
    ],
    "read_first": [
      "docs/history/paseo-013-compat/CONTEXT.md",
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/job_verbs.rs",
      "packages/bee-rs/crates/bee/src/doctor.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "A. Run flag, per D1 (store 5477c172). In herding/paseo.rs `run_argv`, replace the token `\"-d\".to_string(),` (second element of the initial argv vec, right after `\"run\".to_string(),`) with `\"--no-wait\".to_string(),`. Update the three run_argv unit tests that pin it — `rg -n '\"-d\",' packages/bee-rs/crates/bee/src/herding/paseo.rs` hits `1239:                \"-d\",`, `1268:                \"-d\",`, `1306:                \"-d\",` — to expect `\"--no-wait\",`. B. Steer argv, per D3 (store ad803822) and D6 (store 11719b08). Directly below `send_argv` in paseo.rs add `pub fn send_steer_argv(id: &str, text: &str) -> Vec<String>` returning exactly [send, <id>, --behavior, steer, --no-wait, --, <text>] and add that shape to the existing `new_argv_builders_produce_expected_tokens` test. C. Version floor, per D2 (store 4236f83d) and D5 (store 5ccde9fb): every Paseo floor becomes 0.12.0. doctor.rs: the tuple at `} else if !crate::herding::paseo::version_at_least(&out, (0, 10, 3)) {` becomes (0, 12, 0); the two `upgrade to 0.10.3 with npm @getpaseo/cli` FIX strings and the ok detail `version >= 0.10.3` say 0.12.0. herding/run.rs: both `Ok(out) => paseo::version_at_least(&out, (0, 10, 3)),` sites (near the spawn and the continue path) become (0, 12, 0); the refusal strings `paseo version 0.10.3 or newer required — FIX: install or update paseo (npm @getpaseo/cli 0.10.3+)` (two sites) and `start the daemon with paseo daemon start (npm @getpaseo/cli 0.10.3+)` say 0.12.0. Tests: in run.rs the ~42 fixtures `Ok(\"0.10.3\\n\")` become `Ok(\"0.12.0\\n\")` (one bulk replace) and the asserts `assert!(msg.contains(\"0.10.3\"));` (two sites) and `...(npm @getpaseo/cli 0.10.3+)\"))` say 0.12.0; in doctor/tests.rs every `version_out: Ok(\"0.10.3\"...)` fixture (13 sites, including the AppImage string `[desktop] app startup { version: '0.10.3' }\\n0.10.3`) becomes 0.12.0, the old-version fixture `version_out: Ok(\"0.10.2\".to_string()),` becomes 0.11.4, and the two `.contains(\"upgrade to 0.10.3 with npm @getpaseo/cli\")` asserts say 0.12.0. Afterwards `rg -n '0\\.10\\.3' packages/bee-rs/crates/bee/src/doctor.rs packages/bee-rs/crates/bee/src/herding/run.rs packages/bee-rs/crates/bee/src/doctor/tests.rs` must print nothing (paseo.rs's own `version_at_least_checks_semver` test may keep its 0.10.3 sample strings). D. Steer verb, per D3, D4 (store dc38fecc) and D7 (store 87defaa3). In herding/job_verbs.rs change `steer_job_with` to take `paseo_cli: Option<&dyn crate::herding::paseo::PaseoCli>` instead of `runner: &dyn crate::herding::paseo_steer::SteerRunner`; `steer_job` (hit `steer_job_with(main_root, job_id, text, &crate::herding::paseo_steer::RealSteerRunner)`) passes None. Inside the `is_paseo` branch keep the two `paseo_steer_unsupported` refusals (unreadable or missing paseo config) and the `paseo_steer_failed` refusal for a missing `paseo_agent_id`; delete the `if spec.provider != \"pi\"` split, the `steerable` check (hit `if !crate::herding::paseo_steer::steerable(&spec.provider) {`), the `cli_package_dir` lookup and the `runner.steer(...)` call. In their place, for EVERY provider: resolve the CLI lazily exactly as `interrupt` does (hit `real_cli = crate::herding::paseo::RealPaseoCli::new(cmd).with_timeout(Duration::from_secs(5));` — `Some(c)` uses it, `None` builds from `crate::herding::paseo::paseo_command(&cfg)` with a 5 s timeout, `cfg` is already read in this branch), call `cli.call(&crate::herding::paseo::send_steer_argv(paseo_agent_id, text))`, map Err to code `paseo_steer_failed` with message `herding steer: paseo steer failed: {e}. FIX: check the Paseo daemon`, and `return Ok(next_n)` — a Paseo job never writes the mailbox steer file. The non-Paseo path (the `steer_obj` / `steer-{next_n}.json` write) stays byte-for-byte. Delete `packages/bee-rs/crates/bee/src/herding/paseo_steer.rs` and `paseo_steer.mjs`, and remove `pub(crate) mod paseo_steer;` from herding.rs. Tests in job_verbs.rs: delete `FakeSteerRunner` and `setup_fake_cli` (hits `struct FakeSteerRunner {`, `impl crate::herding::paseo_steer::SteerRunner for FakeSteerRunner {`, `fn setup_fake_cli(tmp: &Path)`); reuse the in-file `FakePaseoCli` (hit `struct FakePaseoCli {`, with `calls` and a settable `result`) and rewrite the four steer tests (`steer_job_with_fake_runner_for_claude`, `steer_job_with_fake_runner_for_pi`, `steer_job_with_unknown_provider`, `steer_job_with_runner_error`) into: a pi Paseo job and a claude Paseo job each make exactly one call equal to `send_steer_argv(agent_id, text)` and write no `steer-1.json`; a provider name bee has never seen (e.g. `omp`) makes the same single call (no allow-list); a failing CLI yields `paseo_steer_failed` with `FIX: check the Paseo daemon`; a job whose agent has no paseo config yields `paseo_steer_unsupported` (new case — today no test covers it). `steer_writes_text_verbatim_as_numbered_files` in herding/tests.rs (non-Paseo job) stays green; adjust only its call site if the signature forces it. Afterwards `rg -n 'paseo_steer::|mod paseo_steer|paseo_steer\\.(rs|mjs)' packages/bee-rs/crates/bee/src` must print nothing (the error codes paseo_steer_failed / paseo_steer_unsupported stay). No code comments anywhere.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee herding:: && PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee doctor && ! rg -n 'paseo_steer::|mod paseo_steer|paseo_steer\\.(rs|mjs)|0\\.10\\.3' packages/bee-rs/crates/bee/src/herding/job_verbs.rs packages/bee-rs/crates/bee/src/herding/run.rs packages/bee-rs/crates/bee/src/herding.rs packages/bee-rs/crates/bee/src/doctor.rs packages/bee-rs/crates/bee/src/doctor/tests.rs",
    "must_haves": {
      "truths": [
        "run_argv starts with run, --no-wait, --json and never carries -d",
        "send_steer_argv(id, text) is [send, id, --behavior, steer, --no-wait, --, text]",
        "bee doctor and bee herding run refuse a Paseo CLI older than 0.12.0 with a FIX line naming 0.12.0 and accept 0.12.0 and 0.13.2",
        "bee herding steer on a Paseo job of any provider makes one paseo send call equal to send_steer_argv and writes no mailbox steer file",
        "a paseo send error yields paseo_steer_failed with FIX: check the Paseo daemon",
        "a job whose agent has no paseo config yields paseo_steer_unsupported",
        "a non-Paseo job still writes steer-N.json verbatim and makes no CLI call",
        "no file under src refers to the paseo_steer module"
      ],
      "artifacts": [
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo.rs", "substantive": "run_argv uses --no-wait; send_steer_argv exists with a unit test"},
        {"path": "packages/bee-rs/crates/bee/src/herding/run.rs", "substantive": "both floors (0, 12, 0) and the three FIX strings name 0.12.0; fixtures moved"},
        {"path": "packages/bee-rs/crates/bee/src/doctor.rs", "substantive": "floor tuple (0, 12, 0) and FIX/ok strings name 0.12.0"},
        {"path": "packages/bee-rs/crates/bee/src/herding/job_verbs.rs", "substantive": "steer_job_with takes Option<&dyn PaseoCli>, builds the real CLI lazily with 5 s, calls send_steer_argv; fake-CLI tests cover pi, claude, unknown provider, error and no-config"},
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo_steer.rs", "substantive": "deleted"},
        {"path": "packages/bee-rs/crates/bee/src/herding/paseo_steer.mjs", "substantive": "deleted"}
      ],
      "key_links": [
        "job_verbs.rs steer path calls crate::herding::paseo::send_steer_argv",
        "herding.rs no longer declares mod paseo_steer",
        "doctor.rs and run.rs call version_at_least with (0, 12, 0)"
      ],
      "prohibitions": [
        "no change to send_argv, logs_argv, permit_argv, the wait loop or the non-Paseo mailbox steer file format",
        "no change to the semver sample strings inside paseo.rs's version_at_least_checks_semver test",
        "no change outside the nine listed files"
      ]
    },
    "trace": {"behavior_change": true}
  },
  {
    "id": "p13-2",
    "feature": "paseo-013-compat",
    "title": "Document the --no-wait spawn, the 0.12.0 floor and the CLI steer",
    "lane": "standard",
    "role": "docs",
    "deps": [],
    "decisions": ["5477c172", "4236f83d", "ad803822", "dc38fecc", "5ccde9fb", "11719b08", "87defaa3"],
    "files": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "read_first": [
      "docs/history/paseo-013-compat/CONTEXT.md",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Update docs/knowledge/areas/bee-herding/the-paseo-channel.md from the locked decisions in CONTEXT.md, in ASD-STE100 style like the rest of the file. (1) The line `The runner executes `paseo run -d --json` with the following arguments:` becomes `paseo run --no-wait --json`; add one sentence: `-d`/`--background` is deprecated since Paseo 0.12.0 (D1, store 5477c172). (2) Every `0.10.3` mention (`rg -n '0\\.10\\.3' docs/knowledge/areas/bee-herding/the-paseo-channel.md`: the spawn refusal text and its FIX line, the daemon-down FIX line, the doctor bare-version example, the paragraph `First, Paseo 0.10.3 is the minimum required version. It supports steering for the Pi provider.` and the paragraph `Second, the npm package `@getpaseo/cli` 0.10.3 can start the daemon.`) becomes 0.12.0, with the reason: 0.12.0 is the first release with `paseo run --no-wait` and `paseo send --behavior steer`, and the floor is one number at doctor and at the pre-spawn check (D2 store 4236f83d, D5 store 5ccde9fb). (3) Rewrite the `bee herding steer` paragraph (the one containing `This daemon helper requires Node.js and the npm package `@getpaseo/cli`.`): the command runs `paseo send <agent-id> --behavior steer --no-wait -- <text>` for every provider through the same CLI seam as the other calls with a 5 s timeout; the `--` separator keeps text that starts with a dash a positional (D6 store 11719b08); Paseo owns the capability check and a steer that reaches an idle worker opens a new turn, it is not dropped (D7 store 87defaa3); a Paseo job never writes a mailbox steer file; a CLI error yields `paseo_steer_failed` with `FIX: check the Paseo daemon`; `paseo_steer_unsupported` remains only for a job whose agent has no paseo config (D3 store ad803822, D4 store dc38fecc). Remove every mention of the Node helper, `activeTurnBehavior`, the WebSocket interface and the provider allow-list. (4) Delete or rewrite the paragraph `Third, the CLI command `paseo send` has no steer flag. Steering requires the daemon WebSocket interface.` — keep its last sentence about `.pi/extensions/` receiving `PASEO_AGENT_ID`. (5) Add the seven store ids (5477c172, 4236f83d, ad803822, dc38fecc, 5ccde9fb, 11719b08, 87defaa3) to the frontmatter `decisions` list as one `paseo-013-compat D1-D7` entry and `docs/history/paseo-013-compat/CONTEXT.md` to `sources`. Do not edit any other file.",
    "verify": "! rg -n '0\\.10\\.3|paseo run -d|daemon helper|Node\\.js|activeTurnBehavior|has no steer flag|WebSocket interface' docs/knowledge/areas/bee-herding/the-paseo-channel.md && rg -q 'paseo-013-compat' docs/knowledge/areas/bee-herding/the-paseo-channel.md",
    "must_haves": {
      "truths": [
        "the channel doc names paseo run --no-wait, the 0.12.0 floor at doctor and pre-spawn, and paseo send --behavior steer with the -- separator for every provider",
        "the channel doc no longer names the Node helper, 0.10.3, the WebSocket interface or a pi-only steer file"
      ],
      "artifacts": [
        {"path": "docs/knowledge/areas/bee-herding/the-paseo-channel.md", "substantive": "run argv, version floor and steer paragraphs rewritten; frontmatter cites paseo-013-compat D1-D7 and its CONTEXT.md"}
      ],
      "key_links": ["frontmatter decisions list carries the seven paseo-013-compat store ids"],
      "prohibitions": ["no edit outside the channel doc"]
    },
    "trace": {"behavior_change": false}
  }
]
```

## Test matrix

| Case | Cell | Pass when |
|---|---|---|
| Happy: `run_argv` carries `--no-wait` right after `run` and never `-d` | p13-1 | the three argv tests assert `["run","--no-wait","--json","--provider",…]` |
| Happy: `send_steer_argv("a1","go")` | p13-1 | equals `["send","a1","--behavior","steer","--no-wait","--","go"]` |
| Edge: doctor on CLI 0.11.4 | p13-1 | failure row detail contains `version is below 0.12.0 — FIX: upgrade to 0.12.0 with npm @getpaseo/cli` |
| Happy: doctor on CLI 0.12.0 | p13-1 | no version failure row (moved fixtures) |
| Edge: `herding run` pre-spawn on CLI 0.11.4 | p13-1 | refusal text contains `paseo version 0.12.0 or newer required` (existing run.rs tests at the moved fixtures) |
| Happy: steer on a Paseo job with provider pi, claude, omp | p13-1 | fake CLI records one call equal to `send_steer_argv(agent_id, text)`; no `steer-1.json` |
| Error: fake CLI returns Err | p13-1 | `paseo_steer_failed`, message contains `FIX: check the Paseo daemon` |
| Error: job agent has no paseo config | p13-1 | `paseo_steer_unsupported` |
| Edge: non-Paseo (herdr) job | p13-1 | `steer-1.json` written verbatim, no CLI call (`steer_writes_text_verbatim_as_numbered_files` green) |
| Main vs head: `cargo test -p bee herding::` and `doctor` | p13-1 | green on head with the `paseo_steer` tests gone and no other test removed |
| Docs pointer check | p13-2 | the verify rg prints nothing and the feature slug is present |

## Open Questions

(none)

## Out of scope

Background workspaces for bee worktrees (`workspace create --background`);
`paseo usage` for token accounting; the `--host` global option; a `--`
separator on `send_argv` (its nudge text is fixed and never starts with a
dash); the steer counter shown on Paseo jobs (always 1 now, cosmetic).
