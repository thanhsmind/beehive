<!-- bee:no-read-set: bugfix from the Windows CI log of run 43189304799 and a local cargo check --target x86_64-pc-windows-gnu reproduction -->

# windows-test-build — context

The Windows CI lane has failed to compile the bee tests since commit
a4e5c302f ("Let a supervising repo read the bee store of its
waggledance-managed projects"). That commit added three tests that only work
on Unix. One makes a symlink with `std::os::unix::fs::symlink`. Two use a
helper that writes a `#!/bin/sh` script, sets mode `0o755` with
`PermissionsExt`, and joins `PATH` with `:`. A build error hides every other
Windows failure (pattern 20260821-a-platform-job-red-on-a-build-error).

## Locked Decisions

| ID | Decision | Reason |
|---|---|---|
| D1 | Gate the three tests and the shell-script helper with `#[cfg(unix)]`. Do not port them. | They drive a POSIX shell script and a Unix symlink. A Windows port is new test behavior, out of scope for a build fix. |
| D2 | After the build compiles on Windows, report any newly visible Windows test failures as backlog rows. Do not fix them in this change. | A build-red lane hides an unknown number of failures. Triage them from a real run. |
