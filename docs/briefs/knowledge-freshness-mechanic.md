---
title: Cơ chế độ mới cho tri thức dự án
audience: owner
size: M
ask: build
---

**Là gì:** Mỗi mẩu tri thức ghi rõ nó neo vào tệp mã nào và được xác nhận ở bản mã nào. Bee tự chấm mỗi mẩu là còn đúng, đáng ngờ, hay mồ côi, và in điểm đó ở mọi chỗ agent tra cứu.

**Vì sao:** Hôm nay hai phần ba tri thức vùng kể về mã đã đổi, và agent không có cách biết. Tri thức cũ không bị đánh dấu còn hại hơn không có tri thức.

**Nguồn gốc:** Buổi brainstorm ngày bảy tháng mười một về tri thức bắt đầu từ mã. Bản đồ khám phá đã chốt tám quyết định; việc này xây phần cơ chế.

**Chi phí và rủi ro:** Mỗi lần đóng ô việc, người làm phải xác nhận lại hoặc sửa tri thức neo vào tệp vừa đổi. Tốn một bước ở đúng người biết rõ nhất. Rủi ro là bước đó bị né bằng lối thoát có ghi lý do.

**Anh cần làm:** Duyệt định nghĩa neo: danh sách đường dẫn tường minh, gieo một lần từ phần Pointers. Đây là phỏng đoán của agent, anh có thể đổi trước khi cắt nội dung.

**Chuẩn xong:**
- Kiểm tra bundle chấm bốn trạng thái; tra cứu in trạng thái cạnh mỗi kết quả.
- Đóng ô việc bị chặn khi tri thức neo vào tệp đã đổi chưa được xác nhận.
- Báo cáo đếm số lần agent kéo tri thức theo tính năng.
- Dự án khác nhận cơ chế qua onboarding, không cần cấu hình thêm.
