---
title: Chạy tự động bỏ qua việc không có cách kiểm tra xong, đã làm
audience: internal
size: S
ask: drop
---

**Là gì:** Ý tưởng: khi bee tự chạy việc mà không có người trông, nó bỏ qua việc mà máy không thể tự kiểm tra là đã xong. Đó là việc có tiêu chuẩn xong không chỉ ra câu lệnh nào, vị trí nào trong file, hay trạng thái nào để xem.

**Vì sao:** Nếu không có gì để kiểm tra, trợ lý không biết khi nào việc xong. Khi đó nó có thể chạy mãi, hoặc dừng sai lúc và báo xong khi việc chưa xong.

**Nguồn gốc:** Luật này đã có từ ngày 1 tháng 9. Skill herding giờ bảo trợ lý đọc tiêu chuẩn xong của mỗi việc trước khi nhận. Nếu tiêu chuẩn không kiểm tra được, trợ lý bỏ qua việc đó và ghi lý do.

**Chi phí và rủi ro:** Không tốn gì. Mục này chỉ chưa được đóng sau khi luật vào skill. Nếu để lại, nó làm danh sách việc dài hơn và có thể bị làm lại.

**Anh cần làm:** Không cần làm gì. Tôi đề nghị bỏ mục này, vì luật đã nằm trong skill herding và đang chạy.

**Chuẩn xong:**
- Mục này được đóng.
- Lý do đóng ghi rõ là luật đã có trong skill herding.
