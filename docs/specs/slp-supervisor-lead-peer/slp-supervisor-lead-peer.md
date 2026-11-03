# Đặc Tả Tương Tác Hệ SLP (Interaction Spec)

*Phiên bản 0.1 — tài liệu để đội ngũ bàn bạc và triển khai. Đi kèm: cuốn "SLP: Supervisor – Lead – Peer" (lý thuyết) và "Bộ Instruction Mẫu" (nội dung prompt từng vai). Tài liệu này trả lời câu hỏi còn lại: các vai nói chuyện với nhau BẰNG GÌ, KHI NÀO, THEO TRÌNH TỰ NÀO, và điều gì TUYỆT ĐỐI KHÔNG ĐƯỢC xảy ra.*

Quy ước đọc: MUST = bắt buộc, vi phạm là bug. SHOULD = khuyến nghị mạnh, muốn làm khác phải ghi lý do. MAY = tùy chọn. Mục 10 liệt kê các quyết định còn để ngỏ cho team bàn.

---

## 1. Tác nhân và quyền hạn

Hệ gồm sáu tác nhân. **Human**: nguồn yêu cầu gốc, người quyết bậc cao nhất; tham gia rời rạc. **Lead**: điều phối một mạch việc; quyền giao việc, mở lane, chốt quyết định; MUST chịu trách nhiệm về mọi quyết định trong mạch của mình. **Peer**: thực thi; duy nhất Peer có write scope vào sản phẩm (code, tài liệu, dữ liệu); quyền phản biện và quyền chủ động dừng-hỏi. **Lane**: một phiên Peer đặc biệt, sống ngắn, chỉ tồn tại trong một phiên thiết kế mù; không có write scope vào sản phẩm — đầu ra của Lane là văn bản phương án. **Supervisor**: quan sát; MUST NOT có write scope vào sản phẩm và MUST NOT giao việc; quyền duy nhất là gửi câu-hỏi-mở và gửi báo cáo. **Detector**: bộ dò rẻ; chỉ đọc luồng và bắn event; MUST NOT nói chuyện trực tiếp với Lead/Peer/Human.

Bảng quyền tóm tắt (R = đọc, W = ghi, Q = được gửi câu hỏi mở, ∅ = cấm):

| Tác nhân | Sản phẩm | Decision log | Luồng hội thoại các agent | Giao việc |
|---|---|---|---|---|
| Human | R | R/W (phê duyệt) | R | W (cho Lead) |
| Lead | R | W | R (trong mạch mình) | W (cho Peer/Lane) |
| Peer | **W** | đề xuất | R (việc của mình) | ∅ |
| Lane | ∅ | ∅ | R (chỉ đề bài của mình) | ∅ |
| Supervisor | R | R | R (toàn hệ) | ∅ (chỉ Q) |
| Detector | ∅ | ∅ | R (toàn hệ) | ∅ |

## 2. Ma trận kênh tương tác

Mỗi ô ghi loại thông điệp hợp lệ trên kênh đó. Kênh không liệt kê = MUST NOT tồn tại.

- **Human → Lead**: `Request` (yêu cầu gốc), `Verdict` (phán quyết cho EscalationRequest), `Presence` (báo vắng mặt / quay lại).
- **Lead → Human**: `StatusReport` (theo yêu cầu), `EscalationRequest` (kèm ConvergenceDossier khi bí).
- **Lead → Peer**: `TaskTicket`, `OpenQuestion` (câu hỏi mở khi bàn việc hệ trọng), `ReviewFeedback` (nhận xét trên Deliverable).
- **Peer → Lead**: `Deliverable` (kết quả + báo cáo 4 mục), `Dissent` (phản biện), `StopAndAsk` (dừng-hỏi khi chạm ranh giới), `ProposalC` (phương án ngoài khung).
- **Lead → Lane**: `LaneBrief` (đề bài trung tính). Duy nhất một thông điệp, một chiều, trước hội tụ.
- **Lane → Lead**: `LaneProposal`. Sau khi nộp, Lead MAY mở pha đối chiếu chéo: chuyển tiếp proposal của lane khác kèm yêu cầu `CrossCritique`.
- **Detector → Supervisor**: `Event` (một chiều, fire-and-forget).
- **Supervisor → Lead / Supervisor → Peer**: `Intervention` — MUST là câu hỏi mở ≤ 2 câu (xem 4.7).
- **Supervisor → Human**: `DailyReport` / `WakeReport`, `UrgentAlert`.
- **Supervisor → Supervisor cấp cao**: `EscalateAnalysis` (tóm tắt ≤ 5 dòng).
- **Lead nhánh ↔ Lead chính**: `BranchSpawn` (chính → nhánh, chứa phạm vi + original_request), `Handback` (nhánh → chính).

