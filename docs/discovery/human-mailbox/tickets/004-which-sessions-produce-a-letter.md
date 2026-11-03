---
type: grilling
status: closed
claimed-by: none
blocked-by: none
---

## Question

Which runs produce a letter: (a) only unattended runs (herding and the
like); (b) every session, attended or not; (c) every session records
entries, but only unattended runs compose and file a letter.

➡️ Recommendation: (c). Recording is cheap and makes an attended
session that later turns into an overnight one seamless; filing a letter
for work the human watched happen just fills the inbox with things they
already saw.

## Answer

(c) — logged as **D9 (d970d6fc)**. Every session appends entries,
attended or not; only an unattended run composes and files a letter. A
session that starts attended and becomes an overnight run keeps a
complete record of its whole span.

Extended by **letter-digest D2 (aedb5be9)**: every `bee close` now
files its close letter at the moment of close, attended sessions
included — D9's rule stays only for the run-end letter, which still
waits on an unattended run.
