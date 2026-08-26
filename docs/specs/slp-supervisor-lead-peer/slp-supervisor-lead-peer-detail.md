# SLP: Supervisor – Lead – Peer

*Cẩm nang xây dựng hệ AI agent theo mô hình ba vai trò — tổng hợp từ session thực chiến, đối chiếu với nghiên cứu, và lộ trình từ người mới đến chuyên gia*

**Phiên bản 2** — biên soạn lại từ bản 1 sau khi tiếp nhận transcript session "SLP – Supervisor Lead Peer". Bản này thay đổi căn bản cách tiếp cận: bản 1 trình bày Supervisor / Hierarchical / Peer như ba *topology* (ba cách nối dây giữa các agent); bản này trình bày SLP như ba *vai trò* trong một hệ thống duy nhất, đúng theo tinh thần của session. Hai góc nhìn không mâu thuẫn — topology là bộ xương, vai trò là linh hồn — nhưng vai trò mới là thứ quyết định hệ thống của bạn chạy tốt hay tệ. Phụ lục C đối chiếu chi tiết hai cách nhìn.

---

## Cách dùng cuốn sách này

Phần I xây hai nền tảng nhận thức mà toàn bộ mô hình SLP đứng trên: kinh tế học attention của LLM, và tính ngẫu nhiên của đầu ra. Không nắm hai điều này, mọi kỹ thuật ở phần sau đều thành công thức học vẹt. Phần II là trọng tâm theo đúng yêu cầu của bạn: ba chương, mỗi chương trả lời một câu hỏi — làm sao xây một Peer tốt, một Lead tốt, một Supervisor tốt. Phần III dạy vận hành: thang leo quyết định, khi nào dùng SLP, cải tiến hằng tuần, và triết lý hạ tầng. Phần IV là thực hành theo ba mức độ, trong đó mức một không cần bất kỳ công cụ nào ngoài hai cửa sổ chat.

Quy ước: những chỗ ghi *[Session]* là ý được tổng hợp trực tiếp từ transcript; những chỗ ghi tên tác giả và năm (ví dụ Du 2023) là kiến thức từ nghiên cứu công bố, được đưa vào để bạn hiểu *vì sao* các kinh nghiệm trong session lại đúng.

---

# PHẦN I — HAI NỀN TẢNG NHẬN THỨC

## Chương 1: Kinh tế học attention — lỗi của model thường không phải là thiếu năng lực

### Quan sát khởi điểm

*[Session]* Mở đầu session là một quan sát tinh tế về hiện tượng ai dùng AI viết code cũng gặp: model viết một bài test dở, nhưng khi được hỏi lại "mày có đang vi phạm anti-pattern nào khi viết unit test không?" thì nó *tự nhận ra* lỗi. Nghĩa là năng lực nhận diện lỗi vốn có sẵn trong model. Vấn đề không phải model "ngu" — vấn đề là tại thời điểm viết, nó chưa phân bổ đủ năng lực tính toán và attention cho câu hỏi "test này viết thế nào cho đúng". Một câu hỏi đúng lúc làm model *tái phân bổ* compute vào đúng chỗ dễ sai, và chất lượng lập tức khác.

Đây là tiền đề số một của toàn bộ mô hình SLP: **phần lớn lỗi của agent là lỗi phân bổ attention, và thứ hệ thống cần không phải là một model to hơn, mà là một cơ chế tái phân bổ attention đúng thời điểm.** Vai trò Supervisor sinh ra từ chính tiền đề này.

### Vì sao câu hỏi phải mở, không được dẫn dắt

*[Session]* Điểm tinh tế thứ hai: cách hỏi quyết định tất cả. Nếu Supervisor *khẳng định* "mày đang vi phạm anti-pattern", con agent đang cầm write scope — vốn có xu hướng muốn làm hài lòng người hỏi — sẽ cố *tìm ra bằng được* một lỗi nào đó để nhận, kể cả khi không có. Câu hỏi mở ("mày có đang vi phạm anti-pattern nào không?") trung tính hơn hẳn: nó kích hoạt việc tự rà soát mà không áp đặt kết luận.

Nghiên cứu xác nhận trực giác này từ nhiều phía. Sharma và cộng sự (2023) ghi nhận hiện tượng **sycophancy** — model tìm kiếm sự tán đồng của người dùng theo những cách không mong muốn, sẵn sàng đổi câu trả lời theo ý kiến được gợi trong prompt. Các nghiên cứu về "certainty robustness" cho thấy chỉ một câu "Are you sure?" cũng đủ khiến nhiều model *bỏ câu trả lời đúng* để đổi sang sai — tức là ngay cả áp lực hội thoại nhẹ nhất cũng bẻ cong được model. Và dòng nghiên cứu về mặt tối của self-correction (Zhang và cộng sự, 2024) chỉ ra rằng tự-sửa-lỗi nội tại có thể khiến model dao động, nhiễm thiên kiến từ chính prompt sửa lỗi, thậm chí mắc các lỗi kiểu con người như overthinking. Bài học thực hành rút ra: **can thiệp của Supervisor phải nhỏ, trung tính, dạng câu hỏi mở; mọi khẳng định có định hướng đều là thuốc độc trộn trong thuốc bổ.**

### Bottleneck thật sự là attention của con người

