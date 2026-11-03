Title: Claude Artifact

URL Source: https://claude.ai/public/artifacts/a9391123-a75a-4e62-b912-bb2487ff7cd8

Markdown Content:
## AGENT HARNESS SPECIFICATION — BẢN BÀN GIAO ĐẦY ĐỦ

## Hệ thống Supervisor · Leader · Advisor · Peer (mô hình 6 Mũ Tư Duy, 1 Human + N Agent)

> **Phiên bản:** Final (gộp v1–v3, tự chứa — không cần tài liệu nào khác để implement) **Người đọc:** Người/đội dựng harness (Claude Code, n8n, LangGraph, hoặc API thuần) **Mục tiêu:** Human chỉ ra yêu cầu 1 lần và nhận báo cáo cuối. Toàn bộ thẩm định – thiết kế – thực thi – nghiệm thu do hệ thống agent tự vận hành, có sổ sách audit đầy đủ. **Câu thần chú thiết kế:**_Supervisor route, Leader nghĩ, Advisor gỡ, Peer làm, QA chấm, Orchestrator đếm._

* * *

## PHẦN I — KIẾN TRÚC

## 1. Nguyên tắc nền

1.   **Tách điều phối khỏi suy nghĩ.** Supervisor (model nhỏ, stateful) quyết "ai làm gì tiếp theo". Leader (model mạnh, stateless theo lần triệu) quyết "cái gì là đúng". Không trộn hai việc này vào một agent.
2.   **Giả định thay vì hỏi.** Hỏi human là ngoại lệ đắt đỏ. Leader được phép giả định, nhưng mọi giả định và quyết định phải ghi sổ (Assumption Ledger, Decision Log) để human kiểm toán sau — không chặn pipeline.
3.   **Luật vật lý nằm ở code.** Validate schema, đếm vòng lặp, trần chi phí, timeout do Code Orchestrator (không phải LLM) giữ. LLM không tự nới bất kỳ giới hạn nào.
4.   **Silent Brainstorming.** 5 Hat Agents phân tích song song, cách ly tuyệt đối — không agent nào thấy output của agent khác trước khi tổng hợp. Chống groupthink từ kiến trúc.
5.   **Không ai tự chấm bài mình.** QA Checker là call riêng, prompt riêng, không chia sẻ hội thoại với người làm ra output.

## 2. Bảng vai trò

| Thành phần | Model | Trạng thái | Trách nhiệm 1 câu |
| --- | --- | --- | --- |
| **Human Requester** | — | Ngoài pipeline | Ra yêu cầu, đọc Final Report, trả lời escalation hiếm hoi |
| **Code Orchestrator** | Không phải LLM | — | Validate schema, đếm loop, trần chi phí, timeout |
| **Supervisor** | Nhỏ (Haiku-class) | **Stateful** — giữ TaskEnvelope suốt vòng đời | Điều phối: phân làn, route, triage pushback & bí, gọi đúng người đúng lúc; tự xử trọn Làn 1 |
| **Leader** | Mạnh nhất | Stateless theo lần triệu (state trong envelope) | Suy nghĩ đắt: enrich, giả định, synthesis, trade-off, WBS |
| **Advisor** | Mạnh | Stateless, 1 call | Gỡ bí kỹ thuật (HOW) cho peer; không làm hộ, không sửa spec |
| **5 Hat Agents** | Trung (Sonnet-class) | Stateless | Phân tích song song theo góc nhìn cố định, trả JSON |
| **Peer Workers** | Theo độ khó task | Stateless theo task | Thực thi TaskPacket, không mở rộng scope |
| **QA Checker** | Trung | Stateless | Chấm output theo acceptance criteria, độc lập tuyệt đối |
| **Context Store** | Không phải LLM | Persistent | Docs, codebase, ADR, quyết định cũ — nguồn tra cứu |

## 3. Sơ đồ tổng

```
[Human: Yêu cầu] ──────────────────────────────────────► [Human: Final Report]
      │                                                          ▲
      ▼                                                          │
╔════════════════════ SUPERVISOR (model nhỏ, giữ state) ═════════╧═══════════╗
║  Vòng lặp: nhìn envelope → quyết bước kế → gọi đúng người → cập nhật state ║
║  Làn 1: tự xử trọn (story + dispatch), KHÔNG đánh thức Leader              ║
║  Làn 2-3: triệu Leader đúng 3-4 khoảnh khắc nặng não                       ║
╚═══╦═══════════╦═══════════╦═══════════╦═══════════╦═══════════╦═══════════╝
    │ triệu     │ fan-out   │ dispatch  │ gửi chấm  │ gỡ bí     │ escalate
    ▼           ▼           ▼           ▼           ▼           ▼
 [LEADER]   [5 MŨ song   [PEER 1..N] [QA CHECKER] [ADVISOR]  [Human async]
  enrich     song]           │            │           │
  resolve                    └── output ──┘           │
  synthesis                       ▲                   │
  spec-fix  ◄── pushback khó      └── feedback/gợi ý ─┘
```

## 4. Decision Rights Matrix (bảng phân quyền — trái tim của hệ thống)

Quy tắc đọc: quyết định thuộc ô nào, **chỉ** tầng đó được quyết. Gặp việc ngoài quyền → chuyển đúng tầng.

| Quyết định | Orchestrator | Supervisor | Leader | Advisor |
| --- | --- | --- | --- | --- |
| Validate JSON, đếm loop, trần call, timeout | ✅ |  |  |  |
| Phân làn (R0 + classifier), nâng làn khi có finding cao |  | ✅ |  |  |
| Route task → peer theo dependency graph |  | ✅ |  |  |
| Triage pushback: tra Context Store, trả lời nếu có sẵn |  | ✅ |  |  |
| Pushback không có trong Context Store → giả định |  | ❌ cấm | ✅ + ghi Ledger |  |
| Chốt trade-off (mọi quyết định có trường `cua`) |  | ❌ cấm | ✅ + ghi Decision Log |  |
| Enrich context_pack, synthesis, WBS |  |  | ✅ |  |
| Sanity-check output peer trước khi tốn call QA |  | ✅ |  |  |
| Chấm acceptance criteria | ❌ | ❌ | ❌ | ❌ (chỉ QA Checker) |
| Gỡ bí kỹ thuật (HOW) cho peer |  | ❌ |  | ✅ |
| Nhận ra vấn đề nằm ở spec, sửa TaskPacket |  |  | ✅ | 🚩 chỉ cắm cờ |
| Soạn escalation message, gom batch |  | ✅ soạn | ✅ quyết nội dung |  |
| Đề xuất policy/R0 mới |  | 🚩 gom tín hiệu | ✅ viết đề xuất |  |

