---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Are `bootstrap-cockpit.sh` (245 lines) and `control-loop.sh` (438
lines) replaced by Rust as part of this effort, or left for later?

## Answer

`control-loop.sh` is replaced in this effort. `bootstrap-cockpit.sh` is
left for later.

`control-loop.sh` carries both Windows blockers — GNU coreutils
`timeout` and the bash-4.3 nameref (`local -n`) — so replacing it is
what actually unblocks Windows. It uses no signals, no job control and
no `mktemp`; what remains is flow control (a poll loop, a stop-file
check, argv assembly, a consecutive-failure ceiling with backoff) plus
a `claude` spawn, and it already delegates its JSON reading to
`bee herding command-template`. A Rust replacement is an extension of a
shape the script already assumes.

`bootstrap-cockpit.sh` only trips on `BASH_SOURCE`, is run once by hand
by the owner, and is not on the path of the first real scenario.
Deferring it is a recorded gap, not an oversight: at the end of this
effort the orchestrator and its control loop run on Windows, while
turning the cockpit on for the first time still wants a bash.

Logged as D08.