*[Session]* Session đẩy ý tưởng lên một tầng nữa: trong kỷ nguyên agent, nút thắt cổ chai không còn là tốc độ code mà là **human attention** — bạn nên chú ý vào điều gì, lúc nào. Người làm bảy tám dự án song song không thể ngồi canh từng prompt. Hệ SLP tồn tại để con người tham gia vào vòng lặp một cách *rời rạc* và *đúng chỗ*: đi ngủ, để Supervisor theo dõi, sáng dậy nghe một bản báo cáo voice tổng hợp những gì cần chú ý ("trong lúc anh ngủ, thằng này lỡ chạy hai test song song tạo flaky test..."). Nói cách khác, SLP là một hệ thống quản trị attention hai tầng: Supervisor quản trị attention của các agent, và quản trị luôn attention của con người.

### Câu hỏi tự kiểm tra chương 1

1. Phân biệt "model không đủ năng lực viết test đúng" với "model chưa phân bổ attention cho việc viết test đúng". Hai chẩn đoán này dẫn tới hai giải pháp khác nhau thế nào?
2. Vì sao "mày vừa vi phạm contract của tao" nguy hiểm hơn "mày có vừa vi phạm contract nào của tao không?" Gọi tên hiện tượng nghiên cứu tương ứng.
3. Tự thí nghiệm: lấy một đoạn code AI viết cho bạn gần đây, đặt một câu hỏi mở kiểu "có anti-pattern nào trong này không?" và quan sát nó tìm ra gì. Sau đó thử phiên bản dẫn dắt "đoạn này sai ở dòng X đúng không?" và so sánh hành vi.

---

## Chương 2: Tính ngẫu nhiên và sức mạnh của thiết kế mù (blind design)

### LLM là máy xổ số có học thức

*[Session]* Tiền đề thứ hai: LLM ngẫu nhiên một cách sâu sắc — cùng một prompt, các session khác nhau cho ra các attention khác nhau, lời giải khác nhau. Với bài toán đóng (một đáp án đúng), điều này là nhiễu. Nhưng với bài toán mở — kiểu "thiết kế cơ chế đồng bộ trạng thái cho game multiplayer", nơi thị trường có nhiều lời giải đều hợp lệ (đồng bộ chặt cho HP trong party, event-driven cho trạng thái stun/knockback, mỗi thể loại game một trường phái) — tính ngẫu nhiên là *tài nguyên*: mỗi lần lấy mẫu là một góc nhìn.

Đây là lý do "người ta thích best-of-N": sinh N phương án độc lập rồi chọn, thay vì đặt cược vào một lần sinh duy nhất.

### Dual-Lane / Three-Lane Design — và chữ "blind" quan trọng nhất

*[Session]* Kỹ thuật trung tâm của Lead trong SLP: với quyết định thiết kế hệ trọng, Lead mở 2–3 "lane" — các session thiết kế độc lập. Điều kiện sống còn: **blind**. Lead *không* đưa framing của mình cho bất kỳ lane nào; các lane không biết gì về nhau, không biết Lead nghiêng về đâu; chúng chỉ nhận đề bài và thiết kế. Sau đó Lead mới hội tụ các phương án, và con người review kết quả hội tụ xem có khớp concept không.

Nghiên cứu về multi-agent debate soi sáng vì sao chữ "blind" quyết định thành bại. Du và cộng sự (2023) cho thấy nhiều instance LLM tranh luận qua nhiều vòng cải thiện độ chính xác và giảm hallucination — hướng "society of minds". Nhưng các nghiên cứu sau đó phát hiện mặt trái: **hiệu ứng conformity** — trong tranh luận, agent yếu có xu hướng bỏ phán đoán đúng của mình để theo số đông; một số phân tích còn cho thấy khi kiểm soát cách tổng hợp, tranh luận qua lại không hơn gì việc *trả lời độc lập rồi tổng hợp* — nút thắt thật nằm ở khâu trích xuất đáp án đúng từ các bất đồng. Đồng thời, dòng nghiên cứu về diversity of thought cho thấy sự đa dạng giữa các agent (model khác nhau, góc nhìn khác nhau) mới là thứ tạo giá trị. Ghép lại, ta có công thức mà session đã đến bằng kinh nghiệm và nghiên cứu đến bằng thí nghiệm: **độc lập khi sinh phương án (để giữ đa dạng, tránh conformity), hội tụ có chủ trì sau đó (để trích xuất được cái đúng).** Blind design chính là công thức đó.

### Hệ quả thiết kế

Ba hệ quả bạn sẽ gặp lại xuyên suốt sách. Một: đừng bao giờ để agent sinh phương án nhìn thấy "đáp án gợi ý" của cấp trên trước khi nó sinh xong — kể cả một câu bâng quơ cũng là framing. Hai: sự hội tụ cần một chủ thể chịu trách nhiệm chọn (Lead), không phải biểu quyết máy móc — số đông có thể cùng sai một kiểu. Ba: đa dạng đáng đồng tiền — nếu được, chạy các lane bằng model khác nhau hoặc ít nhất là session hoàn toàn tách biệt.

### Câu hỏi tự kiểm tra chương 2

1. Vì sao blind design đặc biệt giá trị với bài toán "nhiều lời giải đều hợp lệ" hơn là bài toán "một đáp án đúng"?
2. Hiệu ứng conformity phá hỏng multi-agent debate như thế nào, và blind design né nó bằng cách nào?
3. Nếu ba lane cho ra ba thiết kế gần giống hệt nhau, đó là tín hiệu tốt hay đáng ngờ? (Gợi ý: có hai cách đọc, hãy nêu cả hai.)

