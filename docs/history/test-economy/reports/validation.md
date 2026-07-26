# test-economy — validation report (Gate 3 evidence)

Date: 2026-07-25 · Plan rev 3 · Phương pháp: static-evidence (per D5 — đọc trước chạy; mọi anchor xác minh bởi 2 reviewer độc lập fresh-eyes + 1 plan-checker, tất cả đối chiếu code thật)

## Feasibility — đã chứng minh bằng file:line

1. **capCell nhận `diff_stats` sạch:** signature options-object `cells.mjs:1704`, backward-compatible (caller cũ ⇒ undefined ⇒ fail-open). Caller duy nhất `bee.mjs:1361` — site không né được. capCell không import child_process (imports :4-75) ⇒ tier logic unit-test không cần git.
2. **Handler có git:** `spawnSync` import `bee.mjs:38`; `--files` split :1363-1369; helper mới cần viết (runBacklogGit :3600 là local-cho-backlog).
3. **D6 cắm được:** `runImpactedMode` `run_verify.mjs:970-1013`; `queryRegistry(..., {level})` trả `direct`/`all` (`impact_registry.mjs:463,474`, level là số 1); `SUITES` exported (109 suites). Config reader chưa có — te-3 viết.
4. **Ngưỡng 0.30 sống:** đo thật `state.mjs` all=36/109=33% (trên trần — đúng rationale), union te-1 ≈ 39.4%.
5. **Sync 2 đường xác nhận:** `onboard_bee.mjs` copy_helper :2940-2945 (bee.mjs), copy_lib :2955-2961 (cells.mjs), applySyncSkill :1577 (2 root managed :355-356) + ledger; render_plugin_skill_trees TARGET_ROOTS :38-40 (2 cây plugin). `test_lib_mirror` + `ledger_parity --check` là lưới.
6. **Suite verify thật:** te-1/te-2 `test_cells` + `test_bee_cli` (cap coverage :1686/:4110, matrix :1791-1829); te-3 `test_run_verify_impacted` + `test_impact_registry` (registry map xác nhận); te-4/te-5 `test_doctrine_parity,test_skill_render,test_skill_pointers,test_ledger_parity,test_lib_mirror` (SKILL.md UNMAPPED — đây là gate thật duy nhất).
7. **CI xanh:** 3 run gần nhất trên main success (mới nhất 2026-07-25T11:24Z); không issue verify-red mở.

## Review trail

- Fresh-eyes vòng 1: 13 findings (3 P1) — sửa hết vào CONTEXT.
- Fresh-eyes vòng 2: 1 P1 (cơ chế sync mirror sai) — sửa; 4 note nhỏ hấp thụ.
- Plan-check: 2 P1 (trần 0.4 inert; verify prose) + 7 P2 + 4 P3 — hấp thụ toàn bộ vào rev 3.

Kết luận: khả thi, không giả định chưa kiểm. Đủ điều kiện thực thi.
