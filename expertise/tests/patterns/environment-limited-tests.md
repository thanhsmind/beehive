# Environment-limited tests

## Context

When a case needs a capability the host may not have: symlink creation,
elevated privilege, a POSIX shell, a specific filesystem, a network, a
particular locale. The code is fine; the machine cannot express the
scenario. Left alone, the case fails on those hosts — and a failure that
means "this laptop lacks a privilege" reads exactly like a product defect,
so people learn to expect red and stop reading the suite.

## Mechanism

Probe the capability, then branch:

- **Capable** → run the case normally, with no change in coverage.
- **Not capable** → print one line per skipped case that names the missing
  capability and how to obtain it, and pass.

Three rules keep this honest:

1. **Probe the capability, never the platform.** `cfg!(windows)` is a guess
   about a machine you are not looking at; attempting the operation in a
   temp directory is the fact. Privilege, filesystem, and shell availability
   all vary within a platform.
2. **Skip the narrowest set.** Skip only the cases that genuinely need the
   capability. In a suite where three cases mount a symlink and nine do not,
   skipping the whole file loses nine cases of real coverage.
3. **Name the capability in every skipped line.** "SKIP (env)" tells a
   reader nothing. The line has to say what was missing, so anyone can
   decide whether to enable it — and so a *permanent* skip cannot hide as
   an environmental one.

## Example

```rust
// Probe once: attempt the real operation in a scratch directory.
static SYMLINK_CAPABLE: OnceLock<bool> = OnceLock::new();
fn symlink_capable() -> bool {
    *SYMLINK_CAPABLE.get_or_init(|| {
        let dir = tempfile::tempdir().unwrap();
        std::os::windows::fs::symlink_dir(dir.path(), dir.path().join("l")).is_ok()
    })
}

#[test]
fn mounts_the_companion_as_a_symlink() {
    if !symlink_capable() {
        // Good — names the capability and how to get it.
        eprintln!("SKIP (env: symlink creation denied — needs Developer Mode \
                   or an elevated shell) — mounts the companion as a symlink");
        return;
    }
    …
}
```

```rust
// Bad — a platform guess, not a capability probe: it skips on every Windows
// host, including the ones where the case would have passed.
if cfg!(windows) { return; }
```

## Notes

A skip is a debt, not a resolution. Coverage that only ever runs on some
machines is coverage the team does not really have — so if a capability is
missing on the machines that matter (CI, the release host), fixing the
environment is the real work, and the skip is the marker that says so out
loud until someone does.

Watch for the inverse failure too: a case that *silently* passes because the
capability was absent and the code did nothing. A skip is visible by
construction; a vacuous pass is not.
