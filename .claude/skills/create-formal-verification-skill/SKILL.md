---
name: create-formal-verification-skill
description: "Generate a project-local skill that formally verifies the code where a bug costs the most and tests reach the least, in any language, with whichever verification tool fits. Use for /create-formal-verification-skill, \"model-check this race\", \"prove this invariant\", or when concurrency, retries, state machines, parsers, or unsafe code need more than tests."
disable-model-invocation: true
metadata:
  skillstation_alias: create-formal-verification-skill
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `create-formal-verification-skill`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
