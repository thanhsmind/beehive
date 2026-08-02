# Model Presets — copy vào `.bee/config.json` là xong

Các bộ cấu hình `models` chuẩn (decisions 0012/0015/0019/0021). Chọn một preset, dán đè khối `"models"` trong `.bee/config.json` của repo. Quy tắc chung không đổi:

- **Orchestrator = model phiên** (`ceiling`, không cấu hình) — mở phiên bằng model nào thì model đó điều phối.
- **`review` mặc định opus** — model review không phải model đã implement; `null` = rơi về `generation`.
- Effort chỉ áp được ở runtime hỗ trợ chọn per-agent; executor ngoài mang effort trong chính command.
- Lệnh `wsl ...` dành cho Windows có Codex cài trong WSL (đã smoke-test 2026-07-10, codex-cli 0.143.0, prompt qua stdin OK).

## 1. `all-claude` — mặc định, chỉ có Claude subscription

Không cần sửa gì (đây là default từ 0.1.18). Ghi tường minh:

```json
"models": {
  "claude": { "extraction": "haiku", "generation": "sonnet", "review": "opus" }
}
```

## 2. `all-claude-tuned` — đúng bộ trong hình, thêm effort

Phiên mở bằng Fable. Review Opus xhigh, implementer Sonnet max:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": { "model": "sonnet", "effort": "max" },
    "review":     { "model": "opus",   "effort": "xhigh" }
  }
}
```

Lưu ý: `max` cho generation là đắt — cân nhắc `high` nếu đa số cell là wiring thường.

## 3. `gpt-adversarial-review` — Claude làm, GPT phản biện (WSL + Codex)

Review đi qua Codex CLI, read-only (reviewer không được sửa gì):

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": "sonnet",
    "review": {
      "kind": "cli",
      "command": "wsl bash -lc \"codex exec --skip-git-repo-check -s read-only -c model_reasoning_effort=xhigh -\""
    }
  }
}
```

Muốn ghim model phía GPT: thêm `-m <model-id>` trước dấu `-` cuối (dấu `-` = đọc prompt từ stdin, bắt buộc giữ). Chạy trong WSL trực tiếp thì bỏ phần `wsl bash -lc` và cặp nháy.

## 4. `codex-implements` — plan big (Claude), execute small (GPT)

Worker generation là Codex, cần quyền ghi workspace để reserve/implement/verify/cap qua `.bee/bin`; review vẫn là Opus độc lập:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": {
      "kind": "cli",
      "command": "wsl bash -lc \"codex exec --skip-git-repo-check -s workspace-write -\""
    },
    "review": "opus"
  }
}
```

An toàn đi kèm (tự động, decision 0018/0019): mọi `[DONE]` từ executor ngoài đều bị orchestrator chạy lại verify + check frozen-judge, không có ngoại lệ spot-check. Giữ `-s workspace-write` — không dùng `--yolo`/bypass toàn máy làm mặc định.

> **Dấu `-` ở cuối lệnh là bắt buộc, không phải trang trí (AO19).** `codex exec --help`: *"[PROMPT] — nếu không truyền làm tham số (hoặc dùng `-`), prompt được đọc từ stdin"*. Bỏ dấu `-` đi thì codex **không bao giờ đọc prompt**, và tham số trần cuối cùng bị nuốt thành câu hỏi. Đúng lỗi này đã sống trong `.bee/config.json` của repo: `... --yolo ... workspace-write` — `workspace-write` đứng vào chỗ PROMPT, `-s` bị mất, nên advisor nhận đúng chuỗi `workspace-write` làm câu hỏi và sandbox chưa từng bật (`--yolo` = `--dangerously-bypass-approvals-and-sandbox`). Sửa 2026-07-14.

**Mẹo vận hành** (từ pattern codex-first): kết quả cuối lấy qua `-o <file>` thay vì parse stream; nén stderr (`2>/dev/null`) để thinking noise không phình context; sửa lỗi tiếp theo dùng `codex exec resume --last` (giữ context, rẻ hơn chạy mới) — quá 2 vòng resume fail thì `[BLOCKED]` leo thang; cell cần MCP/secrets/tool của phiên thì không bao giờ route ra ngoài.

## 5. `antigravity-review` — Claude làm, Gemini/Antigravity (`agy`) phản biện

`agy` không đọc prompt từ stdin (khác `codex exec -`) và không có `-o`. Giữ nguyên transport stdin của bee bằng wrapper `"$(cat)"`:

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": "sonnet",
    "review": {
      "kind": "cli",
      "command": "bash -lc 'agy -p \"$(cat)\" --model \"Gemini 3.1 Pro (High)\" --mode plan --print-timeout 30m'"
    }
  }
}
```

Dùng `agy` làm worker **gather** (slot `generation`) — đọc repo, trả digest, **không cần quyền ghi**:

```json
"generation": {
  "kind": "cli",
  "command": "bash -lc 'agy -p \"$(cat)\" --model \"Gemini 3.5 Flash (High)\" --sandbox --mode plan --print-timeout 30m'"
}
```

Cờ cần biết (`agy --help`, smoke-test 2026-07-14):

