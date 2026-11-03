---
title: Duyệt nghiệm thu trên luồng riêng nhưng lệnh gộp không thấy, đã sửa
audience: internal
size: S
ask: drop
---

**Là gì:** Một tính năng có thể chạy trên một luồng riêng. Trước đây, khi anh duyệt bước nghiệm thu cho luồng đó, bee ghi kết quả vào hồ sơ của luồng. Lệnh gộp mã lại không đọc hồ sơ này, nên nó vẫn từ chối gộp.

**Vì sao:** Mỗi tính năng chạy trên luồng riêng không thể gộp theo cách bình thường. Người làm phải dùng lối thoát khẩn cấp và ghi lại đó là ngoại lệ.

**Nguồn gốc:** Lỗi này xảy ra thật ngày 18 tháng 8, khi gộp một tính năng.

**Chi phí và rủi ro:** Lỗi đã được sửa. Bước kiểm tra trước khi gộp bây giờ đọc ba nguồn theo thứ tự. Hồ sơ của luồng là một trong ba nguồn đó. Mã có ghi rõ rằng trước đây bước này bỏ sót hồ sơ của luồng.

**Anh cần làm:** Không cần làm gì. Tôi đề nghị bỏ việc này, vì lỗi đã được sửa trong mã.

**Chuẩn xong:**
- Việc được đánh dấu bỏ, có ghi lý do là lỗi đã được sửa.
- Nếu lỗi xảy ra lại, ta mở một việc mới có bằng chứng mới.
