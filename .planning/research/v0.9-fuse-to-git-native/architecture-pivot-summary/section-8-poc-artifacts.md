[index](./index.md)

# 8. POC Artifacts

All artifacts are in `poc/` subdirectory of this research folder:

| File | Purpose |
|---|---|
| `poc/git-remote-poc.py` | Python implementation of a `stateless-connect` + `export` hybrid helper. Demonstrates partial clone, lazy blob fetching, push via fast-import, and push rejection with custom error messages. Extended from the read-path POC to cover the full hybrid. |
| `poc/run-poc.sh` | Runner script for the read-path POC. Runs inside `alpine:latest` Docker container (git 2.52). Demonstrates: partial clone with `--filter=blob:none`, lazy blob fetch via `cat-file`, sparse-checkout batching. |
| `poc/run-poc-push.sh` | Runner script for the push-path POC. Extends `run-poc.sh` with: commit + push via `export`, push rejection with custom error message, capability-usage counting. |
| `poc/poc-helper-trace.log` | Full protocol trace from the read-path POC. Shows three helper invocations (clone, lazy fetch #1, lazy fetch #2) with request/response byte counts and missing-blob counts. |
| `poc/poc-push-trace.log` | Full protocol trace from the push-path POC (114 lines). Shows 2 `stateless-connect` invocations + 2 `export` invocations (accept + reject). |

## Running the POCs

Read-path POC:
```bash
docker run --rm -v $(pwd)/.planning/research/v0.9-fuse-to-git-native/poc:/work alpine:latest \
  sh -c 'apk add --quiet --no-cache git python3 && \
         cp /work/git-remote-poc.py /work/git-remote-poc && \
         chmod +x /work/git-remote-poc && /work/run-poc.sh'
```

Push-path POC:
```bash
docker run --rm -v $(pwd)/.planning/research/v0.9-fuse-to-git-native/poc:/work alpine:latest \
  sh -c 'apk add --quiet --no-cache git python3 && \
         cp /work/git-remote-poc.py /work/git-remote-poc && \
         chmod +x /work/git-remote-poc && /work/run-poc-push.sh'
```

## POC bugs documented for Rust port

Three non-obvious bugs were discovered during POC development. They are documented in `push-path-stateless-connect-findings.md` with root causes and fixes:

1. **Refspec namespace collapse (critical).** Advertising `refs/heads/*:refs/heads/*` causes fast-export to emit an empty delta. Fix: use a private namespace like `refs/heads/*:refs/reposix/*`.
2. **Naive `commit ` line matching in export parser.** A `line.startswith("commit ")` check also matches commit message bodies that start with "commit ". Fix: require `line.startswith("commit refs/")`, or better, use a state-machine parser that skips data payloads.
3. **Python subprocess stdin handling.** `proc.communicate()` after `proc.stdin.close()` raises `ValueError`. Not applicable to Rust, but relevant for agent prototyping.
