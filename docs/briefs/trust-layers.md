---
title: Bắt lỗi ở tầng cao hơn — bốn phần còn thiếu của kim tự tháp tin cậy
audience: owner
size: M
ask: test
---

**Là gì:** Việc này thêm bốn phần còn thiếu. Một: bee tìm lint và type check của project và đề nghị chạy chúng trước test. Hai: một trang hướng dẫn biến một lỗi thành luật lint của project. Ba: lời sửa của anh và phát hiện từ review được đếm chung với lỗi máy tự ghi. Bốn: bee báo độ lớn phần văn xuôi máy phải đọc.

**Vì sao:** Lỗi bắt ở tầng cao thì rẻ. Một lời nhắc lặp lại hai lần nghĩa là luật đang nằm quá thấp.

**Nguồn gốc:** Anh gửi hình kim tự tháp năm tầng ngày 2026-11-13, nhờ đánh giá, rồi nói làm toàn bộ.

**Chi phí và rủi ro:** Cỡ vừa. Không có luật chặn mới. Bee chỉ đề nghị lệnh, anh chọn. Bee im khi đã có lời từ chối được ghi.

**Anh cần làm:** Thử trên main sau khi việc được gộp, rồi chấp nhận hoặc từ chối.

**Chuẩn xong:**
- Cài bee vào project có lệnh lint thì thấy lời đề nghị thêm lint trước test.
- Trang hướng dẫn luật tĩnh có mặt và cây thăng cấp trỏ tới nó.
- Ghi được một lời sửa có nguồn là anh, và nguồn sai bị từ chối.
- Bản tổng hợp feedback in một dòng về độ lớn văn xuôi.