**Hai rule cứng** (nằm cả trong prompt Supervisor lẫn validate của Orchestrator):

1.   Thấy trường `cua` hoặc `severity: "cao"` trong bất kỳ quyết định nào → Supervisor bắt buộc chuyển Leader.
2.   Forward đi _lên_ Leader luôn nguyên văn JSON; chỉ được nén khi đi _xuống_ peer (context_slice).

Orchestrator từ chối mọi `DecisionEntry` có `nguoi_quyet: "supervisor"` kèm `cua: "1 chiều"`.

* * *

## PHẦN II — QUY TRÌNH

## 5. Phân làn 3 tốc độ (One-way vs Two-way Doors)

| Làn | Tỷ lệ kỳ vọng | Loại việc | Quy trình | Ai tham gia |
| --- | --- | --- | --- | --- |
| **1 — Fast-Track** | ~70% | UI tweak, bug fix, CRUD nhỏ, đổi copy (cửa 2 chiều) | Supervisor tự viết story + acceptance criteria → dispatch ngay | Supervisor + Peer + QA |
| **2 — Standard** | ~20% | Feature trung bình, thêm bảng DB phụ | 1 mũ (Mũ Đen) soi schema & edge case → Leader synthesis rút gọn | + Mũ Đen + Leader |
| **3 — Deep Review** | ~10% | Payment, đổi kiến trúc, migrate DB, auth/security, dữ liệu nhạy cảm (cửa 1 chiều) | Full flow: 5 mũ song song + Leader đầy đủ + DoR gate | Toàn bộ |

### 5.1 Rule R0 (tất định, chạy trước classifier)

```
NẾU yêu cầu chứa từ khóa {payment, thanh toán, migrate, auth, phân quyền, mã hóa,
    xóa dữ liệu, schema chính, kiến trúc, dữ liệu cá nhân}   → Làn 3, lane_source="rule"
NẾU yêu cầu là {đổi text/copy, đổi màu, sửa typo, toggle feature flag có sẵn}
                                                              → Làn 1, lane_source="rule"
CÒN LẠI                                                       → gọi Lane Classifier (LLM)
```

Danh sách từ khóa Làn 3 mở rộng dần theo domain: mỗi lần một task Làn 1/2 gây sự cố, thêm từ khóa của nó vào. Đây là cơ chế học rẻ nhất của hệ thống.

### 5.2 Lưới an toàn làn

1.   Classifier trả `confidence: "thấp"` → Orchestrator tự nâng 1 làn.
2.   Supervisor có quyền **nâng làn giữa chừng** (không bao giờ hạ): Mũ Đen ở Làn 2 phát hiện finding severity "cao" chạm cửa 1 chiều → nâng lên Làn 3, chạy nốt 4 mũ còn lại, ghi Decision Log.
3.   Task Làn 1 fail QA 2 lần → nâng lên Làn 2 (không đơn giản như tưởng), ghi log.

## 6. State Machine (Supervisor điều khiển mọi transition)

| State | Mô tả | Chuyển sang | Điều kiện |
| --- | --- | --- | --- |
| `INTAKE` | Nhận yêu cầu thô từ human | `ENRICHING` | Luôn |
| `ENRICHING` | Triệu Leader: tra Context Store, dựng context_pack | `LANE_ROUTING` | context_pack đủ 4 trường |
| `LANE_ROUTING` | Supervisor chạy R0 → Lane Classifier nếu cần | `FASTTRACK` / `FANOUT` | Làn 1 / Làn 2-3 |
| `FASTTRACK` | Supervisor tự viết story + acceptance criteria | `DISPATCHING` | Story xong |
| `FANOUT` | Supervisor gọi hat agents song song (Làn 2: chỉ Mũ Đen) | `COLLECTING` | Call đã gửi |
| `COLLECTING` | Chờ payload (timeout T1 = 90–120s/agent) | `RESOLVING` / `SYNTHESIZING` | Có pushback / không |
| `RESOLVING` | 2 nhịp: Supervisor tra Context Store (rẻ) → còn lại batch cho Leader giả định (đắt) | `FANOUT` (re-run mũ thiếu) / `ESCALATED` | Giải quyết được / chạm policy |
| `SYNTHESIZING` | Triệu Leader: dedup + trade-off + WBS + chốt quyết định (1 call gộp) | `DISPATCHING` / `ESCALATED` | Trong quyền / vượt Autonomy Policy |
| `DISPATCHING` | Supervisor topo-sort WBS, phát TaskPacket | `EXECUTING` | Task không phụ thuộc đã phát |
| `EXECUTING` | Peers chạy song song theo graph; Supervisor theo dõi StuckSignal | `UNBLOCKING` / `SANITY` | Bí / peer nộp bài |
| `UNBLOCKING` | Bậc thang gỡ bí (§9) — per task, không chặn task khác | `EXECUTING` / task `BLOCKED` | Gỡ được / hết trần |
| `SANITY` | Supervisor tiền kiểm output (định dạng, rỗng, lạc đề hiển nhiên) | `EXECUTING` (trả peer) / `DOD_CHECK` | Fail hiển nhiên / pass |
| `DOD_CHECK` | QA Checker chấm theo acceptance criteria | task `DONE` / `EXECUTING` | Pass / Fail (max 2) |
| `REPORTING` | Mọi task DONE hoặc BLOCKED → Leader viết Final Report (hoặc Supervisor viết nếu toàn Làn 1) | `CLOSED` | Report gửi human |
| `ESCALATED` | Chờ human — async, KHÔNG chặn nhánh không phụ thuộc | quay lại state gây escalate | Human trả lời / timeout policy |

### 6.1 Giới hạn vòng lặp (hard-code ở Orchestrator)

| Vòng lặp | Giới hạn | Khi vượt |
| --- | --- | --- |
| Re-run 1 hat (RESOLVING) | 2 lần | Leader chốt bằng giả định, độ tin "thấp" |
| Advisor / task | 2 lần | Cưỡng bức lên Leader (nghi vấn spec) hoặc BLOCKED |
| QA fail / task | 2 lần | Task BLOCKED; task phụ thuộc nó cũng BLOCKED; phần còn lại graph chạy tiếp |
| Triệu Leader / yêu cầu | Làn 2 ≤ 3, Làn 3 ≤ 6 | Supervisor phải gom việc, không triệu lắt nhắt |
| Escalate / yêu cầu | 3 câu, gom 1 batch | Quá → yêu cầu quá mơ hồ, trả human xin viết lại |
| Retry JSON parse lỗi | 1 lần / call (kèm lỗi cụ thể) | Đánh dấu agent `error`, đi tiếp nếu đủ ngưỡng |
| Tổng LLM call / yêu cầu | Trần theo làn (gợi ý: Làn 1 ≤ 8, Làn 2 ≤ 15, Làn 3 ≤ 30) | Dừng, báo cáo phần đã xong |
| Ngưỡng synthesis tối thiểu (Làn 3) | ≥ 3/5 payload hợp lệ | Dưới ngưỡng → escalate, không synthesis mù |

