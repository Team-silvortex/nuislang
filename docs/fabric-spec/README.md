# Fabric Spec Notes

This folder currently holds historical and design-oriented notes about the
`data.fabric` / Fabric IR surface.

## Current Status

The checked-in implementation has moved forward since the older `DFIR.md`
draft. In particular:

* current `data.fabric` work is expressed through `YIR` / verifier behavior
* the active primitive-family model is documented in the current references
* some older algebra examples in `DFIR.md` no longer match today's verifier
  rules one-to-one

So this folder should be read as design background, not as the fastest source
of implementation truth.

## Read This First

If you want the current repository behavior, prefer:

* [docs/reference/yir-reference.md](../../docs/reference/yir-reference.md)
* [docs/reference/yir-langref.md](../../docs/reference/yir-langref.md)
* [docs/grammar/nuis-ir.md](../../docs/grammar/nuis-ir.md)

## Historical Draft

* [DFIR.md](DFIR.md)
  legacy Fabric IR algebra draft kept for continuity
