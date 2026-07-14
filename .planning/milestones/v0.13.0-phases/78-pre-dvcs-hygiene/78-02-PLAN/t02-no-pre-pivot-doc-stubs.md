← [back to index](./index.md)

# Task 02-T02 — Author `no-pre-pivot-doc-stubs.sh`

<read_first>
- `quality/gates/structure/freshness-invariants.py:303-335` (Python impl).
- `quality/catalogs/freshness-invariants.json:361-397` (row's `expected.asserts`).
- `mkdocs.yml` (top of file — confirms the file exists; the .sh greps it).
</read_first>

<action>
Create `quality/gates/structure/no-pre-pivot-doc-stubs.sh`. The Python branch
has slightly more nuance (it checks `redirect_maps` + `nav` + `not_in_nav`);
the simplest correct shell translation is "stub <500 bytes must appear
somewhere in mkdocs.yml" (substring match on the basename — same simplification
the Python branch already uses at line 325):

```bash
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
```

Make executable. Smoke-test exits 0.

Edit `freshness-invariants.py` line ~303 — add the path-forward one-line
comment above `def verify_no_pre_pivot_doc_stubs`.
</action>

<acceptance_criteria>
- File exists, executable, between 5 and 30 lines (will be ~24 lines).
- `bash quality/gates/structure/no-pre-pivot-doc-stubs.sh` exits 0 + prints `PASS: ...`.
- Synthetic FAIL smoke: `printf 'x' > docs/fake-stub.md && bash <script>; rc=$?; rm docs/fake-stub.md; [[ $rc == 1 ]]` succeeds.
- `freshness-invariants.py` has path-forward comment.
</acceptance_criteria>
