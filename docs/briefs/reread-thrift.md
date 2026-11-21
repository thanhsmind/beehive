---
title: Cắt phần token bị đọc lại mỗi lượt của leader và worker
audience: internal
size: M
ask: build
---

**Là gì:** Mỗi lượt gọi, trợ lý đọc lại toàn bộ ngữ cảnh. Việc này cắt sáu chỗ làm ngữ cảnh phình ở leader, worker và brief.

**Vì sao:** Phiên đo được tốn 103 triệu token. Leader chiếm 80 triệu. Nếu đổi phiên sau mỗi merge, leader bớt khoảng 35 triệu.

**Nguồn gốc:** Anh hỏi còn điểm nào tốn token, rồi nói "Làm toàn bộ đi" với danh sách đã đo.

**Chi phí và rủi ro:** Cỡ vừa, tám cell. Rủi ro chính: worker mất danh sách skill mà vẫn cần skill, và file luật leader bị cắt nhầm luật. Cả hai có test và phép đo chặn.

**Anh cần làm:** Thử trên main sau khi merge, rồi chấp nhận hoặc trả lại.

**Chuẩn xong:**
- Merge xong, bee in dòng nhắc đổi phiên; mở feature mới trong cùng phiên bị từ chối nếu không nêu lý do.
- Lệnh xem kết quả của một job in bản tóm tắt dưới 1500 ký tự.
- Worker mỏng khởi động với ngữ cảnh đầu không quá 15 nghìn token.
- File luật leader không quá 15000 byte, brief nhỏ hơn 25 phần trăm.
