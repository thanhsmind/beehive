# Gray-Area Probes

Use these to generate 2–4 product decisions during shaping's Explore move. Pick only
questions that are unstated, user-facing, and likely to change planning output.
Frame each probe as an outcome ("what should the user see when…"), not as a
technical property. Cite existing repo patterns from the quick scout where they
constrain the answer.

Stop generating when 2–4 probes pass the materiality test
(`shaping-reference.md`, "Materiality test"). The domain lists below are
menus, never checklists — don't walk one top to bottom, and don't ask a
probe whose answer the scout can already cite.

## Domain Probes

### SEE (user-visible surface)

- Layout: list, table, card grid, timeline, canvas, or other?
- Density: compact power-user view or spacious casual view?
- Responsive behavior: same layout, stacked, hidden columns, or separate mobile flow?
- States: what does the user see when empty, loading, or on error?
- Interaction: read-only, inline edit, explicit save, drag, destructive confirmation?
- Lists: pagination, load more, infinite scroll, sorting, filtering, truncation?

### CALL (API, CLI, webhook, SDK, service interface)

- Input shape: params, request body, flags, payload, or SDK object?
- Success shape: full resource, ID, status, stream, or async job handle?
- Caller and auth: internal, user, anonymous, API key, JWT, OAuth, session?
- Errors: what does a failed call tell the caller — status/code format, structured details, retry/rate-limit signal?
- Behavior: idempotent, retryable, dry-run, side effects, timeout expectations?

### RUN (job, script, service, pipeline)

- Trigger: cron, event, message, command, webhook, or manual run?
- Concurrency: single instance, bounded pool, parallel allowed, lock needed?
- Output: stdout, log, database, notification, report, artifact?
- Progress: none, counts, percentage, verbose mode?
- Failure: abort, continue with logged failures, retry/backoff, notify, archive?
- Modes: dry-run, force, resource limits?

### READ (docs, emails, reports, notifications)

- Structure: narrative, step-by-step, reference table, anchors, multi-page?
- Audience: beginner, intermediate, expert, operator, customer?
- Tone and depth: conversational, formal, terse reference, explain-from-scratch?
- Required sections, examples, callouts, warnings, versioning, maintenance cadence?

### ORGANIZE (data model, file layout, taxonomy, config)

- Grouping: type, domain, date, owner, status, priority?
- Nesting depth and membership rules?
- Naming: casing, prefixes, suffixes, stability, conflict resolution?
- Exceptions: ungrouped items, orphan handling, multi-group items?
- Evolution: migration needed, stable from day one, approval process for new groups?

## Cross-Cutting

- What is explicitly out of scope?
- Which adjacent feature is touched but not owned?
- What existing pattern should this follow or intentionally diverge from? (cite the scouted path)
- Who or what consumes the output?
- Does the output need to be human-readable, machine-readable, or both?
- What breaks for users if this ships wrong — and how would they notice?

## Anti-Probes (never ask these here)

These are implementation choices; they belong to `bee-planning`:

- library, framework, or storage engine selection
- performance targets, caching strategy, index design
- module boundaries, class shapes, function signatures
- test framework or CI wiring
- anything whose answer the user would reasonably delegate with "you pick"
