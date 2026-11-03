---
title: Danh sách lệnh của lõi bee-design khớp với các lệnh nó nhắc tới
audience: owner
size: S
ask: build
---

**Là gì:** Phần lõi của bộ thiết kế có một danh sách lệnh được phép gọi. Nhưng các tài liệu trong lõi lại nhắc tới nhiều lệnh khác, như ds soul, vr gate và evidence. Sau việc này, mỗi lệnh lõi nhắc tới đều có trong danh sách, hoặc được ghi rõ là không dùng trong dự án.

**Vì sao:** Máy làm thiết kế đọc tài liệu lõi và có thể gọi một lệnh mà danh sách cấm. Các gói đã được sửa lỗi này, còn lõi thì chưa.

**Nguồn gốc:** Phát hiện khi làm việc design-os-checks ngày 2026-10-30.

**Chi phí và rủi ro:** Nhỏ. Danh sách lõi hiện là file của upstream, nên cần chọn: thêm một danh sách riêng của bee cho lõi, hoặc xin upstream mở rộng. Chờ thì máy vẫn có thể gọi sai lệnh ở phần lõi.

**Anh cần làm:** Làm ngay, hoặc để sau. Tôi đề nghị làm sau đợt nâng 0.6.2, vì cách làm đã có sẵn từ các gói.

**Chuẩn xong:**
- Mỗi lệnh mà tài liệu lõi nhắc tới đều có trong một danh sách.
- Bài kiểm tra sẽ báo lỗi khi lõi nhắc tới một lệnh lạ.
