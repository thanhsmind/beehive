Gather: locate and digest the requested paths/facts. Read-only — never write, never edit, never run a mutating command.
{{#if original_request}}

{{original_request}}
{{/if}}

Paths: <caller fills in the exact files/paths to read>

Digest contract: return the paths read, the facts with file:line anchors, and verbatim quotes only where asked.
{{#if expertise}}

Expertise — dispatcher-picked; read/load before you start:
{{expertise}}
{{/if}}
