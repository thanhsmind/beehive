# mistake-fix-at — plan-step hat wave synthesis

Three seats (hat-facts-gaps, hat-alternatives, hat-user-impact), opened
2026-09-22 under decision 2ad8669e, over CONTEXT.md D1–D5 and plan.md
revision 1. Synthesized by the leader; plan.md revision 2 carries the
result. Budget: all three seats returned inside the 10-minute ceiling; no
seat dropped.

PLAN CHECK
Work: mistake-fix-at, single slice, cells mfa-1..mfa-4 (revision 2; revision 1 had six)

STRUCTURE
BLOCKERS (all resolved in revision 2):
- cell completeness / mfa-1's signature change had callers outside its files (handlers_close.rs:905,1036; close.rs:5043; mailbox_digest.rs:1242) / door and cap merged into mfa-1, test callers named as compile-fix files
- cell completeness / the prompt edit reds drivers/tests.rs:9551 which no verify ran / docs cell runs regen then that test target
- key links / no cell could produce .bee/bin/prompts/worker-cell.md / regen copies it from the on-disk template (onboard/plan.rs:811); docs cell owns it
- claims membership / CAP_FLAGS, regen chain, generated AGENTS.md region, "nothing reads trace.mistakes" / rows 13, 14, 16, 39, 42 added
WARNINGS (noted):
- D2 source set narrower than its wording / D2a decision d1d83be7 logged (touches D2)
- one-commit window where help advertised a flag the door refused / gone with the merge
- MistakesAnswer element type undecided / named: Mistake { wrong, better, fix_at }
- mfa-6 verify reached one of four must_haves / verify now diffs all three mirrors and rg-checks each file

CELLS (reviewed: 4, leader's cold-pickup pass)
CRITICAL FLAGS: none open
MINOR FLAGS: mfa-1 is the largest cell (8 files, 5 test filters) — accepted because it is one compile unit; its prohibitions fence the two compile-fix files
CLEAN CELLS: mfa-2, mfa-3, mfa-4

ALTERNATIVES ACTED ON
- mfa-5 + mfa-6 → one docs cell, last (one regen, one manifest write)
- dedupe via read_jsonl + is_finding_row; append via fsutil::append_jsonl; only backlog_finding_row lifted
- trouble_lines returns (key, verbatim); brake applied to the what
- registry proof spelled with --test targets (frt-1 precedent)
- prompt edit last because prepare.rs:3545 refuses on skew

USER IMPACT ACTED ON
- refusal string self-sufficient: names the missing flag, lists the four values, gives the remedy; "two" → three in both homes
- tie-break sentence in the doctrine line
- close line in the tail's voice with the bee backlog findings command, after the Retired line
- lesson row names the key and quotes the other run
- cells-and-proof.md moves into mfa-1

DISMISSED
- alternatives 1 (keep the door/cap split with compile-fix files): a compile-fix that leaves --mistake accepting no layer would violate D1 for one wave; merged instead.
- user-impact 3 (host-side re-onboard nudge in the registry drift hint): outside every locked decision; the self-sufficient refusal is the recovery and bee onboard --apply is the existing upgrade step.

SUMMARY: The claims table held byte-for-byte (33/34, the one miss was a
command anchored to the wrong checkout). The shape was sound; the seams
between source and generated copies were not — the wave moved the door and
the cap into one cell, the two docs cells into one last cell, and added the
skew test, the regen step and the honest lesson row. Every locked decision
D1–D5 stands as written.
