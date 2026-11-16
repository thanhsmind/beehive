---
name: product-description
description: "Build a \"product description\" repo for a software product — a set of prose documents describing, from the outside in, what the user sees, what they can do, and exactly what happens when they do it, written from the code and tests, then verified against the running product and triaged into a bug list. Works for any product with a user (canvas editors, web apps, CLIs, chat products, mobile apps). Use when the user asks to \"write a product description for X\", \"describe the user experience of X\", \"document how X behaves for the user\", \"make a behavior-spec repo\", or wants a feature-by-feature, event-by-event account of an app's behavior rather than API docs. Also use to resume or extend an existing product description repo."
metadata:
  skillstation_alias: product-description
---
This skill lives in the Skillstation hub.
Load it with the CLI first: run `skillstation skill show product-description --json` and follow its content field as this skill when manifest.status is active.
Call the skillstation MCP tool `skill_get` with skill_id `product-description` instead when that content sends you to a relative file of the skill, when the status is not active, or when the CLI is missing or fails.
If neither works, tell the user to run `skillstation connect -g`.