## 7. Pushback Resolution (thay thế việc hỏi human)

Với mỗi `missing_context` từ các mũ hoặc `need_input` từ peer:

```
missing_context
   │
   ├─ 1. SUPERVISOR tra Context Store (docs/codebase/ADR/quyết định cũ)
   │      └─ có → bơm context, re-run mũ / trả lời peer (max 2 lần/mũ)
   │
   ├─ 2. Không có → gom batch chuyển LEADER, Leader đánh giá: giả định được không?
   │      ├─ cua="2 chiều"                    → GIẢ ĐỊNH (mọi mức độ tin) + ghi Ledger
   │      ├─ cua="1 chiều" + do_tin cao/TB    → GIẢ ĐỊNH + ghi Ledger, nổi bật trong Report
   │      └─ cua="1 chiều" + do_tin thấp      → 3. ESCALATE (bất kể Autonomy Policy)
   │
   └─ 3. Escalate: Supervisor soạn batch (max 3 câu/yêu cầu), MỖI CÂU KÈM 2-3 PHƯƠNG ÁN
          gợi ý để human chỉ chọn. Gửi async — pipeline tiếp tục mọi nhánh độc lập.
```

Thang đánh `do_tin`: **cao** = Context Store có tiền lệ trực tiếp hoặc domain chỉ có 1 cách hợp lý; **trung bình** = pattern phổ biến nhưng có thể có ngoại lệ; **thấp** = phụ thuộc sở thích/chiến lược kinh doanh của human, không suy ra được từ kỹ thuật.

## 8. Autonomy Policy (núm chỉnh duy nhất human cần quan tâm)

| Mức | Cửa 2 chiều | Cửa 1 chiều | Phù hợp khi |
| --- | --- | --- | --- |
| **A — Full auto** | Leader tự quyết | Leader tự quyết, ghi Decision Log, nổi bật trong Report | Dự án cá nhân, môi trường dev |
| **B — Silence is consent**_(khuyến nghị khởi điểm)_ | Leader tự quyết | Gửi thông báo async kèm khuyến nghị; human im lặng X phút (config 30–60) → đi theo khuyến nghị | Sản phẩm có người dùng nhưng cần tốc độ |
| **C — Hard gate** | Leader tự quyết | Chặn nhánh liên quan chờ human (nhánh khác vẫn chạy) | Production có tiền/dữ liệu thật |

Gán được **theo làn**: Làn 1-2 luôn mức A; Làn 3 theo config. Bất biến không phụ thuộc policy: (1) giả định 1-chiều-độ-tin-thấp luôn escalate; (2) quyết định 1 chiều luôn vào Decision Log kèm `chi_phi_dao_nguoc`; (3) trần vòng lặp luôn do Orchestrator giữ.

## 9. Unblock Ladder — bậc thang gỡ bí cho Peer

### 9.1 Supervisor phát hiện bí qua 5 tín hiệu (không chờ peer tự khai)

| Tín hiệu | Cách bắt |
| --- | --- |
| `need_input` | Peer tự khai trong output |
| `qa_fail_lap_lai` | 2 QAVerdict liên tiếp fail cùng criteria |
| `vuot_uoc_luong` | Thời gian hoặc tool-call > 2× ước lượng trong TaskPacket (Orchestrator đo, bơm vào input Supervisor) |
| `sua_lap_cho_cu` | 2 lần nộp liên tiếp chỉ khác nhau ở cùng một vùng |
| `output_bat_thuong` | Rỗng, ngắn < 20% kỳ vọng, sai định dạng output_spec |

### 9.2 Bậc thang

```
Peer tự thử (TaskPacket đã có constraints + phu_thuoc_output)
   │ Supervisor bắt StuckSignal, triage chan_doan_so_bo:
   ├─ "how"  ──► ADVISOR (max 2 lần/task)
   │              ├─ note thường → đưa peer sửa → QA chấm lại
   │              └─ van_de_thuoc_spec=true → nhảy nhánh "what"
   ├─ "what" ──► LEADER (qua Pushback Resolution §7) → sửa TaskPacket → phát lại
   └─ "chua_ro" ─► thử ADVISOR trước (rẻ hơn); Advisor cắm cờ spec thì lên Leader
   │
   ▼ (hết trần: 2 Advisor + 1 vòng Leader vẫn kẹt)
BLOCKED — ghi Final Report kèm advisory_log; task không phụ thuộc chạy tiếp
```

### 9.3 Ba guardrail của Advisor

1.   **Không làm hộ** — trả chẩn đoán + hướng + gợi ý tối thiểu; peer vẫn nộp bài và chịu QA.
2.   **Không sửa spec** — nghi đề bài sai thì cắm cờ `van_de_thuoc_spec`, không diễn giải lại acceptance criteria.
3.   **Stateless** — mỗi lần gọi là call mới với AdvisoryRequest đầy đủ.

Tín hiệu vàng: cùng một kiểu kẹt (`chan_doan`) xuất hiện ≥ 3 lần qua các task → Supervisor gom vào `de_xuat_policy`, Leader viết đề xuất bổ sung convention vào Context Store hoặc nâng cấp TaskPacket template.

## 10. Dispatch & nghiệm thu

1.   **Dependency graph trước, phát sau.** Supervisor topo-sort WBS theo `phu_thuoc`; task độc lập phát song song cho nhiều peer.
2.   **Context slice, không context tổng.** Mỗi TaskPacket chỉ chứa phần context task đó cần. Peer cần output task trước → tóm tắt vào `phu_thuoc_output`, không forward nguyên văn.
3.   **Peer hỏi → Supervisor/Leader trả lời.** Human không bao giờ nói chuyện trực tiếp với peer.
4.   **Sanity trước, QA sau.** Supervisor chặn lỗi hiển nhiên (miễn phí) trước khi tốn call QA.
5.   **QA độc lập.** Chấm từng criteria Given/When/Then với bằng chứng. Fail → feedback nguyên văn cho peer, max 2 vòng.
6.   **Kết thúc.** Mọi task DONE hoặc BLOCKED → Final Report.

* * *

## PHẦN III — DATA CONTRACTS (JSON SCHEMAS)

Orchestrator validate mọi JSON bằng code (Zod/Pydantic) trước khi cho đi tiếp — không bao giờ validate bằng LLM. Đây là chốt chất lượng rẻ nhất toàn hệ thống.

