# Plan: paseo-cli-autodetect

## Summary
bee calls `paseo` from PATH unless `herding.paseo.command` is set. On this
machine PATH `paseo` is the desktop AppImage wrapper: every call opens the
Paseo app and prints no JSON, so `bee herding run` fails with `spawn_failed`.
The user wants bee to use the CLI that belongs to the running daemon, with no
per-machine config. `paseo_command` now resolves config, then `PASEO_CLI`,
then the running daemon's CLI, then plain `paseo` (decision 03ff4564).

Mode: `small` — 1 risk flag: external-systems. One cell, one source file plus
its knowledge concept.

## Load-bearing claims

| # | Claim | Label | Anchor | Verbatim evidence |
|---|-------|-------|--------|-------------------|
| 1 | every Paseo call site gets its command from one resolver | read | packages/bee-rs/crates/bee/src/herding/paseo.rs:76 | pub fn paseo_command(cfg: &Value) -> String { |
| 2 | the paseo home resolver exists and is reusable | read | packages/bee-rs/crates/bee/src/herding/paseo_state.rs:39 | pub(crate) fn home_from(command: &str, paseo_home: Option<OsString>, user_home: Option<PathBuf>) -> Option<PathBuf> { |
| 3 | the daemon writes its pid as JSON in the paseo home | ran | ~/.paseo/paseo.pid | {"pid":1559719,"startedAt":"2026-10-12T02:26:00.968Z",...,"desktopManaged":true,...} |
| 4 | the daemon process carries PASEO_CLI in its environment | ran | /proc/1559719/environ | PASEO_CLI=/tmp/.mount_Paseo.C0veTi/resources/bin/paseo |
| 5 | that CLI matches the daemon and its run --json works | ran | $PASEO_CLI --version; $PASEO_CLI run -d --json | 0.11.2; {"agentId": "3eca3204-…", "status": "running", …} |

## Cells — current slice (preview)

| id | title | files | deps | you see | proof |
|---|---|---|---|---|---|
| pcad-1 | Find the paseo CLI of the running daemon without config | paseo.rs, the-paseo-channel.md | — | bee herding run spawns a Paseo worker with no herding.paseo.command set | herding::paseo tests |

```json
[
  {
    "id": "pcad-1",
    "feature": "paseo-cli-autodetect",
    "lane": "small",
    "role": "code",
    "change_class": "bugfix",
    "title": "Find the paseo CLI of the running daemon without config",
    "deps": [],
    "decisions": ["03ff4564"],
    "files": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "docs/knowledge/areas/bee-herding/the-paseo-channel.md"
    ],
    "read_first": [
      "packages/bee-rs/crates/bee/src/herding/paseo.rs",
      "packages/bee-rs/crates/bee/src/herding/paseo_state.rs"
    ],
    "affects_skills": [],
    "affects_specs": ["docs/knowledge/areas/bee-herding/the-paseo-channel.md"],
    "action": "Per decision 03ff4564, in packages/bee-rs/crates/bee/src/herding/paseo.rs change paseo_command(cfg) (line 76): return herding.paseo.command (or the legacy top-level paseo.command) when set and non-blank; else the PASEO_CLI env value when it names an existing file; else the running daemon's CLI: read <paseo home>/paseo.pid (JSON, key pid; home from crate::herding::paseo_state::home_from with an empty command, PASEO_HOME, HOME), then the PASEO_CLI entry of /proc/<pid>/environ when it names an existing file, else <parent of /proc/<pid>/exe>/resources/bin/paseo when that file exists; else \"paseo\". Put the discovery in a pure helper that takes the env value, the paseo home and a proc root path, so tests drive it with a tempdir. Under cfg(test), paseo_command must not read the real process env or the real /proc (follow the TEST_PASEO_HOME thread_local pattern in paseo_state.rs), so paseo_command_reads_config_and_defaults_to_paseo still passes inside a Paseo agent. Tests red first: config wins over env; env wins over daemon; daemon environ PASEO_CLI; daemon exe-dir fallback; a missing or garbage pid file falls through to paseo; an env value naming a missing file is skipped. In docs/knowledge/areas/bee-herding/the-paseo-channel.md replace 'The key `herding.paseo.command` sets the CLI executable name. The default command is `paseo`.' with the new order, citing decision 03ff4564. Write no code comments.",
    "verify": "PATH=\"${CARGO_HOME:-$HOME/.cargo}/bin:$PATH\" cargo test --release --manifest-path packages/bee-rs/Cargo.toml -p bee --bin bee herding::paseo",
    "must_haves": {
      "truths": [
        "with no config, bee uses PASEO_CLI when it names an existing file",
        "with no config and no PASEO_CLI, bee uses the CLI of the daemon named in paseo.pid",
        "a set herding.paseo.command always wins",
        "with nothing found, the command is plain paseo"
      ],
      "prohibitions": [
        "no new config key",
        "no change to any paseo_command call site",
        "tests never read the real /proc or the real PASEO_CLI env"
      ]
    }
  }
]
```
