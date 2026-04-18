# Phase 21 Wave A — HARD-00 Audit Notes

Audit date: 2026-04-15
Auditor: gsd-executor (Wave A)

## Results

pre-push: PASS
ssrf: PASS

## Evidence — credential pre-push hook

- Script: scripts/hooks/test-pre-push.sh
- Exit code: 0
- Test cases green: 6/6
- Last-modified commits verified: f357c92, 5361fd5
- Token-in-repo grep (excluding scripts/hooks, .planning, target, .git): no matches
- Note: a real ATATT3 token exists in `.env` on disk, but `.env` is gitignored (confirmed via `git check-ignore -v .env` → `.gitignore:19`) and not tracked; `git grep` on tracked files returns zero hits. The pre-push hook correctly scans only committed file content, not the filesystem.

## Evidence — SSRF regression tests

- Command: cargo test -p reposix-confluence --test contract adversarial_ --locked
- Tests green: 3/3
  - adversarial_links_base_does_not_trigger_outbound_call
  - adversarial_webui_link_does_not_trigger_outbound_call
  - adversarial_host_in_arbitrary_string_field_is_ignored
- Decoy server pattern (.expect(0)) intact: 6 sites
- Last-modified commit verified: ea5e548
- Test output: `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.02s`

## Gaps

None. HARD-00 closes.

## Follow-ups for waves B–E

- Wave B may proceed with confidence that If-Match semantics in the sim are preserved.
- Wave C may proceed with confidence that Confluence adversarial-field handling is regression-safe.
- No new tasks spawned by this audit unless "Gaps" above is non-empty.
