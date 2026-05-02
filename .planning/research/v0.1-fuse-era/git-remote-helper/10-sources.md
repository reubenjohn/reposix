← [back to index](./index.md)

# 10. Sources

### Authoritative (HIGH confidence)
- [git-scm.com — gitremote-helpers documentation](https://git-scm.com/docs/gitremote-helpers) — the canonical wire protocol spec.
- [git-scm.com — git-fast-import documentation](https://git-scm.com/docs/git-fast-import) — fast-import stream format.
- [git-scm.com — git-fast-export documentation](https://git-scm.com/docs/git-fast-export) — what git pipes into our stdin during `export`.
- [kernel.org — gitremote-helpers(7) man page](https://www.kernel.org/pub/software/scm/git/docs/gitremote-helpers.html) — same spec, mirror.

### Reference implementations (HIGH confidence — actual source code reviewed)
- [felipec/git-remote-hg](https://github.com/felipec/git-remote-hg) — Python; the textbook `import`/`export` helper. Code quoted verbatim above.
- [git-bug/git-bug](https://github.com/git-bug/git-bug) — Go; the cautionary counter-example (custom CLI instead of helper protocol).
- [git-bug third-party bridges doc](https://github.com/git-bug/git-bug/blob/master/doc/usage/third-party.md) — explains the bridge UX (a CLI, not git-push).
- [awslabs/git-remote-s3](https://github.com/awslabs/git-remote-s3) — Rust; closest existing Rust prior art (uses bundle protocol, not fast-import).

### Background / context (MEDIUM confidence)
- [Andrew Nesbitt — Git Remote Helpers](https://nesbitt.io/2026/03/18/git-remote-helpers.html) — recent (Mar 2026) overview survey.
- [Apriorit — Developing a Custom Remote Git Helper](https://www.apriorit.com/dev-blog/715-virtualization-git-remote-helper) — implementation walkthrough (couldn't fully extract via WebFetch but referenced for further reading).
- [git-remote-gcrypt](https://spwhitton.name/tech/code/git-remote-gcrypt/) — minimal `connect`-style helper in bash.
- [drgomesp/gitrmt](https://github.com/drgomesp/gitrmt) — Go library claiming to abstract the helper protocol; potentially worth a look but not reviewed in depth.

### Project context (read directly, HIGH confidence)
- `/home/reuben/workspace/reposix/.planning/PROJECT.md`
- `/home/reuben/workspace/reposix/docs/research/initial-report.md` §"Distributed Synchronization: The Git Remote Helper Protocol"
