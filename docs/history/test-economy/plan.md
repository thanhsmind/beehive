# test-economy — plan

**Lane:** standard · **Plan rev:** 3 · **Date:** 2026-07-25
**Source of truth:** `docs/history/test-economy/CONTEXT.md` (D1-D8)
**Rev 2 delta:** hấp thụ 2 vòng fresh-eyes — enforcement site chuyển sang handler `cells cap` trong `templates/bee.mjs` (per D1), sync mirror hai đường (onboard `--apply` + render), D6 chỉ cắt đuôi transitive, đếm file untracked ngoài numstat, pin nhánh high-risk×bc=false.
**Rev 3 delta (plan-check):** D6 default 0.30 (0.4 inert trên số liệu thật — state.mjs all=36/109=33%, union te-1 = 39.4%); verify te-4/te-5 thành lệnh thật; risk meta-circularity viết lại (cap của MỖI cell chạy dưới luật MỚI vì onboard --apply nằm trong cell); pin refactor×high-risk; dedupe mirror khỏi ratio; evidence JSON bắt buộc cho field mới; thêm rows matrix null×bc=true và no-git fail-open; cap D6 no-op khi --level 1; re-anchor cites (0009 = :1793-1820, D3 floor = :1826-1856, runImpactedMode = :970-1013).

## Shape

5 cells, tuần tự (te-1 → te-5). Mỗi cell đụng template: sửa canonical `skills/bee-hive/templates/` (và/hoặc `skills/<skill>/SKILL.md`), rồi sync bằng CẢ HAI đường trong cùng cell — `node skills/bee-hive/scripts/onboard_bee.mjs --repo-root . --apply` (`.bee/bin/`, `.claude/`, `.agents/`, ledger `.bee/onboarding.json`) và `node scripts/render_plugin_skill_trees.mjs` (`.claude-plugin/`, `.codex-plugin/`). `--files` lúc cap gồm đủ mirror + ledger; `test_lib_mirror` + `ledger_parity --check` là lưới đỏ nếu sót.

Số liệu nền: 109 suites trong registry; 64.5k dòng test / ~27k dòng source canonical.

## Ngưỡng & pin chốt (per Agent's Discretion)

- **D3 tỷ lệ:** `test_lines_added / max(source_lines_changed, 1)` — tiny/small: warning (trường `warnings` trong kết quả cap) khi >3; standard/high-risk: refuse khi >4 trừ evidence khai `ratio_waiver` ≥20 chars (audit-log một decision). Đo: `git diff --numstat` cho file tracked **+ đếm dòng file untracked mới** (`git status --porcelain` `??` + `wc -l`) — numstat mù với untracked, đúng ca file test mới toanh. **Dedupe mirror:** path thuộc `.bee/bin/`, `.claude/skills/`, `.agents/skills/`, `.claude-plugin/`, `.codex-plugin/` bị LOẠI khỏi cả tử số lẫn mẫu số — chỉ đếm canonical (đo thật: template edit không dedupe permissive ~6×, còn file unmirrored như run_verify.mjs chịu luật chặt hơn bất công).
- **D3 new-suite:** file `test_*.mjs` mới (untracked hoặc added) trong `files_changed` ⇒ evidence bắt buộc `new_suite_reason` ≥20 chars; class refactor/formatting ⇒ cấm thẳng, reason không cứu (D1 thắng D3). **Evidence JSON bắt buộc** cho `new_suite_reason`/`ratio_waiver`: `parseVerificationEvidence` (cells.mjs:98-108) trả `{}` với prose — refusal message phải nói rõ "evidence phải là JSON có field X".
- **Pin refactor/formatting × high-risk:** lệnh cấm file test mới của D1 giữ; red-first KHÔNG áp (refactor thật không có hành vi mới để red-test — bắt red là ép misclassify); yêu cầu = suite có sẵn xanh, như mọi lane. D2 "high-risk mọi class" đọc là "mọi class có hành vi để chứng minh" — behavior/api/bugfix/security/migration.
- **D6:** key `verify_impacted_cap` (float 0-1, **default 0.30** — đo thật: state.mjs all=36/109=33% phải nằm TRÊN trần mới đúng rationale; 0.4 là van chết) đọc từ `.bee/config.json` — run_verify.mjs chưa có config reader, te-3 viết (fs + REPO_ROOT sẵn, missing-file ⇒ 0.30). Điểm cắm: `runImpactedMode` (run_verify.mjs:970-1013), dùng `queryRegistry(..., {level})` — `level` là SỐ 1, không phải chuỗi (impact_registry.mjs:463,474). Trần chỉ áp khi chạy transitive (không `--level 1`); caller đã `--level 1` ⇒ no-op, không banner. Vượt trần ⇒ chạy level-1 `direct` (đã bao suite tự đổi — registry :408), in `impacted N/<total> exceeds cap — transitive tail delegated to CI`, exit = kết quả phần chạy; best-effort `gh workflow run CI` (lỗi/thiếu gh ⇒ một dòng note, không đỏ).
- **Pin high-risk × bc=false × unclassified:** giữ như hôm nay — không matrix check (guard 0009 chỉ bắn khi bc=true); D2 "high-risk mọi class" áp cho cell CÓ class hoặc bc=true. Không regression hai chiều.
- **Enforcement site (D1):** handler `cells cap` trong `templates/bee.mjs` (caller duy nhất của capCell — bee.mjs:1361, `--files` split :1363-1369; spawnSync import :38; helper git hiện có `runBacklogGit` :3600 là local-cho-backlog — te-1 viết helper diff riêng) tính `diff_stats = {new_test_files[], test_lines_added, source_lines_changed}` truyền vào `capCell` (options-object :1704, backward-compatible — caller cũ ⇒ undefined ⇒ mọi check D1/D3 skip, fail-open). Git lỗi/không có repo ⇒ fail-open + warning vào logs/hooks.jsonl (fixture tmpdir không-git của test_bee_cli đi đường này — phải xanh). capCell nhận `diff_stats` thuần — tier logic test bằng unit không cần git.

