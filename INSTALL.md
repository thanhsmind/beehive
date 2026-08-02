# Installing bee

Source: **https://github.com/thanhsmind/beehive**

## Quick install (recommended): the install script

One command does everything below — fetches bee, installs the skills for the chosen runtimes, and onboards the target repo (greenfield or brownfield). **The current directory is the target by default** — `cd` into your project first. It always shows the exact plan and asks before writing (skip prompts with `-y`/`-Yes`).

macOS / Linux / Git Bash:

```bash
curl -fsSL https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.sh | bash -s -- -y
```

Windows PowerShell:

```powershell
iwr -useb https://raw.githubusercontent.com/thanhsmind/beehive/main/scripts/install.ps1 -OutFile install-bee.ps1
.\install-bee.ps1 -Yes
```

To target another directory instead, add `-d /path/to/project` (bash) / `-Directory C:\path\to\project` (PowerShell).

From a local clone: `scripts/install.sh [-d <target>]` / `.\scripts\install.ps1 [-Directory <target>]`.

Useful flags (same semantics in both scripts):

| bash                            | PowerShell        | Effect                                                                                                                                     |
| ------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `--dry-run`                   | `-DryRun`       | Show the exact plan for YOUR repo; write nothing                                                                                           |
| `--runtime claude\|codex\|both` | `-Runtime …`   | Which runtime skills to install (default both)                                                                                             |
| `--global-skills`             | `-GlobalSkills` | Also copy skills into the legacy global runtime dirs (`~/.claude/skills`, `~/.codex/skills`). Off by default — see "two layers" below |
| `--no-claude-md`              | `-NoClaudeMd`   | Skip writing/extending CLAUDE.md with the`@AGENTS.md` import (written by default)                                                        |
| `--claude-md`                 | `-ClaudeMd`     | Accepted for compatibility; a no-op alias of the default (CLAUDE.md is written unless`--no-claude-md`/`-NoClaudeMd` is passed)         |
| `--no-hooks`                  | `-NoHooks`      | Skip repo-local hook wiring for Claude Code                                                                                                |
| `--no-git-init`               | `-NoGitInit`    | Greenfield: don't offer`git init`                                                                                                        |
| `--source <path>`             | `-Source …`    | Use a local bee checkout instead of cloning                                                                                                |
| `-y`                          | `-Yes`          | Non-interactive                                                                                                                            |

**Greenfield** (new/empty directory): the script creates the directory, offers `git init`, and installs everything fresh. **Brownfield** (existing repo): existing `AGENTS.md`/`CLAUDE.md` content is preserved byte-for-byte outside the managed BEE markers; `.bee/` state, decisions, and cells are never overwritten; `.claude/settings.json` merges get a `.bak` backup; re-running is idempotent (`up_to_date`). Run `--dry-run` first if you want to see the plan before anything is written.

The script uses the manual-copy route for skills. If you prefer the Claude Code **plugin route** (hooks ship automatically, centrally updatable), use Option A below instead and then run only step 3.

---

## Manual installation

bee installs in two layers:

1. **Repo layer** (once per project, the default): onboarding installs the `AGENTS.md` BEE block, the `.bee/` runtime directory, the vendored `bee` binary, a `CLAUDE.md` `@AGENTS.md` import, and a per-project copy of the `bee-*` skills into the repo itself — `<repo>/.claude/skills` for Claude Code, `<repo>/.agents/skills` for Codex. These skill trees are committed to the host repo (same policy as the vendored CLI), so every teammate and CI job sees identical skills without any machine-wide install; re-onboarding refreshes them.
2. **Runtime layer** (opt-in, once per machine): a legacy global copy of the `bee-*` skills into `~/.claude/skills` and/or `~/.codex/skills`. Nothing in this layer is touched unless you pass `--global-skills` (`-GlobalSkills`) — the per-project copy above is what agents actually discover by default. On Claude Code, the hook skeleton still needs one of the routes below (the plugin, or `--repo-hooks` during onboarding).

Requirement for both on x86_64 Linux/Windows: **none** — each installer downloads the release binary for the platform, checks it against the release `SHA256SUMS`, and falls back to a source build only if no asset fits or `--build-from-source` / `-BuildFromSource` is given. For that fallback: **a Rust toolchain** (`cargo --version`, stable). bee ships
as a single native binary and, by decision 1f4262ca, no prebuilt binaries live in
the repo — you build it once per machine from the source checkout:

```bash
cargo build --release --manifest-path packages/bee-rs/Cargo.toml
```

Then copy (or symlink) the result into the repo that will use it:

