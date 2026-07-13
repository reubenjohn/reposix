# v0.14.0 GOOD-TO-HAVES

> **Purpose.** OP-8 +2 reservation Slot 2 — improvements (clarity, perf, consistency,
> grounding) the planned phases observed but didn't fold in. Sized XS / S / M; XS items
> always close; M items default-defer to the next milestone. Drained by P111
> (good-to-haves polish + milestone close, Slot 2 of the v0.14.0 +2 reservation).

_Append below this line as polish opportunities surface._

## GOOD-TO-HAVES-01 — `leaf-isolation-nontool-backstop` — non-tool backstop for guards B/C

**Discovered during:** P102

**Size:** M (rough effort estimate)

**Source:** the P102 leaf-isolation guards live in a PreToolUse Bash-*tool* hook
(`.claude/hooks/leaf-isolation-guard.sh`). By construction the hook fires only on the
Claude Code Bash tool — a `reposix init` / `git config` write spawned by a subprocess or a
shell script bypasses it entirely. The `.githooks/pre-commit` git-native backstop closes
the *commit* leg on that bypass path (guard A), but there is NO non-tool backstop for guard
B (leaf-setup location) or guard C (shared-`.git/config` write): a script that shells out
`reposix init .` or `git config core.bare true` in the shared tree is unguarded.

**Acceptance:** a non-tool enforcement point catches guard-B/C violations regardless of how
the command is spawned. Candidate designs: (a) a `reposix`-binary-side check — `reposix
init|attach|sync` refuses to run when cwd resolves inside a repo whose `origin` is the
coordinator's shared checkout and no explicit `--allow-shared` / `/tmp` target is given;
(b) a git alias / `core.fsmonitor`-adjacent wrapper that intercepts `git config core.bare`
in the shared repo; (c) a filesystem-level guard (read-only bind of the shared `.git/config`
during autonomous runs). Proven by a transcript showing a scripted (non-Bash-tool) bypass
attempt is blocked.

**Default disposition for P111:** M default-defers to the next milestone with a named
carry-forward target (v0.15.0 fleet-safety hardening); close early only if a cheap
binary-side check (design (a)) proves < 1h.

**STATUS:** OPEN

## GOOD-TO-HAVES-02 — `file-size-drain-residual` — 56 files still over the progressive-disclosure budget

**Discovered during:** P103 Lane B (OP-8 file-size debt drain)

**Size:** M (rough effort estimate — 56 files across three extensions)

**Source:** `bash quality/gates/structure/file-size-limits.sh` (blocking mode). Lane B
split the two BIG offenders — `.planning/milestones/v0.13.0-phases/{SURPRISES-INTAKE,
GOOD-TO-HAVES}.md` (109K + 129K) — into per-part child dirs (`surprises-intake/`,
`good-to-haves/`, byte-exact, INDEX rewritten), dropping the count 58 → 56 and adding the
reusable `scripts/split_ledger.py`. The residual 56 (45 `*.md` / 6 `*.py` / 5 `*.sh`) were
deliberately deferred, each for a concrete reason:

