---
title: Kiểm tra biểu đồ, sơ đồ và chuyển động của design-os trong bee
audience: owner
size: M
ask: build
---

**Là gì:** Hôm nay bee mang phần gu thiết kế cho biểu đồ, sơ đồ và chuyển động, nhưng chưa kiểm tra chúng. Các gói còn bảo máy chạy lệnh mà phần lõi cấm. Sau việc này, mỗi gói có danh sách lệnh riêng, đúng với bản 0.6.2. Bước kiểm tra thiết kế cũng xem cả biểu đồ, sơ đồ, luồng màn hình và khối chuyển động.

**Vì sao:** Máy làm biểu đồ hay sơ đồ hiện không có gì chặn lỗi. Lệnh trong gói lại mâu thuẫn với lõi, nên máy dễ làm sai.

**Nguồn gốc:** Yêu cầu từ waggledance ngày 2026-10-30.

**Chi phí và rủi ro:** Cỡ vừa. Rủi ro chính: bản 0.6.2 thêm một nhóm kiểm tra mới. Trên dự án thật, nhóm này làm số lỗi tăng mạnh. Bee sẽ không tính nhóm mới cho tới khi anh ghi lại mốc so sánh.

**Anh cần làm:** Làm ngay, hoặc giữ bản cũ. Tôi đề nghị làm ngay, vì nhóm kiểm tra mới chỉ được báo, không chặn việc.

**Chuẩn xong:**
- Mỗi gói có danh sách lệnh riêng, không còn mâu thuẫn với lõi.
- Bước kiểm tra chạy được trên biểu đồ và sơ đồ mẫu.
- Dự án thật vẫn qua bước kiểm tra với mốc cũ.
