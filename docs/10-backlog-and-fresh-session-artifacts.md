# 10 — Fresh-Session Artifacts and the PBI Layer: What bee Adopts Next

Two analyses merged (2026-07-08 session), same verdict format as [08](08-harness-adoption.md)/[09](09-harness-course-adoption.md): **already covered / adopt now / skip**. Sources: learn-harness-engineering lecture 03 (Fresh Session Test), lecture 06 (initialization as its own phase), and a PBI/task "Project Policy" document the user supplied (the docs/delivery backlog + task-file pattern).

Two findings, one per part:

- **Part A.** bee *detects* Fresh Session Test holes (grooming probe, docs/09 item 4) but nothing *generates* the missing artifacts. A legacy repo onboarded to bee still fails Q1–Q3 from repo contents alone. Detection without generation leaves backlog items hanging.
- **Part B.** bee's task layer (cells) is a strict superset of the policy document's task machinery — but bee has **no product-backlog layer above cells**. A request the user defers dies in the chat; nothing in the repo answers "what do we plan to build next, in what order". Agent-native session todo lists live outside the repo and die with the session; the durable list must be a repo artifact (lecture 03: information not in the repo does not exist).

## Part A — Fresh-session artifact generation

Fresh Session Test coverage today:

| Question | Owner artifact | State |
|---|---|---|
| What is this system? | `docs/specs/system-overview.md` | hole — exists only after scribing has run once |
| How is it organized? | `docs/specs/reading-map.md` | hole — same |
| How do I run it? | `.bee/config.json` commands | half — mechanism exists (09 item 1), capture is a skippable question |
| How do I verify it? | `commands.test` + baseline gate | half — same |
| Where are we now? | `bee_status` + state + cells | covered (stronger than the course) |

### A1. Scribe bootstrap mode (`bee-capturing`) — the generator

When onboarding succeeds and `docs/specs/` is empty (or missing system-overview/reading-map), offer a bounded skeleton pass: generate `system-overview.md` + `reading-map.md` containing **only what code mechanically proves** — top-level structure, entry points, README first paragraph — every *meaning* an Open Gap, `coverage: partial`. Never runs silently; the user approves the run. Never-invent holds: bootstrap writes provable facts and questions, nothing else. This is harvest-lite with a hard budget — inventory only, no interviews (harvest proper fills the gaps later).

### A2. Command auto-detection, user-confirmed

A detector (in the onboarding path and available to exploring) scans conventional sources — `package.json` scripts, `Makefile` targets, `pyproject.toml`, `composer.json`, `*.csproj`, `go.mod`+Makefile — and **proposes** `setup/start/test/verify` candidates. The agent presents one pre-filled confirmation question; confirmed values are written to `.bee/config.json` commands. A pre-filled question beats the current open skippable one; never-invent is preserved because the machine nominates and the human confirms.

### A3. AGENTS.md outside-markers audit — propose, never write

Onboarding gains a check: does the entry file *outside* the BEE markers answer Q1 in a few lines (one "what this project is" line + pointers)? If not, the plan output gains a `changes_needed` item proposing a minimal header. The consent mechanism already exists ("never replace content outside markers without explicit consent") — it has just never had anything to propose for that region.

### A4. Preamble project-map lines

`inject.mjs` preamble gains 2–4 lines: when specs exist — pointers to `system-overview.md` / `reading-map.md` + specced-area count; when missing — one warning line ("Q1/Q2 unanswerable from repo — scribing bootstrap available"). Near-zero token cost; every session sees the map first.

### A5. Grooming probe items point at the fix

The Fresh Session Test probe (grooming reference) files backlog items today; each item now names the one-command fix (A1 bootstrap or A2 detect) instead of an open-ended task.

## Part B — The PBI layer

### Policy document vs bee — the map

| Policy | bee | Verdict |
|---|---|---|
| Task with requirements/verification/files/status (§4.1) | cell: must_haves + verify + evidence + files + trace; cap mechanically refuses without proof | covered, stronger (refusal beats prose) |
| No code without an agreed task (§2.1, §2.5) | intake gate, hook-enforced | covered, stronger |
| User authority (§2.1.4) | three human gates, never self-approved | covered |
| Commit carries task id (§4.9) | one commit per cell, cell id in message | covered |
| Status history log (§4.8) | decisions.jsonl + cell trace + git | covered, event-sourced |
| Index ↔ task-file status sync (§4.4) | one file per cell — nothing to sync | **skip** — the sync rule cures a self-inflicted disease (two locations for one fact); adopting dual-location is a regression |
| One InProgress per PBI (§4.7) | reservations + orchestrator-assigned cells | **skip** — re-confirms 09 (WIP=1 deletes swarming's reason to exist) |
| PBI mini-PRD at "Agreed" (§3.6) | CONTEXT.md, born at exploring with D-IDs | **skip** — creating it before exploring writes the spec before asking the questions |
| Six-state PBI machine + transition ceremonies (§3.4) | — | **skip the shape, adopt the substance** (below): three statuses, prose rules, grooming audit — no hook enforcement; PBI transitions are human priority calls, low risk; mechanical enforcement stays where evidence exists (cells) |
| **Product backlog in-repo, prioritized (§3.2–3.3)** | nothing — `.bee/backlog.jsonl` is friction/bee-improvement only | **adopt — the one real gap** |

### B1. `docs/backlog.md` — the product backlog artifact

One human-first markdown table, owned by `bee-capturing` ("Scribe") with merge rules (the `docs/specs/` pattern applied forward-looking): `ID | Story | CoS (one line) | Status | Feature`. Status enum is three values — `proposed / in-flight / done` — not six. Ordered by priority, highest first. One file forever, updated in place, never forked. Specs describe what IS; backlog describes what's NEXT.

### B2. Proactive capture of deferred requests

Extension of decision 0007: when the user defers work ("để sau", "phase 2", "sẽ làm", a deferred idea leaving exploring), the agent adds a PBI row **in the same turn, unprompted** — announce-then-do, exactly like settlement capture. Exploring's existing "Deferred Ideas" section feeds B1 instead of dying in a history file.

### B3. Chain wiring — no new gate

- exploring opens a feature that matches a PBI row → row flips `in-flight` + feature slug (create the row if the request never passed through the backlog).
- feature close (scribing sync / compounding) → row flips `done` + link to `docs/history/<feature>/`.
- grooming audits drift: an `in-flight` row with no active feature, a `done` feature with no row, duplicate rows for one story.
- `bee_status` counts rows; the preamble gains one line: `PBI: N done / N in-flight / N proposed`. Q5 answered at product level from the first second of a session.

### B4. Cells gain an optional `pbi` field

Feature → PBI traceability, one optional field on the cell schema. No ceremony.

### B5. Direction of truth

Session todo lists (TaskCreate and equivalents) are ephemeral projections of cells/PBIs — never the reverse. The repo artifact is the source.

## Sequencing

A2 first (cheapest, cuts daily friction — the skippable question becomes a pre-filled confirmation). Then A1 (the actual generator). A3/A4 are small onboarding/inject slices. B1–B5 is one `small` slice (template + scribing paragraph + hive/grooming wiring + status counter). A5 and B3's grooming lines are reference-doc paragraphs riding whichever slice lands last. All through bee's own chain, dogfood posture as 08/09.
