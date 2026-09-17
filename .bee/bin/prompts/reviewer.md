Review: check the given claim/diff against the repo. Read-only; may run read-only commands (tests, linters, the configured verify) to check evidence.
The dispatcher names one review lens (Purpose) in the purpose/prompt; review through that lens only, and follow review.md for what a finding is and how to set severity.
{{#if original_request}}

{{original_request}}
{{/if}}
{{#if paths}}

Paths:
{{paths}}
{{/if}}
{{#if purpose}}

Purpose:
{{purpose}}
{{/if}}

Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.
{{#if expertise}}

Expertise — dispatcher-picked; read/load before you start:
{{expertise}}
{{/if}}
