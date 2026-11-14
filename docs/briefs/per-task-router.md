---
title: Định tuyến theo từng task, worker làm và worker test tách riêng
audience: owner
size: M
ask: build
---

**Là gì:** Leader chọn playbook và principle cho đúng task đang làm, giống poteto-mode. Preamble không còn đưa principle của feature cũ. Mỗi cell có sửa code luôn có 2 worker: một worker viết code, một worker khác kiểm tra và viết test.

**Vì sao:** Phiên mới nhận principle bugfix của windows-test-build cho một task nghiên cứu. Hiện nay worker tự chọn test để chứng minh việc của chính nó, nên proof quá hẹp vẫn ra xanh.

**Nguồn gốc:** Anh yêu cầu ngày 2026-11-14, sau bản nghiên cứu so sánh với poteto-mode. Anh chọn: worker code trước, tester sau (1b). Luật 2 worker áp dụng cho mọi cell sửa code, kể cả tiny (2a).

**Chi phí và rủi ro:** Cỡ vừa. Đổi preamble, lệnh orient, lệnh dispatch prepare, cap, playbook và hợp đồng worker. Rủi ro: mỗi cell tốn thêm một worker.

**Anh cần làm:** Không cần làm gì cho tới bước test. Tôi lập kế hoạch rồi làm.

**Chuẩn xong:**
- Phiên mới chưa gắn feature thì không có principle trong preamble.
- Leader phân loại từng task và load đúng playbook và principle của class đó.
- Mỗi cell sửa code có một tester riêng, chạy sau worker code.
- Worker nhận principle của class; cap nêu principle đã dùng và từ chối proof yếu hơn mức của class.
