---
title: Giới hạn số thợ chạy cùng lúc chỉ ghi ở một nơi
audience: owner
size: S
ask: build
---

**Là gì:** Bee giới hạn số thợ, tức tác tử phụ, chạy cùng lúc. Tài liệu hiện ghi hai con số khác nhau: một chỗ ghi khoảng ba đến bốn, một chỗ ghi bốn. Mã có một mục cấu hình cho giới hạn này, nhưng kho này không đặt mục đó. Vì vậy mã không giới hạn gì cả.

**Vì sao:** Tài liệu hứa một giới hạn mà mã không thực hiện. Nếu nhiều thợ chạy quá mức, máy chậm và chi phí tăng mà không ai hay.

**Nguồn gốc:** Một đợt nghiên cứu xếp việc này hạng một. Tôi đã kiểm tra lại, và mọi điều trên vẫn đúng.

**Chi phí và rủi ro:** Cỡ nhỏ. Rủi ro: một giới hạn mặc định quá thấp làm chậm các đợt chạy song song đang dùng hiện nay.

**Anh cần làm:** Làm. Mã đã giữ con số. Chỉ còn sửa một tài liệu và thêm một bài kiểm tra.

**Chuẩn xong:**
- Con số giới hạn chỉ nằm ở một nơi.
- Mọi chỗ trong tài liệu chỉ đến nơi đó, không ghi số riêng.
- Một bài kiểm tra cho thấy bee từ chối thợ vượt giới hạn mặc định.
