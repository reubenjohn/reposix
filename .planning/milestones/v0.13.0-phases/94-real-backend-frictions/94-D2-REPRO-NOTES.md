# P94 D2 prove-before-fix — repro notes (DP-2 discipline)

**Author:** P94 D2 lane. **Date:** 2026-07-05. **Finding:** SURPRISES-INTAKE.md L602-610
(git-2.43 single-backend push failure, HIGH). **Method:** container-based
(`docker run ubuntu:24.04`, stock apt git 2.43.0, `--network host` to reach a host-spawned
`reposix-sim` on 127.0.0.1:7878, host-built binaries bind-mounted read-only — host glibc <=
container glibc so the host binary runs in-container). Runnable driver:
`.planning/phases/94-real-backend-frictions/94-D2-git243-repro.sh`; committed RED transcript:
`94-D2-repro-transcript.txt`. Reusable regression gate:
`quality/gates/agent-ux/p94-git243-fallback-sentinel.sh`.

## Why a container

This dev box's system git is 2.25.1 (< the project's `git >= 2.34` floor), so the
2.43-windowed regression cannot be reproduced natively. `ubuntu:24.04` (noble) ships git
2.43.0 in stock apt — an apples-to-apples proof environment. `git init`/fetch
(`stateless-connect git-upload-pack`) works on 2.43; the failure is push-only. The
`GIT_CHANNEL=ppa` toggle upgrades the container to the git-core PPA (2.54.0, this project's
CI version) for the version-comparison run below.

## DP-2 result — the finding EXECUTES, and the intake MISDIAGNOSED the mechanism

**RED (pre-fix helper, git 2.43.0):** a real `reposix init sim::demo` + edit + `git push
origin main` exits **128** with no further output — matching the intake's documented symptom
("exit 128, no helper output"). Proven twice (see transcript).

**Root-cause trace (`GIT_TRANSPORT_HELPER_DEBUG=1` + a helper verb-log at
`RUST_LOG=debug`) — the intake's diagnosis is WRONG:**

The intake claimed: *"git tries `stateless-connect git-receive-pack` FIRST for the push;
the helper's custom `unsupported service:` reply (instead of the `fallback` sentinel) aborts
the push."* The trace disproves this. git **never** probes `stateless-connect` for the push
direction on EITHER version:

```
git 2.43.0:  capabilities -> option object-format -> helper `unsupported` -> git DIES 128
                                                     (before any list/export)
git 2.54.0:  capabilities -> list -> export -> ok -> push SUCCEEDS
                             (no `option object-format` sent at all)
```

The real, version-windowed blocker: the helper blanket-answered **`option object-format`**
with `unsupported`. git 2.43 sends that option (because the helper advertises the
`object-format` capability) and treats an `unsupported` reply as **fatal**. git 2.54 skips
the option entirely (the advertised `object-format=sha1` capability suffices), which is why
`>= 2.34` did not protect and CI (git 2.54) never saw it. The stateless-connect `fallback`
sentinel the intake fixated on is dead code for the push path.

**GREEN (fixed helper, git 2.43.0):** with the helper answering `option object-format
{<empty>|true|sha1}` → `ok` (reposix cache is sha1-only; non-sha1 → `error`), the same real
push exits **0** (`* [new branch] main -> main`). Confirmed by the container arm of the
verifier.

## The fix (two parts, both committed)

1. **`crates/reposix-remote/src/main.rs` `option` handler** — answer `option object-format`
   with `ok` for the sha1/true/bare forms, `error` for other algorithms. **This is what
   actually closes the git-2.43 push regression** (catalog assert c).
2. **`crates/reposix-remote/src/stateless_connect.rs`
   `handle_stateless_connect`** — reply the git-remote-helpers(7) `fallback` sentinel (not a
   custom `unsupported service:` line) for a non-upload-pack service, returning
   `StatelessConnectOutcome::FellBack` so the verb loop resumes. Spec-compliance /
   defensive; satisfies catalog asserts a+b. NOT the push-unblocker (proven: buggy and
   fallback-only helpers both exit 128 at the `option` stage).

The e2e test `stateless_connect_replies_fallback_for_non_upload_pack_service` (renamed from
the lying `_rejects_`) and the new `option_object_format_{sha1_replies_ok,
non_sha1_replies_error}` tests pin both halves.

## DP-2 verdict

**GREEN** on the true claim ("a real git-2.43 single-backend push exits 0 after the fix"),
proven RED→GREEN in the ubuntu:24.04 container. The intake's fallback-sentinel mechanism was
a misdiagnosis; the catalog row `agent-ux/p94-git243-fallback-sentinel` description +
assert-c rationale inherit that misdiagnosis (the OUTCOME — push exit 0 — still holds, so the
row's asserts are met). Filed to SURPRISES-INTAKE for a catalog-description correction pass,
and flagged upward to the coordinator.
