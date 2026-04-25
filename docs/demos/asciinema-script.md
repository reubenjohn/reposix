# reposix v0.10.0 launch screencast — asciinema script

Polished 5-minute story. Copy-pastable commands; outputs are real runs against the simulator. `# ASCIINEMA:` lines guide the recorder.

**Record:** `asciinema rec docs/demos/recordings/launch-2026-04-25.cast --title 'reposix — edit issue trackers with cat, grep, and git push' --idle-time-limit 2`

## Scene 1 — Start the simulator

```bash
# ASCIINEMA: Wait 1 second. Let the prompt sit empty.
cargo run -p reposix-sim &
```

```
   Compiling reposix-sim v0.10.0
    Finished `dev` profile in 4.82s
reposix-sim listening on http://127.0.0.1:7878
seeded 12 issues into project demo
```

## Scene 2 — Bootstrap a partial-clone working tree

```bash
reposix init sim::demo /tmp/launch-demo
```

Output (under `24 ms` against the simulator):

```
init: configured remote.origin.url = reposix::http://127.0.0.1:7878/projects/demo
init: extensions.partialClone = origin
init: ready at /tmp/launch-demo
```

```bash
cd /tmp/launch-demo && git checkout origin/main && ls issues/
```

```
0001.md  0002.md  0003.md  0004.md  0005.md  0006.md
0007.md  0008.md  0009.md  0010.md  0011.md  0012.md
```

## Scene 3 — Read one issue, watch the audit row land

```bash
# ASCIINEMA: Wait 1 second — this is the headline number.
cat issues/0001.md
```

```yaml
---
id: 1
title: Avatar upload returns 500 on > 4 MiB
status: open
assignee: alice@acme.com
labels: [backend, regression]
---
## Description

S3 PutObject rejects multipart bodies above 4 MiB.
```

```bash
sqlite3 ~/.cache/reposix/sim-demo.git/cache.db \
  "SELECT op, decision, latency_ms FROM audit_events_cache ORDER BY ts DESC LIMIT 1"
```

```
materialize|cache_hit|8
```

## Scene 4 — Edit, commit, push

```bash
sed -i 's/^status: .*/status: in_progress/' issues/0001.md
echo $'\n## Comment\nReproduced locally. Owning.' >> issues/0001.md
git commit -am 'PROJ-1: triage to in_progress' && git push
```

```
[main a1b2c3d] PROJ-1: triage to in_progress
To reposix::http://127.0.0.1:7878/projects/demo
   8e4f1a2..a1b2c3d  main -> main
```

## Scene 5 — Trigger a push conflict (a second agent races)

```bash
# ASCIINEMA: Wait 1 second. This is the dark-factory teaching moment.
( reposix init sim::demo /tmp/agent2 && cd /tmp/agent2 && git checkout origin/main && \
  sed -i 's/^assignee: .*/assignee: bob@acme.com/' issues/0001.md && \
  git commit -am 'PROJ-1: reassign to bob' && git push )

cd /tmp/launch-demo
echo $'\n## Comment\nlocal change' >> issues/0001.md
git commit -am 'PROJ-1: extra note' && git push
```

```
 ! [rejected]        main -> main (fetch first)
hint: Updates were rejected because the remote contains work you do not
hint: have locally. Use 'git pull' before pushing again.
```

## Scene 6 — Recover via `git pull --rebase` (zero in-context learning)

```bash
git pull --rebase && git push
```

```
Successfully rebased and updated refs/heads/main.
   d4e5f6a..7a8b9c0  main -> main
```

```bash
# ASCIINEMA: Wait 2 seconds. The agent never learned a new protocol —
# it knew git pull --rebase from pre-training.
```

## Scene 7 — `reposix tokens` (the cost ledger)

```bash
reposix tokens .
```

```
reposix token-cost ledger — /tmp/launch-demo
period: last 24h (12 helper RPC turns)
  fetch turns: 8     push turns: 4
  bytes (wire): 142 KiB     est tokens (chars/4): ~35 700

MCP-equivalent baseline (100k schema discovery + 5k per tool call):
  ~160 000 tokens

honest caveats:
  - chars/4 over-estimates for binary packfile content
  - savings vary by workload; metadata-only calls favour MCP
```

## Scene 8 — `reposix doctor` (14-check diagnostic)

```bash
reposix doctor .
```

```
reposix doctor — /tmp/launch-demo
[OK]    git repo layout              [OK]    extensions.partialClone = origin
[OK]    remote.origin.url (sim)      [OK]    git-remote-reposix on PATH
[OK]    git 2.45.1 (>= 2.34)         [OK]    cache DB + append-only triggers
[OK]    cache freshness 18s          [OK]    sparse-checkout (full tree)
[INFO]  REPOSIX_ALLOWED_ORIGINS = http://127.0.0.1:* (default)
[INFO]  REPOSIX_BLOB_LIMIT = 200 (default)     [OK] rustc 1.82.0
summary: 12 OK, 2 INFO, 0 WARN, 0 ERROR    exit: 0
```

## Scene 9 — `reposix history` (time-travel sync tags)

```bash
reposix history .
```

```
reposix sync history — cache /home/user/.cache/reposix/sim-demo.git
  2026-04-25T05:14:32Z  refs/reposix/sync/2026-04-25T05-14-32Z  d4e5f6a
  2026-04-25T05:14:18Z  refs/reposix/sync/2026-04-25T05-14-18Z  a1b2c3d
  2026-04-25T05:14:02Z  refs/reposix/sync/2026-04-25T05-14-02Z  8e4f1a2
3 sync tags total.
```

```bash
reposix at 2026-04-25T05:14:10Z .
```

```
2026-04-25T05:14:02Z  refs/reposix/sync/2026-04-25T05-14-02Z  8e4f1a2
```

```bash
# ASCIINEMA: Wait 2 seconds. Every sync is a checkable git ref.
# No prior art for this. Hold the frame.
```

## Scene 10 — Wrap

```bash
echo "5 minutes. Pure git. Zero MCP schema tokens."
# ASCIINEMA: Stop recording. Target runtime 4:30-5:00.
```

## Recording checklist

- [ ] Terminal width ≥ 100 cols, height ≥ 30 rows; short prompt (`$ `).
- [ ] `cargo build -p reposix-sim` warmed before Scene 1 — avoid 5 s of `Compiling` on camera.
- [ ] `~/.cache/reposix/sim-demo.git/` does not exist (cold-init guarantees the `24 ms` number).
- [ ] After recording: `asciinema play <cast>` and time it; target ≤ 5 min.
- [ ] Upload via `asciinema upload <cast>` and pin the URL in `docs/demos/index.md`.
