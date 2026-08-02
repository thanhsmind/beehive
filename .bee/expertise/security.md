# Building it secure

Security is not a feature added at the end; it is a property of where you
put the checks. Almost every real breach traces back to one of a small
number of mistakes — a boundary that was never drawn, a permission that
was checked once instead of every time, data that was interpreted as
code, a credential that lived somewhere it could be read. This guide is
about those mistakes and where they hide.

Two boundaries on the work itself. Testing that probes a running system —
scanning, fuzzing, attempting an exploit — is done only against systems
you are authorized to test, and the authorization is confirmed before the
first request, not assumed from the fact that you have access. And a
vulnerability found in someone else's system is reported to them, never
demonstrated at their expense.

## Where to look

| Situation / goal | Entry |
|---|---|
| Starting any security thinking at all | Draw the boundary first |
| Deciding what a request is allowed to do | Authentication is not authorization |
| Writing a permission check | Check the object, every time |
| Choosing what to allow through an input | Validate at the boundary, allow-list first |
| Building a query, command, path, or markup from input | Injection is data mistaken for code |
| Rendering anything a user supplied | Encode for the destination |
| Storing or comparing a password | Passwords are hashed, never encrypted |
| Reaching for encryption, tokens, or randomness | Use the boring primitive |
| Issuing sessions or tokens | Trust nothing the client hands back |
| A credential, key, or token is needed by code | Secrets live outside the code |
| Granting access to a service, job, or database user | Least privilege applies to machines |
| Adding or updating a dependency | The supply chain is part of your code |
| Writing a log line or an error message | Log the event, never the secret |
| Deciding what to defend at all | Threat modeling in one sitting |
| Reviewing a change for security | Where to look first in a review |
| A vulnerability has been found in your system | Handling a live vulnerability |

## Draw the boundary first

When beginning any security reasoning → identify the trust boundaries
before anything else. A trust boundary is any line where data or control
passes between parties with different privileges: the network edge, the
browser and the server, one service and another, your process and a
subprocess, your code and a third-party library, the user's row and
another user's row.

Everything arriving from across a boundary is attacker-controlled until
proven otherwise, and "everything" is broader than it feels: request
bodies and headers, URL parameters and paths, cookies, file uploads and
their names and declared types, values you previously wrote to the client
and are receiving back, records another team wrote to a shared store,
webhook payloads, environment configuration on a shared host, and the
contents of any file whose path was influenced by input.

The single most useful sentence in a security discussion is *"which side
of the boundary is this check on?"* A check the client performs is a
usability feature. Only the check on the trusted side is a control.

## Authentication is not authorization

When handling a request → keep the two questions separate, because
conflating them is the most common serious flaw in real systems.
Authentication answers *who is this*. Authorization answers *may this
actor do this particular thing to this particular object*. A valid
session proves the first and says nothing about the second.

The pattern that follows is **deny by default**: the absence of an
explicit permission is a denial, never a gap. Build it so a new endpoint,
a new field, a new record type is unreachable until someone grants access
deliberately. The opposite arrangement — allow unless denied — fails
every time someone adds a surface and forgets the rule, and the failure
is silent and total.

Two corollaries. Enforce on the trusted side, always: a hidden menu item,
a disabled button, and a client-side role check are presentation, and the
request they were hiding can be sent by hand. And keep the permission
model small enough to reason about — a system whose access rules nobody
can state in a paragraph has access rules nobody can review.

## Check the object, every time

When a request names a resource — an id in a path, a key in a body, a
filename → verify that *this* actor may access *that* object, on every
request, at the point of use. Checking at the front door and then trusting
the identifier is the flaw behind an enormous share of real data
exposure: the caller changes the number in the URL and receives someone
else's record, because the code confirmed they were logged in and then
asked the store for whatever id it was handed.

Two habits make this durable rather than heroic. Scope the *query*, not
the result — fetch "this record belonging to this owner" in one operation
instead of fetching by id and then comparing owners afterward, because
the comparison is what gets forgotten in the next handler. And apply it
to every operation on the object, including the ones that feel harmless:
list, count, export, sub-resource, bulk endpoint, and the "check if this
exists" call whose response reveals existence.

In a system with tenants, this check is the tenant boundary. Everything
else about multi-tenancy is bookkeeping; this is the property that keeps
customers separate.

## Validate at the boundary, allow-list first

When accepting input → validate as it enters the trusted side, and define
what is *acceptable* rather than what is *forbidden*. A deny-list enumerates
the attacks you thought of; an allow-list enumerates what your system
actually needs, which is a much smaller and much more stable set.

Validate the shape completely: type, length, range, format, and
membership in a permitted set. Check length before doing expensive work
with a value, and cap sizes explicitly — a field with no maximum is a
memory budget donated to strangers, and the same is true of upload sizes,
array lengths, and pagination limits.

Two traps worth naming. **Normalize before validating, then use exactly
what you validated** — if you decode, unescape, or canonicalize a value
*after* checking it, you validated a different string than the one you
use. And **validation is not sanitization**: rejecting a bad value is a
boundary decision; making a value safe to *use* is a destination decision,
and it belongs at the destination, which is the next two entries.

