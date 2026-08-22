---
type: bee.area
title: Bee OKF Profile — the two-level conformance check and its finding codes
description: "How a bundle is graded: OKF errors versus the profile's own errors and warnings, the exact code each finding carries, the emitter-first codec behind them, and the checker's never-writes boundary."
timestamp: 2026-07-22
bee:
  id: okf-profile-conformance-check
  lifecycle: active
  areas: [okf-profile]
  required_context: [areas/okf-profile/overview.md]
  decisions: [D2, D4, D10, D12, D13, D18, D19, D23, D31, G14 (okf-switchover-f3 — the profile-error severity that fails a non-strict chain run)]
  sources: ["okf-foundation cell okf-1 (knowledge.mjs core — emitter-first frontmatter codec, concept model, two-level check verb; trace in `.bee/cells/`, report `docs/history/okf-foundation/reports/`, 2026-07-22)", CONTEXT.md `docs/history/okf-foundation/CONTEXT.md`, "docs/specs/okf-profile.md#B1", "docs/specs/okf-profile.md#B2", "docs/specs/okf-profile.md#B3", "docs/specs/okf-profile.md#B4", "docs/specs/okf-profile.md#E2", "docs/specs/okf-profile.md#P1"]
  authoritative_for: "okf-profile: the two-level conformance check and its finding codes"
---

# Bee OKF Profile — The Two-Level Conformance Check and Its Finding Codes

This concept owns how a bundle is graded: the two severities the check reports, the exact codes it
emits, the emitter-first codec that makes those codes possible at zero dependency cost, and the
boundary that keeps the checker a reader and never a writer.

## Behaviors & Operations

**B1 — Two-level check, OKF errors vs. profile findings (D4).** `bee knowledge check` grades every
`.md` inside `docs/knowledge/` (D23). **OKF errors** are the spec's own MUSTs; the **profile** layer
is bee's own, and it has two severities: **profile errors**, which fail on their own, and **profile
warnings**, bee's SHOULD layer, which fail only under `--strict` (`--strict` promotes every warning
to an error). The `--json` shape is
`{okf:{errors:[...]}, profile:{errors:[...], warnings:[...]}, counts}`, and the command exits
non-zero when an OKF error exists, when a profile error exists, or (under `--strict`) when any
finding at all exists. `counts` carries `files`, `concepts`, `errors` (OKF), `profile_errors`, and
`warnings`.

The profile-error severity exists because the chain runs this verb **without** `--strict` (D13), so
an invariant parked in `warnings` is decorative. Exactly the findings that guard *one subject, one
readable authority* live there — see the profile-error table in B2.

**B2 — Exact finding codes emitted by `.bee/bin/lib/knowledge.mjs`.**

OKF-error codes (each finding also carries `file` and a human `message`):

| Code | Fires when |
|---|---|
| `unreadable` | The file could not be read at all (a filesystem failure, not itself an OKF clause — but an unreadable file cannot be graded conformant, so it is surfaced as an error). |
| `missing_frontmatter` | A non-reserved `.md` inside the bundle (a concept, D23) has no leading `---` block. |
| `unparseable_frontmatter` | A concept's or the root `index.md`'s frontmatter starts but fails to parse under the emitted subset (D12) — the underlying parser error code and line are folded into the message. |
| `empty_type` | `type` is absent, not a string, or blank (OKF §4.1 MUST). |
| `index_frontmatter` | A **non-root** `index.md` carries any frontmatter at all (OKF §6: index files carry none). |
| `root_index_extra_keys` | The **root** `index.md` carries any key besides `okf_version` (OKF §9). |
| `log_heading_not_iso` | A `## `-level heading inside `log.md` is not an ISO 8601 date (OKF §7 MUST). |

Profile-**error** codes — reported in `profile.errors` and **chain-failing on their own**, with no
`--strict` (G14 layer 3, D4). The chain runs `knowledge check` non-strict by design, so a finding that
guards single-home rules and anti-fork invariants cannot live in `warnings`: a backstop that never
blocks is not a backstop. Exempt trees (`docs/history/`, `docs/discovery/`, `docs/specs/`, `.bee/`)
are skipped during rule marker and copy scans.

