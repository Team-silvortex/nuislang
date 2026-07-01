# `nuis` Minor-Version Snapshot Rule

Starting with the `0.16.*` line, each minor version should leave behind one
small, explicit history anchor.

The goal is not to create a giant changelog.

The goal is to make each minor line answer, quickly and honestly:

* what became real enough to stand on
* what changed in the visible toolchain surface
* what was intentionally still narrow at that point
* what commands and release gates were considered canonical

## Required Files Per Minor Line

For each minor line, add:

* `nuis-<minor>.0-snapshot.md`
* `nuis-<minor>.0-release-checklist.md`

Example:

* `nuis-0.16.0-snapshot.md`
* `nuis-0.16.0-release-checklist.md`

This repository currently treats the `.0` file as the anchor for the whole
minor line unless a later patch needs its own separate historical note.

## Snapshot Scope

Each snapshot should stay short and should include:

* what the version means here
* the highest-signal current surface
* what is still intentionally narrow
* the best current reading order
* the recommended practical commands
* one short rule of thumb

It should not try to be:

* a full compatibility spec
* a full release note dump
* a replacement for implementation truth

## Checklist Scope

Each checklist should stay operational and include:

* documentation alignment
* toolchain/test validation
* the version-facing surfaces that should be reconfirmed
* one explicit version-number decision note when relevant

## Naming And Routing Rule

Whenever a new minor snapshot is added:

* add it to [README.md](README.md)
* add it to [../current-mainline-map.md](../../docs/current-mainline-map.md)
* keep the newest minor line clearly marked as the current mainline anchor

Older snapshot files remain historical anchors and should not be rewritten into
current-truth documents.

## Documentation Honesty Rule

If a future sketch and a minor snapshot disagree:

* prefer implementation truth
* narrow the snapshot if it overstates reality
* do not silently blur “planned” and “real”

## Practical Rule Of Thumb

Each minor snapshot should be something a teammate can read in a few minutes
and then know:

* how to work on this line,
* what to trust,
* and what not to overclaim yet.
