# reposix demo suite

reposix demos come in three tiers. Pick the one that matches your
question:

- **Tier 1** — four ~60-second demos, one per audience. If you have
  one minute and want to know whether this thing is worth the next
  ten, start here.
- **Tier 2** — the full 9-step walkthrough. The monolith demo that
  ships as `scripts/demo.sh` (which is now a shim to
  [`scripts/demos/full.sh`](../../scripts/demos/full.sh)). Watch
  this if Tier 1 convinced you to spend more than a minute.
- **Tier 3** — sim-vs-real-backend parity demo. Tier 3 lands later
  in Phase 8 once the `reposix-github` read-only adapter is wired
  up; until then this table lists it as `(coming)`.

All demos are `set -euo pipefail`, self-cleaning, and bounded by a
90-second `timeout`. The smoke suite in
[`scripts/demos/smoke.sh`](../../scripts/demos/smoke.sh) runs the
four Tier 1 demos through [`scripts/demos/assert.sh`](../../scripts/demos/assert.sh)
and is what the `demos-smoke` CI job invokes on every push.

## Tier 1 — audience-specific 60s demos

| # | Demo                                                                       | Audience    | Runtime | What it proves                                                                                      | Recording                                                                                                                                                                         |
|---|----------------------------------------------------------------------------|-------------|--------:|-----------------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 1 | [`01-edit-and-push.sh`](../../scripts/demos/01-edit-and-push.sh)           | developer   |    ~60s | FUSE mount + `cat`/`sed` edit + `git push` round-trip changes server state end-to-end.              | [typescript](recordings/01-edit-and-push.typescript) · [transcript](recordings/01-edit-and-push.transcript.txt)                                                                   |
| 2 | [`02-guardrails.sh`](../../scripts/demos/02-guardrails.sh)                 | security    |    ~60s | SG-01 allowlist refusal + SG-02 bulk-delete cap + SG-03 sanitize-on-egress all fire on camera.      | [typescript](recordings/02-guardrails.typescript) · [transcript](recordings/02-guardrails.transcript.txt)                                                                         |
| 3 | [`03-conflict-resolution.sh`](../../scripts/demos/03-conflict-resolution.sh) | skeptic     |    ~60s | `If-Match` + 409 `version_mismatch` is what git turns into a native merge conflict on push.         | [typescript](recordings/03-conflict-resolution.typescript) · [transcript](recordings/03-conflict-resolution.transcript.txt)                                                       |
| 4 | [`04-token-economy.sh`](../../scripts/demos/04-token-economy.sh)           | buyer       |    ~10s | 92.3% fewer tokens vs MCP-mediated baseline for the same task.                                      | [typescript](recordings/04-token-economy.typescript) · [transcript](recordings/04-token-economy.transcript.txt)                                                                   |

### Audience guide

- **developer** — will I want to use this? (Demo 1 shows the shell
  ergonomics are real.)
- **security** — can I hand this to an autonomous agent? (Demo 2
  shows the lethal-trifecta cuts are mechanical.)
- **skeptic** — what happens when two agents race on the same
  issue? (Demo 3 shows conflicts surface as first-class git
  conflicts, not a bespoke protocol.)
- **buyer** — is the ROI claim real? (Demo 4 is a 10-second
  benchmark against an MCP baseline.)

## Tier 2 — full 9-step walkthrough

The unabridged walkthrough (same script pre-Phase-8-A, moved into
the suite but substantively unchanged):

| Demo                                          | Runtime | Recording                                                                                                                |
|-----------------------------------------------|--------:|--------------------------------------------------------------------------------------------------------------------------|
| [`demos/full.sh`](../../scripts/demos/full.sh) | ~90s    | [docs/demo.typescript](../demo.typescript) · [docs/demo.transcript.txt](../demo.transcript.txt) · [walkthrough prose](../demo.md) |

`scripts/demo.sh` remains as a backwards-compat shim that execs
`scripts/demos/full.sh`, so existing users and docs that reference
`bash scripts/demo.sh` keep working.

## Tier 3 — sim vs real-backend parity

| Demo                                       | Runtime | What it proves                                                                                              | Status                          |
|--------------------------------------------|--------:|-------------------------------------------------------------------------------------------------------------|---------------------------------|
| `demos/parity.sh` (coming in Phase 8-B/C)  |   ~30s  | Running `reposix list --backend sim --project demo` and `reposix list --backend github --project octocat/Hello-World` produces structurally identical JSON (modulo content). | shipping with `reposix-github` adapter |

## Running the suite yourself

```bash
# Build release binaries once.
cargo build --release --workspace --bins
export PATH="$PWD/target/release:$PATH"

# One demo.
bash scripts/demos/01-edit-and-push.sh

# One demo, with marker-assertion enforcement.
bash scripts/demos/assert.sh scripts/demos/01-edit-and-push.sh

# Full Tier 1 smoke suite (what CI runs).
bash scripts/demos/smoke.sh
```

Each demo self-installs an EXIT trap that tears down its FUSE mount,
kills the simulator, and removes its tmp scratch directory. Re-runs
are idempotent. A stuck demo is bounded by an internal `timeout 90`
so the smoke suite stays within its CI budget.