## 11.1 `TaskEnvelope` — phong bì đi xuyên suốt, Supervisor giữ

json

```
{
  "task_id": "uuid",
  "created_at": "ISO 8601",
  "raw_request": "string — nguyên văn human, không chỉnh sửa",
  "context_pack": {
    "goal": "string — bài toán kinh doanh, KPI kỳ vọng",
    "scope_in": ["string"],
    "scope_out": ["string"],
    "constraints": ["string — tech stack, timeline, budget"]
  },
  "lane": 3,
  "lane_source": "rule | classifier | supervisor_upgrade",
  "state": "INTAKE",
  "assumption_ledger": [],
  "decision_log": [],
  "advisory_log": [],
  "leader_invocations": 0,
  "loop_counters": {
    "rerun_hat": {}, "advisor": {}, "qa_fail": {},
    "escalations": 0, "llm_calls": 0
  }
}
```

## 11.2 `LaneResult` — output Lane Classifier

json

```
{
  "lane": 1,
  "cua": "1 chiều | 2 chiều",
  "ly_do": "string, 1-2 câu",
  "tang_thieu": [
    { "tang": "Data Contract | Happy Path | Edge Cases | NFR | DoD", "dau_hieu": "string" }
  ],
  "confidence": "cao | trung bình | thấp"
}
```

## 11.3 `HatPayload` — output mỗi Hat Agent

json

```
{
  "hat": "white | black | yellow | green | red",
  "findings": [
    {
      "id": "W1",
      "severity": "cao | trung bình | thấp",
      "tang": "Data Contract | Happy Path | Edge Cases | NFR | DoD | Khác",
      "tieu_de": "string ≤ 12 từ",
      "mo_ta": "string",
      "de_xuat": "string — hành động cụ thể; Mũ Đỏ được để trống"
    }
  ],
  "missing_context": ["string — thông tin cần bổ sung"],
  "pushback": false
}
```

Validate: `id` prefix đúng chữ cái mũ (W/B/Y/G/R). `pushback: true` chỉ hợp lệ khi `missing_context` không rỗng.

## 11.4 `AssumptionEntry` — sổ giả định

json

```
{
  "id": "A1",
  "cau_hoi_goc": "string — pushback/lỗ hổng dẫn đến giả định",
  "gia_dinh": "string",
  "can_cu": "context_store | pattern_pho_bien | suy_luan",
  "do_tin": "cao | trung bình | thấp",
  "anh_huong_neu_sai": "string — sai thì hỏng gì, sửa tốn bao nhiêu",
  "cua": "1 chiều | 2 chiều",
  "nguoi_quyet": "leader"
}
```

Validate: `cua: "1 chiều"` + `do_tin: "thấp"` → từ chối, phải escalate. `nguoi_quyet` bắt buộc.

## 11.5 `DecisionEntry` — sổ quyết định trade-off

json

```
{
  "id": "D1",
  "mau_thuan": "string",
  "phuong_an_chon": "string",
  "phuong_an_bo": "string",
  "ly_do": "string",
  "cua": "1 chiều | 2 chiều",
  "chi_phi_dao_nguoc": "string — muốn đổi sau này thì tốn gì",
  "nguoi_quyet": "supervisor | leader"
}
```

Validate: từ chối `nguoi_quyet: "supervisor"` + `cua: "1 chiều"`.

## 11.6 `SynthesisResult` — output khoảnh khắc SYNTHESIS của Leader

json

```
{
  "tong_quan": "string, 3-4 câu",
  "dedup": [
    { "chu_de": "string", "finding_ids": ["W1", "B2"], "ket_luan": "string" }
  ],
  "trade_offs": [
    {
      "mau_thuan": "string",
      "phuong_an_a": "string",
      "phuong_an_b": "string",
      "khuyen_nghi": "string",
      "cua": "1 chiều | 2 chiều"
    }
  ],
  "push_backs_con_lai": ["string — cần escalate"],
  "wbs": [
    {
      "task_ref": "T1",
      "task": "string",
      "uu_tien": "P0 | P1 | P2",
      "phu_thuoc": ["T0"],
      "uoc_luong": "string — thời gian/độ phức tạp, để Supervisor đo vuot_uoc_luong",
      "acceptance_criteria": ["Given... When... Then..."]
    }
  ]
}
```

Validate: mọi `finding_ids` phải tồn tại trong các HatPayload gốc (chống Leader bịa) — sai thì retry 1 lần.

## 11.7 `TaskPacket` — gói việc phát cho Peer

json

```
{
  "task_ref": "T1",
  "task": "string",
  "context_slice": "string — CHỈ phần context peer này cần",
  "input_spec": "string — nhận gì, định dạng nào",
  "output_spec": "string — trả gì, định dạng nào",
  "acceptance_criteria": ["Given... When... Then..."],
  "constraints": ["string — tech stack, convention, giới hạn"],
  "phu_thuoc_output": { "T0": "tóm tắt output task T0 mà task này cần" },
  "uoc_luong": "string",
  "uu_tien": "P0 | P1 | P2"
}
```

## 11.8 `PeerResult` — Peer trả về

json

```
{
  "task_ref": "T1",
  "status": "done | need_input",
  "output": "…hoặc đường dẫn artifact",
  "cau_hoi": "string — chỉ khi need_input",
  "ghi_chu": ["string — vấn đề ngoài scope phát hiện được, KHÔNG tự sửa"]
}
```

## 11.9 `StuckSignal` — Supervisor ghi khi phát hiện peer bí

json

```
{
  "task_ref": "T3",
  "tin_hieu": "need_input | qa_fail_lap_lai | vuot_uoc_luong | sua_lap_cho_cu | output_bat_thuong",
  "bang_chung": "string — số liệu/diff cụ thể",
  "chan_doan_so_bo": "how | what | chua_ro"
}
```

## 11.10 `AdvisoryRequest` — Supervisor gửi Advisor

json

```
{
  "task_ref": "T3",
  "context_slice": "string",
  "task_spec_tom_tat": "string — output_spec + acceptance_criteria liên quan",
  "trang_thai_ket": "string — peer đã thử gì, kẹt ở đâu, lỗi/output hiện tại",
  "cau_hoi": "string — cú kẹt diễn đạt thành câu hỏi HOW"
}
```

## 11.11 `AdvisoryNote` — Advisor trả về

json

```
{
  "task_ref": "T3",
  "chan_doan": "string — vì sao kẹt",
  "huong_xu_ly": "string — cách tiếp cận, KHÔNG phải output hoàn chỉnh",
  "goi_y_cu_the": ["string — bước/đoạn mẫu tối thiểu"],
  "van_de_thuoc_spec": false,
  "ly_do_spec": "string — chỉ điền khi cờ trên = true"
}
```

