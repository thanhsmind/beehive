---
title: Tri thức chạy cùng luồng feature
audience: owner
size: L
ask: build
---

**Là gì:** Hôm nay chỉ cell nhận tri thức. Shaping và planning không nhận gì, và close chỉ để lại đề xuất không ai áp dụng. Sau việc này, bee nạp một gói tri thức cùng code nó trỏ tới lúc shaping và planning. Mỗi cell ghi tri thức cùng commit với code. Một worker gom tri thức trước merge, nên cả hai vào main cùng lúc.

**Vì sao:** Leader gần như không tự đọc tri thức: 17 lần trên 10174 lệnh ở waggledance. Đề xuất tồn đọng: 45 ở đây, 114 ở waggledance.

**Nguồn gốc:** Spec drop sup-20261109-57a6 từ waggledance, 9 quyết định anh đã chốt.

**Chi phí và rủi ro:** Cỡ lớn, chạm dispatch, close, merge, báo cáo và bốn skill. Rủi ro là tri thức phình ra; bước gom cắt trùng lặp. Không thêm cửa duyệt mới.

**Anh cần làm:** Thử trên main sau merge. Tôi chạy luôn theo chế độ tự duyệt đang bật.

**Chuẩn xong:**
- Một feature chạy trọn thấy gói tri thức nạp lúc shaping và planning.
- Mỗi commit cell mang phần tri thức của nó; merge có code và tri thức cùng nhau.
- Báo cáo tri thức thấy nhóm có ghi nhận ở mức cell và mức feature.
- Nợ cũ ở repo này được xả một lần.