---
# PHẦN II — XÂY DỰNG BA VAI TRÒ

## Chương 3: Xây một Peer tốt

### Peer là gì trong SLP

Peer là agent trực tiếp làm việc — cầm write scope, thiết kế, viết code, viết test. Nhưng định nghĩa đó chưa chạm vào linh hồn của vai trò. *[Session]* Câu định nghĩa thật nằm ở đây: nếu bạn dùng sub-agent kiểu mặc định, "Sub Agent không bao giờ phản đối lại thằng Lead" — Lead đưa phương án A hoặc B, nó chọn A, B hoặc block, không bao giờ đưa ra phương án C. **Một Peer đúng nghĩa là một agent có năng lực độc lập: dám phản đối Lead khi Lead sai, dám đề xuất phương án C nằm ngoài khung được đưa.** Nếu Peer của bạn chưa từng phản đối Lead, thì theo lời session, "nó đang chưa đủ tốt".

### Ba năng lực phải xây cho Peer

**Năng lực thứ nhất: phản biện.** *[Session]* Tin tốt: không cần phức tạp. Một bản instruction khoảng 30–40 dòng, viết tử tế, là đủ để Peer có khả năng phản biện. Nội dung cốt lõi của instruction đó xoay quanh: mày là một kỹ sư độc lập, không phải một hàm thực thi; khi nhận phương án từ Lead, nhiệm vụ đầu tiên của mày là đánh giá nó bằng phán đoán chuyên môn của chính mày; nếu mày thấy phương án có vấn đề, mày có nghĩa vụ nói ra kèm lập luận; mày được phép và được khuyến khích đề xuất phương án ngoài khung nếu mày tin nó tốt hơn; đồng thuận dễ dãi là một hành vi thất bại, không phải hành vi hợp tác. Lưu ý cân bằng: phản biện không phải cãi cùn — instruction cũng cần dạy Peer *nhượng bộ khi lập luận đối phương mạnh hơn*, nếu không bạn tạo ra một agent bất trị thay vì một agent độc lập (nghiên cứu về certainty robustness gọi hai cực hỏng này là "đổi ý dưới áp lực" và "cứng đầu trước phản hồi hợp lệ" — cả hai đều phá lòng tin).

**Năng lực thứ hai: biết điều gì không phải việc của mình quyết.** *[Session]* Ví dụ đắt giá trong session: Peer đang implement tính năng đồng bộ trạng thái, gặp vấn đề bandwidth khi quá nhiều player trong một AoE. Muốn thỏa mãn requirement, nó có thể *tự ý* quantize hướng di chuyển từ int16 xuống int8 — một quyết định nghe có vẻ kỹ thuật vặt nhưng thực chất đổi cả trade-off của sản phẩm: sai lệch vị trí, server–client phải reconcile nhiều hơn, có thể lag hơn. Một Peer tốt phải *cảm nhận được* ranh giới: quyết định nào là chi tiết cài đặt (tự quyết), quyết định nào chạm vào design và requirement (phải đưa về Lead, thậm chí hội đồng lane hoặc con người). Hãy viết thẳng vào instruction các tín hiệu vượt ranh: thay đổi contract/API công khai, đánh đổi chất lượng dữ liệu hay trải nghiệm để đạt chỉ tiêu kỹ thuật, thêm phụ thuộc mới, bất kỳ điều gì khiến mày phải "bẻ" một yêu cầu để thỏa một yêu cầu khác.

**Năng lực thứ ba: né các anti-pattern đã biết.** *[Session]* Khi giao Peer viết test theo TDD, session khuyên cho nó một danh sách 10–20 gạch đầu dòng anti-pattern để né. Danh sách này dùng *chung* cho các project (đừng tối ưu quá mức theo từng dự án); cái nào đủ generic thì kết tinh thành skill hoặc instruction. Case study kinh điển về vì sao cần nó nằm ở Phụ lục A (hiện tượng test "mint" ra API).

### Checklist Peer tốt

Peer của bạn đạt chuẩn khi: đã từng phản đối Lead ít nhất một lần với lập luận xác đáng trong log thực tế; đã từng đề xuất phương án C ngoài khung A/B được giao; đã từng chủ động dừng lại hỏi "quyết định này có cần đưa về Lead không" trước một trade-off hệ trọng; và không tái phạm các anti-pattern có trong danh sách của bạn. Nếu sau hai tuần vận hành mà chưa thấy dấu hiệu nào trong bốn dấu hiệu trên, sửa instruction trước khi đổ lỗi cho model.

---

## Chương 4: Xây một Lead tốt

### Lead là bộ não thật, không phải bưu tá

*[Session]* Câu hỏi trong session: "Lead có bao giờ làm nhiệm vụ explore không hay chỉ ra decision?" — và câu trả lời dứt khoát: có chứ. Lead phải là một bộ não thực sự điều phối, có luồng suy nghĩ riêng, tự tìm hiểu đủ để ra quyết định tốt; thứ duy nhất Lead không làm là những việc thực thi nặng. Một Lead chỉ chuyển tiếp tin nhắn giữa human và Peer là một tầng trung gian vô dụng — bạn đang trả tiền token cho một bưu tá.

### Nghệ thuật số một của Lead: hỏi mà không framing

*[Session]* Đây là kỹ năng được nhấn mạnh nhiều nhất trong session, và là chỗ Lead hay hỏng nhất. Ba quy tắc:

