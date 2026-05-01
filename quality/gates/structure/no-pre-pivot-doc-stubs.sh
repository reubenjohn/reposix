#!/usr/bin/env bash
# P78-02 HYGIENE-02 — verifier for catalog row
# `structure/no-pre-pivot-doc-stubs`. TINY shape.
#
# Asserts every docs/<slug>.md whose byte size <500 is referenced in
# mkdocs.yml (nav: / not_in_nav: / redirect_maps section). Pre-pivot
# stubs (e.g. v0.4-era FUSE doc remnants) get caught here.
#
# Owner-hint on RED: add the stub to mkdocs.yml redirect_maps or remove
# the file.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" &> /dev/null && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." &> /dev/null && pwd)"
cd "${REPO_ROOT}"
[[ -d docs ]] || { echo "PASS: docs/ does not exist; nothing to check"; exit 0; }
[[ -f mkdocs.yml ]] || { echo "FAIL: mkdocs.yml missing — cannot verify" >&2; exit 1; }
UNMAPPED=()
while IFS= read -r -d '' stub; do
  size=$(wc -c < "${stub}")
  (( size >= 500 )) && continue
  base=$(basename "${stub}")
  grep -qF "${base}" mkdocs.yml || UNMAPPED+=("${base}")
done < <(find docs -maxdepth 1 -name '*.md' -type f -print0)
if (( ${#UNMAPPED[@]} > 0 )); then
  echo "FAIL: top-level docs/*.md stubs <500 bytes not referenced in mkdocs.yml: ${UNMAPPED[*]}" >&2
  echo "owner_hint: add to mkdocs.yml redirect_maps or remove" >&2
  exit 1
fi
echo "PASS: every docs/*.md stub <500 bytes is referenced in mkdocs.yml"
exit 0
