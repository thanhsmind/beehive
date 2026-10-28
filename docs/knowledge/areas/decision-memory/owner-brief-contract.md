---
type: bee.area
title: Decision Memory — the owner brief contract (docs/briefs/<id>.md)
description: "The one home of the owner brief: the file every live backlog item and active feature carries so the owner can decide on it, its frontmatter and six labelled paragraphs, what the body must never hold, who writes it and when, how bee checks it, and the folding rule every consumer applies."
timestamp: 2026-10-28
bee:
  id: decision-memory-owner-brief-contract
  lifecycle: active
  areas: [decision-memory]
  required_context: [areas/decision-memory/backlog-store.md]
  decisions: [owner-brief D1 35853825 (contract), "D2-D12 of docs/history/owner-brief/CONTEXT.md (forbidden content, language, the add door, live and active, folding, the check, the doctor note, the ledger, Lock and deferral, examples)"]
  sources: [docs/history/owner-brief/CONTEXT.md, docs/history/owner-brief/spec.md, docs/briefs/owner-brief.md]
  authoritative_for: "decision-memory: the owner brief contract"
---

# Decision Memory — The Owner Brief Contract

## Purpose

The owner reads every bee project's work on an external board, and decides from what the board
shows. A backlog item that carries only a title and a one-line `cos` gives the owner no reason,
no cost and no choice, so the owner cannot approve it or act on it. The why lives in the
deferring feature's `CONTEXT.md` and `plan.md`, which the owner never reads.

An owner brief closes that gap. It is written once, at creation, by the agent that holds the
context, in the owner's language. Consumers such as waggledance (`/api work-items`) and
agent-works (`mirror.mjs`) parse it byte for byte, so the format below never changes shape.

## Data Dictionary

- **Path** — `docs/briefs/<id>.md`, where `<id>` is the PBI id (`p-<8hex>` or a legacy `P<n>`)
  or the feature slug.
- **Frontmatter** — flat `key: value` lines between two `---` lines, with no nesting and no
  leading whitespace. Four keys are required; other keys are allowed:

  | Key | Values |
  |-----|--------|
  | `title` | Non-empty, at most 80 characters, in the owner language, states the outcome. |
  | `audience` | `owner` or `internal`. |
  | `size` | `S`, `M` or `L`. |
  | `ask` | `build`, `park`, `drop`, `answer` or `test`. |

  Keys and enum values stay English in every owner language.
- **Owner language** — `brief.language` in `.bee/config.json`: `vi` (the default when the key is
  absent) or `en`. An unknown value validates with the `vi` labels, and the check names the
  unknown value.
- **Body** — everything after the closing `---`: exactly six paragraphs, in this order, each
  opening with its bold label on its own line start:

  | Paragraph | `vi` label | `en` label | Content |
  |-----------|-----------|-----------|---------|
  | 1 | `**Là gì:**` | `**What:**` | What happens today, and what happens after the work. |
  | 2 | `**Vì sao:**` | `**Why:**` | The pain it removes, and who feels it. |
  | 3 | `**Nguồn gốc:**` | `**Origin:**` | One line: where the work came from. |
  | 4 | `**Chi phí và rủi ro:**` | `**Cost and risk:**` | The size, one or two risks, and what it depends on. |
  | 5 | `**Anh cần làm:**` | `**Your call:**` | Two or three options, plus one recommendation with a one-line reason. |
  | 6 | `**Chuẩn xong:**` | `**Done when:**` | Two to four observable bullets, each a line that starts with `- `. |

  Each paragraph holds text after its label. No other body line starts with `**`.
- **Word count** — the body holds 120 to 220 words, both ends included. A word is a
  whitespace-separated body token with at least one letter or digit: the labels count, a bare
  `-` bullet marker does not.

## Business Rules

- **The body is for the owner, never for the code.** It holds no ids, no file paths and no
  function names; consumers add their own technical footer. The check refuses each of these
  and names the token it found:
  - a PBI-id token: `p-` followed by 8 lowercase hex characters;
  - a token that holds `/`, after stripping surrounding punctuation;
  - a token ending in `.rs`, `.md`, `.json`, `.js`, `.mjs`, `.ts`, `.py`, `.sh`, `.toml`,
    `.yml` or `.yaml`;
  - any backtick;
  - a call shape: a run of letters or underscores followed at once by `(`.
- **The adding agent writes the brief.** It drafts the brief before the add, then passes it to
  `bee backlog pbi add --brief-file <draft>` or `bee backlog propose --brief-file <draft>`. The
  door validates the draft against this contract. A valid draft is copied unchanged to
  `docs/briefs/<id>.md` under the new id. A malformed draft refuses the add, names every defect,
  and writes nothing: no PBI event and no brief.
- **No brief is recorded and flagged, never silent.** With no `--brief-file`, the PBI is still
  added. The result carries `brief: "missing"` in JSON, plus a warning line that names
  `docs/briefs/<id>.md` and `bee backlog briefs`.
- **bee-shaping's Lock writes the feature brief.** Lock writes `docs/briefs/<feature>.md` from
  the Feature Boundary and the Locked Decisions, in the owner language, then checks it with
  `bee backlog briefs --id <feature>`.
