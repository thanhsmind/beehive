---
name: bee-principle-the-rebuild-test
description: "Apply when you write or judge a spec. Ask whether a competent stranger could rebuild the behavior on a different stack from the spec alone."
---

# The Rebuild Test

A good spec passes one test: a competent stranger, given only the spec, could
rebuild the behavior on a different stack — different language, different
framework, different storage — and users could not tell the difference.

That test picks the content for you. It rules OUT code tours ("the handler
calls the service, which calls the repository"), because a rebuilder on another
stack has no handler and no repository. It rules IN behavior and rules: what
comes in, what goes out, what is guaranteed, what is rejected, what happens on
failure.

> Fails: "OrderValidator runs the checks in `checks/` and throws
> ValidationError on the first failure."
>
> Passes: "An order is rejected if any line quantity is zero or negative, if
> the total exceeds the customer's credit limit, or if the shipping country is
> not supported. The first failing rule is reported; the rest are not
> evaluated."

Run the test explicitly when you finish a spec: pick a behavior, cover the
code, and ask whether the spec alone pins it down. Every question the imagined
rebuilder would have to ask you is a hole.

**Why:** the second version survives a rewrite. The first is dead the day
`checks/` is renamed.

**Depth:** `.bee/expertise/documentation.md` § The rebuild test.
