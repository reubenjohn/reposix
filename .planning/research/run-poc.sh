#!/bin/sh
# Runs the partial-clone POC inside an alpine container with git 2.52.
# Mounted at /work; the helper is at /work/git-remote-poc.
# NOTE: set -e is intentionally OFF — `grep -c '^?'` legitimately returns
# 1 when zero objects are missing, and that's the success state.
cd /work

echo "=== git version ==="
git --version

echo
echo "=== Step 1: build a fresh bare repo to lazy-clone from ==="
rm -rf src bare.git
mkdir src
cd src
git init -q -b main .
echo small > small.txt
head -c 200000 /dev/urandom | base64 > big1.txt
head -c 200000 /dev/urandom | base64 > big2.txt
head -c 200000 /dev/urandom | base64 > big3.txt
mkdir docs
echo doc-one > docs/d1.md
echo doc-two > docs/d2.md
git add -A
git -c user.email=t@t -c user.name=t commit -q -m init
HEAD_OID=$(git rev-parse HEAD)
echo "HEAD = $HEAD_OID"
cd ..
git clone --bare src bare.git -q
# Enable filter advertisement on the server side. Without this, upload-pack
# does NOT advertise the `filter` capability and the client silently drops
# --filter=blob:none.
git -C bare.git config uploadpack.allowFilter true
git -C bare.git config uploadpack.allowAnySHA1InWant true

echo
echo "=== Step 2: clone via the remote helper, with --filter=blob:none --no-checkout ==="
# --no-checkout keeps blobs missing in the ODB so we can prove lazy-fetch.
# (A normal clone+checkout would fetch them all immediately during checkout.)
export PATH="/work:$PATH"
export REPOSIX_POC_LOG=/work/helper-all.log
rm -f /work/helper-all.log
rm -rf client
GIT_TRACE_PACKET=/work/clone-packets.log \
  git -c protocol.version=2 \
  clone --filter=blob:none --no-checkout "poc::/work/bare.git" client 2> /work/clone.log
echo "clone RC=$?"

echo
echo "=== Step 3: check partial-clone state ==="
echo "promisor config:"
git -C client config --get remote.origin.promisor
git -C client config --get remote.origin.partialclonefilter
echo
MISSING_BEFORE=$(cd client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "blobs MISSING right after partial clone: $MISSING_BEFORE"
echo
PACK_BYTES=$(du -b client/.git/objects/pack/*.pack 2>/dev/null | awk '{s+=$1} END {print s}')
echo "total .pack bytes after clone: $PACK_BYTES (compare to original bare.git which has 600KB+ of blobs)"
echo "(small pack = filter worked: blobs not transferred)"

echo
echo "=== Step 4: trigger lazy fetch by reading ONE missing blob ==="
# big1.txt blob's OID is in the tree (which we DO have). cat-file -p invokes
# the promisor remote to fetch the blob on demand.
BLOB1=$(git -C client rev-parse HEAD:big1.txt)
echo "blob1 OID: $BLOB1"
echo "before fetch — is it missing?"
git -C client cat-file -e "$BLOB1" 2>&1 || echo "  (cat-file -e said: missing)"
echo "fetching via cat-file -p (this should invoke the helper):"
git -C client cat-file -p "$BLOB1" | head -c 60
echo
echo
MISSING_AFTER=$(cd client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "missing blob count after lazy-fetch: $MISSING_AFTER (was $MISSING_BEFORE)"

echo
echo "=== Step 5: try reading a missing blob via show ==="
BLOB2=$(git -C client rev-parse HEAD:docs/d1.md)
echo "docs/d1.md blob: $BLOB2"
git -C client show "$BLOB2"
MISSING_AFTER2=$(cd client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "missing blob count after second lazy-fetch: $MISSING_AFTER2"

echo
echo "=== Step 5b: sparse-checkout interaction (Q5 evidence) ==="
# Fresh partial clone, then sparse-checkout to docs/ ONLY. Then `git
# checkout main`. Hypothesis: ONLY docs/ blobs get fetched, not big*.
rm -rf sparse-client
git -c protocol.version=2 clone --filter=blob:none --no-checkout \
  "poc::/work/bare.git" sparse-client 2>/dev/null
git -C sparse-client sparse-checkout init --cone
git -C sparse-client sparse-checkout set docs
echo "AFTER sparse-checkout set docs, before checkout: missing blobs ="
( cd sparse-client && git rev-list --objects --missing=print HEAD | grep -c '^?' )
git -C sparse-client checkout main
echo "AFTER checkout main with sparse=docs: missing blobs ="
MISSING_SPARSE=$(cd sparse-client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "$MISSING_SPARSE"
echo "files in working tree:"
ls sparse-client
echo "(big*.txt should be ABSENT if sparse worked; only docs/ should be filled)"

echo
echo "=== Step 5c: agent overshare scenario — git grep across all paths ==="
# Make a fresh partial clone, then `git grep` something. Does it trigger
# lazy-fetch of EVERY blob? Per Q4 of the research, this is a concern: we
# want the helper to be able to see and potentially refuse such broad
# requests.
rm -rf grep-client
git -c protocol.version=2 clone --filter=blob:none --no-checkout \
  "poc::/work/bare.git" grep-client 2>/dev/null
echo "BEFORE grep: missing blobs ="
GREP_BEFORE=$(cd grep-client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "$GREP_BEFORE"
echo "running 'git grep doc-one HEAD' (would touch every blob in HEAD tree):"
( cd grep-client && timeout 10 git grep doc-one HEAD 2>&1 ) | head -10 || true
echo "AFTER grep: missing blobs ="
GREP_AFTER=$(cd grep-client && git rev-list --objects --missing=print HEAD | grep -c '^?')
echo "$GREP_AFTER"
echo "(if grep fetched everything, AFTER count should be much lower)"

echo
echo "=== Step 5d: Q4 evidence — show single fetch request can carry many wants ==="
echo "Helper saw these many distinct 'want <oid>' lines per request:"
grep -oE "want [0-9a-f]{40}" /work/helper-all.log | wc -l
echo "(if >1, batched fetches happen — helper has full visibility for refusal logic)"

echo
echo "=== Step 6: helper invocations (count distinct PIDs) ==="
echo "distinct helper PIDs that handled requests:"
grep -oE 'helper pid=[0-9]+' /work/helper-all.log | sort -u
echo
echo "=== helper trace from /work/helper-all.log (full) ==="
cat /work/helper-all.log