## Injection is data mistaken for code

When building a query, command, path, or document out of values you did
not write → the whole family of injection flaws has one cause: a string
that mixes instructions with data, parsed by something that cannot tell
which is which. Escaping is a patch on that mistake. The fix is
structural — keep the data out of the instruction entirely.

- **Database queries** — pass values as parameters, never concatenate
  them into the statement. This holds for every query language, not only
  SQL, and it holds for the "safe-looking" case: an integer id from your
  own UI is still a value from across the boundary.
- **Shell commands** — prefer invoking the program directly with an
  argument list over composing a command line for a shell to parse. When
  a shell is genuinely required, the quoting rules are subtle enough that
  a single missed case is a full compromise.
- **File paths** — resolve the final path and verify it is inside the
  directory you intended; never assume that stripping `..` was enough.
  Path traversal is the same bug with a different parser.
- **Templates and markup** — see the next entry.
- **Deserialization** — never deserialize untrusted input into a format
  that can construct arbitrary objects or invoke code. Prefer a plain
  data format parsed into known shapes.
- **Anything that takes an expression** — a search DSL, a filter
  language, a spreadsheet formula, a URL you fetch on the caller's behalf.
  If a value can steer *what the system does* rather than *what it
  computes with*, it belongs in the same family.

The unifying question: *can this value change the structure of what gets
executed, rather than only its parameters?* When the answer is yes,
change the mechanism, do not add escaping.

## Encode for the destination

When placing user-supplied data into an output → encode it for the exact
context it lands in, at the moment it lands there. The same string is
harmless in one position and executable in another: inside HTML text,
inside an attribute, inside a URL, inside a script, inside a style,
inside a header, inside a generated document. There is no single
"sanitized" form that is safe everywhere, which is why sanitizing on
input and forgetting it later is unreliable.

Prefer mechanisms that encode automatically by construction, and treat
every escape hatch that inserts raw markup as a place a reviewer must
stop and justify. When rich content genuinely must be accepted, run it
through a real parser-based sanitizer with an allow-list of elements and
attributes — never a regular expression, which cannot parse the format it
is trying to filter.

Two adjacent leaks belong to the same rule. **Never reflect input into a
redirect, a header, or a filename without validating it against a known
set** — an open redirect and a header injection are both this mistake.
And do not let generated output reveal internals: a stack trace, a query,
or an internal hostname in an error page is an information leak that
makes every other attack cheaper.

## Passwords are hashed, never encrypted

When storing a password → use a purpose-built password hash with a work
factor and a per-password salt, and nothing else. Not encryption
(reversible by design, which is the wrong property), not a fast
general-purpose digest (built to be quick, which is exactly what an
attacker with your database wants), not your own construction.

The properties to insist on, whatever the current algorithm names are:
deliberately slow and memory-hard, salted per password so identical
passwords hash differently, and tunable so the cost can be raised as
hardware gets faster. Verify with the library's own comparison function,
which is constant-time — a naive equality check on secrets leaks their
contents through timing.

The same guide applies to anything password-shaped: recovery codes, API
tokens you store, and any secret you only ever need to *check*, never to
*read back*. Store the hash. If you can recover the original, so can
whoever reads your database.

## Use the boring primitive

When reaching for encryption, signing, tokens, or randomness → use the
well-reviewed implementation your platform or ecosystem already provides,
at its documented defaults. Cryptographic code fails silently: a
construction with a subtle flaw produces output that looks identical to
correct output, passes every test you would think to write, and is broken.
Nothing in your test suite will tell you.

Three concrete rules cover most of it. Use the platform's *cryptographic*
random source for anything security-bearing — tokens, ids, salts, nonces
— never the general-purpose random function, which is predictable by
design. Never reuse a nonce or an initialization vector with the same
key. And prefer an authenticated construction, so tampering is detected
rather than decrypted into garbage that the next layer parses.

The rule extends beyond crypto: authentication protocols, session
management, and permission frameworks are also things where the
well-trodden implementation beats the one you understand better because
you wrote it.

## Trust nothing the client hands back

When issuing a session or token → remember that everything given to a
client comes back modified sometimes. A value is only trustworthy if you
can verify it: signed by you and checked on every use, or a meaningless
handle whose meaning lives in your own store.

The properties worth being deliberate about: an expiry short enough that
a stolen credential has a limited life; a revocation path that actually
works before the expiry — a token you cannot revoke is a permission you
cannot withdraw; transport that cannot be observed; and storage on the
client that is not readable by injected script. Rotate the session
identifier when privilege changes, especially at login, so a value the
attacker planted earlier does not survive into an authenticated session.

Verify the claims you rely on, not the ones that are convenient — a token
that is validly signed by *someone* is not a token issued for *your*
system, for *this* audience, and still within its window. Check all of it,
with the library's verification path rather than by decoding and reading
fields.

## Secrets live outside the code

When code needs a credential, key, or token → it comes from the
environment or a secret store at run time. Never from the source, never
from a config file that is committed, never from a comment, a fixture, a
test file, or a sample script that "will be cleaned up later."