```bash
cp packages/bee-rs/target/release/bee     <your-repo>/.bee/bin/bee      # macOS / Linux
cp packages/bee-rs/target/release/bee.exe <your-repo>/.bee/bin/bee.exe  # Windows
```

`.bee/bin/bee[.exe]` is machine-local and git-ignored. **Node.js is no longer
required** — the entire runtime (CLI, hooks, statusline, dev tools) is that one
binary. The single exception is Codex on native Windows, whose hook transport
uses a ~10-line `node -e` launcher to find the repo root before handing off to
the binary; every other transport is Node-free.

> Path used in the examples: `D:\projects\tools\AI\bee`. Replace with wherever this plugin lives (a local clone of `thanhsmind/beehive` or the git URL).

---

## 1. Claude Code

### Option A — plugin install (recommended)

The plugin ships skills **and** the 9-script hook automation skeleton (`hooks/hooks.json`); both load automatically once installed.

Inside a Claude Code session:

```text
/plugin marketplace add D:\projects\tools\AI\bee
/plugin install bee@bee
```

(For a git-hosted copy: `/plugin marketplace add <owner>/<repo>` or the full URL, then the same install command.)

Restart the session, then verify:

- `/plugin` → bee shows as installed and enabled.
- Ask: "What bee skills do you have?" → the 15 `bee-*` skills should be listed.
- Hooks self-arm only in onboarded repos (they exit silently when `.bee/onboarding.json` is absent), so no hook activity is expected yet — that changes after step 3.

### Option B — no plugin system (fallback)

If you can't (or don't want to) use the plugin manager, onboarding (step 3 below) copies the skills into the repo for you by default — no manual step needed. To copy by hand instead (or to seed the legacy global dir):

