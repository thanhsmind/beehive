---
type: bee.pattern
title: A test that runs the canonical source can never catch vendoring drift — only a fresh-install subprocess can
description: "A guard crashed fail-open on every fresh install for a whole release cycle: its sibling import was never vendored, node --check passes on missing imports, and every suite ran the canonical file where the import always resolves. The only detector was a live canary spawning the actually-vendored copy from a freshly-onboarded fixture."
tags: [vendoring, fresh-install, fail-open, canary, import-closure]
timestamp: 2026-07-24
bee:
  id: pattern-20260724-canonical-source-tests-cannot-see-vendoring-drift
  lifecycle: active
  sources: [i54-closeout-9 (tokenize-command.mjs missing from HOOK_FILENAMES; write guard ERR_MODULE_NOT_FOUND fail-open on fresh repo-hooks installs; caught only by canary P5), docs/history/i54-closeout/reports/validation-canary.md]
  polarity: pitfall
  critical: true
---

# A test that runs the canonical source can never catch vendoring drift — only a fresh-install subprocess can

When an install step copies files that import sibling files, three blind spots
stack: the failure lands inside a fail-open path (silently permissive, never
loud); syntax-only checks (node --check) exit 0 on a missing import; and every
suite that executes the CANONICAL source cannot reproduce the bug, because the
canonical sibling always resolves. The drift is only visible where it exists —
in the vendored copy — so the only honest detectors are (a) a static
import-closure check over the ship list itself, and (b) a subprocess spawn of
the actually-installed file inside a fresh-install fixture. Mechanized here as
scripts/tests/test_hook_vendor_closure.mjs.

**The tell:** a hand-maintained ship/allowlist next to files that import each
other, with green tests that all import from the source tree.
