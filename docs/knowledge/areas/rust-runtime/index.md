<!--
GENERATED FILE — do not hand-edit.
Rendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).
Regenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.
Deterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,
never a generation timestamp or any other wall-clock value.
-->

# areas/rust-runtime/

## Concepts

- [Compiled runtime: purpose, guarantees, and the artifacts it writes](overview.md) — The compiled runtime replaces the interpreted reference on the paths a session pays for every turn, promising identical output and store writes with no child process spawned. What it guarantees, what stays dark until activation, the one additive artifact it writes, its fail-open crash contract, and the reference defect it reproduces on purpose.
- [Compiled runtime: how a port is proven faithful](parity-and-conformance-proof-discipline.md) — The house discipline for proving equivalence against a frozen reference — parity legs and scenarios, the single volatility allowlist, per-leg negative controls, oracle rules for importable and command-only units, the conformance rig's five elements, environment pinning, and the byte-versus-parsed comparison rule with the meta-test that guards the instrument itself.
- [Compiled runtime: per-command performance budgets and the host-real fixture floors](performance-budgets-and-fixture-floors.md) — Speed is a gated contract, not an aspiration: per-command budgets measured spawn-inclusive over a store pinned to real sizes, both cache states reported, the reference figure recorded beside every result. Includes the status supersession to an interim 70 ms, its measured cause, and the mandatory follow-up that tightens it.
