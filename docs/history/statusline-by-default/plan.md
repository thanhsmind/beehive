# statusline-by-default — plan

Route: class=feature · lane=standard · flags=2 (public-contracts, cross-platform) · files=5
Class playbook: `bee-planning/references/planning-reference.md` ("Class playbooks" → feature)
Worktree: `beehive--wt--statusline-by-default` (branch `wt/statusline-by-default`)

## The change in one line

Onboarding stops waiting to be invited: a host whose `.claude/settings.json`
declares no `statusLine` key gets the canonical project-level entry written for
it and the script vendored beside it. A host that already declares one is
untouched.

## Locked decisions

- **4cac0774** (supersedes **102efe08**) — "Onboarding installs the status
  display by default (opt-out) … when a host `.claude/settings.json` carries NO
  `statusLine` key, onboarding writes the canonical project-level entry AND
  vendors `statusline-command.sh`; when the key is already present it is
  preference and is left byte-for-byte alone (same add-only contract as
  `ensure_codex_statusline`). A `--no-statusline` flag on `bee onboard`
  suppresses the write entirely."
- **c6ee6b6e** — anchored project-level detection (R3) and canonical/vendored
  byte-equality (R4). Both survive: R3 still decides *drift healing* for hosts
  that already carry an entry, R4 is untouched.
- **102efe08**'s surviving half — never impose on a host that chose otherwise.
  It lives on as the add-only rule (a present `statusLine` is preference) plus
  the `--no-statusline` opt-out.

## Discovery — reality touches

| Claim | Label | Anchor | Evidence |
|---|---|---|---|
| Statusline vendoring is gated entirely on the host already declaring the entry | read | `src/onboard/plan.rs:785` | `if hw::statusline_opt_in(repo_root) {` wraps the whole of stage 3b |
| Onboarding never writes the `statusLine` key anywhere | read | `src/onboard/hooks_wiring.rs:584-598` | `statusline_opt_in` only *reads* `.claude/settings.json`; no writer exists |
| `.claude/settings.json` is written only under `--repo-hooks` | read | `src/onboard/plan.rs:916-919` | `merge_repo_hook_settings` sits inside `if opts.repo_hooks` |
| `--plugin-source` forces `repo_hooks: false` | read | `src/onboard/mod.rs:361-365` | `repo_hooks: if args.plugin_source { false } else { … }` |
| The installer defaults to repo-copy + repo-hooks | read | `scripts/install.sh:118,124` | `DISTRIBUTION_MODE="repo-copy"`, `REPO_HOOKS=1` |
| `merge_hooks_file` preserves every foreign key and returns whole-file text | read | `src/onboard/hooks_wiring.rs:205-264` | `let mut out = existing; out.insert("hooks", …)` |
| The hook-settings apply writes a `.bak` before its atomic write | read | `src/onboard/apply.rs:506-518` | `std::fs::copy(&target, …".bak")` then `write_file_atomic` |
| `ensure_codex_statusline` is the add-only precedent | read | `src/onboard/apply.rs:529-544`, `plan.rs:944-946` | skips when the key is already present; machine-level target |
| The vendoring plan/apply path has NO live test | read | `src/onboard/tests.rs` (statusline appears only at l.43, l.81) | `rg statusline src/onboard/tests.rs` — fixture wiring only, no case |
| Canonical/vendored byte-equality IS guarded | ran | `tests/statusline_contract.rs:47` | `canonical_and_vendored_statusline_are_byte_identical` |
| This repo's own entry is the canonical shape to write | read | `.claude/settings.json` | `bash "${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh"` |
| `read_json_if_exists` collapses unparseable AND missing into `None` | read | `src/onboard/util.rs:55-61` | `serde_json::from_str(&text).ok()` — a broken file is indistinguishable from an absent one |
| The apply loop is sequential and every settings writer re-reads disk | read | `src/onboard/apply.rs:321`, `hooks_wiring.rs:210` | plan-time merge results are never reused at apply; `plan.rs:917` is a yes/no only |
| `.claude/settings.json` is NOT the only settings writer bee has | read | `src/devtools/plugin_distribution.rs:40,505` | `clean_hook_config` already rewrites it in plugin-first; a `statusLine` key passes through |
| The status script hard-requires `jq` and prints a RED error without it | read | `packages/bee/statusline/statusline-command.sh:8-13` | `[ -z "$JQ" ] && { printf … '\033[31mstatusline: jq not found\033[0m'; exit 0; }` |
| There is no PowerShell variant of the Claude repo hook commands | read | `src/onboard/hooks_wiring.rs:60-69`, `merge.rs:48-58` | `host_shell_is_powershell` exists but only switches the AGENTS.md doctrine section (`plan.rs:618`) |
| Both `.claude` files are tracked in this repo | ran | `git ls-files .claude/…` | `settings.json` and `statusline-command.sh` both listed |
| `LEDGER_GROUPS` maps `statusline` to a directory that is never written | read | `src/onboard/plan.rs:239` vs `:788` | ledger says `.bee/bin/statusline`, the vendor target is `.claude/` — pre-existing, now load-bearing |

