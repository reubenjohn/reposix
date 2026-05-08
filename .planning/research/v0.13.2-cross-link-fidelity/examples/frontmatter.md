---
# Example frontmatter for a target doc that wants to provide grading context to L3 judges.
# This is what would appear at the top of docs/concepts/dvcs-topology.md after v0.13.2 ships.

# Existing frontmatter fields can coexist with the cross_link_fidelity block.
id: dvcs-topology
title: DVCS topology

# Cross-link-fidelity grounding for L3 judges grading inbound edges to this doc.
cross_link_fidelity:
  schema_version: "1.0.0"
  grading_audience: "agent or developer planning a v0.14+ phase that touches the cache or helper"
  grading_context: |
    This doc is the canonical reference for the three DVCS roles
    (SoT-holder / mirror-only consumer / round-tripper) and the
    mirror-lag refs invariant.

    Inbound edges from anchor READMEs should forecast that a reader
    can navigate to the right role section without reading the whole
    doc. Inbound edges from how-to guides should at minimum mention
    that mirror-lag refs measure the SoT-edit→mirror-sync gap, NOT
    current SoT state — this is a recurring confusion.
  must_forecast:
    - "three DVCS roles"
    - "mirror-lag refs invariant"
  scope_override:
    max_level: "L3"
---

# DVCS topology

[doc body would follow here]
