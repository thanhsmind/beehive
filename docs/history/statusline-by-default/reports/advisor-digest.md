# statusline-by-default — advisor digest (plan-step hat wave)

Two seats, dispatched `--kind advisor` at the plan step: **blast radius** and
**contracts & proof**. Synthesis and the three tie-break calls are in
`../plan.md` ("Settled design points"); this file is the evidence they rest on.

## Seat 1 — blast radius

1. **No data collision between the two settings writers.** The apply loop runs
   items in plan order (`apply.rs:321`) and `merge_repo_settings` reads the file
   fresh at apply time (`hooks_wiring.rs:210`), preserving every foreign top-level
   key (`hooks_wiring.rs:261-262`). Plan-time merge results are never reused
   (`plan.rs:917` is a yes/no). Two sequential, read-fresh, add-only items are
   safe. Do **not** fold the statusline into `merge_hooks_file` — that writer is
   gated on `opts.repo_hooks` and this must not be.
2. **plugin-first must write it too.** `--plugin-source` forces
   `repo_hooks: false` (`mod.rs:361-365`), so `merge_repo_hook_settings` is
   absent there. A Claude plugin cannot supply a `statusLine`. "plugin-first
   never touches settings.json" is not a property anything relies on —
   `bee dev plugin-distribution --apply` already rewrites that file via
   `clean_hook_config` (`plugin_distribution.rs:40,505`), and a `statusLine` key
   passes through it untouched. `managed.statusline` needs no plugin-source
   carry-forward arm (unlike `managed.repo_hooks` at `apply.rs:694-700`) because
   it is rebuilt from the flag every run.
3. **Windows.** `host_shell_is_powershell` exists (`merge.rs:48-58`) but only
   switches the AGENTS.md doctrine section (`plan.rs:618`); the Claude repo hook
   commands have no Windows variant (`hooks_wiring.rs:60-69`) and only Codex gets
   `commandWindows` (`hooks_wiring.rs:394`). A failed hook is silent; a failed
   `statusLine` is drawn on **every prompt**, and a bare `bash` on native Windows
   can resolve to the WSL launcher, which cannot open a `C:\…` path — the repo
   already warns about this at `hooks_wiring.rs:350`.
4. **`jq`.** `statusline-command.sh:8-13` hard-requires `jq` and prints
   `statusline: jq not found` in **red** without it. macOS and stock Ubuntu ship
   none. Under opt-in that was the user's choice; under opt-out it is bee's.
5. **The manifest converges in one pass** — hashes come from the source template,
   not the host copy (`plan.rs:292-299`), so they are deterministic per bee
   version. Two conditions: the `statusline` bool is computed once and means "bee
   owns the entry"; and the written command must satisfy `statusline_opt_in`.
   The canonical `bash "${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh"`
   **does** match (`hooks_wiring.rs:613-618`, the `}/.claude/…` tail arm). The
   bare `bash "$CLAUDE_PROJECT_DIR"/.claude/…` shape **does not**
   (`hooks_wiring.rs:1040`) and must never be what bee writes — otherwise the
   host silently un-adopts: no re-vendoring, `managed.statusline` dropped, and
   nothing says so.
6. **Backup-of-a-backup.** `merge_repo_hook_settings` copies to `.bak` before
   writing (`apply.rs:511-515`). A second writer taking its own `.bak` leaves the
   backup holding an already-modified file — in either order — and
   `scripts/install.sh:79` promises `.bak` restores the pre-bee state.

**Top risks if built naively:** (a) a malformed `settings.json` rewritten as bare
`{"statusLine": …}`, because `read_json_if_exists` collapses broken and missing
into the same `None` (`util.rs:55-61`) — the exact bug `merge_hooks_file` was
hardened against (`hooks_wiring.rs:197-229`); (b) a red error on every prompt for
most new hosts (no `jq`; or a tracked `settings.json` pointing at an untracked,
not-yet-committed `.sh` on a teammate's clone — the script is not in
`GITIGNORE_BLOCK_PATTERNS`); (c) silent un-adoption per point 5.

Side finding, pre-existing: `LEDGER_GROUPS` maps `statusline` to
`.bee/bin/statusline` (`plan.rs:239`) while the vendor target is `.claude/`
(`plan.rs:788`) — the regen obligation (`verbs/cells/obligation.rs:112`) guards a
directory that does not exist. Harmless under opt-in; worth a backlog row now.

## Seat 2 — contracts & proof

**Confirmed:** `src/onboard/tests.rs` has zero `statusLine` fixtures — the
vendoring plan/apply path genuinely has no live test, exactly as the knowledge
doc's Open Gaps says. `hooks_wiring.rs:1023-1062` covers detection only;
`tests/statusline_contract.rs` covers byte-equality and script execution only.

**R1** dies as written and is replaced (onboarding writes the entry when the key
is absent, creating the file; a present key is preference; `--no-statusline`
suppresses both). **R2** is reworded, keeping its soul — an unrecognized shape
means *leave alone*, never abort. **R3** (project-level anchoring) survives in
substance, renamed from "opt-in" to "project-level reference"; its negative half
is newly load-bearing, since a user-level path is now a *present* key and so is
preference. **R4** (byte-equality) survives unchanged but for the same rename.

**Edge cases.** l.133-136 keep their outcome (no action) but change their reason.
"Opting out after having been opted in" becomes the ordinary path rather than a
corner, and splits: a hand-deleted key is re-added on the next run, while
`--no-statusline` on a host that already carries the record leaves both a stale
managed record and an orphaned vendored script. Newly reachable and owed a case
each: absent `.claude/`, absent `settings.json`, valid file without the key, both
writers in one apply, and `bee dev regen` on this checkout staying a no-op.

**Doc edit list** (row-level, with current text quoted) — seat 2's table covers
`status-display-vendoring.md` lines 3, 4, 11, 13, 16, 18-22, 35, 36, 40-45,
47-52, 59-65, 115-117, 133-136, 139-144, 148-152, 172-173; plus
`overview.md:22,52`, `index.md:21`, `docs/specs/onboarding.md:13`,
`docs/product-description/maintenance/onboarding.md:172,183,197`. The spec's
"Was" column is historical by design and stays. `observability/status.md` needs
nothing.

**Flag surfaces:** `router.rs:98-99`, `generated/registry_payload.json`
(`onboard`, after `no-claude-md`), `onboard/mod.rs` header comment + `parse_args`
(beside `--no-claude-md` at l.170-171), `scripts/install.sh:63-68` help + l.143
parser + l.386-387 `ONBOARD_FLAGS` + a Safety line near l.85-87,
`scripts/install.ps1:48,536`, `INSTALL.md:33-34,175-176,184`, `README.md:546,616`.

**verify-app:** `.bee/verify/verify-app/features/onboard.md` exists (last commit
2026-09-02, `5a8d1921`). It maps plan/apply/idempotent/target but says nothing
about the statusline. It owes a "Confirm what landed" line for both `.claude`
files and a Gotchas line beside the existing `--repo-hooks` row, stating the
opposite default.

**Test cases owed:** 20, split into the five that close the pre-existing
vendoring gap and the fifteen new to opt-out. The list is reproduced in
`../plan.md` ("Proof") in condensed form and expanded in the cells.

Side finding, not this change's job: `docs/specs/onboarding.md:33` cites
`scripts/okf_migrate.mjs --check onboarding`; that script no longer exists.
