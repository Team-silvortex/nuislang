# Typed Selected Region Return Handoff V1

Exact owned i32/f32/f64 values can now cross the enclosing return boundary of
admitted selected effect regions. This beta-0.16 worktree follow-up extends the
[typed snapshot proof](nuis-native-typed-effect-snapshots-v1.md), not the ordinary
computed-print or general effect profile. It uses the existing lexical, purity,
original/expanded budget and bool liveness contracts.

## Admission And Exit Authority

The return-value policy is separate from print captures and selected-region
data. Owned bool/i64 return tails retain their previous capture restrictions.
New owned i32/f32/f64 result functions enter only through the selected-region
route, and may capture exact owned bool/i64/i32/f32/f64 values there. Borrowed,
optional, generic, resource and aggregate captures remain excluded. New typed
tail locals also require those exact scalar kinds; legacy local admission is
not silently tightened.

Original source scopes, declared/inferred types and reachable continuations
validate before substitution. Checked source and expanded staged plans share
the existing statement/expression/depth ledgers, before helper names or clones
are installed. A failed proof leaves the original module unchanged.

Full tails, partial tails and exit-only continuing tails use the existing pure
helper machinery. Inactive i32/f32/f64 values are seeded at their exact kinds,
not with an i64 zero. An independent bool exit/readiness value determines whether
the parent returns. A real return of positive zero, negative zero or a NaN is
still a real return. A continuation carrying a neutral value is not one.

Internal returns suppress later selected work and parent continuation. Unused
checked work on a continuing path remains evaluated. Selected division by zero
still traps; an unselected division is never executed. Helper conditions and
value-producing calls are not replayed to reconstruct exit readiness.

## Source And Native Evidence

The shared source fixture covers three exact result kinds, let/const selected
bindings, full and partial tails, exit-only tails, and full/partial tails without
internal prefix exits. Unit checks inspect exact signal fields, bool-only masks,
helper purity, idempotence and atomic lexical/type/effect/resource/budget vetoes.
Zero-valued real exits and continuation seeds have separate structural checks.

The independent oracle uses sign-extended i32 words and raw floating words:
positive/negative zero, signed subnormals, finite values and signed quiet/signaling
NaN payloads. The floating completion helper uses the preceding
[sign-negation recipe](nuis-native-float-sign-negation-v1.md); no floating-point
arithmetic is used in the bit oracle.

Ordinary and reversed node/edge/function/body order both undergo registered YIR
open/event/close execution. The fixture has nine state words, all independently
checked after each callback. Its 1296 successful input/order/mode cases cover
3888 lifecycle transitions. Selected helper calls and print order are checked
on every transition.

Native evidence uses ordinary AOT, not the pure scalar session bridge. Thirty
linked success artifacts execute the compiled source `event` function on those
1296 inputs and read its actual returned word. Volatile counters instrument the
compiled `choose`, `observe` and `finish` definitions; the test driver supplies
inputs, reads results and observes calls, never replaces source computation.
The native invocation is a direct source-function call, not a native registered
callback or native nine-word state transport certificate.

Twelve isolated native trap artifacts cover both selected divisions for each
type in both orders. They must fail with a trap signal and retain only the
source print `99` before the fault, with no tail print or fabricated result.
A test-only delegating print wrapper flushes observations for reliable partial
stdout; it neither changes production printing nor replaces the calculation.

The existing pure native scalar session bridge still rejects these effectful
callbacks. Its rejection is explicitly checked, not worked around by widening
its instruction whitelist. Bounded native callback effect transport is the
next boundary to prove before treating this as native application closure.
The later [explicit literal-print policy](nuis-native-literal-print-policy-v1.md)
adds a separately checked registered native effect route. It does not change this
historical ordinary-AOT receipt or the default pure bridge's rejection.

## Reproduction

Run from the repository root with serial builds and no incremental artifacts:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --test native_application_bridge --locked -j1 -- conditional_return_typed_handoff --test-threads=1 --quiet
```

## Acceptance And Limits

Typed selected-return acceptance: The 2026-10-07 selected acceptance passes 407
distinct tests across completed cohorts:

* 303 compiler/control/frontend/pure-catalog tests
* 67 native bridge/ordinary-AOT tests
* 29 tensor tests and two source-free workflows covering six artifact variants
* Five application-session tests and one host-path policy test

The compiler subtotal combines 295 successful tests from the initial 301-test
broad run with the subsequent passing eight-test cohort: the six repaired old
rejection tests and two new effect-boundary tests. Production code did not change
between those cohorts. A single fully green 303-test broad rerun after the
test-only alignment is not claimed. Focused/repeated executions overlap these
cohorts and add no distinct tests. All twelve newly added tests pass.

Static drift passes 2100 definitions/14333 literal patterns. All 77 changed/new
file caps, formatting/whitespace, 4760 recognized UTF-8 text files and 4721 local
Markdown links pass. The rebuilt CLI confirms 2100/2100 drift checks and clean
coverage, hierarchy and lineage; all six complete preceding floating-sign fields
pass SHA-256 identity checks. Post-receipt tensor reruns overlap this selection.

Earlier floating-sign and typed-snapshot receipts remain historical, not added
to this selection. The mainline stays `active/99`, with all six complete preceding
floating-sign tensor fields retained under named history markers.

No fresh Linux/GPU run, full-workspace acceptance, measured speedup, formal
memory-safety certification, new resource authority or completed native
application closure is claimed. Historical socket/strict-Clippy findings remain
pending. Computed typed printing and arbitrary effectful helpers remain outside
this return handoff profile.

The broader return regression exposed six older rejection tests that conflated
pure return trees with the already-existing selected-region route. Their nested
literal-print fixtures now explicitly require both pure-tail and ordinary
print-prefix preparers to reject, and the bounded selected-region proof to
accept. Rebinding, wrong scopes/types, resource kinds and budgets still veto.
Two additional tests check eight such source shapes across 64 source cases
(128 ordinary/reversed executions), plus sixteen default-source native AOT
variants with exact selected/continuing print order. This is test-contract
alignment, not new production print or helper-effect authority.
