# host-packaging-gaps — hat wave synthesis

Seats: `hat-facts-gaps`, `hat-alternatives`, `hat-user-impact` (claude,
opus), over plan revision 1. Revision 2 of `plan.md` applies every
finding below; its `## Hat wave` section lists what changed.

```text
PLAN CHECK
Work: host-packaging-gaps, slice 1 (all four cells)

STRUCTURE
BLOCKERS:
- Key links / release manifest: installer edits stale the manifest
  (release_manifest.rs INVENTORY_ROOTS lists both installers) / fix:
  hpg-3 lists and regens docs/history/codex-harness-hardening/release-manifest.json. APPLIED.
- Cell completeness / version read: python3 on windows-latest unproven,
  one red row blocks every asset / fix: sed, the pattern install.sh:387
  already uses. APPLIED.
WARNINGS:
- Coverage / D1 narrowed to "manifest absent" / onboarding.json first. APPLIED.
- DAG / hpg-3 live run needed hpg-1 / moot after the re-cut: the live run is at feature close.
- Scope / plugin-first + pi yields no skills or hooks / refuse early. APPLIED.
- Key links / the exact helper call is install.sh:698 / named in hpg-3. APPLIED.
- Completeness / onboarding product doc and mod.rs doc comment missed / added to hpg-2. APPLIED.
- Proof / pi_plugin_contracts not run by the onboard filter / added to hpg-2 verify. APPLIED.
- Risk / five runners, publish needs all / named in the risk map. APPLIED.

ALTERNATIVES
- Verdict PASS on D1-D4. Re-cut cells by file so all four run in one
  wave. APPLIED. team+models both present -> team. APPLIED.
  config-sample read-only. APPLIED.

USER IMPACT
- HIGH: plain-terminal Pi users still BLOCKED on herding_transport ->
  became D5 (store 974a2285) on the user's word. APPLIED as hpg-1 (B).
- Host remedy said cargo / onboard -> installer only. APPLIED.
- Silent config rewrite -> merged-config check, indent and line ending
  kept, notice, offered once. APPLIED.
- No Pi default model -> banner text; doctor row out of scope. APPLIED.
- ARM glibc floor -> smoke run after checksum, ubuntu-22.04-arm. APPLIED.

CELLS  (reviewed: 4, leader self-check)
CRITICAL FLAGS: none open.
MINOR FLAGS: hpg-4's real proof is the next release run; named at close.
CLEAN CELLS: hpg-1, hpg-2, hpg-3, hpg-4

SUMMARY: Two blockers and seven warnings, all applied in revision 2.
One finding became a new locked decision (D5) with the user's word. No
blocker is open, so no second pass is needed.
```
