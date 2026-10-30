---
title: Bộ chặn đọc lệnh CLI ở cả hai mục cấu hình team và models
audience: internal
size: S
ask: build
---

**Là gì:** Trong tệp cấu hình, một vai có thể chạy bằng một công cụ dòng lệnh ngoài, gọi là CLI. Bộ chặn là phần kiểm tra lệnh trước khi lệnh chạy. Bộ chặn cần biết lệnh nào là CLI hợp lệ. Hiện nay nó chỉ đọc một mục: nếu có mục team, nó bỏ qua mục models.

**Vì sao:** Một CLI ghi trong mục models không được nhận ra khi tệp cấu hình cũng có mục team. Khi đó một lệnh hợp lệ có thể bị coi là lạ và bị chặn.

**Nguồn gốc:** Việc nội bộ. Tôi đã đọc mã hiện tại, và lỗi vẫn còn. Hàm kiểm tra lấy mục team trước, và chỉ đọc mục models khi không có mục team.

**Chi phí và rủi ro:** Rất nhỏ: sửa một hàm và thêm một bài kiểm tra. Rủi ro thấp. Mục models nay đã đổi tên thành team, nên ít dự án còn gặp lỗi này.

**Anh cần làm:** Anh không cần làm gì. Kế hoạch mặc định là sửa trong một lượt việc nhỏ.

**Chuẩn xong:**
- Bộ chặn nhận ra CLI ghi trong mục models khi tệp cấu hình cũng có mục team.
- Có một bài kiểm tra cho trường hợp này, và bài kiểm tra chạy đạt.