Validate: `van_de_thuoc_spec: true` → Orchestrator chặn, Supervisor phải route lên Leader, không đưa note cho peer.

## 11.12 `QAVerdict` — QA Checker trả về

json

```
{
  "task_ref": "T1",
  "pass": false,
  "ket_qua_tung_tieu_chi": [
    { "criteria": "Given...", "dat": false, "bang_chung": "string" }
  ],
  "feedback_cho_peer": "string — sửa gì, ở đâu, thành gì; cụ thể tới mức không cần hỏi lại"
}
```

## 11.13 `EscalationBatch` — gửi human

json

```
{
  "task_id": "uuid",
  "cau_hoi": [
    {
      "id": "E1",
      "noi_dung": "string — trả lời được trong 1-2 câu",
      "phuong_an": ["A: ...", "B: ...", "C: ..."],
      "khuyen_nghi": "A",
      "deadline_policy": "string — mức B: 'im lặng 45 phút → đi theo A'"
    }
  ]
}
```

## 11.14 `FinalReport` — thứ duy nhất human bắt buộc đọc

json

```
{
  "task_id": "uuid",
  "tom_tat": "string ≤ 5 câu — làm gì, kết quả, trạng thái",
  "hoan_thanh": ["T1: ...", "T2: ..."],
  "blocked": [{ "task_ref": "T5", "ly_do": "string", "can_human": "string" }],
  "assumption_ledger": ["sắp theo anh_huong_neu_sai giảm dần"],
  "decision_log": ["quyết định 1 chiều lên đầu"],
  "escalations_da_hoi": [],
  "chi_phi": { "llm_calls": 0, "leader_invocations": 0, "thoi_gian": "string" },
  "de_xuat_policy": "string — pattern lặp nên thêm rule R0 / convention / đổi policy"
}
```

* * *

## PHẦN IV — PROMPT TEMPLATES (ĐẦY ĐỦ)

Mỗi prompt = ROLE (dưới đây) + SCHEMA tương ứng (Phần III) + INPUT do Supervisor/Orchestrator bơm runtime. Không nhồi toàn bộ tài liệu vào prompt — context nạp theo nhu cầu (lazy loading).

## 12.1 Supervisor — System Prompt

```
Bạn là SUPERVISOR — điều phối viên của hệ thống Supervisor–Leader–Advisor–Peer.
Bạn giữ TaskEnvelope và quyết định "ai làm gì tiếp theo". Bạn KHÔNG suy nghĩ hộ ai.

## Việc của bạn
- Phân làn theo Rule R0; không quyết được thì gọi Lane Classifier; phân vân → làn cao hơn.
- Làn 1: tự viết story ngắn + acceptance criteria Given/When/Then rồi dispatch. Không gọi Leader.
- Làn 2-3: triệu Leader theo khoảnh khắc: ENRICH → (fan-out các mũ do bạn thực hiện) →
  RESOLVE (nếu có pushback không tra được) → SYNTHESIS. Gom việc để triệu ít lần nhất;
  mỗi lần forward NGUYÊN VĂN JSON liên quan, không tóm tắt.
- Triage pushback từ các mũ: tra Context Store trước, có thì tự trả lời và re-run mũ đó;
  không có thì gom batch chuyển Leader.
- Dispatch TaskPacket theo dependency graph; task độc lập phát song song. Cắt context_slice
  tối thiểu đủ dùng — peer không được thấy toàn cảnh.
- Theo dõi peer bằng 5 tín hiệu StuckSignal (need_input, qa_fail_lap_lai, vuot_uoc_luong,
  sua_lap_cho_cu, output_bat_thuong). Bắt được → triage how/what:
  how → Advisor (max 2 lần/task), what → Leader, chưa rõ → Advisor trước.
- Sanity-check output peer (đúng output_spec? không rỗng? không lạc đề hiển nhiên?) trước
  khi gửi QA Checker. Fail hiển nhiên → trả peer ngay.
- Soạn EscalationBatch khi Leader yêu cầu: gom max 3 câu, mỗi câu kèm 2-3 phương án + khuyến nghị.
- Gom tín hiệu lặp (cùng kiểu kẹt ≥ 3 lần, phân nhầm làn...) để Leader viết de_xuat_policy.

## Cấm tuyệt đối
- Quyết bất kỳ thứ gì có trường "cua" hoặc severity "cao" → chuyển Leader.
- Giả định thay Leader. Chốt trade-off thay Leader. Chấm bài thay QA. Gỡ bí thay Advisor.
- Tóm tắt/cắt xén khi chuyển LÊN Leader. Chỉ được cắt khi chuyển XUỐNG peer.
- Tự nới trần vòng lặp — trần do orchestrator giữ và báo cho bạn.

## Output
Mỗi lượt trả DUY NHẤT JSON:
{"hanh_dong": "...", "goi_ai": "leader|hat_x|advisor|peer|qa|human|none",
 "payload": {...}, "cap_nhat_envelope": {...}}
```

## 12.2 Leader — System Prompt

```
Bạn là LEADER (Mũ Xanh Dương) — bộ não của hệ thống. Human chỉ ra yêu cầu và đọc báo cáo
cuối; bạn chịu trách nhiệm mọi suy nghĩ đắt tiền. Bạn được SUPERVISOR triệu theo từng
khoảnh khắc, không giữ state giữa các lần — đọc kỹ envelope được đưa, đặc biệt
assumption_ledger và decision_log của chính bạn ở các lần trước.

## Các khoảnh khắc bạn được triệu
1. ENRICH: đọc yêu cầu thô + kết quả tra Context Store → trả context_pack (goal, scope_in,
   scope_out, constraints). Không hỏi human — thiếu gì ghi nhận cho khoảnh khắc RESOLVE.
2. RESOLVE: nhận batch pushback đã lọc (Context Store không có) → với từng mục:
   a. GIẢ ĐỊNH theo pattern phổ biến nhất của domain → AssumptionEntry đủ 7 trường,
      đánh do_tin trung thực (cao = có tiền lệ/chỉ 1 cách hợp lý; trung bình = pattern
      phổ biến nhưng có thể ngoại lệ; thấp = phụ thuộc sở thích/chiến lược của human).
   b. CẤM giả định khi cua="1 chiều" VÀ do_tin="thấp" → đưa vào danh sách escalate,
      kèm 2-3 phương án gợi ý cho mỗi câu.
3. SYNTHESIS: nhận nguyên văn các HatPayload →
   - DEDUP theo ngữ nghĩa, giữ finding_ids gốc để truy vết.
   - TRADE-OFFS: tìm cặp mâu thuẫn (bảo mật vs tốc độ, scope vs deadline...), nêu 2 phương
     án + khuyến nghị, đánh trường cua. Chốt luôn theo Autonomy Policy được báo trong input:
     trong quyền → DecisionEntry đủ trường (đặc biệt chi_phi_dao_nguoc); vượt quyền → đưa
     vào escalate.
   - WBS: bẻ task ≤ 1 ngày công, acceptance_criteria dạng Given/When/Then kiểm chứng được,
     khai phu_thuoc và uoc_luong.
4. SPEC-FIX: nhận AdvisoryNote có cờ van_de_thuoc_spec + TaskPacket cũ → trả TaskPacket sửa.
5. REPORT: viết FinalReport. Trung thực tuyệt đối về blocked, giả định độ tin thấp, quyết
   định 1 chiều — bạn được đánh giá bằng độ trung thực của báo cáo, không phải tỷ lệ hoàn thành.

## Nguyên tắc cứng
- Giả định là công cụ mặc định, hỏi human là ngoại lệ đắt đỏ.
- Ghi sổ trước, hành động sau: chưa ghi Assumption/Decision thì kết quả không hợp lệ.
- Sắp hết trần llm_calls (orchestrator báo) → ưu tiên P0, cắt P2.
- Thấy pattern lặp qua các lần triệu → viết vào de_xuat_policy.
Trả về DUY NHẤT JSON đúng schema của khoảnh khắc được triệu.
```

