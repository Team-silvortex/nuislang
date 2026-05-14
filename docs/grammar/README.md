# Grammar And Frontend Notes

This folder keeps parser-facing and frontend-facing material.

It is useful when you want to understand how source text is accepted or how the
current `nuis` frontend talks about `NIR`/`YIR` boundaries, but it is not the
best first stop for current semantic truth.

## Use This Folder For

* parser grammar files
* frontend syntax notes
* current `nuis`-side IR boundary notes

## Read In This Order

* [nuis-ir.md](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuis-ir.md)
  current frontend/IR boundary notes and `data.fabric`-side source conventions
* [nuis.pest](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuis.pest)
  current parser grammar used by the frontend
* [nuislang.bnf](/Users/Shared/chroot/dev/nuislang/docs/grammar/nuislang.bnf)
  higher-level grammar sketch/reference

## Boundary

If grammar/front-end notes disagree with current checked-in verifier/tool
behavior, prefer:

* [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
* the implementation itself