Three practices make the rule survive contact with reality. **Assume any
secret that has ever been committed is compromised** — rewriting history
does not un-copy it, and rotation is the only real remediation. **Give
each environment and each service its own credential**, so a leak has a
blast radius and a rotation has a scope. And **make the leak detectable**:
a check that refuses to commit secret-shaped strings catches the mistake
at the only moment when it is still cheap.

Secrets also leak sideways, through paths that feel unrelated: an error
message, a log line, a crash report, a debug endpoint, a URL query string
that lands in access logs and browser history, an environment dump on a
status page. Treat every place data is written down as a place a secret
can end up.

## Least privilege applies to machines

When granting access to a service, a job, a build, or a database user →
grant exactly what that component needs to do its work, and nothing else.
The account a background job uses to read one table should not be able to
drop it; a service that never deletes should not hold the permission; a
build that publishes one artifact should not hold credentials for
everything else.

This is what turns a compromise into an incident instead of a catastrophe.
The attacker's next move after any foothold is to use whatever credentials
that foothold holds, and the size of the damage is decided in advance by
how those credentials were scoped. The same reasoning applies to network
reach (what can this component talk to), to filesystem access, and to the
lifetime of the credential.

Check it from the other direction periodically: for each credential in
the system, name what would happen if it leaked today. Any answer longer
than a sentence is a scope that needs narrowing.

## The supply chain is part of your code

When adding or updating a dependency → understand that you are adding
code you did not write, that runs with your privileges, and that will
change under you. The judgment before adopting: is it maintained, is it
widely used, how much does it pull in transitively, and what would this
package need permission to do — because a formatting helper that reads
the network is a question, not a detail.

Then hold it in place: pin versions with a lockfile so builds are
reproducible, keep something watching for known vulnerabilities in what
you depend on, and review updates rather than accepting them
automatically — a compromised release of a package you already trust is
the attack that bypasses all of the judgment above.

Build and deployment inputs are part of the same chain. A pipeline that
fetches a script from the internet and executes it, a base image that
floats to whatever is newest, or a tool installed from an unverified
source is an untrusted party inside your trusted process.

## Log the event, never the secret

When writing a log line → aim for a record that lets you reconstruct what
happened without becoming a second copy of the data you are protecting.
Log the actor, the action, the object, the outcome, and the time.
Do not log credentials, tokens, keys, session identifiers, full payment
details, or personal data that has no business being in a searchable
store with a long retention and a wide audience.

The security-relevant events specifically worth recording: authentication
success and failure, permission denials, privilege changes, changes to
access rules themselves, and administrative actions. These are what turn
"we think something happened" into an answer, and they are almost always
found to be missing *during* the incident that needed them.

Assume logs are read by more people and kept for longer than anyone
intends, and make them tamper-evident where the record matters.

## Threat modeling in one sitting

When deciding what to defend → spend twenty minutes on four questions
before spending a week on controls. *What are we protecting* — name the
data and the capabilities that would actually hurt to lose. *Who would
want it* — an opportunistic scanner, a logged-in customer poking at ids,
an insider, a competitor, someone who compromised a dependency. *What
could they try* — walk each trust boundary and ask what crossing it
unauthorized would let someone do: pretend to be someone else, change
something they shouldn't, read something they shouldn't, deny service to
others, or do something with no record left behind. *What would we do
about it* — for each plausible one, mitigate, monitor, or accept out loud.

The output is a short list of specific risks with named dispositions, not
a document. Its value is almost entirely in the second question: teams
that skip it defend against the attacker they imagined, who is usually
more sophisticated and less relevant than the one they will get.

## Where to look first in a review

When reviewing a change for security → the highest-yield places are the
same every time, so start there rather than reading linearly:

- **Anything that crosses a boundary** — new endpoints, new parameters,
  new file reads, new outbound calls, new event consumers.
- **Every permission check the change touches**, and every one it should
  have touched. A new handler beside an existing one is the classic place
  where the authorization line was not copied.
- **String construction** feeding a query, a command, a path, or markup.
- **Changes to authentication, session handling, or crypto** — these get
  a slower read than anything else in the diff.
- **New dependencies**, and any change to how dependencies are resolved.
- **Anything that writes to a log, an error, or a response** that now
  carries data it did not carry before.
- **What the change made *reachable*** — a route registered, a flag
  defaulted on, a guard relaxed "temporarily."

A finding needs the same evidence standard as any other: name the input,
the path it takes, and the consequence. "This looks unsafe" is a place to
investigate, not a finding to file.

## Handling a live vulnerability

When a real vulnerability is found in a running system → stop the bleeding
first and understand it second. Reduce exposure with whatever is fastest
and reversible — disable the path, block the pattern, revoke the
credential — then work out the fix, because a system under active
exploitation is not a place for a considered redesign.

Then, in order: determine whether it was exploited and what was reached,
using the logs you hopefully have; rotate every credential the flaw could
have exposed, on the assumption that it did; fix the class, not only the
instance, by searching for the same shape everywhere else; and honor the
disclosure obligations you have to the people whose data it was. Write
down what let it ship — the missing check, the missing test, the boundary
nobody drew — because that answer, not the patch, is what prevents the
next one.