## Smaller path check

*Is there a cheaper shape that still honors 4cac0774?*

Considered: teach `scripts/install.sh` to seed the `statusLine` key before it
calls `bee onboard`, leaving the Rust untouched. **FAIL** — the installer is
one of several doors (`bee onboard` is run directly on upgrade, and by the
release flow); a fix that lives in one door leaves every other door on the old
behavior, and the decision says *onboarding* installs it. The shape below is
the smallest that puts the rule where the rule belongs.

## Settled design points

The advisor wave disagreed on three things the decision does not settle. Calls,
with reasons:

1. **A broken `settings.json` never blocks.** One seat wanted the new writer to
   join the 2b preflight and refuse like `merge_hooks_file`; the other pointed
   out that a refusal turns a default into a blocker for every host with an
   unparseable settings file. Taken: unparseable or non-object → *not absent*,
   so zero statusline actions and the run proceeds. This keeps R2's soul ("any
   shape it does not positively recognize means leave alone") and adds no new
   `blocked_*` path.
2. **One `.bak` per settings file per apply run.** One seat wanted the
   statusline writer to skip the backup entirely, the other to take one like the
   hooks merge. Either alone is wrong: two backups destroy the pristine copy
   (`scripts/install.sh:79` promises `.bak` restores the pre-bee state), and no
   backup leaves plugin-first hosts with none at all. Taken: whichever writer
   touches `.claude/settings.json` first takes the backup; the second sees it
   already taken and does not.
3. **The durable opt-out is `.bee/config.json`, not a remembered flag.**
   `--no-statusline` is per-run. Deleting the `statusLine` key by hand is a
   no-op — the next run adds it back — so the project-level, permanent answer is
   `.bee/config.json` `"statusline": false`, read exactly the way `host_shell`
   already is (`onboard/merge.rs:48-58`). Onboarding reads it and never writes
   it: no new state, no sticky-record machinery, and a second durable opt-out
   already exists for free (any foreign `statusLine` value is preference and is
   never rewritten).

## Shape

Four moving parts, in dependency order.

**1. A writer for the settings key** (`hooks_wiring.rs`).
`statusline_settings_absent(repo_root) -> bool` — true **only** when the file is
missing or empty, or it parses as an object with no `statusLine` key. A file
that does not parse, or whose top level is not an object, answers **false**: it
is not "absent", it is broken, and a naive "no key → write" would rewrite it as
bare `{"statusLine": …}` — the exact bug `merge_hooks_file` was hardened against
(`hooks_wiring.rs:197-229`). This cannot reuse `read_json_if_exists`, which
collapses broken and missing into the same `None` (`util.rs:55-61`). A present
key of any shape counts as declared, so a user-level path is preference and is
left alone.
`merge_statusline_settings(settings_path) -> Result<Merged, String>` — inserts
the canonical entry, preserves every foreign key, returns whole-file text plus
`changed`, and refuses with the same message shape as `merge_hooks_file` on
non-JSON or a non-object top level.

**2. The plan item** (`plan.rs`). Widen stage 3b's gate from
`statusline_opt_in` to `opts.statusline && (statusline_opt_in || settings_absent)`,
and push a `merge_statusline_settings` item **after stage 5** (so it runs after
the hooks merge — see part 3) when the key is absent. Gated on `--no-statusline`
alone, never on `repo_hooks` or `plugin_source`: a Claude plugin cannot supply a
`statusLine`, so a plugin-first host gets the display only if onboarding writes
it, and "plugin-first never touches settings.json" is not a property anything
relies on (`plugin_distribution.rs:40,505` already rewrites that file there).

**3. The apply case** (`apply.rs`), with an apply-time re-check of its own
condition (the `ensure_codex_statusline` plan-to-apply race pattern,
`apply.rs:534`) and the backup-once rule from settled point 2. No 2b preflight
row: settled point 1 means this writer has no failure mode to preflight.

**4. Convergence** (`plan.rs`, `build_managed_versions`). The manifest's
`statusline` key is conditional on the opt-in, and `scripts/install.sh:681`
fails the whole install when the post-apply recheck is not `up_to_date`. Two
things pin it, and both are one-line rules:

- The `statusline` bool is computed **once per run** and means *"bee owns the
  entry"* = `opts.statusline && (opt_in || key_absent)` — the same value feeds
  stage 3b, the new item, and `build_managed_versions`. Under `--no-statusline`
  it is `false` in plan and apply alike, or the ledger flips between flagged and
  unflagged runs (the trap `repo_hooks` solved with its sticky record,
  `mod.rs:361-365`).
- The command bee writes **must** satisfy `statusline_opt_in` on the next run,
  or the host silently un-adopts: no further re-vendoring, `managed.statusline`
  dropped, and nothing says so. Part 6's guarded string is verified against the
  oracle for exactly this reason.

