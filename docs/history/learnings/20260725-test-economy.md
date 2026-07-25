# Learnings — test-economy (2026-07-25)

Feature: giảm gánh nặng test/verify (issue #66). 5 cells (te-1..te-5), commits df13878, 6ffdcaf, 84ae734, 9ef2e6b, aa19f35. Goal-check judge PASS cho cả 3 behavior cells.

## Suite census (per test-economy D4 — baseline đầu tiên)

- Suites trong registry: **109**
- Tổng dòng test (test_*.mjs, mọi cây): **67,007** (canonical ~1/5 do mirror)
- Delta feature này: 27 files, +567/−41 (phần lớn là luật guard + rows table-driven — mỗi guard nới đi kèm negative-control giữ)

## Learnings

1. **Nới một guard an toàn cần đúng ba thứ:** phạm vi mới phát biểu theo taxonomy có sẵn (change_class × lane), amend đích danh các decision cũ (0009, e54878b1, 8ef2bae6), và negative-control hai chiều trong CÙNG commit (test-economy D8). Cả 3 review vòng ngoài (2 fresh-eyes + 1 plan-check) đều bắt đúng lỗ ở một trong ba thứ này.
2. **Enforcement site đi theo capability, không theo nơi luật "thuộc về":** capCell không có git — guard diff phải sống ở handler CLI và truyền dữ liệu thuần vào lib. Reviewer bắt được vì đọc imports, không phải chạy thử (read-first thắng).
3. **Ngưỡng phải đo trước khi chốt:** default 0.4 cho verify_impacted_cap là van chết trên số liệu thật (hot-file lớn nhất = 33%); 0.30 mới sống. Đo mất 2 lệnh, tránh ship một feature không bao giờ kích hoạt.
4. **Meta-circularity của self-hosting:** onboard --apply nằm trong cell nghĩa là cap của cell chạy dưới luật của chính nó — chiều thuận (luật mới nghiêm hơn với chính người viết nó) là tự-dogfood miễn phí, nhưng phải báo trước cho worker để không bất ngờ.
5. **Red-first bằng git stash là red-first sạch:** te-2/te-3 chứng minh đỏ bằng cách stash code mới, chạy row mới trên code cũ, ghi fail cụ thể — môi trường không thể feed oracle (tránh đúng pattern 20260724 red-first-oracle).

## Follow-ups đã file

- p-fb5764e6: trần D6 kích hoạt mà level-1 = 0 suite ⇒ exit 0 im lặng — cân nhắc note to hơn.
- p-842955b5: verify_policy "deferred" cho host project (D7a).
- p-722dfd0a: tách god-module state.mjs giảm fan-out (D7b).
