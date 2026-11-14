---
name: bee-verify-upkeep
description: "Periodic audit that keeps a project-local `verify-app` skill and its feature map honest: parallel source readers per feature, one live session driving every feature, at most one worktree and one cell of proven corrections. Use PERIODICALLY, when a repo already carries a verification skill and its map may have drifted — after a release, after user-facing churn, when a drive recipe stops matching the app, or when the user asks to audit the verify skill. Not for creating one where none exists — that is bee-verifying."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-verify-upkeep
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-verify-upkeep`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
