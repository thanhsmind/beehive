# Kết luận

**Đúng: Bee hiện chưa chạy trên Codex tương đương Claude Code.** Nhưng nguyên nhân chính không phải Codex “kém hiểu prompt” hơn. Core của Bee—`bee`, gates, cells, reservations, verify evidence, handoff—khá tốt và có thể dùng chung. Khoảng cách nằm ở **lớp tích hợp runtime**: Bee vẫn là kiến trúc Claude-first, sau đó thêm các nhánh tương thích Codex vào cùng bộ skill.

Tôi khuyên:

> **Không fork toàn bộ 15 skill thành hai bộ độc lập.**
> Hãy giữ một Bee Core, nhưng tách rõ `Claude adapter` và `Codex adapter`, rồi generate các skill dành riêng cho từng runtime.

Codex hiện có hooks, project-scoped custom agents, model riêng cho từng agent, sandbox riêng và `developer_instructions`. Bee vẫn đang xem Codex như runtime không có các khả năng này, nên tự làm yếu chính nó. ([OpenAI Developers][1])

---

# Vì sao Bee chạy tốt với Claude nhưng hụt trên Codex?

## 1. Bee đang có nhiều “sự thật” mâu thuẫn về Codex

Trong `INSTALL.md`, Bee vẫn khẳng định:

> “Codex has no lifecycle hooks”

và chỉ hướng dẫn Codex dựa trên `AGENTS.md` cùng `bee`.

Tài liệu kiến trúc `docs/06-runtime-integration.md` cũng tiếp tục thiết kế Codex như runtime không có hooks và chấp nhận khoảng trống mechanical enforcement.

