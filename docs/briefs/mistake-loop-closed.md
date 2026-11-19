---
title: Đóng kín vòng lặp sửa lỗi: ghi nhận trung thực, món nợ hiện rõ và chốt chặn có chứng cứ
audience: internal
size: L
ask: build
---

**Là gì:** Trợ lý ghi mỗi lỗi kèm tầng sửa. Lúc đóng tính năng, bee chỉ ghi lỗi thành một dòng P3 trong backlog rồi dừng. Việc này nối bước còn thiếu: từ dòng P3 đến một chốt chặn thật như lint, test, hook.

**Vì sao:** Có 343 dòng đang chờ. Chỉ một lỗi đã được chặn thật bằng luật clippy. Trigger đồng hồ nợ đủ điều kiện từ lâu nhưng vẫn chờ người bấm. Cửa đóng việc cũng hở: một câu trả lời chung xóa nợ của mọi cell.

**Nguồn gốc:** Hai tính năng trước cố ý dừng ở dòng P3. Bước biến dòng thành chốt được giao cho grooming, nhưng chưa ai làm đều.

**Chi phí và rủi ro:** Cỡ lớn, chia lát. Lát đầu làm món nợ hiện ra trong trạng thái và cho trigger tự bắn, rủi ro thấp. Siết cửa đóng việc có thể chặn nhầm phiên điều phối.

**Anh cần làm:** Duyệt để định hình, làm lát đầu trước.

**Chuẩn xong:**
- Trạng thái phiên hiện số dòng còn mở và tuổi dòng cũ nhất.
- Trigger đồng hồ nợ tự bắn khi đủ điều kiện.
- Mỗi vòng lặp làm một chốt, dọn chỗ cũ, ghi dòng đã đóng kèm tên chốt.
- Cửa đóng việc nêu tên từng cell chưa trả lời.
