<!--
GENERATED FILE — do not hand-edit.
Rendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).
Regenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.
Deterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,
never a generation timestamp or any other wall-clock value.
-->

# areas/rust-runtime/

## Concepts

- [Compiled runtime: the command surface — flow verbs, aliases, and the plumbing namespace](command-surface.md) — Why the CLI shows a small default surface instead of its whole registry, which verbs earn a place in it, how a flow verb is an alias rather than a second implementation, what every porcelain verb owes its caller at the point of contact, and why drift detection still hashes the full registry the split hides.
- [Compiled runtime: purpose, guarantees, and the artifacts it writes](overview.md) — The compiled runtime replaces the interpreted reference on the paths a session pays for every turn, promising identical output and store writes with no child process spawned. What it guarantees, what stays dark until activation, the one additive artifact it writes, its fail-open crash contract, and the reference defect it reproduces on purpose.
- [Compiled runtime: how a port is proven faithful](parity-and-conformance-proof-discipline.md) — The house discipline for proving equivalence against a frozen reference — parity legs and scenarios, the single volatility allowlist, per-leg negative controls, oracle rules for importable and command-only units, the conformance rig's five elements, environment pinning, and the byte-versus-parsed comparison rule with the meta-test that guards the instrument itself.
- [Compiled runtime: per-command performance budgets and the host-real fixture floors](performance-budgets-and-fixture-floors.md) — Speed is a gated contract, not an aspiration: per-command budgets measured spawn-inclusive over a store pinned to real sizes, both cache states reported, the reference figure recorded beside every result. Includes the status supersession to an interim 70 ms, its measured cause, and the mandatory follow-up that tightens it.
- [Compiled runtime: prompt files, and the learned-context block that closes the learn-to-use loop](prompt-files-and-learned-context.md) — Why every machine-assembled prompt body lives in a file rather than in code, what the split between template wording and computing logic buys, how a dispatched worker is handed the project's own learned context instead of re-deriving it, and why every source in that resolution chain fails silently.
