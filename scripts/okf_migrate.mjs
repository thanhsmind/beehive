#!/usr/bin/env node
// okf_migrate.mjs — migration tooling for the OKF knowledge bundle
// (okf-foundation cell okf-5, D20/D24/D29/D33/D35/D37; rebuilt on
// content-addressed pins by okf-migration-f2 cell f2-1b, decision F8).
//
// Bee-repo tooling ONLY — lives in scripts/, never shipped to hosts (D24).
// The authoring itself is agent work, not script magic: this script
// INVENTORIES a source BA spec's numbered anchors and VERIFIES a finished
// migration's coverage; it never writes a concept, a stub, or anything else.
//
// ─── the property this file exists to hold (F8) ─────────────────────────────
//
//   NO EXTRACTION RESULT MAY READ AS A PASS UNLESS IT WAS POSITIVELY VERIFIED.
//
// The first version of this gate compared a hand-authored ANCHOR_REGISTRY
// constant against a hand-authored stub map and hand-authored concept claims:
// internal consistency dressed as coverage (promoted as a critical pattern —
// docs/knowledge/patterns/20260722-a-coverage-gate-derives-ground-truth-*.md).
// The advisor consult for okf-migration-f2 then MEASURED a second, worse
// defect: the extractor is format-blind. docs/specs/onboarding.md's 22 rules,
// written `- **R1** — …`, inventoried as R0 and its unnumbered behaviors as
// B0, so *lost* content read as content that never existed. Reproducing 26
// and 47 proved nothing — advisor-protocol is the very file those regexes
// were written against, so the test passed by construction.
//
// The fix has four parts, and every one of them is asserted at check time:
//
//   1. PINS. Ground truth comes from a content-addressed pin
//      {commit, path, blob_sha, scheme, expected_counts} — see PIN_REGISTRY.
//      All of it is asserted: the blob is resolved, its sha verified, the
//      declared scheme applied, and the derived counts compared against
//      expected_counts. A count MISMATCH is a loud typed failure, not just an
//      empty set. An area with no declared scheme is REFUSED, never passed
//      0/0.
//   2. SHALLOW-CLONE FALLBACK. `git show <sha>:<path>` fails outright in a
//      `--depth 1` clone, so each pinned source is also committed verbatim
//      under docs/history/okf-migration-f2/sources/<area>.md and verified
//      with `git hash-object` against blob_sha. Missing BOTH is exit 1 —
//      never a skip.
//   3. UNPARSED-LINE REPORTING. Every inventory reports how many
//      block-starting lines it did not classify, per section, plus the raw
//      unclassified-line count. This is the visibility that makes
//      format-blindness detectable at all — and it is what f2-3 used to
//      MEASURE the blindness instead of forcing a scheme around it: it found
//      doctrine-layer reporting more unparsed blocks (21) than derived anchors
//      (20). f2-4 then widened the classifier to the id forms the other areas
//      actually use (see "the id forms" below). `--inventory docs/specs/
//      onboarding.md` MUST still report a non-zero unparsed count — its 20
//      unnumbered bold-lead behaviour paragraphs carry no id and are reported,
//      never invented. A clean parse there would mean the extractor had
//      started fabricating structure.
//   4. SCHEME AWARENESS. Three schemes exist today — `ba-nine-section`
//      (B*/R*/E*/P*), `flat-pattern-list` (PAT*), and `narrative-sections`
//      (S-<slugified heading>, f2-11). The shape is declared per area in the
//      pin and the extractor dispatches on it. An area whose shape has not
//      been decided carries `scheme: null` and a refusal reason, and the gate
//      refuses it BY NAME rather than passing it 0/0. Every registered area
//      declares a scheme today; the refusal path stays asserted for the next
//      area that arrives without one.
//
// Subcommands:
//   --inventory <path>       Parse a file with the nine-section BA scheme and
//                            emit its anchor inventory + unparsed report. JSON
//                            to stdout. Diagnostic; asserts nothing.
//   --inventory-pin <area>   Same, but against the area's PINNED blob, with
//                            every pin assertion applied.
//   --derive <pin-json>      Assert an ad-hoc pin object (the authoring aid
//                            for a new area's pin, and the gate's own test
//                            surface). Exit 0 green, exit 1 with typed codes.
//   --verify-pins            Assert every pin in PIN_REGISTRY.
//   --check <area>           Coverage check (D35), chain-failing. Asserts
//                            set-equality across three sets for the area:
//                              (1) the anchors DERIVED from the area's pinned
//                                  pre-migration blob (the "no loss" baseline
//                                  that survives the source becoming a stub),
//                              (2) the anchors recorded in the pointer stub's
//                                  anchor map (docs/specs/<area>.md, D37),
//                              (3) the anchors claimed by concepts under
//                                  docs/knowledge/areas/<area>/ via bee.sources
//                                  entries of the form
//                                  "docs/specs/<area>.md#<ANCHOR>".
//                            Every anchor must be owned by EXACTLY ONE concept
//                            (no loss, no duplication), the stub map's target
//                            path must be the claiming concept's own path, and
//                            that file must exist. Exit 0 with counts on green,
//                            exit 1 naming every violation on red.
//   --check-patterns         The same coverage law for critical-patterns.md's
//                            migration into docs/knowledge/patterns/ (okf-6),
//                            against the flat-pattern-list scheme.
//   --fidelity <area>        The F11 per-anchor overlap table for one area.
//                            Diagnostic view of a guard --check already runs.
//   --telemetry              The F12 shape ratios for every pinned area, plus
//                            the whole-bundle invariants.
//
// No --strict flag exists here on purpose: the check is already binary.
//
// ─── what f2-2 added (F11/F12, and the f2-1b judge's residual gap) ──────────
//
// f2-1b made the ground truth DERIVED. That closes "was this anchor claimed?"
// but not "was it actually carried across?" — a concept that summarises its
// anchor into one sentence still satisfies every count, which is precisely the
// degradation mode of a long serial re-authoring run. Three additions:
//
//   F11 FIDELITY FLOOR. Every anchor's text in the pinned blob must survive in
//       the body of the concept that claims it, at >= 0.60 normalized token
//       overlap. Below the floor names the anchor, its owner, the ratio, and
//       the missing tokens. The tuning rule is inverted from the usual
//       instinct and is enforced by the suite: both shipped areas clear it
//       UNEDITED — a failure means the NORMALIZATION is wrong, never the
//       migrated content, and never the threshold.
//   F12 DRIFT TELEMETRY. Per area, anchors_per_concept and
//       concepts_per_100_source_lines, compared against the running median of
//       the pinned areas outside a [0.5x, 2x] band. Fewer than three pinned
//       areas is no median at all, so it reports and never fails. Plus the
//       whole-bundle invariants — authority uniqueness, zero not_canonical,
//       index freshness — on EVERY check.
//   MANDATORY unparsed_blocks. A pin omitting expected_counts.unparsed_blocks
//       is PIN_INCOMPLETE. `total` only asserts the counts still add up;
//       unparsed_blocks is what asserts the extractor still SEES the same
//       shape, and opting out of that guard silently was still possible.

import { spawnSync } from "node:child_process";
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "..");

