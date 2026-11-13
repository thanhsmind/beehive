---
title: Mọi project chạy thử sản phẩm trước khi deploy
audience: owner
size: M
ask: test
---

**Là gì:** Bee viết cho mỗi project một bản ghi và một script deploy. Hiện script đó build, test, rồi đẩy thẳng lên môi trường thật. Việc này thêm một bước bắt buộc: trước bước đụng môi trường thật, script chạy toàn bộ bộ chạy thử sản phẩm của project. Hỏng thì dừng. Project chưa có bộ chạy thử thì script cũng dừng, trừ khi người chạy tự cho phép bỏ qua lần đó.

**Vì sao:** Anh muốn việc chạy thử khi release là cơ chế cho mọi project, không chỉ beehive.

**Nguồn gốc:** Anh nói ngày 2026-11-13, sau khi beehive có bước chạy thử trong script release.

**Chi phí và rủi ro:** Cỡ vừa. Hai mẫu, hai skill, và một lời nhắc mới khi cài hoặc nâng cấp bee. Project có script deploy cũ sẽ thấy lời nhắc ở lần nâng cấp tới, và agent thêm bước khi chạy deploy lần sau.

**Anh cần làm:** Thử trên main sau khi việc được gộp, rồi chấp nhận hoặc từ chối.

**Chuẩn xong:**
- Mẫu script deploy có bước chạy thử trước bước đầu tiên đụng môi trường thật.
- Project có script deploy cũ thấy lời nhắc khi chạy cài đặt bee.
- Bộ chạy thử đầy đủ báo lỗi bằng mã thoát khi có tính năng hỏng.
