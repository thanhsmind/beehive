---
name: bee-design-charts
description: "Use when building or critiquing charts, diagrams or multi-screen flows with bee-design."
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# bee-design-charts — the charts pack

Project type: charts, diagrams and multi-screen flows. This pack adds that taste to `bee-design`. Load
`bee-design` first: its path names and gate check apply here, and the rendered result comes back to its
Check step. When `bee-design` is not installed, say so.

| Need | Read |
| --- | --- |
| A standalone chart: the shared contract, honest encoding, token inheritance, accessible inline SVG | [chart-craft.md](references/chart-craft.md) |
| One chart grammar (bar, gantt, line, pyramid, quadrant, radar, scatter, timeline, venn) | [chart-grammars/](references/chart-grammars/) |
| A diagram: grammar routing, the admission bar, density, accessible inline SVG, critique | [diagram-craft.md](references/diagram-craft.md) |
| One diagram grammar (architecture, data flow, ER, flowchart, sequence, state, swimlane and more) | [diagram-grammars/](references/diagram-grammars/) |
| The monochrome 24x24 icon set for diagrams and charts, with per-icon attribution | [diagram-icons.md](references/diagram-icons.md) |
| A multi-screen flow: the flow.json model and the flow lint | [flow-craft.md](references/flow-craft.md) |

Commands: this pack adds the `ui` commands in [contract.md](references/contract.md) to the core contract. Call
them only with this pack loaded. Do not run the commands in its "Not for host work" section in a host.

Mark the root of a chart with `data-chart-owned="true"` and the root of a diagram with
`data-diagram-owned="true"`, and name a flow model `flow.json` or `<name>.flow.json`. If not, the design
gate never checks it.

License: the design-os taste in this skill is PolyForm Shield 1.0.0 — [LICENSE.md](LICENSE.md) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) in this skill; upstream https://github.com/jangtrinh/design-os.
