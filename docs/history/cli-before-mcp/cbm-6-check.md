# cbm-6 check — skillstation alias help text

Checked cell: cbm-5, commit 03e7c31dc3df9237c97c7bbbd7c25e374ad9b33d.
Decision: 069eed40 (D5, contract:cli-first-hub-texts).

## Truth

The skillstation alias help text names `skillstation skill show <id> --json`
before `skill_get`, and no other command help in `registry_payload.json`
changed.

**PASS.**

- Anchor: `packages/bee-rs/crates/bee/src/generated/registry_payload.json`,
  command `skillstation.alias`, field `description`.
- Old phrase: "a short body telling the agent to call the skillstation MCP
  tool `skill_get` with skill_id `<id>`."
- New phrase: "a short body telling the agent to run
  `skillstation skill show <id> --json` first and call the skillstation MCP
  tool `skill_get` as the fallback."
- Scope: both payload versions were parsed as JSON and compared per command
  and per field. Both hold 237 commands; the only top-level key that differs
  is `commands`, and the only differing (command, field) pair is
  (`skillstation.alias`, `description`). The commit touches that one file.

## Verify

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --bin bee -- registry skillstation`

Result: 59 passed; 0 failed; 0 ignored; 5257 filtered out.