// ─── the pin registry (F8) — the ONLY ground truth ──────────────────────────
// Replaces the hand-authored ANCHOR_REGISTRY / PATTERNS_ANCHOR_REGISTRY
// constants. Nothing here is an anchor list: a pin says *where the truth
// lives* and *what shape it has*, and the anchors are derived from the pinned
// bytes on every run.
//
// A pin's fields, all asserted (derivePin):
//   commit          the commit whose tree holds the pre-migration source
//   path            that source's repo-relative path at that commit
//   blob_sha        the exact blob id — the content address; a differing blob
//                   at commit:path, or a committed copy that hashes to
//                   anything else, is PIN_SHA_MISMATCH
//   scheme          the anchor scheme to extract with (null = undecided,
//                   refused by name)
//   expected_counts the counts the extraction MUST produce, `total` required;
//                   any declared key that differs is PIN_COUNT_MISMATCH
//   source_copy     the verbatim committed copy used when git cannot resolve
//                   the blob (a --depth 1 clone); missing it while git works
//                   is PIN_COPY_MISSING, missing BOTH is PIN_UNRESOLVED
//   repaired_from   (f2-10, optional — mandatory TOGETHER with repair_reason)
//   repair_reason   the provenance blob at commit:path, plus why the pinned
//                   bytes differ from it. Only for a source that could not be
//                   pinned as it stood: hook-runtime carried the rule id `R14`
//                   twice, and a duplicate id is unmigratable by construction
//                   (anchors are keyed by id, so the first R14's text was
//                   overwritten by the second and unmeasurable by the fidelity
//                   floor, and set-equality cannot see a pair's second member).
//                   The repair is minimal and made BEFORE the pin is captured,
//                   so the pinned blob already carries distinct ids. commit:path
//                   then addresses the PROVENANCE — asserted exactly, so
//                   drifting provenance stays as loud as a drifting pin — and
//                   the committed copy is the pinned bytes' only content
//                   address. An undeclared disagreement is never read as a
//                   repair; it is PIN_SHA_MISMATCH exactly as before.
//
// How the two shipped pins were chosen: each area's source became a pointer
// stub in the commit that migrated it, so the pin is the commit BEFORE that
// one — advisor-protocol: a0ea0cc^ = 19c0e50 (okf-4); critical-patterns:
// b0d495d^ = a0ea0cc (okf-5).
export const PIN_REGISTRY = {
  "advisor-protocol": {
    kind: "area",
    commit: "19c0e50d6d9464a3dfb0b5aa56a97bb1b53aade9",
    path: "docs/specs/advisor-protocol.md",
    blob_sha: "f3f123173726517c6a5068fd07d2b6c048a94043",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 4,
      rules: 9,
      edges: 6,
      pointers: 7,
      total: 26,
      // The pinned blob is a clean nine-section spec: every block-starting
      // line in an anchor-bearing section IS an anchor. Asserting 0 here is
      // what stops a future extractor regression from silently reclassifying
      // anchors as prose while the totals happen to still add up.
      unparsed_blocks: 0,
    },
    source_copy: "docs/history/okf-migration-f2/sources/advisor-protocol.md",
    note: "the 202-line BA spec as of okf-4, immediately before okf-5 (a0ea0cc) turned it into a D37 pointer stub",
  },
  "doctrine-layer": {
    kind: "area",
    // The pin is HEAD at the moment f2-3 ran, taken BEFORE the stub replaced
    // the content — the same "the commit before the migrating commit" rule the
    // two shipped pins follow, expressed the only way a cell can express it
    // about its own commit: pin the parent, copy the bytes, hash both.
    commit: "ed65720b726333ce6fafe5a9470e1d6c490e04c7",
    path: "docs/specs/doctrine-layer.md",
    blob_sha: "351bf72adc2c34f9f18d9f3612735a6c92efac02",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 10,
      rules: 17,
      edges: 5,
      pointers: 7,
      total: 39,
      // NOT zero, and deliberately pinned at what it really is. Two block
      // starts in this source carry no id and none is invented for them (D10):
      // a bold-lead continuation line inside B8 (`**for unit execution** …`,
      // L176) and the unnumbered `- **The verify ladder …**` bullet at L305.
      // Both are unparsed for ID purposes and BOTH still travel with the
      // anchor whose block they sit in — B8 and R17 respectively — so the
      // fidelity floor measures them and no content is homeless. Asserting 2
      // here is what makes a future classifier change that swallows or splits
      // them a loud failure instead of a silent re-shaping.
      unparsed_blocks: 2,
    },
    source_copy: "docs/history/okf-migration-f2/sources/doctrine-layer.md",
    note: "the 386-line BA spec as of ed65720 (f2-4, the classifier widening), immediately before f2-3 turned it into a D37 pointer stub — the cleanest conforming area of the nine",
  },
  "critical-patterns": {
    kind: "patterns",
    commit: "a0ea0cc40bf199192c40d425cdc11f50d4b943e5",
    path: "docs/history/learnings/critical-patterns.md",
    blob_sha: "2bf112090761cb8d1b2fbc63c897b525ac0f3b9f",
    scheme: "flat-pattern-list",
    expected_counts: { patterns: 47, total: 47, unparsed_blocks: 0 },
    source_copy: "docs/history/okf-migration-f2/sources/critical-patterns.md",
    note: "the 703-line flat dated pattern list as of okf-5, immediately before okf-6 (b0d495d) migrated it into docs/knowledge/patterns/",
  },

  "decision-memory": {
    kind: "area",
    // The pin is HEAD at the moment f2-5 ran, taken BEFORE the stub replaced
    // the content — same rule as doctrine-layer's pin: pin the parent, copy
    // the bytes, hash both.
    commit: "8710d03afa1d0253124fbbb081fde47ad5b3209e",
    path: "docs/specs/decision-memory.md",
    blob_sha: "2e8ec59cfa540827cc568cfefcb5f81bebe2d822",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 0,
      rules: 9,
      edges: 0,
      pointers: 0,
      total: 9,
      // f2-4 widened the id forms and this area's nine rules — written
      // `- **R1 — …**` — were derivable all along; "shapeless" was a verdict
      // about the reader, not the file. Zero unparsed blocks: the cleanest
      // area of the nine (no Behaviors/Edge Cases/Pointers sections exist in
      // the source at all, and every Business Rules bullet is a numbered
      // block start the classifier can already see).
      unparsed_blocks: 0,
    },
    source_copy: "docs/history/okf-migration-f2/sources/decision-memory.md",
    note: "the 39-line BA spec as of 8710d03 (f2-3), immediately before f2-5 turned it into a D37 pointer stub — nine business rules, no behaviors/edges/pointers sections",
  },

  "verify-pipeline": {
    kind: "area",
    // The pin is HEAD at the moment f2-6 ran, taken BEFORE the stub replaced
    // the content — same rule as doctrine-layer's and decision-memory's pins:
    // pin the parent, copy the bytes, hash both.
    commit: "72fd8284fa1e1303d026eb4f94e31fed07a30542",
    path: "docs/specs/verify-pipeline.md",
    blob_sha: "eab70d7e665ec73a5a8dc227bee5c16ada74dbe8",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 0,
      rules: 5,
      edges: 4,
      pointers: 5,
      total: 14,
      // The source's "Behaviors & Operations" section carries 7 bold-lead
      // bullets with no B-id at all — none is invented into an anchor (D10).
      // Every Business Rules / Edge Cases / Pointers bullet IS a classified
      // block start, so the 7 unparsed blocks are entirely those unnumbered
      // behaviors, confirmed by --inventory before any edit.
      unparsed_blocks: 7,
    },
    source_copy: "docs/history/okf-migration-f2/sources/verify-pipeline.md",
    note: "the 132-line BA spec as of 72fd828 (f2-5), immediately before f2-6 turned it into a D37 pointer stub — 5 rules, 4 edge cases, 5 pointers, and 7 unnumbered behavior bullets",
  },

  "performance-log": {
    kind: "area",
    // The pin is HEAD at the moment f2-7 ran, taken BEFORE the stub replaced
    // the content — same rule as doctrine-layer's, decision-memory's, and
    // verify-pipeline's pins: pin the parent, copy the bytes, hash both.
    commit: "46a56a4172eea71958728fac67b6436d7e8cdd11",
    path: "docs/specs/performance-log.md",
    blob_sha: "efdc9f2149cb6ae64f0a4f4a416de539dab82d93",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 0,
      rules: 11,
      edges: 5,
      pointers: 7,
      total: 23,
      // The source's entire "Behaviors & Operations" section — 7 bold-lead
      // paragraphs (Opening/Closing/One-shot/Reading-rendering/Populating the
      // store/Building the matrix/Measurement rules) plus 3 of Measurement
      // rules' own un-ided sub-bullets — carries no B-id at all, so none of
      // it is invented into an anchor (D10). Every Business Rules bullet is a
      // `- **Rn — …**` block start the classifier already sees, and every
      // Edge Cases / Pointers bullet is a top-level `- ` block start, so the
      // 10 unparsed blocks are entirely those Behaviors & Operations blocks,
      // confirmed by --inventory before any edit.
      unparsed_blocks: 10,
    },
    source_copy: "docs/history/okf-migration-f2/sources/performance-log.md",
    note: "the 225-line BA spec as of 46a56a4 (f2-6), immediately before f2-7 turned it into a D37 pointer stub — 11 rules, 5 edge cases, 7 pointers, and 10 unparsed Behaviors & Operations blocks",
  },

  "feedback-digest": {
    kind: "area",
    // The pin is HEAD at the moment f2-8 ran, taken BEFORE the stub replaced
    // the content — same rule as doctrine-layer's, decision-memory's,
    // verify-pipeline's, and performance-log's pins: pin the parent, copy the
    // bytes, hash both.
    commit: "3d69a2d3baf00917e578144f7b8f500baf02a4ca",
    path: "docs/specs/feedback-digest.md",
    blob_sha: "eeb447ee3e9a274935a94d8d0633b41e7107730b",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 0,
      rules: 15,
      edges: 6,
      pointers: 8,
      total: 29,
      // The source's entire "Behaviors & Operations" section (B1-B5) is five
      // markdown SUBHEADINGS, not bullets, so none of it is a classified
      // block start at all — 18 unnumbered bold-lead paragraphs
      // (Triggers/What is read/What is emitted/What blocks it/What each
      // actor observes/Reproducibility/How the boundary is
      // enforced/Why the reader distrusts the writer/What it does) plus 8
      // un-ided continuation bullets (B2's re-examination list, B4's
      // grouping/score/floor bullets) — none is invented into an anchor
      // (D10). Every Business Rules bullet is a `- **Rn** (…) — …` block
      // start the classifier already sees; Edge Cases and Pointers carry no
      // explicit id at all in the source (unlike Business Rules) but every
      // bullet there IS still a top-level `- ` block start, so the
      // classifier derives E1-E6 and P1-P8 from bullet order. The 26
      // unparsed blocks are entirely the Behaviors & Operations prose,
      // confirmed by --inventory before any edit. This is the highest
      // unparsed ratio of the five pinned areas so far — roughly half this
      // area's substance lives in unnumbered prose, so the coverage gate
      // governs less of it than it does elsewhere.
      unparsed_blocks: 26,
    },
    source_copy: "docs/history/okf-migration-f2/sources/feedback-digest.md",
    note: "the 355-line BA spec as of 3d69a2d (f2-7), immediately before f2-8 turned it into a D37 pointer stub — 15 rules, 6 edge cases, 8 pointers, and 26 unparsed Behaviors & Operations blocks",
  },

  onboarding: {
    kind: "area",
    // The pin is HEAD at the moment f2-9 ran, taken BEFORE the stub replaced
    // the content — same rule as every pin before it: pin the parent, copy the
    // bytes, hash both.
    commit: "a06f59d9ce4cd2884ce2072e126166945461731e",
    path: "docs/specs/onboarding.md",
    blob_sha: "c78ca9b40a5c24715b99870df17fe08800522a91",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 0,
      // 28, not 27: R20b is LETTER-SUFFIXED. It is the id f2-4's widening was
      // written for, and the reason the stub-row parser and the bee.sources
      // claim matcher were widened alongside it in f2-3 — a derived `R20b`
      // that no stub row and no concept claim can match would read as LOST
      // however faithfully it was migrated.
      rules: 28,
      edges: 15,
      pointers: 15,
      total: 58,
      // The largest area pinned so far, and its whole "Behaviors &
      // Operations" section carries no ids at all: 17 unnumbered bold-lead
      // paragraphs (Detect / Vendor / Heal drift / Stay out / Context colour /
      // Manage the ignore section / Warn on already-tracked silenced paths /
      // Select and prove exactly one distribution source / An opt-in is
      // remembered / Install skills into the project itself / Provide the
      // assistant-instructions import / Retire superseded helper scripts /
      // Fetch the workflow source without a full working tree / Wire the
      // second-runtime guards / Guarantee the second runtime's status display
      // / Guarantee the state-layer landing pages), the "What the status
      // display renders" lead paragraph, and 3 un-ided continuation bullets
      // (the ignore section's three exhaustive cases) — 20 blocks, none of
      // them invented into an anchor (D10). Every Business Rules bullet is a
      // `- **Rn** — …` block start the classifier sees; Edge Cases and
      // Pointers carry no explicit id in the source but every bullet there IS
      // a top-level `- ` block start, so E1-E15 and P1-P15 are derived from
      // bullet order. Confirmed by --inventory before any edit.
      unparsed_blocks: 20,
    },
    source_copy: "docs/history/okf-migration-f2/sources/onboarding.md",
    note: "the 689-line BA spec as of a06f59d (f2-8 close), immediately before f2-9 turned it into a D37 pointer stub — 28 rules (R20b included), 15 edge cases, 15 pointers, and 20 unparsed Behaviors & Operations blocks",
  },

  "hook-runtime": {
    kind: "area",
    // The pin is HEAD at the moment f2-10 ran — but, uniquely so far, the
    // pinned BYTES are not HEAD's bytes: this source could not be pinned as it
    // stood. See repaired_from/repair_reason below and the resolvePinnedSource
    // note. The provenance blob is still asserted exactly; the repaired bytes
    // are content-addressed by the committed copy.
    commit: "ab8cf6ec864b121fb16d33ddfd2250093a4f3eef",
    path: "docs/specs/hook-runtime.md",
    blob_sha: "a8907ce092fe0cc6a8a07d28b8796e36adc83ded",
    repaired_from: "83001d724c95e80b072e589c6b94936b43db919d",
    repair_reason:
      "the source carried the rule id R14 TWICE — the gate-bypass block-verdict rule (L450, added by 1f3d25c for GitHub #18) and the write-guard dual command-shape rule (L477, added by 1ef6fb6 for shim-retire D3/bbc6bcea). They are two genuinely distinct rules, not one rule stated twice, and the f2-4 sweep flagged the collision as pre-existing. Anchors are keyed by id in a Map, so the FIRST R14's text was silently overwritten by the second and was unmeasurable by the fidelity floor forever, while set-equality could not see the pair's second member at all. Neither rule may be dropped or merged to remove the duplicate, so the repair is the minimum that makes both individually measurable: the SECOND occurrence in document order (the write-guard rule) is renumbered R14a, one token on one line, no other byte changed. The gate-bypass rule KEEPS R14 because every live citation of `hook-runtime R14` means it — this spec's own R4, R10 and gate-bypass pointer, skills/bee-hive/references/routing-and-contracts.md, and decision 4c1c5921 — so the repair leaves every existing reference resolving correctly and re-homes only the id that had no external reader. R14a is a DISAMBIGUATION suffix, not a refinement suffix like R8a/R8b; the pointer stub's anchor map says so, so a reader arriving at either old id finds the rule it meant.",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 22,
      rules: 24,
      edges: 17,
      pointers: 18,
      // 81 anchors, and — after the repair — 81 DISTINCT ids. Before it, the
      // same 81 array members carried only 80 distinct ids, which is exactly
      // why counts alone never caught this: array length is blind to a
      // collision the Map silently collapses.
      total: 81,
      // The largest area pinned so far and the LOWEST unparsed ratio of the
      // eight: this spec numbers nearly everything. All 8 unparsed blocks sit
      // in "Behaviors & Operations" and none is invented into an anchor (D10):
      // B2's wrapped continuation line that happens to open with a bold run
      // (`**one** deliberate exception is the gate-bypass net (B15)…`, L78),
      // B3's three un-ided outcome bullets (all targets provable / intercepted
      // but not provable / outer event malformed, L86-L92), and B16's four
      // un-ided case bullets (Both present / Choice only / Tier only /
      // Neither, L243-L268). Every one of them travels with the anchor whose
      // block it sits in — B2, B3 and B16 — so the fidelity floor measures
      // them and no content is homeless. Every Business Rules bullet is a
      // `- Rn — …` block start the classifier sees (R8a, R8b and the repaired
      // R14a included); Edge Cases and Pointers carry no explicit id in the
      // source but every bullet there IS a top-level `- ` block start, so
      // E1-E17 and P1-P18 are derived from bullet order. Confirmed by
      // --inventory before and after the repair: 81/8 both times.
      unparsed_blocks: 8,
    },
    source_copy: "docs/history/okf-migration-f2/sources/hook-runtime.md",
    note: "the 762-line BA spec as of ab8cf6e (f2-9 close) WITH the duplicate-R14 repair applied by f2-10 before the pin was captured — 22 behaviors, 24 rules (R8a, R8b and the disambiguated R14a included), 17 edge cases, 18 pointers, 8 unparsed Behaviors & Operations blocks. These bytes are in no commit's tree by construction, so `via` is always committed-copy here",
  },

  "worktree-parallelism": {
    kind: "area",
    // The pin is HEAD at the moment f2-11 ran, taken BEFORE the stub replaced
    // the content — same rule as every unrepaired pin before it: pin the
    // parent, copy the bytes, hash both.
    commit: "687ac5909add9f3fdfa575deeddbed1d4b5f8398",
    path: "docs/specs/worktree-parallelism.md",
    blob_sha: "df2f441bb88e9632a1f95d288331a3e356c68e19",
    // THE ONE AREA OF THE ELEVEN THAT NEEDED A THIRD SCHEME. Every previous
    // "shapeless" verdict turned out to be a blind reader: decision-memory's
    // nine rules were written `- **R1 — …**` and f2-4's widening found them.
    // This file is the real thing. It carries no `## Behaviors`, `## Business
    // Rules`, `## Edge Cases Settled` or `## Pointers` section, and not one
    // `B*`/`R*`/`E*`/`P*` id anywhere in its 225 lines — so `ba-nine-section`
    // derives 0 anchors AND 0 unparsed blocks, which is what genuine
    // shapelessness looks like. F9 forbids forcing it into the nine-section
    // shape and D10 forbids inventing numbered ids the source never had, so
    // the anchors are the source's OWN `## ` headings, slugified: the
    // structure the author actually wrote IS the ground truth. This mirrors
    // `flat-pattern-list`, which already treats a `## [YYYYMMDD] …` heading as
    // an anchor.
    scheme: "narrative-sections",
    expected_counts: {
      // Ten narrative sections, in document order: What problem this solves /
      // The trust model / Registering a worktree / Entering / Returning /
      // Routing rule / Cross-worktree holds / The three tiers / Boundary /
      // Where it lives.
      sections: 10,
      total: 10,
      // NOT zero, and pinned at what it really is. The source's first ten
      // lines sit BEFORE the first `## ` heading — the document title and the
      // `**Area:**` / `**Status:**` metadata block — so they belong to no
      // section and no anchor can own them. Two of those lines are block
      // starts, and neither is invented into an anchor (D10): they are
      // reported, exactly as the BA scheme reports its unnumbered bold-lead
      // paragraphs. This file carries no `###` subheading at all, so all 2
      // unparsed blocks are preamble. Asserting 2 here is what makes a future
      // change that swallows the preamble — or that starts promoting
      // subheadings to anchors — a loud failure instead of a silent reshaping.
      unparsed_blocks: 2,
    },
    source_copy: "docs/history/okf-migration-f2/sources/worktree-parallelism.md",
    note: "the 225-line narrative area spec as of 687ac59 (f2-10 close), immediately before f2-11 turned it into a D37 pointer stub — 10 `## ` heading anchors, 0 numbered ids of any kind, 2 unparsed preamble blocks; the only area of the eleven that needed a scheme of its own",
  },

  "workflow-state": {
    kind: "area",
    // THE SECOND REPAIRED PIN (f2-12), and the largest source of the eleven.
    // The pinned BYTES are not HEAD's bytes: this source could not be pinned as
    // it stood either. Same branch as hook-runtime's — the provenance blob at
    // commit:path is still asserted exactly, and the committed copy is the
    // repaired bytes' only content address.
    commit: "df3072d561107443c4491bba36d7be0fa28adedb",
    path: "docs/specs/workflow-state.md",
    blob_sha: "506fef94814b9197706c6dcab9a09e56941609d4",
    repaired_from: "ed1644c487043b0373b0ceea727143d4ddbfa3e8",
    repair_reason:
      "the source carried THREE duplicated rule ids — R19, R20 and R21 each appeared twice inside one `## Business Rules` section, flagged as pre-existing by the f2-4 sweep. The first family (L891-902) is the fresh-session-handoff triple: R19 planned-next preconditions live in the verb, R20 auto-resume authority exists only at the fresh-session boundary, R21 the work puller never widens authority (fresh-session-handoff D1/D2, validation-s4 C10/C11). The second family (L916-930) is the chain-integrity triple: R19 the learning-capture phase is never settable, R20 recording a knowledge sync demands executed work, R21 the terminal state demands learning capture plus zero spec debt (chain-integrity D1-REVISED/D2/D3/D4). Six genuinely distinct rules, three collisions, none a rule stated twice. Anchors are keyed by id, so each first member's text was silently overwritten by its second and unmeasurable by the F11 floor forever, while set-equality could not see any pair's second member — 140 anchors carrying only 137 distinct ids. No rule may be dropped or merged, so the repair is the minimum that makes all six individually measurable: the SECOND occurrence of each id in document order (the chain-integrity family) is renumbered R19a/R20a/R21a — three tokens on three lines, no other byte changed. The chain-integrity family was chosen as the renumbered side because BOTH sides carry ZERO live citations (`grep skills/ scripts/ hooks/ .bee/bin/ AGENTS.md docs/specs/` finds no citation of any workflow-state rule id at all, in either direction — no `workflow-state.md#Rnn` reference exists anywhere yet), so no reference is churned whichever side moves; the tie is broken exactly as hook-runtime's R14 was, by renumbering the second occurrence in document order, and the only surviving external mention — docs/history/fresh-session-handoff/reports/validation-s5.md, which cites `workflow-state.md B15/B16/R19-R21` — means the FIRST family, so it keeps resolving correctly. R19a/R20a/R21a are DISAMBIGUATION suffixes, not refinement suffixes like R8a/R8b; the pointer stub's anchor map will say so, so a reader arriving at either old id finds the rule it meant.",
    scheme: "ba-nine-section",
    expected_counts: {
      behaviors: 37,
      rules: 58,
      edges: 25,
      pointers: 20,
      // 140 anchors, and — after the repair — 140 DISTINCT ids. Before it, the
      // same 140 array members carried only 137 distinct ids: three collisions
      // that array-length counting is blind to by construction.
      total: 140,
      // All 7 unparsed blocks sit in "Behaviors & Operations" and none is
      // invented into an anchor (D10). Two are wrapped continuation lines that
      // happen to open with a bold run — B9a's `**read-only** with an evidence
      // bundle …` (L214) and B16's `**actively owned by another live session**
      // …` (L356). The other five are the un-ided bold-lead paragraphs of the
      // `### Closing a feature — the tail of the chain` subsection (L547, L553,
      // L558, L564, L577), which a `###` heading does not close, so they travel
      // inside B24's block. Every one of them travels with the anchor whose
      // block it sits in, so the fidelity floor measures them and no content is
      // homeless.
      unparsed_blocks: 7,
    },
    source_copy: "docs/history/okf-migration-f2/sources/workflow-state.md",
    note: "the 1464-line BA spec as of df3072d (f2-11 close) WITH the duplicate-R19/R20/R21 repair applied by f2-12 before the pin was captured — 37 behaviors, 58 rules (B9a and the disambiguated R19a/R20a/R21a included), 25 edge cases, 20 pointers, 7 unparsed Behaviors & Operations blocks. These bytes are in no commit's tree by construction, so `via` is always committed-copy here. The area is migrated across SEVERAL cells (F10), so its chain gate is wired by the LAST of them, not by f2-12",
  },

  "okf-profile": {
    kind: "area",
    // THE TWELFTH AND LAST AREA (f3-5, G6) — and the only one whose source
    // DEFINES the bundle it is moving into. The pin is HEAD at the moment f3-5
    // ran, taken BEFORE the stub replaced the content: pin the parent, copy the
    // bytes, hash both, exactly as every unrepaired pin before it. No repair was
    // needed — the source carries 24 DISTINCT ids and no collision.
    commit: "53d8111769c5c2fa9a479dd82a0ce5a9278b1448",
    path: "docs/specs/okf-profile.md",
    blob_sha: "9267d3eaef2cc016d32a96bca1e6d99822bca3c3",
    scheme: "ba-nine-section",
    expected_counts: {
      // 13, B6b included: this source uses the LETTER SUFFIX as a REFINEMENT
      // (B6b refines B6's budget cut with the relevance ranking), the same form
      // f2-4's widening was written for and f2-3 taught the stub-row parser and
      // the claim matcher to read.
      behaviors: 13,
      // ZERO, and measured rather than assumed. This source's `## Business
      // Rules` section carries NINE top-level bullets and not one of them
      // opens with an `R<n> —` id — they are written as plain prose bullets.
      // D10 forbids inventing the nine R-ids they never had, so `rules` is 0
      // and those nine blocks are counted in unparsed_blocks below. They are
      // still MIGRATED (verbatim, into the concept whose topic each states);
      // what they are not is anchor-gated, exactly as feedback-digest's
      // unnumbered behavior prose is not.
      rules: 0,
      edges: 4,
      pointers: 7,
      total: 24,
      // 17, and every one of them identified before any edit:
      //   • 5 in "Behaviors & Operations" — B6's wrapped continuation line that
      //     happens to open with a bold run (`**transitively** with a cycle
      //     guard …`, L217), B6b's three un-ided property bullets (`- **Floor.**`
      //     L249, `- **Conservation.**` L254, `- **Zero-signal guard.**` L259),
      //     and B10's wrapped `**86 anchors across five areas** …` (L302). Each
      //     travels with the anchor whose block it sits in — B6, B6b, B10 — so
      //     the fidelity floor measures them and no content is homeless.
      //   • 9 in "Business Rules" — the nine un-ided rule bullets described
      //     above (L351, L354, L356, L357, L359, L361, L363, L364, L370).
      //   • 3 accounted to "Pointers" that are really TEMPLATES prose, and this
      //     is the one hazard this source carries: the extractor does not track
      //     markdown CODE FENCES, and the `bee.area` template at L406-438
      //     contains a fenced nine-section skeleton whose `## Pointers
      //     (implementation)` line (L437) opens a spurious pointers accounting
      //     section that runs to the real `## Open Gaps` at L574. Three
      //     bold-lead paragraphs in that stretch (L440 the anti-fork field,
      //     L449 the body contract, L456 the rebuild bar) are counted there.
      //     It changes NO anchor: that stretch contains no top-level `- `
      //     bullet, so P1-P7 are still exactly the seven bullets of the real
      //     `## Pointers (implementation)` section at L594-611, verified by
      //     dumping each derived anchor's text before the pin was written.
      //     Asserting 17 is what makes a future fence-aware extractor — which
      //     would move these 3 out of the count — a loud failure instead of a
      //     silent reshaping.
      unparsed_blocks: 17,
    },
    source_copy: "docs/history/okf-migration-f2/sources/okf-profile.md",
    note: "the 611-line BA spec as of 53d8111 (f3-1..f3-4 close), immediately before f3-5 turned it into a D37 pointer stub — 13 behaviors (B6b included), 0 rules (the nine Business Rules bullets carry no R-id and none was invented), 4 edge cases, 7 pointers, 17 unparsed blocks. The spec that defines the profile, migrated by the profile's own loop and graded by the profile's own gate",
  },
};

