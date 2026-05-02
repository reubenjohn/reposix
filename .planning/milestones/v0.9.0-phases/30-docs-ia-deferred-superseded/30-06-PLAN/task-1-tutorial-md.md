← [back to index](./index.md)

<task type="auto">
  <name>Task 1: Write docs/tutorial.md — 4 steps with aha in step 4</name>
  <files>docs/tutorial.md</files>
  <read_first>
    - `docs/tutorial.md` (current skeleton from plan 30-02 — preserve structure; fill each Step)
    - `docs/demo.md` (source — lines 58-248 cover steps 3, 4, 6, 7 to be distilled)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-RESEARCH.md` §Tutorial Pattern (lines 765-780 — 4-step structure), §"Competitor Narrative Scan → Pattern G (Stripe)" (aha placement)
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-PATTERNS.md` §docs/tutorial.md (demo.md lines 58-82 step pattern + FUSE-specific-write-gotcha)
    - `.planning/notes/phase-30-narrative-vignettes.md` §"Hero vignette" (lines 109-181 — voice to match)
  </read_first>
  <action>
    Replace the ENTIRE contents of `docs/tutorial.md` with the following markdown:

```markdown
# Try it in 5 minutes

In 5 minutes you will edit a simulated tracker ticket with `sed`, `git push` the change, and `curl` the simulator to watch the version bump from 1 to 2. The whole tutorial runs offline, with zero credentials — the "tracker" is `reposix-sim`, a full-fidelity fake that ships in this repo.

## Prerequisites

Linux (fuse3 required):

- Rust stable 1.82+ (we tested with 1.94.1).
- `fusermount3` (Ubuntu: `sudo apt install fuse3`).
- `jq`, `sqlite3`, `curl`, `git` (>= 2.20) on `$PATH`.

If this is your first build:

\`\`\`bash
git clone https://github.com/reubenjohn/reposix
cd reposix
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"
\`\`\`

## Step 1 — Start the simulator

The simulator is an in-process axum server that speaks the same HTTP shape as a real tracker. It seeds itself with 6 issues in a project called `demo`.

\`\`\`bash
reposix-sim \
    --bind 127.0.0.1:7878 \
    --db /tmp/tutorial-sim.db \
    --seed-file crates/reposix-sim/fixtures/seed.json &

curl -sf http://127.0.0.1:7878/healthz       # waits for "ok"
curl -s http://127.0.0.1:7878/projects/demo/issues | jq 'length'
# => 6
\`\`\`

Six seeded issues. No credentials. Leave the simulator running in the background — every subsequent step talks to it.

## Step 2 — Connect the tracker as a folder

`reposix mount` asks the kernel to expose the simulator at a path on disk. After this command, `/tmp/tutorial-mnt/issues/` is a real directory full of `.md` files.

\`\`\`bash
mkdir -p /tmp/tutorial-mnt
reposix mount /tmp/tutorial-mnt \
    --backend http://127.0.0.1:7878/projects/demo &

sleep 1   # give the mount a moment
ls /tmp/tutorial-mnt/issues/
# => 00000000001.md  00000000002.md  00000000003.md  ...
cat /tmp/tutorial-mnt/issues/00000000001.md
# shows YAML frontmatter + markdown body
\`\`\`

That is the whole abstraction. The folder is a git working tree and every file is a ticket.

## Step 3 — Edit a ticket

We change the status of issue 1 from `open` to `in_progress`. We do NOT use `sed -i` — the FUSE filesystem only accepts filenames matching `<padded-id>.md`, and `sed -i` writes a temp file like `sed.XYZ` which is rejected with `EINVAL`. Use `printf` or a pipe instead:

\`\`\`bash
NEW="$(sed 's/^status: open$/status: in_progress/' \
       /tmp/tutorial-mnt/issues/00000000001.md)"
printf '%s\n' "$NEW" > /tmp/tutorial-mnt/issues/00000000001.md

cat /tmp/tutorial-mnt/issues/00000000001.md | grep '^status:'
# => status: in_progress
\`\`\`

Your change is local. No HTTP call has left the machine yet.

## Step 4 — `git push` and watch the version bump

This is the moment. `git init` the mount, commit the change, `git push` to the simulator. Then `curl` the simulator to see the server-side version bumped from 1 to 2.

\`\`\`bash
cd /tmp/tutorial-mnt
git init -q
git remote add origin \
    reposix::http://127.0.0.1:7878/projects/demo
git add -A
git commit -q -m "start issue 1"
git push origin HEAD:main

# Version before push was 1 — seeded. Watch the server:
curl -s http://127.0.0.1:7878/projects/demo/issues/1 | jq '.version'
# => 2
\`\`\`

That last `jq` print is the whole story. Your local edit flowed through `git push` as a `PATCH` call, the simulator bumped the version to prove the state change landed, and `git log` on your end recorded the intent.

## What just happened

You edited a tracker ticket as a text file, committed with `git`, and pushed — and the server-side state confirms it. No REST endpoint learning, no SDK, no MCP schema. That is the core loop. The [how-it-works](how-it-works/index.md) section opens the mechanism; the [mental model](mental-model.md) page states the three ideas in 60 seconds.

## Cleanup

\`\`\`bash
cd ~
reposix umount /tmp/tutorial-mnt   # or: fusermount3 -u /tmp/tutorial-mnt
pkill -f 'reposix-sim.*7878'
rm -f /tmp/tutorial-sim.db
\`\`\`

!!! info "This tutorial is run end-to-end in CI"
    `scripts/test_phase_30_tutorial.sh` executes every command above against the simulator and asserts the version bump. If this tutorial drifts out of sync with the code, the test fails.
```