Thứ nhất, **cấm câu hỏi Yes/No và cấm ép chọn A/B** khi bàn việc hệ trọng. Lead hỏi đóng thì Peer thành cái hàm: trả về A, B hoặc block, không bao giờ có C. Thay vào đó là câu hỏi mở: "với bài toán này, em thiết kế thế nào?"

Thứ hai, **giữ ý tưởng riêng trong đầu**. Lead được phép — nên — có sẵn một framing, một phương án nghiêng về. Nhưng khi phát vấn đề cho các lane, nó giữ ý đó lại. Ý riêng của Lead dùng để làm gì? Để đối chiếu lúc hội tụ: lane nào trùng ý nó, lane nào *phản bác được lập luận của nó* — trường hợp sau quý hơn vàng, vì đó là lúc Lead cần suy nghĩ lại. Một Lead tốt được thiết kế để *tìm kiếm* sự phản bác chứ không phòng thủ trước nó.

Thứ ba, **hội tụ như chủ trì một cuộc họp ba người**. Hình ảnh trong session: Lead gọi hai Peer vào phòng — "anh có câu hỏi này, hai em đưa phương án của mình đi" — để họ trình bày, đối chiếu; ý của Peer A lệch với Peer B thì điều phối cho chúng hợp nhất lập luận; và cuối cùng **Lead không được hòa nhã, không được bênh vực ai — nó phải chọn một phương án.** Sự dứt khoát này không phải tính cách, nó là chức năng: nghiên cứu debate cho thấy khâu yếu nhất là trích xuất cái đúng từ bất đồng — đó chính xác là việc của Lead, và một Lead ba phải là một khâu trích xuất hỏng.

### Quản trị vòng đời và context của Lead

*[Session]* Lead sống dài — "có những thằng Lead làm việc cả tuần". Hai kỹ thuật giữ nó khỏe:

**Compact không sợ hãi.** Đừng ám ảnh chuyện context Lead đầy. Miễn là Lead đang đi một flow thẳng (một mạch việc nhất quán), nén context (compact) định kỳ vẫn giữ chất lượng tốt. Nỗi sợ bloat không đáng để bạn phá vỡ tính liên tục của một Lead đang chạy tốt.

**Tách Lead cho nhánh rẽ.** Tình huống: Lead đang điều phối implement Authorization thì phát hiện hệ thống chưa có Authentication. Sai lầm phổ biến là để Lead hiện tại "tiện tay" ôm luôn — context của nó lập tức nhiễm hai mạch việc chồng chéo. Đúng bài: tạo một Lead *mới* cho Authentication, làm xong hand back, rồi Lead cũ đi tiếp. Nguyên tắc nền: một Lead, một mạch việc thẳng; nhánh rẽ lớn thì đẻ Lead mới chứ không bẻ cong Lead cũ.

### Lead và các task nhỏ

*[Session]* Chi tiết dễ bỏ qua mà rất khôn: task nhỏ cũng *không giao thẳng cho Peer*. Vẫn đưa qua Lead kèm chỉ dẫn "giao cho Peer làm" — vì khi đó Lead *chịu trách nhiệm* và tự động review Peer qua workflow review của nó. Cái bạn mua bằng một lượt gọi thêm là một tầng trách nhiệm và kiểm soát chất lượng. (Còn task nào nhỏ đến mức không đáng cả điều đó — xem Chương 7.)

### Checklist Lead tốt

Lead của bạn đạt chuẩn khi: phát vấn đề cho lane mà không lộ ý riêng (kiểm tra được bằng cách đọc lại prompt nó gửi); từng rút lại quyết định của chính nó sau khi bị Peer phản bác có lý — và Supervisor tường thuật được khoảnh khắc đó cho bạn; luôn chốt được một phương án thay vì trả về "cả hai đều có ưu nhược điểm"; biết đẻ Lead nhánh thay vì ôm việc chéo mạch; và các phiếu giao việc xuống Peer có mục tiêu, phạm vi, định dạng đầu ra rõ ràng.

---

## Chương 5: Xây một Supervisor tốt

### Định nghĩa lại: Supervisor không phải là sếp

Đây là chỗ SLP khác xa sách vở thông thường (và khác bản 1 của cuốn sách này). Trong đa số tài liệu, "supervisor" là agent trung tâm giao việc. Trong SLP, **giao việc là của Lead; Supervisor là người quan sát và kích hoạt attention** — nó theo dõi toàn bộ luồng làm việc từ design đến implement, giữ context của mình thật sạch, và can thiệp bằng những cú chạm nhỏ đúng thời điểm. Nó gần với một meta-cognition layer (tầng siêu nhận thức) của cả hệ thống hơn là một người quản lý.

### Nhiệm vụ 1: Attention trigger — cú chạm nhỏ, trung tính, đúng lúc

*[Session]* Khi Peer viết test, Supervisor không sửa test; nó hỏi: "mày có đang vi phạm anti-pattern nào khi viết unit test không?" Khi Peer định quantize int8, nó hỏi: "hướng đi này của mày có cần đưa về Lead để hội đồng design không?" Toàn bộ Chương 1 là nền tảng lý thuyết cho hành vi này: câu hỏi mở tái phân bổ attention của agent vào đúng chỗ dễ sai mà không gây hiệu ứng sycophancy. Ba tính chất bắt buộc của một cú can thiệp: *nhỏ* (một câu hỏi, không phải một bài giảng), *trung tính* (mở, không khẳng định, không gợi đáp án), *đúng lúc* (ngay khi tín hiệu xuất hiện, không phải 15 phút sau khi mọi thứ đã trôi).

