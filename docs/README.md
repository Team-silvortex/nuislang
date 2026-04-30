# Docs Index

This folder is the documentation entry point once you move past the top-level
`README`.

The docs are currently split into two broad categories:

* current reference / implementation-facing material
* longer-range design/spec material

## Read This First

If you want to understand the repository as it exists today, start here:

* [reference/yir-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-reference.md)
* [reference/yir-langref.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-langref.md)
* [reference/yir-tools-reference.md](/Users/Shared/chroot/dev/nuislang/docs/reference/yir-tools-reference.md)
* [fabric-spec/DFIR.md](/Users/Shared/chroot/dev/nuislang/docs/fabric-spec/DFIR.md)

Those files are the closest to “current implementation truth”.

## Grammar And Frontend Notes

Use these when you want parser/frontend context:

* [grammar/nuislang.bnf](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuislang.bnf)
* [grammar/nuis.pest](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuis.pest)
* [grammar/nuis-ir.md](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuis-ir.md)

## Design / Spec Direction

These folders describe broader architecture direction and are useful, but they
should be read together with the current reference docs above:

* `fabric-spec/`
* `glm-spec/`
* `versioning/`
* `yir-spec/`

Important current reading rule:

* if a broader design note and the current checked-in tool/reference behavior
  differ, prefer the current `reference/` documents plus the implementation
  itself