const PATTERNS_SOURCE = "docs/history/learnings/critical-patterns.md";
const PATTERNS_CONCEPT_DIR = "docs/knowledge/patterns";

// ─── pin resolution ─────────────────────────────────────────────────────────

function git(args) {
  const r = spawnSync("git", args, {
    cwd: REPO_ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return { ok: r.status === 0, stdout: r.stdout || "", stderr: r.stderr || "", error: r.error };
}

/** git's own blob id for a byte string, computed in-process. Used as the
 *  no-git fallback for hashObject and as a cross-check: the two must agree,
 *  and accepting either keeps a checkout whose filters differ from CI from
 *  failing the gate for a reason that has nothing to do with coverage. */
function blobShaOf(buf) {
  const body = Buffer.isBuffer(buf) ? buf : Buffer.from(buf, "utf8");
  return crypto
    .createHash("sha1")
    .update(Buffer.concat([Buffer.from(`blob ${body.length}\0`, "utf8"), body]))
    .digest("hex");
}

/** The blob ids a working-tree file could legitimately carry: `git
 *  hash-object` (what git itself would store) and the in-process computation
 *  (raw bytes). Identical in this repo; both are returned so a filtered
 *  checkout or a missing git binary is not mistaken for content drift. */
export function hashObject(absPath) {
  const shas = new Set();
  const r = git(["hash-object", "--", absPath]);
  if (r.ok && /^[0-9a-f]{40}$/.test(r.stdout.trim())) shas.add(r.stdout.trim());
  try {
    shas.add(blobShaOf(fs.readFileSync(absPath)));
  } catch { /* the caller already checked existence */ }
  return [...shas];
}

function issue(code, message) {
  return { code, message };
}

/**
 * Resolve a pin's pinned bytes, asserting the content address on the way.
 * Order: the git object database first (`git rev-parse <commit>:<path>` —
 * which is exactly what fails in a `--depth 1` clone), then the committed
 * verbatim copy verified with `git hash-object` against blob_sha.
 *
 * Returns { ok, text, via, issues }. `via` is "git" or "committed-copy" —
 * never absent on a green result, so a caller can never mistake a skip for a
 * verification.
 */
export function resolvePinnedSource(pin) {
  const issues = [];
  let gitText = null;

  if (pin.commit && pin.path) {
    const rev = git(["rev-parse", "--verify", "--quiet", `${pin.commit}:${pin.path}`]);
    const resolved = rev.ok ? rev.stdout.trim() : null;
    if (resolved && /^[0-9a-f]{40}$/.test(resolved)) {
      if (resolved !== pin.blob_sha) {
        // ─── the ONE legitimate disagreement: a REPAIRED pin (f2-10) ───────
        // A source can carry a defect that makes it UNPINNABLE as it stands —
        // hook-runtime shipped the same rule id `R14` twice, and because
        // anchors are keyed by id, the first one's text was silently
        // overwritten by the second and became unmeasurable by the fidelity
        // floor forever, while set-equality could not see the pair's second
        // member at all. Migrating that is prohibited, and so is deleting or
        // merging a rule to make the collision go away; the only honest move
        // is a minimal repair of the SOURCE before the pin is captured.
        //
        // Those repaired bytes exist in no commit's tree, so the git leg can
        // no longer be the content address. It becomes the PROVENANCE
        // address instead — and it is still asserted EXACTLY: `repaired_from`
        // must equal what commit:path really resolves to, so a provenance
        // that drifts is exactly as loud as a pin that drifts. The pinned
        // bytes themselves are then addressed by the committed copy alone,
        // hash-verified against blob_sha below like every other pin.
        //
        // Both fields are mandatory together and neither is inferable: a
        // repaired_from that does not match, a missing repair_reason, or a
        // pin that simply drifted and never declared a repair at all, are all
        // still PIN_SHA_MISMATCH. This branch is dead code for every pin that
        // has not declared a repair — the no-op is the safety property here
        // exactly as it is for the classifier widening.
        const repairDeclared =
          typeof pin.repaired_from === "string" &&
          typeof pin.repair_reason === "string" &&
          pin.repair_reason.trim().length > 0;
        if (!repairDeclared || pin.repaired_from !== resolved) {
          issues.push(
            issue(
              "PIN_SHA_MISMATCH",
              `${pin.commit.slice(0, 8)}:${pin.path} resolves to blob ${resolved}, but the pin declares ${pin.blob_sha} — the pin and the history disagree about what the pinned source IS` +
                (repairDeclared
                  ? `. The pin declares a repair of blob ${pin.repaired_from}, which is NOT what ${pin.commit.slice(0, 8)}:${pin.path} resolves to — a repaired pin must name the exact provenance blob it was repaired from`
                  : `. If these bytes were deliberately repaired before pinning, the pin must SAY so: declare repaired_from (the provenance blob at commit:path) and repair_reason. An undeclared disagreement is never read as a repair`),
            ),
          );
          return { ok: false, text: null, via: null, issues };
        }
        if (!pin.source_copy) {
          issues.push(
            issue(
              "PIN_COPY_MISSING",
              `the pin declares a repair of ${pin.commit.slice(0, 8)}:${pin.path} (blob ${resolved} -> ${pin.blob_sha}) but no committed source copy. The repaired bytes are in no commit's tree, so the copy is the pin's ONLY content address — commit it (F8)`,
            ),
          );
          return { ok: false, text: null, via: null, issues };
        }
        // gitText stays null on purpose: the resolvable blob is the
        // PROVENANCE, never the pinned source. The committed copy below is.
      } else {
        const cat = git(["cat-file", "blob", resolved]);
        if (cat.ok) gitText = cat.stdout;
      }
    }
  }

  // The committed copy is verified whenever the pin declares one — including
  // when git already answered. A copy that has drifted from the blob is the
  // failure the fallback would otherwise hide the day a clone goes shallow.
  let copyText = null;
  if (pin.source_copy) {
    const abs = path.join(REPO_ROOT, ...pin.source_copy.split("/"));
    if (!fs.existsSync(abs)) {
      if (gitText !== null) {
        issues.push(
          issue(
            "PIN_COPY_MISSING",
            `the pinned source copy ${pin.source_copy} is missing. git can still resolve the blob here, but a --depth 1 clone cannot — commit the verbatim copy (F8)`,
          ),
        );
        return { ok: false, text: null, via: null, issues };
      }
      issues.push(
        issue(
          "PIN_UNRESOLVED",
          `neither git nor a committed copy can produce the pinned source: ${pin.commit ? `${pin.commit.slice(0, 8)}:${pin.path}` : "<no commit pinned>"} is unresolvable and ${pin.source_copy} does not exist. This is exit 1, never a skip`,
        ),
      );
      return { ok: false, text: null, via: null, issues };
    }
    const shas = hashObject(abs);
    if (!shas.includes(pin.blob_sha)) {
      issues.push(
        issue(
          "PIN_SHA_MISMATCH",
          `the committed source copy ${pin.source_copy} hashes to ${shas.join(" / ") || "<unhashable>"}, but the pin declares blob ${pin.blob_sha} — the copy is not the pinned source`,
        ),
      );
      return { ok: false, text: null, via: null, issues };
    }
    copyText = fs.readFileSync(abs, "utf8");
  }

  if (gitText !== null) return { ok: true, text: gitText, via: "git", issues };
  if (copyText !== null) return { ok: true, text: copyText, via: "committed-copy", issues };

  issues.push(
    issue(
      "PIN_UNRESOLVED",
      `the pinned blob ${pin.blob_sha} could not be resolved from git and the pin declares no committed source copy to fall back to. This is exit 1, never a skip`,
    ),
  );
  return { ok: false, text: null, via: null, issues };
}

// ─── inventory / extraction ─────────────────────────────────────────────────

const BA_SECTIONS = {
  behaviors: (h) => h.startsWith("behaviors"),
  rules: (h) => h.startsWith("business rules"),
  edges: (h) => h.startsWith("edge cases"),
  pointers: (h) => h.startsWith("pointers"),
};

function baSectionOf(heading) {
  const h = heading.toLowerCase();
  for (const [name, test] of Object.entries(BA_SECTIONS)) {
    if (test(h)) return name;
  }
  return null;
}

// A "block start" is a line that opens a new unit of meaning in an
// anchor-bearing section: a top-level bullet or a bold-lead paragraph. Wrapped
// continuation prose and nested bullets are neither — they belong to whatever
// block precedes them. Counting UNCLASSIFIED block starts is what makes
// format-blindness visible: docs/specs/onboarding.md's 22 `- **R1** —` rules
// and its unnumbered `**Detect (every run).**` behaviors are all block starts
// that no shipped regex classifies, so they surface as unparsed instead of
// silently not existing.
const BLOCK_START_RE = /^(-\s+|\*\*)/;

// ─── the id forms (f2-4) ────────────────────────────────────────────────────
// The classifier shipped by okf-5 required a BARE id at the head of the block:
// /^\*\*(B\d+)\s+—/ and /^-\s+(R\d+)\s+—/. Those were written against
// advisor-protocol, and five of the nine remaining areas simply write the same
// anchors in a different hand — the id bold-wrapped, or carrying a citation
// before the em dash, or letter-suffixed:
//
//   - **R1** — …                     doctrine-layer L213, onboarding L337
//   - **R1 — …**                     decision-memory L16, performance-log L140
//   - **R1** (D1) — …                feedback-digest L259
//   - **R7 (not yet implemented — P24)** — …   onboarding L365
//   - R8a — …                        hook-runtime L412
//   **B3a — …**                      doctrine-layer L89, workflow-state L207
//
// Read by the narrow patterns, doctrine-layer derived R0 while carrying R1–R17,
// and decision-memory derived 0 anchors and was filed by the planning sweep as
// "shapeless — needs a bespoke scheme". It is not shapeless: its nine rules were
// invisible. A "no structure here" verdict that is really "this reader cannot
// see the structure" is the same lie the derived gate exists to prevent, one
// level up — so the classifier is widened to the id FORMS, and the sweep
// re-classified (docs/history/okf-migration-f2/reports/inventory-sweep.md).
//
// Two boundaries hold this in place:
//
//   1. IT READS IDS, IT NEVER INVENTS THEM. An unnumbered bold-lead paragraph
//      (`**Detect (every run).**`, onboarding L96) stays UNPARSED and keeps
//      showing in the unparsed report. Assigning it a positional B-id would
//      fabricate structure the source never had (D10) and would collide with
//      the source's own B-ids in the areas that use both.
//   2. THE NO-OP IS THE SAFETY PROPERTY. advisor-protocol must still derive
//      exactly 26 {4,9,6,7} with unparsed_blocks 0, and critical-patterns 47,
//      from their pinned blobs with expected_counts untouched. A widening that
//      moves either is too broad and gets narrowed — never the pin relaxed.
//      Asserted in scripts/tests/test_okf_pins.mjs (sections 24-26).
const BOLD_MARKER_RE = /\*\*/g;
/** Bold markers dropped, for ID MATCHING ONLY — never for anchor text. */
const unbold = (line) => line.replace(BOLD_MARKER_RE, "");
// `(?:\([^)]*\))?` is the optional citation between the id and its em dash;
// `[a-z]?` is the letter suffix (B3a, R8a, R20b). The em dash itself is still
// required, so an ASCII-hyphen bullet is prose, exactly as before.
const BEHAVIOR_ID_RE = /^(B\d+[a-z]?)\s*(?:\([^)]*\))?\s+—/;
const RULE_ID_RE = /^-\s+(R\d+[a-z]?)\s*(?:\([^)]*\))?\s+—/;

