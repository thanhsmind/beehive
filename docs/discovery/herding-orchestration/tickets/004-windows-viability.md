---
type: research
status: closed
claimed-by: none
blocked-by: none
---

## Question

Can a Rust orchestrator driving herdr actually run on Windows, or does
the requirement collapse on a platform dependency?

## Answer

It can. Both halves are already there.

**bee side** — Windows is already the stated primary platform, not an
aspiration. `.github/workflows/windows.yml:4-5` says "win32 is bee's
primary platform" and runs the full unexcluded `cargo test --release`
suite on `windows-latest` for every push and PR; a second job
syntax-checks `scripts/install.ps1` against PowerShell 5.1.
`release-binaries.yml:44-51` ships `x86_64-pc-windows-msvc` beside
`x86_64-unknown-linux-gnu`. The crate carries real platform splits:
`windows-sys` vs `libc` dependencies, Win32 process-liveness and
sharing-violation handling in `src/lock.rs:126-186`, a from-scratch
Git-Bash resolver in `src/shell.rs` that deliberately excludes the WSL
launcher, `dunce` for UNC prefixes, and `MAIN_SEPARATOR`-aware path
handling in `src/roots.rs`.

**herdr side** — its release notes record native Windows support: an
app-local ConPTY runtime shipped in the Windows archive, Windows
`agent start`, Windows agent detection across Git-Bash `exec`
boundaries, detached Windows servers surviving OpenSSH logout, and
Windows `pane send-keys` / `agent send-keys` key semantics.

**The actual blockers are the two bash scripts, and only a few lines of
them.** `control-loop.sh` uses GNU coreutils `timeout` (absent on stock
Windows) and a bash-4.3 nameref (`local -n`, no Windows equivalent).
`bootstrap-cockpit.sh` uses `BASH_SOURCE`. Neither script uses signals,
job control, or `mktemp` — everything else in them is flow control that
a Rust subcommand replaces outright. Both already treat `.bee/bin/bee`
as the source of JSON truth, so a `bee herding` subcommand is an
extension of the existing shape, not a new architecture.

Two gaps to carry into planning:

- The crate has **no async runtime and no thread-pool crate** today —
  no tokio, async-std, rayon, or crossbeam. Concurrency is a new
  dependency decision either way.
- The crate has **no test seam for an external binary**. Every `git`
  call is a bare `std::process::Command`. Testing a herdr driver needs
  one; the nearest existing precedent is the `BEE_POSIX_SHELL` override
  and the PATH-prepend trick in `src/shell.rs:74-89`.

Logged as D04.
