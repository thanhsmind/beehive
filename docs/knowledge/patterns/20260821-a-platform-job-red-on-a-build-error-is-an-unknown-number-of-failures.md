---
type: bee.pattern
title: A platform job red on a build error is an unknown number of failures
description: A compile error on one platform hides every test behind it, so the visible failure count is one and the real one is unknown until the build is green
tags: [failure, ci, cross-platform, tests, verification]
timestamp: 2026-08-21
bee:
  id: pattern-20260821-a-platform-job-red-on-a-build-error-is-an-unknown-number-of-failures
  lifecycle: active
  areas: [verify-pipeline, rust-runtime]
  sources: [".bee/cells/archive/windows-build-red/wbr-1.json", ".bee/cells/archive/windows-suite-green/wsg-1.json", ".bee/cells/archive/windows-suite-green/wsg-2.json", "original features: windows-build-red, windows-suite-green"]
  polarity: pitfall
  critical: true
  evidence: prose
  evidence_ref: "the Windows lane runs `cargo test --release` with no preceding build step (.github/workflows/windows.yml), so a compile error and a failing assertion arrive as the same red; the Linux lane (.github/workflows/ci.yml) already builds separately"
---

# A platform job red on a build error is an unknown number of failures

`herding/run.rs` used `std::os::unix::fs::PermissionsExt` in a test with no
`cfg(unix)`. The Windows job did not fail a test — it failed to BUILD, and the
whole crate went unrun on that platform. The lane showed one red. The moment
that one attribute landed, eleven real Windows failures appeared that had been
invisible for as long as the build was broken.

Those eleven carried three root causes, and the mix is the point — two were
test-only and one was a production defect nobody could see:

- Assertions hard-coding forward-slash paths while the renderer builds them
  with `Path::join` and renders with `display()`, which emits backslashes.
- A fixture binding a tempdir's raw path while the hook realpaths its root, so
  on a runner with 8.3 short names the transcript is never found. One such
  fixture had been passing for the wrong reason without ever failing.
- `expand_tilde` reading only `HOME`, which is normally unset on Windows, so a
  `~/...` path never expanded — a real production bug.

**The rule:** a platform job that goes red on a BUILD error is not one failure,
it is an unknown number of them. Fix the build first, then re-read the list.
Never triage, estimate, or schedule against a failure count taken from a lane
that did not compile.

**The check that beats this prose:** split the build from the test run in the
lane itself, so the two reds are distinguishable without reading logs. `cargo
test --no-run` is the right split rather than `cargo build`, because it
compiles the `cfg(test)` code — which is exactly where this defect lived and
which a plain build never touches.