Run Vale to confirm no banned tokens leak outside code fences:

```bash
~/.local/bin/vale --config=.vale.ini docs/tutorial.md
```

Expected: exit 0. The tutorial mentions "FUSE" once (step 3, in a rationale for not using `sed -i`) — BUT that mention is in prose, so Vale will flag it. Rephrase: replace "the FUSE filesystem" with "the mounted folder" OR "the reposix folder" to remove the banned token. Alternatively, move the gotcha into a code comment inside the bash block, where Vale's IgnoredScopes exempts it. The simplest rewrite: put the gotcha as a comment inside the step-3 bash block and delete it from the prose. Choose whichever keeps the tutorial readable without triggering the linter.

After rewrite, rerun Vale until exit 0.
  </action>
  <verify>
    <automated>test -f docs/tutorial.md && grep -c '^## Step ' docs/tutorial.md | awk '{exit !($1 == 4)}' && grep -c '## Prerequisites' docs/tutorial.md | grep -q '^1$' && grep -c '## What just happened' docs/tutorial.md | grep -q '^1$' && grep -c '## Cleanup' docs/tutorial.md | grep -q '^1$' && grep -c 'jq .\.version.' docs/tutorial.md | awk '{exit !($1 >= 1)}' && ~/.local/bin/vale --config=.vale.ini docs/tutorial.md</automated>
  </verify>
  <acceptance_criteria>
    - `grep -c '^## Step ' docs/tutorial.md` returns exactly `4`.
    - `grep -c '^## Prerequisites' docs/tutorial.md` returns `1`.
    - `grep -c '^## What just happened' docs/tutorial.md` returns `1`.
    - `grep -c '^## Cleanup' docs/tutorial.md` returns `1`.
    - `grep -c 'jq .\.version.' docs/tutorial.md` returns `>= 1` (the aha moment is a `jq .version` print).
    - `grep -c 'printf .%s.n. "\$NEW"' docs/tutorial.md` returns `1` (printf-not-sed-i gotcha present).
    - `grep -c 'reposix::http' docs/tutorial.md` returns `>= 1` (git remote helper URL scheme).
    - `grep -cE '\breplace\w*\b' docs/tutorial.md` returns `0`.
    - `~/.local/bin/vale --config=.vale.ini docs/tutorial.md` exits 0 (all banned terms scoped to code fences).
    - `wc -l docs/tutorial.md` reports `>= 80`.
    - `mkdocs build --strict` exits 0.
  </acceptance_criteria>
  <done>
    Tutorial complete with 4 steps, aha in step 4, cleanup, and CI-tested-in-prod admonition. Vale clean.
  </done>
</task>
