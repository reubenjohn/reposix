← [back to index](./index.md) · phase 83 research

## Common Pitfalls

### Pitfall 1: `synced_at` write order vs `head` write order

**What goes wrong:** If `head` is written AFTER `synced_at`, an observer reading mid-write sees `synced_at > head` for a moment, which violates the invariant "synced_at is the timestamp the head ref was last brought current."

**Why it happens:** Lazy programming. Both writes are independent ref edits; ordering looks irrelevant.

**How to avoid:** Always write `head` FIRST, then `synced_at`. On the partial-fail path, `head` is written but `synced_at` is intentionally skipped — preserving the invariant `synced_at <= head_ts_implicit`.

**Warning signs:** A fault-injection test where `synced_at` parses to a time later than the cache's most recent commit timestamp.

### Pitfall 2: Mirror push uses `--force-with-lease`

**What goes wrong:** P84's webhook workflow uses `--force-with-lease` because it races with bus pushes. If P83's bus-push helper *also* uses `--force-with-lease`, two concurrent bus pushes can each pass the lease check (different leases) and one will silently overwrite the other's commit.

**Why it happens:** Cargo-cult from P84.

**How to avoid:** Plain `git push <mirror> main` — no `--force-with-lease`, no `--force`. SoT-first means by the time we get here, our SoT write IS the new authoritative state. Whatever's on the mirror is either (a) what we just left there last push, or (b) drift that PRECHECK A already trapped at the top of `handle_bus_export`. If a concurrent webhook-sync raced in between PRECHECK A and our mirror push, our push will fail with a non-fast-forward error → that's the partial-fail path, audit it, return ok, recover on next push.

**Warning signs:** Any code change that introduces `--force` or `--force-with-lease` flags into `push_mirror`. Reject in code review.

### Pitfall 3: Confluence partial-write semantics (PATCH 1 succeeds, PATCH 2 fails)

**What goes wrong:** If `apply_writes_bus` is partway through a 5-action plan (`Update id=1, Update id=2, Create id=99, Delete id=3`) and the PATCH for id=2 returns 500, what's the SoT state? Answer: id=1 IS updated; id=2 is NOT; id=99 was never attempted; id=3 was never attempted. The cache has NO record of id=1's new version (because the per-action loop bails on first error per `handle_export:504-512`).

**Why it happens:** The current `handle_export` per-action loop is best-effort-stop-on-first-error. There's no transaction boundary spanning the actions; each REST call is a discrete write to the SoT.

**How to avoid:**
1. **Document explicitly** in P83's plan that the bus path inherits this semantic (no atomicity across actions). The architecture-sketch is silent; this research closes that ambiguity: **non-atomic, fail-stop, partial state is observable on SoT after partial failure.**
2. **The recovery story:** the next push from any pusher reads the new SoT state via PRECHECK B's `list_changed_since`, sees id=1 has a newer version, and either accepts the local change (if the prior version still matches) or rejects with conflict (if not). The audit row records which actions executed (the `summary` field is `1,2,99,3`-ordered; on partial fail, `summary` reflects only the actions that completed — modify the existing summary-build to occur AFTER each successful action, not from the planned set).
3. **Test:** the fault-injection (b) scenario (kill confluence-write mid-stream) asserts the exact partial state — id=1 has new version, id=2 unchanged, no audit row for ids 99 / 3. Mirror unchanged.

**Warning signs:** A test that asserts "all-or-nothing" behavior. Reject — that's not what the helper does.

### Pitfall 4: Audit-row ordering on the partial-fail path

**What goes wrong:** Defensive form is "write audit row BEFORE the operation that might fail." On the SoT-succeed-mirror-fail path, this means writing `helper_push_partial_fail_mirror_lag` BEFORE attempting `git push <mirror>`. But if mirror push *succeeds*, we'd have a lying audit row (says "partial fail" when in fact everything worked).

**Why it happens:** Reasonable defensiveness applied at the wrong layer.

**How to avoid:** Write the audit row AFTER the mirror-push outcome is known. The atomicity property we want is "the audit row reflects the actual end-state," not "the audit row predicts the end-state." On a crash between mirror-push-result and audit-row-write, we lose ONE audit row but the ref state is internally consistent (head reflects SoT, synced_at reflects last-successful-mirror-sync). Acceptable trade — same shape as `handle_export`'s mirror-ref writes, where ref-write failures WARN-log without poisoning the push ack (per `mirror_refs.rs` module doc: "Ref writes are best-effort").

**Warning signs:** An audit row written speculatively before the operation completes. Reject in code review.

### Pitfall 5: First-push case (no `last_fetched_at` cursor, no prior mirror refs)

**What goes wrong:** PRECHECK B's no-cursor path returns Stable (per `precheck_sot_drift_any:359` — first-push policy). The bus handler proceeds to write fan-out. Mirror is empty. `git push <mirror> main` succeeds — but `refs/mirrors/<sot>-head` write needs a SoT SHA, and `refresh_for_mirror_head` fires `cache.build_from()` which itself does a full `list_records` walk on first run... wait, we're on the L1 path now (P81). `precheck_export_against_changed_set` returned Stable, so `prior` was synthesized from the cache's existing tree state. The cache's tree state is empty (first push). `plan(empty_prior, parsed) → Vec<Create>` for every record in the push.

**Why it's tricky:** The existing `handle_export` first-push path already works (P80 integration test `mirror_refs::write_on_success_updates_both_refs` exercises it). Bus path inherits.

**How to avoid:** Add a first-push integration test in P83a (happy-path) that asserts: empty cache + empty mirror + N records in push → SoT receives N create_record calls; mirror receives a fresh `main` ref; both `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at` are populated; `audit_events_cache` has `helper_push_started + helper_push_accepted + mirror_sync_written` rows.