## Cells

### te-1 — D1+D2 proof-tier trong capCell + handler diff_stats (behavior_change: true, class: behavior)
- **files:** `skills/bee-hive/templates/lib/cells.mjs`, `skills/bee-hive/templates/bee.mjs`, suite cap hiện có (per D5: trích suite gần nhất trước, thêm row table-driven — không file test mới), mirrors 2 đường + ledger.
- **action:** thêm `'refactor'` vào `CHANGE_CLASSES` :85. `requiredProofTier(change_class, lane)` per bảng D1 (7 nhánh). Thu hẹp guard 0009 (:1794-1820) + sàn self-correcting-loop (:1832-1853) theo D2-amend: chỉ security/migration (mọi lane) + high-risk (class khai hoặc bc=true); sàn 80-char/chống-trùng giữ nguyên ở các nhánh đó. Handler bee.mjs tính `diff_stats`, truyền vào capCell. Comment dùng `test-economy D#` (D8).
- **negative-control (D8, cùng cell):** table-driven hai chiều — security thiếu red ⇒ refuse; high-risk behavior thiếu red ⇒ refuse; bugfix small targeted-green ⇒ pass; **null×standard bc=true targeted-green ⇒ pass (nới lớn nhất — phải có row)**; **null×high-risk bc=true thiếu red ⇒ refuse**; refactor + file test mới ⇒ refuse (điểm này của te-2 nhưng row đặt sẵn khi te-2 bật); **no-git tmpdir ⇒ cap pass + warning logged (fail-open)**.
- **verify:** `node scripts/run_verify.mjs --only test_cells --only test_bee_cli` (unit thuần trong `templates/tests/test_cells.mjs` — matrix/derive :1791-1829, red caps :1249/:1319; handler e2e trong `templates/tests/test_bee_cli.mjs` — cap :1686/:4110; hai nửa cell đều phải có suite gọi tên).
- **assertion cũ nới đi:** mỗi assertion red-first bị nới phải có row negative-control chiều ngược trong cùng commit.

### te-2 — D3 test-shape guard tại cap (behavior_change: true, class: behavior)
- **files:** `skills/bee-hive/templates/lib/cells.mjs` (+ bee.mjs nếu diff_stats cần mở rộng), cùng suite test te-1 (thêm row, không file mới), mirrors + ledger.
- **action:** trong capCell, từ `diff_stats`: (a) `new_test_files` không rỗng + thiếu `new_suite_reason` ⇒ refuse (refactor/formatting ⇒ refuse vô điều kiện per D1); (b) ratio per ngưỡng chốt (mirror-deduped) — warn tiny/small, refuse standard+ trừ `ratio_waiver`. Refusal messages nêu rõ evidence JSON.
- **verify:** `node scripts/run_verify.mjs --only test_cells --only test_bee_cli` (+ row direct-import trong `test_cli_cells.mjs:1194-1223` nếu đụng waiver path).