- `-p/--print` = chạy một prompt non-interactive; **prompt là tham số, không phải stdin** → bắt buộc `"$(cat)"`.
- `--model` nhận đúng tên hiển thị trong `agy models` (`Gemini 3.1 Pro (High)`, `Claude Opus 4.6 (Thinking)`, `Gemini 3.5 Flash (Low)`…) — chạy `agy models` để lấy danh sách hiện có.
- `--mode plan` = không sửa file; `--sandbox` = hạn chế terminal. Dùng **cả hai** cho mọi slot advice-class (gather, review) — AO8: slot chỉ đưa lời khuyên thì không cần quyền ghi.
- `--print-timeout` mặc định **5 phút**, quá ngắn cho một cell thật → luôn nâng lên (30m).
- `--add-dir` nếu cell cần đọc ngoài repo.

> **`--dangerously-skip-permissions` đã bị gỡ khỏi mọi preset (2026-07-14).** Đo được: `agy --sandbox --mode plan` đọc repo và trả lời đúng câu hỏi thật mà **không** cần cờ đó — quyền ghi là thừa, không phải điều kiện. Và một worker CLI **nằm ngoài tầm write-guard của bee**: bee chỉ thấy dòng `bash -lc`, trích ra zero đường dẫn, rồi cho qua; mọi byte CLI ghi sau đó bee không thấy. Worker của bee có write-guard làm lưới chắn prompt-injection — worker CLI thì không có gì. Đừng bật lại cờ này làm mặc định (`swarming-reference.md:91`).

> **Chưa dùng cli cho slot `generation` để CHẠY CELL.** Đo được 2026-07-14: cwd của một CLI ngoài **không phải** repo root (agy tự dời sang sandbox riêng của nó), nên các lệnh `.bee/bin/bee` bằng đường dẫn tương đối sẽ chạy vào một thư mục `.bee/` **ma** — reservation, cap, kết quả verify đều ghi vào nơi bee không đọc, trong khi worker vẫn báo done. Đường gather (chỉ đọc) thì an toàn; đường thực thi cell đang chờ lập kế hoạch lại.

## 6. `opencode-review` — reviewer qua opencode CLI

Chưa smoke-test trên máy này (opencode chưa cài) — cờ lấy từ docs opencode.ai, kiểm lại bằng `opencode run --help` trước khi tin.

```json
"models": {
  "claude": {
    "extraction": "haiku",
    "generation": "sonnet",
    "review": {
      "kind": "cli",
      "command": "bash -lc 'opencode run --agent plan -m anthropic/claude-opus-4-6 \"$(cat)\"'"
    }
  }
}
```

Worker implement (ghi được file):

```json
"generation": {
  "kind": "cli",
  "command": "bash -lc 'opencode run --agent build --auto -m anthropic/claude-sonnet-4-6 \"$(cat)\"'"
}
```

Cờ cần biết:

- `opencode run "<message>"` — prompt cũng là tham số, **không có stdin** ([issue #25508](https://github.com/anomalyco/opencode/issues/25508) còn mở) → lại dùng `"$(cat)"`.
- `-m/--model` theo dạng `provider/model` (`anthropic/claude-opus-4-6`, `openai/gpt-5.5`…).
- `--agent plan` = không sửa file (read-only cho reviewer); `--agent build` + `--auto` = auto-approve, dùng cho worker.
- Siết cứng hơn thì khai báo agent riêng trong `opencode.json`: `"permission": { "edit": "deny", "bash": "deny" }`.
- Không có `-o` → lấy kết quả từ stdout; `--format json` nếu muốn parse.

## Ràng buộc chung cho mọi executor ngoài

- **Prompt luôn đi qua stdin** từ `.bee/workers/<cell-id>.prompt.md` (giao kèo dispatch trong `bee-swarming`). CLI nào không đọc stdin thì bọc `bash -lc '... "$(cat)"'` — đừng nhét prompt thẳng vào command trong config.
- `"$(cat)"` truyền prompt qua argv nên vẫn dính trần ARG_MAX; prompt cực dài (đính kèm diff to) thì cân nhắc rút gọn contract file hoặc dùng cơ chế attach file của chính CLI đó.
- Slot `review` phải **read-only** (`--mode plan` / `--agent plan` / `-s read-only`); chỉ slot `generation` mới được quyền ghi, và giới hạn trong repo — không lấy bypass toàn máy làm mặc định.
- Mọi `[DONE]` từ executor ngoài đều bị orchestrator chạy lại verify + frozen-judge (decision 0018/0019), không có ngoại lệ.

## 7. `budget` — tiết kiệm tối đa

Review rơi về generation (chấp nhận mất reviewer độc lập — hợp lane tiny/small):

```json
"models": {
  "claude": { "extraction": "haiku", "generation": "sonnet", "review": null }
}
```

## Config demo chạy được ngay

Muốn một file `.bee/config.json` hoàn chỉnh thay vì ghép từng khối: [`.bee/config-sample-cli-executors.json`](../.bee/config-sample-cli-executors.json) — `generation` đi qua **agy** (`Gemini 3.5 Flash (High)`), `review` đi qua **opencode** (read-only), `extraction` giữ `haiku`. Copy đè `.bee/config.json` là chạy.

Đã kiểm 2026-07-14: `resolveTier` đọc file này ra `generation -> cli`, `review -> cli` với command nguyên vẹn, và đúng lệnh `generation` trong đó chạy qua transport stdin của bee trả về kết quả thật. Chỉ copy khi máy đã cài `agy`/`opencode` — thiếu CLI thì mọi dispatch worker sẽ fail.

## Đổi preset

Sửa `.bee/config.json` → chạy `.bee/bin/bee status` xem dòng `Models (claude): ...` xác nhận. Không cần onboard lại.
