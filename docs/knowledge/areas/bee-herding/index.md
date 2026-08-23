<!--
GENERATED FILE — do not hand-edit.
Rendered by `bee knowledge index` from concept frontmatter inside docs/knowledge/ (okf-foundation D21).
Regenerate: `bee knowledge index`. Check freshness: `bee knowledge index --check`.
Deterministic: byte-identical for the same bundle contents — path-sorted entries, LF endings,
never a generation timestamp or any other wall-clock value.
-->

# areas/bee-herding/

## Concepts

- [Bee Herding — which agent a pane runs as, and how its command is built](agent-resolution-and-spawn-commands.md) — The config tier route that sends a whole purpose through a pane, the named-agent registry, the four-step precedence a bare run obeys, per-agent pane environment, and why bee keeps no list of agent kinds.
- [Bee Herding — handing a foreign agent its brief, and knowing it arrived](handing-a-foreign-agent-its-brief.md) — The mailbox channel a bee-ignorant worker is briefed over, the standalone-executor contract that keeps it bee-ignorant, and the delivery receipt rule: the worker's own ack file is the only evidence the brief was ever received — herdr lifecycle state is a failure detector, never the receipt.
- [Bee Herding — the three-role cockpit, its safety boundaries, and adoption](overview.md) — A cockpit that runs several Claude Code sessions in parallel worktrees, over whichever pane transport one config key names (herdr or tmux): a dispatch loop that starts work behind an owner interlock, a merge gesture the owner runs by hand, and the safety boundaries that make unattended dispatch acceptable while keeping every landing in main a human act.
- [Bee Herding — the run verb, its signal ladder, and how a worker's wait ends](the-run-verb-and-worker-outcomes.md) — bee herding run as an entry point: the ladder of signals its native poll decides on, the typed outcomes a wait can end in — done, died, paused by a usage limit, timed out — what each does to the pane, and the hang case that is still unsolved.
- [Bee Herding — waves over running workers, and counting the slots they occupy](waves-and-occupancy.md) — A wave as a fan-out over workers that already exist, the append-only ledger written at the moment of the spawn, occupancy as a liveness question rather than a pane count, and why an unverifiable count refuses instead of guessing.
