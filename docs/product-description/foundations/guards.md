# The guards

## Summary

The guards are the hooks that stand between the agent's tool calls and the repository: before an edit, a shell command, a read, or a subagent dispatch runs, the matching guard examines it and answers *allow*, *warn*, *repair*, or *deny*. A deny stops the tool call with exit code 2 and a message that always names its remedy; a warning attaches advice without pre-approving anything; a repair (the model guard only) rewrites the dispatch to what configuration says and announces the rewrite. Two postures govern failure: guards **fail open** wherever a broken input would turn a permitted action into a refusal, and **fail closed** wherever the broken input is coordination state — a store that cannot be read must not be treated as empty. The guards are a safety net, never the authority: an unblocked write is not an approved write. This document is the catalog; [gates](gates.md) says why the phases gate, [worktrees](worktrees.md) why containment holds.

## The simple case

The agent, mid-exploring, tries to edit a source file before Gate 2:

> bee gate: phase is "exploring" and gate "execution" is not approved — writing "src/lib.rs" is blocked. Allowed now: .bee/, docs/history/, plans/, AGENTS.md. Get execution approval (bee-hive) before touching source files.

The edit never happens. The agent routes the work through the gate, gets the approval, and the same edit passes silently. Every deny in the family follows this shape: what was attempted, why it is blocked, and the sanctioned way forward. Following the remedy is the whole protocol; working around a deny — retrying variants, writing through a different tool — is the one forbidden move.

## The catalog

What each guard watches, when it fires, and what its deny opens with. Write-capable tools are Edit, Write, MultiEdit, Bash, and patch application; read tools are Read, Glob, Grep.

### Phase and containment (write-capable tools)

- **Gate guard** — gated phases (`exploring`, `planning`) with `execution` unapproved: source writes outside `.bee/`, `docs/history/`, `plans/`, `AGENTS.md` are denied (the message above).
- **Idle intake gate** — at `idle`/`compounding-complete`: writes outside `.bee/`, `docs/`, `plans/`, `AGENTS.md` are denied: `bee intake gate: no bee work is active (phase: …) — … is blocked. FIX: commit or write bookkeeping directly … or route the request through bee-hive first … Last resort, repo-level opt-out: set guards.idle_gate to false in .bee/config.json`. The same gate classifies git subcommands: read-only git always passes, modeled bookkeeping mutations pass, `git push` never passes by exemption, and an unrecognized mutation (`git init` among them) is refused "rather than assumed safe".
- **Outside-the-worktree containment** — any write target that cannot be canonically contained inside the physical worktree is denied (traversal, absolute paths, symlink escapes included). Exactly two exemptions, matched per user: the agent's memory root and its scratchpad.
- **Unknown-phase guard** — a phase value the workflow does not recognize refuses writes entirely: restore a valid phase first.
- **Worktree-first guard** — in the main checkout, a source write for a code-touching active feature is denied when the feature holds (or should hold) a worktree: the deny names the grant to open or `bee worktree new` to run, and the deliberate override (`worktree_first: "off"`).

### Store integrity

- **Direct-edit guard** — CLI-owned store files deny hand edits in every phase, naming the verb ([the store](store.md) carries the table).
- **Docs-history code guard** — a code-extension file written under `docs/history/` (the tech-agnostic knowledge layer) is denied toward the project's own scripts directory.
- **Scratch-shape guard** — ephemeral probe/verdict/digest files landing in a tracked directory are denied toward `.bee/tmp/` (or `.bee/spikes/` for feasibility proofs), swept later by `bee tmp sweep`.
- **Plan-freeze guard** — a feature's `plan.md` denies direct edits once its shape gate is approved; the remedy is a stamped revision (`bee state plan-rev bump`).

### Coordination (write-capable tools)

