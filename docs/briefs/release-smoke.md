---
title: Release tự chạy thử bản bee thật trước khi gắn tag
audience: owner
size: S
ask: test
---

**Là gì:** Script release hiện chạy regen và bộ test, rồi gắn tag và đẩy lên. Không bước nào chạy thử bản bee thật. Việc này thêm một bước: script build bản bee, cài nó vào một project mẫu dùng xong bỏ, chạy vài lệnh mà người dùng gặp đầu tiên, rồi đọc kết quả. Một lệnh hỏng thì release dừng.

**Vì sao:** Test xanh chưa chắc bản cài chạy được. Một bước do script chạy thì không ai quên.

**Nguồn gốc:** Anh nói ngày 2026-11-13 rằng verify nên tự chạy khi release, rồi chọn cách đưa vào script.

**Chi phí và rủi ro:** Cỡ nhỏ. Hai script và tài liệu đi kèm. Mỗi lần release dài thêm vài phút. Bước thử dừng release trước khi có commit hay tag, nên không để lại gì dở dang. Cờ bỏ qua test cũng bỏ qua bước này, và script ghi rõ điều đó ra một dòng.

**Anh cần làm:** Thử trên main sau khi việc được gộp, rồi chấp nhận hoặc từ chối.

**Chuẩn xong:**
- Chạy lệnh thử một lần thì thấy từng bước báo đạt, và project mẫu được dọn.
- Một bước hỏng thì lệnh thoát với mã lỗi và nêu tên bước.
- Script release gọi lệnh thử sau bộ test và trước commit release.
