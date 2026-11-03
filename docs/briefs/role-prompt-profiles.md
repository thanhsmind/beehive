---
title: Cho phép sửa lời dặn của từng vai trò mà không cần build lại bee
audience: owner
size: M
ask: park
---

**Là gì:** Khi bee giao việc cho một máy phụ, nó gửi kèm một lời dặn. Nhiều lời dặn nằm cứng trong chương trình, nên muốn sửa thì phải build lại bee. Đề xuất: mỗi lời dặn có bản mặc định, và mỗi repo có thể thay nó bằng một file riêng, giống thư mục expertise.

**Vì sao:** Mỗi dự án chỉnh được cách máy phụ làm việc mà không đổi bee. Phần quy ước cố định, như dạng kết quả và mã trạng thái, vẫn bị khóa.

**Nguồn gốc:** Anh hỏi ngày 2026-08-29. Tôi đã kiểm tra mã nguồn. Bee chưa đọc file thay thế nào, và các mũ cố vấn chưa có file lời dặn.

**Chi phí và rủi ro:** Cỡ vừa. Mỗi lời dặn phải tách thành phần cố định và phần sửa được. Rủi ro: một repo sửa sai, và máy phụ trả kết quả lệch quy ước.

**Anh cần làm:** Hoãn, hoặc làm ngay. Tôi đề nghị hoãn đến khi một dự án thật cần sửa lời dặn.

**Chuẩn xong:**
- Mỗi vai trò có lời dặn mặc định, và repo có thể thay nó bằng một file.
- Thiếu file thay thế thì bee dùng bản mặc định.
- Phần quy ước cố định không bị thay, và có test giữ.
