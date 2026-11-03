# Hat wave digest — verification-in-the-flow (plan step, 5 seats)

Opened `84485816`; synthesis `1fb5dc3e`. Anchored on `plan.md` rev 0.

**Unanimous verdict on the plan's open question:** retire D2 (flatten `.bee/verify/`),
keep the nested `.bee/verify/verify-app/`. Four independent grounds — no observable
behavior change (D1 already gives one constant path), loss of a deliberately-commented
renderer design and of multi-skill support, a one-product limit pinned into the format
rather than the naming convention, and an old-binary skew in which a flattened root
renders `features/` as a phantom skill nothing in bee can prune.

## facts-gaps
One BLOCKER: claim 12's anchor named `pstack-adoption/CONTEXT.md`; the bytes live at
`pstack-gaps/CONTEXT.md:69`. Three drifted line numbers. Three prose-only load-bearing
<!-- bee:not-a-deferral: a wave digest records what the seats found at that moment; every gap it names was answered in the plan revision that followed -->
claims. Gaps: the 8-cell truth table filled 4 ways; where the existence check reads;
CONTEXT's deferred preamble question never answered; no test row pinning D4/D5 text.
<!-- /bee:not-a-deferral -->
One seat claim corrected by the leader: `bee-verifying` has NOT shipped in a release.

## alternatives
Slice boundary reshuffle — Rust alone in slice 1, all text in one serial pass in slice 2,
saving one serial pass and one regen. D3's minimum is reword-one-constant plus add-one.
D5 is a fourth parenthesised case, not a new bullet. D7 is a move, not authorship.
No cheaper shape for D6, the test matrix, or D1.

## user-impact
Four-state SEE mock. The existing constant opens "This project has no command that
proves it works" — false for a repo declaring a test command, so states A and B need
different first sentences. Onboard re-runs on version mismatch and bee tags several
releases a day, so a blind re-offer is a daily nag. A mapless repo is the majority case
on day one; absent and empty are safe to skip, stale at planning is not.

## risks
R1 rendered-copy overwrite of hand-maintained map memory. R2 the no-migration claim
expires at the next release tag. R3 D6 deletes the knowledge needed to decompose a
composed host. R4 regen drift is invisible — `rule_index_parity.rs` compares marker sets
only and CI has no regen-diff. R5 the old `verify-bee` twin is non-`bee-` named so no bee
path will prune it. R6 the legacy-key rows are missing from the matrix.
Cleared outright: every deletion path in bee was traced; a non-`bee-` named
`verify-app` source tree is read-only to bee in all of them, and an interrupted render
self-heals.

## value
D1 MATERIAL (load-bearing). D2 CEREMONY, negative value. D3 MATERIAL, the core.
D4 THIN — an untested theory; falsifier named. D5 MATERIAL but only if a pointer lands
beside the cap step, since a case in the contract's middle rots. D6 MATERIAL and cheapest
now, while the host population is zero; the retirement must also name `bbedc1d2` as
mooted. D7 THIN as dogfood, MATERIAL as the only live proof of the render path.
80/20 subset: D1 + D3 + D6 + D5's row.