### te-3 — D6 trần impacted trong run_verify (behavior_change: true, class: behavior)
- **files:** `scripts/run_verify.mjs` (gồm config reader mới — chưa tồn tại, size cell tính đủ), mở rộng `scripts/test_run_verify_impacted.mjs` + `scripts/test_impact_registry.mjs` (suite thật đang map file này — per D5 thêm row, không file mới).
- **action:** per "Ngưỡng & pin chốt" D6 ở trên. Negative-control (D8): dưới trần chạy đủ; trên trần level-1 vẫn chạy + suite đỏ trong level-1 ⇒ exit đỏ; cap=1 tắt máy cắt; config thiếu ⇒ 0.30; `--level 1` ⇒ no-op không banner; gh thiếu ⇒ note không đỏ.
- **verify:** `node scripts/run_verify.mjs --only test_run_verify_impacted --only test_impact_registry`.
- **cap của chính cell:** chạy dưới luật MỚI (te-1/te-2 đã live trong `.bee/bin` sau onboard --apply + commit) — khai `change_class: "behavior"`, evidence JSON, ratio mirror-deduped tự thoả.

### te-4 — Skill text: proof-tier + read-first (D1/D2/D3/D5)
- **files:** `skills/bee-executing/SKILL.md` (bảng proof-tier thay red-first :65-71; luật D3; luật D5 "trích test gần nhất"; ghi chú amend e54878b1/8ef2bae6), `skills/bee-validating/SKILL.md` (evidence tĩnh tiny/small, giả-thuyết-trước-repro, max 2 vòng), sync skill copies (onboard --apply + render).
- **action:** canonical statement ở bee-executing, nơi khác trỏ về. Goal-check judge không đổi.
- **verify:** `node scripts/run_verify.mjs --only test_doctrine_parity --only test_skill_render --only test_skill_pointers --only test_ledger_parity --only test_lib_mirror` (SKILL.md UNMAPPED trong registry — đây là các gate thật duy nhất).

### te-5 — D4 grooming prune + compounding census
- **files:** `skills/bee-grooming/SKILL.md`, `skills/bee-compounding/SKILL.md`, sync copies.
- **action:** mũi test-prune (review-tier quét trùng/giá-trị-thấp, verify xanh sau prune); census suite trong báo cáo compounding (số suite, tổng dòng test, delta feature — shell inline, không CLI verb mới). Close action: PBI D7a/D7b đã file (p-842955b5, p-722dfd0a) — xác nhận trong done-report, không file lại.
- **verify:** `node scripts/run_verify.mjs --only test_doctrine_parity --only test_skill_render --only test_skill_pointers --only test_ledger_parity --only test_lib_mirror`.

## Rủi ro & kiểm soát

1. **Nới nhầm chỗ** ⇒ D8 negative-control trong cùng cell, hai chiều.
2. **Assertion cũ assert red-first** ⇒ nới kèm row chiều ngược cùng commit (te-1).
3. **Mirror/ledger drift** ⇒ hai đường sync trong cùng cell; `test_lib_mirror` + `ledger_parity` nằm trong impacted tự nhiên.
4. **Meta-circularity (đảo so với rev 2):** onboard `--apply` nằm TRONG cell, trước cap ⇒ cap của te-1 trở đi chạy dưới luật MỚI của chính nó (`.bee/bin` đã được ghi đè). Mỗi cell te-1..te-5 vì thế phải tự thoả guard mới: khai `change_class`, evidence JSON, ratio mirror-deduped, `new_suite_reason` nếu lỡ cần file test mới. Worker được báo trước điều này trong dispatch.
5. **CI cửa sổ mù ~24h cho đuôi delegated (D6)** — chấp nhận có chủ đích + best-effort `gh workflow run`.

## Test matrix (12-edge, rút theo lane)

- class × lane: security×tiny (red-first giữ), migration×small (giữ), bugfix×small (targeted-green pass), behavior×standard (một behavior test, không red), api×standard (như behavior), behavior×high-risk (red-first), refactor×standard (cấm file test mới), refactor×high-risk (pin: suite xanh đủ, không red, cấm file mới), formatting×tiny (suite sẵn xanh đủ), null×tiny bc=false (như cũ), null×standard bc=true (nới lớn nhất: targeted-green pass), null×high-risk bc=true (red vẫn refuse), null×high-risk bc=false (pin: như cũ), no-git fail-open (cap pass + warning logged).
- D3: file mới ±reason; refactor+reason (vẫn refuse); ratio 2.9/3.1 (tiny warn), 3.9/4.1 (standard refuse), waiver ±; evidence prose (không JSON) + file mới ⇒ refusal nêu JSON.
- D6: N/109 dưới/trên 0.30; level-1 đỏ ⇒ exit đỏ; cap=1; config thiếu ⇒ 0.30; `--level 1` no-op; gh thiếu (note, không đỏ).
