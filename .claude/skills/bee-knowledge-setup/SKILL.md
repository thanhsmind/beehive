---
name: bee-knowledge-setup
description: "Build a project's `docs/knowledge/` bundle from the tree: one area concept per real subsystem, the orientation file, seeded anchors, and a strict check that proves it. Use ONCE per repo, when the repo has no `docs/knowledge/` bundle and the user asks to set up, create, bootstrap or start the knowledge base, or when onboarding finds none. Not for the periodic audit of a bundle that already exists — that is bee-knowledge-upkeep — and not for the in-flow write of one settled fact, which stays with bee-capturing."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-knowledge-setup
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-knowledge-setup`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
