---
title: Chạy song song nhiều máy phụ: tránh mở nhầm ô và gõ phím hai lần
audience: internal
size: M
ask: build
---

**Là gì:** Lệnh herding mở máy phụ trong các ô, tức khung terminal. Còn hai lỗi nghi ngờ. Lỗi thứ nhất: mỗi lần mở máy phụ, lệnh hỏi lại ô nào đang được chọn. Khi nhiều máy mở cùng lúc, ô đang được chọn có thể là ô của máy khác, nên ô mới mở sai chỗ. Lỗi thứ hai: khi ô bận, lệnh thử lại và có thể gõ một phần phím hai lần.

**Vì sao:** Hai lỗi này có thể làm máy phụ nhận sai lời giao việc.

**Nguồn gốc:** Phát hiện từ tính năng giao lời nhắn cho máy phụ. Cả hai lỗi vẫn còn trong mã. Một bản sửa trước chỉ làm cho tab mới không lấy mất chỗ đang chọn.

**Chi phí và rủi ro:** Cỡ vừa: hai chỗ sửa, mỗi chỗ có một bài kiểm tra. Rủi ro: nếu xóa dòng nhập sai cách, chữ đang gõ có thể bị mất.

**Anh cần làm:** Anh không cần làm gì. Kế hoạch mặc định là sửa trong lượt herding tới.

**Chuẩn xong:**
- Lệnh ghi lại ô gốc một lần lúc bắt đầu chạy.
- Khi thử lại, lệnh không gõ phím hai lần.
- Mỗi lỗi có bài kiểm tra riêng, và bộ kiểm tra herding chạy đạt.
