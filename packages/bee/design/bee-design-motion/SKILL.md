---
name: bee-design-motion
description: "Use when adding animation, scroll cinema, WebGL or gradient effects, or generated media assets to UI with bee-design."
metadata:
  version: '0.1'
  ecosystem: bee
  dependencies: []
---

# bee-design-motion — the motion pack

Project type: interfaces that move — animation, scroll-driven scenes, WebGL and gradient effects, and the
generated media they need. This pack adds that taste to `bee-design`. Load `bee-design` first: its
workflow, path names and gate check apply here. When `bee-design` is not installed, say so.

| Need | Read |
| --- | --- |
| Choose the animation tier, T1 CSS to T6 WebGL, and keep the motion floors | [motion-craft.md](references/motion-craft.md) |
| T5 GSAP timelines, ScrollTrigger, lifecycle and reduced motion | [gsap-motion-direction.md](references/gsap-motion-direction.md) |
| A scroll-scrubbed camera flight: architecture, seams, the scrub-encode and phone floors | [scroll-cinema-direction.md](references/scroll-cinema-direction.md) |
| A T6 ShaderGradient field: presets, surfaces, colour binding, the two fallbacks | [shader-gradient-direction.md](references/shader-gradient-direction.md), roster in [shader-gradient/](references/shader-gradient/) |
| A T6 Canvas UI WebGL effect: the one-per-viewport cap and the install handoff | [canvas-effect-direction.md](references/canvas-effect-direction.md), roster in [canvas-ui/](references/canvas-ui/) |
| Order and gate a multi-asset production run | [asset-production-orchestration.md](references/asset-production-orchestration.md) |
| Generate video or images through Google Flow | [gflow-hand.md](references/gflow-hand.md) |
| Resolve reproduced images to their originals | [delivery-assets.md](references/delivery-assets.md) |

Commands: this pack adds the `ui` commands in [contract.md](references/contract.md) to the core contract. Call
them only with this pack loaded. Do not run the commands in its "Not for host work" section in a host.

License: the design-os taste in this skill is PolyForm Shield 1.0.0 — [LICENSE.md](LICENSE.md) and [THIRD_PARTY_LICENSES.md](THIRD_PARTY_LICENSES.md) in this skill; upstream https://github.com/jangtrinh/design-os.
