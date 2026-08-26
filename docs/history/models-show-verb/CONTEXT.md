# models-show-verb — locked context

Feature: PBI p-0a5e6c44 — one bee verb returns the whole models role table
with descriptions, and every role-choosing door reminds the agent to read
it before assigning.

## Locked decisions

- **D1 (user, 2026-08-26):** Reading the role table is a bee VERB, never
  hand-written config parsing by an agent: "lấy thông tin models từ config
  nên là 1 verb trong bee để nhận trọn bộ không cần phải viết code đọc."
  The verb returns the RAW `models.<runtime>` table — every slot shape
  verbatim, `description` intact (the normalized view strips it by design,
  and that strip is what keeps resolution blind; the verb reads raw).
- **D2 (user, 2026-08-26):** The doors where an agent picks a role remind
  it to read first: "nhắc là nếu chưa đọc thì nên đọc" — the missing-role
  refusal and `bee cells add --help` name the verb; guess-and-fill is the
  defect being replaced.
- **D3 (user, 2026-08-26, PBI CoS 3):** The onboarding seed
  (`default_config`) carries a `description` on each role bee publishes
  (`code`, `read`, `extraction`, `generation`) — a fresh install ships the
  self-teaching table. Codex seeds stay null.
- **D4 (PBI CoS 2):** `bee status --json` keeps descriptions in its models
  section — merged onto the normalized slots for display; internal
  resolution keeps using the normalized (stripped) map.
- **D5 (PBI CoS 5/6):** The session-preamble roles line stays the
  session-start summary (it already carries descriptions); resolution, the
  model guard, and `dispatch prepare` are untouched.

## Naming

`bee models show` — new `models` group, one verb, read-only, `--runtime`
filter (default: all runtimes), `--json` like every other read verb.
