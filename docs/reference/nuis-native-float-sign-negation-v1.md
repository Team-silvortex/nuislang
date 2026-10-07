# Native Floating Sign Negation v1

This bounded CPU follow-up closes the nonliteral gap retained by the
[literal-negation checkpoint](nuis-native-float-literal-negation-v1.md).
It is worktree development after `dff1bdbc` on `beta-0.16.*`, not a release,
new ABI or complete heterogeneous-application acceptance.

## Contract

Unary negation still lowers and types its operand and resolves overloads first.
Exact owned f32/f64 literal atoms retain their sign-spelling toggle. Nonliteral
operands use existing NIR/YIR bit transport instead of subtraction from zero:

```text
f32: unpack_f32_word(pack_f32_word(operand) XOR (1_i64 << 31))
f64: unpack_f64_word(pack_f64_word(operand) XOR i64::MIN)
```

Each operand tree occurs once. These conversions are bitcasts, not numeric
casts; f32 uses its existing zero-extended word. Negation flips only the IEEE
sign bit and preserves every other bit, including quiet/signaling NaN payloads.
Positive zero becomes negative zero, negative zero becomes positive zero and
double negation restores the original bits. Arithmetic within an operand keeps
its own existing arithmetic semantics; this is not a promise that arbitrary
NaN-producing arithmetic preserves payloads.

Only exact owned scalar types select this builtin recipe. Reference, optional
and generic-shaped types are not silently treated as floats. Integer negation,
trait dispatch and expected-type validation retain their existing paths. No new
YIR instruction, Nustar dependency, foreign runtime or ownership/GLM exemption
is added.

Pure-value admission gains only exact i64 XOR beside its existing bool XOR.
Its other operator/type families are not widened. Aggregate detection recurses
through the float-word wrappers so a negated field access still takes the
scoped aggregate iteration path. Existing input, call, effect, lexical and
budget checks remain in force.
Call-dependency discovery also traverses all four wrappers: negated calls still
release callers only after their callees validate, independent of declaration
order. Recursive call cycles and effectful helpers gain no provisional authority.

Native-session admission permits the existing checked XOR instruction. Actual
LLVM lowering still validates operand kinds and callback closure limits; this
is not an untyped whitelist escape. Typed selected-region data and positive-zero
inactive seeds remain separate from bool-only path masks and the enclosing
bool/i64 return-tail/computed-print profile.

## Reproduction

Run from the repository root, serially:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- unary_float_literals unary_float_negation conditional_return_typed_snapshots --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- unary_float_literals unary_float_negation --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- lowering::buffer_loop_outline::scalar_helpers::tests lowering::buffer_loop_outline::control_values:: --test-threads=1
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p yir-lower-llvm --lib --locked -j1 -- --include-ignored --test-threads=1
```

Two native positive cases first failed on the old path: a positive-zero input
produced positive zero in every field instead of negative zero in three fields.
The repair was made in shared lowering, not by weakening the integer bit oracle.

The fixture computes a five-field source record: original value, negation,
double negation, negated noinline helper result and negated computed value.
For each kind, fourteen raw patterns cover both zero signs, minimum subnormals,
finite values, maximum finite values, infinities and distinct signed quiet and
signaling NaNs. The arithmetic variant covers the first eight finite patterns
with multiplication by one; arbitrary NaN arithmetic is deliberately excluded.

Ordinary and reversed node/edge/function/body storage orders exercise
open/event/close in YIR and eight linked native artifacts. The 88 session cases
produce 264 transitions per execution path. Expected values come from integer
sign XOR, not host float negation. Native output is checked word-for-word,
including callback status and a volatile invocation counter inserted into the
compiled source helper. YIR also checks that helper is called exactly once per
transition. The driver only invokes registered callbacks and observes returned
words; it does not replace source computation with a driver implementation.
Bounded process execution and scratch cleanup are retained.

A native negative case rejects wrong mask, pack and unpack kinds for both
scalar types. Frontend structure checks verify variables, arithmetic, calls and
nested negation under exact types; pure-value units verify exact i64 XOR,
unchanged bool admission, mismatched kinds and aggregate classification.
The pure-helper fixture puts callers before their callees and reverses source
declarations; negated recursive cycles and effectful callees remain rejected.

A selected-region unit covers 128 source cases with negation either in a helper
or in the selected result expression, both let/const bindings, internal returns,
selected/unselected division traps and ordinary/reversed execution. Successful
paths retain exact observed typed values and once-only source helper calls.
Unselected work and real source exits are not replaced by inactive seeds.

## Acceptance And Limits

Floating sign negation acceptance: The 2026-10-07 final selection passes 389
distinct tests:

* 171 LLVM units, including the explicitly enabled host-Clang transport test
* 118 compiler/control/pure-catalog units and 63 native bridge tests
* 29 tensor units and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The compiler subtotal counts the 103-test control/frontend selection and the
17-test catalog selection with their two shared units counted once. All seven
new units pass. These focused reproduction commands are not the full acceptance
selection. All final compiler, catalog, native bridge, source-free and application
selections ran after the complete production repairs. Earlier and repeated runs
overlap these tests and are not added to the total.

Static drift passes 2087 definitions/14246 literal patterns. All 51 changed/new
file caps, 4753 recognized UTF-8 text files, 4712 local Markdown links, formatting
and whitespace checks pass. The rebuilt CLI confirms 2087/2087 drift checks and
clean coverage, hierarchy and lineage. All six complete preceding literal
checkpoint fields pass SHA-256 identity checks. The post-receipt tensor rerun
overlaps the final selection and is not counted as additional tests.

The mainline stays `active/99`. The preceding literal 367-test receipt remains
historical, with all six complete tensor fields retained under named markers and
SHA-256 checks. This follow-up does not retroactively expand earlier acceptance.

Direct typed enclosing return-tail/computed-print widening remains separate.
The subsequent [typed selected-return handoff](nuis-native-typed-effect-returns-v1.md)
closes the bounded selected-region return part with its own ordinary AOT proof;
it does not change this historical receipt or admit computed typed printing.
No fresh Linux/GPU run, full-workspace acceptance, measured speedup or formal
memory-safety certification is claimed. Historical socket/strict-Clippy findings
remain pending. Existing host-Clang transport testing requires a host driver;
no new runtime or additional implementation language is introduced.
