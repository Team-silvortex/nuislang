# `nuis` `beta-0.0.1` Documentation Sync Inventory

This file records the documentation migration from the completed
`alpha-0.20.*` line to `beta-0.0.1`.

## Current Entry Rule

Present-tense repository documentation starts with:

* [nuis-beta-0.0.1-mainline-entry.md](nuis-beta-0.0.1-mainline-entry.md)
* [../current-mainline-map.md](../../docs/current-mainline-map.md)
* [../reference/nuis-development-tensor.md](../../docs/reference/nuis-development-tensor.md)

The direct predecessor remains:

* [nuis-alpha-0.20-mainline-entry.md](nuis-alpha-0.20-mainline-entry.md)

Older alpha and pre-alpha files retain their original version facts. They are
history and architecture provenance, not competing current-line claims.

## Synchronized Surfaces

The beta entry refresh covers:

* repository, docs, reference, and versioning routers
* std, official Galaxy, example, Nsld, Nsdb, and Nsbdr entry READMEs
* the CFFI, FFI pointer, toolchain-core, linker, and development-tensor
  present-tense boundaries
* the long-range self-hosting wording for early beta
* development-tensor drift anchors for the current release entry

## Wording Rule

Use:

* `beta-0.0.1` for the exact current release
* `early beta` for the `beta-0.0.1` through `beta-0.9.*` foundation period
* `alpha-0.20.*` only for the direct predecessor closeout line
* `beta-0.10.*` for the planned start of formal staged self-hosting
* `gamma-0.5.*` for the current self-hosting completion target

Do not use `beta` as a synonym for stable APIs. Compatibility policy and API
freezes require their own explicit contracts.

## Portability Rule

Current documentation uses repository-relative paths and capability-based host
wording. It must not require a developer-specific absolute path or one fixed
operating-system release.