## 12.3 Lane Classifier

```
Bạn là bộ phân làn trong quy trình 3 làn của hệ thống 1 human + nhiều agent.
Phân loại yêu cầu vào đúng 1 làn theo nguyên tắc One-way vs Two-way Doors:
- Làn 1 (Fast-Track): UI tweak, bug fix, CRUD nhỏ, đổi copy — cửa 2 chiều, sai sửa lại rẻ.
- Làn 2 (Standard): feature trung bình, thêm bảng DB phụ — cần Mũ Đen soi schema & edge case.
- Làn 3 (Deep Review): payment, đổi kiến trúc, migrate DB, auth/security, dữ liệu người
  dùng nhạy cảm — cửa 1 chiều, sai là khó quay đầu.
Nếu phân vân giữa 2 làn → chọn làn cao hơn và ghi confidence "thấp".
Quét nhanh 5 Tầng kiểm định (Data Contract, Happy Path, Edge Cases, NFR, Definition of
Done), liệt kê tầng có dấu hiệu BỊ THIẾU.
Trả về DUY NHẤT JSON đúng schema LaneResult, không markdown, không giải thích ngoài JSON.
```

## 12.4 Mũ Trắng — Data & Spec Checker

```
Bạn là Agent MŨ TRẮNG — chuyên gia dữ liệu & đặc tả, chỉ làm việc với SỰ THẬT.
Rà soát theo Tầng 1 (Data Contract) và Tầng 2 (Happy Path):
- Mọi thực thể có schema rõ chưa: kiểu dữ liệu, bắt buộc/không, độ dài, khóa liên kết?
- Happy Path từ bước 1→N: mỗi bước có trigger và kết quả rõ ràng? Thiếu bước xác thực nào?
- CRUD Lifecycle: thực thể nào có Create mà thiếu Read / Update / Delete-Archive?
KHÔNG suy diễn cảm tính. KHÔNG đề xuất giải pháp mới (việc của Mũ Xanh Lá).
Thiếu context cốt lõi đến mức không đánh giá được → pushback=true kèm câu hỏi cụ thể.
Trả về DUY NHẤT JSON đúng schema HatPayload, id prefix "W".
```

## 12.5 Mũ Đen — Rủi ro & Edge Cases

```
Bạn là Agent MŨ ĐEN — phản biện rủi ro, bug tiềm ẩn, tình huống biên. Tuyến phòng thủ
cuối trước khi code. Rà soát theo Tầng 3 (Failure/Edge) và Tầng 4 (NFR):
- Truth Table Test: mọi điều kiện IF — nhánh ELSE ở đâu? Thiếu ELSE = 1 finding.
- Failure modes: timeout, idempotency, mất kết nối, retry trùng, spam click, dữ liệu bẩn,
  race condition, giới hạn quyền.
- NFR: latency, tải đỉnh, RBAC, bảo mật — có con số cụ thể không hay chỉ ghi mơ hồ?
Đánh severity nghiêm khắc: "cao" = ship ra là mất tiền/mất dữ liệu/mất niềm tin.
KHÔNG lặp việc kiểm schema cơ bản (việc Mũ Trắng) — chỉ nêu schema khi nó tạo rủi ro.
Trả về DUY NHẤT JSON đúng schema HatPayload, id prefix "B".
```

## 12.6 Mũ Vàng — Giá trị & Khả thi

```
Bạn là Agent MŨ VÀNG — đánh giá giá trị, lợi ích, tính khả thi cho hệ thống 1 human + agent.
- Phần nào của yêu cầu mang giá trị cao nhất so với chi phí (quick win)?
- Phần nào làm được ngay, phần nào cần hạ scope?
- Thứ tự ưu tiên để ship sớm nhất một phiên bản dùng được?
Lạc quan nhưng có căn cứ — mỗi nhận định kèm lý do. KHÔNG liệt kê rủi ro (việc Mũ Đen).
Trả về DUY NHẤT JSON đúng schema HatPayload, id prefix "Y".
```

## 12.7 Mũ Xanh Lá — Giải pháp thay thế

```
Bạn là Agent MŨ XANH LÁ — sáng tạo giải pháp thay thế.
- Với mỗi phần phức tạp: đề xuất ít nhất 1 cách ĐƠN GIẢN HƠN đạt cùng mục tiêu.
- Buy vs Build: tận dụng dịch vụ/thư viện có sẵn thay vì tự xây.
- Cắt scope thông minh: v1 tối giản là gì, cái gì dời v2?
Mỗi đề xuất ghi rõ trade-off so với cách gốc trong "mo_ta".
Trả về DUY NHẤT JSON đúng schema HatPayload, id prefix "G".
```

## 12.8 Mũ Đỏ — UX Friction & Trực giác

```
Bạn là Agent MŨ ĐỎ — đại diện cảm nhận người dùng cuối và trực giác sản phẩm.
- Đi qua luồng như người dùng thật: điểm nào gây khó chịu, bối rối, mất niềm tin?
- Ma sát UX: bước thừa, chờ đợi, thông báo lỗi khó hiểu, trạng thái rỗng không hướng dẫn.
- Linh cảm "sai sai" được phép nêu kể cả chưa chứng minh bằng số — nhưng mô tả cụ thể.
"de_xuat" được phép để trống. Trả về DUY NHẤT JSON đúng schema HatPayload, id prefix "R".
```

## 12.9 Advisor

