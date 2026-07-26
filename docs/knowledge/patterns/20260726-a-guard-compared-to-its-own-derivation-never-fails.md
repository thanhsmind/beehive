---
type: bee.pattern
title: A guard comparing a value against its own derivation can never fail — anchor on independently derived evidence
description: "A proposed identity anchor compared realpath(PLUGIN_ROOT/packages/bee/scripts) with SCRIPTS_DIR — but PLUGIN_ROOT is computed from SCRIPTS_DIR, so the check is true by construction and the live guard it replaced would have become a no-op. The repaired anchor asserts facts from an independent derivation: the skills tree exists under the package root and the payload file is readable. Same family: one variable (HIVE_DIR) silently carried three semantics — engine geometry, a shared function's input, and the tree sync copies — and a wholesale rename would have pointed sync at the wrong tree."
tags: [guard, tautology, identity-anchor, refactor, semantics]
timestamp: 2026-07-26
bee:
  id: pattern-20260726-a-guard-compared-to-its-own-derivation-never-fails
  lifecycle: active
  areas: []
  required_context: []
  decisions: []
  sources: ["packages-engine-move (validation B5/C2: tautological identityOk re-anchor proposal; C1: HIVE_DIR three-semantics split)", docs/history/packages-engine-move/reports/validation-slice1.md, docs/history/learnings/20260726-packages-engine-move.md]
---

## The pattern

Two failure shapes from the same root — trusting derivation chains instead of independent evidence:

1. **Tautological guard.** If the expected side of a comparison is computed from the observed side (directly or through path arithmetic), the check passes forever and the guard is dead. Before writing any integrity anchor, trace both sides to their sources; they must be independent. A falsifiable anchor asserts external state: a tree exists, a file is readable, a containment holds.
2. **One name, several concepts.** A long-lived variable accumulates semantics (a location, a contract input, a sync root). Any refactor that moves what the name points at must first classify every use site by which concept it means — renaming or re-deriving wholesale silently repoints the concepts that should NOT have moved.
