# Roadmap

A bird's-eye view of where reposix is heading — grouped by **capability, not by release
number or date**, so the map stays true without weekly edits.

> **Source of truth.** This page mirrors the private planning ledger at
> [`.planning/PROJECT.md`](https://github.com/reubenjohn/reposix/blob/main/.planning/PROJECT.md)<!-- SYNC: paired with .planning/PROJECT.md § Current Milestone. Edit either side → update the other; re-color the arcs (shipped / active / future) at milestone close. -->,
> driven by the GSD planning workflow. It is a public snapshot — it lags the ledger by
> design and is refreshed at milestone close, so it never promises a date it has to chase.

```mermaid
graph TB
    SHIPPED["<b>Shipped</b><br/>Edit issues and wiki pages as files, across four trackers<br/>Push edits back with commit and push<br/>Docs, tutorials and quality gates<br/>Team mirror and sync"]

    FLOOR["<b>Now — Floor</b><br/>Fix known correctness gaps<br/>Every error message teaches the fix<br/>Honest, freshly measured numbers<br/>Simpler docs and planning"]

    F1["Stronger quality gates"]
    F2["Rebuilt docs and navigation"]
    F3["Verified benchmark numbers"]
    F4["End-to-end journey walkthroughs"]

    GOAL["<b>Public launch</b><br/>Live terminal demo<br/>Honest headline numbers<br/>One-command install<br/>Show-HN launch kit"]

    SHIPPED ==> FLOOR
    FLOOR ==> F1 & F2 & F3 & F4
    F1 & F2 & F3 & F4 ==> GOAL

    classDef shipped fill:#c8e6c9,stroke:#2e7d32,stroke-width:1px,color:#12331a;
    classDef active fill:#bbdefb,stroke:#1565c0,stroke-width:2px,color:#0d2c4d;
    classDef future fill:#eeeeee,stroke:#8d8d8d,stroke-width:1px,color:#2b2b2b;
    classDef goal fill:#ffce3a,stroke:#e08e00,stroke-width:3px,color:#3d2e00;

    class SHIPPED shipped;
    class FLOOR active;
    class F1,F2,F3,F4 future;
    class GOAL goal;
```

## How to read this

The map has three states plus the end state we are steering toward. Each box lists its
arcs; the arrows flow from what is done, through what we are doing now, into the launch.

- **Green — Shipped.** Capabilities you can use today: edit a tracker's issues and a wiki's
  pages as plain files across four backends, push your edits back with a commit, and rely on
  the docs, tutorials, quality gates, and team mirroring already in place.
- **Blue — Now (the "Floor" milestone).** The launch-readiness floor we are building:
  closing known correctness gaps, hardening every error message so it teaches the fix,
  re-measuring the numbers honestly, and simplifying the docs and planning.
- **Grey — Ahead.** The arcs still in front of us. They fan out from the Floor —
  stronger quality gates, rebuilt docs and navigation, verified benchmark numbers, and
  end-to-end journey walkthroughs — and converge on the launch.
- **Gold — Public launch.** The end state: a live terminal demo, honest headline numbers,
  a one-command install, and a Show-HN launch kit.

Because the map is drawn by capability, it names no phase numbers and no dates. For live
phase status and the current milestone in detail, follow the source-of-truth link above.
