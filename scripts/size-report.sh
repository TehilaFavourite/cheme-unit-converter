#!/usr/bin/env bash
# Prints the gzipped size of the built web assets as a Markdown table and
# fails if they exceed the performance budget. Run after `npm run build`.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
assets="$root/web/dist/assets"

# Budgets for the gzipped download, in bytes.
wasm_budget=$((100 * 1024))
js_budget=$((20 * 1024))
css_budget=$((8 * 1024))

shopt -s nullglob
files=("$assets"/*.wasm "$assets"/*.js "$assets"/*.css)
if [[ ${#files[@]} -eq 0 ]]; then
  echo "error: no built assets in $assets; run npm run build first" >&2
  exit 1
fi

status=0
echo "| Asset | Raw (bytes) | Gzipped (bytes) | Budget (bytes) |"
echo "| --- | ---: | ---: | ---: |"
for file in "${files[@]}"; do
  raw="$(wc -c <"$file" | tr -d ' ')"
  gzipped="$(gzip -9 -c "$file" | wc -c | tr -d ' ')"
  case "$file" in
    *.wasm) budget=$wasm_budget ;;
    *.js) budget=$js_budget ;;
    *.css) budget=$css_budget ;;
  esac
  marker=""
  if ((gzipped > budget)); then
    marker=" **over budget**"
    status=1
  fi
  echo "| $(basename "$file") | $raw | $gzipped$marker | $budget |"
done
exit "$status"
