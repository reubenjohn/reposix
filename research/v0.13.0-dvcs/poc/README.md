# v0.13.0 DVCS POC (POC-01)

**Throwaway research code per CARRY-FORWARD POC-DVCS-01.** Lives outside
`crates/`; not a workspace member; not shipped in any release; not a
catalog row; not a regression target.

## What this is

End-to-end POC exercising three integration paths against the simulator
BEFORE the production `reposix attach` subcommand is designed:

- **Path (a)** — `reposix attach`-shaped reconciliation against a
  deliberately-mangled checkout. Exercises the 5-row reconciliation
  table from `architecture-sketch.md`.
- **Path (b)** — bus-remote-shaped push observing mirror lag. SoT-first
  sequencing with simulated mirror failure + recovery.
- **Path (c)** — cheap-precheck refusing fast on SoT version mismatch.

## How to run

```bash
bash research/v0.13.0-dvcs/poc/run.sh
```

Assumes the workspace is buildable. `run.sh` starts/stops a sim
subprocess on a non-default port (`POC_SIM_PORT`, default `7888`) so it
won't collide with a developer's running sim on `:7878`.

Findings live in `POC-FINDINGS.md`. Transcripts land in `logs/`.

## Cleanup

```bash
rm -rf /tmp/reposix-poc-79*
```

The runner uses a dedicated cache dir (`REPOSIX_CACHE_DIR=/tmp/reposix-poc-79-cache`)
and dedicated working tree (`/tmp/reposix-poc-79-checkout`) so cleanup
is fully scoped to `/tmp/reposix-poc-79*`.

## What this is NOT

- Production code. Lives in `research/`, not `crates/`.
- A workspace member. The scratch crate at `scratch/Cargo.toml` has
  its own empty `[workspace]` table to declare standalone.
- A catalog row. Production rows land in 79-02.
- A regression target. The simulator-driven `run.sh` is the success
  contract — no PR-level pre-push gate is added.
