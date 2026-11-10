---
title: Viết lại knowledge hiện có theo năm luật mới
audience: owner
size: L
ask: build
---

**Là gì:** Năm luật viết knowledge mới chỉ áp dụng cho mục viết từ nay. Việc này sửa các mục cũ: thêm dòng "Read when" cho 140 area concept và 183 pattern, đổi khoảng 40 anchor thư mục thành anchor file, tách 27 concept dài hơn 300 dòng.

**Vì sao:** Pack chỉ hiện một dòng cho mỗi concept. Mục cũ không có dòng "Read when", nên agent vẫn phải đoán từ tiêu đề. Anchor thư mục làm concept bị đánh dấu nghi ngờ ở hầu hết commit. Concept dài chiếm hết ngân sách đọc của lane nhỏ.

**Nguồn gốc:** Đo ngày 10 tháng 11 khi làm tính năng knowledge-useful-rules. Lệnh kiểm tra knowledge giờ báo ba lỗi này.

**Chi phí và rủi ro:** Cỡ lớn, phần lớn là viết. Rủi ro: tách concept làm hỏng liên kết, nên mỗi lần tách phải chạy lại lệnh kiểm tra. Nên chia theo area để nhiều worker làm song song.

**Anh cần làm:** Chọn duyệt để làm, hoãn, hoặc bỏ. Tôi đề nghị duyệt sau khi có số liệu dùng knowledge từ vài tính năng, để làm trước các concept được dùng nhiều nhất.

**Chuẩn xong:**
- Lệnh kiểm tra knowledge không còn báo thiếu dòng "Read when".
- Không còn anchor thư mục.
- Không còn concept dài hơn 300 dòng.
