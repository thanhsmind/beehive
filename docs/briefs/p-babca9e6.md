---
title: Ghi lệnh tạo lại gói giao việc cho mỗi việc trong một đợt
audience: internal
size: S
ask: build
---

**Là gì:** Lệnh giao việc theo đợt chuẩn bị nhiều việc trong một lần gọi. Mỗi việc nhận một gói giao việc. Nếu một gói bị mất, hiện không có chỗ nào nói cách tạo lại gói đó. Việc này ghi lệnh tạo lại vào kết quả, hoặc vào phần trợ giúp.

**Vì sao:** Khi mất gói, agent phải đoán lệnh, hoặc chạy lại cả đợt. Code đã cho phép tạo lại gói, nhưng không chỗ nào ghi lệnh.

**Nguồn gốc:** Một báo cáo từ waggledance ngày 6 tháng 10. Hôm nay tôi đọc phần trợ giúp. Nó nói tên của agent thợ, nhưng không ghi lệnh tạo lại.

**Chi phí và rủi ro:** Cỡ nhỏ, chỉ thêm chữ vào kết quả hoặc phần trợ giúp. Rủi ro thấp. Lệnh vẫn từ chối chạy trong worktree, như quyết định cũ.

**Anh cần làm:** Anh không cần quyết định gì. Đây là việc nội bộ, nên tôi đề nghị làm khi có chỗ trống.

**Chuẩn xong:**
- Mỗi việc trong đợt có ghi lệnh tạo lại gói.
- Lệnh đó cho ra gói giống hệt gói cũ.
- Lệnh vẫn từ chối chạy trong worktree.
