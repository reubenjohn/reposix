---
name: 🔌 Connector proposal
about: Propose a new BackendConnector for a REST-addressable issue tracker / KB
title: "[CONNECTOR] "
labels: connector
---

## Backend overview

- Name:
- URL:
- Auth model: (API token, OAuth, basic)
- Rate limits:

## Read endpoints

- List equivalent (paginated?):
- Get equivalent:
- Delta query (since timestamp / cursor / etag):

## Write endpoints

- Create:
- Update (optimistic locking? ETag?):
- Delete / close:

## Conflict semantics

- Version field name:
- Behavior on stale write:

## Webhooks / events (for future delta-sync optimization)

## Test / sandbox accounts

Available for CI? If so, naming convention to avoid collisions:

## Owner / maintainer commitment

Are you willing to maintain this connector? Cadence?
