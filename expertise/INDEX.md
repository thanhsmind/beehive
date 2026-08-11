# Expertise index

Two questions route to two halves. *How should this work be done?* — the
craft guides. *What am I building right now?* — the domain guides.

Pick the one guide the task in front of you needs; load it on demand.
Never load all of them — route, then read exactly one. A task that spans
both halves takes one from each, not the shelf.

## Craft — how the work is done

| File | Covers | Load when |
|---|---|---|
| [thinking.md](thinking.md) | Choosing a reasoning approach, challenging assumptions, root-causing, stress-testing an argument | Deciding how to reason about a problem; an assumption, argument, or conclusion needs pressure before you act on it |
| [planning.md](planning.md) | Shaping work: size, risk, units of work, ordering, slices, scope integrity, plan contracts | Turning a request into a plan; sequencing or splitting work; scope will not fit; a plan is about to be presented or amended |
| [architecture.md](architecture.md) | Structural decisions, boundaries, dependencies, spotting anti-patterns | Deciding where responsibility lives; drawing or crossing a boundary; a dependency or structure smells wrong |
| [decisions.md](decisions.md) | Which decisions to record, record anatomy, locked vs open, superseding, citing decisions | A choice just settled; a locked decision is contested, inconvenient, or changing; a change exists to honor a decision |
| [tests.md](tests.md) | Coverage audits, behavior vs structure, test levels and doubles, determinism, choosing cases and properties, red-before-green, suite speed, running the suite, plus indexed patterns under `tests/patterns/` | Writing or judging tests; a test flakes; fixing a reported bug; a bug shipped despite a green suite; the cases multiply; the suite is too slow to run; matching a second implementation to an existing one |
| [review.md](review.md) | Findings, severity, adversarial reading, verification, evidence standards, scope, asking for review, answering a finding, self-review | Reviewing a change; filing, disputing, or answering a finding; handing work to a reviewer; no fresh reviewer is available |
| [documentation.md](documentation.md) | Rebuild-grade specs: what vs how, currency, precision, lookup structure, honest gaps | Writing or updating a spec; behavior changed under a doc; a doc is stale, vague, or silent on an edge |
| [knowledge.md](knowledge.md) | The project knowledge base as a system: craft vs project layers, the orientation file, harvesting from finished work, recorded trust, signals over scores, routing indexes, the always-loaded budget, dated freshness, migration rot, retirement, plus indexed patterns under `knowledge/patterns/` | Standing up or growing a project's own knowledge layer; a piece of work just finished and something was learned; judging how far to trust an agent-written entry; the base is duplicating, piling up, or describing a system that no longer exists |
| [debugging.md](debugging.md) | Repro-first, error reading, hypotheses, instrumentation, bisection, environment, order-dependent failures, failures you cannot attach to, fix closure | A bug is in front of you; a fix "works" but is unexplained; it passes alone and fails in the suite; it only fails somewhere you cannot reach; a symptom looks familiar or impossible |
| [merges.md](merges.md) | Seeing the whole conflict state, recovering intent from primary sources, resolving hunks by preserving or picking-and-recording, never inventing behavior, checks as part of the resolution, finishing versus aborting, the rebase variant | A merge or rebase lands in conflict; resolving a worktree-merge conflict; a rebase keeps colliding commit after commit |

## Domain — what is being built

| File | Covers | Load when |
|---|---|---|
| [data.md](data.md) | Schema as contract, modeling facts, constraints, keys, indexes, the N+1 shape, transactions and concurrency anomalies, expand/migrate/contract, backfills, deletion, time, query plans, restores | Designing or changing anything stored; a query is slow; a migration or backfill is coming; deciding what deletion means; judging an inherited schema |
| [apis.md](apis.md) | Contracts across an ownership boundary: caller-first design, error contracts, idempotency, timeouts and retries, compatibility, versioning, pagination, partial failure, long operations, events, deprecation | Designing or changing an endpoint, RPC, message, webhook, or public function; calling something you do not own; a change might break callers |
| [security.md](security.md) | Trust boundaries, authentication vs authorization, object-level checks, input validation, injection and encoding, password storage, crypto primitives, sessions and tokens, secrets, least privilege, supply chain, logging, threat modeling, security review, live vulnerabilities | Anything crossing a trust boundary; writing or reviewing a permission check; building a query, command, path, or markup from input; handling credentials; a vulnerability is found |
| [operations.md](operations.md) | Reversibility, deploy vs release, rollout strategies, config and environment parity, health checks, observability, alerting, incident response, postmortems, scheduled work, graceful degradation, runbooks | Shipping a change; choosing a rollout; deciding what to instrument or alert on; something is broken right now; a job runs on a schedule; writing down a procedure |
| [performance.md](performance.md) | Measure-first, budgets, bottlenecks, the cost ladder, growth shapes, caching contracts, tail latency, honest benchmarking, parallelism, memory, perceived speed, pinning a win | Something is slow, or must not become slow; a cache is proposed; the average looks fine and users complain; benchmarks disagree between runs |
| [frontend.md](frontend.md) | Loading/empty/error/loaded states, state ownership, address as state, platform controls, accessibility, forms, optimistic updates, error and empty screens, perceived speed, input-driven layout, list rendering cost, interface wording, client trust, testing surfaces | Building or changing anything a person looks at or types into; a surface feels slow or broken under failure; deciding what the client may be trusted with |