- **6 `*.py` runners** — `quality/runners/{run,verdict,_audit_field,test_audit_field,
  test_realbackend}.py` + `scripts/shell_coverage.py`. `_audit_field.py` + `test_audit_field.py`
  were being modified by a CONCURRENT cargo/runner lane during P103; splitting them is a
  module refactor with collision risk (charter #3 — new coupling, not <1h-safe). Sketch:
  extract `_freshness.py`/`_row.py` helpers from `run.py`/`verdict.py`; move `_audit_field`
  parsing into a sibling module; split the two big test files by fixture group.
- **5 `*.sh` gates/hooks** — `.claude/hooks/leaf-isolation-guard.sh`, `.githooks/test-pre-push.sh`,
  `quality/gates/agent-ux/{dark-factory/dvcs-third-arm,real-git-push-e2e,zero-shot-onboarding}.sh`.
  Each is a single-responsibility script; splitting needs a `lib/` source-helper convention
  (new dependency between scripts) — factor in one focused pass, not mid-flight.
- **`.planning/milestones/*/ROADMAP.md` (v0.13.0 60K, v0.13.2 30K, v0.14.0 30K)** — GSD
  tooling + the `no-loose-roadmap-or-requirements` invariant couple to the monolithic
  ROADMAP.md path; splitting risks the layout rule (`.planning/CLAUDE.md` § Milestones
  layout). Defer until a ROADMAP-shard convention exists.
- **Remaining `*.md`** — archived phase bundles / research chapters (`.planning/phases/89-91/*`,
  `.planning/research/v0.13.0-real-backend-frictions/*`), live planning ledgers
  (`STATE.md` — concurrent-writer, never hand-edit; `PROJECT.md`, `CONSULT-DECISIONS.md`,
  `ORCHESTRATION.md`, `RETROSPECTIVE.md`), and docs (`docs/guides/troubleshooting.md`,
  `docs/reference/cli.md`, `quality/PROTOCOL.md`). Each is a real split but per-file
  judgement; `scripts/split_ledger.py` handles the `## `-delimited ledgers directly.

**Acceptance:** `bash quality/gates/structure/file-size-limits.sh` exits 0 with NO waiver.
Drive the residual toward zero the same way Lane B did the intakes: reuse
`scripts/split_ledger.py` for ledgers, factor a `lib/` convention for the `*.sh`, extract
runner helpers for the `*.py`. The `structure/file-size-limits` waiver in
`freshness-invariants.json` was NARROWED by Lane B to name only these residual files (the
two split intakes were removed from it).

**Default disposition for P111:** M default-defers to the next milestone (post-v0.14.0);
close individual extensions early (e.g. the 5 `*.sh` in one pass) if < 1h each.

**STATUS:** OPEN

## GOOD-TO-HAVES-03 — `gh404-caller-path-raw-slug-guard` — caller-path guard that a caller cannot re-sanitize before Cache::open

**Discovered during:** P104 remediation (S-260707-gh404 WARNING #3)

**Size:** M (needs a backend-injection test seam into the caller path)

**Source:** the GitHub 404 bug actually lived in the CALLERS — `reposix-remote`
`main.rs:293`, `reposix-cli` `sync.rs:105`, `attach.rs:164` all fed a slug to
`Cache::open`. The landed regression test
(`crates/reposix-cache/tests/github_project_slug_not_sanitized.rs`) calls
`Cache::open(_, "github", "owner/repo")` DIRECTLY, so it proves the cache honors a raw
slug but BYPASSES the callers — it would not catch a caller re-introducing
`sanitize_project_for_cache(&parsed.project)` before the `Cache::open` call. Partial
coverage already exists at the PARSE seam: `backend_dispatch::tests::parse_remote_url_github`
pins `parsed.project == "reubenjohn/reposix"` (raw). The gap is the segment BETWEEN parse
and `Cache::open`.

**Acceptance:** an integration test drives a real caller path (helper `main`, `sync`, or
`attach`) with a RECORDING backend and a `github::owner/repo` spec, and asserts the backend
receives the raw `owner/repo` at `list_records_complete` — i.e. no caller sanitizes between
`parse_remote_url` and `Cache::open`. Blocked today because those callers instantiate the
connector INTERNALLY from config + env (`backend_dispatch::instantiate`, and the GitHub
connector needs `GITHUB_TOKEN`), with no seam to inject a recording backend — closing this
needs a `#[cfg(test)]` backend-injection hook (or a thin `open_cache_with(backend, spec)`
extraction) shared by the three call sites. That refactor is new scaffolding + a new
test-only coupling across two crates (charter #3 — not <1h-safe, hence filed not fixed).
A cheaper stopgap if the refactor slips: a `structure`-dimension grep gate asserting no
`sanitize_project_for_cache` call appears on the line-path between `parse_remote_url` and
`Cache::open` in the three caller files.

**Default disposition for P111:** M default-defers to the next milestone (post-v0.14.0)
with carry-forward target v0.15.0; close early if the `open_cache_with` extraction proves
< 1h across the three call sites.

**STATUS:** OPEN

## GOOD-TO-HAVES-04 — `rbf-lr-03-file-size-split` — split the two P105 files over the soft-warn threshold

**Discovered during:** P105 (RBF-LR-03 docs fix-twice lane, ownership noticing)

**Size:** S

**Source:** the P105 fix wave produced two files over the `structure/file-size-limits`
soft-warn threshold: `crates/reposix-remote/src/fast_import.rs` (~42.7k, parent-chaining +
`deleteall` rebuild + its `#[cfg(test)]` module) and
`quality/gates/agent-ux/rebase-recovery-reconciles.sh` (~30.7k, two drift scenarios + the
negative-guard blocks that prove each assert bites). Both are currently WAIVED to 2026-08-08
in `freshness-invariants.json` — a time-boxed soft warn, not a hard fail.

**Acceptance:** when the 2026-08-08 waiver expires, both files sit under the soft-warn
threshold with NO waiver. Split `fast_import.rs` by extracting the `#[cfg(test)]` module
into a sibling `fast_import_tests.rs` (and, if still over, factor the import-stream builder
from the parent-resolution logic); split `rebase-recovery-reconciles.sh` by extracting the
negative-guard blocks (parentless non-descendant guard, deletion overlay-vs-rebuild guard)
into a sourced `lib/` helper shared with the scenario body. Re-run
`bash quality/gates/structure/file-size-limits.sh` → exit 0 without a waiver.

**Default disposition for P111:** S closes-or-defers; safe to defer until the waiver
approaches expiry (2026-08-08) since it is a soft warn, not a live fail. Close early if a
mechanical `#[cfg(test)]`-extraction pass proves < 1h.

**STATUS:** OPEN

## GOOD-TO-HAVES-05 — `shell-verdict-determinism-unittest` — regression test for the canonical deterministic verdict schema

**Discovered during:** P102 fleet-safety persist-gate fix (commit `309f0b6`, D-P96-01 extended)

**Size:** S (rough effort estimate)

**Source:** `quality/runners/_shell_verdict.py` (new module) is the single source of truth
for the canonical deterministic verdict — the exact key order + `json.dumps(indent=2)+"\n"`
byte shape that both producers (`quality/gates/agent-ux/lib/transcript.sh` and `run.py`'s
write-back) funnel through to STOP the fleet-safety JSON re-dirty bug. That byte-determinism /
key-order contract is currently proven only empirically (the 2×-clean pre-push gate run), NOT
by a unit test: `quality/runners/test_run.py` / `test_verdict.py` have zero references to
`_shell_verdict` / `canonical_verdict`. A future refactor of key order or `dumps()` formatting
would silently reintroduce the re-dirty bug with no test catching it.

**Acceptance:** `quality/runners/test_shell_verdict.py` exists and asserts byte-identical
output + stable key order for a fixed input (and, ideally, that two serializations of the
same dict are byte-equal). Runs green under the existing runner unit-test suite.

**Default disposition for P111:** S closes-or-defers; close early (< 1h, no new dependency —
a focused pytest against the existing module).

**STATUS:** OPEN

## GOOD-TO-HAVES-06 — `run-py-anti-bloat-cap` — `run.py` exceeds the 15000-char anti-bloat cap

**Discovered during:** P102 fleet-safety persist-gate fix (commit `309f0b6`, D-P96-01 extended)

**Size:** S (rough effort estimate)

**Source:** `quality/runners/run.py` is now ~24063 chars → PRE-EXISTING pre-commit WARN on
every commit (already visible each commit; filed so it is tracked as intended debt, not just
noise). `_audit_field.py` / `_realbackend.py` were split out for exactly this reason; `run.py`
still needs a further extract. Related to (and narrower than) GOOD-TO-HAVES-02, which tracks
the broader `structure/file-size-limits.sh` residual — this row names the specific 15000-char
anti-bloat WARN on `run.py` post-persist-gate wiring.

**Acceptance:** `run.py` drops back under the 15000-char anti-bloat cap with no WARN. Sketch:
extract the shell-subprocess / verdict-writing helpers (and/or the new canonical-verdict
wiring) into a submodule, mirroring the `_audit_field` / `_realbackend` splits.

**Default disposition for P111:** S closes-or-defers; safe to defer (a WARN, not a fail).
Close early if a mechanical helper-extraction pass proves < 1h.

**STATUS:** OPEN

## GOOD-TO-HAVES-07 — `fleet-safety-verdict-gitignore-exception` — `!`-exception inconsistency for the 3 fleet-safety verdicts

**Discovered during:** P102 fleet-safety persist-gate fix (commit `309f0b6`, D-P96-01 extended)

**Size:** XS (rough effort estimate)

**Source:** the p93 tracked verdicts are explicitly `!`-excepted in `.gitignore`, but the 3
`quality/reports/verifications/agent-ux/fleet-safety-*.json` are tracked only because of a
past `git add -f` — they are NOT `!`-excepted. They work today, but a fresh clone + a stray
`git rm --cached` could silently un-track them, leaving the fleet-safety verdicts ungrounded.

**Acceptance:** explicit `!`-exception lines for the 3 `fleet-safety-*.json` verdict paths
added to `.gitignore`, matching the p93 convention; `git check-ignore` confirms none are
ignored.

**Default disposition for P111:** XS always closes.

**STATUS:** OPEN

## GOOD-TO-HAVES-08 — `stale-rfc3339-transcript-sweep` — pre-existing per-run RFC3339 transcripts never swept

**Discovered during:** P102 fleet-safety persist-gate fix (commit `309f0b6`, D-P96-01 extended)

**Size:** XS (rough effort estimate — cosmetic)

**Source:** the persist-gate fix switched the 3 fleet-safety gates to a STABLE transcript
filename (`<slug>.txt`, gitignored, overwritten per run), which stops NEW accumulation — but
pre-existing `...-<RFC3339>.txt` transcripts from prior runs are gitignored and never swept,
so they linger on disk.

**Acceptance:** a one-time cleanup removes the orphaned RFC3339-stamped transcripts, and
(optionally) the gate lib gains a sweep step that clears orphaned per-run transcripts on each
run so the residue cannot re-accumulate.

**Default disposition for P111:** XS always closes.

**STATUS:** OPEN

## GOOD-TO-HAVES-09 — `slug-to-id-durable-create-model` — model create as a durable slug→id translation (interrupted-create duplicate elimination)

**Discovered during:** P108 (paperwork-closure filing of the ADR-010 slug→id waiver as a first-class intake remainder — the item was documented OPEN in ADR-010 §3 but never carried a severity+sketch row here)

**Size:** M (design-level, multi-crate — the v0.14.0 reconciliation-redesign headline pivot)

**Severity:** MEDIUM-HIGH — data-integrity hazard, but confined to id-reassigning REAL backends (GitHub Issues / JIRA / Confluence), recoverable by hand-deleting one duplicate; sim + client-id backends are unaffected. Owner-signed, WAIVED-until-v0.14.0 known limitation.

**One-line hazard:** a `create` against an id-assigning real backend that is cut off mid-push can, on retry, leave one duplicate record — because ADR-010's convergence contract ("already-landed writes are diffed away against the recomputed base") holds for UPDATEs (stable ids) but is FALSE for CREATEs, whose backend-assigned id is unknown until the interrupted call completes, so the retry cannot recognize its own prior landing.

**Fix sketch:** redesign reconciliation to model a create as a durable **slug→id translation** in the data model — the client mints a stable local slug (or intent-token) BEFORE the push, the commit-sequence models the create as "slug X → (pending) → backend id N", and an interrupted create leaves a well-defined resumable state (a pending slug with no confirmed id) that the retry reconciles against instead of blindly re-creating. Concretely: commit-sequence modeling with a slug→id map persisted alongside `oid_map`, so a partial create is idempotently continuable.

**Pointer:** ADR-010 §3 (`docs/decisions/010-l2-l3-cache-coherence.md` — `SotPartialFail` recovery + the WAIVED known-limitation marker); user-facing framing in `docs/concepts/dvcs-topology.md:206` + `docs/guides/troubleshooting.md:403`; root-cause diagnosis in `.planning/debug/p93-partial-fail-recovery-real-confluence.md:60`; carried OPEN in `.planning/milestones/v0.14.0-phases/RELIEF-HANDOVER-C2-wave-2.md:89`.

**Default disposition for P111:** M default-defers — this is the v0.14.0 reconciliation-redesign headline pivot itself, not a Slot-2 polish item; it graduates to a dedicated design phase, not an eager-fix. **Explicitly NOT cleared by P108** (P108 is paperwork closure of the SEPARATE, already-shipped Fork-A prune-completeness gate; this slug→id item remains fully OPEN and unstarted).

**STATUS:** DEFERRED-TO-v0.15.0 — owner scope call, 2026-07-12 (explicit deferral, not a silent slip; see ROADMAP.md Phase 108 headline note and root `.planning/GOOD-TO-HAVES.md` GOOD-TO-HAVES-09).

## GOOD-TO-HAVES-10 — `p94-forkb-assert-orphaned-on-forka-row` — Fork-B NotFound-idempotency assert has no mapped test under the Fork-A row's command

**Discovered during:** P108 (verifier finding, 2026-07-11)

**Size:** S

**Severity:** LOW-MEDIUM — latent honesty-gate mismatch, not a live failure (the gate currently reads PASS; the mismatch bites when `agent-ux/test-name-vs-asserts` next grades this row).

**One-line hazard:** the Fork-B "delete-time NotFound idempotency … (unit test)" assert lives on the Fork-A prune-completeness row but has no Fork-B test under that row's declared `command` (`cargo test -p reposix-cache --test pagination_prune_safety` tests ONLY Fork-A completeness gating), violating the `test-name-vs-asserts` honesty gate — every `expected.asserts` entry must map to a test the row's command actually runs.

**Fix sketch (option (a) — Fork B DID ship, verified below):** relocate `asserts[3]` to its own catalog row whose `command` runs the Fork-B unit test. Fork B is shipped AND tested — `is_delete_notfound` + `delete_of_already_absent_record_is_idempotent_success` live in `crates/reposix-remote/src/main.rs` (~L577/L597/L736), run by `cargo test -p reposix-remote`, NOT by the Fork-A cache command. So the assert simply needs its own row citing the `-p reposix-remote` command. Option (b) (remove-and-track-pending) does NOT apply here — Fork B is not unshipped. **Also fix the stale source path:** the row's `sources` + `owner_hint` cite `crates/reposix-remote/src/write_loop.rs` (and asserts[3] text says "write_loop.rs") for the Fork-B logic, but `reposix-cache` has NO `write_loop.rs` and the remote `write_loop.rs` has no NotFound handling — the real home is `reposix-remote/src/main.rs::execute_action`. Correct the pointer when relocating.

**Pointer:** `quality/catalogs/agent-ux.json` row `agent-ux/p94-pagination-prune-completeness-gate` `expected.asserts[3]` (~L2089); P108 verifier finding 2026-07-11. Real Fork-B code+test: `crates/reposix-remote/src/main.rs:517,597,736`.

**Default disposition for P111:** S closes-or-defers; close early (< 1h — a single catalog-row split + one source-path correction, no code change since Fork B already ships and passes under `-p reposix-remote`).

**STATUS:** OPEN

## GOOD-TO-HAVES-11 — `05-blob-limit-noncone-sparse-warnings` — leading-slash the sparse-checkout paths to silence NON-CONE warnings

**Discovered during:** P106 (verifier noticing, 2026-07-12)

**Size:** S

**Severity:** LOW — cosmetic. Selection is correct and the recovery works; git just prints 4 scary `NON-CONE PROBLEMS` warnings a skeptical first-time dev reads as breakage.

**One-line hazard:** `examples/05-blob-limit-recovery/run.sh` calls `git sparse-checkout set` with non-cone patterns lacking a leading slash (e.g. `issues/1.md`), so git emits `warning: pass a leading slash before paths such as 'issues/1.md'` — non-fatal but alarming in a demo meant to showcase clean recovery.

**Fix sketch:** add a leading `/` to each sparse-checkout path in `examples/05-blob-limit-recovery/run.sh` (git's recommended non-cone form — `/issues/1.md`). Identical selection, silences all 4 warnings. < 1h, no new dependency.

**Pointer:** `examples/05-blob-limit-recovery/run.sh` (the `git sparse-checkout set` invocation); P106 verifier noticing 2026-07-12.

**Default disposition for P111:** S closes-or-defers; close early (< 1h — a mechanical path-prefix edit in one demo script, no code change).

**STATUS:** OPEN

## GOOD-TO-HAVES-12 — `orphan-quinn-lockfile-entries` — drop unreachable quinn/quinn-proto/quinn-udp from Cargo.lock

**Discovered during:** P107 (RUSTSEC posture noticing, 2026-07-12)

**Size:** XS

**Severity:** LOW — harmless. `quinn 0.11.9`, `quinn-proto 0.11.15`, and `quinn-udp 0.5.14` survive only as **orphan `Cargo.lock` entries** — no crate in the current feature-resolved dependency tree depends on them (`cargo tree -i quinn-proto` prints "nothing to print" under all targets/features), so they are never built. They tripped a false "is RUSTSEC-2026-0185 reachable?" investigation in P107 that resolved to transitive-and-absent.

**One-line hazard:** stale lockfile entries invite a future agent to re-investigate a phantom advisory (as P107 did) and muddy `cargo tree` reachability audits.

**Source:** P107 evidence artifact `.planning/milestones/v0.14.0-phases/evidence/p107-cargo-audit-2026-07-12.txt` (`cargo tree -i quinn-proto` → "nothing to print"; orphan entries named in Cargo.lock).

**Acceptance:** a `cargo update` / lockfile regen drops the three orphan entries; `grep -E 'name = "quinn' Cargo.lock` returns nothing (or only genuinely-reachable entries). No behavior change — the crates are already never compiled. **Do NOT run `cargo update` opportunistically** — it must land as its own reviewed lockfile-hygiene change under the one-cargo-at-a-time budget (a broad regen can shift many pins).

**Default disposition for P111:** XS always closes — but gate it behind a dedicated lockfile-regen review, not an incidental edit.

**STATUS:** OPEN

## GOOD-TO-HAVES-13 — `mechanical-gate-artifact-empty-asserts` — mechanical real-backend gate artifacts record empty asserts even on genuine PASS

**Discovered during:** B3 (v0.14.0 tag-remediation, attach-sync re-run, 2026-07-13)

**Size:** S

**Source:** B3 noticed `quality/reports/verifications/agent-ux/{attach-sync-real-backend,p93-partial-failure-recovery-real-confluence}.json` show `asserts_passed: []` / `asserts_failed: []` even on a real exit-0 PASS. The `ASSERT <name>: PASS/FAIL` line convention that `lib/transcript.sh` greps is bash-scenario-only; plain `cargo test` never emits it (intentional per D2's transcript.sh comment). Net: the artifact JSON alone carries zero signal about WHAT passed — you must open the transcript to see `test result: ok. N passed`.

**Acceptance:** mechanical-kind cargo-test gate artifacts capture at least the cargo `test result: ok. N passed; M failed` summary (or the pass/fail counts) in a structured field, so the artifact is self-describing without the transcript. No change to gate pass/fail semantics.

**Default disposition for P111:** S closes-or-defers — carry-forward target v0.15.0 observability lane.

**STATUS:** OPEN

## GOOD-TO-HAVES-14 — `confluence-comment-unreadable-adf-page-url` — comment id fed into a page-shaped unreadable-ADF recovery message

**Discovered during:** items 4a/4b code review (2026-07-13)

**Size:** XS

**Severity:** LOW — comments are read-only today (never pushed), so the mis-typed recovery
message never fires on a write path; it can only surface inside a fetched, unreadable comment
body.

**Source:** `crates/reposix-confluence/src/types.rs:241` (`Comment::body_markdown`) passes a
**comment** id into `crate::adf::unreadable_adf_body(root_type, &self.id)`, whose parameter is
named `page_id` and whose teaching text (`adf.rs:109`) says "open page {page_id} in your browser"
and builds a `/wiki/api/v2/pages/{page_id}?body-format=storage` recovery URL (`adf.rs:112`) — the
wrong entity type for a comment (comments live under `/pages/{id}/{inline,footer}-comments`, not
`/pages/{id}`). A reader following the recovery URL for an unreadable comment lands on the parent
page, not the comment.

**Acceptance:** the unreadable-ADF recovery text/URL is kind-aware (page vs comment) — either
`unreadable_adf_body` takes an entity-kind + id and renders the correct `/pages/{id}` vs
`/pages/{page}/…-comments` URL, OR the comment caller suppresses the page-URL and names the
comment id without a misleading `/pages/{id}` link.

**Default disposition for P111:** XS closes-or-defers; safe to defer (dead path today — comments
never push). Close early if a kind-aware signature proves < 1h.

**STATUS:** OPEN

## GOOD-TO-HAVES-15 — `attach-seed-nonmain-default-branch` — attach lineage-seed hardcodes `main`, no-ops on a `master`-default mirror

**Discovered during:** items 4a/4b code review (2026-07-13)

**Size:** S

**Severity:** LOW — no data loss. A mirror whose default branch is `master` (or anything but
`main`) simply gets NO lineage seed at attach; the first fetch parentless-seeds exactly as an
un-anchored tree does, so the Pattern-C `git pull --rebase` heal silently no-ops instead of
reconciling. Not a regression — `main` is hardcoded pervasively across the codebase.

**Source:** `crates/reposix-cli/src/attach.rs:476` (`seed_tracking_ref`) resolves the seed value
from the literal `refs/remotes/{mirror_name}/main` only; `resolve_import_parent`
(`reposix-remote/src/main.rs`) and init's fetch refspec likewise target `.../main`. On a
`master`-default mirror `refs/remotes/<mirror>/main` does not exist, so `seed_tracking_ref` hits
its no-mirror-ref skip branch and item 4a's fix never engages.

**Acceptance:** attach's seed (and, ideally, the shared tracking-ref path) resolves the mirror's
actual default branch via `git symbolic-ref refs/remotes/<mirror>/HEAD` (falling back to `main`)
instead of hardcoding `main`, so a `master`-default round-tripper gets the same lineage anchor. A
round-trip regression on a `master`-default mirror proves it.

**Default disposition for P111:** S closes-or-defers; carry-forward target v0.15.0 (the broader
`main`-hardcoding sweep is bigger than the attach seed alone — scope the attach-local fix first).

**STATUS:** OPEN

## Entry format

```markdown
## GOOD-TO-HAVES-NN — `<short-id>` — one-line title

**Discovered during:** P<N>

**Size:** XS|S|M (rough effort estimate)

**Source:** where this was noticed.

**Acceptance:** what "done" looks like.

**Default disposition for P111:** XS always closes; S closes-or-defers; M default-defers
to the next milestone with a named carry-forward target.

**STATUS:** OPEN
```
