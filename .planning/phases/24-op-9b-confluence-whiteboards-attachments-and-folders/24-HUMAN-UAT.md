---
status: partial
phase: 24-op-9b-confluence-whiteboards-attachments-and-folders
source: [24-VERIFICATION.md]
started: 2026-04-16T00:00:00Z
updated: 2026-04-16T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. whiteboards/ directory lists <id>.json entries
expected: `ls mount/whiteboards/` shows one or more `<id>.json` entries after mounting against a live Confluence space
result: [pending]

### 2. cat whiteboard JSON returns valid ConfWhiteboard
expected: `cat mount/whiteboards/<id>.json` returns valid JSON with id, title, space_id, created_at fields
result: [pending]

### 3. .attachments/ directory lists sanitized filenames
expected: `ls mount/pages/<id>.attachments/` lists attachment filenames with only [a-zA-Z0-9._-] characters
result: [pending]

### 4. cat attachment returns binary content
expected: `cat mount/pages/<id>.attachments/<file.png>` returns binary bytes (e.g. piped to `file -` shows correct MIME)
result: [pending]

### 5. >50 MiB attachment returns EFBIG
expected: reading an attachment with file_size > 52_428_800 bytes returns an error (EFBIG); no memory spike
result: [pending]

### 6. Folder-parented pages appear in tree/ hierarchy
expected: pages with parentType==folder appear nested under their parent in `mount/tree/` (not orphaned at root)
result: [pending]

## Summary

total: 6
passed: 0
issues: 0
pending: 6
skipped: 0
blocked: 0

## Gaps