With both held, the recheck converges in one pass: the hashes come from the
source template, not the host copy (`plan.rs:292-299`), so they are deterministic
per bee version.

**5. The opt-out** (`mod.rs`, `router.rs`, `scripts/install.sh`).
`--no-statusline` on `bee onboard` sets `Options.statusline = false`, and so
does `.bee/config.json` `"statusline": false` (settled point 3). Surfaces that
must list the flag: `router.rs:98`, `generated/registry_payload.json`
(`onboard`, beside `no-claude-md`), `scripts/install.sh` (help, parser,
`ONBOARD_FLAGS`), `scripts/install.ps1` (`-NoStatusline`), `INSTALL.md`,
`README.md:546,616`.

**6. Make the display safe to impose** (`packages/bee/statusline/statusline-command.sh`
and its vendored twin). Two defects are harmless under opt-in and are bee's
fault under opt-out:

- *No `jq`* prints `statusline: jq not found` in red on **every prompt**
  (`statusline-command.sh:8-13`). macOS and stock Ubuntu ship no `jq`. It must
  render nothing and exit 0, like every other unavailable fact.
- *No script* — `.claude/settings.json` is tracked and team-shared, the vendored
  `.sh` is not in the gitignore block, so a teammate's fresh clone (or a linked
  worktree) has the pointer without the file and gets a shell error on every
  prompt. The written command therefore guards its own target:
  `[ -f "${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh" ] && bash "${CLAUDE_PROJECT_DIR:-.}/.claude/statusline-command.sh"`.
  Verified against the detection oracle: the `}/.claude/…` tail arm matches
  (`hooks_wiring.rs:613-618`), so the host still reads as opted in on the next
  run. The bare `"$CLAUDE_PROJECT_DIR"/.claude/…` shape does **not** match
  (`hooks_wiring.rs:1040`) and must never be the string bee writes.

**7. Windows.** `host_shell_is_powershell(repo_root)` true → treat the run as
`--no-statusline` and say so in a notice. A hook that fails is silent; a
statusLine that fails is drawn on every prompt, and a bare `bash` on native
Windows can resolve to the WSL launcher, which cannot open a `C:\…` path
(`hooks_wiring.rs:350` already warns about this).

## Slices

**Slice 0 — make the display safe to impose.** Part 6's script fixes plus part
7's PowerShell skip. This lands *first*: default-on is only defensible once a
host without `jq`, without the script, or on a PowerShell shell gets silence
rather than a red line on every prompt.

**Slice 1 — the behavior, end to end.** Parts 1-5 plus their tests. A host with
no `statusLine` gets the key and the script in one apply, and an immediate
re-check reports `up_to_date`. This is the walking skeleton: no stubs, real
files, real convergence.

**Slice 2 — the written record.** The exact row-by-row edit list is in
`docs/history/statusline-by-default/reports/advisor-digest.md`. In short:
`docs/knowledge/areas/onboarding/status-display-vendoring.md` (title, frontmatter,
R1 replaced, R2 reworded, R3/R4 renamed only, "Stay out" recut, Edge Cases and
Open Gaps re-cut), `docs/knowledge/areas/onboarding/overview.md` and `index.md`,
`docs/specs/onboarding.md:13`, `docs/product-description/maintenance/onboarding.md:172,183,197`,
`INSTALL.md`, `README.md`, and `.bee/verify/verify-app/features/onboard.md`
(a "Confirm what landed" line for both `.claude` files, a Gotchas line for the
present-key and `--no-statusline` cases).

## Proof

- `cargo test -p bee onboard` and `cargo test -p bee statusline` — the changed
  module plus the standing byte-equality guard.
- New cases the change owes:
  - absent `statusLine` → plan carries `merge_statusline_settings` + one
    `copy_statusline` per canonical file
  - present `statusLine` (project-level) → no settings write, copy still planned
    on drift
  - present `statusLine` (user-level path) → settings untouched, no copy
  - `--no-statusline` → zero statusline items of either kind
  - foreign keys and an existing `hooks` block survive the write byte-for-byte
  - unparseable / non-object settings.json → `blocked_hooks_merge`, zero writes
  - apply then re-plan → empty plan (the convergence case)
  - both writers in one run → one `.bak`, holding the ORIGINAL file
  - the entry bee writes reads back as opted in — `statusline_opt_in()` is
    `true` against the exact string the new writer produces (the anti-silent-
    un-adoption case)
  - `--no-statusline` on a host bee already wrote to → still `up_to_date`, no
    ledger flip
  - no `jq` on PATH → the script prints nothing and exits 0
  - the settings entry with the script deleted → nothing drawn, exit 0
- `bee onboard --repo-root <sandbox> --apply` against a throwaway host, then a
  re-check that must print `up_to_date` (`verify-app` evidence).

## Open questions

None blocking. The advisor wave's findings are merged into the sections above
before the gate.
