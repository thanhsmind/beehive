# Test Economy — Context

**Feature slug:** test-economy
**Date:** 2026-07-25
**Exploring session:** complete (scoping synthesis — design settled in-session from issue #66 + mechanism analysis)
**Scope:** Standard
**Domain types:** RUN | ORGANIZE

## Feature Boundary

Giảm gánh nặng test/verify của bee xuống mức "bằng chứng rẻ nhất đủ trung thực" — proof yêu cầu khi cap scale theo loại thay đổi và lane, hình dạng test bị ràng buộc, suite có lực xoá đối trọng, debug/validate ưu tiên đọc trước chạy, và impacted run có trần — mà KHÔNG hạ chuẩn bằng chứng cho security/migration/high-risk. Kết thúc ở: máy móc cap/verify/skill text; không đụng CI workflow, không tách god-module (backlog), không thêm verify_policy mới (backlog).

## Locked Decisions

These are fixed. Planning must implement them exactly — cited, never reinterpreted.
Changing one requires the user, a new D-ID or an explicit supersession note, never
a silent edit.

| ID | Decision | Rationale (only if it changes implementation) |
|----|----------|-----------------------------------------------|
| D1 | Proof yêu cầu khi cap suy ra từ `change_class × lane` (proof-tier), không còn nghi thức nặng nhất cho mọi behavior_change. Thêm `refactor` vào `CHANGE_CLASSES`. Ma trận ĐỦ 7 nhánh class: `refactor`/`formatting` = suite có sẵn xanh + CẤM file test mới (cấm tuyệt đối — `new_suite_reason` của D3 KHÔNG override được; refactor cần suite mới nghĩa là phân loại sai); `bugfix` = per D2; `behavior` = một test mức hành vi/public-interface, targeted-green (red-first chỉ khi lane high-risk); `api` = như `behavior` (một test mức contract, targeted-green; high-risk lane ⇒ red-first); `security`/`migration` = red-first đầy đủ, giữ nguyên sàn 80-char + chống trùng; **unclassified (null)**: `bc=true` ⇒ derive `behavior` như hiện tại rồi theo tier behavior; `bc=false` ⇒ như hiện tại (không matrix check, targeted-green theo lane) — muốn hưởng tier nhẹ hơn behavior phải KHAI class, mặc định không nới. **Site enforcement: handler `cells cap` trong bee.mjs** (capCell không có child_process/git — cells.mjs:4-75): handler chạy `git status --porcelain`/`git diff --numstat` trên `files_changed`, truyền `diff_stats` đã tính vào capCell; git lỗi ⇒ fail-open kèm warning ghi log (cùng triết lý hooks fail-open), không chặn oan. | Đo repo: 64.5k dòng test / ~27k dòng source; guard red-evidence theo behavior_change (cells.mjs:1794-1853) là nguồn phát test lớn nhất |
| D2 | Red-first thu hẹp phạm vi: chỉ còn bắt buộc cho change_class `security`/`migration` (mọi lane) và lane `high-risk` (mọi class). Bugfix/behavior/api ở tiny/small/standard: targeted test xanh là đủ. **Supersede đích danh:** guard decision 0009 (cells.mjs:1794-1820 — mọi bc-cell phải có red_failure_evidence) và sàn D3 self-correcting-loop (cells.mjs:1832-1853 — ≥80 chars, chống trùng) chỉ còn áp cho các nhánh red-first ở trên; các nhánh được nới thì hai guard này không áp. Decisions `e54878b1`/`8ef2bae6` (scoped red-first) được AMEND: *hình* red-first (chạy đúng test của cell, không full suite) giữ nguyên ở nơi red-first còn áp; *phạm vi khi nào* thu hẹp theo D-này. | User chốt theo triết lý tốc độ ship; giữ nghi thức nặng đúng chỗ rủi ro thật |
| D3 | Luật hình dạng test: ≥3 case cùng hành vi bắt buộc table-driven; kịch bản mới thêm vào suite có sẵn; file `test_*.mjs` MỚI trong diff cap (detect từ `diff_stats` do handler truyền — per D1) phải khai `new_suite_reason` — thiếu thì cap từ chối (trừ class refactor/formatting: cấm thẳng per D1); trần tỷ lệ dòng-test-thêm/dòng-source-đổi từ `git diff --numstat` (tiny/small cảnh báo, standard+ chặn — ngưỡng cụ thể quyết ở planning). | Mỗi file test mới tự thành suite CI bắt buộc vĩnh viễn (run_verify.mjs:315-337 auto-discovery) — điểm sinh nghĩa vụ phải có gác |
| D4 | `bee-grooming` thêm mũi săn test-prune (dedupe/merge/xoá test giá trị thấp, verify xanh sau prune, agent review-tier quét); `bee-compounding` báo cáo census suite (số suite, tổng dòng, delta của feature). | Suite hiện tăng đơn điệu, không có lực xoá đối trọng auto-discovery |
| D5 | Read-first/run-second: repro script chỉ được viết SAU khi ghi giả thuyết kèm bằng chứng đọc code (file:line); tối đa 2 vòng repro sai rồi bắt buộc quay về đọc/instrument; validating tiny/small dùng bằng chứng tĩnh (trích dẫn file:line) trừ khi đụng công nghệ chưa có tiền lệ trong repo; trước khi viết test mới phải trích test có sẵn gần nhất + vì sao chưa cover. **Đây là luật prose** — sống trong skill text (bee-executing/bee-validating) và được review bắt, không có hook máy kiểm; chấp nhận có chủ đích (pattern "nothing tests prose" đã cân nhắc). | User: "bắt đầu bằng đọc code hiểu sẽ rẻ hơn rất nhiều" |
| D6 | Trần impacted — áp CHỈ cho đuôi transitive (level ≥2): suite tự đổi (self-selected) và suite import TRỰC TIẾP file đổi (level-1 `direct` trong registry — impact_registry.mjs trả cả `direct` lẫn `all`) LUÔN chạy, không bao giờ bị trần cắt (giữ nguyên critical pattern sibling-suite: shared-mutator surface luôn re-run sibling suites). Khi tổng impacted transitive vượt ngưỡng (mặc định 40% tổng suite, key `verify_impacted_cap` trong `.bee/config.json`): chạy self-selected + level-1, in `impacted N/<total> exceeds cap — transitive tail delegated to CI`, **exit code = kết quả phần đã chạy** (đỏ vẫn đỏ — chỉ phần delegated im lặng, khác nhánh unmapped exit-0-vô-điều-kiện). Nhánh delegate best-effort `gh workflow run CI` (client-side, không sửa workflow file; gh thiếu/lỗi ⇒ bỏ qua kèm một dòng note — CI hiện chạy cron hằng ngày nên cửa sổ mù tối đa ~24h là chấp nhận có chủ đích). | Lib nóng (state.mjs) làm transitive impacted ≡ full run vì registry đo theo file (impact_registry.mjs:416-442); level-1 rẻ và chính là nơi bug lan đầu tiên |
| D7 | HOÃN ra ngoài feature: (a) `verify_policy: "deferred"` cho host project — backlog PBI, đo lại sau khi D1-D6 hạ khối lượng; (b) tách god-module `state.mjs` — giao mũi grooming, không thuộc feature này. | Chống phình scope |
| D8 | Negative-control bắt buộc cho mọi lần nới: mỗi guard được thu hẹp phạm vi (D1/D2/D6) phải ship trong CÙNG cell với test table-driven hai chiều — chiều nới (case được phép qua) và chiều giữ (case tier nặng VẪN bị từ chối: security thiếu red evidence, high-risk thiếu red, refactor kèm file test mới, suite level-1 vẫn chạy khi vượt trần). Comment trong code dùng id đầy đủ `test-economy D#` (namespace "D3"/"0009" đã bị chiếm trong cells.mjs). | Critical pattern "Clearing a red by widening the threshold is not fixing the check" — negative control là thứ phân biệt hai hành vi |

### Agent's Discretion

- Ngưỡng số cụ thể của D3 (trần tỷ lệ theo lane) và D6 (40% mặc định) — chọn ở planning, ghi vào plan.md, config-key đặt tên nhất quán với `commands.*` hiện có.
- Vị trí chính xác các luật D5 trong skill text (bee-executing / bee-validating / AGENTS.md guardrail) — planning quyết, miễn là luật enforce được nêu đúng một nơi canonical và các nơi khác trỏ về.
- Cách khai `new_suite_reason` (flag của `cells cap` hay field trong evidence-stdin) — chọn theo pattern `verification_evidence` sẵn có.
- Hình dạng cụ thể của `diff_stats` handler truyền vào capCell (per D1) — planning quyết theo signature capCell hiện có.

## Terms

| Term | Meaning in this feature |
|------|-------------------------|
| proof-tier | Bậc bằng chứng tối thiểu capCell chấp nhận, suy từ change_class × lane — không phải mức trần, worker được phép nộp nhiều hơn |
| targeted-green | Suite hẹp của cell chạy xanh, không đòi chứng minh trạng thái đỏ trước đó |
| hot-file fan-out | Impacted set phình gần bằng full suite vì file bị đổi được gần như mọi suite import |
| level-1 / direct | Suite import trực tiếp file đổi theo registry (`direct`); suite tự đổi luôn là direct của chính nó (impact_registry.mjs:408) — nên "level-1" đã bao self-selected |
| negative-control | Test chứng minh guard VẪN từ chối ở nhánh không được nới — cặp bắt buộc của mỗi lần nới (D8) |

## Specific Ideas And References

- Issue #66 (repo này) — 5 hướng: integration-over-unit (Testing Trophy), table-driven, luật prompt test, core-domain focus, AI prune. Gói D1-D6 là bản máy-móc-hoá của cả 5.
- Tiền lệ trong repo: nhánh unmapped của run_verify đã biết in "full verify delegated to CI" — D6 tái dùng HÌNH THÔNG ĐIỆP, không tái dùng exit code (per D6: exit = kết quả phần đã chạy; unmapped exit 0 vì chạy 0 suite).

## Existing Code Context

### Reusable Assets

- `skills/bee-hive/templates/lib/cells.mjs` (canonical; 5 mirrors, HAI đường sync: `onboard_bee.mjs --repo-root . --apply` sinh `.bee/bin/`, `.claude/skills/`, `.agents/skills/` + refresh ledger `.bee/onboarding.json`; `scripts/render_plugin_skill_trees.mjs` sinh `.claude-plugin/`, `.codex-plugin/` — docs/06-runtime-integration.md:91-94. `--files` lúc cap gồm đủ mirror + `.bee/onboarding.json`; suite `test_lib_mirror` + `ledger_parity --check` đỏ nếu sót) — `capCell` :1704, refusal ladder :1741-1869, `CHANGE_CLASSES` :85, `deriveChangeClass` :87-91, `RED_EVIDENCE_MIN_CHARS` :1690. D1/D2/D3 sửa tại đây; guard git-diff sống ở handler `cells cap` trong `templates/bee.mjs` (:1358 files_changed, :3600 git helper, spawnSync :38 — capCell không có child_process; caller duy nhất bee.mjs:1361 nên site không né được). `templates/bee.mjs` cùng quy tắc 5-mirror.
- `scripts/run_verify.mjs` — `discoverSuites` :315-337, unmapped-delegate :981-1002. D6 sửa tại đây.
- `scripts/impact_registry.mjs` — closure file→suites :416-442. D6 đọc, không đổi.
- `skills/bee-executing/SKILL.md` :65-71, :116-118, :149-151 — chỗ neo luật proof/red-first hiện tại; D1/D2/D3/D5 sửa text.
- `skills/bee-validating/SKILL.md`, `skills/bee-grooming/SKILL.md`, `skills/bee-compounding/SKILL.md` — D5 (evidence tĩnh), D4 (prune + census).

### Established Patterns

- Template là canonical — mọi sửa lib/skill đi từ `skills/bee-hive/templates/`, rồi sync bằng CẢ HAI đường: `onboard_bee.mjs --apply` (runtime `.bee/bin/` + `.claude/` + `.agents/` + ledger) và `render_plugin_skill_trees.mjs` (2 cây plugin). Không sửa mirror tay.
- Refusal-with-reason trong capCell (throw message dài, actionable) — luật mới theo cùng giọng.
- Sentinel/config pattern: `NO_TEST_SENTINEL`, `commands.*` trong `.bee/config.json` — key mới của D6 theo cùng chỗ.

### Integration Points

- `capCell` evidence path (`--evidence-stdin`, `verification_evidence`) — `new_suite_reason` và proof-tier checks cắm vào đây.
- `run_verify.mjs --impacted-from-git` selection — trần D6 cắm sau bước map suites, trước execute.
- Goal-check judge (routing-and-contracts.md :284-288) — KHÔNG đổi; xác nhận judge không đòi coverage nên không cần chỉnh.

## Canonical References

- `docs/knowledge/index.md` ("Critical patterns") — các pattern red-first/oracle liên quan; luật mới không được mâu thuẫn với pattern "Clearing a red by widening the threshold..." (D1-D2 đổi *yêu cầu*, có negative-control test chứng minh tier nặng vẫn enforce).
- GitHub issue #66 — nguồn vấn đề.

## Outstanding Questions

### Deferred To Planning

- [ ] Ngưỡng số D3/D6 — chọn từ số liệu thật (109 suite trong registry, phân bố impacted điển hình).
- [ ] Test cho chính máy móc mới (meta): dùng suite cap/cells hiện có mở rộng table-driven — tự tuân D3. (Câu hỏi derive `refactor` đã đóng trong D1: unclassified không bao giờ tự nhận tier nhẹ.)

## Deferred Ideas

- `verify_policy: "deferred"` cho host project (per D7a) — backlog PBI.
- Tách god-module `state.mjs` thành module hẹp để fan-out tự teo (per D7b) — mũi grooming tương lai.

## Handoff Note

CONTEXT.md is the source of truth. Decision IDs are stable. Planning reads locked
decisions, code context, canonical references, and deferred-to-planning questions.
Validating and reviewing use locked decisions for coverage and UAT.
