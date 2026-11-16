---
name: bee-verify-upkeep
description: "Periodic audit of an existing project-local verify-app skill and its feature map. Use when a repo already has a drive harness and it may have drifted: after a release, when drive recipes stop matching the app, or on request to audit it. Not for creating one (bee-verifying)."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-verify-upkeep
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-verify-upkeep`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