Nhưng repo hiện tại lại đã có `.codex/hooks.json` với `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `SubagentStop`, `PreCompact` và `Stop`.

Codex hiện cũng chính thức hỗ trợ các sự kiện này và đọc hook từ `<repo>/.codex/hooks.json`. ([OpenAI Developers][2])

Điều này làm Bee có ít nhất ba mô hình khác nhau:

1. Tài liệu cài đặt: Codex không có hooks.
2. Tài liệu kiến trúc: Codex chỉ dựa vào prompt và helper.
3. Code hiện tại: Codex đã có hooks.

Khi installer, skill, host project hoặc người dùng dựa vào những phiên bản khác nhau, hành vi không thể ổn định.

---

## 2. Khả năng rất cao các Codex hooks chưa được trust

Codex chỉ chạy project hooks khi:

* Project `.codex/` được trust.
* Người dùng đã review và trust đúng hash hiện tại của từng hook.
* Hook thay đổi thì có thể cần review lại.

Codex cung cấp `/hooks` để kiểm tra trạng thái này. Hook chưa được trust sẽ bị bỏ qua. ([OpenAI Developers][2])

Đây có thể là một nguyên nhân thực tế rất lớn: file `.codex/hooks.json` tồn tại nên Bee tưởng hook đang hoạt động, nhưng Codex có thể đang chạy hoàn toàn bằng prompt.

Hiện quy trình verify Codex trong `INSTALL.md` chỉ yêu cầu mở session và hy vọng agent tự đọc `AGENTS.md`; không có bước `/hooks`, không kiểm tra hook đã chạy, cũng không kiểm tra project trust.

Vì vậy Bee cần phân biệt rõ ba trạng thái:

```text
hooks_file_present
hooks_discovered
hooks_trusted_and_observed
```

Có file chưa có nghĩa là hook đang chạy.

---

## 3. Bee chưa dùng custom agents native của Codex

`.codex/config.toml` hiện chỉ có:

```toml
approval_policy = "never"
```

Không có `[agents]`, không có giới hạn swarm, không có project-scoped agent definitions.

Trong khi đó, Codex hiện hỗ trợ custom agents đặt tại:

```text
.codex/agents/*.toml
```

Mỗi agent có thể có:

* `name`
* `description`
* `developer_instructions`
* `model`
* `model_reasoning_effort`
* `sandbox_mode`
* MCP riêng
* cấu hình skill riêng

Codex cũng có sẵn các role `default`, `worker`, `explorer`; project có thể tạo thêm các role riêng. ([OpenAI Developers][1])

Nhưng `bee-swarming` vẫn ghi:

> “Codex has no per-agent subagent type equivalent”

và vì vậy chỉ dùng “read budget + output cap” trong prompt.

Câu này hiện không còn chính xác. Codex không có đúng tham số `subagent_type` giống Claude, nhưng nó đã có **functional equivalent mạnh hơn** thông qua `.codex/agents/*.toml`.

Bee nên có tối thiểu:

```text
.codex/agents/
  bee-explorer.toml
  bee-validator.toml
  bee-worker.toml
  bee-reviewer.toml
```

Trong đó:

* `bee-explorer`: read-only, model nhanh, chỉ trả evidence digest.
* `bee-validator`: read-only, kiểm tra feasibility và cell contracts.
* `bee-worker`: workspace-write, chỉ một cell, reserve → implement → verify → cap.
* `bee-reviewer`: read-only, correctness/security/tests/architecture.

---

## 4. Critical worker contract đang nằm ở tầng prompt quá yếu

Codex xây prompt theo thứ tự ưu tiên:

```text
system
developer
user
assistant
```

`AGENTS.md` và metadata của skills nằm trong nhóm user instructions. Trong khi `developer_instructions` của custom agents nằm ở tầng developer—ưu tiên cao hơn. ([OpenAI][3])

Hiện các luật quan trọng như:

* Chỉ nhận đúng một cell.
* Không tự chọn việc.
* Phải reserve trước khi ghi.
* Không cài package.
* Phải verify và cap.
* Chỉ trả một status token.

đều chủ yếu nằm trong `bee-executing/SKILL.md`.

Codex dùng progressive disclosure: ban đầu nó chỉ nhận name, description và path của skill; toàn bộ `SKILL.md` chỉ được nạp khi model quyết định dùng skill. ([OpenAI Developers][4])

Vì vậy worker contract không nên chỉ phụ thuộc vào câu:

> “Load the bee-executing skill immediately.”

Nó phải được đặt trực tiếp trong `developer_instructions` của `bee-worker.toml`. Skill vẫn chứa quy trình chi tiết, nhưng custom agent contract phải chứa các invariant không được phép quên.

---

## 5. Cùng một skill đang được copy nguyên trạng cho cả hai runtime

`skills/bee-swarming/SKILL.md`, `.agents/skills/bee-swarming/SKILL.md` và `.claude/skills/bee-swarming/SKILL.md` đang có cùng SHA.

Điều này giải thích tại sao bên trong một skill có cả:

* Claude `Agent` tool.
* `.claude/agents/bee-*.md`.
* `subagent_type`.
* Codex `spawn_agent`.
* `wait_agent`.
* external CLI Codex.
* advisor fallback bằng `claude -p`.
* các ngoại lệ AO10/AO11/AO12.

Khi một skill vừa là doctrine, vừa là runtime driver, vừa là compatibility matrix, model phải đọc rất nhiều nhánh không liên quan rồi tự chọn đúng nhánh. Claude được hỗ trợ bằng hook và agent type nên thường chọn đúng. Codex phải tự suy ra nhiều hơn.

Issue #7 và #8 trước đây kết luận không có “prose Codex riêng” và xử lý triệu chứng bằng refresh skill.

Nhưng ở góc độ kiến trúc, **không có prose Codex riêng lại chính là vấn đề**: hai runtime có primitive khác nhau nhưng đang dùng nguyên cùng một tài liệu điều khiển.

---

## 6. Codex hook mapping vẫn còn dấu vết Claude

Trong `.codex/hooks.json`, state sync đang match:

```text
TaskCreate|TaskUpdate|TodoWrite
```

Đây là tên tool theo hệ Claude. Plan tool native của Codex là `update_plan`. ([OpenAI][3])

Do đó state-sync có thể không chạy sau các thao tác planning native của Codex.

Ngược lại, hook Codex cho phép match:

* `update_plan`
* `Agent` cho `spawn_agent`
* `Bash`
* `apply_patch`, hoặc alias `Edit|Write`
* MCP tool names

([OpenAI Developers][2])

Trong Claude config, Bee có riêng `bee-model-guard` cho `Agent|Task`.

Nhưng `.codex/hooks.json` chưa có guard tương đương cho `Agent`. Vì thế Codex có thể spawn một generic agent dù Bee muốn role hoặc tier cụ thể.

---

## 7. Codex plugin hiện chỉ đóng gói skills, không đóng gói hooks

`.codex-plugin/plugin.json` hiện chỉ khai báo:

```json
"skills": "./skills/"
```

không có `hooks`.

Codex plugin chính thức có thể bundle `hooks/hooks.json` hoặc chỉ định hook path trong manifest. ([OpenAI Developers][5])

Do đó hiện tại:

* Claude plugin route mang theo automation skeleton.
* Codex plugin route chủ yếu mang theo skills.
* Mechanical parity của Codex phụ thuộc vào onboarding repo có tạo `.codex/hooks.json` đúng hay không.

Đây chưa phải “first-class runtime parity”.

Cần lưu ý: nếu vừa bundle plugin hooks vừa cài project hooks, Codex sẽ chạy tất cả matching hooks; các nguồn không override nhau. ([OpenAI Developers][2])

Vì vậy installer phải chọn rõ:

```text
plugin hooks
hoặc
repo-local hooks
```

không cài cả hai một cách mù quáng.

---

## 8. `AGENTS.md` đang gánh quá nhiều nhiệm vụ

`AGENTS.md` hiện chứa:

* startup protocol
* handoff semantics
* baseline gate
* routing
* gates
* 15 critical rules
* delegation rubric
* Codex waiting protocol
* hook-equivalent behavior
* multi-session coordination
* session finish

Codex chỉ đọc tối đa tổng cộng 32 KiB project instruction theo mặc định; các `AGENTS.md` từ root tới current directory cùng chia sẻ giới hạn đó. ([OpenAI Developers][6])

Tôi chưa khẳng định file hiện tại đã vượt ngưỡng, nhưng nó đang quá gần vai trò của một full playbook. Điều này gây hai rủi ro:

1. Instruction dilution: luật nào cũng được viết như critical nên không còn luật nào thật sự nổi bật.
2. Các repo host có thêm `AGENTS.md` ở thư mục con dễ chạm ngưỡng tổng.

Root `AGENTS.md` nên là **kernel**, không phải toàn bộ operating manual.

---

# Có nên tách bộ skill không?

## Có, nhưng chỉ tách phần runtime-sensitive

Không nên tạo:

```text
skills-claude/15 skills
skills-codex/15 skills
```

và sửa tay hai bên. Cách đó sẽ tạo drift rất nhanh.

Nên dùng:

```text
skill-src/
  shared/
    bee-hive.core.md
    bee-swarming.core.md
    bee-executing.core.md
    ...

  adapters/
    claude/
      bootstrap.md
      dispatch.md
      worker.md
      advisor.md
      hooks.md

    codex/
      bootstrap.md
      dispatch.md
      worker.md
      advisor.md
      hooks.md

build/
  render-skills.mjs

dist/
  claude/skills/bee-*/
  codex/skills/bee-*/
