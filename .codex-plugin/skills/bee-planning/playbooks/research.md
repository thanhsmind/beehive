# research

A question to answer or a cause to find, with no change to ship: pick the situation below and open its file.

## Situations

- `investigation` — how does X work, why was Y built this way, are we sure about Z, should we do X or Y, explain this code, trace this path, X hoạt động thế nào, tại sao làm thế này, có chắc về điều này không, nên chọn X hay Y — delivers: a cited account with anchors a reader can open, the sources that came up empty, and a recommendation with a tradeoffs table when the question chooses between alternatives
- `runtime-forensics` — why is X leaking, spinning, slow at runtime, idle but busy, intermittent glitch, heap grows, CPU stuck high, tại sao rò bộ nhớ, tiến trình chạy không nghỉ, chậm lúc chạy, lỗi chập chờn — delivers: a cited diagnosis from a live capture, the proof of the mechanism, and the source location, with no fix
- `trace-forensics` — here is a cpuprofile, Trace json.gz, spindump, heapsnapshot, why is this slow or unresponsive from this artifact, đây là file profile, phân tích trace này, vì sao chậm từ artifact này, đọc heap snapshot — delivers: a cited diagnosis from the artifact, the reduced finding mapped to source, and whether a paired capture confirmed it, with no fix
- `eval` — evaluate this skill variant, compare two prompts, does this change affect agent behavior, run a blind experiment, test a structural change before promoting it, đánh giá biến thể skill, so sánh hai prompt, thí nghiệm mù, thử trước khi áp dụng — delivers: the variant under test, the rubric, per-candidate notes, the blind judge's verdict, and a recommendation on promoting the variant

No situation fits, or the work is large and cross-cutting: use bee-figure-it-out.

Proof floor: the cap refuses a result weaker than this class's floor and names it; the table is the cap's own (`bee cells finish --help`).