1. Copy the skills to a skills directory Claude Code reads:

   - per repo (default, what onboarding does): `<repo>\.claude\skills\`
   - or per user (opt-in, legacy): `%USERPROFILE%\.claude\skills\` (macOS/Linux: `~/.claude/skills/`) — pass `--global-skills`/`-GlobalSkills` during onboarding/install to have the script do this too

   ```powershell
   Copy-Item -Recurse D:\projects\tools\AI\bee\skills\* <repo>\.claude\skills\
   ```
2. Wire the hooks per repo during onboarding with `--repo-hooks` (step 3 below) — this copies the hook scripts into `<repo>\.bee\bin\hooks\` and merges the 6 entries into `<repo>\.claude\settings.json` (a `.bak` backup is created; re-running never duplicates entries).
3. CLAUDE.md's `@AGENTS.md` import is written by default during onboarding (opt out with `--no-claude-md`) so the BEE block auto-loads even if hooks are disabled.

---

## 2. Codex

### Option A — plugin manifest

For Codex builds with plugin support, install from the plugin directory/repo; the manifest at `.codex-plugin/plugin.json` exposes `skills: ./skills/`, and the skills are self-prefixed (`bee-*`), so they stay namespaced even as a plain copy.

### Option B — manual skills copy (always works)

Onboarding (step 3 below) populates the repo-level path by default — no manual step needed. Codex's **repo-level** skill discovery path is `<repo>/.agents/skills/` (cwd up to the repo root), **not** `.codex/skills` — that repo-level location is not a Codex discovery path at all. `~/.codex/skills` (`$CODEX_HOME/skills/`, default `~/.codex/skills/`) is the legacy **global** location; it's opt-in via `--global-skills` and is only populated by the install scripts (`install.sh`/`install.ps1`), not by `bee onboard` directly.

To copy by hand instead:

```bash
cp -r /d/projects/tools/AI/bee/skills/* <repo>/.agents/skills/    # repo-level (what onboarding does)
cp -r /d/projects/tools/AI/bee/skills/* ~/.codex/skills/          # legacy global (opt-in)
```

```powershell
Copy-Item -Recurse D:\projects\tools\AI\bee\skills\* <repo>\.agents\skills\        # repo-level
Copy-Item -Recurse D:\projects\tools\AI\bee\skills\* $env:USERPROFILE\.codex\skills\  # legacy global
```

Codex loads project hooks from `.codex/hooks.json` (8 lifecycle events shipped: SessionStart, UserPromptSubmit, PreToolUse, PostToolUse, SubagentStart, SubagentStop, PreCompact, Stop) — the earlier claim that Codex lacked hook support was stale. Bootstrap still comes from the `AGENTS.md` BEE block (installed in step 3) regardless of hook state, and every gate- and integrity-critical rule is enforced by the vendored `bee` binary, identically to Claude Code — hooks are a second belt, not the only one. See §4 below for the Codex hook verify procedure and [docs/06-runtime-integration.md](docs/06-runtime-integration.md) for the parity matrix.

### Codex permission policy vs bee gate_bypass

Codex's `approval_policy` (in `.codex/config.toml`) and bee's `gate_bypass` (in `.bee/config.json`, set via the `bee-bypass-gate` skill) are **distinct concepts** governing different layers — setting one never sets the other. `approval_policy` decides whether Codex asks before running a tool call (edit, shell command, etc.); `gate_bypass` decides whether bee auto-approves its own workflow gates (the exploring/planning/validating/reviewing chain). A third, independent layer sits underneath both: Codex hook **trust** — a changed `.codex/hooks.json` may be skipped pending a `/hooks` review no matter what `approval_policy` or `gate_bypass` are set to (see the Codex hook verify procedure in §4).

bee ships **no `approval_policy` default** in any distributed template or renderer — the host repo owner chooses. Two recommended profiles:

| Profile           | `approval_policy` | `gate_bypass` | Trade-off                                                                                                                                                 |
| ----------------- | ------------------- | --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `bee-safe`      | `on-request`      | off /`normal` | Codex asks before risky tool calls, bee asks at every gate — slowest, most supervised                                                                    |
| `bee-autopilot` | `never`           | `total`       | Codex never interrupts for tool approval, bee never stops for a gate — fastest, zero-supervision; only appropriate when you trust the agent and the repo |

This repo's own working copy keeps `approval_policy = "never"` locally (`.codex/config.toml`) with `gate_bypass: "total"` in `.bee/config.json` — a deliberate local choice by this repo's owner, not the distributed default.

---

## 3. Onboard each repository (both runtimes)

From any terminal, plan first (dry-run, changes nothing):

```bash
bee onboard --repo-root <your-repo> --json
```

Review the reported plan, then apply:

```bash
bee onboard --repo-root <your-repo> --apply
```

Flags:

| Flag                | Effect                                                                                                                                                                                |
| ------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `--apply`         | Actually install (without it: report-only)                                                                                                                                            |
| `--repo-hooks`    | Additionally copy hooks into`.bee/bin/hooks/` and merge them into `<repo>/.claude/settings.json` (Claude Code fallback when not using the plugin manager)                         |
| `--no-claude-md`  | Skip writing/extending CLAUDE.md's`@AGENTS.md` import (written by default)                                                                                                          |
| `--claude-md`     | Accepted for compatibility; a no-op alias of the default                                                                                                                              |
| `--global-skills` | Also sync the legacy global`~/.claude/skills` root (Claude Code only — Codex's `~/.codex/skills` global copy is handled by the install scripts, not this script). Off by default |
| `--json`          | Machine-readable output                                                                                                                                                               |

What onboarding installs:

```
<repo>/AGENTS.md          ← BEE block between <!-- BEE:START --> / <!-- BEE:END --> (content outside markers untouched)
<repo>/CLAUDE.md          ← @AGENTS.md import, appended once (default; opt out with --no-claude-md)
<repo>/.bee/              ← onboarding.json, state.json, config.json (+ empty cells/, logs/)
<repo>/.bee/bin/          ← bee[.exe] (the binary, machine-local) + prompts/
<repo>/.claude/skills/    ← bee-* skills, per-project copy for Claude Code (committed to the repo)
<repo>/.agents/skills/    ← bee-* skills, per-project copy for Codex repo-level discovery (committed to the repo)
<repo>/docs/history/learnings/critical-patterns.md   ← stub if missing
```

Existing `state.json`, `decisions.jsonl`, and `cells/` are **never** overwritten; re-running is idempotent and reports `up_to_date`.

Alternatively, do it conversationally: open a session in the repo and say **"Onboard this repository for bee"** — `bee-hive` runs the same command and asks before `--apply`.

---

## 4. Verify the install

In the onboarded repo:

```bash
.bee/bin/bee status --json
```

Expect `onboarding.installed: true`, `phase: "idle"`, all gates `false`.

Check the per-project skill trees landed (both are committed to the repo, both populated by default):

```bash
ls <repo>/.claude/skills | grep bee-   # Claude Code project discovery
ls <repo>/.agents/skills | grep bee-   # Codex repo-level discovery
```

Each should list all 15 `bee-*` skill dirs. If you passed `--global-skills`, also expect `~/.claude/skills/bee-*` (and, via the install scripts, `~/.codex/skills/bee-*`).

Claude Code (plugin route) — start a new session in the repo: the session should begin with the bee preamble (phase, gates, critical-patterns digest) injected by `bee-session-init`. Quick hook check by hand:

```bash
echo '{"tool_name":"Write","tool_input":{"file_path":"src/x.ts"}}' | .bee/bin/bee hook write-guard
```

(with `--repo-hooks` install; for the plugin route the hooks run from the plugin directory — just watch the session preamble instead).

Codex — start a session in the repo: the agent should follow the AGENTS.md BEE block and run `bee status` as its first scout step. Then try: "Route this through bee: fix the typo in README" → expect tiny-lane routing, not ceremony.

Codex hook verify procedure — three-state model, `hooks_file_present ≠ hooks_discovered ≠ hooks_trusted_and_observed`; a file shipping in `.codex/hooks.json` is never by itself evidence that a hook ran:

1. **Trust** — confirm the project directory is trusted. Codex refuses to run project-local hooks/commands from an untrusted directory, so an untrusted `.codex/` never reaches "discovered".
2. **Review** — where the installed Codex version exposes a `/hooks` step, review and trust the bee hooks there. Trust semantics for that step are still being confirmed by the capability spike; treat this step as conditional on your installed version, not guaranteed.
3. **Observed** — evidence differs by event. `.bee/logs/tools.jsonl` (written by `bee-tools-logger`, the general PostToolUse success log) is where a healthy PostToolUse hook run shows up; `.bee/logs/hooks.jsonl` records only crashes and a narrow set of subagent-lifecycle cases, so a healthy session does **not** necessarily add a `hooks.jsonl` row — its absence is not evidence of failure. Check `tools.jsonl` for the PostToolUse row, or watch the session for a hook's `statusMessage` (e.g. `bee: state sync`), to confirm `hooks_trusted_and_observed` for a given event.

Smoke the enforcement (any runtime, any agent):

```bash
.bee/bin/bee cells claim --id anything --worker w1
# → refuses: gate "execution" is not approved  ✔ the CLI is armed
```

---

## 5. Update / uninstall

**Update:** pull/copy the new plugin version, then re-run onboarding per repo (`--apply`) — it detects drift via managed versions in `.bee/onboarding.json` and refreshes the AGENTS block + CLI. Plugin route: `/plugin update bee` (or re-add the marketplace) as well.

**Uninstall (per repo):** delete the BEE block (everything between and including the `BEE:START`/`BEE:END` markers) from `AGENTS.md`, remove `.bee/`, and — if `--repo-hooks` was used — remove every `bee-*` entry from `.claude/settings.json`. `docs/history/` is yours; keep it.

**Uninstall (runtime):** `/plugin uninstall bee` on Claude Code, or delete the copied skill folders.

---

## Troubleshooting

| Symptom                                                                                | Cause / fix                                                                                                                                                                                                                                                                                       |
| -------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Skills don't appear                                                                    | Plugin not enabled (`/plugin`), or the repo hasn't been onboarded yet (per-project `.claude/skills`/`.agents/skills` are populated by onboarding, not by a separate install step); restart the session after installing                                                                     |
| Codex doesn't see bee skills                                                           | Repo-level discovery is`.agents/skills`, not `.codex/skills` — check that path was populated by onboarding; `~/.codex/skills` is legacy/global and only exists if you passed `--global-skills` to the install script                                                                     |
| `install.ps1` fails to parse on Windows PowerShell 5.1                               | Historically caused by non-ASCII bytes (em-dashes) in a UTF-8-no-BOM file decoding as cp1252 smart quotes, which terminate strings mid-line.`install.ps1` is ASCII-only now and a repo test guards `scripts/*.ps1` against non-ASCII bytes — report this as a regression if you still hit it |
| No session preamble in Claude Code                                                     | Repo not onboarded (`.bee/onboarding.json` missing — hooks self-arm only after onboarding), or hook disabled in `.bee/config.json → hooks.session-init`                                                                                                                                     |
| `claim`/`cap` refuse unexpectedly                                                  | Working as designed: check`bee status` for gate states — execution must be approved (Gate 3), cells must have a passing recorded verify before capping                                                                                                                                         |
| Hook crash suspected                                                                   | Hooks are fail-open; check`.bee/logs/hooks.jsonl`                                                                                                                                                                                                                                               |
| `cargo` not found                                                                    | Only reached when no published binary fits this host (or `--build-from-source` was passed). Install Rust (rustup) and reopen the terminal/session                                                                                                                                                                            |
| `bee: command not found`, or hooks print `bee: hook binary missing (.bee/bin/bee)` | The binary has not been built or copied into`<repo>/.bee/bin/`. Run the `cargo build --release` + copy from the Requirements section above                                                                                                                                                    |
