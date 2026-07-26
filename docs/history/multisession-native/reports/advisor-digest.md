# Advisor consult — stage 0-1 execution (Gate 3)

Advisor: claude/fable (ceiling), read-only, 2026-07-24.
VERDICT: proceed-with-conditions.

- C1 (msn-1): wrap alone insufficient — heartbeatSession reads the session record OUTSIDE its lock (claims.mjs:242 vs :246); move that read inside the lock symmetrically, and confirm no bind/unbind caller nests inside the sessions lock (O_EXCL, non-reentrant).
- C2 (msn-2): post-reacquire fence must check HEAD (worktree-store.mjs:1456) + MERGE_HEAD (:1486) + staged-tree identity (index hash — HEAD-only inadequate) + grant (:1392) before the commit (:1553); on drift: merge --abort + mainUntouchedProof + typed refusal. hardening-4b serialization holds for grant/create ops; second concurrent merge self-blocks on isTreeDirty (:1410).
- C3 (msn-3): contention append must be lock-free (plain appendFileSync mirroring timings.jsonl), never routed through lock primitives (infinite recursion), fully fail-open (swallow all errors incl. EBUSY).
- C4: concurrency tests use deterministic interleaving seams (precedent _takeoverSeam/_postRenameSeam lock.mjs:461-464), never sleeps; do not build on the disclosed v1.16.1 flaky red.

Sequencing sound: msn-1 → msn-3 → msn-4 → msn-2 sequential (shared regen artifacts). Slices 2-5 correctly deferred. No blocking red flag.
