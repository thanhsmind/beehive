#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ] || [ ! -d "$1/knowledge" ] || [ ! -d "$1/skills/design-os-designer" ]; then
  echo "usage: scripts/sync-design-os.sh <design-os-path>" >&2
  exit 2
fi

src="$(cd "$1" && pwd)"
dest="$(cd "$(dirname "$0")/.." && pwd)/packages/bee/design"
designer="$src/skills/design-os-designer"
repo_url="https://github.com/jangtrinh/design-os"
commit="$(git -C "$src" rev-parse --short=7 HEAD)"
version="$(sed -n 's/^  "version": "\([^"]*\)",$/\1/p' "$src/package.json" | head -n 1)"

skills=(bee-design bee-design-web bee-design-charts bee-design-motion bee-design-apple bee-design-figma)
entries_bee_design="accessibility.md color-science.md token-taxonomy.md taste-rubric.md design-tells.md content-design.md page-structures.md build-loop.md ux-psychology.md ds-guidelines.md design-review.md verification-honesty.md mode-constraints.md need-routing.md component-catalog.md generation-craft-defaults.md design-soul.md design-reuse-ladder.md qualified-delivery.md user-evidence.md visual-regression.md domain-packs/"
entries_bee_design_web="web-technique-craft.md web-techniques/ patterns/ pattern-cards.md signature-devices.md designmd-format.md ds-kit-onboarding.md"
entries_bee_design_charts="chart-craft.md chart-grammars/ diagram-craft.md diagram-grammars/ diagram-icons.md flow-craft.md"
entries_bee_design_motion="motion-craft.md gsap-motion-direction.md scroll-cinema-direction.md shader-gradient-direction.md shader-gradient/ canvas-effect-direction.md canvas-ui/ asset-production-orchestration.md gflow-hand.md delivery-assets.md"
entries_bee_design_apple="apple-swiftui-craft.md native-ios-craft.md native-ios/ native-ipados-craft.md native-ipados/ native-macos-craft.md native-macos/"
entries_bee_design_figma="figma-craft/ figma-agent-hand.md figma-ds-onboarding.md"
designer_copied="concepts.md rules.md contract.md"

tooling="upstream's own agent tooling and governance, not taste"
research="research inputs, not taste"
scripts_reason="designer Node script: bee's check is the ui gate, and the three-width look sheet is deferred to the bee backlog"
excluded=(
  "knowledge/README.md|$tooling"
  "knowledge/index.json|$tooling"
  "knowledge/capability-profiles.json|$tooling"
  "knowledge/authoring-standard.md|$tooling"
  "knowledge/librarian-loop.md|$tooling"
  "knowledge/recall-mind.md|$tooling"
  "knowledge/world-class-learning-loop.md|$tooling"
  "knowledge/versioning-semver.md|$tooling"
  "knowledge/design-agents.md|$tooling"
  "knowledge/prompt-modes.md|$tooling"
  "knowledge/prompt-plan-orchestration.md|$tooling"
  "knowledge/persona-index.md|$tooling"
  "knowledge/personas/|$tooling"
  "knowledge/sources/|$research"
  "knowledge/benchmarks/|$research"
  "skills/design-os-designer/SKILL.md|bee writes its own bee-design SKILL.md from this workflow, pointing at bee's paths and check"
  "skills/design-os-designer/build.mjs|$scripts_reason"
  "skills/design-os-designer/validate.mjs|$scripts_reason"
  "skills/design-os-designer/validate-checks.mjs|$scripts_reason"
  "skills/design-os-designer/look.mjs|$scripts_reason"
  "skills/design-os-designer/look-probe.mjs|$scripts_reason"
  "skills/design-os-designer/look-shots.mjs|$scripts_reason"
  "skills/design-os-designer/shared.mjs|$scripts_reason"
  "skills/design-os-designer/contract-render.mjs|$scripts_reason"
  "skills/design-os-designer/look.md|guide to the three-width look sheet, which is deferred to the bee backlog"
  "skills/design-os-designer/examples/|starter pages for the designer's build step, which bee does not carry"
  "skills/design-os-designer/ATTRIBUTION.md|provenance of the upstream skill text, which bee replaces with its own SKILL.md"
)

entries_of() {
  local var="entries_${1//-/_}"
  printf '%s\n' ${!var}
}

is_listed() {
  local want="$1" row
  for row in "${excluded[@]}"; do
    [ "${row%%|*}" = "$want" ] && return 0
  done
  return 1
}

unmapped=""
for path in "$src"/knowledge/* "$designer"/*; do
  name="${path##*/}"
  [ -d "$path" ] && name="$name/"
  case "$path" in
    "$designer"/*) rel="skills/design-os-designer/$name"; mapped=" $designer_copied " ;;
    *) rel="knowledge/$name"; mapped=" $(for s in "${skills[@]}"; do entries_of "$s"; done | tr '\n' ' ') " ;;
  esac
  case "$mapped" in *" $name "*) continue ;; esac
  is_listed "$rel" || unmapped="$unmapped $rel"
done
if [ -n "$unmapped" ]; then
  echo "sync-design-os: upstream entries with no home and no exclusion:$unmapped" >&2
  exit 1
fi

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

for d in "${skills[@]}"; do
  prog="$work/$d.sed"
  : > "$prog"
  for s in "${skills[@]}"; do
    for e in $(entries_of "$s"); do
      ee="${e//./\\.}"
      if [ "$s" = "$d" ]; then
        home="references/$e"
        link="$e"
      else
        home="$s/references/$e"
        link="../../$s/references/$e"
        case "$e" in */) ;; *) printf 's#](%s)#](%s)#g\n' "$ee" "$link" >> "$prog" ;; esac
      fi
      printf 's#\\.\\./\\.\\./knowledge/%s#%s#g\n' "$ee" "$link" >> "$prog"
      printf 's#knowledge/%s#%s#g\n' "$ee" "$home" >> "$prog"
    done
  done
  for row in "${excluded[@]}"; do
    x="${row%%|*}"
    case "$x" in
      knowledge/*)
        x="${x#knowledge/}"
        printf 's#knowledge/%s#design-os:%s#g\n' "${x//./\\.}" "$x" >> "$prog"
        ;;
      */SKILL.md) printf 's#](SKILL\\.md)#](../SKILL.md)#g\n' >> "$prog" ;;
      */) ;;
      *)
        printf 's#](%s)#](%s/blob/%s/%s)#g\n' "${x##*/}" "$repo_url" "$commit" "$x" >> "$prog"
        ;;
    esac
  done
  printf 's#`knowledge/` floors#`references/` floors#g\n' >> "$prog"
  printf 's#a `knowledge/` brain#a `references/` brain#g\n' >> "$prog"

  rm -rf "$dest/$d/references"
  mkdir -p "$dest/$d/references"
  for e in $(entries_of "$d"); do
    cp -R "$src/knowledge/${e%/}" "$dest/$d/references/"
  done
  if [ "$d" = bee-design ]; then
    for f in $designer_copied; do
      cp "$designer/$f" "$dest/$d/references/"
    done
  fi
  find "$dest/$d/references" -type f | while IFS= read -r f; do
    sed -f "$prog" "$f" > "$work/out"
    cat "$work/out" > "$f"
  done
  cp "$src/LICENSE" "$dest/$d/LICENSE.md"
  cp "$src/THIRD_PARTY_LICENSES.md" "$dest/$d/THIRD_PARTY_LICENSES.md"
done

cp "$src/LICENSE" "$dest/LICENSE.md"
cp "$src/THIRD_PARTY_LICENSES.md" "$dest/THIRD_PARTY_LICENSES.md"

{
  printf '{\n'
  printf '  "repo": "%s",\n' "$repo_url"
  printf '  "commit": "%s",\n' "$commit"
  printf '  "version": "%s",\n' "$version"
  printf '  "excluded": [\n'
  last=$((${#excluded[@]} - 1))
  for i in "${!excluded[@]}"; do
    row="${excluded[$i]}"
    sep=","
    [ "$i" -eq "$last" ] && sep=""
    printf '    { "path": "%s", "reason": "%s" }%s\n' "${row%%|*}" "${row#*|}" "$sep"
  done
  printf '  ]\n'
  printf '}\n'
} > "$dest/UPSTREAM.json"

echo "sync-design-os: $dest from $commit ($version)"
