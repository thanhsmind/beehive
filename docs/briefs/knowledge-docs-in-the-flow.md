---
title: Tài liệu thành một phần của luồng làm việc
audience: owner
size: M
ask: build
---

**Là gì:** Hôm nay agent tự tìm tài liệu khi lên kế hoạch, cập nhật theo trí nhớ khi xong việc, và viết theo khuôn riêng. Sau việc này, mỗi feature ghi rõ đã đọc tài liệu nào và code nào lúc shape, cửa duyệt kiểm tra phần đó, mỗi loại tài liệu có khuôn hỏi đúng thông tin cần thêm, và mục thiếu liên kết quyết định bị từ chối.

**Vì sao:** Tài liệu phình to nhưng không ai đọc, lý do của quyết định mất dần. Agent phiên sau và anh chịu thiệt khi phải giải thích lại.

**Nguồn gốc:** Chỉ đạo của anh ngày 2026-11-08, nối tiếp bản đồ discovery về tri thức từ code.

**Chi phí và rủi ro:** Cỡ vừa, chạm skill shaping, planning, capturing và hai verb kiểm tra. Rủi ro là khuôn nặng quá khiến agent điền cho có. Phụ thuộc cơ chế độ tươi vừa merge.

**Anh cần làm:** Cho làm ngay, hoặc gom với việc cắt gọn tài liệu bee. Tôi đề nghị làm riêng trước, vì việc cắt gọn cần khuôn này.

**Chuẩn xong:**
- Feature mới shape xong có mục đọc gì, cửa duyệt chặn khi thiếu.
- Mỗi khuôn mở đầu từng mục bằng câu hỏi nó trả lời.
- Mục thiếu liên kết quyết định bị kiểm tra từ chối.
