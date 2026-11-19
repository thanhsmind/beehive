<!-- sha256: 1d4b1b75abcb6e894da5de9115d25a4ba0398e0a0413c7577fae62dd3f837cdc -->
# Generated command contract

Generated from `src/core/command-signatures.ts` through `ui schema --json`. Do not edit: validate.mjs
re-renders this file from the installed `ui` and refuses an edited or stale copy. Flags not listed here do not exist.

## ui gate

Run all linter families + autofix dry-run over an HTML file; exit 1 on any error-severity finding

Usage: `ui gate <file.html> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--tokens` | string | DS token file; enables the taste Consistency raw-hex check and grades required token-coverage against THIS file |
| `--family` | string | Persona family slug — applies its gate_policy from design-os:personas/families.json |
| `--skip` | string | Comma-separated <family>:<reason> pairs — declared partial gating |
| `--dir` | string | Project directory to probe for tokens/DS (default: the file's folder; coverage mode: cwd) |
| `--token-coverage-floor` | string | Minimum ui token-coverage score before the gate fails (default: 0.8) |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`, `TOKENS_NOT_READABLE`, `FAMILY_NOT_FOUND`

## ui validate-layout

Lint an HTML file; exit 1 on error-severity findings

Usage: `ui validate-layout <file.html>`

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui a11y-lint

Lint an HTML file for static a11y violations; exit 1 on error-severity findings

Usage: `ui a11y-lint <file.html>`

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui taste-lint

Check the machine-verifiable taste-rubric subset; exit 1 on violations

Usage: `ui taste-lint <file.html> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--tokens` | string | DS token file; enables the Consistency raw-hex check |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui tell-lint

Report design tells across HTML, and (as extractors land) JSX, Vue, Swift and Dart

Usage: `ui tell-lint [target] [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--coverage` | boolean | Print the rule x extractor matrix and exit |
| `--no-advisory` | boolean | Hide advisory findings (they never affect the exit code) |
| `--render` | boolean | Also run the 7 rendered rules, driving a browser already on this machine |
| `--browser` | string | Path to Chrome/Chromium/Edge; defaults to $CHROME_PATH then the platform's usual paths |
| `--viewport` | string | Viewport for the rendered pass, e.g. 390x844 (default 1280x800) |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui content-lint

Lint an HTML file for content defects; exit 1 on error-severity findings

Usage: `ui content-lint <file.html>`

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui autofix

Apply the 5 deterministic HTML fix rules to a file

Usage: `ui autofix <file.html> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--write` | boolean | Overwrite the input file (default: print to stdout) |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`, `WRITE_ERROR`

## ui token-coverage

Score an HTML file's CSS (color/font-family/font-size/spacing/radius/shadow) against a token file; exit 1 below floor

Usage: `ui token-coverage <file.html> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--tokens` | string | Token file to grade against (default: auto-detect via the design-dir contract, then brand/design/design.tokens.json) |
| `--floor` | string | Minimum overall coverage to exit 0 (default: 0.8) |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`, `TOKENS_NOT_FOUND`, `BAD_JSON`

## ui ds context

Emit the active design system as a context block for the host model

Usage: `ui ds context [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--strict` | boolean | Prepend the registered-components-only enforcement preamble |
| `--with-theme` | boolean | Also emit the compiled Tailwind v4 @theme block (full token map, immune to --max-bytes) |
| `--include` | string | Comma-separated sections: tokens,registry,naming,anti-patterns,soul (default); guidelines is opt-in: pack file index, scenario titles and tier-A rulings within --max-bytes |
| `--format` | markdown\|json | Output format (default markdown) |
| `--max-bytes` | string | Budget for the variable sections (token/registry/naming/anti-pattern tables); the soul chain and --with-theme are fixed prose outside it (default 4096) |
| `--dir` | string | Override the project directory |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `DS_NOT_FOUND`, `DS_TAMPERED`, `BAD_MANIFEST`

## ui registry list

List all components, optionally filtered by category

Usage: `ui registry list [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--category` | string | Filter by category |
| `--file` | string | Registry file path |

Error codes: `BAD_ARG`, `REGISTRY_NOT_FOUND`, `BAD_REGISTRY`, `READ_ERROR`

## ui registry lookup

Find a component by canonical name

Usage: `ui registry lookup <name> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--file` | string | Registry file path |

Error codes: `BAD_ARG`, `BAD_NAME`, `NOT_FOUND`, `REGISTRY_NOT_FOUND`, `BAD_REGISTRY`, `READ_ERROR`

## ui memory context

Emit a compact memory prior for the host model

Usage: `ui memory context [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--for` | generate\|critique\|why | Consumer mode (default generate) |
| `--rank-file` | string | JSON array of ranked event ids to splice into the prior (never for --for critique) |
| `--components` | string | Exact owner component names as CSV or a JSON string array |
| `--patterns` | string | Exact owner pattern names as CSV or a JSON string array |
| `--max-bytes` | string | Truncate the block, sections whole (default 2048) |
| `--now` | string | Decay clock (deterministic when fixed) |
| `--dir` | string | Project directory (default: cwd) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `BAD_LEDGER`, `BAD_LESSON`, `FILE_NOT_FOUND`, `READ_ERROR`, `CONTEXT_OVERFLOW`, `DS_TAMPERED`, `BAD_MANIFEST`, `BAD_DS`

## Added for the core references

The entries from here to the end are not from the upstream render. bee added them from `ui schema --json` so every `ui` command the core references name is covered; the sha256 header above does not cover them, and a re-sync of this file drops them until `scripts/design-os-contract.mjs` carries the list. `ui gate coverage` is the exception: `ui schema` has no entry for it, so its entry is taken from `ui gate --help`.

## ui gate coverage

List every check the gate can run, with per-project activity

Usage: `ui gate coverage [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory to probe for tokens/DS (default: cwd) |

Error codes: not listed; `ui gate --help` names none for this mode

## ui vr gate

Diff every baseline PNG against the same-named current render; exit 1 on any regression

Usage: `ui vr gate <baseline-dir> <current-dir> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--threshold` | string | Per-pixel matching tolerance, 0–1 (default 0.1) |
| `--max-ratio` | string | Max changed-pixel ratio, 0–1, that still passes (default 0) |
| `--include-aa` | boolean | Count anti-aliased pixels as real differences |
| `--out-dir` | string | Write per-file diff PNGs into this directory |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_PNG`

## ui vr diff

Compare two PNGs; exit 1 if the changed-pixel ratio exceeds --max-ratio

Usage: `ui vr diff <base.png> <head.png> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--threshold` | string | Per-pixel matching tolerance, 0–1 (default 0.1) |
| `--max-ratio` | string | Max changed-pixel ratio, 0–1, that still passes (default 0) |
| `--include-aa` | boolean | Count anti-aliased pixels as real differences |
| `--mask` | string | Rectangles to ignore: "x,y,w,h" separated by ";" |
| `--out` | string | Write the diff PNG to this file |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_PNG`, `BAD_MASK`

## ui vr accept

Promote current renders to baselines (copies *.png current → baseline)

Usage: `ui vr accept <current-dir> <baseline-dir>`

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`

## ui ds soul

Scaffold + structure-lint the declared design stance (design/soul.md); 'factory' prints the shipped design:os baseline

Usage: `ui ds soul <init|check|factory> [flags]`

Invocations: `ui ds soul init`, `ui ds soul check`, `ui ds soul factory`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: cwd) |
| `--force` | boolean | 'soul init' only: overwrite an existing soul.md |
| `--studio` | boolean | Target $EASE_DESIGN_HOME/studio-soul.md instead of the project soul; conflicts with --dir |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `EXISTS`, `WRITE_ERROR`, `READ_ERROR`

## ui ds a11y

Token-pair contrast audit (text×surface ≥ AA, incl. hover/active state pairs); declared tokens only

Usage: `ui ds a11y [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: cwd) |
| `--pairs` | string | Pin explicit text:surface token pairs (skips name-role inference) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `DS_NOT_FOUND`, `BAD_JSON`

## ui ds guidelines

Compile the sealed design system into a guidelines pack (design/guidelines/) and lint it

Usage: `ui ds guidelines <build|check|export> [flags]`

Invocations: `ui ds guidelines build`, `ui ds guidelines check`, `ui ds guidelines export`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: discovered from cwd) |
| `--out` | string | build/check: pack directory (default: <project>/design/guidelines); export: destination folder holding figma-guidelines/ and UPLOAD.md (default: <project>/design/figma-export) |
| `--pack` | string | 'export' only: the pack directory to export (default: <project>/design/guidelines) |
| `--target` | figma | 'export' only: the bundle target (figma is the only one) |
| `--budget` | string | 'export' only: trim the bundle to <n>kb (n <= 200) in priority order, reporting every drop; no trimming without it |
| `--profile` | figma | 'check' only: with --bundle, validate an exported bundle against Figma's limits instead of a pack |
| `--bundle` | string | 'check --profile figma' only: the exported figma-guidelines folder |
| `--rulings` | string | Owner rulings file (default: <project>/design/knowledge/rulings.json); absent = omitted + skippedSources |
| `--allow-unsealed` | boolean | Proceed when the store no longer matches its sealed hashes (manifest seal.status unverified; check warns SEAL_UNVERIFIED) |
| `--now` | string | 'build' only: ISO clock written to the manifest; same store + same --now = identical bytes |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `DS_NOT_FOUND`, `DS_TAMPERED`, `BAD_MANIFEST`, `PACK_NOT_FOUND`, `READ_ERROR`, `WRITE_ERROR`, `BAD_RULINGS`, `UNKNOWN_NAME`, `GENERATED_BLOCK_TAMPERED`, `SUPERSEDED_CITED`, `AVOIDED_TERM`, `DUPLICATE_RULE`, `PACK_INCOMPLETE`, `SEAL_UNVERIFIED`, `SCENARIO_INCOMPLETE`, `OPEN_QUESTION`, `UNBOUND_KNOWLEDGE`, `ORPHANED_ANSWERS`, `STALE_FILE`, `OPEN_QUESTIONS`, `PACK_INVALID`, `BUDGET_EXCEEDED`, `OVER_SOFT_BUDGET`, `BUNDLE_FILE_COUNT`, `BUNDLE_EXTENSION`, `BUNDLE_DIR_NOT_CLEAN`, `BUNDLE_NOT_FOUND`, `BAD_ANSWERS_STORE`, `BUILD_FAILED`, `EXPORT_FAILED`

## ui ds import

Onboard an existing flat tokens.json into the DTCG store

Usage: `ui ds import <tokens.json> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory to write design/ into (default cwd) |
| `--name` | string | DS name for the sealed manifest (default imported-ds) |
| `--force` | boolean | Overwrite an existing design.tokens.json |
| `--reset-registry` | boolean | Confirm --force may wipe a non-empty registry (required alongside --force when the registry has components) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `BAD_NAME`, `FILE_NOT_FOUND`, `BAD_JSON`, `EXISTS`, `REGISTRY_NOT_EMPTY`, `EMPTY_IMPORT`, `KIT_DOWNGRADE`, `WRITE_ERROR`

## ui ds init

Compile a project-scoped design system from a persona + intent

Usage: `ui ds init <name> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--persona` (required) | string | Persona slug from personas.json |
| `--intent` (required) | string | Plain-language design intent (max 512 chars) |
| `--brand-hex` | string | Brand seed color (#RRGGBB) |
| `--dir` | string | Override the project directory |
| `--force` | boolean | Overwrite an existing DS (preserves changelog) |
| `--bare` | boolean | Skip the default component kit (start with an empty registry) |
| `--persona-data` | string | Override the personas.json path (test support) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `BAD_NAME`, `BAD_INTENT`, `BAD_BRAND_HEX`, `PERSONA_NOT_FOUND`, `BAD_TOKEN`, `DS_EXISTS`, `KIT_DOWNGRADE`, `WRITE_ERROR`

## ui ds diff

Compare two DS states → semver + visual-breaking classification

Usage: `ui ds diff <base-dir> <head-dir> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--format` | markdown\|json\|pr-comment | Output format (default markdown) |
| `--base-version` | string | Base semver to compute the recommended next version from |
| `--color-tolerance` | string | OKLab ΔE below which a colour change is a patch (default 0.02) |
| `--dim-tolerance` | string | Dimension % change below which it is a patch (default 5) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `BAD_JSON`, `READ_ERROR`

## ui ds docs

Regenerate component reference docs from the registry (decay-proof)

Usage: `ui ds docs [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: cwd) |
| `--out` | string | Write the docs to a file instead of stdout |
| `--format` | markdown\|json | Output format (default markdown) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `REGISTRY_NOT_FOUND`, `BAD_REGISTRY`, `WRITE_ERROR`

## ui ds reseal

Re-seal a store whose files drifted from the manifest hashes (preview by default; --apply --reason writes)

Usage: `ui ds reseal [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--apply` | boolean | Adopt the on-disk hashes into the manifest (default: preview, writes nothing) |
| `--reason` | string | Required with --apply (max 256 chars); recorded in the manifest changelog |
| `--dir` | string | Override the project directory |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `RESEAL_REASON_REQUIRED`, `RESEAL_REGISTRY_SWITCHED`, `RESEAL_KIT_BOUND`, `DS_NOT_FOUND`, `DS_TAMPERED`, `BAD_MANIFEST`, `BAD_REGISTRY`, `BAD_TOKEN`, `BAD_JSON`, `READ_ERROR`, `DANGLING_ALIAS`, `ALIAS_CYCLE`, `TYPE_MISMATCH`, `WRITE_ERROR`

## ui ds kit

Stage, validate, adopt and verify a complete source-bound owner component kit

Usage: `ui ds kit <plan|theme|prepare|validate|adopt|verify> [<candidate-root>] [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--source` | string | plan: raw Figma scan or normalized code registry |
| `--kind` | figma\|registry | plan: source capture contract |
| `--tokens` | string | plan: captured DTCG token file |
| `--out` | string | plan/adopt: absent destination, never overwritten |
| `--name` | string | plan: exact owner DS display name |
| `--dir` | string | verify: adopted project directory |
| `--now` | string | adopt: explicit ISO clock for deterministic manifest output |

Error codes: `BAD_ARG`, `BAD_NAME`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_JSON`, `BAD_KIT`, `KIT_FORMAT`, `KIT_PATH`, `KIT_IO`, `KIT_SOURCE`, `KIT_TOKENS`, `KIT_THEME`, `KIT_ALIAS`, `KIT_MODE`, `KIT_UNDECLARED`, `KIT_BLOCKED`, `KIT_EVIDENCE`, `KIT_EVIDENCE_STALE`, `KIT_TAMPERED`, `KIT_CHANGED`, `KIT_NOT_BOUND`, `KIT_TARGET_EXISTS`, `KIT_PLAN_INCOMPLETE`, `KIT_ADOPTION_INCOMPLETE`, `DS_NOT_FOUND`, `DS_TAMPERED`, `BAD_MANIFEST`, `WRITE_ERROR`

## ui evidence add

Append a finding; a quote must be a verbatim substring of its source

Usage: `ui evidence add [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--kind` | quote\|metric\|observation | Evidence kind (default quote) |
| `--finding` | string | The synthesised insight (required) |
| `--quote` | string | Verbatim source text (required for kind=quote) |
| `--source` | string | Source file to ingest and verify against |
| `--medium` | string | interview \| survey \| analytics \| support \| review \| … |
| `--locator` | string | Where in the source (line 42, 0:14:20) — provenance only |
| `--metric` | string | Metric value (required for kind=metric) |
| `--unit` | string | Metric unit |
| `--n` | string | Sample size |
| `--tags` | string | Comma-separated tags |
| `--dir` | string | Evidence store directory (default 'design') |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `QUOTE_TOO_SHORT`, `QUOTE_MISMATCH`, `SOURCE_COLLISION`, `BAD_EVIDENCE`

## ui evidence list

List every recorded finding with support level and verify status

Usage: `ui evidence list [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Evidence store directory (default 'design') |

Error codes: `UNKNOWN_FLAG`, `READ_ERROR`, `BAD_EVIDENCE`

## ui evidence show

Print one finding by id

Usage: `ui evidence show <id> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Evidence store directory (default 'design') |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `READ_ERROR`, `NOT_FOUND`, `BAD_EVIDENCE`

## ui evidence verify

Re-check every quote finding against its stored source; exit 1 on any break

Usage: `ui evidence verify [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Evidence store directory (default 'design') |

Error codes: `UNKNOWN_FLAG`, `READ_ERROR`, `SOURCE_MISSING`, `BAD_EVIDENCE`

## ui critique-coverage

Report uncovered acceptance criteria + coverage %; exit 1 on any gap

Usage: `ui critique-coverage <spec.json> <manifest.json> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--require-evidence` | boolean | Treat criteria with no evidence provenance as assumptions, not real coverage |
| `--evidence-dir` | string | Resolve criterion evidence[] as ids in the T6 evidence ledger at DIR |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_JSON`, `BAD_EVIDENCE`

## ui reuse-plan check

Validate a reuse plan's names and token paths against the project registry; exit 1 on findings, 2 when it cannot run

Usage: `ui reuse-plan check <file> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: cwd) |
| `--file` | string | Registry file to check against (same as 'ui registry list/lookup --file') |
| `--tokens` | string | Token file for rung 6 (default: <project>/design/design.tokens.json) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `REGISTRY_NOT_FOUND`, `BAD_REGISTRY`

## ui delivery validate

Validate a design brief, generation contract, or qualification record

Usage: `ui delivery validate <file.json>`

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_DELIVERY`

## ui build-evidence lint

Check r1-r8 over a builder's evidence folder; PASS/FAIL/SKIPPED per rule

Usage: `ui build-evidence lint <dir> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--widths` | string | Comma-separated required widths (default 375,768,1440) |
| `--mode` | string | "persuade" forces r8 (Persuade evidence) on |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `DIR_NOT_FOUND`

## ui changelog

Fold the DS manifest changelog + recorded decisions into a Keep-a-Changelog history

Usage: `ui changelog [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Project directory holding design/ (default: cwd) |
| `--format` | markdown\|json | Output format (default markdown) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `NO_MEMORY`, `BAD_JSON`

## ui audit

Audit a node-export JSON against a DS spec; exit 1 on any violation

Usage: `ui audit <nodes.json> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--tokens` | string | DTCG token file; enables the raw-hex-vs-token check |
| `--registry` | string | Component registry; enables detached/raw-icon/deprecated checks |
| `--grid` | string | Base grid for off-grid radius/spacing (default 4) |

Error codes: `BAD_ARG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_JSON`

## ui schema

Emit the typed invocation contract (flags, positionals, enums, error codes)

Usage: `ui schema`

Error codes: none

## ui synthesize-conventions

Compile a scan-conventions usage-dna.json into a CONVENTIONS.md of measured DO/DON'T

Usage: `ui synthesize-conventions <usage-dna.json> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--ds` | string | DTCG tokens.json; cross-references the DS scale/grid to split valid values from real deviations |
| `--out` | string | Output directory (default: current working directory) |
| `--seed-memory` | boolean | Also seed 'ui memory' (a harvested anchor + prefers/avoids insight events) |
| `--now` | string | Deterministic clock for the seeded memory graph (ISO-8601) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `READ_ERROR`, `BAD_JSON`, `BAD_DNA`, `BAD_DS`, `MEMORY_LOCKED`, `MEMORY_COMMITTED`, `WRITE_ERROR`

## ui knowledge activate

Resolve a typed artifact surface into a route, assurance level, and delivery-claim policy

Usage: `ui knowledge activate <request.json> [flags]`

| Flag | Type | Meaning |
| --- | --- | --- |
| `--dir` | string | Repo root holding knowledge/capability-profiles.json (default: packaged knowledge) |

Error codes: `BAD_ARG`, `UNKNOWN_FLAG`, `FILE_NOT_FOUND`, `BAD_ACTIVATION`, `NO_CATALOG`, `BAD_CATALOG`, `UNKNOWN_CAPABILITY`, `UNSUPPORTED_INPUT`, `CAPABILITY_UNQUALIFIED`

## Not for host work

The core references name these commands. Do not run them in a host.

- `ui knowledge check` — it lints upstream's own knowledge folder, not host work
- `ui init` — it writes design-os's own adapter skills and commands, which bee's packs replace
