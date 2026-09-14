# Advisor Digest: Semantic Role Routing

**Plan revision:** 1
**Brief SHA-256:** `f59f94d0f634d26e13f6c5cf4272b3cd0fd2639dec1edf8726930f210f92a2f8`
**Final verdict:** ready after corrections

## Hat Results

| Hat | Initial verdict | Report |
|-----|-----------------|--------|
| Facts and gaps | not-ready | `.bee/mailbox/job-1789371126038-163790-1/report-1.md` |
| Risks | not-ready | `.bee/mailbox/job-1789371126039-163796-1/report-1.md` |
| Value | not-ready | `.bee/mailbox/job-1789371126039-163798-1/report-1.md` |
| Alternatives | not-ready | `.bee/mailbox/job-1789371126039-163800-1/report-1.md` |
| User impact | ready-with-corrections | `.bee/mailbox/job-1789371126040-163803-1/report-1.md` |

## Shared Findings and Corrections

1. **Packet schema was implicit.** The plan now defines the exact v2
   discriminator, section, fields, roster digest, coverage rule, stored key,
   malformed cases, and byte-identical legacy path.
2. **Runtime scope was missing.** The role plan now names one runtime. Preview
   and prepare validate its current complete roster and digest. Cross-runtime
   execution requires a new plan revision.
3. **Non-cell stage transport was missing.** The plan now defines `--feature`,
   `--stage`, session-lane fallback, native markers, audit fields, and typed
   refusals.
4. **Reroute authority was ambiguous.** The plan now defines one atomic command,
   the shared cell lock, decision validation, exact history record, immutable
   original packet, and the only permitted effective-role chain.
5. **Release authorization was not trustworthy.** The plan now binds a two-hour,
   one-use authorization to session, feature, plan, runtime, stage, role,
   version, and main commit. The script consumes it before all mutation paths.
6. **Pi used one root for two facts.** The plan now separates the active source
   root from the main control root. Existing Pi validation stays strict.
7. **Delivery order contradicted dependencies.** Only `slr-3` and `slr-7` start
   together. Dependent cells follow their declared edges. Deployment receives
   preflight proof only; this feature publishes no release.
8. **Refusal recovery was unclear.** The plan now has a refusal matrix with the
   mutation boundary and one recovery path for each new reason family.

## Residual Risks

- `scripts/release.sh` authorization must execute before both normal and resume
  mutation paths. Cell `slr-6` carries explicit tests for both.
- Native markers must stay absent on legacy prompts. Cell `slr-4` owns exact-byte
  regression tests.
- Pi extension changes are not planned. Cell `slr-7` can add the file only after
  a recorded re-route if a red contract test proves the assumption false.

No advisor finding remains unresolved in the Gate 2 packet.
