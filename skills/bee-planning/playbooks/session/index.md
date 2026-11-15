# session

Situations that belong to a session, not to a feature route: pick the situation below and open its file.

## Situations

- `session-pickup` — take over this, resume this conversation, continue from this transcript, you're taking over, pick up where X left off, continue the pushed branch, tiếp quản, làm tiếp từ phiên trước, tiếp tục từ chỗ đã dừng — delivers: where the prior agent stopped, the resume point, the inherited claims checked on the real artifact, and the routed next step
- `pause-safely` — pause safely, I need to go offline, board my flight, stop here and checkpoint, tạm dừng an toàn, tôi phải offline, nghỉ ở đây và lưu lại — delivers: a durable checkpoint a cold-start agent can resume from, the wip commit, and the first action on resume
- `autonomous-run` — going to bed, run until done, keep going until X, don't stop, work through the night, tôi đi ngủ, chạy đến khi xong, cứ làm tiếp đừng dừng — delivers: a checkable exit condition, the iterations driven to it, and the final predicate state
- `orchestrate` — a whole project, multi-day program, dozens of cells and workers, standing coordinator, điều phối cả dự án, chương trình nhiều ngày, nhiều worker song song — delivers: the program's briefs, the drained queue, the green merge frontier, and the report of landed and parked units
- `worktree-cleanup` — clean up worktrees, free disk space, prune old worktrees, disk is full, dọn worktree, giải phóng ổ đĩa, xóa worktree cũ — delivers: disk reclaimed with before and after sizes, the worktrees pruned, and a reason for each one held back
