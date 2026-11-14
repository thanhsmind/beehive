---
name: bee-principle-migrate-callers-then-delete
description: "Apply when a new internal API ships while old callers still exist. Move every caller and delete the old API in the same change."
---

# Migrate Callers, Then Delete

Two APIs for one job is a debt nobody owns. When the new API lands, move
every caller to it and remove the old one in the same change, so the tree
never holds both.

The tests move too. Those that exercised behavior now assert the new
contract; those that only pinned the old internals are deleted, not ported.
Skip the compatibility shim: inside one repo the compiler and the tests find
every caller.

**Check:** after this change, does any caller still reach the old API, and
does the old API still exist?

**Not when:** a user outside this change depends on the old surface — a
published package, another live worktree, a stored format. Then a bridge is
real; file it with its removal condition.

**Why:** a bridge kept "for a while" is never removed, and the new API ends
up serving only half its callers.

**Depth:** `.bee/expertise/architecture.md` § Migrate Callers, Then Delete.
