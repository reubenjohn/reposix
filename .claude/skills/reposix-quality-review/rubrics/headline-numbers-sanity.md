# Rubric: subjective/headline-numbers-sanity

**Phase:** P61 Wave C scaffold (this stub).

**Implementation phase:** Wave E (61-05).

This stub is a placeholder so `dispatch.sh` can resolve the path. Wave E rewrites this file with the verbatim headline-numbers-sanity prompt body that cross-checks every headline number in README.md + docs/index.md against the source-of-truth artifacts in `benchmarks/` + `docs/benchmarks/`.

Numeric scoring: 10 = every headline number cites a benchmarks/ artifact and value is current; 5 = some numbers cite source but are stale (3-5pp drift); 2 = numbers stand alone with no source citation. PASS threshold >=7. Drift tolerance: <=2pp for percentage claims, within stated CI for absolute numbers.

See `.planning/phases/61-subjective-gates-skill-freshness-ttl/61-05-PLAN.md` for the prompt design.
