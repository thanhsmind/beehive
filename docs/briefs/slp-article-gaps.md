---
title: Cho worker biết phần nào được hỏi lại, và đo tác dụng mỗi lớp kiểm tra
audience: internal
size: M
ask: build
---

**Là gì:** Brief giao cho worker sẽ tách ba phần: mục tiêu, ràng buộc bắt buộc, và lựa chọn thiết kế đang dùng. Worker được hỏi lại phần thứ ba nếu có bằng chứng. Bee cũng có thêm lệnh lệnh báo cáo kết quả. Lệnh này cho biết mỗi lớp kiểm tra đã đổi kết quả bao nhiêu lần.

**Vì sao:** Bài viết về SLP chỉ ra lỗi "chiếc dù": worker chỉ xác nhận lựa chọn có sẵn. Trong bee, worker gần như không bao giờ phản biện. Bee cũng chưa đo được lớp kiểm tra nào thật sự có ích.

**Nguồn gốc:** Review bài vhlam.com về SLP, ngày 2026-11-04.

**Chi phí và rủi ro:** Cỡ vừa. Không bắt worker phải cãi, vì việc đó chỉ tốn token. Lệnh mới chỉ đọc, không đổi dữ liệu.

**Anh cần làm:** Thử lệnh lệnh báo cáo kết quả sau khi xong.

**Chuẩn xong:**
- Brief của worker ghi rõ ba phần, và cell có thể mang danh sách lựa chọn mở.
- lệnh báo cáo kết quả in số liệu cho từng lớp. Lớp không có dữ liệu ghi "not recorded".
- Không có câu nào bắt worker phải tìm lỗi.
