# Research Protocol — Step Rules

Load only after `bee-researching` is active.

## Best For

- unfamiliar repos or territory the repo has no precedent for
- ambiguous or version-sensitive integrations
- capabilities that might already be supported by the repo or its framework
- high-risk work where a wrong assumption is the expensive path

Not for: tiny obvious edits, mechanical renames, product-decision locking (`bee-shaping`), feasibility proofs and spikes (the opt-in `spike` lane), or re-research after a brief is accepted.

## 1. Stack Ledger

Classify the repo from evidence: app, service, package, plugin, library, CLI, infrastructure, automation, or mixed monorepo. Build the ledger from artifacts:

- manifests and lockfiles: `package.json`, `pyproject.toml`, `Cargo.toml`, `go.mod`, workspace files
- build/runtime config: `tsconfig*`, Docker/compose, CI workflows, plugin/MCP manifests
- contracts and docs: `AGENTS.md`, `README.md`, repo-local architecture docs
- tests, scripts, entrypoints

Capture languages, runtimes, frameworks, packaging shape, major tools, external services, and verification commands. Verify installed binary/lockfile versions when runtime behavior depends on them and the check is cheap.

## 2. Local Reuse

Inspect feature-adjacent code, tests, scripts, docs, workflows, experiments, config, and env validation. Repo-intelligence tools (code-graph capability, grep) accelerate; the files that prove behavior are what count. The step must answer:

- what already exists
- what can be reused as-is
- which extension points are available
- what is genuinely missing

Do not claim something is missing until likely code, config, docs, and tests have been checked.

## 3. Upstream Patterns

Capability: `upstream-pattern-research` — best-effort and non-blocking; fallback is reading the public repos directly. A capability gap is never a reason to skip the step silently.

Prefer sources close to the repo's stack: the framework repo, the library repo, official starters, near-identical integration examples. The goal is reusable proof, not generic inspiration. Note how closely each pattern matches this repo's shape and version.

## 4. Current Official Docs

Capability: `web-docs-search` — fallback WebSearch/WebFetch/browser. Bias toward official domains and docs matching the repo's installed versions; use beta/canary guidance only when specifically relevant. A secondary write-up is not evidence: follow every claim back to the source that owns it — official docs, the source code, the spec, the first-party API. The step answers:

- does the framework/library already support the requested capability?
- what is the currently recommended API or workflow?
- which version caveats, deprecations, or migration risks apply to *this* repo?

Local behavior beats docs when they disagree — record the mismatch. With no web capability at all, label the affected claims `Inference` and route each as an open question into `bee-planning`'s shape gate as a proof obligation.

## Tool Roles

| Need | Primary path | Rule |
|---|---|---|
| Current repo truth | Local files, manifests, configs, tests, scripts | First and required |
| Public patterns | `upstream-pattern-research` capability | Best-effort, non-blocking |
| Current official guidance | `web-docs-search` capability | Official, version-aware |
| Synthesis | approach (in-chain) / research brief (standalone) | Keep Local / Upstream / Docs / Inference separate |

## Ask Only When It Matters

Finish the research first, then ask **one** targeted question only when:

- viable paths differ materially in product behavior, operational risk, or migration cost
- repo evidence conflicts with the request or a locked decision in a way that changes the recommendation
- version or environment uncertainty would change the implementation path

Otherwise make the best evidence-backed recommendation.

## Smell Test

The brief must answer four things: what exists, what is reusable, what the docs say, and which path to take — with the rejected rungs stated.
