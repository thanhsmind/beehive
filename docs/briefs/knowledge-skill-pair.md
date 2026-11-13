---
title: Kho knowledge có skill tạo và skill bảo trì, giống verify
audience: owner
size: M
ask: test
---

**Là gì:** Bee có thêm hai skill cho kho knowledge. Một skill dựng kho một lần cho mỗi project. Một skill kiểm tra định kỳ và sửa những concept có dấu hiệu sai. Khi làm xong một việc, agent có thể báo "concept này sai", và báo cáo knowledge đếm số lần đó. Luật viết có thêm hai điều: ghi rõ khi nào một ý đúng, và ghi cách tìm chi tiết dễ đổi thay vì ghi chính chi tiết đó.

**Vì sao:** Concept sai hoặc cũ hiện không ai sửa, và agent vẫn tin nó. Verify đã có cặp skill tạo và bảo trì; knowledge chưa có.

**Nguồn gốc:** Anh nói ngày 2026-11-13, sau khi đọc bài ReHarness (arXiv 2610.12668).

**Chi phí và rủi ro:** Cỡ vừa. Hai skill mới, một cờ mới ở bước cap, một mục báo cáo mới. Không thêm cửa chặn nào. Các ngưỡng là mặc định ban đầu.

**Anh cần làm:** Thử trên main sau khi việc được gộp, rồi chấp nhận hoặc từ chối.

**Chuẩn xong:**
- Có skill tạo kho và skill bảo trì kho knowledge.
- Bước cap nhận câu trả lời "concept sai" và báo cáo liệt kê nó.
- Lệnh in luật viết in ra bảy luật.
- Mỗi lần bảo trì sửa tối đa 5 concept.
