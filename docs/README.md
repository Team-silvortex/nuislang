# Docs Index

This folder is the documentation entry point once you move past the top-level
`README`.

The docs are currently split into two broad categories:

* current reference / implementation-facing material
* longer-range design/spec material
* historical archived material

## Read This First

If you want to understand the repository as it exists today, start here:

* [reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* [repo-layout.md](/Users/Shared/chroot/dev/nuislang/docs/repo-layout.md)

Those files are the closest to “current implementation truth”.

## Grammar And Frontend Notes

Use these when you want parser/frontend context:

* [grammar/README.md](/Users/Shared/chroot/dev/nuislang/docs/grammar/README.md)

## Design / Spec Direction

These folders describe broader architecture direction and are useful, but they
should be read together with the current reference docs above:

* [fabric-spec/README.md](/Users/Shared/chroot/dev/nuislang/docs/fabric-spec/README.md)
* `glm-spec/`
* `versioning/`
* `yir-spec/`
* [historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)

Important current reading rule:

* if a broader design note and the current checked-in tool/reference behavior
  differ, prefer the current `reference/` documents plus the implementation
  itself
* `fabric-spec/DFIR.md` is historical draft material, not a current verifier
  contract

## Historical Archive

These files are kept on purpose, but they are no longer part of the shortest
path for understanding the current repository:

* [historical/README.md](/Users/Shared/chroot/dev/nuislang/docs/historical/README.md)
* [historical/nuislang-whitepaper-v0.44b.md](/Users/Shared/chroot/dev/nuislang/docs/historical/nuislang-whitepaper-v0.44b.md)
