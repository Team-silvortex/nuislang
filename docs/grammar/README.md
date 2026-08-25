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
A loop body may also replace the matched value with a pure, loop-state-independent
variant from the same enum family. The `pattern_exit` contract executes at most
one matching iteration and reconstructs the post-loop enum through `cpu.select`,
including an initially mismatched value. Ordered integer payload fields can also
cross multiple backedges through the dynamic tag/payload carry contract. Hidden
carry indices follow source pattern order, while reconstruction matches fields
by name, so constructor field order cannot change state identity. The entry block
re-reads `pattern_carryN` every iteration, and payload consumers read the previous
backedge value. A conditional transition can rebuild the matched variant or move
to a same-enum unit variant. Structured `continue` and `break` conditions also
read those previous payloads, so control decisions retain source-level binding
values even after the next enum state has been prepared. LLVM/native coverage
proves both `Active(3) -> Active(2) -> Active(1) -> Done` and the two-field route
`{ value: 1, step: 1 } -> { value: 2, step: 2 } -> { value: 4, step: 3 } ->
{ value: 7, step: 4 } -> Done`, while preserving an initially mismatched `Done`.
Owned payload fields and non-affine payload replacement remain explicitly
rejected rather than being hoisted or treated as loop invariants.

## Boundary

If grammar/front-end notes disagree with current checked-in verifier/tool
behavior, prefer:

* the executable parser and its regression tests
* [docs/reference/README.md](../../docs/reference/README.md)
* the implementation itself