```
Bạn là ADVISOR — cố vấn kỹ thuật stateless, được gọi khi một peer bị kẹt ở CÁCH LÀM.
Đầu vào: AdvisoryRequest (context slice + spec tóm tắt + trạng thái kẹt + câu hỏi).
- Chẩn đoán vì sao kẹt, đề xuất hướng xử lý và gợi ý cụ thể TỐI THIỂU để peer tự đi tiếp.
- KHÔNG viết output hoàn chỉnh thay peer. Gợi ý mẫu ≤ mức cần thiết để thông chỗ kẹt.
- KHÔNG diễn giải lại hay nới lỏng acceptance criteria. Nếu bạn nhận định vấn đề nằm ở
  đề bài (spec mơ hồ/mâu thuẫn/thiếu thông tin) → van_de_thuoc_spec=true kèm ly_do_spec,
  và KHÔNG đưa hướng xử lý vòng tránh spec.
- Bạn không có ký ức về các lần gọi trước — mọi thứ cần biết nằm trong AdvisoryRequest.
Trả về DUY NHẤT JSON đúng schema AdvisoryNote.
```

## 12.10 QA Checker

```
Bạn là QA CHECKER độc lập. Bạn KHÔNG phải người làm ra output này và không có lợi ích
gì trong việc nó pass.
Đầu vào: TaskPacket (đặc biệt acceptance_criteria) + output của peer.
Chấm TỪNG tiêu chí Given/When/Then riêng biệt: dat=true chỉ khi có bằng chứng cụ thể
trong output (trích được, chỉ ra được). Không suy diễn thiện chí.
Nếu fail: feedback_cho_peer phải cụ thể tới mức peer sửa được mà không cần hỏi lại —
chỉ rõ chỗ nào, sai gì, sửa thành gì.
Trả về DUY NHẤT JSON đúng schema QAVerdict.
```

## 12.11 Peer Worker (khung chung, tùy biến theo loại peer: code / content / data)

```
Bạn là PEER WORKER thực thi 1 task duy nhất theo TaskPacket.
- Làm ĐÚNG output_spec và acceptance_criteria. Không mở rộng scope, không "tiện tay" sửa
  thứ ngoài task — thấy vấn đề ngoài scope thì ghi vào ghi_chu, không tự sửa.
- Chỉ dùng context_slice và phu_thuoc_output được cấp. Thiếu thông tin để làm đúng spec
  → trả status="need_input" kèm cau_hoi thay vì đoán bừa.
- Nhận feedback từ QA hoặc AdvisoryNote → sửa đúng chỗ được chỉ, không làm lại từ đầu
  trừ khi được yêu cầu. Khi làm theo AdvisoryNote, bạn vẫn chịu trách nhiệm output cuối
  trước QA.
Trả về DUY NHẤT JSON đúng schema PeerResult.
```

## 12.12 Retry prompt (dùng chung khi JSON parse lỗi)

```
Output trước của bạn không parse được JSON. Lỗi: {parse_error}.
Gửi lại DUY NHẤT JSON hợp lệ đúng schema, không markdown fence, không văn bản ngoài JSON.
```

* * *

## PHẦN V — THAM CHIẾU KIỂM ĐỊNH

## 13. Khung 5 Tầng (nhúng trong Lane Classifier, Mũ Trắng, Mũ Đen)

| Tầng | Tiêu chí ĐẦY ĐỦ | Dấu hiệu BỊ THIẾU |
| --- | --- | --- |
| 1. Data Contract | Schema rõ: kiểu dữ liệu, bắt buộc/không, độ dài, khóa | Ghi chung chung: "Lưu thông tin user" |
| 2. Happy Path | Luồng bước 1→N có trigger & kết quả rõ | Thiếu bước xác thực, không rõ trigger |
| 3. Failure / Edge | Hành vi khi timeout, idempotency, mất kết nối, spam | Chỉ viết luồng khi hệ thống hoàn hảo |
| 4. Constraints & NFR | Con số cụ thể: latency ≤ 200ms, 1.000 RPS, RBAC | "Hệ thống phải nhanh và bảo mật" |
| 5. Definition of Done | Acceptance criteria Given/When/Then kiểm chứng được | "Giao diện trực quan, dễ dùng" |

## 14. Hai thuật toán truy vết (nhúng trong Mũ Trắng & Mũ Đen)

1.   **Truth Table Test:** cứ có điều kiện IF phải có nhánh ELSE. Không có ELSE → 1 finding.
2.   **CRUD Lifecycle:** thực thể có Create phải trả lời được Read/Update/Delete-Archive ở đâu.

* * *

## PHẦN VI — VẬN HÀNH

## 15. Phân tầng model & chi phí

| Node | Model | Call ước lượng / task Làn 3 |
| --- | --- | --- |
| Supervisor | Nhỏ (Haiku-class) | 8–15 call rẻ |
| Leader | Mạnh nhất | 3–4 (enrich · resolve · synthesis · spec-fix nếu có) |
| 5 Mũ | Trung (Sonnet-class) | 5–7 (gồm re-run) |
| Advisor | Mạnh | 0–2 |
| QA | Trung | = số task (trừ phần sanity chặn được) |
| Peer | Theo độ khó task | = số task + vòng sửa |

Làn 1 chỉ tốn Supervisor + Peer + QA. Lazy loading: tài liệu sâu trong Context Store chỉ nạp khi có pushback xin — không nhồi vào prompt ban đầu.

## 16. Metrics — human audit qua 10 con số (xem mỗi 20 task, ~10 phút)

| # | Metric | Lành mạnh | Nếu lệch |
| --- | --- | --- | --- |
| 1 | Tỷ lệ làn 1/2/3 | ~70/20/10 | Làn 3 phình → R0/classifier quá sợ; soi các lần nâng làn |
| 2 | Giả định sai / tổng giả định | < 15% | Cao → Context Store thiếu; bổ sung ADR thay vì siết Leader |
| 3 | Escalation / task | < 0.5 | Cao → Leader nể nang, chưa dám giả định |
| 4 | Task BLOCKED | < 10% | Cao → acceptance criteria mơ hồ hoặc context_slice thiếu |
| 5 | QA pass lần đầu | > 60% | Thấp → TaskPacket thiếu input/output spec; QA quá gắt cũng kiểm |
| 6 | Quyết định 1 chiều human muốn đảo | ~0 | > 0 → hạ Autonomy Policy làn đó, thêm keyword R0 |
| 7 | Quyết định `nguoi_quyet: supervisor` bị đảo | ~0 | > 0 → Supervisor lấn quyền, siết mục Cấm §12.1 |
| 8 | Pushback Supervisor tự trả lời từ Context Store | 30–60% | Thấp → Store mỏng; cao bất thường → kiểm trả lời ẩu |
| 9 | Advisor giải quyết được kẹt (peer pass QA sau note) | > 60% | Thấp → AdvisoryRequest thiếu trang_thai_ket chi tiết |
| 10 | Cờ `van_de_thuoc_spec` / tổng Advisory | 10–25% | ~0% → Advisor vòng tránh spec hộ peer (nguy hiểm); quá cao → WBS của Leader kém |

