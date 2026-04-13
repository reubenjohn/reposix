# reposix — end-to-end demo

One-liner: reposix mounts a REST-based issue tracker as a POSIX directory
tree and translates `git push` into HTTP PATCH/POST/DELETE. This walkthrough
runs the full flow against the in-process simulator. An LLM agent can
`ls`, `cat`, `grep`, edit, and `git push` issues without ever seeing a JSON
schema or REST endpoint.

## Reproduce in 5 minutes

Prereqs (Linux only for v0.1):

- Rust stable 1.82+ (we tested with 1.94.1).
- `fusermount3` (Ubuntu: `sudo apt install fuse3`).
- `jq`, `sqlite3`, `curl`, `git` (>= 2.20) on `$PATH`.

Then:

```bash
git clone https://github.com/reubenjohn/reposix
cd reposix
bash scripts/demo.sh
```

Expect ~30 seconds total after the first release build completes (the
script builds release binaries up front so the recording is not dominated
by `cargo` compile noise).

The same script is what the recording in `docs/demo.typescript` captures
verbatim — there is no hand-edited demo flow.

## Walkthrough

Each of the 9 steps below corresponds to a banner in `scripts/demo.sh`.
Commands are paste-ready against a fresh clone with the prereqs above.

### 1/9 — What we have

```bash
cargo --version
ls crates/
```

Five crates: `reposix-cli`, `reposix-core`, `reposix-fuse`, `reposix-remote`,
`reposix-sim`.

### 2/9 — Test suite

```bash
cargo test --workspace --no-fail-fast
```

133 tests passed across 20 binaries on the recorded run; 3 ignored
(`#[ignore]`-gated FUSE-mount + sim-watchdog scenarios that the CI
integration job runs under `--ignored`).

### 3/9 — Start the simulator

```bash
target/release/reposix-sim \
    --bind 127.0.0.1:7878 \
    --db /tmp/demo-sim.db \
    --seed-file crates/reposix-sim/fixtures/seed.json &
curl -sf http://127.0.0.1:7878/healthz   # waits for "ok"
curl -s http://127.0.0.1:7878/projects/demo/issues | jq 'length'
# => 6
```

