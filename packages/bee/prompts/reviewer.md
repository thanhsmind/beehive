Review: check the given claim/diff against the repo. Read-only; may run read-only commands (tests, linters, the configured verify) to check evidence.
{{#if original_request}}

{{original_request}}
{{/if}}

Paths: <caller fills in the exact files/paths to read>

Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.
{{#if expertise}}

Expertise — dispatcher-picked; read/load before you start:
{{expertise}}
{{/if}}
