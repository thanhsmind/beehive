# Work on another project goes to its leader — Context

**Feature slug:** cross-project-guard
**Date:** 2026-11-11
**Shaping session:** complete (owner correction, decision cc1d4957)
**Scope:** Standard (one guard rule, one doctrine rule, their tests and docs)
**Domain types:** RUN

## Feature Boundary

A bee session in project A got a request about code in project B. It read
B's code itself instead of handing the work to B's leader. The owner wants
a mechanism, not a memory note. After this feature, the write-guard refuses
every read, search or shell command that touches another project's tree,
and the refusal names the remedy: hand the work to that project's leader
through waggledance. The leader doctrine carries the same rule.

## Locked Decisions

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | A path belongs to another project when it is outside this project's main root and its worktrees, and its nearest enclosing root (a directory holding `.git` or `.bee/onboarding.json`) is a different repo. A directory whose direct child is such a root also counts. A path in no repo (`/tmp`, `~/.config`) is not a project and stays allowed. (cc1d4957) | The worktrees of this repo share its git common dir and are this project. |
| D2 | The write-guard refuses Read, Glob and Grep whose path is in another project, and Bash whose working directory or any absolute or `~` path token is in another project. It runs for leaders and workers alike, in every phase, idle included. Exit 2. (cc1d4957) | Today an outside-root Read/Glob/Grep is never checked (write_guard/main.rs:249-251). |
| D3 | The refusal names the project root and the remedy: do not read or work it here; call `waggledance_ask_state` for that project, then `waggledance_dispatch` with `leader: true`. If the user asked to research that repo from here, the user adds the path to `guards.cross_project_allow`. (cc1d4957) | |
| D4 | Allowed even when D1 matches: paths under `guards.cross_project_allow` in `.bee/config.json`, paths under a `permissions.additionalDirectories` entry of the user, project or project-local Claude settings, and harness-owned surfaces the guard already allowlists. `guards.cross_project_allow` is a governed key: `bee config set` and agent edits are refused, the user edits it. (cc1d4957) | An agent that can widen its own allowlist has no mechanism. |
| D5 | The leader doctrine gains rule `agents-other-project-to-its-leader`: work for another project goes to that project's leader through waggledance, after `waggledance_ask_state`; the session never reads or explores that project's code. (cc1d4957) | |

### Agent's Discretion

- Where the check lives (a new function in write_guard, called from main.rs before the read and write branches) and how roots are cached per call.
- The exact refusal wording, inside D3's content.

### Known costs the owner accepted

- A user who wants a cross-repo research read must add the path to `guards.cross_project_allow` first.
- `ls` of a folder of repos is refused; `waggledance_projects` lists projects instead.

## Read set

### Knowledge

| Concept | Grade at read | Why it matters |
|---------|---------------|----------------|
| docs/knowledge/areas/hook-runtime/write-guard-request-shapes.md | unread | The Bash target shapes the guard parses. |
| docs/knowledge/areas/doctrine-layer/router-triage-and-the-agents-md-duplication-boundary.md | read | The rule index the new rule joins. |

### Code

- packages/bee-rs/crates/bee/src/hooks/write_guard/main.rs (read branch 249-274, Bash branch 322-470)
- packages/bee-rs/crates/bee/src/hooks/write_guard/hook_local.rs (lexical_rel_path 317-334, harness_allowlisted_target 242)
- packages/bee-rs/crates/bee/src/hooks/write_guard/guards.rs (check_read 243, tokenize_deep 413)
- packages/bee-rs/crates/bee/src/verbs/config.rs (governed_refusal 103)
- packages/bee/LEADER.md (rule markers)
