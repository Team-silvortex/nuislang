# Native Floating Literal Negation v1

This bounded frontend follow-up closes the source literal gap discovered in the
[typed effect-snapshot proof](nuis-native-typed-effect-snapshots-v1.md). It is
worktree development after `dff1bdbc` on `beta-0.16.*`, not a release or complete
heterogeneous-application acceptance.

## Contract

Unary negation first lowers and types its operand and resolves unary overloads.
Only a resulting f32/f64 literal atom has its leading minus sign toggled. The
decimal spelling and scalar kind are retained without host floating conversion.
Consequently source `-0.0` becomes an exact negative-zero atom, double negation
restores positive zero, and triple negation restores negative zero. Literal
module constants can participate after their existing resolution.

Expected-type failures are not bypassed. Integers, trait overloads, references,
optional values and resources keep their existing paths. Nonliteral variable,
arithmetic and call operands are not folded, duplicated or replayed. No new YIR
instruction, ABI, native transport, Nustar coupling or ownership/GLM exemption is
introduced.

CPU LLVM literal materialization also changes from positive-zero addition to
integer-bitcast constants. It parses each validated YIR literal at its exact
f32/f64 kind, matching the existing executor, then bitcasts its bits without
floating arithmetic. This avoids a second signed-zero loss in native output and
handles non-exact binary decimals without relying on LLVM decimal spelling.

The existing effect-region signed-atom test now begins with source `-0.0`, rather
than replacing its parsed NIR. Positive and negative zero atoms still require a
selector after both source arms validate. Typed inactive seeds stay positive
zero; typed data admission and bool-only masks are unchanged.

## Reproduction

Run from the repository root, serially:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- unary_float_literals conditional_return_typed_snapshots --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- unary_float_literals --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1
```

Four new frontend units cover 48 typed let/const expression cases, six spelling
and sign patterns, inferred f64, typed calls/returns/record fields, module-const
resolution, exact nonliteral operand structure and five invalid expected types.
Before the fix, two of these units reproduced subtraction from positive zero;
the nonliteral and rejection units already passed.

Two new native bridge units cover f32 and f64 in ordinary and reversed YIR
storage order. Each fixture transports eight fields through open/event/close:
negative zero, double-negated positive zero, triple-negated negative zero,
negative finite, double-negated finite, a negated nonliteral finite call, a
rounded negative decimal and the negative smallest subnormal of its scalar kind.
An independent Rust bit oracle checks all fields, including the sign bit, both
in YIR and in four linked native artifacts. The native test driver only invokes
existing registered callbacks and prints returned words; it does not supply the
source computation or use an interpreter to produce native results. A bounded
process runner and scratch cleanup are retained.

Three new LLVM units inspect ordinary/reversed literal materialization for zero,
finite values, rounded decimals, subnormals, infinities and parsed NaN, and reject
invalid literals and wrong arity. Their NaN check concerns constant bits only,
not source NaN negation or preservation of arbitrary NaN payloads.
The LLVM command also explicitly includes its host-Clang transport test; a host
Clang driver is required for that native check.

After the frontend repair alone, the two new native units still failed because
LLVM literal materialization added positive zero. The other 58 selected native
bridge units passed. This second defect was fixed in the shared CPU literal
emitter, not by weakening the independent bit oracle.

## Acceptance And Limits

Floating literal negation acceptance: The 2026-10-07 final selection passes 367
distinct tests:

* 171 LLVM units, including the explicitly enabled host-Clang transport test
* 99 return-control/frontend units and 60 native bridge tests
* 29 tensor units and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

All nine new units pass. Focused and repeated selections overlap these tests and
are not added to the total. The final compiler, native bridge, source-free and
application selections all ran after both production repairs. Static drift
passes 2076 definitions/14178 literal patterns. All 44 changed/new file caps,
4750 recognized UTF-8 text files, 4706 local Markdown links, formatting and
whitespace checks pass. The rebuilt tool confirms 2076/2076 drift checks and
clean coverage, hierarchy and lineage. Its complete six preceding typed-snapshot
fields pass SHA-256 checks. The 27-test post-receipt tensor rerun overlaps the
final selection rather than adding tests to this receipt.

The mainline stays `active/99`. The preceding 179-test typed snapshot receipt is
historical and is not added to this follow-up's total. The complete six preceding
tensor fields remain behind named history markers, with SHA-256 identity checks.

At this literal-only checkpoint, the following gap was retained:
General nonliteral float negation still uses positive-zero subtraction. In
particular, negating a variable or call returning positive zero is not certified
to produce negative zero. NaN sign/payload negation is not certified either.
These need a separately typed negation contract, not a blanket algebraic rewrite
or a transport exemption. Direct typed return-tail/print widening also remains
outside the effect-region profile. No fresh Linux/GPU run, full-workspace
acceptance, measured speedup or formal memory-safety certification is claimed.
Historical socket/strict-Clippy findings remain pending.

The subsequent [floating sign-negation follow-up](nuis-native-float-sign-negation-v1.md)
closes that nonliteral sign/payload gap with separately checked bit transport.
The literal checkpoint's counts and historical limits above are not rewritten
as evidence for that later contract.
