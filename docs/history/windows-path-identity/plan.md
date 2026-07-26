---
artifact_contract: bee-plan/v1
mode: standard
approved_gate2: 2026-07-27  # auto-approved under gate_bypass=total
---

# windows-path-identity — one root cause, seven suites, four standing exclusions

## Mode gate

Flags: **cross-platform** (the whole subject), **multi-domain** (worktree store, herding resolution, three test suites, the Windows workflow itself), **changes behavior an existing test asserts** (removing the four standing exclusions changes what CI asserts). Three flags → `standard`. Product files: `packages/bee/lib/worktree-store.mjs`, `packages/bee/lib/herding.mjs`, `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs`, plus test and workflow files. No hard-gate flag: nothing here touches auth, data deletion, or an external provider — a mis-resolved worktree is refused (`return null`), never merged wrongly, so the failure mode is a false negative rather than data loss.

## The finding

Windows CI has been red for four consecutive days while main CI is green on the same commits. Diagnosis: **one mechanism family, not four bugs** — path-string identity. Two sources produce paths that are byte-identical on POSIX and need not be on Windows:

- **git's stdout** (`git rev-parse --path-format=absolute --git-common-dir`, `git show --name-only`) always emits `/`, on every platform.
- **Node's path machinery** (`path.join`, `path.resolve`, `fs.realpathSync`) emits the native separator, in whatever case the filesystem reported — and Windows is case-insensitive, with 8.3 short-name aliasing on top.

Comparing the two with `===` works everywhere except where it matters.

This is not new. `.github/workflows/windows.yml:84-103` already excludes **four** suites for exactly this cause, names "path normalization is the likely shared root cause" as the fix direction, cites a P1 backlog row from 2026-07-21, and warns explicitly: *do NOT silently extend this list without a matching backlog row.* Since then the list has effectively grown by three more suites — not by being added to the exclusion, but by failing.

### The three currently-failing sites

