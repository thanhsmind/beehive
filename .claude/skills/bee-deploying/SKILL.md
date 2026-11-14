---
name: bee-deploying
description: "Learn, record and replay how one project deploys or releases: find the project's deploy record and run it, stopping before every step that touches a live environment, or, with no record yet, read the CI, container, package and platform files, ask only the gaps, walk one deploy with the user and write docs/DEPLOY.md, scripts/deploy.sh and a pointer in CLAUDE.md or AGENTS.md. Use when the user says deploy, release, ship, or asks to record how this project deploys. Not for the bee repo's own release, which scripts/release.sh owns, and not for a server a project-specific deploy skill already owns."
metadata:
  skillstation_alias: bee-deploying
---
This skill lives in the Skillstation hub.
Call the skillstation MCP tool `skill_get` with skill_id `bee-deploying`.
Follow the returned content as this skill.
If the tool is missing, tell the user to run `skillstation connect -g`.
