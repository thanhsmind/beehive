---
name: statechart
description: "Check a TLA+ model of a system that moves between states, such as a product flow, a protocol, or a service, and draw it as a Mermaid statechart. The report gives the shortest path to each state that breaks a rule. Use for /statechart, or to show a bug as a path through states."
disable-model-invocation: true
metadata:
  skillstation_alias: statechart
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show statechart --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `statechart` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
