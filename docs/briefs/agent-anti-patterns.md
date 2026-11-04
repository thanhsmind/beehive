---
title: Plan chỉ khóa kết quả và giao kèo, không viết sẵn code bằng chữ
audience: owner
size: S
ask: build
---

**Là gì:** Mỗi việc giao cho worker sẽ nói rõ kết quả cần có và giao kèo phải giữ. Plan không còn chỉ cách đặt tên hàm hay sắp xếp code bên trong. Plan cũng chỉ chia nhiều bước khi có lý do thật. Lệnh `bee report outcomes` có thêm một dòng đếm số lần worker làm khác plan.

**Vì sao:** Bài viết về anti-pattern chỉ ra rằng plan quá chi tiết biến những đoán chưa kiểm thành luật. Trong bee, 144 lần worker thấy plan sai về một sự thật.

**Nguồn gốc:** Review bài vhlam.com về anti-pattern, ngày 2026-11-04.

**Chi phí và rủi ro:** Nhỏ. Việc này đổi chữ trong hướng dẫn và thêm một dòng vào báo cáo.

**Anh cần làm:** Không cần làm gì. Sau vài tuần, xem dòng "plan departure" trong `bee report outcomes`.

**Chuẩn xong:**
- Luật viết cell nói rõ: khóa giao kèo, để worker tự chọn cách làm.
- Chia bước chỉ khi có phụ thuộc thật.
- `bee report outcomes` có dòng "plan departure".
