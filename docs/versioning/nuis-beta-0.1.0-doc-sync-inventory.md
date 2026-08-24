# `nuis` `beta-0.1.0` Documentation Sync Inventory

> Historical synchronization snapshot. For the current mainline, start with
> [nuis-beta-0.6.0-mainline-entry.md](nuis-beta-0.6.0-mainline-entry.md).

This file records the then-current documentation migration from the first beta
foundation snapshot to `beta-0.1.0`.

## Current Entry Rule

Present-tense repository documentation starts with:

* [nuis-beta-0.1.0-mainline-entry.md](nuis-beta-0.1.0-mainline-entry.md)
* [../current-mainline-map.md](../current-mainline-map.md)
* [../reference/nuis-development-tensor.md](../reference/nuis-development-tensor.md)

The recorded predecessor remains:

* [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)

Older beta, alpha, and pre-alpha files retain their original version facts.
They are historical snapshots and architecture provenance, not competing
current-line claims.

## Synchronized Surfaces

The `beta-0.1.0` refresh covers:

* repository, docs, reference, and versioning routers
* std, tooling, and Nsld entry READMEs
* CFFI, Nustar, linker, and development-tensor present-tense boundaries
* the new first-class `official.cffi` registration and `mod cffi` source rule
* provider-neutral DPU/IPU wording under the Data Nustar direction
* development-tensor drift anchors for the current release entry

## Wording Rule

Use:

* `beta-0.1.0` for the exact release recorded by this snapshot
* `early beta` for the `beta-0.0.*` through `beta-0.9.*` foundation period
* `beta-0.0.1` only for the recorded first-beta predecessor snapshot
* `alpha-0.20.*` for the alpha closeout line
* `beta-0.10.*` for the planned start of formal staged self-hosting
* `gamma-0.5.*` through `gamma-0.10.*` for the self-hosting completion window

Do not use `beta` as a synonym for stable APIs. Compatibility policy and API
freezes require their own explicit contracts.

## Portability Rule

Current documentation uses repository-relative paths and capability-based host
wording. It must not require a developer-specific absolute path, fixed
operating-system release, or one vendor's DPU/IPU implementation.
