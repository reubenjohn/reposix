← [back to index](./index.md) · phase 80 plan 01 · [← 06a](./06a-T04-integration-tests.md)

## Task 80-01-T04 (continued) — Verifier flip + CLAUDE.md update + per-phase push

> **Split note:** this chapter covers §§ 4b–4d (verifier flip, CLAUDE.md update,
> per-phase push) plus the verify/done contract. The integration test code
> (§ 4a) is in [06a-T04-integration-tests.md](./06a-T04-integration-tests.md).

### 4b. Verifier flip — runner re-grades catalog rows

Run each verifier shell to confirm they now exit 0:

```bash
bash quality/gates/agent-ux/mirror-refs-write-on-success.sh
bash quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh
bash quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
```

If any fail, diagnose + fix-forward; the verifier shells exercise the
same scenarios as the integration tests but at the shell layer (cargo
build + sim subprocess + actual git push), so they may surface
fixture-isolation issues the Rust integration tests miss.

Then run the catalog runner:

```bash
python3 quality/runners/run.py --cadence pre-pr
```

The runner re-grades the 3 mirror-refs rows from `FAIL` → `PASS` (the
verifier shells now exit 0). Inspect `quality/catalogs/agent-ux.json`
to confirm `status` updated for all 3 rows.

### 4c. CLAUDE.md update

Edit CLAUDE.md to document the `refs/mirrors/<sot-host>-{head,synced-at}`
namespace convention. Find the most natural insertion point — likely
in the § Architecture section (which already discusses the cache's
bare repo + ref namespaces like `refs/reposix/sync/...`) or § Threat
model (which discusses audit log + ref hygiene). Insert ONE
paragraph:

```markdown
**Mirror-lag refs.** `crates/reposix-cache/` writes two refs per
SoT-host on every successful single-backend push (and, post-P83, on
every successful bus push and webhook-driven mirror sync):

- `refs/mirrors/<sot-host>-head` — direct ref pointing at the cache's
  post-write synthesis-commit OID.
- `refs/mirrors/<sot-host>-synced-at` — annotated tag with
  message-body first line `mirror synced at <RFC3339>`.

Where `<sot-host>` is the SoT backend slug (`sim` | `github` |
`confluence` | `jira`). Refs live in the **cache's bare repo**, NOT in
the working tree's `.git/`. Vanilla `git fetch` brings them along via
the helper's `stateless-connect` advertisement; `git log
refs/mirrors/<sot>-synced-at -1` reveals when the mirror last caught
up. **Important (Q2.2 doc-clarity contract):** `synced-at` is the
timestamp the mirror last caught up to the SoT — it is NOT a "current
SoT state" marker. The staleness window the refs measure IS the gap
between SoT-edit and webhook-fire. Full docs treatment defers to P85
(`docs/concepts/dvcs-topology.md`).
```

The paragraph is ≤ 200 words. CLAUDE.md's progressive-disclosure rule
(≤ 40k chars) is comfortably preserved.

### 4d. Per-phase push (terminal)

Stage the CLAUDE.md edit + the catalog flip + commit + push:

```bash
git add CLAUDE.md quality/catalogs/agent-ux.json
git commit -m "$(cat <<'EOF'
docs(claude.md): document refs/mirrors/<sot>-{head,synced-at} namespace + Q2.2 staleness clarification (DVCS-MIRROR-REFS-01..03 close)

- CLAUDE.md § Architecture — gains a paragraph documenting the mirror-lag refs namespace convention + the Q2.2 doc-clarity contract carrier (full docs defer to P85)
- quality/catalogs/agent-ux.json — 3 mirror-refs rows status FAIL → PASS (re-graded by python3 quality/runners/run.py --cadence pre-pr)
- Updated in-phase per CLAUDE.md "CLAUDE.md update in same PR" rule (QG-07)

Phase 80 / Plan 01 / Task 04 part B / DVCS-MIRROR-REFS-01..03 (terminal).
EOF
)"

git push origin main
```

If pre-push BLOCKS:

1. Read stderr; the hook names the violated invariant.
2. Likely fail modes:
   - `cargo fmt` drift on T02 / T03 / T04 commits: run `cargo fmt --all`,
     NEW commit, re-push.
   - `cargo clippy` failure at workspace level (not caught by per-crate
     runs): a missing `# Errors` doc or a `clippy::pedantic` lint leaks.
     Diagnose, fix, NEW commit, re-push.
   - Catalog runner FAIL: a row regressed — diagnose; if the catalog
     flip was missed, re-run the runner; NEW commit if catalog needs
     updating.
   - Mermaid / docs-build gates fail: unrelated to P80 — file as
     SURPRISES-INTAKE if the failure pre-existed; otherwise fix in this
     phase.
3. NEVER `--no-verify` per CLAUDE.md git safety protocol.
4. NEVER `--amend` per CLAUDE.md git safety protocol — always NEW
   commit.

After the push lands, this plan is COMPLETE. The orchestrator
(top-level) takes over for:

- Verifier-subagent dispatch per `quality/PROTOCOL.md` § "Verifier
  subagent prompt template" (verbatim) — grades the 3 P80 catalog rows
  from artifacts with zero session context.
- Verdict file at `quality/reports/verdicts/p80/VERDICT.md`.
- STATE.md cursor advanced (P80 SHIPPED → next P81).
- REQUIREMENTS.md DVCS-MIRROR-REFS-01..03 checkboxes flipped `[ ]` → `[x]`.

NONE of those orchestrator-level actions are part of this plan; they
are top-level coordinator actions AFTER 80-01 T04 pushes.

<verify>
  <automated>cargo nextest run -p reposix-remote --test mirror_refs && bash quality/gates/agent-ux/mirror-refs-write-on-success.sh && bash quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh && bash quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh && python3 quality/runners/run.py --cadence pre-pr</automated>
</verify>

<done>
- `crates/reposix-remote/tests/mirror_refs.rs` exists with 4 tests.
- `cargo nextest run -p reposix-remote --test mirror_refs` exits 0; all
  4 tests pass.
- `bash quality/gates/agent-ux/mirror-refs-write-on-success.sh` exits 0.
- `bash quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh`
  exits 0.
- `bash quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh`
  exits 0.
- `python3 quality/runners/run.py --cadence pre-pr` exits 0; all 3 P80
  catalog rows status now `PASS`.
- `quality/catalogs/agent-ux.json` row statuses are PASS for the 3
  mirror-refs rows (the runner rewrites the file).
- CLAUDE.md has the `refs/mirrors/<sot>-{head,synced-at}` namespace
  paragraph in § Architecture (or the closest natural section).
- `git log -1 --oneline` shows the CLAUDE.md edit + catalog flip
  commit (the terminal commit).
- `git push origin main` exits 0 (push lands; pre-push GREEN).
- The remote `main` is up to date with the local.
- `cargo nextest run -p reposix-remote -p reposix-cache` exits 0 (full
  cache + helper suite GREEN).
- The CLAUDE.md update is documentationally accurate against the
  just-shipped behavior (no drift).
- Cargo serialized: T04 cargo invocations run only after T03's commit
  has landed; per-crate fallback used.
</done>
