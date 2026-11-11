---
title: Dùng lệnh waggledance trước, công cụ MCP sau
audience: owner
size: S
ask: build
---

**Là gì:** Khi bee cần nói với waggledance, máy chạy lệnh waggledance trước. Nếu lệnh không có hoặc lỗi, máy mới dùng công cụ MCP. Bee không dùng lệnh wg.

**Vì sao:** Lệnh trả về ít chữ hơn nhiều. Trạng thái mọi dự án: 324.669 byte qua MCP, 6.796 byte qua lệnh. Lệnh và MCP chạy cùng một mã, nên kết quả như nhau.

**Nguồn gốc:** Yêu cầu từ waggledance ngày 2026-11-12 (sup-20261112-1646). Anh nói: "chuyển qua hỗ trợ wg cli là chính khi cần dùng waggledance", rồi "Làm hoàn chỉnh rồi release bản mới".

**Chi phí và rủi ro:** Cỡ nhỏ. Chỉ đổi lời hướng dẫn và câu báo lỗi của guard. Rủi ro: bản waggledance đang cài còn cũ, chưa có lệnh state. Vì vậy máy phải quay về MCP khi lệnh không có.

**Anh cần làm:** Không cần làm gì. Tôi làm xong rồi phát hành bản bee mới.

**Chuẩn xong:**
- Quy tắc gửi việc cho dự án khác ghi lệnh waggledance trước, MCP sau.
- Câu báo lỗi của guard ghi lệnh trước, MCP sau.
- Không chỗ nào trong bee dùng wg.
- Có bản bee mới đã phát hành.
