# scripts-tests-move — plan

**Lane:** standard (theo số file; 0 risk flag, thuần cơ học) · **Date:** 2026-07-25 · **Class:** refactor

## Mục tiêu

`scripts/` = 18 script sản xuất chìm giữa 42 file test + `fixtures/`. Dời test về `scripts/tests/` — đồng quy ước với `skills/bee-hive/templates/tests/`. Không đổi hành vi nào; bằng chứng = suite xanh sau move (proof-tier refactor, test-economy D1 — cấm sinh test mới).

## Việc (1 cell — stm-1)

1. `git mv scripts/test_*.mjs scripts/tests/` (42 file) + `git mv scripts/fixtures scripts/tests/fixtures`. `scripts/lib/` GIỮ NGUYÊN (helpers dùng chung).
2. Import tương đối trong file dời sâu thêm 1 cấp (`../` thành `../../` v.v.) — sửa máy móc, soi cả import scripts/lib và templates.
3. `scripts/run_verify.mjs`: `DISCOVERY_ROOTS` — thay root `scripts` bằng `scripts/tests` cho phần discover test (đọc kỹ: root `scripts` có còn được dùng cho EXTRA_SUITES/việc khác không — nếu EXTRA_SUITES trỏ path scripts/*.mjs sản xuất thì giữ những entry đó, chỉ đổi chỗ discover `test_*`).
4. Path reference quét toàn repo: `rg -l 'scripts/test_'` — test_verify_manifest (MANDATORY_SUITES nếu có path test), EXCLUDE/ARGS_OVERRIDE/SERIAL_EXCEPTIONS trong run_verify (kiểm basename hay full path), CI workflow (chỉ gọi run_verify — chắc không), docs chỉ sửa nơi chỉ dẫn vận hành (README/knowledge nếu nêu path test), lịch sử docs/history KHÔNG sửa.
5. Regen: `node scripts/impact_registry.mjs --write` + `node scripts/release_manifest.mjs --check` (scripts/ ngoài manifest roots — xác nhận lại, nếu cần thì --write).
6. Verify: registry/manifest/discovery suites + mẫu suite dời chạy được từ chỗ mới + đếm suite trước = sau (105).

## Rủi ro

- Suite dời không được discover ⇒ số suite tụt — `test_verify_manifest` floor + đếm 105 trước/sau bắt.
- Import gãy ⇒ suite đỏ ngay tại verify.
- Trùng basename giữa root cũ (nay không còn test) tự hết một nửa (scripts vs templates vẫn còn cặp templates — PBI p-cb0e94e3 xử sau).
