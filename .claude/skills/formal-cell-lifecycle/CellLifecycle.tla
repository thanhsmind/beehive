---------------------------- MODULE CellLifecycle ----------------------------
EXTENDS Naturals, FiniteSets

CONSTANTS Sessions, None, Variant, MaxClaims

Variants == {"code", "owner-blind-clear", "no-ownership-check"}

Statuses == {"open", "claimed", "blocked", "capped", "dropped"}
Ops == {"claim", "cap", "block", "unclaim", "reopen", "drop", "sweep"}
CloseOps == {"cap", "block", "unclaim", "reopen", "drop"}

VARIABLES status, owner, token, claimNo, stale, revoked, pc, op, seen, swept, foreign

vars == <<status, owner, token, claimNo, stale, revoked, pc, op, seen, swept, foreign>>

Init ==
    /\ status = "open"
    /\ owner = None
    /\ token = None
    /\ claimNo = 0
    /\ stale = [s \in Sessions |-> FALSE]
    /\ revoked = FALSE
    /\ pc = [s \in Sessions |-> "idle"]
    /\ op = [s \in Sessions |-> "none"]
    /\ seen = [s \in Sessions |-> 0]
    /\ swept = [s \in Sessions |-> None]
    /\ foreign = FALSE

OwnershipOk(s) ==
    \/ Variant = "no-ownership-check"
    \/ token = None
    \/ stale[token]
    \/ token = s

LiveForeignOwner(s) ==
    /\ status = "claimed"
    /\ owner \notin {None, s}
    /\ ~revoked

Start(s, o) ==
    /\ pc[s] = "idle"
    /\ pc' = [pc EXCEPT ![s] = "lock"]
    /\ op' = [op EXCEPT ![s] = o]
    /\ UNCHANGED <<status, owner, token, claimNo, stale, revoked, seen, swept, foreign>>

Finish(s) ==
    /\ pc' = [pc EXCEPT ![s] = "idle"]
    /\ op' = [op EXCEPT ![s] = "none"]

ClaimFile(s) ==
    /\ pc[s] = "lock" /\ op[s] = "claim"
    /\ IF token = None /\ claimNo < MaxClaims
         THEN /\ token' = s
              /\ claimNo' = claimNo + 1
              /\ stale' = [stale EXCEPT ![s] = FALSE]
              /\ pc' = [pc EXCEPT ![s] = "store"]
              /\ UNCHANGED op
         ELSE /\ Finish(s)
              /\ UNCHANGED <<token, claimNo, stale>>
    /\ UNCHANGED <<status, owner, revoked, seen, swept, foreign>>

ClaimCellUnderLock(s) ==
    /\ pc[s] = "store" /\ op[s] = "claim"
    /\ IF status = "open"
         THEN /\ status' = "claimed"
              /\ owner' = s
              /\ revoked' = FALSE
              /\ UNCHANGED token
         ELSE /\ token' = IF token = s THEN None ELSE token
              /\ UNCHANGED <<status, owner, revoked>>
    /\ Finish(s)
    /\ UNCHANGED <<claimNo, stale, seen, swept, foreign>>

CloseTo(o) ==
    CASE o = "cap"     -> "capped"
      [] o = "block"   -> "blocked"
      [] o = "unclaim" -> "open"
      [] o = "reopen"  -> "open"
      [] o = "drop"    -> "dropped"

CloseAllowed(s, o) ==
    CASE o = "cap"     -> status \notin {"capped", "dropped"} /\ OwnershipOk(s)
      [] o = "block"   -> OwnershipOk(s)
      [] o = "unclaim" -> status = "claimed" /\ OwnershipOk(s)
      [] o = "reopen"  -> status \in {"blocked", "capped", "dropped"} /\ OwnershipOk(s)
      [] o = "drop"    -> OwnershipOk(s)

CloseCellUnderLock(s) ==
    /\ pc[s] = "lock" /\ op[s] \in CloseOps
    /\ IF CloseAllowed(s, op[s])
         THEN /\ foreign' = (foreign \/ LiveForeignOwner(s))
              /\ status' = CloseTo(op[s])
              /\ owner' = None
              /\ seen' = [seen EXCEPT ![s] = IF token = None THEN 0 ELSE claimNo]
              /\ pc' = [pc EXCEPT ![s] = "clear"]
              /\ UNCHANGED op
         ELSE /\ Finish(s)
              /\ UNCHANGED <<status, owner, seen, foreign>>
    /\ UNCHANGED <<token, claimNo, stale, revoked, swept>>

ClearClaimFile(s) ==
    /\ pc[s] = "clear"
    /\ token' = IF Variant # "owner-blind-clear" /\ (token = None \/ claimNo # seen[s]) THEN token ELSE None
    /\ seen' = [seen EXCEPT ![s] = 0]
    /\ Finish(s)
    /\ UNCHANGED <<status, owner, claimNo, stale, revoked, swept, foreign>>

SweepClaimFile(s) ==
    /\ pc[s] = "lock" /\ op[s] = "sweep"
    /\ IF token \notin {None, s} /\ stale[token]
         THEN /\ swept' = [swept EXCEPT ![s] = token]
              /\ token' = None
              /\ pc' = [pc EXCEPT ![s] = "reset"]
              /\ UNCHANGED op
         ELSE /\ Finish(s)
              /\ UNCHANGED <<token, swept>>
    /\ UNCHANGED <<status, owner, claimNo, stale, revoked, seen, foreign>>

SweepResetCell(s) ==
    /\ pc[s] = "reset"
    /\ IF status = "claimed" /\ owner = swept[s]
         THEN /\ status' = "blocked"
              /\ owner' = None
         ELSE UNCHANGED <<status, owner>>
    /\ swept' = [swept EXCEPT ![s] = None]
    /\ Finish(s)
    /\ UNCHANGED <<token, claimNo, stale, revoked, seen, foreign>>

ClaimGoesStale ==
    /\ token # None
    /\ pc[token] = "idle"
    /\ ~stale[token]
    /\ stale' = [stale EXCEPT ![token] = TRUE]
    /\ revoked' = (revoked \/ token = owner)
    /\ UNCHANGED <<status, owner, token, claimNo, pc, op, seen, swept, foreign>>

SweepPending(o) == \E s \in Sessions : pc[s] = "reset" /\ swept[s] = o

Next ==
    \/ \E s \in Sessions, o \in Ops : Start(s, o)
    \/ \E s \in Sessions :
         \/ ClaimFile(s)
         \/ ClaimCellUnderLock(s)
         \/ CloseCellUnderLock(s)
         \/ ClearClaimFile(s)
         \/ SweepClaimFile(s)
         \/ SweepResetCell(s)
    \/ ClaimGoesStale

Spec == Init /\ [][Next]_vars

TypeOK ==
    /\ Variant \in Variants
    /\ status \in Statuses
    /\ owner \in Sessions \cup {None}
    /\ token \in Sessions \cup {None}
    /\ claimNo \in 0..MaxClaims
    /\ stale \in [Sessions -> BOOLEAN]
    /\ revoked \in BOOLEAN
    /\ pc \in [Sessions -> {"idle", "lock", "store", "clear", "reset"}]
    /\ seen \in [Sessions -> 0..MaxClaims]
    /\ swept \in [Sessions -> Sessions \cup {None}]
    /\ foreign \in BOOLEAN

ClaimedCellHoldsItsOwnersClaimFile ==
    status = "claimed" => (token = owner \/ SweepPending(owner))

NoForeignCloseOfALiveClaim == ~foreign

=============================================================================
