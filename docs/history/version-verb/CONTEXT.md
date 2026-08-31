# version-verb — CONTEXT

## Asked

`bee version` does not exist. `bee --version` refuses as an unknown
command (screenshot: an agent tried both and got the unknown-command
refusal). Add a version verb.

## Found

- The release version is `crate::version::BEE_VERSION`, parsed at
  compile time from `.claude-plugin/plugin.json` — the one source of
  truth since the R6 cutover (`crates/bee/src/version.rs`).
- `bee rs-info` already prints it, buried in a diagnostic JSON blob.
- Wiring recipe for a new top-level verb is pinned by commit 5678b2f8
  (`bee blind check`): probe in the dispatch chain, `router.rs` PORTED
  line, hand-edited `registry_payload.json` entry with a runnable
  example (`bee dev regen` does not write that file). No new flag
  spelling → `catalog.rs` PINNED_FLAG_COUNT stays 198.

## Will do

One cell: serve `bee version` (and the conventional spellings
`--version` / `-V`) from the router, rootless like `rs-info` — prints
`bee <BEE_VERSION>`, `--json` gives `{"version": …, "binary": …}`.
Registry entry + PORTED line + tests. No store touch, no behavior
change anywhere else.