Six seeded issues. The seed includes adversarial fixtures: issue 1's body
has a literal `<script>` tag and issue 3's body contains a `version: 999`
line designed to escape into frontmatter parsers (it doesn't — see step 6).

### 4/9 — Mount FUSE

```bash
mkdir /tmp/demo-mnt
target/release/reposix-fuse /tmp/demo-mnt \
    --backend http://127.0.0.1:7878 --project demo &
ls /tmp/demo-mnt | sort
# 0001.md  0002.md  0003.md  0004.md  0005.md  0006.md
```

The kernel sees a new VFS at `/tmp/demo-mnt`. Filenames are `<id>.md`
zero-padded to 4 digits per SG-04. The FUSE daemon enforces this on every
path-bearing op — `ls`, `cat`, `read`, `write`, `create`, `unlink`.

### 5/9 — Browse with shell tools

```bash
cat /tmp/demo-mnt/0001.md
grep -ril database /tmp/demo-mnt
```

Each file is YAML-frontmatter Markdown:

```
---
id: 1
title: database connection drops under load
status: open
labels:
- bug
- p1
created_at: 2026-04-13T00:00:00Z
updated_at: 2026-04-13T00:00:00Z
version: 1
---

The <script>alert(1)</script> test harness drops connections after ~500
concurrent requests.
...
```

`grep -ril database /tmp/demo-mnt` returns `/tmp/demo-mnt/0001.md`.
Agent-style read paths work end-to-end.

### 6/9 — Edit through FUSE

```bash
NEW="$(sed 's/^status: open$/status: in_progress/' /tmp/demo-mnt/0001.md)"
printf '%s\n' "$NEW" > /tmp/demo-mnt/0001.md
curl -s http://127.0.0.1:7878/projects/demo/issues/1 \
    | jq '{id, status, version, body_len: (.body | length)}'
# {"id": 1, "status": "in_progress", "version": 2, "body_len": 563}
```

Note we do NOT use `sed -i`: the FUSE FS only accepts filenames matching
`<id>.md`, and `sed -i` creates a temp file like `sed.XYZ`, which gets
`EINVAL`. We instead read, transform in memory, and write back via a
single `open(O_TRUNC) + write`.

The server's `version` bumped from 1 → 2. Crucially, the `version: 999`
line in issue 3's body did **not** propagate to the server's authoritative
version field — every outbound write goes through `Tainted<T> → sanitize()`
which strips `id`, `version`, `created_at`, `updated_at` from the
client-supplied frontmatter (SG-03).

### 7/9 — git push round-trip

```bash
mkdir /tmp/demo-repo && cd /tmp/demo-repo
git init -q
git symbolic-ref HEAD refs/heads/main   # `git init -b main` needs git ≥ 2.28
git config user.email demo@reposix.local
git config user.name reposix-demo
git remote add origin reposix::http://127.0.0.1:7878/projects/demo

# Bootstrap: helper imports the snapshot as refs/reposix/origin/main.
git fetch origin || true   # spurious "fatal:" exit 128 — actual fetch succeeded
git checkout -B main refs/reposix/origin/main
ls   # 0001.md ... 0006.md

sed -i 's/^status: in_progress$/status: in_review/' 0001.md
git commit -am 'request review' -q
git push origin main
# To reposix::http://127.0.0.1:7878/projects/demo
#  * [new branch]      main -> main

curl -s http://127.0.0.1:7878/projects/demo/issues/1 | jq -r '.status'
# in_review
```

The `git-remote-reposix` helper translates the diff into a `PATCH
/projects/demo/issues/1` request. Conflicts (If-Match on stale version)
would surface as native git rejections; v0.1's helper retries internally
once.

### 8/9 — Guardrails on camera

This is the section that matters. Three guardrails fire visibly:

#### 8a — Outbound HTTP allowlist (SG-01)

```bash
mkdir /tmp/demo-allow-mnt
REPOSIX_ALLOWED_ORIGINS=http://127.0.0.1:9999 \
    target/release/reposix-fuse /tmp/demo-allow-mnt \
        --backend http://127.0.0.1:7878 --project demo &
ls /tmp/demo-allow-mnt
# ls: reading directory '/tmp/demo-allow-mnt': Permission denied
# stderr: WARN reposix_fuse::fs: readdir fetch failed
#         error=origin not allowlisted: http://127.0.0.1:7878/projects/demo/issues
```

Setting `REPOSIX_ALLOWED_ORIGINS` to a port that doesn't match the
configured backend causes every fetch to refuse at the
`reposix_core::http::client()` factory — the only legal HTTP-client
constructor in the workspace, enforced by a clippy `disallowed-methods`
lint on `reqwest::Client::new`.

The demo runs this in a *second* mount (`/tmp/demo-allow-mnt`) so the
primary mount stays alive for the rest of the demo.

#### 8b — Bulk-delete cap (SG-02)

```bash
cd /tmp/demo-repo
git rm -q 0001.md 0002.md 0003.md 0004.md 0005.md 0006.md
git commit -am cleanup -q
git push origin main
# error: refusing to push (would delete 6 issues; cap is 5;
#        commit message tag '[allow-bulk-delete]' overrides)
# ! [remote rejected] main -> main (bulk-delete)

git commit --amend -q -m '[allow-bulk-delete] cleanup'
git push origin main
# To reposix::http://127.0.0.1:7878/projects/demo
#    b118598..311e5a4  main -> main

curl -s http://127.0.0.1:7878/projects/demo/issues | jq 'length'
# 0
```

Defends against a stray `rm -rf` on the mount point cascading into a
DELETE storm. The override tag is required to be in the commit message
(intent has to be local + reviewable, not transient flag).

#### 8c — Audit log truth (SG-06)

```bash
sqlite3 -header -column /tmp/demo-sim.db \
  'SELECT method, path, status FROM audit_events ORDER BY id DESC LIMIT 5;'
# method  path                   status
# GET     /projects/demo/issues  200
# DELETE  /projects/demo/issues  204
# DELETE  /projects/demo/issues  204
# DELETE  /projects/demo/issues  204
# DELETE  /projects/demo/issues  204
```

Every network-touching action is in SQLite, append-only (a `BEFORE
UPDATE` and a `BEFORE DELETE` trigger on the `audit_events` table raise
on any mutation attempt, asserted by a Phase-1 test against
`pragma table_info`).

### 9/9 — Cleanup

`fusermount3 -u` + `pkill` + `rm -rf /tmp/demo-*`. Trap-driven; runs on
any exit path of the script (success, failure, Ctrl-C).

## What the recording shows

The file `docs/demo.typescript` is the raw `script(1)` recording (3.6 KB,
102 lines). `docs/demo.transcript.txt` is the same file with ANSI escape
sequences stripped, suitable for GitHub rendering inside a PR or issue
comment. Both were produced from a single `bash scripts/demo.sh`
invocation under `script(1)` — the recording is not hand-edited.

Three lines in the recording are the "guardrails on camera" proof:

- `SG-02 fired as expected` (bulk-delete cap)
- `WARN reposix_fuse::fs: readdir fetch failed error=origin not allowlisted: ...`
  (allowlist refusal)
- The step-6 `printf > 0001.md` + `curl ... | jq` pair, which proves the
  server's authoritative `version: 2` survives a client write whose body
  contained `version: 999` — the `Tainted<T> → sanitize()` egress filter
  strips server-controlled fields before PATCH.

## Limitations / honest scope

This is v0.1 alpha, built autonomously overnight on 2026-04-13. What's
**not** in the demo:

- **No real backend.** Simulator-only. Real Jira/GitHub/Confluence
  integration is v0.2. See [PROJECT.md → Out of Scope](../.planning/PROJECT.md).
- **No man page, .deb, or brew formula.** Clone-and-`cargo build`.
- **Linux only.** FUSE3/FUSE2. macOS-via-macFUSE is a follow-up.
- **Threat model is taken seriously but not exhaustively mitigated.**
  See [`threat-model-and-critique.md`](../.planning/research/threat-model-and-critique.md)
  — the SG-01/02/03 cuts demonstrated here close the most lethal-trifecta
  paths but do not cover every M-* finding in the red-team report. Those
  are deferred to v0.2.
- **`git fetch` exit 128 is a v0.1 helper compatibility wart.** The
  helper exposes refs as `refs/reposix/origin/main`; git's post-fetch
  step tries to update a non-existent `refs/remotes/origin/main` and
  emits a misleading "fatal:" message. The actual fetch succeeds. v0.2
  will normalise the helper's `list` output so git's tracking-branch
  logic is happy.