```

Build script ghép:

```text
shared skill contract
+ runtime adapter
= runtime-specific SKILL.md
```

Sau đó test phải đảm bảo phần shared có cùng semantic hash, còn phần adapter được phép khác.

## Chỉ khoảng năm skill cần adapter mạnh

### Phải tách runtime adapter

1. `bee-hive`
2. `bee-swarming`
3. `bee-executing`
4. `bee-validating`
5. `bee-reviewing`

Đây là những skill liên quan trực tiếp đến:

* bootstrap
* agent spawn
* model selection
* wait/follow-up
* advisor
* parallel review
* hooks

### Có thể tiếp tục dùng chung gần như nguyên trạng

* `bee-exploring`
* `bee-planning`
* `bee-briefing`
* `bee-scribing`
* `bee-compounding`
* `bee-grooming`
* `bee-xia`
* `bee-bypass-gate`
* `bee-writing-skills`
* `bee-evolving`

Những skill này chủ yếu là workflow semantics và artifact contracts.

---

# Có nên tách prompt guide không?

**Có, nhưng không tạo thêm một “Codex manual” khổng lồ.**

Nên có ba tầng:

## Tầng 1 — `AGENTS.md` kernel

Chỉ giữ khoảng 60–100 dòng:

```text
1. Bee active: run bee status.
2. Route through bee-hive before source changes.
3. No source write before execution approval.
4. One worker owns one cell.
5. Verification evidence is mandatory.
6. Never edit .bee state manually.
7. Load the selected skill.
8. Runtime-specific behavior comes from the adapter.
```

## Tầng 2 — Skills

Chứa workflow từng phase và anti-rationalization rules.

## Tầng 3 — Runtime contracts

Claude:

```text
.claude/agents/
hooks/hooks.json
runtime-claude.md
```

Codex:

```text
.codex/agents/*.toml
.codex/hooks.json hoặc plugin hooks
runtime-codex.md
```

Critical worker invariants của Codex phải được đặt trong custom-agent `developer_instructions`, không chỉ nằm trong skill.

---

# Kiến trúc Codex-native tôi đề nghị

```text
Main Codex session
  └── bee-hive
        ├── exploring/planning: main session
        ├── bee_explorer: read-only custom agent
        ├── bee_validator: read-only custom agent
        ├── bee_worker: workspace-write custom agent
        └── bee_reviewer: read-only custom agent
```

## `.codex/config.toml`

```toml
[agents]
max_threads = 6
max_depth = 1
interrupt_message = true
```

Không nên hard-code `approval_policy = "never"` trong bản phân phối mặc định. Permission policy của Codex và Gate bypass của Bee là hai khái niệm khác nhau. Subagents kế thừa permission mode của parent, nên đặt `never` mặc định có thể tạo hành vi khó hiểu hoặc làm các action cần approval thất bại thay vì hỏi. ([OpenAI Developers][1])

Bee có thể cung cấp các profile riêng:

```text
bee-safe       -> on-request
bee-autopilot  -> never
```

nhưng không nên đánh đồng chúng với `gate_bypass`.

## `.codex/agents/bee-worker.toml`

Nội dung cốt lõi nên giống:

```toml
name = "bee_worker"
description = "Execute exactly one assigned Bee cell."
sandbox_mode = "workspace-write"

developer_instructions = """
You execute exactly one parent-assigned Bee cell.

Never select another cell.
Read AGENTS.md, CONTEXT.md, plan.md, and the assigned cell.
Reserve every declared file before writing.
Do not install packages or redesign architecture.
Run the exact verify command.
Record its real output.
Cap only after verification passes.
Release reservations before returning.
Return exactly one status: DONE, BLOCKED, HANDOFF, or NOOP.
"""
```

Chi tiết command tiếp tục nằm trong `bee-executing`.

## Codex hooks nên đảm nhiệm

* `SessionStart`: inject state và bắt buộc route qua Bee ở developer context.
* `UserPromptSubmit`: reminder ngắn, có dedupe.
* `PreToolUse Bash|Edit|Write`: gate, reservation, privacy.
* `PreToolUse Agent`: chỉ cho phép sanctioned Bee role khi đang swarm.
* `PostToolUse update_plan`: state sync.
* `SubagentStart`: inject assigned cell contract vào subagent.
* `SubagentStop`: collect/nudge.
* `PreCompact/PostCompact`: handoff and reload.
* `Stop`: reservation/cell hygiene.

Codex `SessionStart` và `SubagentStart` có thể inject thêm developer context, rất phù hợp để đưa các luật không thể quên vào đúng priority level. ([OpenAI Developers][2])

---

# Thứ tự triển khai

## P0 — Làm ngay

### 1. Xóa toàn bộ sự thật lỗi thời

Sửa đồng bộ:

* `INSTALL.md`
* `docs/06-runtime-integration.md`
* `README.md`
* `bee-swarming`
* `swarming-reference`
* Codex capability matrix

Không còn câu “Codex has no lifecycle hooks” hoặc “Codex has no role equivalent”.

### 2. Tạo `bee doctor --runtime codex`

Doctor phải báo:

```text
Codex version
project .codex trusted?
hooks file present?
hooks observed this session?
hooks pending review?
skills discovered?
custom agents discovered?
active permission mode?
duplicate plugin/project hooks?
```

Không được báo “Codex ready” chỉ vì file tồn tại.

### 3. Sửa hook matcher

Ít nhất:

```text
TaskCreate|TaskUpdate|TodoWrite
→ update_plan
```

và thêm guard cho:

```text
Agent
```

### 4. Bỏ `approval_policy = "never"` khỏi default package

Chuyển nó thành một optional autopilot profile.

### 5. Bundle Codex hooks đúng plugin

Thêm vào manifest:

```json
"hooks": "./hooks/hooks.json"
```

hoặc đặt đúng default path. Installer phải tránh cài thêm repo hook nếu plugin hook đã active.

---

## P1 — Codex-native orchestration

### 6. Tạo bốn custom agents

* explorer
* validator
* worker
* reviewer

### 7. Tách adapter cho năm skill nhạy runtime

Không còn các đoạn Claude/Codex xen kẽ trong cùng một execution path.

### 8. Chuyển worker invariants vào `developer_instructions`

Skill là hướng dẫn chi tiết; agent profile là luật cứng.

### 9. Viết lại advisor transport

Trong `bee-executing`, model-shaped advisor hiện hướng worker dùng “your own Agent tool”, fallback bằng `claude -p`.

Codex adapter phải dùng Codex custom agent/follow-up transport, không fallback sang Claude trừ khi người dùng cấu hình external Claude advisor rõ ràng.

---

## P2 — Đo parity thật, không đánh giá bằng cảm giác

Tạo black-box conformance suite chạy cùng fixture trên Claude và Codex:

1. Tiny typo: không ceremony thừa.
2. Standard feature: đúng Gates 1–3.
3. Write trước Gate 3: bị chặn.
4. Worker chỉ nhận một cell.
5. File reservation conflict: `[BLOCKED]`.
6. Verify red: không cap.
7. Package install: checkpoint.
8. Subagent timeout: không duplicate dispatch.
9. Compaction: handoff đúng.
10. Feature finish: không auto-review.
11. Review được gọi: fan-out đúng.
12. Hook chưa trust: doctor fail-closed.

Các chỉ số cần đo:

```text
route_accuracy
unauthorized_write_count
gate_correctness
delivery_tool_calls
time_to_first_edit
cell_completion_rate
verify_evidence_rate
duplicate_dispatch_count
unrequested_review_count
handoff_resume_success
```

Mục tiêu không phải Claude và Codex có transcript giống nhau. Mục tiêu là:

```text
100% invariant parity
gần tương đương delivery cost
runtime-native orchestration
```

---

# Quyết định cuối cùng

| Câu hỏi                                                        | Khuyến nghị                                 |
| -------------------------------------------------------------- | ------------------------------------------- |
| Có nên giữ chung Bee Core?                                     | **Có**                                      |
| Có nên copy toàn bộ skill thành hai bộ?                        | **Không**                                   |
| Có nên tách runtime adapter?                                   | **Có, bắt buộc**                            |
| Có nên có Codex prompt guide riêng?                            | **Có, nhưng mỏng và generated**             |
| Có nên dùng Codex custom agents?                               | **Có, đây là thay đổi quan trọng nhất**     |
| Có nên tiếp tục chỉ dùng read-budget prompt cho Codex workers? | **Không**                                   |
| Có nên dựa vào hooks là đủ?                                    | **Không; hooks + helpers + agent contract** |
| Có nên giữ `approval_policy="never"` mặc định?                 | **Không**                                   |

Đánh giá của tôi:

* **Bee Core:** khoảng 8/10.
* **Claude integration:** khoảng 8–9/10.
* **Codex integration hiện tại:** khoảng 5–6/10.
* Sau khi thêm native agents, hook trust doctor và runtime adapters: hoàn toàn có thể đạt **8–9/10 trên Codex**, nhưng không bằng cách tiếp tục bổ sung prose vào bộ skill chung.

Bước triển khai đúng nhất là mở một initiative duy nhất: **`codex-native-runtime-v2`**, gồm năm nhánh công việc P0/P1 ở trên; đừng tiếp tục vá từng biểu hiện riêng lẻ như “Codex review nhiều” hoặc “Codex không spawn đúng agent”.

[1]: https://learn.chatgpt.com/codex/agent-configuration/subagents "
  Subagents | ChatGPT Learn
"
[2]: https://learn.chatgpt.com/codex/hooks "
  Hooks | ChatGPT Learn
"
[3]: https://openai.com/index/unrolling-the-codex-agent-loop/ "Unrolling the Codex agent loop | OpenAI"
[4]: https://developers.openai.com/codex/skills/ "
  Build skills | ChatGPT Learn
"
[5]: https://learn.chatgpt.com/codex/build-plugins "
  Build plugins | ChatGPT Learn
"
[6]: https://learn.chatgpt.com/codex/agent-configuration/agents-md "
  Custom instructions with AGENTS.md | ChatGPT Learn
"
