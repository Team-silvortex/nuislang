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
statement or expression `else if` chains, statement or expression `if let`, and
statement `while let`.
`loop { ... }` normalizes to `while true { ... }`; `else if` normalizes to
nested `if`; `if let` normalizes to a two-arm `match`; `while let` normalizes to
an unbounded loop containing a two-arm match gate whose mismatch arm breaks.
These forms therefore share the existing AST, NIR, YIR, ownership,
pattern-binding, and lowering contracts rather than creating parallel
control-flow nodes.

Syntax admission does not widen backend semantics silently. Normalized `loop`
forms now preserve mixed terminal `continue`/`break`/function-`return` trees,
and state/carry post-flow loops can use an explicit unbounded entry mode with a
real native backedge. A `while let` over a loop-invariant enum scrutinee can now
project its payload into scalar post-flow control, enter on `Some`, exhaust on
`None`, and execute through LLVM as a native binary. Compile-time false gates
also skip unreachable payload projection across inlined helper boundaries.
Rebuilding or rebinding
the matched enum on each backedge still requires the dynamic variant-state
carry contract; the compiler rejects that shape explicitly instead of hoisting
its condition incorrectly.

## Boundary

If grammar/front-end notes disagree with current checked-in verifier/tool
behavior, prefer:

* the executable parser and its regression tests
* [docs/reference/README.md](../../docs/reference/README.md)
* the implementation itself