- **A deferral says why and what waiting costs.** When shaping, planning or capturing moves
  product scope out into the backlog, the agent adds the PBI with `--brief-file`. The cost
  paragraph (`Chi phí và rủi ro` / `Cost and risk`) states why the scope moved out and what it
  costs to wait. Friction and findings keep the plain `bee backlog add`.
- **A live item** is a PBI with status `proposed` or `in-flight`. `parked`, `done` and `declined`
  need no brief. bee has no `ready` status.
- **An active feature** is a feature whose pipeline record — the default `.bee/state.json` or a
  `.bee/lanes/<feature>.json` lane — has a phase other than `idle` and `compounding-complete`.

## Folding (for every consumer)

Show one card per piece of work. A PBI with status `in-flight` whose `feature` names an active
feature belongs inside that feature's card, never as its own card, and needs no brief of its
own: the feature's brief covers it. A `proposed` PBI that names the same feature stays its own
card. bee applies this rule in `bee backlog briefs`; waggledance, agent-works and any other
consumer that lists bee work for the owner apply the same rule.

## Behaviors & Operations

- **`bee backlog briefs [--id <id>] [--json]`** — the owner view and the check. With no `--id`,
  it lists the active features, each with its folded PBIs nested, then the unfolded live PBIs.
  Each entry carries its brief status, `ok`, `missing` or `malformed`, plus the defect list. It
  exits 1 when any listed brief is missing or malformed. With `--id <id>`, it checks the one
  brief at `docs/briefs/<id>.md` and exits 1 when that brief is missing or malformed.
- **`bee doctor`** — an advisory `owner_briefs` note row, never a voting row. It counts the live
  items and active features without a valid brief and points at `bee backlog briefs`. A brief
  gap never changes doctor's overall verdict.
- **`docs/backlog.md`** (`bee backlog render`) stays the internal ledger, unchanged. The owner
  view is `bee backlog briefs`. The unified store behind both is
  `docs/knowledge/areas/decision-memory/backlog-store.md`.

## Examples

### Good: this feature's own brief

It states the change from today, the owner's pain, the size and the risks, a recommendation
with its reason, and four results the owner can see.

```markdown
---
title: Mỗi việc và mỗi tính năng có bản tóm tắt đủ để anh duyệt
audience: owner
size: M
ask: build
---

**Là gì:** Hiện nay một việc chỉ có tên và một câu ngắn. Sau thay đổi này, mỗi việc mới và mỗi tính năng mới có một bản tóm tắt sáu đoạn bằng tiếng Việt, viết ngay lúc tạo. Công cụ báo khi bản tóm tắt thiếu hoặc sai mẫu.

**Vì sao:** Anh không hiểu một câu quá ngắn, nên không thể duyệt hay chọn hướng. Lý do và cái giá của việc nằm trong tài liệu nội bộ.

**Nguồn gốc:** Yêu cầu từ waggledance ngày 28 tháng 10.

**Chi phí và rủi ro:** Cỡ vừa. Rủi ro: người viết bỏ qua bước tóm tắt, nên công cụ báo việc thiếu. Đổi mẫu có thể làm hỏng bảng của agent-works, nên mẫu giữ nguyên. Phụ thuộc mẫu đã thống nhất với waggledance.

**Anh cần làm:** Chọn duyệt để xây, hoãn, hoặc bỏ. Tôi đề nghị duyệt, vì các dự án khác chờ bản này để bổ sung tóm tắt.

**Chuẩn xong:**
- Việc mới có tóm tắt sai mẫu bị từ chối.
- Việc mới không có tóm tắt vẫn vào danh sách nhưng bị gắn cờ.
- Bước kiểm tra liệt kê mọi việc đang mở còn thiếu tóm tắt.
- Việc đang làm của một tính năng hiện trong tính năng đó.
```

### Bad: a one-line CoS in place of a brief

This is the whole text an agent-works item gave the owner:

```text
A comment by a person on a mirrored task reaches the mapped project's leader as a dispatch; deferred from wd-mirror shaping
```

It fails the owner in five ways:

- No why: it never says whose pain the work removes.
- No cost: it gives no size, no risk and no dependency.
- No reason for the deferral: "deferred" with nothing about why the scope moved out or what
  waiting costs.
- No ask: the owner cannot tell whether to build, park, drop or answer.
- An internal slug: `wd-mirror` names a feature the owner has never seen.

## Pointers (implementation)

- Validator, owner language, brief status and the owner view with folding:
  `packages/bee-rs/crates/bee/src/verbs/brief.rs`.
- The `--brief-file` door on `pbi add` and `propose`, and the `backlog briefs` verb:
  `packages/bee-rs/crates/bee/src/verbs/backlog.rs`.
- The `owner_briefs` advisory note row: `packages/bee-rs/crates/bee/src/doctor.rs`.
- Skills that teach the writing moments: `skills/bee-shaping/SKILL.md` (Lock, Explore deferral),
  `skills/bee-planning/SKILL.md` (Scope integrity), `skills/bee-capturing/SKILL.md` (deferral
  bullet).
