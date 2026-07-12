#!/usr/bin/env bash
# Validate the FIX mechanism: an import stream that emits `from <current-tip>`
# produces a DESCENDANT commit -> git fast-import advances refs/reposix/origin/main
# as a fast-forward, so `git pull --rebase` would replay cleanly.
# Contrast with the current parentless stream which git refuses.
set -uo pipefail
export GIT_AUTHOR_NAME=R GIT_AUTHOR_EMAIL=r@t.test GIT_COMMITTER_NAME=R GIT_COMMITTER_EMAIL=r@t.test
D=$(mktemp -d /tmp/rbf-fix.XXXXXX); cd "$D"
git init -q repo && cd repo
# Seed: an initial "Sync from REST snapshot" tracking commit (parentless), like init.
printf -- '---\nid: 1\n---\nv1\n' > /tmp/rbf-fix-blob1
git fast-import --quiet <<EOF
blob
mark :1
data $(wc -c < /tmp/rbf-fix-blob1)
$(cat /tmp/rbf-fix-blob1)
commit refs/reposix/origin/main
mark :2
committer reposix-helper <bot@reposix> 0 +0000
data 24
Sync from REST snapshot
M 100644 :1 issues/1.md
done
EOF
TIP=$(git rev-parse refs/reposix/origin/main); echo "TIP after seed: $TIP"

echo "### CURRENT BEHAVIOR: second snapshot PARENTLESS, changed content ###"
printf -- '---\nid: 1\n---\nv2-CHANGED\n' > /tmp/rbf-fix-blob2
set +e
git fast-import --quiet <<EOF 2>&1 | sed 's/^/OLD> /'
blob
mark :1
data $(wc -c < /tmp/rbf-fix-blob2)
$(cat /tmp/rbf-fix-blob2)
commit refs/reposix/origin/main
mark :2
committer reposix-helper <bot@reposix> 0 +0000
data 24
Sync from REST snapshot
M 100644 :1 issues/1.md
done
EOF
echo "OLD exit=$? ; ref now: $(git rev-parse refs/reposix/origin/main)"
set -e

echo "### PROPOSED FIX: second snapshot with from <TIP> (descendant) ###"
set +e
git fast-import --quiet <<EOF 2>&1 | sed 's/^/FIX> /'
blob
mark :1
data $(wc -c < /tmp/rbf-fix-blob2)
$(cat /tmp/rbf-fix-blob2)
commit refs/reposix/origin/main
mark :2
committer reposix-helper <bot@reposix> 0 +0000
data 24
Sync from REST snapshot
from $TIP
M 100644 :1 issues/1.md
done
EOF
echo "FIX exit=$? ; ref now: $(git rev-parse refs/reposix/origin/main)"
set -e
echo "=== ref history (should be linear, 2 commits) ==="
git log --oneline refs/reposix/origin/main | sed 's/^/  /'
echo "DONE D=$D"
