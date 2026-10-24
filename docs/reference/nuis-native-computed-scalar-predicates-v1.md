# Native Computed Scalar Predicates V1

This bounded CPU follow-up continues the [guarded expression proof](nuis-native-guarded-scalar-joins-v1.md)
from Git beta-0.17.5 at c3e6a14f without changing the version.

## Computed Predicate Contract

[Scalar join analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs)
adds an independent computed-predicate proof. The predicate and both arms must
be non-logical pure value expressions. All three trees share one existing
4,096-work/64-depth syntax budget, before recursive type inference or cloning.
The completed pure-value catalog proves every call, argument and result type;
the predicate must produce bool and both arms the same explicit scalar type.
Both branches still declare the same fresh name observed before redeclaration.
Outer writes, prefixes, incomplete/untyped arms, logical trees and effectful
callees remain excluded. The ready-predicate rule is not relaxed.

The [existing helper factory](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
evaluates the complete original predicate once as argument zero. Remaining
arguments are already-bound captures. The helper branches on that single bool;
arm arithmetic and calls remain under its guard. A predicate failure precedes
both value arms and prevents record/State publication. The strict scalar-write
validator is unchanged; there is no engine-specific lowering or new public ABI.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/computed_scalar_join.ns)
returns early from an effectful loop through a field of an effectful record
call. A fallible pure `decide` function computes the typed declaration predicate
from `1 / condition` and the original choice. Conditions 1/-1 preserve choice;
2 and the minimum i64 invert it. A zero condition fails before either arm.
The selected arm calls a fallible quotient or computes remainder, and a later
unused record field must still execute before publication.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/computed_scalar_join_cases.rs)
reuses the preceding input tables but computes predicate choice and failure
order independently of lowering. Healthy sessions retain live-State dependency,
skip poisoned predicates/arms at zero width or outside selection, and exercise
both computed outcomes. Eight failure cases retain the six preceding semantic
faults and add predicate failure plus failure after a computed choice inversion.
Ordinary wrapping arithmetic and checked division/remainder remain distinct.
The earlier unary-minimum-literal parser limitation is not changed.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/computed_scalar_join.rs)
verify normal/reversed YIR, last-complete-State retention and blocked further
events after failure. Reference call traces count zero or one predicate call.
The native probe observes `decide` entry with marker 83, without adding a source
print or granting effect authority. Exact flushed native traces must contain
one entry marker for each reached predicate, including the failed predicate,
and none on skipped paths. This pins predicate-before-arm ordering and catches
both duplication and speculation. Probe instrumentation is test-only.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_computed_tests.rs)
pin the original predicate at argument zero, guarded quotient/remainder arms,
function-order independence, idempotence, dependency/effect revocation, scalar
comparison types and the shared three-tree budget. Native fixture limits remain
4 loop-work units, 64 helper entries and the existing 64-function profile.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/computed_scalar_join.rs)
checks fresh/cache and two independent source-free restorations. Each image runs
healthy sessions, all eight failure cases and a further healthy session, retaining
binary, host-object, envelope, lifecycle and packaging identities. Trapping CLI
children need not flush stdout: published State, trap status and any flushed
prefix are checked there; the direct native probe proves the complete prefix.

## Validation

Bounded local M2 CPU validation passed 126 distinct tests, including seven new
tests. Repeated checks are counted once:

- Compiler: 43 focused/adjacent checks passed in 13.85 seconds, including four
  new computed-predicate controls and both preceding scalar-join contracts.
- Native/reference: eight checks passed in 7.96 seconds, including the two new
  predicate witnesses, both preceding guarded-expression witnesses and four
  full-fixture function-bound, recursion/depth and entry-budget controls.
- CLI delivery: the computed and preceding guarded witnesses passed in 90.21
  seconds. Both exercise fresh/cache and two independent source-free images,
  each with healthy sessions before/after respectively eight and six failures.
- Borrowed-buffer integration: 17 checks passed in 37.58 seconds; host failure
  state: three passed; repository maintenance: 24 passed; file-line limits:
  two passed; status tensor: 27 passed.

The ordinary CLI rebuild passed in 7.57 seconds. Formatting, whitespace, UTF-8
across 5,654 files, 4,202 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,617 checks and 24,728 required
patterns. The rebuilt CLI matches all six source fields, complete predecessor
hashes and inert historical commands. Coverage, hierarchy and lineage are clean;
the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks. The full workspace, earlier four-variant
independent/shared matrix and 31-failure delivery, other historical delivery
families, GPU/Linux execution and physical performance were not rerun. Earlier
receipts remain historical evidence, not new computed-predicate results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Logical predicate/value trees need composition with the shared
short-circuit normalizer so skipped RHS checks and calls stay guarded. Effectful
predicates, branch prefixes, outer writes, resources, I/O/async captures, stable
ABI, GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- computed_scalar_join:: guarded_scalar_join:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- computed_scalar_join guarded_scalar_join --test-threads=1 --nocapture
```

## Predecessor Lineage

The status tensor preserves all six complete preceding guarded-expression
fields with these SHA-256 identities. Historical validation commands remain
inert inside a quoted here-document.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `8ef39939b023949fd7fd94300ec5264f3dcf6f8ee8a8a0568f79caf74406b579` |
| `next_step` | `95138e425eddf55dfb51606fb0cb764e1b3d9f3648819fee3fed7abba00033b5` |
| `blocker` | `d1352dd39e8a136e632ce6dbee546ccbbbdef1ffc4df0619e1b95ef505dcf850` |
| `next_action` | `f50c5388ab50555b108b0c7cea80b897768dd84a4a806705178d18be41091b94` |
| `validation_command` | `45d9d67ec522c913ffe0845557f0f6c9b0626600ffa8149629246f68923075a6` |
| `expected_artifact` | `c748501f2e137ecd4acb404d95b98e2bf7cf1819da5ab548cf8963756554a021` |

