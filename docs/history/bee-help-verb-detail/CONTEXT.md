# bee help — per-verb flag detail and the read-once rule — Context

**Feature slug:** bee-help-verb-detail
**Date:** 2026-08-21
**Shaping session:** complete
**Scope:** Quick
**Domain types:** SEE

## Feature Boundary

`bee <verb> --help` becomes the one detailed reference for that verb — every
flag with its meaning and required marker — and the workflow docs tell agents
to read it once before a verb's first use instead of guessing. Ends at help
rendering plus two doc sentences; no new dump-all surface.

## Locked Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| D1 | Single-verb help (exactly one rendered entry) prints one line per flag: `--name`, `*` when required, the type, then the registry parameter description. Multi-entry renders (`--help --all`, group listings) keep today's compact flags line. The `json` flag stays omitted under the existing header note. | Descriptions already exist in registry `parameters.properties` but are never printed; detail only where asked keeps tokens bounded. |
| D2 | AGENTS.md and the session-preamble Command-surface line carry the read-once rule: before the FIRST use of a verb in a session, read `bee <verb> --help` — never guess flags; one read per verb, never re-read. | The reminder lives where the agent already looks, costing one sentence. |
| D3 | No new dump-everything surface. | Token discipline is the point. |

## Existing Code Context

### Integration Points

- `packages/bee-rs/crates/bee/src/verbs/help.rs:521` (`flags_line`) and `:563` (`render_help_text`) — the renderer; single-entry detail lands here.
- `packages/bee-rs/crates/bee/src/hooks/session_preamble/budget.rs:160` — the Command-surface preamble sentence.
- `AGENTS.md` ("Spend full text on `bee <command> --help`") — the workflow-doc sentence.

## Outstanding Questions

None.
