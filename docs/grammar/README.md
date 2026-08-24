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

* [parser.rs](../../tools/nuisc/src/frontend/parser.rs) and its sibling
  `parser_*.rs` modules
  executable source-language truth used by `nuisc`
* [nuis-ir.md](nuis-ir.md)
  current frontend/IR boundary notes and `data.fabric`-side source conventions
* [nuislang.bnf](nuislang.bnf)
  higher-level grammar sketch/reference

## Control-Flow Surface

The executable frontend accepts `while`, unbounded `loop`, `break`, `continue`,
statement or expression `else if` chains, and statement or expression `if let`.
`loop { ... }` normalizes to `while true { ... }`; `else if` normalizes to
nested `if`; `if let` normalizes to a two-arm `match`. These forms therefore
share the existing AST, NIR, YIR, ownership, pattern-binding, and lowering
contracts rather than creating parallel control-flow nodes.

Syntax admission does not widen backend semantics silently. A normalized
`loop` supports the same proven lowering shapes as `while true`; arbitrary
mixed `continue`/`break`/`return` trees remain a separate loop-lowering closure
task.

## Boundary

If grammar/front-end notes disagree with current checked-in verifier/tool
behavior, prefer:

* the executable parser and its regression tests
* [docs/reference/README.md](../../docs/reference/README.md)
* the implementation itself
