# Provenance — bee-planning body rules

The planning body states its rules bare (provenance exile, skill-token-diet D8). This table maps each
body rule to the decision(s) that authorize it and the rationale in one line. Long-form records:
`docs/decisions/`, `.bee/decisions.jsonl` (via `bee.mjs decisions search`), and
`docs/history/<feature>/CONTEXT.md` for the feature that locked each ID.

| Body rule | Decision IDs / labels | Rationale |
|---|---|---|
| Mode gate runs first, before lane-scaled bootstrap | mode-gate D8 | Tiny work must not pay full context reads before it knows it is tiny |
| Risk-flag list; covered bugfix scores 0 on the last two flags | mode-gate D7 | A fix that keeps existing tests green and adds one is not "changing proven behavior" |
| Lane file caps count product files only | mode-gate D6 | `.bee/**`, docs, plans, and generated projections are bookkeeping, not risk surface |
| Mode-gate record location by lane (cell / logged decision / plan.md) | planning D3, D4 | Ceremony scales down when the cell itself carries the whole shape |
| Greenfield init lane — one init cell before feature work | P1, docs/09 item 6 | A repo with no build gets infrastructure first; declined offers are recorded, never dropped |
| Lane-scaled bootstrap ordering; area-truth reading order `bundle → decisions → history` | harness10 G1/G4; okf fences G2 | Both branches (bundle / no bundle) are live guidance; a never-migrated repo keeps working unchanged |
| Re-lane checkpoint (evidence-based demotion, once per feature, only if exploring was skipped) | lane-lean D1/D2; spec #81 P3 | Measured evidence demotes to the smallest honest lane; hard-gate flags never demote |
| Discovery levels L0-L3; L2+ dispatches `bee-xia` in-chain | discovery levels (planning §3, pre-migration) | Lowest level that removes real uncertainty; precedent beats research |
| Delegation-tier dispatch for lane-scaled bootstrap and ad-hoc research | delegation D2/D3; decisions 0013/0015 reversed, 0016, 0023 | Transport is mandatory on every dispatch; gather work runs down-tier as I/O |
| Artifact fan-out — separate discovery.md/approach.md/implement-plan.md only when earned | decision 0009 | A small/standard feature that spawned four files restating the same "current state" is the anti-pattern this closes |
| tiny/small: no plan.md by default; standard/high-risk: one plan.md | planning D3, D4 | The cell or the logged scoping synthesis carries the shape at the smallest lanes |
| Plan freeze — content immutable after Gate 2, approval stamp only | planning D1 | The artifact the human approved must stay byte-equal to the artifact that ships |
| Gate 2 bypass mechanics (stamp, audit line, auto-approve message) | decisions 0010, dcf01d7b | The human chose the level in advance; the recommended option is the approval at that level |
| Tiny/small merged gate — preview before persist, one question covers both gates | fast-path D5 | Approval covers exactly the previewed packet; cells persist only after the merged yes |
| Walking skeleton — slice 1 is the thinnest end-to-end runnable path | spec #81 P2 | A slice proves it runs before structural work rides along |
| One trailing test cell per slice (slice-tail-test-batching) | spec #80/#85 P2 | Implementation cells cap on existing-green; the slice's net behavior is tested once, not per cell |
| Verify is scoped, never the full configured chain | verify-scoping D2, decision 20534ea9; ci-owned-verify D1/D6 | The full suite is CI-owned; a cell's verify is the narrowest honest check |
| `state set` needs a real phase-enum value, never invented | chain-integrity D6 | An agent that hits the refusal and improvises the state machine is exactly how the chain broke |
| Scope-Reduction Prohibition — answer SPLIT RECOMMENDED, never shrink a locked decision | planning invariant (pre-migration Scope-Reduction Prohibition) | A locked decision is cited, never quietly reinterpreted or dropped to fit a budget |
| Headless — never self-approve Gate 2 or the merged gate; ambiguities go to Outstanding Questions | hive law 14 (Headless mode) | Headless is not bypass; every mode still stops at its gate |
