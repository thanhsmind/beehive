# CREATION-LOG — bee-bypass-gate

## Origin

Owner request (2026-07-09): an opt-in mode, invoked as `bee-bypass-gate`, where bee "luôn lựa chọn theo agent suggest và không cần phải human chọn." Added under the decision 0002 gate via **decision 0010**, which names the workflow gap: a trust/speed autopilot for low-risk repos that the "four gates never self-approved" invariant did not allow.

## Scope decisions (owner-confirmed)

- **Q1 — how far does bypass reach?** → *Chừa high-risk/hard-gate.* Auto-approve Gates 1-3 for `tiny`/`small`/`standard`; `high-risk` lane and hard-gate flags (auth, authorization, data loss, audit/security, external provider, validation removal, migration/schema) always stop for the human.
- **Q2 — secrets & UAT?** → *Giữ hỏi human.* Secret-file reads always need approval; Gate 4 UAT items always go to the human; only the merge auto-approves and only when P1 = 0 and all UAT passed.

## Design

- One persistent per-repo switch: `.bee/config.json` `gate_bypass` (default `false`). The skill only flips it (`on`/`off`/`status`); the behavioral rule lives once in the Gate Presentation Contract (`bee-hive/references/routing-and-contracts.md`), which every gate point already references.
- Mechanical guards unchanged: `claimCell` + write-guard still require `approved_gates.execution: true`. Bypass = the agent records that approval itself for eligible work, with a logged audit decision and a `⚡ auto-approved` chat notice. No second enforcement path.
- Not headless: headless still stops at every gate; the two switches are independent.
- Always visible: `inject.mjs` preamble and `bee_status` both print a loud `GATE BYPASS ON` line while active.

## Discipline / debt

The safety floor is prose-enforced (the agent must check lane/flags before self-approving). RED-baseline pressure-testing of the floor on a real high-risk feature across runtimes/tiers is recorded debt before 1.0 — the failure mode to test is a bypass that leaks past its floor onto a hard-gate change. Same debt class as the v0.1 skills (decision 0010, Consequences).

## Version

Shipped in bee 0.1.6. Skill catalog row #13.
