---
type: bee.pattern
title: "Agent-runtime discovery paths are version-moving targets — probe the binary, not memory"
description: "Probe the binary, not memory"
tags: [process, codex, claude-code, skills, discovery]
timestamp: 2026-07-14
bee:
  id: pattern-20260714-agent-runtime-discovery-paths-are-version-moving-targets
  lifecycle: active
  sources: ["docs/history/learnings/critical-patterns.md#PAT34", "original feature: installer-hardening"]
  polarity: practice
  critical: false
---

# Agent-runtime discovery paths are version-moving targets — probe the binary, not memory

Codex's repo-level skill path is `.agents/skills` (cwd → repo root; `~/.codex/skills` is
legacy-global), Claude Code's is `.claude/skills` — neither reads the other's dir, so a
per-project install must materialize BOTH trees. Verified empirically with
`codex debug prompt-input` (renders the exact skill roots table the model sees) rather
than from docs memory; that command is the ground truth for "does the agent see skill X".