| Site | What breaks | Product or test |
|---|---|---|
| `packages/bee/tests/test_cli_cells.mjs:878-879` | compares `git show --name-only` output (always `/`) against `path.join('.bee','backlog.jsonl')` (`\` on win32). Cannot pass on Windows. The product's own pathspec is fine — git accepts either separator as an argument. | **test** |
| `packages/bee/tests/test_herding_cli.mjs:70-71` against `packages/bee/lib/herding.mjs:42-51` and `.claude/skills/bee-herding/scripts/dispatch-interlock.mjs:67-80` | `main_root` is `path.dirname(<git stdout>)` — never normalized, so it keeps git's forward slashes and git's reported casing; the test compares it with `===` against `fs.realpathSync(mkdtempSync(...))`. | **test**, plus a real canonicalization gap in the product's return value |
| `packages/bee/lib/worktree-store.mjs:1098-1110` (and the sibling resolution at `:491`) | the bidirectional gitdir-pointer check normalizes separators but then compares with raw `!==` — no realpath, no case fold. A linked worktree fails to resolve, so `bee worktree merge` never reaches its verify child, so `scripts/tests/test_worktree_merge_queue.mjs:91` times out waiting for a marker, so `test_msn_invariants.mjs` invariants 7 and 12 fail loud. | **product** — and windows.yml already classifies it as such |

The FAIL LOUD is not a missing suite: the shard filter only restricts run_verify's own discovery list, while `test_msn_invariants.mjs` spawns the merge-queue suite by absolute path from the repo root, independent of the filter. The file is present and genuinely failing.

Timeline confirms one mechanism: the first two sites began failing on the first Windows run after their shared 2026-07-23 merge; the third began a day later purely because its code merged on 2026-07-25. Exposure dates track each feature's introduction, not a shifting cause.

### A fourth defect, found while reading

`.github/workflows/windows.yml:114` runs `node scripts/test_portable_paths.mjs`. The file is at `scripts/tests/test_portable_paths.mjs`. That step cannot succeed; it has been masked because the job fails earlier.

## Approach

Build one canonical path-identity comparison and use it at every site where two independently-obtained paths are compared. Canonical means: resolve, normalize separators, follow symlinks where the path exists, and fold case **only** on platforms whose filesystem is case-insensitive. Compare filesystem identity (device + inode) when both paths exist and the platform provides it, falling back to the normalized-string comparison when it does not — identity is the property actually wanted, and the string comparison is a proxy for it.

Then fix the two test-side sites so they stop asserting a contract the product never promised, canonicalize the herding resolver's return value so no future caller rediscovers the same mismatch, repair the workflow's broken guard path, and remove the four standing exclusions.

**Rejected:** loosening the merge-queue suite's timeout, or skipping invariant 12 on Windows. Both hide a product bug the workflow's own comments already identified — the exact papering-over the comment block warns against.

## Risk map

| Component | Risk | Proof needed |
|---|---|---|
| Canonical comparison helper | **MEDIUM** — a comparison that folds case on a case-sensitive filesystem would merge two genuinely distinct paths | Tests that pin both platform behaviours explicitly, including a negative control where two paths differing only in case must compare UNEQUAL on a case-sensitive platform |
| Worktree gitdir identity | **MEDIUM** — this feeds `bee worktree merge`; a false positive would resolve the wrong worktree | A fixture with a mixed-separator and mixed-case pointer that resolves correctly, plus one with a genuinely wrong pointer that still returns null |
| Removing four exclusions | **MEDIUM** — they were excluded for this cause, but may hide unrelated failures too | Each previously-excluded suite runs green locally before the exclusion is removed; the Windows run after push is the acceptance evidence |
| No Windows machine here | **HIGH for verification** | Local proof is semantic (the helper's platform branches are unit-tested both ways); the real acceptance is the CI run, recorded in the report rather than claimed from the developer machine |

## Cells

1. **wpi-1** — the canonical path-identity helper plus its platform-branch tests, applied at the worktree-store gitdir sites.
2. **wpi-2** — the two test-side fixes and the herding resolver's canonicalization.
3. **wpi-3** — repair the workflow's guard path, remove the four standing exclusions, prove each previously-excluded suite green locally, and record the Windows CI result as acceptance.

## Open questions for validating

- Does any caller depend on `main_root` keeping git's exact spelling rather than a canonical one? A canonicalized return is a behaviour change for anyone string-matching it.
- Are the four standing exclusions genuinely all this cause, or does one of them hide a second, unrelated Windows failure? That is only knowable by running them — locally they pass, so the answer arrives with the CI run.

---

# CORRECTION (2026-07-27, after validation) — this plan's central claim was wrong

The validation pass refuted the load-bearing claim above and five other things. This section supersedes what it contradicts; the text above is kept unedited so the error is legible rather than erased.

**The four standing exclusions are NOT one cause.** `.github/workflows/windows.yml:90-92` — the very block cited as evidence — enumerates for `test_bee_cli.mjs`: *"linked-worktree reverse gitdir pointer mismatch on C:\ paths, **plus doctor-wording/recovery-window/registry-example failures**"*. Four failure classes; this slice addresses one. The backlog row behind it says the same. `test_bee_write_guard_hook.mjs` and `test_misc.mjs` have no recorded cause at all — attributing them to path identity was assertion, not finding.

**A second, independent Windows cause exists.** `packages/bee/tests/test_cells.mjs:1634-1639` skips its write-failure simulation when `process.geteuid?.() === 0`. On win32 `geteuid` is undefined, so the skip never fires, while Node's `chmod` on Windows only sets the read-only attribute and does not stop entry creation in a directory — the forced write succeeds and the assertion fails. That is a permission-model difference, not a path-string one.

**The local-pass gate was vacuous.** All four excluded suites pass on this Linux machine — measured. They were excluded *because Windows differs*, so a Linux pass is definitionally not evidence about them. Removing exclusions on that basis would have shipped both known-red causes into CI as surprise-red.

**Consequently the exclusions are NOT removed in this slice.** What ships instead: the product fix for the one cause that is real and diagnosed, the two test-side corrections, and the workflow's two broken step paths. Each remaining Windows cause is filed with what is actually known about it. Removing an exclusion is gated on a Windows CI run showing that suite green — evidence this machine cannot produce.

**Four further corrections to the approach:**

1. **Identity by device+inode is unsafe as stated.** Node reports `ino = 0` on filesystems with no file index (FAT/exFAT, many SMB/UNC shares, historically directories) — two distinct directories then compare EQUAL, which is precisely the "wrong pointer accepted" failure the fix exists to prevent. Treat a zero inode as unusable and fall back to the string comparison. Hardlinks and junctions also compare EQUAL by identity while being distinct paths.
2. **Case sensitivity is per-volume, not per-platform.** APFS can be either; Windows supports per-directory case sensitivity (WSL sets it); Linux can mount case-insensitive. A `platform === 'win32'` fold is itself the "EQUAL for two paths that must stay distinct" bug this plan's own risk map warns about. Detect the behaviour of the volume in play rather than assuming from the platform.
3. **`worktree-store.mjs:491` is not a comparison.** `readWorktreeGitVerifiedId` extracts an id and returns `path.basename(gitdir)`, which is git's registry key and must keep git's exact spelling. Only `:1110` is a comparison. Canonicalizing at `:491` would be a category error and case-folding there actively harmful.
4. **The herding resolvers must not share a helper, and their returned value must not change.** `packages/bee/lib/herding.mjs:26-41` records that the interlock script is standalone with zero bee dependencies, that its whole purpose is byte-for-byte agreement with its twin, and that calling into it from bee automation is forbidden in either direction. Separately, `skills/bee-herding/SKILL.md:142,303` derives the worktree grant key by string surgery on `main_root`'s basename — canonicalizing the returned value would change that key. The safe shape is: canonicalize **for comparison**, never mutate what is returned. Also: `.claude/skills/...` is an onboard-managed mirror; the source is `skills/...`, and an edit to the mirror is reverted by this slice's own regen chain while the release-manifest check stays green because that mirror is not hashed.

**Fifth broken workflow path found during validation:** `.github/workflows/windows.yml:117` runs `node scripts/test_config_validate.mjs`; the file exists only under `scripts/tests/`. Same defect as the guard path at `:114`. Both are fixed here.

**And every cell's verify was green on the unmodified tree** — measured. A verify that cannot distinguish "fix landed" from "nothing done" is not a proof surface. Each cell now names its own new test file explicitly, and the workflow cell gains an assertion over the workflow file itself, since nothing in the repo currently validates that a workflow step's script path exists.

## CORRECTION 2 (2026-07-27, after wpi-2's goal check) — item 4's reason was wrong; the rule survives on a different one

Correction item 4 above justified "never canonicalize `main_root`'s returned value" by `skills/bee-herding/SKILL.md`'s grant-key derivation. That reason points at the wrong function: the skill doc takes `main_root` from `bee worktree list --json`, which resolves through `resolveMainRoot`/`resolveRoots`, not through `herding.mjs`'s `resolveHerdingMainRoot` — whose only consumers are herding enable, disable and status. Verified independently twice, by the executing worker and again by the goal-check judge.

The prohibition stands, on this reason instead: `packages/bee/tests/test_herding_cli.mjs` asserts that bee's resolver and the standalone interlock script agree byte-for-byte on the marker path, and `packages/bee/lib/herding.mjs:26-41` forbids sharing a helper into that script. Canonicalizing only bee's side would break the agreement the interlock exists to guarantee.

Recorded rather than quietly amended, because a rule resting on a wrong reason binds the next reader to the wrong constraint.

## CORRECTION 3 (2026-07-27, after wpi-2's third round) — a limit, not a defect

The goal-check judge found that reverting wpi-2's corrected assertions to their pre-fix `===` form left both suites green, and marked the finding "authority" — a decision to be made, not an automatic fix. The orchestrator overrode that and directed a third round: run each assertion a second time against a win32-rendered value, so a revert would become visible.

The worker implemented exactly that. The orchestrator then measured it: reverting the original assertion **still passes**, because the second pass is a separate line the revert never touches. On POSIX the two comparison forms agree by construction for these values, so no local test can distinguish them — the property the third round chased does not exist on this platform.

The judge was right; the override was wrong, and it cost a round.

The second passes stay, because what they prove is real and is the property actually under test: the canonical comparison **accepts** a win32-rendered value that a bare `===` **rejects**. What no local proof can establish is that the assertion lines themselves use the canonical form. That is established by reading them, and confirmed by a Windows run.
