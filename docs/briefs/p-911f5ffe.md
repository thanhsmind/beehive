---
title: Cho phép lưu bản kế hoạch đã khóa vào kho mã
audience: internal
size: S
ask: build
---

**Là gì:** Sau khi anh duyệt hình dạng một tính năng, bản kế hoạch của tính năng đó bị khóa, và không ai được sửa nó nữa. Nhưng bộ chặn cũng chặn lệnh đưa tệp vào danh sách chờ lưu. Vì vậy bản kế hoạch đã khóa không lưu vào kho mã được.

**Vì sao:** Bản kế hoạch đã duyệt phải được lưu lại làm hồ sơ. Hiện nay người làm phải đưa cả thư mục cha vào thay cho tệp đó. Cách này lách bộ chặn và dễ đưa nhầm tệp khác vào.

**Nguồn gốc:** Lỗi này xảy ra ngày 29 tháng 8 khi làm một tính năng. Tôi đã đọc mã: bộ chặn vẫn coi lệnh đưa tệp vào là một lần ghi.

**Chi phí và rủi ro:** Cỡ nhỏ, chỉ sửa bộ chặn. Rủi ro: nếu sửa quá rộng, một lệnh ghi thật có thể lọt qua.
**Anh cần làm:** Không cần anh quyết định. Mặc định là sửa khi có chỗ trống, vì đây là lỗi rõ ràng.

**Chuẩn xong:**
- Lệnh đưa bản kế hoạch đã khóa vào danh sách chờ lưu chạy được.
- Lệnh sửa nội dung bản kế hoạch đó vẫn bị từ chối.
- Một bài kiểm tra kiểm cả hai trường hợp trên.
