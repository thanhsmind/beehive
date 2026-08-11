# bee harness — the whole picture, drawn

> One page of diagrams. Each diagram answers one question: what contains what,
> what each part means, and how a piece of work flows through the machine.
> Prose stays minimal — the [overview](overview.md) carries the doctrine, the
> [register](register.md) carries the state detail, and each
> [stage page](index.md) carries its stage. On any disagreement the source wins.

## 1. What contains what — the component map

Every part of bee sorts into one of three layers (Machine / Craft / Memory),
plus the runtime state the machine writes into a host project. Arrows here mean
"is part of / is vendored into", not data flow.

```mermaid
flowchart TB
    subgraph HOST["Host project (any repo bee is onboarded into)"]
        direction TB
        subgraph BEE[".bee/ — runtime state (the registers)"]
            BIN[".bee/bin/bee — THE vendored binary<br/>every hook and command runs this"]
            subgraph CONTROL["control plane — shared across worktrees"]
                WF["runtime/workflows/&lt;wf-id&gt;/state.json<br/>the workflow record — SOURCE OF TRUTH"]
                LEASES["runtime/leases/ — sharded cell + path leases"]
                MAILBOX["runtime/handoffs/&lt;wf-id&gt;/ — handoff mailbox"]
                QUEUE["runtime/integration/queue/ — merge queue"]
            end
            subgraph DATA["data plane — per checkout"]
                CELLS["cells/*.json — units of work"]
                DEC["decisions.jsonl — append-only WHY"]
                CFG["config.json — the one hand-edited file:<br/>commands.test, gate_bypass, models"]
                PROJ["state.json — read-only projection<br/>phase · gates · feature"]
                LOGS["logs/ — test-results, timings, dispatch"]
            end
            EXP[".bee/expertise/ — vendored craft guides"]
        end
        KNOW["docs/knowledge/ — the knowledge bundle<br/>areas/ · patterns/ · work/ (read FIRST)"]
        HIST["docs/history/&lt;feature&gt;/ — CONTEXT.md, plan.md<br/>(archaeology, read LAST)"]
        WT["feature worktrees — one per code-touching feature<br/>main checkout keeps integration + docs + release"]
    end

    subgraph SOURCE["bee source (this repo) — what onboarding vendors FROM"]
        direction TB
        subgraph MACHINE["Machine layer — flow, state, gates, proof"]
            RS["packages/bee-rs/ — one Rust binary<br/>verbs · hooks · onboard · devtools"]
            HOOKS["hooks catalog — 9 events, all fail-open:<br/>session-init · prompt-context · write-guard<br/>model-guard · state-sync · chain-nudge<br/>session-close · tools-logger · codex-subagent-audit"]
        end
        subgraph CRAFT["Craft layer — how to work well"]
            SKILLS["skills/ — 9 SKILL.md instruction sets<br/>bee-hive routes; the rest are stages + side steps"]
            EXPSRC["expertise/ — 9 craft + 6 domain guides"]
        end
        subgraph MEMORY["Memory layer — why bee is this way"]
            DOCS["docs/ — decisions, history, specs,<br/>knowledge bundle, this handbook"]
        end
        PAYLOAD["packages/bee/ — payload assets:<br/>prompts/, hooks json, AGENTS.block.md"]
    end

    RS -- "cargo build + onboard" --> BIN
    HOOKS -- "rendered per runtime" --> BIN
    EXPSRC -- "vendored" --> EXP
    SKILLS -- "projected to plugin trees" --> HOST
    PAYLOAD -- "vendored" --> BEE
```

Meaning, in one line each:

| Part | Child of | Means |
|---|---|---|
| `bee` binary | Machine | The only door to state. Agents never hand-edit `.bee/*.json`. |
| hooks | Machine | Safety **net**, not authority: an unblocked write is still not an approved write. |
| skills | Craft | The instructions a session loads to run a stage. |
| expertise | Craft | Judgment that holds in any repo (tests, debugging, architecture…). |
| workflow record | control plane | Where a running feature's truth lives; `state.json` is only its projection. |
| leases / claims / holds | control plane | How parallel sessions coordinate — never around them. |
| knowledge bundle | Memory (host) | What the system *is* today. Read before code. |
| `docs/history/` | Memory (host) | How it got that way. Read last. |
| worktrees | host workspace | Isolation per code-touching feature; merge is the only road back. |

## 2. When does the flow run — the lifecycle