### Nhiệm vụ 2: Theo dõi các khoảnh khắc hệ trọng

*[Session]* Supervisor subscribe vào các sự kiện đáng chú ý của hệ: Lead vừa ra một quyết định lớn; Peer đang struggling — vật lộn với thứ gì mơ hồ, phải đổi hướng giữa chừng; một mâu thuẫn Lead–Peer vừa nổ ra; một quyết định của Peer bắt đầu chạm vào design/requirement. Và nó tường thuật cho con người ở đúng độ cao: "hôm nay bọn nó mâu thuẫn về điều này; sau khi bị Peer phản đối, Lead đã rút quyết định và chọn phương án kia — anh thấy ok không? Ok thì để nó chạy tiếp." Con người ra phán quyết trong mười giây thay vì đọc mười nghìn token log.

### Nhiệm vụ 3: Người gác đêm

*[Session]* Chế độ ngủ: "tao đi ngủ, mày theo dõi xem có sự kiện gì tao cần chú ý khi thức dậy." Sáng ra, một bản report — thậm chí dạng voice để nghe khi lái xe — tổng hợp những gì đã xảy ra và những gì cần người quyết. Đây là nhiệm vụ quản trị human attention đã nói ở Chương 1, và là lý do trực tiếp khiến một người vận hành được bảy tám dự án song song.

### Kiến trúc kích hoạt: từ heartbeat thô sơ đến event-driven

*[Session]* Câu hỏi hay nhất session — "khi nào Supervisor biết mà trigger?" — được trả lời bằng hai mức kiến trúc:

**Mức 1 — Heartbeat (cách "thô thiển hơn nhưng vẫn work"):** cứ 15 phút, Supervisor đọc toàn bộ luồng làm việc của Lead và Peer trong quãng đó, tự hỏi "có gì cần can thiệp không". Dễ dựng, không cần hạ tầng gì. Nhược điểm được nói thẳng: trễ và loãng. Hãy bắt đầu từ đây.

**Mức 2 — Event-driven với bộ dò rẻ tiền:** một model *rất nhỏ và rất rẻ* (session mô tả cỡ vài chục nghìn tham số — bản chất là một bộ phân loại ngữ nghĩa, không phải LLM suy luận) liên tục polling luồng hội thoại của tất cả các agent. Nó chỉ cần bắt tín hiệu bề mặt: từ khóa kiểu "but", "hold on", giọng điệu tự nhận sai, luồng suy nghĩ "không smooth, không thẳng hàng". **False positive là chấp nhận được** — vì nó không can thiệp, nó chỉ bắn event "Supervisor ơi, chú ý đoạn này, cần can thiệp thì can thiệp". Supervisor (một model đủ thông minh) nhận event mới đọc kỹ và *tự quyết có ra tay hay không*. Kiến trúc hai tầng dò-rẻ → nghĩ-đắt này là một bài học thiết kế đáng giá vượt ra ngoài SLP.

Có thể có *nhiều* Supervisor con theo dõi các mảng khác nhau, bắn event về một Supervisor tổng — session xác nhận mô hình này, và ghi nhận thực tế "có lúc hơn 100 Supervisor" chạy đồng thời vẫn bình thường.

### Chọn model cho Supervisor

*[Session]* Không cần model đỉnh. Supervisor cần hai phẩm chất: *rẻ* (vì nó chạy liên tục) và *context dài* (vì nó theo dõi long-term) — Haiku, Flash hay các model tương tự là ứng viên tự nhiên. Và một cơ chế leo thang khôn ngoan: khi gặp tình huống vượt năng lực suy nghĩ của mình, Supervisor rẻ *notify cho một Supervisor bạn mạnh hơn* thay vì tự xử. Đừng dùng búa tạ để canh cửa; hãy để người canh cửa biết lúc nào cần gọi búa tạ.

### Checklist Supervisor tốt

Supervisor của bạn đạt chuẩn khi: các can thiệp của nó trong log đều là câu hỏi mở dưới ba câu; nó đã từng *quyết định không can thiệp* sau khi nhận một event (dấu hiệu nó có phán đoán riêng, không phải máy phát cảnh báo); báo cáo buổi sáng của nó khiến bạn hành động được ngay mà không cần mở log gốc; và chi phí chạy nó mỗi ngày thấp hơn đáng kể chi phí một giờ attention của bạn — vì đó chính là thứ nó đang mua về cho bạn.

---
# PHẦN III — VẬN HÀNH HỆ SLP

## Chương 6: Thang leo quyết định (escalation ladder)

Ghép ba vai trò lại, ta được một chiếc thang mà mọi quyết định trong hệ leo lên tùy độ hệ trọng:

**Bậc 1 — Peer tự quyết:** chi tiết cài đặt thuần túy, không chạm contract, không đổi trade-off. **Bậc 2 — Đưa về Lead:** quyết định chạm vào design, requirement, hoặc mơ hồ về phạm vi. Lead có thể tự chốt nếu đủ rõ. **Bậc 3 — Hội đồng lane:** quyết định hệ trọng và mơ hồ — Lead mở dual/three-lane blind design, hội tụ, chốt. **Bậc 4 — Con người:** *[Session]* khi "không có phương án nào đủ standard hoặc đủ vượt trội so với các phương án còn lại" — tức là chính quá trình hội tụ cũng bất phân thắng bại — thì đưa về human. Supervisor đứng ngoài thang, quan sát mọi bậc, và có quyền *đề nghị* một quyết định đang ở bậc thấp leo lên bậc cao hơn (ví dụ vụ int8: nhắc Peer cân nhắc đưa về Lead).

