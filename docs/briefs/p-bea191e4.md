---
title: Có giữ chế độ tự gộp code khi agent bên ngoài vào làm ở đây?
audience: owner
size: S
ask: answer
---

**Là gì:** Hiện nay repo này đặt điểm dừng thử nghiệm ở bước đóng. Nghĩa là: khi kiểm tra xanh, agent tự gộp code vào nhánh chính và không hỏi anh. Giờ waggledance có thể mở agent chính trong repo này. Vì vậy, một agent do bên ngoài mở cũng có thể tự gộp code.

**Vì sao:** Waggledance không chặn được việc này. Chỉ cấu hình của repo này chặn được. Repo cũng đang cho agent tự qua nhiều cổng duyệt.

**Nguồn gốc:** Một báo cáo từ waggledance, sau khi chỗ ngồi giám sát của nó chạy thật.

**Chi phí và rủi ro:** Phần sửa rất nhỏ, chỉ đổi cấu hình. Nếu giữ: code từ agent bên ngoài có thể vào nhánh chính khi anh vắng mặt. Nếu đổi: mọi lần gộp chờ anh duyệt, nên chậm hơn.

**Anh cần làm:** Anh chọn một trong hai. (a) Giữ như cũ và ghi lại là anh chấp nhận rủi ro. (b) Đổi để mọi lần gộp chờ anh duyệt. Tôi đề nghị (a), nếu anh xem lại nhánh chính hằng ngày. Việc từ waggledance cũng do chính anh giao.

**Chuẩn xong:**
- Quyết định của anh được ghi vào sổ quyết định.
- Nếu anh chọn (b), cấu hình repo đặt điểm dừng ở bước gộp.