The machine's phases, the three gates, and the two deferred stages. Solid
arrows are the default chain; dashed arrows are on-demand or deferred.

```mermaid
stateDiagram-v2
    direction TB
    [*] --> idle
    idle --> exploring : bee state start-feature
    exploring --> planning : Gate 1 — approve CONTEXT.md
    planning --> swarming : Gate 2 — merged shape+execution<br/>(no source edits before this)
    state swarming {
        direction TB
        orchestrate --> execute : dispatch prepare --claim
        execute --> orchestrate : cells finish — green caps, red refuses
    }
    swarming --> closed : bee close — reruns commands.test
    closed --> scribing : deferred — owner's pace
    scribing --> compounding : state scribing-run
    compounding --> compounding_complete : state compounding-run + close commit
    compounding_complete --> idle
    closed --> reviewing : only on explicit user request
    reviewing --> merged : Gate 3 — approve merge
    swarming --> paused : ~65% context — HANDOFF.json
    paused --> swarming : pause is presented, never auto-resumed
```

Key readings:

- **Gate 1** and **Gate 2** are the default chain; **Gate 3 exists only inside a
  review session the user asked for.** A finished feature is truthfully
  *unreviewed* until then.
- Scribing + compounding are one skill (`bee-capturing`) and are **deferred,
  never dropped**: a green close records capture as pending and the reminder
  stands until it runs.
- `gate_bypass` (normal/full/total) can auto-approve Gates 1–2 by level — a
  human sets the level; the agent never approves a gate itself.

## 3. One feature, end to end — who talks to whom

The sequence for a small code-touching feature under `gate_bypass: normal`
(the gates auto-approve; with bypass off the same two points stop and wait
for the human).

```mermaid
sequenceDiagram
    autonumber
    actor U as User
    participant O as Orchestrator session
    participant B as bee CLI (.bee/bin/bee)
    participant W as Worker (bee-build subagent)
    participant S as Store (.bee/ + worktree)

    U->>O: fuzzy request
    O->>B: bee orient
    B-->>O: phase, blockers, next skill
    O->>B: state start-feature / route --set / worktree new
    B->>S: workflow record + feature worktree
    O->>B: decisions log (each answer, the moment it settles)
    O->>B: gate --name context / gate --merge
    Note over O,B: Gates 1–2 — human approval,<br/>or auto under gate_bypass
    O->>B: cells add --stdin
    O->>B: dispatch prepare --claim (from MAIN checkout)
    B->>S: claim cell + reserve files
    B-->>O: rendered worker prompt + tier/model
    O->>W: spawn with that prompt (inside the worktree)
    W->>S: edit reserved files, one commit (cell: id trailer)
    W->>B: cells finish --id
    B->>S: run commands.test — green caps, red refuses with excerpt
    W-->>O: [DONE] / [BLOCKED] / [HANDOFF] / [NOOP]
    O->>B: bee close --feature (reruns tests, retires cells)
    O->>B: worktree merge --id (verify green, worktree removed)
    O-->>U: outcome + capture line
    Note over O: scribing + compounding later,<br/>at the owner's pace
```

## 4. How ceremony scales — lane routing

`bee route` classifies mechanically (risk-flag count + product-file count);
the lane decides how much workflow the work pays for. Memory never scales
down: every lane captures what settles.

```mermaid
flowchart TB
    REQ["request"] --> ROUTE{"bee route<br/>class · lane · flags · files"}
    ROUTE -->|docs only| DOCS["docs lane<br/>no gates: announce → write →<br/>format-check → capture line"]
    ROUTE -->|1 file, no risk flag| TINY["tiny<br/>merged gate inline · one cell<br/>may run inline in-session"]
    ROUTE -->|few files, no risk flag| SMALL["small<br/>merged gate · dispatched worker(s)<br/>scoping logged as a decision"]
    ROUTE -->|more files or 1 risk flag| STD["standard<br/>full chain · plan.md frozen at Gate 2"]
    ROUTE -->|hard-gate territory| HIGH["high-risk<br/>full chain + brief + persona panel<br/>never demotes"]
    ROUTE -->|feasibility unknown| SPIKE["spike<br/>disposable proof → re-route"]
    TINY & SMALL & STD & HIGH --> GATES{"Gates 1–2<br/>(one merged question)"}
    GATES --> WTQ{"code-touching?"}
    WTQ -->|yes| WTREE["feature worktree from the start"]
    WTQ -->|no| MAIN["main checkout"]
    WTREE & MAIN --> SWARM["swarm → execute → close"]
    DOCS --> CAP["capture line or<br/>'nothing settled'"]
    SWARM --> CAP
```

