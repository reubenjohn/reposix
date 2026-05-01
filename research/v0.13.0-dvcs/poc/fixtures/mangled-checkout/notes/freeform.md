<!-- Reconciliation case 3: NO_ID (architecture-sketch.md § "Reconciliation
     cases" row 3). This file has no YAML frontmatter and no `id` field.
     Reconciliation should warn + skip (not a reposix-managed file). -->

# Free-form note

This file has no YAML frontmatter and no `id` field. The reconciliation
walk should treat it as an opaque file the user authored outside
reposix's frontmatter contract — warn and skip without aborting attach.

Reconciliation case 5 (MIRROR_LAG) is also exercised by this fixture
set: the simulator is seeded with a record that has no corresponding
local file. See `path-a.sh` for the seed config.