Chiếc thang này hay ở chỗ nó tiết kiệm đúng thứ cần tiết kiệm: quyết định rẻ được xử lý rẻ, và chỉ những bất định thật sự mới tiêu tốn hội đồng lane hay attention của con người. Khi thiết kế hệ của riêng bạn, hãy viết tiêu chí lên/xuống thang thành văn bản trong instruction của cả ba vai trò — thang chỉ chạy khi cả ba cùng biết nó tồn tại.

### Bài tập chương 6

Với dự án hiện tại của bạn, liệt kê 10 quyết định gần đây và xếp mỗi cái vào một bậc thang. Có quyết định nào đã được xử lý ở bậc thấp hơn mức nó xứng đáng không? Đó chính là chỗ Supervisor của bạn cần một quy tắc dò mới.

---

## Chương 7: Khi nào dùng SLP — và khi nào đừng

*[Session]* Session trả lời câu này rất thẳng. SLP dành cho: **task long-term** — làm dài ngày, bạn không muốn phải chú ý liên tục; **quyết định khó và mơ hồ** — đáng mở dual/three-lane để cùng design và hội tụ; **domain mới mà bạn chưa đủ tự tin phản biện** — "em chưa đủ tự tin vào khả năng phản biện của bản thân thì để thằng Lead nó phản biện dùm em". Điểm cuối này sâu sắc: giá trị của SLP không chỉ là năng suất mà là *thay thế năng lực phản biện mà bạn chưa có* trong một lĩnh vực lạ — một người mới làm game được hưởng cấu trúc phản biện của một hệ thống, thay vì một mình đối mặt với model.

SLP *không* dành cho: task nhỏ, bounded, kết thúc ngay — "hôm nay cổ đông cần một cái landing page giới thiệu hệ thống booking" thì prompt thẳng một hai ba phát bằng CLI/Codex/Claude là xong, "nhét vào SLP làm cái gì cho mất thời gian". Và một sắc thái đã nói ở Chương 4: vùng giữa — task nhỏ nhưng *chạm vào hệ thống* — vẫn nên đi qua Lead để có người chịu trách nhiệm review, chỉ những task thuần túy dùng-xong-vứt mới đáng đi đường tắt.

Cũng cần trung thực về chi phí đầu tư: *[Session]* với người làm một hai project, ROI của cả bộ máy này không cao — bạn đầu tư nhiều mà thu về chưa đủ. SLP tỏa sáng với người làm nhiều project song song hoặc project phức tạp dài hơi. Nếu bạn đang học, hãy dựng nó để *hiểu*, nhưng đừng kỳ vọng phép màu năng suất ngay trên một side-project cuối tuần.

---

## Chương 8: Better SLP — vòng lặp tự cải tiến hằng tuần

*[Session]* Phần này biến SLP từ một cấu hình tĩnh thành một hệ thống sống. Quy trình mỗi tuần: một tiến trình tổng kết ("Better SLP") rà lại toàn bộ tuần làm việc của các phòng và trả lời ba câu hỏi. Một: tuần này có failure mode nào đáng chú ý? Hai: failure mode đó có đủ *generic* để kết tinh thành skill hoặc instruction dùng chung không? Ba: cần chỉnh instruction của vai trò nào? Rồi chỉnh dần. Kỳ vọng đúng mức: "mỗi tuần tốt hơn khoảng vài phần trăm là ok rồi" — sau ba bốn tuần, instruction đã khác hẳn phiên bản đầu.

Hai nguyên tắc kỷ luật đi kèm. **Kết tinh cái generic, bỏ qua cái đặc thù:** danh sách anti-pattern và skill dùng chung cho các project; chỉ những dự án thật sự đặc thù, có kiểu sai lặp lại riêng, mới đáng một skill riêng — và "không cần optimize quá mức điều đó". **Sửa instruction, đừng sửa từng ca:** khi Peer phạm một lỗi, cám dỗ là nhắc nó ngay ca đó rồi thôi; kỷ luật là ghi nhận, đợi cuối tuần, hỏi "lỗi này có tính hệ thống không", và nếu có thì sửa ở tầng instruction để mọi Peer tương lai miễn nhiễm. Đây chính là procedural memory (bộ nhớ quy trình) của hệ thống — kinh nghiệm được kết tinh thành cách làm việc, đúng nghĩa một tổ chức biết học.

### Bài tập chương 8

Dựng phiên bản tối giản của Better SLP ngay tuần này, kể cả khi bạn chưa có SLP: cuối tuần, đưa toàn bộ lịch sử chat AI trong tuần cho một session mới với đề bài "liệt kê các failure mode lặp lại và đề xuất 3 dòng instruction để phòng chúng". Chạy bốn tuần liên tiếp và giữ lại các bản instruction để thấy sự tiến hóa.

---

## Chương 9: Hạ tầng và workflow — bài học iPhone và cái sim

*[Session]* Phần cuối session là một bài giảng về thiết kế sản phẩm, nhân chuyện một thành viên xây tool (Astra) khóa cứng vào SLP. Lập luận: SLP là một workflow **opinionated** — nó mang quan điểm cá nhân và pain point riêng của người tạo ra nó. Nền tảng điều phối bên dưới thì ngược lại: nó chỉ cung cấp *hạ tầng* đủ generic (remote connect, quản nhiều workspace, nhiều provider) và người dùng tự implement workflow của mình lên trên — nó "không hề đưa cho anh cái SLP, chỉ là anh cảm thấy nó đủ hạ tầng để implement cái SLP".