Risk territory (auth, data loss, security, external providers, validation
removal) **parks at any confidence** on the unattended path — risk is a
property of the change, not of the assessor's certainty.

## 5. The memory loop — how knowledge compounds

Why the next session starts smarter: everything that settles is pushed into
the bundle, and the bundle is pushed back into the start of every session.

```mermaid
flowchart LR
    subgraph WORK["during the work"]
        SETTLE["a rule / value settles"] --> DLOG["bee decisions log<br/>same turn, never batched"]
        SETTLE --> STUB["bee capture add — one-line stub"]
        CAPPED["cells cap with traces<br/>(deviations, outcomes, evidence)"]
    end
    subgraph CLOSEOUT["at close + later, owner's pace"]
        PROMOTE["bee close mines traces →<br/>promote-proposals.md (PROPOSAL ONLY)"]
        FLUSH["bee-capturing: review proposal,<br/>merge what survives, flush stubs,<br/>record why the rest was declined"]
        LEARN["dated learnings file +<br/>promoted critical patterns"]
        STAMP["state scribing-run — the receipt"]
    end
    subgraph BUNDLE["docs/knowledge/ — the state layer"]
        AREAS["areas/&lt;area&gt;/ — R#/B# rules,<br/>vocabulary, edge cases, Open Gaps"]
        PATTERNS["patterns/ — dated pitfalls,<br/>critical ones ranked into the preamble"]
    end
    subgraph NEXT["next session"]
        PRE["session preamble — critical-patterns digest,<br/>project map, knowledge-context invitation"]
        ORIENT["bee orient — phase, blockers, next step"]
        KCTX["bee knowledge context --work W —<br/>budget-capped curated manifest.<br/>Anchor arms, in order: work-item ›<br/>history › ledger › backlog row"]
        SEARCH["bee knowledge search --text symptom<br/>mid-flow pull, read-only"]
        SCOUT["shaping scout reads the touched area's<br/>index + Open Gaps BEFORE interviewing"]
    end
    CAPPED --> PROMOTE
    DLOG --> FLUSH
    STUB --> FLUSH
    PROMOTE --> FLUSH
    FLUSH --> AREAS
    FLUSH --> LEARN
    LEARN --> PATTERNS
    FLUSH --> STAMP
    AREAS --> PRE
    PATTERNS --> PRE
    AREAS --> KCTX
    AREAS --> SEARCH
    AREAS --> SCOUT
    PRE --> ORIENT
```

The backlog-row anchor arm (last in the chain) is what lets `knowledge
context` fire **during exploring** — before `CONTEXT.md` exists, the backlog
row is the earliest artifact that names the work.

## 6. The guard layer — what the hooks actually do

Hooks are rendered per runtime from one catalog and all call the same binary.
Every one is **fail-open**: a payload it cannot decide allows the operation
and says so out loud.

```mermaid
flowchart TB
    subgraph EVENTS["runtime events"]
        E1["session starts / clears"]
        E2["user submits a prompt"]
        E3["agent is about to run a tool"]
        E4["session ends"]
    end
    E1 --> H1["session-init — inject the preamble:<br/>phase, gates, route, handoff,<br/>critical patterns, knowledge invitation"]
    E2 --> H2["prompt-context — one-line state echo"]
    E3 --> H3{"write-guard"}
    H3 -->|"edit before Gate 2<br/>secret-shaped path<br/>generated tree<br/>foreign worktree path"| DENY["DENY — names its remedy<br/>(the fix is in the message)"]
    H3 -->|otherwise| ALLOW["allow"]
    E3 --> H4{"model-guard"}
    H4 -->|"dispatch without a tier"| REPAIR["repair or refuse the dispatch"]
    E3 --> H5{"git guard (in write-guard)"}
    H5 -->|"git add with live sibling workers"| PS["refuse — demand path-scoped commit<br/>(shared index protection)"]
    E4 --> H6["session-close — warn on claimed cells,<br/>unreleased reservations, dirty state"]
    DENY -.-> NOTE["a deny is a teaching moment:<br/>follow the named remedy,<br/>never work around the guard"]
```

## Where to go next

- Stage detail: [index.md](index.md) → `stages/<id>.md`
- Every register drawn above: [register.md](register.md)
- Doctrine and vocabulary: [overview.md](overview.md)
