---
name: bee-knowledge-upkeep
description: "Periodic audit that keeps a project's `docs/knowledge/` bundle honest: read the recorded signals, pick at most 5 concepts, one read-only source reader per concept, revise in place, one worktree and one cell of proven corrections. Use PERIODICALLY, when a repo already carries a knowledge bundle — after a release, when a loaded concept was reported wrong, when a concept grades orphan, or when a concept is loaded often and never used. Not for creating a bundle where none exists — that is bee-knowledge-setup — and not for the in-flow write of one settled fact, which stays with bee-capturing."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-upkeep
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-upkeep`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
