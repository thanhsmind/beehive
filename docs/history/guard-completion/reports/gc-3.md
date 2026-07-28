# gc-3 — Restore two dropped bee-scribing routing rules

**[DONE]**

The skill-diet-wave2 thin-body migration collapsed `skills/bee-scribing/SKILL.md`'s
bundleMode routing paragraph into one line, dropping two load-bearing rules the
CI-only `test_bundle_mode.mjs` suite pins: where new area truth is written when a
knowledge bundle exists vs. not, and the `scribingTarget()` empty-subject/two-claimant
refusal law. Read both check bodies (not just the assertion strings) before restoring.

Restored as an explicit `### 2a. Bundle branch` / `### 2b. No-bundle branch` split:
2a routes new area truth to `docs/knowledge/areas/<area>/*.md` via `scribingTarget()`
and names its two refusals (`subject_required`, `duplicate_authority`) plus the
bundle-wide `duplicate_authoritative_for` chain-fail backstop; 2b keeps today's
no-bundle rule verbatim (`**One area = one file, forever.**`). Paid for the restored
bytes by trimming reassurance prose in the Capture, Merge, and Red Flags sections of
the same body — net body size 8135 -> 8145 bytes (fence budget 8151).

Files touched:

- `skills/bee-scribing/SKILL.md`

Full trace and verification evidence: `.bee/cells/gc-3.json`.
