# Seed data — deterministic and realistic

← [back to index](./index.md)

The contract: `reposix-sim --seed 0xC0FFEE --db sim.db init` produces the exact same database every time. This is non-negotiable for reproducible tests and demo scripts.

### 5.1 Seeder shape

```rust
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub struct SeedConfig {
    pub seed: u64,
    pub n_projects: usize,
    pub n_agents_per_project: usize,
    pub n_issues_per_project: usize,
}

pub fn seed(conn: &mut Connection, cfg: SeedConfig) -> rusqlite::Result<()> {
    let mut rng = ChaCha8Rng::seed_from_u64(cfg.seed);
    let tx = conn.transaction()?;
    for p in 0..cfg.n_projects {
        let slug = format!("proj-{p:02}");
        // ... insert project ...
        for a in 0..cfg.n_agents_per_project {
            let role = match a {
                0 => "admin",
                1 | 2 => "viewer",
                _ => "contributor",
            };
            // ... insert agent with deterministic UUID derived from rng ...
        }
        for i in 1..=cfg.n_issues_per_project {
            // pick title from a fixed corpus of 200 phrases, indexed by rng
            // pick state weighted: 50% open, 25% in_progress, 15% in_review, 8% done, 2% closed
            // pick 0..3 labels from a fixed 10-label vocabulary
            // pick 0..2 assignees
            // assign created_at deterministically: base_ts + (i * 60s) + jitter
        }
    }
    tx.commit()
}
```

### 5.2 The default seed

`reposix-sim init` with no flags creates one project `reposix` with 50 issues, 8 agents (1 admin, 2 viewer, 5 contributor), and 5 labels. Just enough to make `ls /mnt/reposix/issues/` look interesting in the demo without being so much that scrolling becomes annoying.

### 5.3 Why `ChaCha8Rng` and not `StdRng`

`StdRng` is documented as not stable across rust releases. `ChaCha8Rng` from `rand_chacha` is. Test seeds need to outlive the crate version of the day they were filed.
