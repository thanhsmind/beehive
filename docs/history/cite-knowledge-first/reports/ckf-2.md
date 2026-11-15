# ckf-2 — checker report for ckf-1

Checked commit: 848d8d1d8 (cell ckf-1). No product file was edited.

## Verify command

`PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test rule_index_parity --test pointer_integrity --test instruction_laws && .bee/bin/bee dev release-manifest --check && .bee/bin/bee onboard --repo-root . --json` exited 0.

| Step | Fresh result |
|------|--------------|
| `instruction_laws` | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `pointer_integrity` | `test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `rule_index_parity` | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` |
| `bee dev release-manifest --check` | `950 file(s) match stored manifest, 1 unhashed artifact(s) present` |
| `bee onboard --repo-root . --json` | exit 0, every target `mode: sync` at 2.106.0, `workspace_protocol.state: valid` |

## Structural checks

- `cmp packages/bee/LEADER.md .bee/LEADER.md`: byte-equal.
- Source order paragraph at line 313 in all six copies of `bee-hive/references/routing-and-contracts.md` (`skills/`, `.agents/skills/`, `.claude/skills/`, `.claude-plugin/skills/`, `.codex-plugin/skills/`, `.opencode/skills/`); `cmp` shows each of the five rendered copies byte-equal to the source.
- Example paths in the Source order block resolve:
  - `docs/knowledge/areas/onboarding/host-project-artifacts.md:322` is the heading "Static checks are offered as members of `commands.test`", the section that holds the propose-only lint rule.
  - `docs/history/trust-layers/CONTEXT.md D1` exists at line 28 and states "Propose-only: bee never writes a detected command".

## ckf-1 truths

- met — the citation rule says a how-it-works claim cites the `docs/knowledge` concept first and `docs/history` only for why: `packages/bee/LEADER.md:210` (rule `agents-verify-before-agreeing`), full form `skills/bee-hive/references/routing-and-contracts.md:313`.
- met — the knowledge-first rule covers a turn that answers how something works before `docs/history` is opened: `packages/bee/LEADER.md:168-170` (rule `agents-knowledge-before-code`).
- met — the LEADER.md history line scopes its cite instruction to the locked decision and its reason: `packages/bee/LEADER.md:53`, with current behavior homed in the knowledge concept at `packages/bee/LEADER.md:54`.
- met — `.bee/LEADER.md` is byte-equal to `packages/bee/LEADER.md` after regen: `cmp` exit 0, `.bee/LEADER.md:210` carries the same Source order line.

## Key links and prohibitions of ckf-1

- Claim labels stays the full form of `agents-verify-before-agreeing`: Source order sits directly after Claim labels in `skills/bee-hive/references/routing-and-contracts.md:313`.
- Rule index spoken lines match the rule blocks: `docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md:226` and `:308`; `rule_index_parity` green.
- No new rule id, no hook or guard change: the ckf-1 diff touches only prose, render manifests, `.bee/onboarding.json` and the release manifest.

## Notes

- The release-manifest check reports 1 unhashed artifact present; the check still passes. Not traced to ckf-1.
- The `.bee/LEADER.md` Source order short form omits "or for its locked wording", which the full form at `routing-and-contracts.md:313` keeps. D1 places the short form in the rule and the full form in Claim labels, so this is within scope.
