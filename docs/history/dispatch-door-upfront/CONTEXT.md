# dispatch-door-upfront — context

## Problem
An agent that wants to fan out a read (gather) learns the right dispatch door only by being refused: AGENTS.md names rendered agents (`bee-gather`, `bee-extract`) and manual spellings (marker, `model` param), but in a repo whose generation/extraction slots are `kind: herding` those agents do not exist and a marker on `Explore` cannot resolve. The session preamble says nothing about the door. Result (observed 2026-08-22): three refused `Explore` dispatches before the agent gave up and read inline.

## Locked decisions
- D1: `bee dispatch prepare --runtime <rt> --kind gather|reviewer|advisor|cell --json` is THE door for every dispatch, stated up front in AGENTS.md. Agent names, markers and `model` params are what `prepare` returns, never a hand-pick.
- D2: the session preamble (and the compaction re-injection) carries one "Dispatch door" block: the prepare command and the resolved tier slots for the claude runtime (`generation`, `extraction`, `review`, `advisor` → model name / herding / cli / session default), so the agent sees before the first dispatch whether an Agent-tool subagent is even a legal transport.

## Out of scope
Changing the guard's refusal text or `dispatch prepare` kinds.
