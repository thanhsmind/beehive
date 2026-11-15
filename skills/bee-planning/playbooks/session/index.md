# session

Situations that belong to a session, not to a feature route: pick the situation below and open its file.

## Situations

- `session-pickup` — take over this, resume this conversation, continue from this transcript, you're taking over, pick up where X left off, continue the pushed branch, tiếp quản, làm tiếp từ phiên trước, tiếp tục từ chỗ đã dừng — delivers: where the prior agent stopped, the resume point, the inherited claims checked on the real artifact, and the routed next step
- `pause-safely` — pause safely, I need to go offline, board my flight, stop here and checkpoint, tạm dừng an toàn, tôi phải offline, nghỉ ở đây và lưu lại — delivers: a durable checkpoint a cold-start agent can resume from, the wip commit, and the first action on resume
- `autonomous-run` — going to bed, run until done, keep going until X, don't stop, work through the night, tôi đi ngủ, chạy đến khi xong, cứ làm tiếp đừng dừng — delivers: a checkable exit condition, the iterations driven to it, and the final predicate state
- `orchestrate` — a whole project, multi-day program, dozens of cells and workers, standing coordinator, điều phối cả dự án, chương trình nhiều ngày, nhiều worker song song — delivers: the program's briefs, the drained queue, the green merge frontier, and the report of landed and parked units
- `worktree-cleanup` — clean up worktrees, free disk space, prune old worktrees, disk is full, dọn worktree, giải phóng ổ đĩa, xóa worktree cũ — delivers: disk reclaimed with before and after sizes, the worktrees pruned, and a reason for each one held back
- `babysit` — babysit this, get it green, all green, merge-ready, watch CI, address the review comments, check on PR X, trông PR cho xanh, theo dõi CI, xử lý comment review, kiểm tra PR — delivers: the declared mode, the frontier cleared to green with fixes versus dismissals reasoned, and the unit landed through bee worktree merge in drive mode
- `shipping` — land the stack, ship it, merge the verified units, second half after babysit, đưa lên main, hạ cánh cả chuỗi, merge các unit đã verify — delivers: an independent verdict per unit, the contiguous verified run merged one unit at a time, and the ceiling with what the next gap needs
- `autopilot-full` — autopilot this queue, full autopilot, one owner per unit, drive the queue to merged, tự lái cả hàng đợi, mỗi unit một owner, chạy hết queue đến khi merge — delivers: the queue with each unit's owner and verdict, the units merged on a clean verdict, and the owner's gates still open
- `autopilot-stack` — autopilot-stack, stack them don't ship, build the stack I'll land it, xếp thành chuỗi rồi tôi duyệt, dựng chuỗi để tôi hạ cánh — delivers: one linear chain of verified units each with its verdict, landed in one ordered pass, and what was parked or excluded
- `opening-a-pr` — open a PR, make a pull request, push and open the PR, ready for review, write the PR description, mở PR, tạo pull request, viết mô tả PR, đưa lên review — delivers: the worktree commits in order, the draft PR with a reviewer-ready title and body, and the bee merge path ready to run

No situation fits, or the work is large and cross-cutting: use bee-figure-it-out.