| Code | Fires when |
|---|---|
| `duplicate_authoritative_for` | Two or more concepts claim the same `bee.authoritative_for` subject (D31). Grouped by the **hardened subject skeleton** (NFKC + lowercase + accent strip + confusable fold + punctuation/whitespace collapse), not the raw string — so claims differing only by a trailing period, case, or a Cyrillic homoglyph are one subject with two authorities, not two subjects. |
| `malformed_authoritative_for` | A concept's `bee.authoritative_for` is present but is not one non-empty string (a list, a boolean, an empty or blank string). A claim bee cannot read is an owner the anti-fork gate cannot see, so it is never silently skipped. |
| `dangling_applied_at` (knowledge-one-home koh-2, D4) | A `bee.applied_at` path (trailing-`*` glob allowed) does not resolve to an existing file inside the bundle or at the repo root. With no repo root, out-of-bundle targets are skipped and noted. |
| `dangling_owns` (knowledge-one-home koh-2, D4) | A `bee.owns.*` path (trailing-`*` glob allowed) does not resolve to an existing file or directory inside the bundle or at the repo root. With no repo root, out-of-bundle targets are skipped and noted. |
| `duplicate_rule_home` (knowledge-one-home koh-2, D4) | The same `<!-- rule: <id> -->` opening marker appears in two or more files across the bundle, `AGENTS.md`, and `skills/**`. Exempt trees are skipped. |
| `unknown_rule_ref` (knowledge-one-home koh-2, D4) | A `(rule: <id>)` reference appears in a scanned file whose `<id>` is homed by no `<!-- rule: <id> -->` marker in the bundle or `AGENTS.md`. Exempt trees are skipped. |
| `applied_at_unlinked` (knowledge-one-home koh-2, D4) | A file listed in a concept's `bee.applied_at` contains no `(rule: <id>)` reference for any rule homed in that concept. |
| `owns_missing` (knowledge-one-home koh-2, D4) | An area overview concept (`areas/<area>/overview.md` of type `bee.area`) carries none of the `owns.*` keys (`owns.code`, `owns.skills`, `owns.tests`). |

Profile-warning codes — reported always, failing only under `--strict`:

| Code | Fires when |
|---|---|
| `unknown_type` | `type` parses but is outside the nine D18 types — an OKF consumer must tolerate it; bee flags it. |
| `missing_profile_field` | One of `title`, `description`, `bee.id`, `bee.lifecycle` is missing or blank (D10: never invented — must be authored). |
| `not_canonical` | Parsing the file's frontmatter and re-emitting it (`emitFrontmatter`) does not byte-match the file — a hand-edited colon/`#`/CRLF/key-order outside the canonical emitted form (the advisor round-trip guard). |
| `duplicate_id` | Two or more concepts share one `bee.id` (D31). |
| `dangling_required_context` (knowledge-distill-trigger kdt-1) | A `bee.required_context` entry does not resolve to a real file inside the bundle OR against the repo root (same two-step order `dangling source` uses) — a promote-generated `docs/history/<feature>/...` pointer that exists on disk is not dangling. Both dangling codes also feed the knowledge-freshness close door: findings scoped to a closing feature's touched areas plus its `work/<feature>/` files block `bee close` unless a recorded deferral decision names the feature. |
| `dangling_supersedes` | A `bee.supersedes` id matches no concept's `bee.id` in the bundle. |
| dangling source (knowledge-usable U2, cell ku-2, 2026-08-10) | A `bee.sources` entry that names an in-repo file path resolving to no file on disk is reported, so a concept citing code a later port retired degrades visibly instead of silently. |
| `dangling_md_link` (knowledge-link-check klc-1, 2026-08-11) | A relative body link ending in `.md` (external `http(s)`/`mailto`, absolute paths, and anchor-only links are skipped) resolved against the containing file's directory does not exist or escapes the bundle. |
| `dangling_wiki_link` (knowledge-link-check klc-1, 2026-08-11) | A `[[target]]` body reference where neither `target` nor `target` minus an optional `pattern-` prefix matches the stem of any `.md` in the bundle. Code fences and inline backtick spans are stripped first — quoted syntax never warns. A host-repo integration test keeps the shipped bundle clean of both link codes. |
| `invalid_evidence_state` (evidence-ladder el-1, 2026-08-11) | A `bee.evidence` value outside present/wired/exercised, named by file and value. Absent stays valid and reads as present; `bee.evidence_ref` names the enforcing hook/guard/doctor check/test. `bee knowledge report` surfaces the ladder — per-state counts over patterns plus the present-only list — so a doc-only pattern is visible risk, never a silent default. |

**B3 — Emitter-first parsing, zero dependencies (D12).** `knowledge.mjs` ships its own frontmatter
codec covering exactly the YAML subset its own emitter can produce; anything outside that subset
fails loudly with a typed `{code, message, line}`, never a silent misparse. This is strictly easier
than parsing the three incompatible legacy frontmatter schemas across 114 files, because the bundle
is authored fresh (D20) — the subset is one bee controls.

**B4 — The bundle is read-only from the checker's side (D2).** `knowledge.mjs` performs no writes
at all, apart from `index`'s generated `index.md` files inside the bundle, and never writes into
`.bee/*.json(l)` runtime stores. The bundle is the knowledge layer; it is never a write path into
runtime state, though reads from it are permitted.

## Business Rules

- The checker never leaves `docs/knowledge/` (D23) and never writes anything, anywhere (D2).

## Edge Cases Settled

- A hand-edited frontmatter block that still parses but does not re-emit byte-identically is a
  `not_canonical` warning naming the file, not a silent pass — the class of error a colon in an
  unquoted title, a `#` mid-value, or CRLF line endings would otherwise cause.

## Pointers (implementation)

- Checker + emitter-first codec + concept model: `.bee/bin/lib/knowledge.mjs` (mirrored from
  `packages/bee/lib/knowledge.mjs` — `scripts/tests/test_lib_mirror.mjs:196` enforces the
  mirror). `checkBundle` is the two-level check; `emitFrontmatter`/`parseFrontmatter` are the D12
  codec; `CONCEPT_TYPES`/`LIFECYCLES`/`PROFILE_REQUIRED` are the D18/D19 tables.
