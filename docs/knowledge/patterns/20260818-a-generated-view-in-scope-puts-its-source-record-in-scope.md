---
type: bee.pattern
title: A generated view in a cell's scope puts its source record in scope too
description: "When a cell names a file whose content is derived from a data record (a catalog rendered off a registry payload, a projection off a store), the record is the real edit surface and belongs in the cell's files — the derived file alone cannot change. Three deliveries hit this on the same pair (catalog off the command registry) before it was named: each cell listed only the derived file and then touched, and had to reserve, the source record as unplanned fallout."
timestamp: 2026-08-18
bee:
  id: pattern-20260818-generated-view-implies-source-record
  lifecycle: active
  areas: [rust-runtime]
  sources: ["wayfinding-flow cell wayf-6 (catalog entries() is data-driven off generated/registry_payload.json; cell listed only the catalog, reserved the payload mid-flight; trace .bee/cells/archive/wayfinding-flow/wayf-6.json, 2026-08-17)", "knowledge-distill-trigger cell kdt-2 precedent (same pair, flag count 150→153)", "docs/history/wayfinding-flow/promote-proposals.md pattern candidate"]
---

A cell scoped "bump the pinned flag count in the catalog" and named
only the catalog source file. But the catalog's entry list is
data-driven: it renders whatever the generated registry record
declares. The real change lived in the registry record; the catalog
was its view. The worker discovered this mid-flight, had to touch and
reserve a file its cell never named, and the same discovery had
already happened twice in sibling features on the same file pair.

**The rule:** when planning scopes a file that is a rendered or
derived view of a data record, the record goes into the cell's
`files` up front — the view alone cannot carry the change, and the
regen/render step that syncs the two belongs to the same cell. The
smell that catches it: the file you are told to edit contains a
comment or a name that says "generated", "derived", "pinned against",
or reads its rows from another artifact.