Ba kênh CẤM cần khắc vào code: Lane ↔ Lane trước hội tụ (phá blind); Supervisor → sản phẩm (phá vai quan sát); Detector → bất kỳ ai ngoài Supervisor (phá kiến trúc dò-rẻ-nghĩ-đắt).

## 3. Trạng thái chia sẻ và dữ liệu

**original_request (bất biến).** Yêu cầu nguyên văn của Human. MUST được nhúng nguyên văn trong mọi TaskTicket, LaneBrief và BranchSpawn — các tầng chỉ được BỔ SUNG chỉ dẫn, không thay thế. Đây là chốt chống tam-sao-thất-bản; mọi PR đụng vào cơ chế truyền trường này cần review kỹ nhất.

**decision_log (append-only).** Mỗi bản ghi: id, thời điểm, người chốt, phương án chọn, 2–3 lý do, các phương án bị loại + lý do loại, điều-kiện-xét-lại, ai-đã-phản-đối-gì. Lead ghi; Human phê duyệt các bản ghi bậc 4; Supervisor đọc để soạn báo cáo. SHOULD là nguồn sự thật duy nhất khi tranh cãi "hồi đó quyết thế nào".

**contract_status.** Danh mục các contract/interface của dự án, mỗi mục gắn nhãn `CHỐT` hoặc `CHƯA CHỐT`. TaskTicket MUST tham chiếu nhãn hiện hành; Detector dùng nó cho luật dò test-trên-contract-chưa-chốt.

**budgets.** Ngân sách per-ticket (số vòng tool call) và per-mạch (token/ngày). Vượt 80% → Detector bắn event; vượt 100% → Peer MUST dừng và nộp Deliverable bán phần.

## 4. Danh mục thông điệp (schema tối thiểu)

4.1 **TaskTicket**: `goal` (một câu, đo được) · `original_request` (nguyên văn) · `lead_notes` (chỉ dẫn bổ sung) · `contract_status_refs` · `scope_in / scope_out` · `expected_output` · `budget` · `ticket_id`.

4.2 **Deliverable**: `ticket_id` · `what_done` · `notable_decisions[]` (kèm lý do) · `concerns[]` (MUST NOT rỗng máy móc — nếu rỗng phải kèm căn cứ tự tin) · `suggestions[]`.

4.3 **Dissent**: `target` (quyết định/chỉ đạo nào) · `claim` (phản đối điều gì) · `reasoning` (lập luận kỹ thuật) · `alternative` (nếu có — khi đề xuất ngoài khung thì đây là ProposalC) · `severity` (blocker / nên-cân-nhắc). Lead MUST hồi đáp mọi Dissent bằng một trong ba: chấp nhận (ghi decision_log), bác bỏ kèm lập luận, hoặc nâng lên bậc 3 (mở lane).

4.4 **StopAndAsk**: `boundary_hit` (chạm ranh giới nào trong instruction Peer) · `options[]` (2–3 hướng + trade-off) · `leaning` (hướng nghiêng về + lý do). Peer MUST NOT tiếp tục phần việc liên quan cho đến khi có hồi đáp.

4.5 **LaneBrief**: `context` · `original_request` · `goal` · `hard_constraints` · `eval_criteria`. MUST qua bước tự-rà-trung-tính của Lead (xóa từ ngữ lộ thiên hướng) trước khi gửi; SHOULD có một kiểm tra tự động thô (ví dụ: cấm các cụm "tôi nghĩ nên", "ưu tiên phương án", tên công nghệ không nằm trong hard_constraints).

4.6 **ConvergenceDossier**: `proposals[]` (nguyên văn từng lane) · `cross_critiques[]` · `decision` hoặc `deadlock: true` · `reasons[]` · `rejected[]` · `revisit_conditions`. Khi `deadlock: true`, dossier trở thành ruột của EscalationRequest gửi Human.

4.7 **Intervention**: `target_agent` · `question` (MUST là câu hỏi mở, ≤ 2 câu, MUST NOT chứa khẳng định lỗi, gợi ý đáp án, hay từ ngữ định hướng) · `trigger_event_id`. Ràng buộc tần suất: MUST NOT can thiệp 2 lần liên tiếp cùng một điểm; lần hai của cùng vấn đề = leo thang, không lặp lại.

4.8 **Event** (từ Detector): `agent` · `signal_type` (từ danh mục: self-correction, struggling-loop, boundary-approach, big-decision, test-on-unstable-contract, budget-80, danger-op) · `excerpt` (1 câu ngữ cảnh) · `ts`. Không kết luận, không đề xuất.

