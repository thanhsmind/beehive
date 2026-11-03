---
artifact_contract: bee-research/v1
topic: ui-ux-pro-max-into-bee-design-port
depth: standard
date: 2026-10-29
---

## Bottom Line

- Recommendation (ladder rung): adapt-upstream, narrow. Port only the WCAG 2.2 rules that bee-design lacks. Reject the catalogs, the search engine and the design-system generator.
- Why this is the lightest credible path: bee-design already covers about 90% of the source's ground, and most of the rest conflicts with bee-design's own taste rules.
- Why the next-best rung lost: a full port adds a Python runtime, a second design source of truth, and data that bee-design's `design-tells.md` marks as AI tells.
- Confidence: 80%
- Suggested next step: bee-shaping, one small cell, if the user picks the narrow port.

## Source Manifest

| Field | Value |
|---|---|
| Repo | https://github.com/nextlevelbuilder/ui-ux-pro-max-skill |
| Ref | default branch, shallow clone |
| Resolved commit SHA | 28b3b8dee8c976df4fa3e94aed4e092ad43fcb03 |
| Version | 2.15.0 (`skill.json`), MIT |
| Narrowed scope | `src/ui-ux-pro-max/` (data, scripts, templates) |

## Repo Snapshot

- Local: bee-design is six markdown-only skills in `packages/bee/design/` (core plus web, charts, motion, apple, figma packs). No skill ships a script.
- Local: the taste is vendored from design-os (PolyForm Shield 1.0.0) by `scripts/sync-design-os.sh`. That script runs `rm -rf "$dest/$d/references"` and re-copies. Any file added by hand under a pack's `references/` is lost on the next sync.
- Local: the only check is `bee design gate` (ease-design `ui gate`, Node 20).
- Upstream: the source is a Python 3 BM25 search CLI over 13 CSV catalogs plus 22 per-stack CSVs, with a skill prose template and a 10-section quick reference.

## Findings

### Local (dependency matrix, condensed)

| Source component | Verdict | Evidence |
|---|---|---|
| `ux-guidelines.csv`, `app-interface.csv`, `landing.csv`, `charts.csv`, `motion.csv` | EXISTS | `ux-psychology.md`, `accessibility.md`, `mode-constraints.md`, `page-structures.md`, `bee-design-charts/references/chart-craft.md`, `bee-design-motion/references/motion-craft.md` cover these as deep or deeper |
| Skill prose and quick reference | EXISTS, mostly | `bee-design/SKILL.md` Concept-Build-Check-Hand over-Adjust; about 35 of 40 checklist items already present |
| WCAG 2.2: focus not obscured, drag alternative, accessible authentication, auto-rotation pause, scrim contrast measured on the real background | NEW | absent from `bee-design/references/` |
| Error summary with focus and links to fields | PARTIAL | one line at `content-design.md:47` |
| `styles.csv` (88 named styles) | NEW, conflicts in content | no style taxonomy local |
| `products.csv`, `ui-reasoning.csv` (192 product types to style, scored) | CONFLICT | `need-routing.md` routes by need class through gates; 47 product rows recommend Glassmorphism |
| `typography.csv` (74 pairings) | CONFLICT | recommends Inter 9 times, Roboto 5, Space Grotesk 5, Plus Jakarta Sans 3; `design-tells.md:105` lists all four as `overused-font` |
| `colors.csv` (192 fixed hex palettes) | CONFLICT | `color-science.md` and `token-taxonomy.md` derive palettes from tokens; 25 rows use purple or violet, the `ai-color-palette` tell at `design-tells.md:75` |
| `design_system.py --persist` (writes `MASTER.md`) | CONFLICT | a second source of truth beside `DESIGN.md` and `design/*.json`; written by the agent on its own initiative |
| Design dials `--variance/--motion/--density` | CONFLICT | `taste-rubric.md:467` maps the dial model as legacy |
| `core.py`, `search.py` BM25 search | NEW | needs Python 3; no local skill ships scripts |
| `stacks/*.csv`, 16 stacks absent locally (flutter, compose, vue, svelte, wpf, ...) | NEW | out of design-taste scope; engineering guidance |
| `react-performance.csv` | NEW | engineering, not design taste |
| `google-fonts.csv` (1934 fonts) | NEW | a lookup, no taste |

### Cross-cutting sweep

- `scripts/sync-design-os.sh` owns every file under the six packs' `references/`. A port into those folders needs a bee-owned home the sync keeps.
- Onboarding pack lists: `packages/bee-rs/crates/bee/src/onboard/skills.rs:1040`, `onboard/tests.rs:19`, `tests/installer_contracts.rs:652`. A new pack touches the Rust installer and its tests.
- Licenses: `THIRD_PARTY_LICENSES.md` in each pack is also copied by the sync; an MIT row must survive it.
- Code fields in `react-performance.csv` and `stacks/*.csv` carry `//` comments, which trip the `no_code_comments` guard if copied into generated files.

### Challenge (port mode)

| Question | Source answer | Local answer | Risk if wrong | Flag |
|---|---|---|---|---|
| Does a product-type style table help taste? | yes, pick style by industry | no, pick a macrostructure by need, then decide | generic, template-looking pages | red |
| Is a font-pairing catalog safe? | yes | no, many entries are named tells | gate-clean pages that read as AI-made | red |
| Can a second design file live beside DESIGN.md? | MASTER.md plus page overrides | one SSOT, derive the rest | two files drift | red |
| Is a Python dependency acceptable? | yes, stdlib only | skills are markdown, check is Node | a new runtime in every host | red |
| Do the WCAG 2.2 rules fit? | yes | yes, same stance as `accessibility.md` | none found | green |
| Do the extra stacks belong in bee-design? | yes | bee-design is taste, not framework guidance | scope creep | red |

### Inference

- Only the accessibility gap is a clean gain. Everything else either duplicates or contradicts the design-os taste that bee-design carries.

## Risks, Unknowns, Follow-Ups

- Home for the new rules: the synced `references/accessibility.md` is wiped on sync. Options: a bee-owned file the sync script keeps, or upstream the rules to design-os.
- The 16 missing stacks are a real gap for non-web hosts, but belong in a separate decision, not in design taste.

## Source Pack

- Local: `packages/bee/design/*/SKILL.md`, `bee-design/references/{design-tells,generation-craft-defaults,mode-constraints,accessibility,content-design,color-science}.md`, `scripts/sync-design-os.sh`, onboarding pack lists above.
- Upstream: `src/ui-ux-pro-max/{data,scripts,templates}` at the SHA above.
- Docs: none needed.
