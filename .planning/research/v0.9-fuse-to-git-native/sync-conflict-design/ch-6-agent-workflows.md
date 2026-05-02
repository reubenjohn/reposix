# 6. Agent Workflow Examples

← [back to index](./index.md)

### 6.1 Read Workflow -- Browsing Confluence Pages

```bash
# One-time setup (done by reposix init, not by the agent)
git clone --filter=blob:none --no-checkout reposix://confluence/ENGINEERING ~/eng-docs
cd ~/eng-docs

# Agent scopes to Q4 pages
git sparse-checkout set pages/2024-Q4/

# Agent reads a specific page -- one blob fetched on demand
cat pages/2024-Q4/quarterly-review.md

# Agent searches within scoped pages -- blobs already cached locally
grep -r "OKR" pages/2024-Q4/

# Agent discovers a page outside scope -- fetches just that blob
git checkout origin/main -- pages/2024-Q3/retrospective.md
cat pages/2024-Q3/retrospective.md
```

### 6.2 Write Workflow -- Updating a GitHub Issue

```bash
cd ~/project-issues

# Agent edits an issue
cat issues/fix-login-timeout.md
# ... reads current content ...

vim issues/fix-login-timeout.md
# ... makes changes ...

git add issues/fix-login-timeout.md
git commit -m "update fix-login-timeout: add reproduction steps"

# Push triggers conflict check + REST write
git push origin main
# To reposix://github/acme/webapp
#    a1b2c3d..e4f5g6h  main -> main
```

### 6.3 Conflict Workflow -- Concurrent Modification

```bash
# Agent pushes, but someone else edited the issue on GitHub
git push origin main
# ! [remote rejected] main -> main (reposix: fix-login-timeout.md was modified on backend since last fetch)
# error: failed to push some refs to 'reposix://github/acme/webapp'

# Standard git recovery
git pull --rebase origin main
# helper fetches ONLY the conflicting file's new content
# rebase auto-merges or drops into conflict resolution

# If auto-merged:
git push origin main
# success

# If conflict markers present:
vim issues/fix-login-timeout.md    # resolve <<<<<<< markers
git add issues/fix-login-timeout.md
git rebase --continue
git push origin main
```

### 6.4 Discovery Workflow -- Monitoring for Changes

```bash
# Agent periodically checks for backend changes
git fetch origin
# helper calls GET /issues?since=<last_fetched_at>
# tree updated, no blobs transferred

# See what changed
git diff --name-only origin/main
# issues/new-feature-request.md
# issues/fix-login-timeout.md

# Fetch only the interesting one
git checkout origin/main -- issues/new-feature-request.md
cat issues/new-feature-request.md
```

### 6.5 Bulk Creation Workflow -- Agent Creates Multiple Issues

```bash
# Agent creates several new issue files
cat > issues/improve-caching.md << 'EOF'
---
title: "Improve Redis caching layer"
state: open
labels: [enhancement, performance]
---

The current caching layer has no TTL management...
EOF

cat > issues/fix-memory-leak.md << 'EOF'
---
title: "Fix memory leak in worker pool"
state: open
labels: [bug, critical]
---

Workers are not releasing connections...
EOF

git add issues/improve-caching.md issues/fix-memory-leak.md
git commit -m "create two new issues"

# Single push, helper creates both via REST
git push origin main
# helper: POST /repos/acme/webapp/issues (improve-caching)
# helper: POST /repos/acme/webapp/issues (fix-memory-leak)
# To reposix://github/acme/webapp
#    h7i8j9k..l0m1n2o  main -> main
```