- **Cross-session hold** — a path held by a live sibling session in the same workspace is a hard block: wait or coordinate.
- **Cross-worktree hold** — hard on exclusive paths (`… is held by checkout "…" … a cross-worktree hold is a hard block.`), advisory otherwise (a warning that `bee worktree merge` will surface any real conflict).
- **Swarm reservation conflict** — during `swarming`, a path reserved by another agent denies with `Reserve the path first or return [BLOCKED] to the orchestrator.`; an intent-kind reservation warns instead of blocking.
- **Write-policy guard** — a second write-capable session in the same checkout is denied by default: `… a second write-capable session defaults to isolation, never a shared write into the same checkout.` — coordinate, wait out the heartbeat, or isolate into a feature worktree.
- **Concurrent-worker git guard** — phase-independent: with more than one live worker in a checkout, tree-sweeping git verbs (`add`, `reset`, `clean`, `checkout`, `restore`, `revert`, `rebase`, `cherry-pick`, `merge`) are refused because the shared index lets one worker sweep another's files into its commit. Inspection is always allowed; the remedy is a path-scoped commit through a private temp index (the deny spells out the exact recipe), with `git add -N` allowed for intent-to-add. When the live-worker count cannot be resolved, the guard assumes "more than one" — a deliberate fail-safe deny.
- **Staging commit guard** — a hand-run `git commit` inside the staging worktree is denied in every phase; staging has exactly two sanctioned writers (`bee staging add`, `bee staging rebuild`).

### Reads

- **Secret guard** — `.env*`, key and certificate files, `id_rsa*`, `credentials*`, `secrets.*`: the *read* is denied — `bee privacy guard: "…" looks like a secret/credential file. Ask the user before reading it.` — and a machine-readable `@@BEE_PRIVACY@@{"file":…,"question":…}@@END@@` block is emitted for the agent to route to the human. Only the human approves a secret read, at every bypass level.
- **Scout guard** — reads inside generated or vendored trees (`node_modules/`, `dist/`, `build/`, `vendor/`, `.git/objects`, coverage and cache dirs): read the source or lockfile instead.
- **Read-size guard** — a Read of a file past the threshold (default 800 lines, `guards.max_read_lines`) is redirected toward a scoped read.

### Command shape and dispatch

- **CLI-shape guard** — Bash only: a `bee` command whose argv does not match the registry schema is denied before the binary runs, with the mismatched field named and the `--help` to read. Same registry as the binary's own refusals, one layer earlier.
- **Model guard** — on Agent/Task dispatch (Codex: `spawn_agent`). Three behaviors: *silent* when it has no opinion (no root, hook disabled, not a dispatch tool); *repair* when the dispatch names a pinned rendered agent wrongly or a `model` param disagreeing with the tier — the input is rewritten to what `models.<runtime>` configures, announced to both agent and human; *deny* for a bare dispatch (`every Agent/Task dispatch needs an explicit role … A bare dispatch would silently inherit the most expensive session model.` — the FIX names `bee dispatch prepare`), a role nothing configures, an ambiguous generic type where the role carries several rendered agents, or a tier that must not ride the Agent tool at all.

## The failure postures

- **Fail closed** — corrupt coordination state: the reservations store, the holds ledger, the workspace record, the lane record, the staging record. Each deny says it is `failing closed … rather than silently treating it as empty` and names the file to inspect and restore.
- **Fail open** — everything whose unreadability would wrongly refuse a permitted act: the session-liveness read, an unresolvable grant, a corrupt grants registry, a non-git root. And the whole hook layer itself: a hook that cannot decide exits 0 with a stderr line, and a missing hook binary prints `bee: hook binary missing (.bee/bin/bee)` and lets the action pass — visible, never silent.
- The one deliberate inversion: the concurrent-worker count, where *unresolvable* means *assume contended* — because the failure there loses another worker's commit, not one's own convenience.

## Modifiers