Phép ẩn dụ chốt hạ: Apple bán iPhone, không bán iPhone dính chặt sim Viettel — về lắp Mobi, Vina, Viettel gì cũng gọi được; nhiều nhất là *tặng kèm* một cái sim tháo ra được. Nếu bạn xây tool cho người khác: workflow opinionated phải là plugin có thể switch off hoặc thay thế, đừng để nó ăn sâu vào hạ tầng — vì ngày mai chính tác giả SLP có thể bỏ SLP để theo một phương pháp khác, và tool của bạn không thể chết theo một workflow. Bài học này cũng đúng khi bạn xây cho chính mình: tách phần *cơ chế* (spawn agent, truyền tin, lưu trạng thái, bắn event) khỏi phần *chính sách* (ai hỏi ai, khi nào escalate) để tuần sau đổi chính sách không phải đập cơ chế.

---

# PHẦN IV — THỰC HÀNH THEO BA MỨC

## Chương 10: Mức 0 — SLP thủ công, không cần công cụ nào

*[Session]* Bạn có thể chạy tinh thần SLP ngay hôm nay với hai cửa sổ chat, bằng cách *tự đóng vai Lead*. Quy trình: mở session A, đưa đề bài thiết kế, nhận phương án. Đừng chất vấn ngược trong chính session A — "nó bị loãng" (và như Chương 1 đã giải thích, session A giờ đã có framing của chính nó). Thay vào đó mở session B sạch, dán phương án của A vào với lời dẫn trung tính: "tao vừa hỏi một agent, nó đưa câu trả lời thế này, ý kiến của mày thế nào?" — không nói bạn thích hay không thích. Session B trở thành lane thứ hai; bạn hội tụ. Muốn chuẩn hơn nữa thì làm blind hoàn toàn: đưa *cùng đề bài* cho B trước, nhận phương án độc lập của B, rồi mới cho hai bên xem chéo. Bài tập bắt buộc của cuốn sách: làm quy trình này với một quyết định thiết kế thật của bạn trong tuần này. Mọi thứ ở Mức 1 và 2 chỉ là tự động hóa những gì tay bạn vừa làm.

## Chương 11: Mức 1 — SLP với heartbeat Supervisor

Dựng đủ ba vai trò bằng framework tùy chọn (hoặc script thuần): một Lead sống dài có compact, các Peer với instruction phản biện 30–40 dòng (Chương 3), và một Supervisor heartbeat — cron 15 phút/lần đọc log các phiên và quyết định có gửi câu hỏi mở nào không, cuối ngày sinh báo cáo tổng hợp cho bạn. Thêm thang escalation thành văn trong instruction cả ba vai. Tiêu chí hoàn thành: hệ chạy một task kéo dài nhiều ngày (ví dụ dựng một module có design thật) mà bạn chỉ tham gia ở các điểm Supervisor gọi tên; và trong log có ít nhất một lần Peer phản biện Lead thành công.

## Chương 12: Mức 2 — Event-driven và đội hình nhiều Supervisor

Nâng cấp: thay heartbeat bằng bộ dò rẻ (một model nhỏ hoặc thậm chí heuristic từ khóa + embedding) polling luồng làm việc, bắn event cho Supervisor; chấp nhận false positive, để Supervisor tự lọc. Chạy dual-lane blind design tự động cho các quyết định bậc 3. Bật Better SLP hằng tuần. Đích đến cuối: chế độ người gác đêm — bạn thật sự đi ngủ, và sáng hôm sau bản báo cáo đầu tiên khiến bạn tin được hệ thống của mình.

---

# PHỤ LỤC

## Phụ lục A — Case study: khi red test "mint" ra API

*[Session]* Đây là ví dụ xuyên suốt session về một anti-pattern nguy hiểm, đáng đọc chậm vì nó minh họa cả ba vai trò cùng lúc.

Bối cảnh: bạn cần tính năng "user mua hàng xong được cộng điểm", nhưng hệ thống *chưa có* khái niệm điểm — bảng User chưa có field point, class User chưa có property point, contract chưa được chốt. Nếu để agent viết unit test ngay (tinh thần TDD máy móc: red test trước), nó buộc phải **mint** — tự bịa ra — một interface có point (một mock, một adapter) để test compile được. Từ đây dây chuyền domino đổ: agent implement sau nhìn test thấy có point, bèn cắm point thẳng vào User cho khớp; test đã *pin* một hành vi chưa hề được thảo luận. Tháng sau bạn quyết định điểm nên nằm trong một view chứ không phải cột trên bảng User — đổi contract một chút, cả loạt test đỏ; model mới vào (không giữ context của model cũ) nhìn test đỏ lại suy luận "test đang pin hành vi đúng, ta phải thỏa mãn nó" và **bẻ implementation của bạn để chiều một cái test vốn được tạo ra rất tệ.** ChatGPT gọi đúng tên hiện tượng: viết test khi contract chưa ổn định và tự mint interface dẫn đến over-specifying — đặc tả chặt quá mức — và nợ kỹ thuật nặng cho test suite.

