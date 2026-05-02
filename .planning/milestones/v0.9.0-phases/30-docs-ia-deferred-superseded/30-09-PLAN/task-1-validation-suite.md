← [back to index](./index.md)

# Task 1: Run the full validation suite (mkdocs, vale, tutorial test, structural linter)

<task type="auto">
  <name>Task 1: Run the full validation suite (mkdocs, vale, tutorial test, structural linter)</name>
  <files></files>
  <read_first>
    - `.planning/phases/30-docs-ia-and-narrative-overhaul-landing-page-aha-moment-and-p/30-VALIDATION.md` §"Validation Sign-Off"
    - `scripts/test_phase_30_tutorial.sh` (plan 30-06)
    - `scripts/check_phase_30_structure.py` (plan 30-01)
    - `.vale.ini` + `.vale-styles/` (plan 30-01)
  </read_first>
  <action>
    Run every automated gate in order. Capture each exit code and log the result. This task MUST be fully automated (autonomous: true at the task level); its verification is that each gate returns 0.

```bash
# Gate 1 — mkdocs build strict
mkdocs build --strict 2>&1 | tee /tmp/30-09-mkdocs.log
# expected: exit 0. Site builds, all internal links resolve, no orphan warnings.

# Gate 2 — Vale lint (P1 + P2 rules over entire docs/)
~/.local/bin/vale --config=.vale.ini docs/ 2>&1 | tee /tmp/30-09-vale.log
# expected: exit 0. No P1 (replace) on index.md; no P2 above Layer 3.

# Gate 3 — Structural linter
python3 scripts/check_phase_30_structure.py 2>&1 | tee /tmp/30-09-structure.log
# expected: exit 0. All pages exist + all deleted paths gone + all counts match.

# Gate 4 — Tutorial end-to-end
# Prerequisite: cargo build --release --workspace --bins has run.
cargo build --release --workspace --bins 2>&1 | tail -5 > /tmp/30-09-cargo.log
bash scripts/test_phase_30_tutorial.sh 2>&1 | tee /tmp/30-09-tutorial.log
# expected: exit 0. Step 1-4 green, version bumped 1 → 2.
```

If any gate returns non-zero, STOP and triage. Do NOT proceed to screenshot capture or SUMMARY composition until all four gates return 0.

For each gate, capture the exit code and timing:

```bash
for log in /tmp/30-09-mkdocs.log /tmp/30-09-vale.log /tmp/30-09-structure.log /tmp/30-09-tutorial.log; do
  echo "=== $log ==="
  tail -5 "$log"
done
```

Summarize in `/tmp/30-09-gate-summary.txt`:

```
Gate 1 mkdocs --strict: PASS (6.8s)
Gate 2 vale docs/:      PASS (2.1s)
Gate 3 structure:       PASS (0.3s)
Gate 4 tutorial e2e:    PASS (18.2s)
Total: ~27s. Budget was 90s.
```

This summary feeds Task 4's SUMMARY composition.
  </action>
  <verify>
    <automated>mkdocs build --strict && ~/.local/bin/vale --config=.vale.ini docs/ && python3 scripts/check_phase_30_structure.py && bash scripts/test_phase_30_tutorial.sh</automated>
  </verify>
  <acceptance_criteria>
    - `mkdocs build --strict` exits 0.
    - `~/.local/bin/vale --config=.vale.ini docs/` exits 0.
    - `python3 scripts/check_phase_30_structure.py` exits 0.
    - `bash scripts/test_phase_30_tutorial.sh` exits 0.
    - Combined runtime < 90 seconds (30-VALIDATION.md max feedback latency).
  </acceptance_criteria>
  <done>
    All 4 automated gates green. Phase 30 artifact invariants pass. Ready for screenshot capture + cold-reader review.
  </done>
</task>
