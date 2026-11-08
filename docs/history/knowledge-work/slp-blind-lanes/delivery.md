---
type: bee.delivery
title: slp-blind-lanes — delivery
description: "Delivery record proposed by bee knowledge promote for work item slp-blind-lanes: 7 capped cell(s), 14 recorded deviation(s)."
timestamp: 2026-08-28
bee:
  id: slp-blind-lanes-delivery
  lifecycle: active
  areas: [advisor-protocol, discovery-wayfinding]
  required_context: [docs/history/slp-blind-lanes/CONTEXT.md, docs/history/slp-blind-lanes/plan.md]
  sources: [docs/history/slp-blind-lanes/CONTEXT.md, docs/history/slp-blind-lanes/plan.md, .bee/cells/archive/slp-blind-lanes/bln-1.json, .bee/cells/archive/slp-blind-lanes/bln-2.json, .bee/cells/archive/slp-blind-lanes/bln-3.json, .bee/cells/archive/slp-blind-lanes/bln-4.json, .bee/cells/archive/slp-blind-lanes/bln-5.json, .bee/cells/archive/slp-blind-lanes/bln-6.json, .bee/cells/archive/slp-blind-lanes/bln-7.json]
---

# slp-blind-lanes — Delivery

## What shipped

- **bln-1** — dispatch prepare --kind advisor --brief-file carries a trimmed LaneBrief into the advisor prompt body and stamps brief_sha256 on the dispatch record and envelope; every other kind, an over-cap brief, an unreadable path and non-UTF-8 bytes each earn a typed refusal (8 file(s) changed)
- **bln-2** — The dispatch door refuses a leaning LaneBrief on two arms — the frozen 17 verdict stems and the four-section shape — through one shared guard the brief bytes alone reach (4 file(s) changed)
- **bln-3** — bee blind check serves the dossier section contract, refuses a malformed one by section or field name, and re-runs the dispatch door's own leaning guard over the recorded brief (7 file(s) changed)
- **bln-4** — The three evidence checks stand green after three fix rounds: citations resolve per lane under two floors and a whole-sentence rule, digests are checked against the dispatch log, and a diet breach refuses while naming itself self-reported (1 file(s) changed)
- **bln-5** — The abbreviation and ellipsis forms refuse, the framing case is a carried limit pinned green, and no line in the module claims the citation check proves faithfulness (1 file(s) changed)
- **bln-6** — Bracketed and quoted abbreviations and list-marker dots no longer end a sentence on either side, and the boundary comment states it raises the cost of a strip rather than preventing one (1 file(s) changed)
- **bln-7** — quote_resolves advances its retry cursor by a character, so a multibyte citation gets a typed refusal instead of a panic (1 file(s) changed)

## Verify

Each cell below was capped only against a recorded passing verify result — bee refuses a cap without one.

- **bln-1** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml devtools::prompts && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch --test registry_contracts && .bee/bin/bee onboard --repo-root . --json && .bee/bin/bee dev release-manifest --check`
- **bln-2** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::drivers && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml brief_lint`
- **bln-3** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml catalog && PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml --test registry_dispatch --test registry_contracts`
- **bln-4** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind`
- **bln-5** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind`
- **bln-6** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind`
- **bln-7** — `PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH" cargo test --release --manifest-path packages/bee-rs/Cargo.toml verbs::blind`

## Deviations

- **bln-1** — Added a brief_not_utf8 refusal arm the cell did not name, instead of the sibling read_file_text lossy decode — a lossy decode silently rewrites the very bytes brief_sha256 exists to pin, so every lane would agree on corrupted text — found a better route
- **bln-1** — Threaded the brief through a new prepare_dispatch_with_brief arity adapter instead of widening prepare_dispatch_with_role, so no existing call site changed — the file already documents that adapter shape for --role — found a better route
- **bln-1** — Could not replace the worktree .bee/bin/bee: it is a symlink into the main checkout and the write guard correctly refuses to write through it, so the vendored binary in this checkout is still one build behind — hit an unforeseen obstacle
- **bln-1** — Could not run dispatch prepare live: the command refuses inside a granted feature worktree, so the CLI end of the flag is proven by a source-plus-registry contract test (brief_file_is_both_accepted_by_the_handler_and_declared_in_the_registry) instead of an invocation — hit an unforeseen obstacle
- **bln-1** — bee dev regen also rewrote .bee/onboarding.json, a file the cell did not list; reserved it under this cell before committing rather than leaving the write unowned — something else had to be fixed first
- **bln-2** — Ran the seven new probes by exact name instead of the cell verify second half — its "brief_lint" filter matches zero tests, the probes living in verbs::drivers::tests, and a zero-test filter proves nothing — the plan was wrong about a fact
- **bln-2** — Dropped a "does not certify neutrality" disclaimer from the refusal text — the probe forbids that word in any form, and a disclaimer beside a refusal trains the reader to expect the claim it denies — found a better route
- **bln-3** — Shipped docs/history/slp-blind-lanes/blind/example-run.md, a path the cell did not declare, after reserving it — tests/registry_dispatch.rs executes every registry entry's first example against the built binary, so the declared example needed a real target, and an inline test now pins that the shipped shape is the shape the parser accepts — hit an unforeseen obstacle
- **bln-3** — The dossier's ## Question records the WHOLE LaneBrief verbatim, not only the brief's Question section as plan.md deferred-answer 3 words it — lint_brief's shape arm requires the brief's four level-2 sections, so a Question-only recording would refuse every dossier, and the digest check in bln-4 needs those bytes — the plan was wrong about a fact
- **bln-3** — The brief and every proposal ride inside fenced blocks and the heading scan skips fenced lines, which the plan's section list did not state — a proposal is arbitrary prose that may quote a heading, and outside a fence a lane's own text would move the dossier's section boundaries — the plan was wrong about a fact
- **bln-4** — Capped by the orchestrator rather than its builder — a failed judge reopened it, bln-5, bln-6 and bln-7 closed the defect, and the round-4 judge verified every truth at HEAD; the code is unchanged since c47fd241 — something else had to be fixed first
- **bln-5** — Capped by the orchestrator rather than its builder — the round-2 judge reopened it, bln-6 and bln-7 closed the residue, and the round-4 judge verified its truths at HEAD — something else had to be fixed first
- **bln-6** — Capped by the orchestrator rather than its builder — the round-3 judge reopened it over a panic in the same function, bln-7 closed that, and the round-4 judge verified its truths at HEAD — something else had to be fixed first
- **bln-7** — followed the plan

## Provenance

Proposed by `bee knowledge promote --work slp-blind-lanes` from 7 capped cell trace(s) in `.bee/cells/` and the anchor `docs/history/slp-blind-lanes/CONTEXT.md`, `docs/history/slp-blind-lanes/plan.md`. Every line above is copied from a trace or from the work item; nothing here is curated truth until a human or agent accepts it.