Phòng chống theo từng vai trò: *Peer* có dòng anti-pattern "không viết unit test khi contract chưa ổn định; nếu contract chưa có, dừng lại hỏi thay vì tự mint interface". *Supervisor* có quy tắc dò: hoạt động viết test trên một feature mà dependency chưa được design là tín hiệu bắn event, kèm câu hỏi mở "contract của phần này đã đủ ổn định để viết test chưa?". *Lead* khi giao việc phải nói rõ trạng thái contract trong phiếu giao việc — mơ hồ ở đề bài là nguồn gốc của mọi cú mint. Và bài học meta: lỗi này không phải "mood" hay sự cẩu thả của model — nó là hệ quả logic của việc bắt một hệ thống tất định hóa thứ chưa được quyết định. Đơn giản hóa nguyên nhân thành "model tùy hứng" là tự tước đi khả năng phòng ngừa có hệ thống.

## Phụ lục B — Người quản trị cũng phải lớn: sói và bầy cừu

*[Session]* Session dành một đoạn đáng nhớ cho khía cạnh con người: "em là một con sói dẫn bầy cừu, chứ không thể là con cừu đòi dẫn bầy sói." Để quản trị AI, bạn phải liên tục bồi đắp năng lực bản thân — vì các agent ngày càng giỏi, và người điều phối không hiểu chuyên môn sẽ dần thành người bị điều phối. Cách học được đề xuất trong session trùng khớp kỳ lạ với cách cuốn sách này ra đời: nhờ AI biên soạn sách theo đúng chủ đề mình cần, đọc mỗi ngày, nghe voice khi lái xe, đọc trong lúc agent đang chạy. Kèm lời cảnh tỉnh: "prompt không là chết dở — cuối cùng thành con cừu dẫn bầy sói thì vỡ mồm." Kỹ năng đặt vấn đề cũng là năng lực nền: nếu bạn trình bày vấn đề mà đồng nghiệp còn không hiểu, đừng mong AI hiểu.

## Phụ lục C — Đối chiếu với bản 1: vai trò và topology

Bản 1 của cuốn sách dùng từ vựng phổ biến trong tài liệu ngành: supervisor (điều phối tập trung), hierarchical (phân cấp), peer/network (ngang hàng + handoff) — ba *topology*. SLP của session dùng ba *vai trò*, và ánh xạ không trùng tên: **Lead của SLP** mới là "supervisor" theo nghĩa sách vở (người giao việc, hội tụ, chịu trách nhiệm); **Peer của SLP** gần với worker nhưng được nâng cấp bằng năng lực phản biện — chính là liều thuốc cho hiệu ứng conformity mà nghiên cứu debate cảnh báo; còn **Supervisor của SLP** hầu như không có tên trong từ vựng topology — nó là một tầng quan sát siêu nhận thức nằm *ngoài* mọi topology, họ hàng gần nhất trong tài liệu là các hệ LLM-as-judge và monitoring, nhưng khác ở chỗ nó can thiệp bằng câu hỏi trong lúc chạy chứ không chấm điểm sau khi xong. Cấu trúc Lead + các lane blind design cũng có thể đọc như mẫu orchestrator–workers ghép với best-of-N và một khâu hội tụ có chủ trì. Hiểu cả hai hệ từ vựng, bạn đọc được tài liệu của cả hai thế giới — và quan trọng hơn, bạn thấy được rằng topology nào cũng cần các vai trò này được làm tử tế thì mới sống.

## Phụ lục D — Bảng thuật ngữ

SLP: mô hình ba vai trò Supervisor–Lead–Peer. Attention trigger: cú can thiệp nhỏ (câu hỏi mở) làm agent tái phân bổ năng lực tính toán vào chỗ dễ sai. Framing: việc vô tình/cố ý áp đặt khung suy nghĩ khiến agent trả lời chiều theo. Sycophancy: xu hướng model chiều ý người hỏi, kể cả nhận lỗi không có thật. Blind design: các lane thiết kế độc lập, không biết ý của Lead và của nhau. Dual/Three-Lane: 2–3 lane thiết kế song song cho một quyết định hệ trọng. Best-of-N: sinh N phương án độc lập rồi chọn. Conformity effect: hiệu ứng agent bỏ phán đoán đúng để theo số đông trong tranh luận. Hội tụ (converge): khâu Lead đối chiếu các lane và chốt phương án. Escalation ladder: thang leo quyết định Peer → Lead → hội đồng lane → human. Heartbeat: Supervisor đọc log theo chu kỳ cố định. Event-driven: bộ dò rẻ bắn sự kiện cho Supervisor khi phát hiện tín hiệu bất thường. Handback: Lead nhánh hoàn thành việc và trả quyền về Lead chính. Compact: nén context của một agent sống dài. Mint (test): test tự bịa ra interface/implementation chưa được quyết định. Over-specifying: test đặc tả chặt quá mức hành vi chưa được thảo luận. Better SLP: vòng tổng kết hằng tuần, kết tinh failure mode thành skill/instruction. Opinionated: mang quan điểm cá nhân (nói về workflow, đối lập với hạ tầng generic). Write scope: quyền ghi/sửa thật vào hệ thống của một agent.

---

*Nguyên tắc xuyên suốt phiên bản này, nói bằng ngôn ngữ của session: lỗi của agent phần lớn là lỗi attention — nên hãy xây hệ thống biết chạm nhẹ đúng lúc (Supervisor), biết hỏi mà không framing rồi dám chốt (Lead), và biết cãi lại khi cần (Peer). Phần còn lại là kỷ luật cải tiến vài phần trăm mỗi tuần.*