---
name: statechart
description: "Check a TLA+ model of a system that moves between states, such as a product flow, a protocol, or a service, and draw it as a Mermaid statechart. The report gives the shortest path to each state that breaks a rule. Use for /statechart, or to show a bug as a path through states."
disable-model-invocation: true
metadata:
  skillstation_alias: statechart
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `statechart`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