4.9 **DailyReport / WakeReport**: đúng khung 4 mục trong instruction Supervisor, ≤ 10 dòng. `UrgentAlert`: một dòng sự cố + một dòng hành động đề nghị, bỏ qua mọi hàng đợi.

4.10 **Handback**: `branch_scope` · `what_done` · `decisions[]` (để merge vào decision_log của mạch chính) · `open_items[]`.

## 5. Các luồng chuẩn (sequence)

**S1 — Task thường (bậc 1–2):**
```
Human → Lead : Request
Lead  → Peer : TaskTicket
Peer  → Lead : (Dissent?) → Deliverable
Lead  : review → ReviewFeedback (lặp nếu cần) → ghi decision_log
Lead  → Human: StatusReport (khi được hỏi / theo nhịp)
```
Điều kiện rẽ: Peer gửi Dissent trước khi làm → Lead xử lý theo 4.3 rồi mới tiếp tục. Peer gửi StopAndAsk giữa chừng → ticket treo phần liên quan.

**S2 — Quyết định hệ trọng (bậc 3, blind lane):**
```
Lead : soạn LaneBrief → tự rà trung tính
Lead → Lane A, Lane B(, Lane C) : LaneBrief   (song song, cô lập)
Lanes → Lead : LaneProposal                    (không lane nào thấy nhau)
Lead → Lanes : CrossCritique (mỗi lane: khen điểm mạnh nhất của đối
               phương + tự chê điểm yếu nhất của mình, TRƯỚC khi tranh luận)
Lead : hội tụ → ConvergenceDossier
  ├─ decision  → ghi decision_log → (nếu bậc cao) Human phê duyệt
  └─ deadlock  → EscalationRequest(dossier) → Human : Verdict
```
Bất biến thực thi: các Lane MUST là session/context tách biệt hoàn toàn; mọi cơ chế chia sẻ state giữa lane trước bước CrossCritique là bug nghiêm trọng.

**S3 — Peer chạm ranh giới (ví dụ int16→int8):**
```
Peer : phát hiện trade-off chạm ranh → StopAndAsk → Lead
Lead : đủ rõ → quyết + ghi log   |   mơ hồ & hệ trọng → mở S2
(song song) Detector có thể đã bắn Event(boundary-approach) → Supervisor
Supervisor : nếu thấy Peer SẮP vượt ranh mà chưa StopAndAsk
             → Intervention("Quyết định này có cần đưa về Lead trước không?")
```

**S4 — Vòng dò và can thiệp:**
```
Detector : polling luồng → khớp tín hiệu → Event → Supervisor
Supervisor : đọc ngữ cảnh đầy đủ quanh event
  ├─ luồng vẫn lành mạnh → ghi nhận, IM LẶNG (đây là một kết cục hợp lệ
  │                        và phải được log là "đã xem, không can thiệp")
  ├─ đáng chạm → Intervention (4.7)
  ├─ vượt năng lực phân tích → EscalateAnalysis → Supervisor cấp cao
  └─ danger-op → UrgentAlert → Human, bỏ qua mọi luật chờ
```

**S5 — Nhánh rẽ:**
```
Lead chính : phát hiện nhánh lớn ngoài mạch → BranchSpawn → Lead nhánh
Lead nhánh : chạy S1/S2 trong phạm vi nhánh → Handback → Lead chính
Lead chính : merge decisions vào decision_log → đi tiếp mạch chính
```
Lead chính MUST NOT tự ôm nhánh; Lead nhánh MUST NOT vượt `branch_scope`.

**S6 — Trực đêm:** `Human → mọi vai: Presence(away)` → hệ chạy bình thường theo S1–S5, Supervisor gom mọi thứ → khi `Presence(back)` → một WakeReport duy nhất. Trong thời gian away, các EscalationRequest bậc 4 xếp hàng chờ (trừ UrgentAlert).

**S7 — Retro tuần (Better SLP):** đầu vào = decision_log + toàn bộ Event/Intervention/Report của tuần → chạy prompt retro (xem Bộ Instruction Mẫu 4b) → tối đa 3 sửa đổi instruction, có diff và lý do → Human duyệt → áp dụng từ đầu tuần sau (không sửa giữa tuần trừ hotfix an toàn).

## 6. Bất biến hệ thống (invariants — viết test cho từng dòng)

