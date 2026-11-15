---
name: bee-principle-model-the-domain
description: "Apply when writing stateful logic, or when code branches a lot or repeats a shape assumption across files. Encode the domain in a structure instead of scattered conditionals."
---

# Model the Domain

Encode the real domain in a data structure instead of scattering it across
conditionals. Scattered booleans, repeated shape assumptions and branching
spread over several files are accidental complexity; a structure that matches
the domain makes invalid states unrepresentable and deletes branches. Choosing
it while you write is cheap, and recovering it later reads as a refactor that
gets deferred.

Reach for a state machine over booleans and phase checks, a typed object over
loose parameters, a map or tagged union over branching across files, a reducer
or command model over ad hoc mutation, a module organised around one body of
domain knowledge over a load, validate, transform, save sequence. When none of
these fits, work out what the code must never allow and how the data is read,
then find the structure that encodes exactly that.

**Check:** does this change grow an if/else chain by one more branch, or add a
second boolean that must stay in sync with the first, and what structure would
make that branch or flag unnecessary?

**Not when:** the current shape is already clear, local and unlikely to grow.
An abstraction that removes no branch, duplicated rule, invalid state or
lifecycle risk only adds indirection (see subtract-before-you-add).

**Why:** the domain is already there; code that does not name it spends every
later change rediscovering it.

**Depth:** `.bee/expertise/architecture.md` § Model the Domain.
