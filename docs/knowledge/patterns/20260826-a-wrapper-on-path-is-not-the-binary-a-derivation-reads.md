---
type: bee.pattern
title: A wrapper on PATH is not the binary a derivation reads
description: A test that derives facts from an installed binary's own bytes silently derives nothing when PATH resolves to a wrapper script or a versionless shim
tags: [failure, tests, environment, path, release]
timestamp: 2026-08-26
bee:
  id: pattern-20260826-a-wrapper-on-path-is-not-the-binary-a-derivation-reads
  lifecycle: active
  areas: [verify-pipeline, rust-runtime]
  sources: ["release 2.22.2 run (capture stub 352153e0, 2026-08-26): opencode_plugin_contracts red locally, green in CI", "packages/bee-rs/crates/bee/tests/opencode_plugin_contracts.rs (resolve_opencode_binary)"]
  polarity: pitfall
  critical: true
  evidence: wired
  evidence_ref: "before trusting a red from a binary-deriving test, run `file $(command -v <tool>)` — a shell script or a shim means the test read wrapper bytes, not the tool; fix the PATH, not the test"
---

# A wrapper on PATH is not the binary a derivation reads

The release test gate went red on this machine while CI stayed green. Two
causes, one shape: `command -v` answered with something that was not the
tool.

- `node` resolved to a version-manager shim with no version set for it —
  the shim exists, executes, and fails, so "node not found on PATH" was
  the honest but confusing symptom.
- `opencode` resolved to a 121-byte bash wrapper that delegates to the
  version manager. The contract test reads the resolved file's own bytes
  to derive the tool registry from its bundled payload — and read the
  wrapper's 121 bytes instead, reporting "the tool-id Set literal was not
  found" as though the tool had changed shape.

The derivation was right to refuse rather than report zero tools. The
lesson is on the reader's side: a test that derives facts from an
installed binary must either see through wrappers and shims or fail with
"this is a script, not the binary". Until it does, the environment must
put the real binary's directory on PATH ahead of the wrappers — the
2.22.2 release run did exactly that (the version manager's real install
directories, prepended before the release script).

The same release surfaced a sibling one-line rule for docs: the shipped
command-spelling guard reads fenced lines one at a time, so a
backslash-continued `bee` invocation shows the guard only its first line
and reads as missing its required flags. Until the extractor joins
continuations (filed in the backlog), a fenced bee invocation stays on
one line.
