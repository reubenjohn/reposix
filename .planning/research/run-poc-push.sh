#!/bin/sh
# Hybrid POC: verify git uses `stateless-connect` for fetch and `export`
# for push when BOTH capabilities are advertised.
#
# Runs in alpine:latest (git 2.52 needed for protocol-v2 filter support).
#
# Test flow:
#   1. Build a bare repo with one commit, enable filter advertisement.
#   2. Partial clone via reposix:: → expect stateless-connect trace.
#   3. Create a new commit locally.
#   4. `git push origin main` → expect export trace, bare repo gets commit.
#   5. Create ANOTHER commit, force helper into reject mode via env var,
#      push → expect `error refs/heads/main <message>` surfaced by git.
cd /work

echo "=== git version ==="
git --version

echo
echo "=== Step 1: build a fresh bare repo ==="
rm -rf src bare.git client helper-push.log
mkdir src
cd src
git init -q -b main .
echo "first commit" > a.txt
mkdir docs
echo "doc v1" > docs/d1.md
git add -A
git -c user.email=t@t -c user.name=t commit -q -m "commit 1"
cd ..
git clone --bare src bare.git -q
git -C bare.git config uploadpack.allowFilter true
git -C bare.git config uploadpack.allowAnySHA1InWant true
echo "bare.git head: $(git -C bare.git rev-parse HEAD)"

echo
echo "=== Step 2: partial clone via reposix:: helper (expect stateless-connect in trace) ==="
export PATH="/work:$PATH"
export REPOSIX_POC_LOG=/work/helper-push.log
rm -f "$REPOSIX_POC_LOG"
git -c protocol.version=2 \
  clone --filter=blob:none "poc::/work/bare.git" client 2>/work/clone.log
echo "clone RC=$?"
cd client
git -c user.email=a@a -c user.name=a config --local user.email a@a
git -c user.email=a@a -c user.name=a config --local user.name a
echo "HEAD after clone: $(git rev-parse HEAD)"
echo "remote URL: $(git config --get remote.origin.url)"

echo
echo "=== Step 3: create a new commit locally ==="
echo "second-commit-body" > b.txt
echo "doc v2" > docs/d1.md
git add -A
git -c user.email=a@a -c user.name=a commit -q -m "commit 2 from client"
echo "client HEAD: $(git rev-parse HEAD)"
cd ..

echo
echo "=== Step 4: push via reposix:: helper (expect export path in trace) ==="
GIT_TRACE_PACKET=/work/push-packets.log \
  git -C client -c protocol.version=2 push origin main 2>/work/push.log
echo "push RC=$?"
echo
echo "---- client-facing push output (push.log) ----"
cat /work/push.log
echo
echo "---- bare.git HEAD after push ----"
git -C bare.git rev-parse HEAD
echo "bare.git log --oneline:"
git -C bare.git log --oneline

echo
echo "=== Step 5: demonstrate rejection — create another commit, force helper into reject mode ==="
cd client
echo "doc v3" > docs/d1.md
git add -A
git -c user.email=a@a -c user.name=a commit -q -m "commit 3 that helper should reject"
cd ..

export REPOSIX_POC_REJECT_PUSH="reposix: issue was modified on backend since last fetch"
echo "pushing with REPOSIX_POC_REJECT_PUSH set..."
git -C client push origin main 2>/work/push-reject.log
echo "push RC=$? (expect non-zero)"
unset REPOSIX_POC_REJECT_PUSH
echo
echo "---- push rejection output surfaced to user ----"
cat /work/push-reject.log
echo
echo "bare.git HEAD after rejected push (should be unchanged from step 4):"
git -C bare.git rev-parse HEAD

echo
echo "=== Step 6: capability-dispatch evidence — summarise trace ==="
echo
echo "Distinct helper invocations (pid + first verb):"
awk '
  /invoked:/ { pid=$0 }
  /<- '\''capabilities'\''/ { next }
  /<- '\''stateless-connect / { print pid " -> stateless-connect (fetch)"; pid="" }
  /<- '\''list for-push'\''/ { print pid " -> list for-push (push setup)"; pid="" }
  /<- '\''list'\''/ { print pid " -> list (fetch setup)"; pid="" }
  /<- '\''export'\''/ { print pid " -> export (push payload)"; pid="" }
' /work/helper-push.log | sort -u

echo
echo "Capability usage counts:"
echo -n "  stateless-connect invocations: "
grep -c "<- 'stateless-connect " /work/helper-push.log
echo -n "  export invocations: "
grep -c "<- 'export'" /work/helper-push.log
echo -n "  list for-push invocations: "
grep -c "<- 'list for-push'" /work/helper-push.log
echo -n "  rejected pushes: "
grep -c "REJECTING" /work/helper-push.log
echo -n "  accepted pushes: "
grep -c "accepting — spawning git fast-import" /work/helper-push.log

echo
echo "=== Step 7: RAW helper trace ==="
cat /work/helper-push.log
