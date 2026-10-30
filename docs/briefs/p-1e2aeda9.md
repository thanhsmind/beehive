---
title: Chặn máy phụ giao việc cho vai mà nó không được phép giao
audience: owner
size: M
ask: park
---

**Là gì:** Máy phụ là tác tử chạy một việc trong ô terminal riêng, và nó có thể tự giao việc cho máy khác. Việc này thêm cho mỗi vai một danh sách vai được phép giao. Giao ngoài danh sách thì bee từ chối.

**Vì sao:** Không có giới hạn, máy phụ có thể tự mở thêm máy phụ, hoặc dùng mô hình đắt hơn mà không ai cho phép.

**Nguồn gốc:** Việc này được tách ra khi học từ dự án piggery. Máy phụ chạy qua Paseo đã bị chặn giao việc. Máy phụ chạy qua herdr và tmux thì chưa bị chặn. Mã hiện tại chưa có danh sách này.

**Chi phí và rủi ro:** Cỡ vừa: một trường cấu hình mới, một bước kiểm tra và các bài kiểm tra. Rủi ro: cấu hình sai chặn cả việc hợp lệ.

**Anh cần làm:** Có hai cách. Cách 1: hoãn đến khi thấy máy phụ giao việc sai trong thực tế. Cách 2: làm ngay. Tôi đề nghị hoãn, vì chưa thấy lỗi này xảy ra.

**Chuẩn xong:**
- Cấu hình nhận một danh sách vai được phép giao cho mỗi vai.
- Bee từ chối khi máy phụ herdr hoặc tmux giao việc cho vai ngoài danh sách.
- Nếu không có danh sách, bee chạy như hôm nay.
