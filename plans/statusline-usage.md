# statusline-usage (tiny)

Vấn đề: statusline chỉ hiện context của model chính; subagent chạy model khác (sonnet/haiku/opus) không được tính → không thấy cost thật của session.

Shape: cell `statusline-usage-1`, 2 files:
- `.claude/statusline-usage.mjs` — đọc stdin JSON của statusline, parse transcript chính + `<session-dir>/subagents/*.jsonl`, dedupe theo `message.id` (dòng cuối thắng), gộp token theo model, tính cost (fable 10/50, opus 5/25, sonnet-5 2/10 intro tới 2026-08-31, sonnet 3/15, haiku 1/5 $/MTok; cache write 1.25×/2× input theo TTL 5m/1h, cache read 0.1×). Cache theo signature size+mtime trong tmpdir. Fail-open: lỗi → in rỗng, exit 0.
- `.claude/statusline-command.sh` — nối segment qua node, guard để thiếu node/script lỗi không phá statusline.

Reality check (đã chạy trước khi shape): xác nhận cấu trúc `message.usage`/`message.model` trong transcript thật, subagents nằm ở `<session>/subagents/agent-*.jsonl`, cùng một `message.id` xuất hiện nhiều dòng với usage lũy tiến → cần dedupe. Giá lấy từ skill claude-api (cached 2026-06-24).

Gate 2 (merged shape+execution): auto-approved (gate_bypass, tiny). Verify: `node --check` + `bash -n` + smoke test với session có subagents.
