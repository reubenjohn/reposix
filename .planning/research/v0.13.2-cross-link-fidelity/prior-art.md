# Cross-link fidelity — OSS prior art survey

Mapping the L0–L3 ladder against existing tools. L0=target resolves, L1=anchor resolves, L2=hash-drift since last grade, L3=LLM-graded fidelity (source still forecasts target).

## Tool comparison

| Tool | Max level | Per-edge config | Sub-section anchors | Git-hook integration | Subjective/LLM |
|---|---|---|---|---|---|
| [lychee](https://github.com/lycheeverse/lychee) | L1 | per-link regex include/exclude | yes (`--include-fragments=anchor-only\|text-only\|full`) | yes (pre-commit + GH Action) | no |
| [markdown-link-check](https://www.npmjs.com/package/markdown-link-check) | L0 | regex skip/replace + inline HTML comments | no (anchors explicitly skipped) | yes (`.pre-commit-hooks.yaml`) | no |
| [linkinator](https://www.npmjs.com/package/linkinator) | L1 | per-link skip patterns | yes (`--check-fragments`) | unclear | no |
| [remark-validate-links](https://github.com/remarkjs/remark-validate-links) | L1 | rule-level `.remarkrc` | yes (heading targets in md files) | yes (remark-cli + husky) | no |
| [remark-lint-no-dead-urls](https://github.com/remarkjs/remark-lint-no-dead-urls) | L0 + remote ID probe | rule-level | partial (probes IDs on rendered pages) | yes | no |
| [markdownlint MD051](https://github.com/DavidAnson/markdownlint/blob/main/doc/md051.md) | L1 (intra-doc) | rule-level, `ignore_case` | yes (heading slugs, `{#named}`, HTML `id`/`name`) | yes (pre-commit) | no |
| [html-proofer](https://github.com/gjtorikian/html-proofer) | L1 | subclass `HTMLProofer::Check` | yes (post-render) | yes (Jekyll CI) | no |
| [mkdocs-htmlproofer-plugin](https://github.com/manuzhang/mkdocs-htmlproofer-plugin) | L1 | URL wildcard ignore + status excludes | yes (rendered HTML) | indirect (runs in `mkdocs build`) | no |
| [awesome_bot](https://github.com/dkhamsing/awesome_bot) | L0 | whitelist + status allow-list | no | yes (CI) | no |
| [broken-link-checker](https://www.npmjs.com/package/broken-link-checker) | L0 | per-link filters | partial (not a focus) | unclear | no |
| [vale](https://vale.sh/) | n/a (prose) | per-rule YAML, scoped by markup | n/a | yes | no — regex/NLP |
| [textlint](https://textlint.org/) | L0 via plugins | `.textlintrc` rule config | unclear | yes | no |
| [DeepEval](https://github.com/confident-ai/deepeval) / G-Eval | L3 (general LLM-judge) | per-test-case Python | n/a | yes (pytest CI) | yes (LLM-as-judge + CoT) |
| [Evidently](https://github.com/evidentlyai/evidently) | embedding/concept drift on data | metric-level | n/a | yes (CI) | yes (statistical + LLM) |

**L2 (hash-drift):** no surveyed link-checker tracks target-content drift since last grade. Generic checksum/FIM tooling does it for files but is not doc-graph-aware. reposix's `quality/catalogs/doc-alignment.json` already does L2-style hash binding for claim→test edges — extending the pattern to `[A](B)` edges is the natural move.

## L3 gap and closest prior art

**The L3 gap is total.** No surveyed OSS tool grades whether `[A](B)` is a faithful framing of `B`'s current content. L0/L1 are saturated (lychee, html-proofer, MD051, linkinator). L2 has no doc-graph-aware implementation. LLM-as-judge frameworks (DeepEval/G-Eval, Evidently) are RAG/output evaluators — they assume a query+response pair, not an edge in a markdown DAG.

**Closest subjective-on-doc-graphs:** none with a doc-graph model. DeepEval+G-Eval is the closest *primitive* (custom-criteria LLM-as-judge with CoT, pytest-native, CI-friendly) and could be wrapped per-edge, but it ships no edge model, no per-edge config, and no hash-drift cache — those are reposix's to build. Vale is rule-based, not semantic. A graph-aware L3 grader stacked on a lychee+MD051-style L0/L1/L2 foundation appears unprecedented in OSS.
