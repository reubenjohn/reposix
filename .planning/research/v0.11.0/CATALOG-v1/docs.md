← [back to index](./index.md)

# Bucket-by-bucket review — `docs/`

#### `docs/index.md`
**Disposition:** keep. Hero, three measured numbers, "Tested against" — fully post-pivot, references `v0.9.0-latency.md`. Owner of v0.10.0 Phase 40.

#### `docs/concepts/{mental-model-in-60-seconds.md, reposix-vs-mcp-and-sdks.md}`
**Disposition:** keep. Both written for the v0.10.0 milestone; clean.

#### `docs/how-it-works/{filesystem-layer.md, git-layer.md, trust-model.md}`
**Disposition:** keep. The post-pivot trio (Phase 41). Each has a mermaid diagram. `filesystem-layer.md` correctly frames the v0.1 FUSE design as superseded.

#### `docs/tutorials/first-run.md`
**Disposition:** keep. The 5-minute tutorial — load-bearing for DOCS-06.

#### `docs/guides/{write-your-own-connector.md, integrate-with-your-agent.md, troubleshooting.md}`
**Disposition:** keep. Newly written for v0.10.0 Phase 42. `troubleshooting.md` references a `[remote rejected] main -> main (fetch first)` flow that maps to ARCH-08.

#### `docs/reference/`
| File | Disposition | Notes |
|---|---|---|
| `cli.md` | rewrite-needed | Lists the deleted `mount` and `demo` subcommands, calls reposix "git-backed FUSE filesystem". Needs full rewrite for the v0.9.0 CLI (`init`, `list`, `refresh`, `sim`, `spaces`, `version`). |
| `crates.md` | rewrite-needed | Lists `reposix-fuse` as a workspace crate; missing `reposix-cache` + `reposix-jira`; references `IssueBackend` (renamed); still says "Ships in v0.2/v0.3". Needs full re-write. |
| `git-remote.md` | rewrite-needed | Capabilities listed as `import`/`export`/`refspec` only — missing `stateless-connect`. Says "the same function the FUSE daemon uses" — pre-pivot. v0.2 backlog section is outdated. |
| `confluence.md` | rewrite-needed | "FUSE daemon and reposix list" + "Mount the space as a POSIX directory of Markdown files" + "## FUSE mount layout (v0.4+)" — full rewrite to match the v0.9.0 init/clone flow. |
| `jira.md` | rewrite-needed | Section "Mount as FUSE Filesystem" — replace with `reposix init jira::TEST /tmp/repo`. Otherwise content is recent (JIRA shipped Phase 28). |
| `http-api.md` | keep (annotate) | Documents the simulator's REST shape; references "FUSE" twice in pass-through context. Two-line annotation suffices. |
| `simulator.md` | keep | Already post-pivot (talks about the cache + helper). |
| `testing-targets.md` | keep | New for v0.9.0 Phase 36; documents the three sanctioned targets. One mention of "FUSE-free transport" — fine framing. |

#### `docs/decisions/`
| File | Disposition | Notes |
|---|---|---|
| `001-github-state-mapping.md` | keep (annotate) | References "FUSE-mounted views" as the user-facing surface. Add a brief note at the top: "v0.9.0 supersedes the FUSE-rendering claim; the status mapping itself is unchanged and authoritative." |
| `002-confluence-page-mapping.md` | keep (annotate) | "Option A: flat" decision is supplanted by ADR-003; ADR-003 is supplanted by the v0.9.0 partial-clone working tree. Add a "scope superseded" header note. |
| `003-nested-mount-layout.md` | keep (annotate) | Scope: "the FUSE mount root layout emitted by `reposix-fuse`" — codebase no longer exists. Add an "obsolete; see filesystem-layer.md" header. |
| `004-backend-connector-rename.md` | keep | Recent (Phase 27); clean. |
| `005-jira-issue-mapping.md` | keep | Recent (Phase 28); 1 stale mention of "FUSE filenames" — minor sweep. |

#### `docs/research/`
| File | Disposition | Notes |
|---|---|---|
| `initial-report.md` | keep | Has explicit "pre-v0.1 design research" banner; correctly historical. |
| `agentic-engineering-reference.md` | keep | The dark-factory / lethal-trifecta reference; load-bearing. |

#### `docs/archive/{MORNING-BRIEF.md, PROJECT-STATUS.md}`
**Disposition:** keep. Both have explicit "Archived" header banners; correctly historical.

#### `docs/development/`
| File | Disposition | Notes |
|---|---|---|
| `contributing.md` | rewrite-needed | Workspace tree includes `reposix-fuse/`; the FUSE-callbacks-are-safe-Rust paragraph; "fusermount3 --version" prereq. Needs an architectural-pivot pass. |
| `roadmap.md` | rewrite-needed | Stops at v0.7. v0.8 (JIRA), v0.9 (architecture pivot), v0.10 (docs) all missing. North-star list still mentions "macFUSE" + "ProjFS / WinFsp + reposix-fuse" — neither applies post-pivot. |

#### Redirect-stub pages
| File | Disposition | Notes |
|---|---|---|
| `architecture.md` | keep | "(moved)" stub pointing to how-it-works trio. |
| `security.md` | keep | "(moved)" stub pointing to trust-model. |
| `why.md` | keep | "(moved)" stub. |
| `demo.md` | keep | "(moved)" stub pointing to tutorials/first-run.md. |
| `connectors/guide.md` | keep | "(moved)" stub pointing to guides/write-your-own-connector. |

#### `docs/demos/`
| File | Disposition | Notes |
|---|---|---|
| `index.md` | rewrite-needed | "FUSE mount + cat/sed edit + git push round-trip" — full rewrite for v0.9.0. Tier 1/2/3 demo lineup needs to be re-cast: half the demos depended on `reposix mount`. |
| `recordings/01-edit-and-push.{transcript.txt, typescript}` | keep (or re-record) | FUSE-era recordings; either keep with an "archive" annotation or re-record per v0.10.0 Phase 45. README itself flags the demo gif as "FUSE-era — Phase 45 will re-record against `reposix init`". |
| `recordings/02-guardrails.{...}` | keep / re-record | Same. |
| `recordings/03-conflict-resolution.{...}` | keep / re-record | Same. |
| `recordings/04-token-economy.{...}` | keep | benchmark recording; not FUSE-tied. |
| `recordings/parity.{...}` | keep | sim-vs-real parity recording; FUSE-neutral. |
| `recordings/swarm.{...}` | keep | swarm load-test recording; only loosely FUSE-tied. |

#### `docs/social/` (17 files)
**Disposition:** keep. LinkedIn + Twitter copy + asset builders + rendered images. The architecture diagram (`architecture.mmd`/`.png`) and demo gif are FUSE-era — owner-flagged as Phase 45 work. Builder scripts (`_build_*.py`) are reusable.

#### `docs/screenshots/` (6 PNGs)
**Disposition:** keep. Site/landing screenshots — refresh in Phase 44/45.

#### `docs/benchmarks/v0.9.0-latency.md`
**Disposition:** keep. Sim column populated; real-backend cells `pending-secrets`. Regenerated by `scripts/v0.9.0-latency.sh`.