function emptyUnparsed() {
  return {
    blocks: { behaviors: 0, rules: 0, edges: 0, pointers: 0, total: 0 },
    lines: { behaviors: 0, rules: 0, edges: 0, pointers: 0, total: 0 },
    samples: [],
  };
}

// Parse a nine-section BA spec's numbered anchors. Returns
// { behaviors, rules, edges, pointers, all, unparsed } where behaviors/rules
// carry the source's own B-/R-numbered ids and edges/pointers are assigned
// E-/P-ids by top-level-bullet position inside their sections (document
// order). Classification is unchanged from okf-5 — deliberately: the two
// shipped areas must still derive 26 and 47 from the same rules. What is new
// is `unparsed`, which reports what the classification could NOT see.
export function inventorySpec(text) {
  const lines = text.split("\n");
  const behaviors = [];
  const rules = [];
  let edgeBullets = 0;
  let pointerBullets = 0;
  let section = null; // classification section (edges/pointers only, as shipped)
  let anchorSection = null; // accounting section (all four)
  const unparsed = emptyUnparsed();
  let lineNo = 0;

  // f2-2 (F11): the anchor's own BYTES, not only its id. An anchor's text runs
  // from its block-starting line up to the next block start or the next `##`
  // heading, so wrapped continuation prose and nested bullets travel with the
  // anchor they belong to. Ids alone can only prove an anchor was CLAIMED; the
  // text is what makes "was it actually carried across?" a measurable question.
  const texts = new Map();
  let current = null;
  const openAnchor = (id) => {
    current = [];
    texts.set(id, current);
  };

  for (const line of lines) {
    lineNo += 1;
    const heading = /^##\s+(.*)$/.exec(line);
    if (heading) {
      const h = heading[1].toLowerCase();
      if (h.startsWith("edge cases")) section = "edges";
      else if (h.startsWith("pointers")) section = "pointers";
      else section = null;
      anchorSection = baSectionOf(heading[1]);
      current = null;
      continue;
    }

    let classified = false;
    const bold = line.startsWith("**") ? BEHAVIOR_ID_RE.exec(unbold(line)) : null;
    if (bold) {
      behaviors.push(bold[1]);
      openAnchor(bold[1]);
      classified = true;
    } else {
      const rule = /^-\s+/.test(line) ? RULE_ID_RE.exec(unbold(line)) : null;
      if (rule) {
        rules.push(rule[1]);
        openAnchor(rule[1]);
        classified = true;
      } else if (/^-\s+/.test(line)) {
        if (section === "edges") {
          edgeBullets += 1;
          openAnchor(`E${edgeBullets}`);
          classified = true;
        } else if (section === "pointers") {
          pointerBullets += 1;
          openAnchor(`P${pointerBullets}`);
          classified = true;
        }
      }
    }

    if (classified) {
      current.push(line);
      continue;
    }
    if (current && anchorSection) current.push(line);
    if (!anchorSection) continue;
    if (!line.trim()) continue;
    if (/^#/.test(line)) continue; // structural sub-headings, not content

    unparsed.lines[anchorSection] += 1;
    unparsed.lines.total += 1;
    if (BLOCK_START_RE.test(line)) {
      unparsed.blocks[anchorSection] += 1;
      unparsed.blocks.total += 1;
      if (unparsed.samples.length < 12) {
        unparsed.samples.push(`${anchorSection} L${lineNo}: ${line.trim().slice(0, 100)}`);
      }
    }
  }

  const edges = Array.from({ length: edgeBullets }, (_, i) => `E${i + 1}`);
  const pointers = Array.from({ length: pointerBullets }, (_, i) => `P${i + 1}`);
  return {
    behaviors,
    rules,
    edges,
    pointers,
    all: [...behaviors, ...rules, ...edges, ...pointers],
    texts: new Map([...texts].map(([id, buf]) => [id, buf.join("\n").trim()])),
    unparsed,
  };
}

/** Parse critical-patterns.md's own shape: one anchor per `## [YYYYMMDD] …`
 *  heading, document order. Mirrors inventorySpec's role for the BA shape.
 *  Its unparsed report counts `##` headings that are NOT dated pattern
 *  headings — for this flat scheme a heading IS the block start — plus the
 *  raw body-line count. */
export function inventoryPatterns(text) {
  const headingRe = /^## \[(\d{8})\] .+$/;
  const anyH2Re = /^##\s+/;
  let n = 0;
  const unparsed = {
    blocks: { headings: 0, total: 0 },
    lines: { body: 0, total: 0 },
    samples: [],
  };
  let lineNo = 0;
  let seenFirst = false;
  const texts = new Map();
  let current = null;
  for (const line of text.split("\n")) {
    lineNo += 1;
    if (headingRe.test(line)) {
      n += 1;
      seenFirst = true;
      current = [line];
      texts.set(`PAT${n}`, current);
      continue;
    }
    if (anyH2Re.test(line)) {
      current = null;
      unparsed.blocks.headings += 1;
      unparsed.blocks.total += 1;
      if (unparsed.samples.length < 12) unparsed.samples.push(`L${lineNo}: ${line.trim().slice(0, 100)}`);
      continue;
    }
    if (current) current.push(line);
    if (!seenFirst) continue; // front matter / document title, before any pattern
    if (/^#/.test(line)) continue;
    if (!line.trim()) continue;
    unparsed.lines.body += 1;
    unparsed.lines.total += 1;
  }
  const all = Array.from({ length: n }, (_, i) => `PAT${i + 1}`);
  return {
    patterns: all,
    all,
    texts: new Map([...texts].map(([id, buf]) => [id, buf.join("\n").trim()])),
    unparsed,
  };
}

// ─── the third scheme: `narrative-sections` (f2-11, F9/S5) ──────────────────
//
// One area of the eleven genuinely has no numbered anchors. Every earlier
// "shapeless" verdict was a blind reader — decision-memory's nine rules were
// written `- **R1 — …**` and f2-4's widening found them — but
// docs/specs/worktree-parallelism.md really does carry no `B*`/`R*`/`E*`/`P*`
// id and none of the four anchor-bearing section headings, so
// `ba-nine-section` derives 0 anchors AND 0 unparsed blocks from it.
//
// F9 forbids forcing it into the nine-section shape; D10 forbids inventing
// numbered ids the source never had. Both would be the same lie in opposite
// directions. What the source DOES have is its own narrative structure: ten
// `## ` headings, written by the author, each opening a self-contained
// subject. So THE HEADINGS ARE THE ANCHORS — derived mechanically from the
// heading text, exactly as `flat-pattern-list` already derives one anchor per
// `## [YYYYMMDD] …` heading in critical-patterns.md. Nothing is invented; the
// ground truth is read off the source's own structure.
//
// Three boundaries hold this in place, all asserted in
// scripts/tests/test_okf_pins.mjs section 27:
//
//   1. A `## ` HEADING IS AN ANCHOR; A `###` SUBHEADING IS NOT. Deeper
//      headings are structure WITHIN a section, so their prose travels with
//      the section that contains it and the fidelity floor measures it there.
//      They are still REPORTED as unparsed blocks, so a source that grows a
//      subheading-heavy shape this scheme cannot see stays visible.
//   2. ZERO `## ` HEADINGS IS A REFUSAL, NEVER 0/0. A scheme that returns an
//      empty set for a file it cannot read converts lost content into content
//      that never existed — the exact defect this whole file exists to
//      prevent. It is typed PIN_EMPTY_EXTRACTION at the scheme level, before
//      derivePin's own empty check can even be reached.
//   3. TWO HEADINGS THAT SLUGIFY THE SAME ARE A REFUSAL. Anchors are keyed by
//      id, so a collision silently overwrites the first section's text and is
//      unmeasurable by the fidelity floor forever. That is the duplicate-`R14`
//      hazard f2-10 had to repair a source to escape; here it is closed at the
//      scheme level, so no narrative source can ever enter that state.
export const NARRATIVE_ANCHOR_PREFIX = "S-";
/** The claim/stub-row form a narrative anchor takes, kept in ONE place so the
 *  extractor, the stub-row parser and the bee.sources claim matcher can never
 *  drift apart (the f2-3 lesson, applied before it can bite). */
export const NARRATIVE_ANCHOR_PATTERN = "S-[a-z0-9]+(?:-[a-z0-9]+)*";

/** Heading text → the slug half of its anchor id. Lowercase, every
 *  non-alphanumeric run collapsed to a single hyphen, ends trimmed. Purely
 *  mechanical: the id is a function of the bytes the author wrote, so no
 *  reader has to trust that someone numbered the sections honestly. */
export function slugifyHeading(heading) {
  return String(heading || "")
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
}

/** Parse a narrative spec's own `## ` headings as its anchors. Returns
 *  { sections, all, texts, unparsed } — `texts` maps each anchor to its
 *  heading line plus the body up to the NEXT `## ` heading, subheadings
 *  included. Mirrors inventorySpec's and inventoryPatterns' role for the
 *  narrative shape.
 *
 *  Its unparsed report has two members, both real and both visible: block
 *  starts sitting BEFORE the first `## ` heading (a document title and its
 *  metadata block belong to no section, and none of it is invented into an
 *  anchor — D10), and `###`+ subheadings, which are block starts this scheme
 *  deliberately does not promote. */
export function inventoryNarrativeSections(text) {
  const sectionHeadingRe = /^##(?!#)\s+(.+?)\s*$/;
  const deeperHeadingRe = /^#{3,}\s+/;
  const sections = [];
  const texts = new Map();
  const unparsed = {
    blocks: { preamble: 0, subheadings: 0, total: 0 },
    lines: { preamble: 0, total: 0 },
    samples: [],
  };
  let current = null;
  let lineNo = 0;

  for (const line of String(text || "").split("\n")) {
    lineNo += 1;
    const heading = sectionHeadingRe.exec(line);
    if (heading) {
      const slug = slugifyHeading(heading[1]);
      const id = `${NARRATIVE_ANCHOR_PREFIX}${slug}`;
      if (texts.has(id)) {
        const err = new Error(
          `narrative-sections: two "## " headings slugify to the same anchor id "${id}" (second at L${lineNo}: ${line.trim()}). Anchors are keyed by id, so the first section's text would be silently overwritten and unmeasurable by the fidelity floor forever — the duplicate-id hazard f2-10 had to repair a source to escape. Disambiguate the headings in the source before pinning it`,
        );
        err.code = "PIN_DUPLICATE_ANCHOR";
        throw err;
      }
      sections.push(id);
      current = [line];
      texts.set(id, current);
      continue;
    }
    if (current) {
      current.push(line);
      if (deeperHeadingRe.test(line)) {
        unparsed.blocks.subheadings += 1;
        unparsed.blocks.total += 1;
        if (unparsed.samples.length < 12) {
          unparsed.samples.push(`subheading L${lineNo}: ${line.trim().slice(0, 100)}`);
        }
      }
      continue;
    }
    // Before the first `## ` heading: real content that belongs to no anchor.
    if (/^#/.test(line)) continue; // the document title, not content
    if (!line.trim()) continue;
    unparsed.lines.preamble += 1;
    unparsed.lines.total += 1;
    if (BLOCK_START_RE.test(line)) {
      unparsed.blocks.preamble += 1;
      unparsed.blocks.total += 1;
      if (unparsed.samples.length < 12) {
        unparsed.samples.push(`preamble L${lineNo}: ${line.trim().slice(0, 100)}`);
      }
    }
  }

  return {
    sections,
    all: [...sections],
    texts: new Map([...texts].map(([id, buf]) => [id, buf.join("\n").trim()])),
    unparsed,
  };
}

export const SCHEMES = {
  "ba-nine-section": (text) => {
    const inv = inventorySpec(text);
    return {
      anchors: inv,
      counts: {
        behaviors: inv.behaviors.length,
        rules: inv.rules.length,
        edges: inv.edges.length,
        pointers: inv.pointers.length,
        total: inv.all.length,
        unparsed_blocks: inv.unparsed.blocks.total,
      },
      unparsed: inv.unparsed,
    };
  },
  "flat-pattern-list": (text) => {
    const inv = inventoryPatterns(text);
    return {
      anchors: inv,
      counts: {
        patterns: inv.all.length,
        total: inv.all.length,
        unparsed_blocks: inv.unparsed.blocks.total,
      },
      unparsed: inv.unparsed,
    };
  },
  "narrative-sections": (text) => {
    const inv = inventoryNarrativeSections(text);
    // Boundary 2: a source with no `## ` heading is REFUSED here, at the
    // scheme, rather than handed onward as an empty set. derivePin refuses an
    // empty extraction too, but a scheme that can name WHY it saw nothing —
    // "this file has no `## ` headings, so narrative-sections is the wrong
    // scheme for it" — is the difference between a reader who fixes the pin
    // and a reader who reads 0/0 as an area with nothing in it.
    if (inv.all.length === 0) {
      const err = new Error(
        `narrative-sections: the source carries no "## " heading at all, so this scheme can derive no anchors from it. That is a REFUSAL, never a 0/0 pass — either the pin names the wrong blob or this file's shape needs a scheme of its own (F9/D10: a "nothing here" verdict that is really "this reader cannot see it" is the defect the whole gate exists to prevent)`,
      );
      err.code = "PIN_EMPTY_EXTRACTION";
      throw err;
    }
    return {
      anchors: inv,
      counts: {
        sections: inv.all.length,
        total: inv.all.length,
        unparsed_blocks: inv.unparsed.blocks.total,
      },
      unparsed: inv.unparsed,
    };
  },
};

/** Dispatch extraction on the pin's declared scheme. An unknown scheme name
 *  is a refusal, never an empty result. */
export function extractByScheme(scheme, text) {
  const fn = SCHEMES[scheme];
  if (!fn) {
    const err = new Error(
      `no extractor for scheme "${scheme}" (known: ${Object.keys(SCHEMES).join(", ")})`,
    );
    err.code = "PIN_UNKNOWN_SCHEME";
    throw err;
  }
  return fn(text);
}

// ─── pin assertion ──────────────────────────────────────────────────────────

/**
 * Assert a pin end to end and return its derived anchors.
 *
 * Every failure mode is typed and every one of them is exit-1 material:
 *   PIN_NO_SCHEME        the area's anchor shape has not been decided
 *   PIN_UNKNOWN_SCHEME   the declared scheme has no extractor
 *   PIN_INCOMPLETE       the pin asserts no total (a pin that expects nothing
 *                        proves nothing)
 *   PIN_SHA_MISMATCH     commit:path, or the committed copy, is not blob_sha
 *   PIN_COPY_MISSING     no committed copy to survive a shallow clone
 *   PIN_UNRESOLVED       neither git nor a copy can produce the source
 *   PIN_EMPTY_EXTRACTION the extraction produced nothing — 0/0 is NEVER a pass
 *   PIN_COUNT_MISMATCH   a derived count differs from expected_counts
 */
export function derivePin(pin, label = "<pin>") {
  const fail = (...issues) => ({ ok: false, label, issues, counts: null, anchors: null, unparsed: null, via: null });

  if (!pin || typeof pin !== "object") {
    return fail(issue("PIN_UNKNOWN_AREA", `no pin object supplied for "${label}"`));
  }
  if (pin.scheme === null || pin.scheme === undefined) {
    return fail(
      issue(
        "PIN_NO_SCHEME",
        `"${label}" declares no anchor scheme, so it is REFUSED rather than passed 0/0. ${pin.refusal || "Declare a scheme in PIN_REGISTRY before gating this area."}`,
      ),
    );
  }
  if (!SCHEMES[pin.scheme]) {
    return fail(
      issue("PIN_UNKNOWN_SCHEME", `"${label}" declares scheme "${pin.scheme}", which has no extractor (known: ${Object.keys(SCHEMES).join(", ")})`),
    );
  }
  if (!pin.blob_sha || !/^[0-9a-f]{40}$/.test(pin.blob_sha)) {
    return fail(issue("PIN_INCOMPLETE", `"${label}" declares no usable blob_sha — a pin without a content address is not a pin`));
  }
  if (typeof pin.expected_counts?.total !== "number") {
    return fail(issue("PIN_INCOMPLETE", `"${label}" declares no expected_counts.total — a pin that expects nothing proves nothing`));
  }
  // f2-2: unparsed_blocks is MANDATORY, closing the f2-1b judge's residual
  // gap. `total` alone only asserts that the counts still add up; it is
  // unparsed_blocks that asserts the extractor still SEES the same shape. A
  // pin omitting it opts out of the format-blindness guard silently — exactly
  // the "absence read as a pass" this file exists to prevent — so omission is
  // a refusal, not a default.
  if (typeof pin.expected_counts.unparsed_blocks !== "number") {
    return fail(
      issue(
        "PIN_INCOMPLETE",
        `"${label}" declares no expected_counts.unparsed_blocks — a pin that does not assert how much of its source the extractor could NOT classify cannot detect format-blindness, and silently opting out of that guard is the failure this gate exists to prevent (f2-2). Declare it, even when it is 0`,
      ),
    );
  }

  const resolved = resolvePinnedSource(pin);
  if (!resolved.ok) return fail(...resolved.issues);

  let extracted;
  try {
    extracted = extractByScheme(pin.scheme, resolved.text);
  } catch (error) {
    return fail(issue(error.code || "PIN_UNKNOWN_SCHEME", `"${label}": ${error.message}`));
  }

  const issues = [];

  // An empty extraction is refused BEFORE the count comparison, so a pin that
  // declares `total: 0` can never launder a format-blind read into a green.
  if (!extracted.anchors.all || extracted.anchors.all.length === 0) {
    issues.push(
      issue(
        "PIN_EMPTY_EXTRACTION",
        `"${label}": scheme "${pin.scheme}" extracted ZERO anchors from the pinned source (${resolved.via}). An empty extraction never reads as a pass — either the pin points at the wrong blob or the scheme cannot see this file's shape`,
      ),
    );
  }

  for (const [key, want] of Object.entries(pin.expected_counts)) {
    const got = extracted.counts[key];
    if (got !== want) {
      issues.push(
        issue(
          "PIN_COUNT_MISMATCH",
          `"${label}": expected_counts.${key} = ${want}, but the pinned blob (${resolved.via}) derives ${got}`,
        ),
      );
    }
  }

  if (issues.length > 0) {
    return { ok: false, label, issues, counts: extracted.counts, anchors: extracted.anchors, unparsed: extracted.unparsed, via: resolved.via };
  }
  return {
    ok: true,
    label,
    issues: [],
    counts: extracted.counts,
    anchors: extracted.anchors,
    unparsed: extracted.unparsed,
    via: resolved.via,
    blob_sha: pin.blob_sha,
    commit: pin.commit,
  };
}

/** derivePin for a registered area, with an explicit refusal for an area that
 *  carries no pin at all. */
export function derivePinForArea(area) {
  const pin = PIN_REGISTRY[area];
  if (!pin) {
    return {
      ok: false,
      label: area,
      counts: null,
      anchors: null,
      unparsed: null,
      via: null,
      issues: [
        issue(
          "PIN_UNKNOWN_AREA",
          `no pin for area "${area}" — it has no content-addressed ground truth, so it cannot be gated (pinned: ${Object.keys(PIN_REGISTRY).join(", ")})`,
        ),
      ],
    };
  }
  return derivePin(pin, area);
}

function reportPinFailure(label, result) {
  console.error(`FAIL okf_migrate pin ${label}: ${result.issues.length} problem(s) — the derived ground truth is NOT trustworthy`);
  for (const i of result.issues) console.error(`  - ${i.code}: ${i.message}`);
}

// ─── F11: the per-anchor fidelity floor ─────────────────────────────────────
//
// Set-equality answers "was this anchor CLAIMED by exactly one concept?". It
// cannot answer "was the anchor actually CARRIED ACROSS?" — a concept that
// summarises its anchor into a sentence still satisfies every count. That is
// the degradation mode of a long serial re-authoring run: anchor-shaped
// compliance. F11 makes it mechanically detectable without a judge.
//
// For each anchor: normalized token overlap between the anchor's text in the
// PINNED BLOB and the body of the concept that claims it.
//
//   overlap = |anchor_tokens ∩ concept_tokens| / |anchor_tokens|
//
// The denominator is the ANCHOR, never the concept: a concept is free to be
// longer, better organised, or to merge several anchors — it is not free to
// drop the anchor's content.
//
// ─── the tuning rule, which is inverted from the usual instinct ─────────────
//
// advisor-protocol's 26 anchors and critical-patterns' 47 must ALL clear this
// floor with ZERO edits to their concepts. If one fails, the NORMALIZATION is
// wrong — never the migrated content, never the threshold. Both are asserted
// in scripts/tests/test_okf_pins.mjs, which also asserts that the MEDIAN sits well
// clear of the floor: a median barely above 0.60 would itself mean the
// normalization had become too strict and the floor was measuring the metric
// rather than the migration. Measured as shipped:
//
//   advisor-protocol   n=26  min 0.977  median 1.000  max 1.000
//   critical-patterns  n=47  min 0.771  median 0.900  max 0.955
export const FIDELITY_FLOOR = 0.6;

// Deliberately small: articles, prepositions, conjunctions, pronouns and
// auxiliaries only. Every dropped word makes the metric STRICTER (it removes
// a cheap hit from the anchor's denominator), so this list is kept to words
// that carry no domain meaning. Modal/negation words — never, always, must,
// only, refuses — are NOT stopwords: in this repo they are the content.
export const FIDELITY_STOPWORDS = new Set([
  "a", "an", "and", "are", "as", "at", "be", "been", "being", "but", "by", "did", "do", "does",
  "for", "from", "had", "has", "have", "he", "her", "here", "hers", "him", "his", "how", "i",
  "in", "into", "is", "it", "its", "me", "my", "of", "on", "onto", "or", "our", "ours", "out",
  "she", "so", "than", "that", "the", "their", "theirs", "them", "then", "there", "these",
  "they", "this", "those", "to", "up", "us", "was", "we", "were", "what", "when", "where",
  "which", "who", "whom", "with", "you", "your", "yours",
]);

/**
 * lowercase → strip markdown emphasis, backticks and all punctuation → split
 * on whitespace → drop one-character tokens and the stopword set → dedupe.
 *
 * Collapsing every non-alphanumeric run to a space is what makes the two
 * sides comparable at all: `--runtime`, "read-only", `docs/specs/x.md` and
 * **bold** all normalize identically whether the source hyphenated, quoted,
 * backticked or bolded them, so a concept is never punished for reformatting
 * what it faithfully kept.
 */
export function normalizeTokens(text) {
  const tokens = new Set();
  for (const raw of String(text || "").toLowerCase().replace(/[^a-z0-9]+/g, " ").split(" ")) {
    if (raw.length < 2) continue; // single characters are noise, never content
    if (FIDELITY_STOPWORDS.has(raw)) continue;
    tokens.add(raw);
  }
  return tokens;
}

/** Fraction of the ANCHOR's normalized tokens that survive in the concept.
 *  An anchor with no meaningful tokens at all scores 1 — there is nothing to
 *  lose, and a divide-by-zero must never read as a failure. */
export function tokenOverlap(anchorText, conceptText) {
  const anchor = normalizeTokens(anchorText);
  if (anchor.size === 0) return 1;
  const concept = normalizeTokens(conceptText);
  let hit = 0;
  for (const token of anchor) if (concept.has(token)) hit += 1;
  return hit / anchor.size;
}

export function medianOf(values) {
  if (!values.length) return null;
  const sorted = [...values].sort((a, b) => a - b);
  const mid = sorted.length >> 1;
  return sorted.length % 2 ? sorted[mid] : (sorted[mid - 1] + sorted[mid]) / 2;
}

/**
 * Pure: measure every expected anchor against its owning concept's body.
 *
 *   expected  anchor ids (the derived ground truth)
 *   texts     Map anchor -> the anchor's text in the pinned blob
 *   claims    Map anchor -> [owner path, ...] (collectClaims' shape)
 *   bodies    Map owner path -> that concept's body text
 *
 * Anchors with no owner, or with more than one, are NOT reported here — that
 * is coverageIssues' job, and reporting them twice would bury the fidelity
 * signal under duplicates of a failure already named.
 */
export function fidelityReport({ expected, texts, claims, bodies, floor = FIDELITY_FLOOR }) {
  const get = (map, key) => (map instanceof Map ? map.get(key) : map?.[key]);
  const rows = [];
  const issues = [];
  for (const anchor of expected) {
    const ownersRaw = get(claims, anchor);
    const owners = Array.isArray(ownersRaw) ? ownersRaw : ownersRaw ? [ownersRaw] : [];
    if (owners.length !== 1) continue;
    const owner = owners[0];
    const anchorText = get(texts, anchor);
    const body = get(bodies, owner);
    if (typeof anchorText !== "string") {
      issues.push(
        `FIDELITY UNMEASURABLE: ${anchor} has no extracted text from the pinned blob — the extractor produced an id it cannot show the bytes for (F11)`,
      );
      continue;
    }
    if (typeof body !== "string") {
      issues.push(`FIDELITY UNMEASURABLE: ${anchor}'s owner "${owner}" could not be read (F11)`);
      continue;
    }
    const anchorTokens = normalizeTokens(anchorText);
    const conceptTokens = normalizeTokens(body);
    const missing = [...anchorTokens].filter((t) => !conceptTokens.has(t));
    const ratio = anchorTokens.size === 0 ? 1 : (anchorTokens.size - missing.length) / anchorTokens.size;
    rows.push({ anchor, owner, ratio, anchor_tokens: anchorTokens.size, missing_tokens: missing.length, missing });
    if (ratio < floor) {
      issues.push(
        `FIDELITY BELOW FLOOR: ${anchor} (owner ${owner}) retains ${ratio.toFixed(3)} normalized token overlap with its text in the pinned blob — the floor is ${floor.toFixed(2)} (F11). ` +
          `${missing.length} of ${anchorTokens.size} anchor tokens are absent from the owning concept: ${missing.slice(0, 15).join(", ")}${missing.length > 15 ? ", …" : ""}. ` +
          `The anchor was summarised away, not migrated — restore the content; never lower the floor`,
      );
    }
  }
  const ratios = rows.map((r) => r.ratio);
  return {
    issues,
    rows,
    stats: {
      n: rows.length,
      min: ratios.length ? Math.min(...ratios) : null,
      median: medianOf(ratios),
      max: ratios.length ? Math.max(...ratios) : null,
      floor,
    },
  };
}

/** Where an area's concepts live and how it cites its source. One place, so
 *  --check, --check-patterns, the fidelity floor and the telemetry can never
 *  disagree about what a given area's wiring is. */
/** The claim-suffix form a nine-section area's anchors can take, kept in ONE
 *  place so the extractor, the stub-row parser and the claim matcher can never
 *  drift apart again (f2-3). `[a-z]?` is f2-4's letter suffix: B3a, B7a, R8a. */
export const AREA_ANCHOR_PATTERN = "[A-Z]\\d+[a-z]?";

/** The anchor form parseStubAnchorMap reads when nobody says otherwise: the
 *  numbered ids every ba-nine-section area and critical-patterns use. Kept as
 *  the DEFAULT rather than folded into a union with the narrative form, so
 *  adding the narrative form cannot change what any shipped stub parses to
 *  (the strict no-op, f2-11). */
export const NUMBERED_STUB_ANCHOR_PATTERN = "[A-Z]+\\d+[a-z]?";

export function wiringFor(area) {
  const pin = PIN_REGISTRY[area];
  if (pin?.kind === "patterns") {
    return {
      dir: PATTERNS_CONCEPT_DIR,
      source: PATTERNS_SOURCE,
      anchorPattern: "[A-Z]+\\d+",
      stubAnchorPattern: NUMBERED_STUB_ANCHOR_PATTERN,
    };
  }
  // f2-11: an area's anchor FORM follows its declared scheme, never its
  // directory. narrative-sections ids are slugs (`S-the-trust-model`), so a
  // claim matcher or stub-row parser left on the numbered pattern would match
  // none of them and report every anchor LOST however faithfully it had been
  // migrated — the unsatisfiable red f2-3 already had to fix once.
  const narrative = pin?.scheme === "narrative-sections";
  return {
    dir: `docs/knowledge/areas/${area}`,
    source: `docs/specs/${area}.md`,
    anchorPattern: narrative ? NARRATIVE_ANCHOR_PATTERN : AREA_ANCHOR_PATTERN,
    stubAnchorPattern: narrative ? NARRATIVE_ANCHOR_PATTERN : NUMBERED_STUB_ANCHOR_PATTERN,
  };
}

function conceptFilesIn(dir) {
  const abs = path.join(REPO_ROOT, ...dir.split("/"));
  if (!fs.existsSync(abs)) return [];
  return fs
    .readdirSync(abs)
    .filter((n) => n.endsWith(".md") && n !== "index.md" && n !== "log.md")
    .sort();
}

async function readConceptBodies(dir, owners) {
  const knowledge = await import(
    pathToFileURL(path.join(REPO_ROOT, ".bee", "bin", "lib", "knowledge.mjs")).href
  );
  const bodies = new Map();
  for (const owner of new Set(owners)) {
    const abs = path.join(REPO_ROOT, ...owner.split("/"));
    if (!fs.existsSync(abs)) continue;
    const raw = fs.readFileSync(abs, "utf8");
    const parsed = knowledge.parseFrontmatter(raw);
    // The BODY only — frontmatter would smuggle the anchor id itself and the
    // source citation into the token pool, letting a stub score above the
    // floor on its own bookkeeping.
    bodies.set(owner, parsed.ok && parsed.present ? parsed.body : raw);
  }
  return bodies;
}

/** The F11 floor for one registered area, end to end: derive the pin, collect
 *  the claims, read the owning bodies, measure. */
export async function areaFidelity(area) {
  const derived = derivePinForArea(area);
  if (!derived.ok) {
    return { ok: false, area, rows: [], stats: null, issues: derived.issues.map((i) => `${i.code}: ${i.message}`) };
  }
  const { dir, source, anchorPattern } = wiringFor(area);
  const { claims, issues: claimIssues } = await collectClaims({ dir, source, anchorPattern });
  const owners = [...claims.values()].flat();
  const bodies = await readConceptBodies(dir, owners);
  const report = fidelityReport({
    expected: derived.anchors.all,
    texts: derived.anchors.texts,
    claims,
    bodies,
  });
  return { ok: true, area, rows: report.rows, stats: report.stats, issues: [...claimIssues, ...report.issues] };
}

// ─── F12: drift telemetry ───────────────────────────────────────────────────
//
// Drift should land as a chain red at cell 4, not be discovered at cell 10.
// Two shape ratios per area, both cheap and both directional:
//
//   anchors_per_concept            too high = a dumping-ground concept
//                                  swallowing a section; too low = concepts
//                                  shredded past usefulness
//   concepts_per_100_source_lines  the same drift measured against the source
//                                  rather than against the anchor count
//
// Compared against the running MEDIAN of already-pinned areas, outside a
// [0.5x, 2x] band. With fewer than three pinned areas there is no median worth
// the name, so telemetry REPORTS and never fails — a two-sample "median" is a
// coin flip, and a gate that fails on a coin flip teaches everyone to ignore
// it.
//
// f2-3: THE MEDIAN IS TAKEN OVER COMPARABLE SHAPES ONLY. F12 says "the running
// median of already-migrated AREAS", and the moment a third pin made a median
// exist at all, the reason for that wording became measurable: critical-patterns
// is a `flat-pattern-list` migration where one anchor IS one concept by
// construction (okf-6), so its anchors_per_concept is pinned at 1.00 forever and
// can never sit inside a band drawn around nine-section AREAS (5.57, 6.5). Left
// pooled, the guard reported drift in already-shipped, already-reviewed work
// that no cell had touched — and would have gone on reporting it whatever any
// future migration did. Rows are still reported for every pinned source; only
// the comparison population is restricted, to pins of the same `kind`.

// f4-7: THE DENOMINATOR IS THE CONCEPTS THAT OWN ANCHORS, NEVER THE FILES IN
// THE DIRECTORY. Both ratios measure ONE thing: how the PINNED SOURCE was
// decomposed. The numerators say so — `anchors` is the pinned source's anchor
// count and `source_lines` is the pinned source's length; neither can be moved
// by anything written after the migration. The denominator
// `conceptFilesIn(dir).length` could: it counted every .md in the area
// directory, migrated or not. So the moment a migrated area grew a concept of
// genuinely NEW truth — which is the healthy thing for a live area to do, and
// the whole point of migrating into a live bundle — the denominator grew while
// the numerator structurally could not, and the area drifted out of band
// permanently, reporting a "decomposition fault" in a decomposition nobody had
// touched. That is not the fault F12 exists to catch; it is the metric
// mistaking growth for drift, and it is unfixable by the area (the only way
// back into band would be to delete true concepts).
//
// The honest denominator is the number of concepts the COVERAGE WALK actually
// attributes at least one pinned anchor to — the same claims map --check reads,
// so telemetry and coverage can never disagree about what "a concept of this
// migration" means. It leaves the metric exactly as sharp as it was: a
// dumping-ground concept still swallows many anchors into one owner (ratio
// spikes), shredded concepts still spread one anchor per owner (ratio
// collapses). Only concepts that own nothing from the pinned source — which
// carry no information about how that source was decomposed — stop being
// counted as if they did.
//
// This is a correction of WHAT IS MEASURED, not a loosened threshold: the band
// stays [0.5x, 2x] and TELEMETRY_MIN_SAMPLES stays 3. It is also why the defect
// hid so long: at migration time an area's concepts all owned anchors, so the
// two counts coincided and every calibration was taken on that coincidence.

export const TELEMETRY_MIN_SAMPLES = 3;
export const TELEMETRY_METRICS = ["anchors_per_concept", "concepts_per_100_source_lines"];

/** Pure: the distinct concepts the coverage walk attributes at least one of
 *  `expected` (the pinned source's derived anchors) to. Claims for anchors the
 *  registry does not expect are ignored here — `coverageIssues` reports those
 *  as EXTRA, and a stray claim must never enlarge or shrink the denominator. */
export function anchorOwningConcepts(claims, expected) {
  const owners = new Set();
  for (const anchor of expected) {
    for (const owner of claims.get(anchor) || []) owners.add(owner);
  }
  return owners;
}

/** One telemetry row per gateable pinned area (an unscheme'd pin has no
 *  derived anchors, so it contributes no shape). */
export async function collectTelemetry() {
  const rows = [];
  for (const [area, pin] of Object.entries(PIN_REGISTRY)) {
    if (!pin.scheme) continue;
    const derived = derivePin(pin, area);
    if (!derived.ok) continue;
    const resolved = resolvePinnedSource(pin);
    const sourceLines = resolved.ok ? resolved.text.split("\n").length : 0;
    const wiring = wiringFor(area);
    const { claims } = await collectClaims(wiring);
    // The denominator of BOTH ratios — see the note above.
    const concepts = anchorOwningConcepts(claims, derived.anchors.all).size;
    const conceptFiles = conceptFilesIn(wiring.dir).length;
    if (!concepts || !sourceLines) continue;
    rows.push({
      area,
      // "area" | "patterns" — the bundle wiring this pin uses. Reported, but
      // no longer the comparability key: see `scheme` below.
      kind: pin.kind || "area",
      // THE COMPARABILITY KEY (f2-3's rule, f2-11's correction). Shape ratios
      // are only meaningful against migrations of the same shape, and the
      // thing that says what shape a source is, is its SCHEME — `kind` only
      // ever approximated it, because until f2-11 every "area"-kinded pin
      // happened to be ba-nine-section. narrative-sections breaks that
      // coincidence: its anchors are whole SECTIONS, so a 225-line source
      // yields 10 of them where a nine-section source of the same length
      // yields 23, and pooling the two would report permanent "drift" in
      // already-shipped work that no cell had touched — the exact defect f2-3
      // fixed for flat-pattern-list. Keying on the scheme is a strict no-op
      // for every pin that existed before: the eight ba-nine-section areas
      // group exactly as the eight `kind: "area"` pins did, and
      // critical-patterns stays alone exactly as `kind: "patterns"` did
      // (asserted in scripts/tests/test_okf_pins.mjs section 29).
      scheme: pin.scheme,
      anchors: derived.counts.total,
      // The measured denominator: concepts owning >= 1 pinned anchor (f4-7).
      concepts,
      // Reported, never measured: every .md in the area directory. The gap
      // between the two is the area's post-migration growth — visible on
      // purpose, so nobody has to re-derive why the two numbers differ.
      concept_files: conceptFiles,
      source_lines: sourceLines,
      anchors_per_concept: derived.counts.total / concepts,
      concepts_per_100_source_lines: (concepts * 100) / sourceLines,
    });
  }
  return rows;
}

/** Pure: is `current` an outlier against the running median of `samples`? */
export function telemetryIssues({ current, samples, minSamples = TELEMETRY_MIN_SAMPLES }) {
  if (!current || !Array.isArray(samples) || samples.length < minSamples) return [];
  const issues = [];
  for (const metric of TELEMETRY_METRICS) {
    const values = samples.map((s) => s[metric]).filter((v) => typeof v === "number" && v > 0);
    if (values.length < minSamples) continue;
    const median = medianOf(values);
    if (!median) continue;
    const value = current[metric];
    if (typeof value !== "number" || value <= 0) continue;
    const ratio = value / median;
    if (ratio > 2 || ratio < 0.5) {
      issues.push(
        `TELEMETRY OUTLIER (F12): ${current.area}'s ${metric} is ${value.toFixed(2)}, ${ratio.toFixed(2)}x the running median ${median.toFixed(2)} across ${values.length} pinned areas — outside the [0.5x, 2x] band. Either this area's decomposition drifted from every area before it, or the areas before it did`,
      );
    }
  }
  return issues;
}

// ─── F12: whole-bundle invariants, run on EVERY check ───────────────────────
//
// Three properties that must hold for the bundle as a whole, not per area:
// authority uniqueness (D31), zero not_canonical concepts, and fresh
// generated indexes (D21). The bundle's own checker already computes all
// three — but two of them are PROFILE WARNINGS there (D13 keeps `knowledge
// check` in the chain non-strict on purpose), so a drifting bundle would stay
// green forever. The coverage gate graduates exactly those two codes to
// gate-failing for itself, without touching the chain's own `knowledge check`
// entry or D13's warning/error split.

const BUNDLE_FATAL_WARNINGS = new Set(["duplicate_authoritative_for", "duplicate_id", "not_canonical"]);

/** Pure: given a parsed `knowledge check --json` payload and the result of
 *  `knowledge index --check`, is the bundle healthy? */
export function bundleInvariantIssues({ check, index }) {
  const issues = [];
  if (!check || typeof check !== "object") {
    issues.push(`BUNDLE UNHEALTHY: \`bee knowledge check --json\` produced no readable report — the bundle's health is unknown, which is never a pass (F12)`);
  } else {
    for (const error of check.okf?.errors || []) {
      issues.push(`BUNDLE UNHEALTHY: OKF error ${error.code} in ${error.file} — ${error.message}`);
    }
    // f3-3 (G14 layer 3): `duplicate_authoritative_for` was PROMOTED out of
    // `profile.warnings` into chain-failing `profile.errors`, and
    // `malformed_authoritative_for` joined it there. This gate must keep
    // seeing them — a code that moves bucket must never fall out of coverage.
    for (const error of check.profile?.errors || []) {
      issues.push(
        `BUNDLE UNHEALTHY: profile error ${error.code} in ${error.file} — ${error.message}. A chain-failing profile finding is a hard failure to the coverage gate too (F12)`,
      );
    }
    for (const warning of check.profile?.warnings || []) {
      if (!BUNDLE_FATAL_WARNINGS.has(warning.code)) continue;
      issues.push(
        `BUNDLE UNHEALTHY: ${warning.code} in ${warning.file} — ${warning.message}. This is a profile warning to \`knowledge check\` (D13) but a hard failure to the coverage gate: a migration cannot be verified against a bundle whose authority or canonicality has drifted (F12)`,
      );
    }
  }
  if (index && index.ok === false) {
    issues.push(
      `BUNDLE UNHEALTHY: the generated indexes are stale (D21) — \`bee knowledge index --check\` reports: ${String(index.output || "").trim().split("\n").slice(0, 4).join(" / ")}`,
    );
  }
  return issues;
}

function beeCli(args) {
  const r = spawnSync(process.execPath, [path.join(REPO_ROOT, ".bee", "bin", "bee.mjs"), ...args], {
    cwd: REPO_ROOT,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return { code: r.status, out: r.stdout || "", err: r.stderr || "" };
}

/** Run the whole-bundle invariants against the live bundle. */
export function runBundleInvariants() {
  const checkRun = beeCli(["knowledge", "check", "--json"]);
  let check = null;
  try {
    check = JSON.parse(checkRun.out);
  } catch {
    check = null;
  }
  const indexRun = beeCli(["knowledge", "index", "--check"]);
  const index = { ok: indexRun.code === 0, output: `${indexRun.out}${indexRun.err}` };
  return { issues: bundleInvariantIssues({ check, index }), check, index };
}

// ─── check mode helpers ─────────────────────────────────────────────────────

/** Anchor → target concept path rows from a D37 pointer-stub anchor map.
 *  Rows look like: | B1 | [docs/knowledge/areas/<area>/x.md](../…) | (also
 *  matches multi-letter anchor prefixes such as critical-patterns.md's
 *  `PAT1`..`PATn`, okf-6 — the BA-spec single-letter anchors B1/R2/E3/P4
 *  are the special case of one-or-more-letters, so this is additive.) */
export function parseStubAnchorMap(text, anchorPattern = NUMBERED_STUB_ANCHOR_PATTERN) {
  const map = new Map();
  const issues = [];
  // f2-11: the row form is parameterized by the area's SCHEME rather than
  // widened to a union. A union would have to accept both `B1` and
  // `S-the-trust-model` in every stub, so a typo'd row in a numbered area
  // could start matching as a narrative anchor. Passing the pattern in keeps
  // each stub read by exactly the form its own scheme derives — and leaves
  // every shipped stub parsing byte-identically, since the default is the
  // pattern this function has always used.
  const rowRe = new RegExp(`^\\|\\s*\`?(${anchorPattern})\`?\\s*\\|\\s*(.+?)\\s*\\|\\s*$`);
  for (const line of text.split("\n")) {
    // `[a-z]?` (f2-3): f2-4 widened the EXTRACTOR to the letter-suffixed id
    // form (B3a, B7a, R8a) but not the two readers that have to agree with it
    // — this row parser and the bee.sources claim matcher below. Left narrow,
    // a derived `B3a` could never be matched by any stub row or any concept
    // claim, so the gate would report it LOST no matter how faithfully it had
    // been migrated: an unsatisfiable red, which is just the format-blindness
    // defect wearing the opposite sign. Strict no-op on both shipped pins
    // (advisor-protocol carries no suffixed id; `PAT1`..`PAT47` are unchanged).
    const row = rowRe.exec(line);
    if (!row) continue;
    const anchor = row[1];
    const cell = row[2];
    const link = /\[([^\]]+)\]\([^)]+\)/.exec(cell);
    const target = (link ? link[1] : cell).replace(/`/g, "").trim();
    if (map.has(anchor)) {
      issues.push(`stub anchor map lists ${anchor} more than once`);
      continue;
    }
    map.set(anchor, target);
  }
  return { map, issues };
}

/** bee.sources claims of the form <source>#<ANCHOR> from every concept in a
 *  directory (index.md/log.md excluded). Shared by both check modes. */
export async function collectClaims({ dir, source, anchorPattern }) {
  const knowledge = await import(
    pathToFileURL(path.join(REPO_ROOT, ".bee", "bin", "lib", "knowledge.mjs")).href
  );
  const absDir = path.join(REPO_ROOT, ...dir.split("/"));
  const claims = new Map(); // anchor -> [concept repo-relative path, ...]
  const issues = [];
  if (!fs.existsSync(absDir)) {
    issues.push(`concept directory missing: ${dir}/`);
    return { claims, issues };
  }
  const claimRe = new RegExp(`^${source.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}#(${anchorPattern})$`);
  for (const name of fs.readdirSync(absDir).sort()) {
    if (!name.endsWith(".md") || name === "index.md" || name === "log.md") continue;
    const rel = `${dir}/${name}`;
    const parsed = knowledge.parseFrontmatter(fs.readFileSync(path.join(absDir, name), "utf8"));
    if (!parsed.ok || !parsed.present) {
      issues.push(`${rel}: frontmatter missing or unparseable — cannot read bee.sources claims`);
      continue;
    }
    const bee = parsed.data.bee && typeof parsed.data.bee === "object" ? parsed.data.bee : {};
    const sources = Array.isArray(bee.sources) ? bee.sources : [];
    for (const entry of sources) {
      const m = typeof entry === "string" ? claimRe.exec(entry) : null;
      if (!m) continue;
      if (!claims.has(m[1])) claims.set(m[1], []);
      claims.get(m[1]).push(rel);
    }
  }
  return { claims, issues };
}

/** The shared D35 coverage law: every derived anchor owned by exactly one
 *  concept, mapped by the stub to that same concept, and that file present. */
function coverageIssues({ expected, stubMap, claims, stubLabel, registryLabel }) {
  const issues = [];
  const expectedSet = new Set(expected);
  let duplicated = 0;
  let lost = 0;

  for (const anchor of expected) {
    const owners = claims.get(anchor) || [];
    const mapped = stubMap.get(anchor);
    if (!mapped) {
      issues.push(`LOST in stub map: ${anchor} has no row in ${stubLabel}'s anchor map (D37)`);
    }
    if (owners.length === 0) {
      lost += 1;
      issues.push(`LOST in concepts: ${anchor} is claimed by no concept's bee.sources (expected one owner)`);
      continue;
    }
    if (owners.length > 1) {
      duplicated += 1;
      issues.push(`DUPLICATED: ${anchor} is claimed by ${owners.length} concepts: ${owners.join(", ")}`);
      continue;
    }
    const owner = owners[0];
    if (mapped && mapped !== owner) {
      issues.push(`MAP MISMATCH: stub map sends ${anchor} to "${mapped}" but the claiming concept is "${owner}"`);
    }
    if (!fs.existsSync(path.join(REPO_ROOT, owner))) {
      issues.push(`MISSING FILE: ${anchor}'s owner "${owner}" does not exist on disk`);
    }
  }
  for (const anchor of stubMap.keys()) {
    if (!expectedSet.has(anchor)) {
      issues.push(`EXTRA in stub map: ${anchor} is not in ${registryLabel}`);
    }
  }
  for (const anchor of claims.keys()) {
    if (!expectedSet.has(anchor)) {
      issues.push(`EXTRA claim: ${anchor} (claimed by ${claims.get(anchor).join(", ")}) is not in ${registryLabel}`);
    }
  }

  const owned = expected.filter((a) => (claims.get(a) || []).length === 1).length;
  return { issues, owned, duplicated, lost };
}

/**
 * The three f2-2 guards, run identically for every area (F11 + F12). Returns
 * { issues, fidelity, telemetry, telemetryNote } — the issues join the
 * coverage issues, and the reports are printed on green too, because a floor
 * whose margin nobody can see is a floor nobody can tune.
 */
async function runGuards(area, claims, bodies, derived) {
  const issues = [];

  const fidelity = fidelityReport({
    expected: derived.anchors.all,
    texts: derived.anchors.texts,
    claims,
    bodies,
  });
  issues.push(...fidelity.issues);

  const telemetry = await collectTelemetry();
  const current = telemetry.find((r) => r.area === area) || null;
  // Same-SHAPE comparison only (f2-3, keyed on the scheme since f2-11) — see
  // the F12 header note and collectTelemetry's `scheme` field.
  const comparable = current ? telemetry.filter((r) => r.scheme === current.scheme) : [];
  const telemetryNote =
    comparable.length < TELEMETRY_MIN_SAMPLES
      ? `${comparable.length} pinned "${current ? current.scheme : "?"}"-shaped source(s) of ${telemetry.length} pinned in total — fewer than ${TELEMETRY_MIN_SAMPLES} comparable samples, so there is no running median yet and telemetry REPORTS ONLY (never fails)`
      : `running median across ${comparable.length} pinned "${current.scheme}"-shaped sources; outlier band [0.5x, 2x]`;
  issues.push(...telemetryIssues({ current, samples: comparable }));

  issues.push(...runBundleInvariants().issues);

  return { issues, fidelity, telemetry, current, telemetryNote };
}

function printGuards(guards) {
  const s = guards.fidelity.stats;
  if (s.n > 0) {
    console.log(
      `     fidelity floor ${s.floor.toFixed(2)} (F11): ${s.n} anchors measured against their owning concept — min ${s.min.toFixed(3)}, median ${s.median.toFixed(3)}, max ${s.max.toFixed(3)} normalized token overlap with the pinned blob`,
    );
  }
  if (guards.current) {
    console.log(
      `     telemetry (F12): anchors_per_concept ${guards.current.anchors_per_concept.toFixed(2)}, concepts_per_100_source_lines ${guards.current.concepts_per_100_source_lines.toFixed(2)} (${guards.current.anchors} anchors, ${guards.current.concepts} anchor-owning concepts of ${guards.current.concept_files} files in the area, ${guards.current.source_lines} source lines) — ${guards.telemetryNote}`,
    );
  }
  console.log(
    `     bundle invariants (F12): authority uniqueness, zero not_canonical, and index freshness all hold across docs/knowledge/`,
  );
}

export async function runCheck(area) {
  // Ground truth first: a pin that cannot be asserted stops the check dead.
  // There is no path from here to a green without a verified extraction.
  const derived = derivePinForArea(area);
  if (!derived.ok) {
    reportPinFailure(`--check ${area}`, derived);
    console.error(`FAIL okf_migrate --check ${area}: ground truth could not be derived — refusing to report coverage`);
    return 1;
  }
  const expected = derived.anchors.all;

  // wiringFor is the single source of the dir/source/anchor-pattern quadruple
  // — re-stating any of it here is exactly how the claim matcher fell a
  // widening behind the extractor (f2-3).
  const wiring = wiringFor(area);
  const issues = [];
  const stubRel = `docs/specs/${area}.md`;
  const stubPath = path.join(REPO_ROOT, "docs", "specs", `${area}.md`);
  let stubMap = new Map();
  if (!fs.existsSync(stubPath)) {
    issues.push(`pointer stub missing: ${stubRel} (the path is never deleted — D20)`);
  } else {
    const parsedStub = parseStubAnchorMap(fs.readFileSync(stubPath, "utf8"), wiring.stubAnchorPattern);
    stubMap = parsedStub.map;
    issues.push(...parsedStub.issues);
  }
  const { claims, issues: claimIssues } = await collectClaims(wiring);
  issues.push(...claimIssues);

  // A repaired pin (f2-10) says so wherever it names its own address: the
  // pinned bytes are commit:path PLUS a declared repair, so a reader who runs
  // `git show <commit>:<path>` and gets different bytes is told why here
  // rather than left to discover a disagreement the gate already asserted.
  const repairNote = PIN_REGISTRY[area]?.repaired_from
    ? `, REPAIRED from provenance blob ${PIN_REGISTRY[area].repaired_from.slice(0, 12)} before pinning — the pinned bytes live only in ${PIN_REGISTRY[area].source_copy}`
    : "";

  const cov = coverageIssues({
    expected,
    stubMap,
    claims,
    stubLabel: stubRel,
    registryLabel: `the anchors derived from ${area}'s pinned source (${derived.commit.slice(0, 8)}:${PIN_REGISTRY[area].path}, blob ${derived.blob_sha.slice(0, 12)}, via ${derived.via}${repairNote})`,
  });
  issues.push(...cov.issues);

  const bodies = await readConceptBodies(`docs/knowledge/areas/${area}`, [...claims.values()].flat());
  const guards = await runGuards(area, claims, bodies, derived);
  issues.push(...guards.issues);

  if (issues.length > 0) {
    console.error(`FAIL okf_migrate --check ${area}: ${expected.length} anchors, ${cov.owned} owned, ${cov.duplicated} duplicated, ${cov.lost} lost`);
    for (const i of issues) console.error(`  - ${i}`);
    return 1;
  }
  console.log(`PASS okf_migrate --check ${area}: ${expected.length} anchors, ${cov.owned} owned, 0 duplicated, 0 lost — every source anchor lands in exactly one concept and the stub map agrees (D35/D37)`);
  console.log(`     ground truth DERIVED from pinned blob ${derived.blob_sha} (${derived.commit.slice(0, 8)}:${PIN_REGISTRY[area].path}, via ${derived.via}, scheme ${PIN_REGISTRY[area].scheme}${repairNote}) — counts asserted ${JSON.stringify(derived.counts)}`);
  printGuards(guards);
  return 0;
}

export async function runCheckPatterns() {
  const derived = derivePinForArea("critical-patterns");
  if (!derived.ok) {
    reportPinFailure("--check-patterns", derived);
    console.error("FAIL okf_migrate --check-patterns: ground truth could not be derived — refusing to report coverage");
    return 1;
  }
  const expected = derived.anchors.all;

  const issues = [];
  const stubPath = path.join(REPO_ROOT, ...PATTERNS_SOURCE.split("/"));
  let stubMap = new Map();
  if (!fs.existsSync(stubPath)) {
    issues.push(`pointer stub missing: ${PATTERNS_SOURCE} (the path is never deleted — D20)`);
  } else {
    const parsedStub = parseStubAnchorMap(fs.readFileSync(stubPath, "utf8"));
    stubMap = parsedStub.map;
    issues.push(...parsedStub.issues);
  }
  const { claims, issues: claimIssues } = await collectClaims({
    dir: PATTERNS_CONCEPT_DIR,
    source: PATTERNS_SOURCE,
    anchorPattern: "[A-Z]+\\d+",
  });
  issues.push(...claimIssues);

  const cov = coverageIssues({
    expected,
    stubMap,
    claims,
    stubLabel: PATTERNS_SOURCE,
    registryLabel: `the anchors derived from critical-patterns' pinned source (${derived.commit.slice(0, 8)}:${PATTERNS_SOURCE}, blob ${derived.blob_sha.slice(0, 12)}, via ${derived.via})`,
  });
  issues.push(...cov.issues);

  const bodies = await readConceptBodies(PATTERNS_CONCEPT_DIR, [...claims.values()].flat());
  const guards = await runGuards("critical-patterns", claims, bodies, derived);
  issues.push(...guards.issues);

  if (issues.length > 0) {
    console.error(`FAIL okf_migrate --check-patterns: ${expected.length} anchors, ${cov.owned} owned, ${cov.duplicated} duplicated, ${cov.lost} lost`);
    for (const i of issues) console.error(`  - ${i}`);
    return 1;
  }
  console.log(`PASS okf_migrate --check-patterns: ${expected.length} anchors, ${cov.owned} owned, 0 duplicated, 0 lost — every critical-patterns.md heading lands in exactly one bee.pattern concept and the stub map agrees (D35/D37)`);
  console.log(`     ground truth DERIVED from pinned blob ${derived.blob_sha} (${derived.commit.slice(0, 8)}:${PATTERNS_SOURCE}, via ${derived.via}, scheme flat-pattern-list) — counts asserted ${JSON.stringify(derived.counts)}`);
  printGuards(guards);
  return 0;
}

// ─── CLI ────────────────────────────────────────────────────────────────────

function printDerived(label, result) {
  console.log(
    JSON.stringify(
      {
        pin: label,
        ok: result.ok,
        via: result.via,
        blob_sha: result.blob_sha ?? null,
        counts: result.counts,
        unparsed: result.unparsed,
        issues: result.issues,
      },
      null,
      2,
    ),
  );
}

async function main() {
  const args = process.argv.slice(2);

  if (args[0] === "--inventory" && args[1]) {
    const abs = path.resolve(REPO_ROOT, args[1]);
    if (!fs.existsSync(abs)) {
      console.error(`okf_migrate --inventory: no such file: ${args[1]}`);
      return 1;
    }
    const inv = inventorySpec(fs.readFileSync(abs, "utf8"));
    console.log(
      JSON.stringify(
        {
          file: args[1],
          scheme: "ba-nine-section",
          counts: {
            behaviors: inv.behaviors.length,
            rules: inv.rules.length,
            edges: inv.edges.length,
            pointers: inv.pointers.length,
            total: inv.all.length,
          },
          // Unparsed blocks are the honesty signal: a block-starting line in
          // an anchor-bearing section that no rule classified. A non-zero
          // count means this scheme cannot see part of this file — the
          // anchors are not missing from the world, they are missing from the
          // extractor.
          unparsed: inv.unparsed,
          anchors: {
            behaviors: inv.behaviors,
            rules: inv.rules,
            edges: inv.edges,
            pointers: inv.pointers,
            all: inv.all,
          },
        },
        null,
        2,
      ),
    );
    return 0;
  }

  if (args[0] === "--inventory-pin" && args[1]) {
    const result = derivePinForArea(args[1]);
    printDerived(args[1], result);
    if (!result.ok) {
      reportPinFailure(args[1], result);
      return 1;
    }
    return 0;
  }

  if (args[0] === "--derive" && args[1]) {
    let pin;
    try {
      pin = JSON.parse(args[1]);
    } catch (error) {
      console.error(`okf_migrate --derive: pin argument is not valid JSON: ${error.message}`);
      return 1;
    }
    const label = pin.path || "<ad-hoc pin>";
    const result = derivePin(pin, label);
    printDerived(label, result);
    if (!result.ok) {
      reportPinFailure(label, result);
      return 1;
    }
    console.log(`PASS okf_migrate --derive ${label}: ${result.counts.total} anchors from blob ${pin.blob_sha} via ${result.via}`);
    return 0;
  }

  if (args[0] === "--verify-pins") {
    let bad = 0;
    for (const [area, pin] of Object.entries(PIN_REGISTRY)) {
      if (pin.scheme === null) {
        console.log(`SKIP-REFUSED ${area}: no anchor scheme declared — this area cannot be gated yet (F9/S5)`);
        continue;
      }
      const result = derivePin(pin, area);
      if (!result.ok) {
        bad += 1;
        reportPinFailure(area, result);
        continue;
      }
      console.log(
        `PASS ${area}: ${result.counts.total} anchors ${JSON.stringify(result.counts)} from blob ${pin.blob_sha} via ${result.via} (scheme ${pin.scheme})`,
      );
    }
    if (bad > 0) {
      console.error(`FAIL okf_migrate --verify-pins: ${bad} pin(s) could not be asserted`);
      return 1;
    }
    console.log("PASS okf_migrate --verify-pins: every declared pin resolves, hashes, extracts, and matches its expected counts");
    return 0;
  }

  if (args[0] === "--fidelity" && args[1]) {
    const result = await areaFidelity(args[1]);
    console.log(
      JSON.stringify(
        {
          area: args[1],
          ok: result.ok && result.issues.length === 0,
          floor: FIDELITY_FLOOR,
          stats: result.stats,
          rows: result.rows.map((r) => ({
            anchor: r.anchor,
            owner: r.owner,
            ratio: Number(r.ratio.toFixed(4)),
            anchor_tokens: r.anchor_tokens,
            missing_tokens: r.missing_tokens,
          })),
          issues: result.issues,
        },
        null,
        2,
      ),
    );
    if (!result.ok || result.issues.length > 0) {
      for (const i of result.issues) console.error(`  - ${i}`);
      console.error(`FAIL okf_migrate --fidelity ${args[1]}: ${result.issues.length} anchor(s) below the ${FIDELITY_FLOOR} floor (F11)`);
      return 1;
    }
    return 0;
  }

  if (args[0] === "--telemetry") {
    const rows = await collectTelemetry();
    const bundle = runBundleInvariants();
    console.log(
      JSON.stringify(
        {
          pinned_areas: rows.length,
          min_samples_for_a_median: TELEMETRY_MIN_SAMPLES,
          // Per shape, not overall (f2-3; keyed on the scheme since f2-11): a
          // median is only drawn across pins of the same shape, so this
          // reports which shapes have enough comparable samples to gate on
          // rather than one global boolean.
          median_available_by_scheme: Object.fromEntries(
            [...new Set(rows.map((r) => r.scheme))].map((scheme) => [
              scheme,
              rows.filter((r) => r.scheme === scheme).length >= TELEMETRY_MIN_SAMPLES,
            ]),
          ),
          rows,
          bundle_issues: bundle.issues,
        },
        null,
        2,
      ),
    );
    return bundle.issues.length > 0 ? 1 : 0;
  }

  if (args[0] === "--check" && args[1]) {
    return runCheck(args[1]);
  }
  if (args[0] === "--check-patterns") {
    return runCheckPatterns();
  }
  console.error(
    "usage: okf_migrate.mjs (--inventory <spec-path> | --inventory-pin <area> | --derive <pin-json> | --verify-pins | --fidelity <area> | --telemetry | --check <area> | --check-patterns)",
  );
  return 1;
}

const isMain = process.argv[1] && pathToFileURL(process.argv[1]).href === import.meta.url;
if (isMain) {
  process.exitCode = await main();
}