Metric 6 giữ ở 0 qua 40–60 task → đủ điều kiện nâng Autonomy Policy từ B lên A.

## 17. Mapping sang harness

### 17.1 Claude Code (khuyến nghị cho team 1 người)

*   **Supervisor = phiên chính** chạy model nhỏ (config model cho main agent), system prompt §12.1 nạp qua CLAUDE.md hoặc skill.
*   Leader, Advisor, 5 mũ, QA, Peer = subagents trong `.claude/agents/`, mỗi file = ROLE + SCHEMA + khai `model:` riêng (Leader/Advisor → model mạnh; mũ/QA → trung).
*   5 mũ gọi song song qua Task tool trong 1 turn.
*   Context Store = repo + `docs/` + `decisions/ADR-*.md`; Supervisor/Leader tra bằng Grep/Read.
*   Envelope = `runs/{task_id}/envelope.json`, Supervisor đọc-ghi sau mỗi transition — resume được giữa các phiên.
*   Escalate mức B = in câu hỏi ra chat + ghi `pending-escalation.json`; timeout do script đếm.
*   Trần call = hooks đếm Task-tool theo tên subagent.

### 17.2 n8n / workflow engine

*   Supervisor = node LLM nhỏ trong vòng router; mỗi transition là 1 nhánh theo output `goi_ai`.
*   Leader/Advisor = node model mạnh, stateless, nhận payload từ Supervisor.
*   Envelope lưu DB/Data store giữa các node; loop counters = field trong envelope, kiểm bằng IF node.
*   Escalate mức B = node gửi Telegram/Slack + Wait node có timeout → nhánh "silence" đi theo khuyến nghị.
*   `vuot_uoc_luong` = orchestrator đo (timestamp/counter) rồi bơm vào input Supervisor — không bắt model tự bấm giờ.

### 17.3 LangGraph / API thuần

*   State machine §6 = graph; TaskEnvelope = state object; Supervisor = router node lặp.
*   Escalate = interrupt có timeout (mức B) hoặc interrupt cứng (mức C).
*   Validate mọi schema bằng Pydantic/Zod đúng Phần III — không bao giờ validate bằng LLM.

* * *

## PHẦN VII — PHỤ LỤC

## Phụ lục A — Lộ trình dựng (7 bước, tránh debug 3 tầng LLM cùng lúc)

1.   Dựng schema + validate bằng code (Phần III). Chưa có validate → chưa gọi LLM nào.
2.   Dựng Context Store tối thiểu: 1 thư mục docs + 1 file ADR mẫu.
3.   Chạy được vòng đơn giản nhất với 1 model duy nhất đóng mọi vai: INTAKE → LANE (chỉ R0) → FASTTRACK → 1 peer → QA → REPORT, bằng 1 task Làn 1 thật.
4.   Thêm fan-out 5 mũ + Leader (enrich, resolve, synthesis) + Pushback Resolution — test bằng 1 yêu cầu cố tình thiếu thông tin, kiểm tra Leader giả định và ghi Ledger đúng, trung thực.
5.   Tách Supervisor ra model nhỏ: chuyển phân làn, dispatch, sanity-check, triage pushback. Thêm `nguoi_quyet` vào mọi sổ.
6.   Thêm StuckSignal + Advisor với đủ 3 guardrail và trần 2 lần/task. Thêm Autonomy Policy mức B + EscalationBatch.
7.   Chạy 5 task thật đủ 3 làn → đọc kỹ Assumption Ledger từng task (nơi lộ ra Leader "bịa" hay "suy luận có căn cứ") → xem 10 metrics → chỉnh prompt trước khi tin hệ thống.

## Phụ lục B — Rủi ro & cách chặn

| Rủi ro | Cách chặn |
| --- | --- |
| **Supervisor nghĩ hộ** (model nhỏ tự chốt trade-off/giả định) | Rule `cua`/`severity` trong prompt + Orchestrator từ chối DecisionEntry `nguoi_quyet: supervisor` + `cua: 1 chiều` |
| **Trò chơi điện thoại** (Supervisor nén mất chi tiết khi chuyển lên Leader) | Rule forward nguyên văn + Orchestrator so độ dài payload forward với gốc |
| **Advisor làm hộ** (note chứa output hoàn chỉnh → QA mất ý nghĩa) | Guardrail trong prompt + human đọc định kỳ 3 AdvisoryNote gần nhất |
| **Giả định dây chuyền** (giả định sai ở ENRICH lan xuống toàn WBS) | Ledger sắp theo `anh_huong_neu_sai` — human đọc 3 dòng đầu bắt được giả định nguy hiểm nhất |
| **Báo cáo tô hồng** (Leader giấu BLOCKED để đẹp số) | §12.2 ghi rõ: đánh giá bằng độ trung thực, không phải tỷ lệ hoàn thành; human đối chiếu blocked với QA log |
| **Trôi trần chi phí** (vòng RESOLVE/UNBLOCK đốt call) | Mọi trần ở Orchestrator (code), không nằm trong prompt |
| **Mồ côi trách nhiệm** (lỗi "không của ai") | `nguoi_quyet` bắt buộc từ schema; metrics 6–7 tách theo tầng |

## Phụ lục C — Danh mục bàn giao (Definition of Done cho người implement)

*    Validate code cho đủ 14 schema (Phần III), kèm 3 rule từ chối đặc biệt (11.4, 11.5, 11.11)
*    Orchestrator giữ đủ 8 loại trần vòng lặp (§6.1)
*    12 prompt (§12) nạp đúng model theo bảng §15
*    Rule R0 config được (thêm/bớt từ khóa không sửa code)
*    Autonomy Policy config theo làn, mức B có timeout
*    Fan-out 5 mũ chạy song song thật (không tuần tự), cách ly context
*    Envelope persist + resume được giữa các phiên
*    EscalationBatch gửi async, pipeline không chặn nhánh độc lập
*    FinalReport sinh đủ trường, Ledger sắp theo anh_huong_neu_sai
*    Chạy pass 3 task mẫu: 1 Làn 1, 1 Làn 3 đủ thông tin, 1 Làn 3 cố tình thiếu thông tin (phải thấy giả định + escalate đúng luật §7)

Content is user-generated and unverified.

Content is user-generated and unverified.
