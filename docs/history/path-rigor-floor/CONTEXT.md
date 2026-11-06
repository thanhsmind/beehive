# Path rigor floor — Context

**Feature slug:** path-rigor-floor
**Date:** 2026-11-06
**Scope:** Standard (route floor check, config rows, gate bypass refusal)
**Domain types:** CALL, RUN

## What was asked

The user asked on 2026-11-06 to build part B of the forgent adoption work
("Làm A, xong rồi tới B"). This feature is item 3 of
`docs/history/research/forgent-xia.md`: a path-to-rigor table.

## Why

Lane and risk are agent judgment today
(`skills/bee-planning/references/planning-reference.md`, the route table).
An agent can route a database migration as `small`. Forgent maps write
paths to a minimum rigor that agents can raise but never lower, and asks
the owner first for migrations and SQL. A data floor turns doctrine into a
check.

## Locked Decisions

The decisions are agent named guesses under `gate_bypass=full`; the user was
not asked. Cite them; never reinterpret them.

| ID | Decision | Store id |
|---|---|---|
| D1 | A path-to-rigor table maps repo path globs to a minimum lane and hard-gate flags. bee ships built-in rows; a repo adds rows in .bee/config.json; a repo row may raise a built-in floor, never lower it. | 4bf3a432 |
| D2 | The floor is checked where files become known: bee cells add and cell update refuse a cell whose files hit a row whose minimum lane or flags exceed the feature route, and name the remedy (raise the route). Agents may always raise a lane; the floor only refuses lowering. | f613e3c2 |
| D3 | Rows marked owner-first (database migrations, SQL schema, release and publish workflows) make the merged shape+execution gate a human gate: gate_bypass at any level never auto-approves a feature whose cells touch them. | f9ba8d88 |
| D4 | Out of scope: the request size rule (at most 3 paths and lane standard means one run). It moves to the backlog. | 8258ae2d |

## Rejected

- Config that can lower a floor: it lets the agent talk itself out of
  rigor (D1).

## Out of scope

- The request size rule (D4) — a backlog item.
