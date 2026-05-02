# Multi-project / multi-tenant

← [back to index](./index.md)

Already baked into the URL shape: `/projects/{slug}/...`. Multi-tenancy is achieved by:

1. **Foreign keys** — every issue has `project_slug`. The handler extracts `slug` from path, validates the project exists (404 otherwise), and scopes every query by `project_slug = ?`. There is no cross-project endpoint in v0.1; agents know they live in one project.
2. **Per-project workflow** — stored as JSON on the `projects` row. Agents `GET /projects/{slug}` to discover the rules. The FUSE layer caches this and the rules are advertised in `getxattr`.
3. **Per-project rate limiter buckets are NOT the design** — buckets are per-agent-token, not per-project. An agent that talks to two projects shares one bucket, mirroring real-world API behavior where the bucket is per credential.
4. **Per-project agent overrides** — table `agent_project_roles(agent_id, project_slug, role)` overrides the global role when present. Lets us model "alice is admin in proj-a, viewer in proj-b" without inventing a new role.

`/projects` listing returns only projects the caller has at least `viewer` access to. Slug collisions are forbidden by primary key; the seeder uses `proj-NN` to avoid them.
