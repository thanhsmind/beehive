# install-orchestrator-link — tiny brief

**Asked (2026-08-30, owner):** the Install section should carry ONE more
link: installing/updating the orchestrator to its own standard. The
normal install script stays what it is — the ordinary per-repo (leader)
setup.

**Found:** README.md § Install documents only the leader path
(scripts/install.sh / install.ps1). The orchestrator (waggledance) has
its own published one-liner installer
(`https://raw.githubusercontent.com/thanhsmind/waggledance/main/install.sh`),
idempotent on re-run. No install doc in beehive mentions it. Context:
decision b59e50c8 — the cockpit-supervisor seat lives on the
waggledance side.

**Will do:** add a short `### Orchestrator (waggledance)` subsection to
README.md § Install (after "Verify / update"): one sentence naming what
the orchestrator is, the one-line installer, and that re-running the
same line updates it in place. No script changes, no INSTALL.md change
(that file documents bee's own installer options only).
