---
name: bee-verifying
description: "Generate a project-local `verify-app` skill that drives the real product the way a user does — any language, framework, or platform — and prove it by running it end to end. Use ONCE per repo, when the repo has no scripted way to prove UI, CLI, TUI, service, desktop or API behavior; when onboarding reports no declared test command and offers verification; or when the user asks for a control skill, a drive harness, or \"a way to actually run this app\" for this repo. Not the periodic audit of a verification skill that already exists — that is bee-verify-upkeep."
disable-model-invocation: true
metadata:
  skillstation_alias: bee-verifying
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-verifying`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