**Warning signs:** A test that hard-codes a `last_fetched_at` value in a fixture without exercising the no-cursor path.

### Pitfall 6: `bus_handler` cwd assumption

**What goes wrong:** Section 2 above asserts the bus_handler runs in the working tree's cwd (because git invokes the helper from the working tree). If a future refactor moves bus_handler invocation to a separate thread or async task with `tokio::spawn`, the cwd may be lost or the env may be inherited differently.

**How to avoid:** **Pin the cwd assumption with a test.** Add a test that asserts `std::env::current_dir()?` inside `handle_bus_export` resolves to the working tree (not the cache dir, not `/tmp`). And/or: capture cwd in `state` at helper-startup time and pass it explicitly to `push_mirror` if the architecture ever needs to.

**Warning signs:** Any future refactor that moves git subprocess calls into async closures. Add a doc comment to `bus_handler.rs` warning about this.

### Pitfall 7: `cache_schema.sql` `op` CHECK list on stale cache.db files

**What goes wrong:** Per the existing comment in `cache_schema.sql:11-27`: *"On stale cache.db files the new ops will fail the CHECK and fall through the audit best-effort path (warn-logged); fresh caches see the full list."*

**How to avoid:** P83's new `helper_push_partial_fail_mirror_lag` op gets added to the CHECK list. Existing caches will reject the row at INSERT time, but the audit helper is best-effort (returns `()`, WARN-logs on error) — so the push still succeeds, the warning is the diagnostic. Fresh caches accept the row immediately. **This is the established pattern (P79 added `attach_walk`, P80 added `mirror_sync_written` — both via CHECK list extension).** No migration needed.

**Warning signs:** A migration script trying to ALTER TABLE the CHECK constraint. Don't — the IF NOT EXISTS pattern is the contract.

## Mirror-Write Algorithm (exact state transitions)

```
Pre-state (after PRECHECK A + PRECHECK B pass; stdin still buffered):
  refs/mirrors/<sot>-head:        OLD_SHA       (or absent if first push)
  refs/mirrors/<sot>-synced-at:   OLD_TS_TAG    (or absent if first push)
  cache.last_fetched_at:          CURSOR_T0     (or absent)
  audit_events_cache rows:        … (existing) …
  audit_events rows:              … (existing) …
  SoT records:                    {id_1: v1, id_2: v2, …}
  mirror main:                    OLD_SHA

Action 1: parse stdin → ParsedExport (BufReader on ProtoReader)
Action 2: apply_writes_bus(...) executes full SoT write loop
   - L1 precheck: returns Proceed { prior } (no conflicts) or Conflicts
   - plan(prior, parsed): returns Vec<Create|Update|Delete>
   - for each PlannedAction: execute_action() does ONE REST call
       - on success: backend adapter writes ONE audit_events row (OP-3)
       - on failure: returns Err; loop continues but any_failure = true
   - on all-success:
       cache.log_helper_push_accepted(files_touched, summary)   ← audit_events_cache
       cache.write_last_fetched_at(now)                         ← cursor advances to T1
       sot_sha = cache.refresh_for_mirror_head().await         ← NEW_SHA
       returns WriteOutcome::SotOk { sot_sha = Some(NEW_SHA), … }
   - on any failure: returns SotPartialFail / Conflict / etc.

Branch on WriteOutcome:
  - SotOk → continue to action 3
  - else → emit reject line; mirror UNCHANGED; return

Action 3: cache.write_mirror_head(<sot>, NEW_SHA)              ← head ref MOVES to NEW_SHA
Action 4: push_mirror(mirror_remote_name)
   subprocess: git push <mirror_remote_name> main
   - on success: return MirrorResult::Ok
   - on failure: return MirrorResult::Failed { exit, stderr_tail }

Branch on MirrorResult:

  MirrorResult::Ok →
     post-state:
       refs/mirrors/<sot>-head:        NEW_SHA               ✓ updated
       refs/mirrors/<sot>-synced-at:   NOW_TS_TAG            ✓ updated (action 5a)
       cache.last_fetched_at:          T1                    ✓ updated
       audit_events_cache rows:        + helper_push_started
                                       + helper_push_accepted
                                       + mirror_sync_written  (action 5c)
       audit_events rows:              + per-record mutations (action 2 inner)
       mirror main:                    NEW_SHA               ✓ updated
     emit: ok refs/heads/main

  MirrorResult::Failed →
     post-state:
       refs/mirrors/<sot>-head:        NEW_SHA               ✓ updated
       refs/mirrors/<sot>-synced-at:   OLD_TS_TAG            ✗ FROZEN (action 5b)
       cache.last_fetched_at:          T1                    ✓ updated
       audit_events_cache rows:        + helper_push_started
                                       + helper_push_accepted
                                       + helper_push_partial_fail_mirror_lag  (action 5d, NEW op)
       audit_events rows:              + per-record mutations (action 2 inner)
       mirror main:                    OLD_SHA               ✗ unchanged (push failed)
     emit warning to stderr: "SoT push succeeded; mirror push failed (Reason: …)"
     emit: ok refs/heads/main         ← Q3.6 contract: SoT promise satisfied
```

**Lag observability:** the difference `head ≠ synced-at-target` is precisely the lag. A vanilla-git `git fetch origin` brings both refs into a Dev-B clone; `git log refs/mirrors/<sot>-synced-at -1` shows the timestamp; `git rev-parse refs/mirrors/<sot>-head` shows the SHA the SoT advanced to. If they disagree, lag is real.