| Modifier | Effect on the guards |
| --- | --- |
| `--json` | Not applicable — guards speak into the tool-call channel, not the CLI's streams. |
| Gate-bypass level | None. No level silences a guard; bypass changes what the *gate verb* will self-approve, and the phases move accordingly. Secret reads stop at every level. |
| Store phase | Selects the governing write rule: gated phase → gate guard; idle/terminal → intake gate; `swarming` → reservations enforced; unknown → refuse. |
| Where it runs | Containment is per physical worktree; the worktree-first and staging guards exist only in main and staging respectively; holds mirror across checkouts through main's ledger. |
| Who runs it | `BEE_AGENT_NAME` identifies a swarm worker to the reservation guard; the write-policy guard counts write-capable *sessions*, not workers. |

## Cancel and interrupt

The guards are instantaneous deciders — there is no extended phase to interrupt. The meaningful rows:

| Event | Behavior |
| --- | --- |
| The process killed | A guard decision is atomic with the tool call; there is no half-denied state. |
| The store unavailable | The postures above: coordination state fails closed, permission inputs fail open, the missing binary announces itself and passes. |
| A sibling changing the target | That is what the coordination guards exist for; the deny names the holder and the wait-or-coordinate remedy. Never write through it; never wait it out in silence. |
| The channel changing | Codex advisory events never emit a block — on that runtime the same guards can only warn; the write guard's hard denies ride the events Codex does gate. The OpenCode plugin is out of scope. |

## Interactions with other systems

**Gates and approval.** The gate and intake guards are the phases' teeth; the authority is the recorded approval, not the hook.

**The store and history.** Guards read the store to decide and write nothing but crash logs; every deny leaves the repository exactly as it was.

**Worktrees and containment.** Containment, worktree-first, the holds ledger, staging's two-writer rule — the geography is [worktrees](worktrees.md).

**Claims, holds, and reservations.** Enforced here, owned by [reservations](../coordination/reservations.md) and the cell machinery.

**Sibling sessions.** The write-policy and hold guards are the only thing standing between two sessions and a shared-checkout mess; a deny naming a sibling is triage data, not a user question.

**What the human sees.** A red or refusal line is never silenced. The privacy marker is routed to the human verbatim; model-guard repairs announce themselves to both parties.

**Configuration.** Per-hook toggles (`hooks.<name>: false`), `guards.idle_gate`, `guards.auto_isolate`, `guards.max_read_lines`, `worktree_first` — each named in its guard's own deny.

**Output modes and exit codes.** Deny = exit 2 with stderr text; warn = exit 0 with attached context; repair = exit 0 with rewritten input; undecidable = exit 0 with a stderr line.

## Edge cases

- A deny's remedy can itself be denied one layer deeper (the temp-index recipe meets the intake gate's unmodeled `git read-tree` at idle). The remedy chain ends at a named fallback — commit the exempt paths directly, or route through the workflow — but the collision is real; see "Open questions".
- The write guard watches AskUserQuestion among its matched tools — the guard family can inspect a question's content, not only file writes.
- Warnings deliberately carry no permission decision, so a warned action still faces the runtime's own approval flow untouched.
- A guard deny inside a herded, unattended run has no human to route to; the letter to the human ([mailbox](../memory/mailbox.md)) is the escalation path there.

## Open questions and verification

- **Remedy collision, filed for triage:** at idle with multiple live workers, the concurrent-worker guard's own prescribed recipe (`git read-tree` into a temp index) is refused by the intake gate's unmodeled-subcommand rule. Both guards behave as designed; the composition strands the agent one step, until the path-scoped `git commit -- <paths>` form (which both allow) is tried. Confirmed live in this repo; the recipe text could name the composed case.
- The full secret-pattern list and the injection-pattern list were read as families, not enumerated; a verification pass should probe the boundary cases (a `.env.example`, a `credentials.rs`).
- The read-size guard's interaction with the harness's own Read limits was not examined.
- Deny texts above are quoted from source and, for the intake gate, the containment deny, the CLI-shape guard, the concurrent-worker git guard, and the direct-edit guard, confirmed live in this repository during this description's own work.

Verified against beehive commit `6b0ae488`.