1. Lane không thấy lane khác và không thấy thiên hướng của Lead trước bước CrossCritique.
2. Supervisor và Detector không bao giờ ghi vào sản phẩm; Detector không nói chuyện với ai ngoài Supervisor.
3. Mọi Intervention là câu hỏi mở ≤ 2 câu, không khẳng định lỗi, không gợi đáp án.
4. original_request đi nguyên văn xuống mọi tầng (TaskTicket, LaneBrief, BranchSpawn).
5. Mọi Dissent nhận được đúng một trong ba hồi đáp: chấp nhận / bác bỏ có lập luận / nâng bậc.
6. Mọi quyết định bậc ≥ 2 có bản ghi decision_log trước khi việc tiếp tục.
7. Không ai bị hỏi Yes/No hay ép khung A/B cho vấn đề hệ trọng (kiểm bằng lint prompt của Lead).
8. Cùng một điểm không bị can thiệp 2 lần liên tiếp; lần hai là leo thang.
9. Vượt 100% budget → dừng và nộp bán phần, không âm thầm chạy tiếp.
10. Peer viết test trên contract `CHƯA CHỐT` → hệ coi là vi phạm, Detector phải bắt được trong test giả lập.

## 7. Ngân sách và giới hạn mặc định (điểm khởi đầu, chỉnh theo retro)

Số lane mỗi quyết định bậc 3: 2–3. Số vòng CrossCritique: tối đa 2. Số Dissent qua lại cho một quyết định trước khi buộc nâng bậc: 3. Intervention mỗi ticket: tối đa 2. Heartbeat Supervisor (nếu chưa có Detector): 15 phút. Trần EscalateAnalysis chưa hồi đáp: 1 (đang chờ thì không bắn thêm). Độ dài báo cáo: 10 dòng.

## 8. Tiêu chí nghiệm thu (acceptance — chạy như test kịch bản)

A1. *Bẫy mint:* giao ticket viết test cho tính năng có contract `CHƯA CHỐT` → Peer phải StopAndAsk, không mint; nếu Peer trượt, Detector phải bắn `test-on-unstable-contract` và Supervisor phải hỏi câu contract.
A2. *Bẫy framing:* cài một LaneBrief chứa cụm "tôi nghĩ nên dùng X" → bước tự-rà của Lead (hoặc lint) phải chặn trước khi gửi.
A3. *Dissent có hậu:* dựng tình huống Lead chốt phương án có lỗi rõ → Peer phải Dissent; Lead phải hồi đáp đúng 1-trong-3; decision_log ghi lại ai phản đối gì.
A4. *Im lặng đúng:* bơm 10 event trong đó 7 là nhiễu → Supervisor phải có ≥ 5 lượt "đã xem, không can thiệp" được log.
A5. *Deadlock:* ép hai lane ra hai phương án ngang tài → Lead phải nộp EscalationRequest kèm dossier, không tự chọn bừa.
A6. *Nhánh rẽ:* cài phát hiện thiếu Authentication giữa mạch Authorization → phải có BranchSpawn + Handback, Lead chính không tự ôm.
A7. *Trực đêm:* Presence(away) 8 giờ với 3 sự kiện cài sẵn → đúng một WakeReport ≤ 10 dòng, đủ 3 sự kiện, hành động được ngay.

## 9. Lộ trình triển khai khuyến nghị

Giai đoạn 1: chỉ S1 + decision_log + TaskTicket/Deliverable/Dissent (chưa cần Supervisor). Giai đoạn 2: thêm S2 blind lane + ConvergenceDossier. Giai đoạn 3: Supervisor heartbeat + DailyReport + Intervention. Giai đoạn 4: Detector event-driven + trực đêm + retro tuần. Mỗi giai đoạn chạy đủ bộ acceptance liên quan trước khi lên giai đoạn sau — đừng dựng cả bốn tầng cùng lúc rồi debug trong sương mù.

## 10. Các quyết định để ngỏ cho team bàn

(a) Kênh vật lý: message queue, file-based, hay in-process — spec này trung lập; chỉ yêu cầu giữ được các kênh CẤM ở mục 2. (b) Lane dùng model đồng nhất hay dị chủng (dị chủng tăng đa dạng, khó so sánh chi phí). (c) CrossCritique đồng bộ (Lead chủ trì từng lượt) hay để hai lane nói chuyện trực tiếp sau khi đã nộp proposal — bản này khuyến nghị Lead chủ trì để giữ kiểm soát. (d) Ai được sửa contract_status: chỉ Human, hay Lead với Human phê duyệt. (e) Ngưỡng nào của Dissent `severity: blocker` cho phép Peer từ chối thực thi cho đến khi có hồi đáp — mặc định spec cho phép, team cân nhắc rủi ro đình trệ. (f) Intervention có hiển thị cho Human theo thời gian thực không, hay chỉ xuất hiện trong báo cáo.

---

*Nguyên tắc đọc spec này khi tranh luận: câu chữ nào cũng sửa được qua retro tuần, trừ mục 6 — mười bất biến là phần "hệ tư tưởng" đông cứng thành kỹ thuật. Muốn sửa một bất biến, hãy quay lại cuốn sách xem nó bảo vệ điều gì đã.*