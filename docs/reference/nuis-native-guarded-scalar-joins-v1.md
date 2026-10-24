# Native Guarded Scalar Joins V1

This bounded CPU follow-up continues the [ready scalar declaration proof](nuis-native-typed-scalar-joins-v1.md)
from Git beta-0.17.5 at c3e6a14f without changing the version.

This records the preceding ready-predicate checkpoint. The
[computed predicate follow-up](nuis-native-computed-scalar-predicates-v1.md)
extends predicate evaluation without relaxing this rule's ready-gate contract.

## Guarded Expression Contract

[Scalar join analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs)
now has a separate guarded-expression proof. The predicate must still be a ready
bool. Both arms must declare the same fresh name with the same explicit scalar
type, and a continuation must observe that identity before redeclaration.
Existing outer writes and branch prefixes remain excluded.

Both expression trees share the existing 4,096-work/64-depth syntax budget,
checked before recursive type inference or cloning. The completed pure-value
catalog must prove every call and exact argument/result type. Missing or
effectful callees, unsupported syntax and logical trees fail closed. Selected
pure calls may still fail: purity is not totality or permission to speculate.
Computed predicates and logical/effectful arms remain independent gates.

The [source phase](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
reuses the existing conditional-value helper factory. Only the ready predicate
and already-bound inputs become call arguments. Arithmetic, projections and
calls stay inside the selected helper arm; an unselected arm is not evaluated.
The strict scalar-write validator is unchanged, and iteration-local declarations
remain under the loop outliner's own authority.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/guarded_scalar_join.ns)
returns early from an effectful loop through a field of an effectful record
call. Its typed conditional chooses a fallible pure quotient call or remainder.
Separate divisors poison the unselected arm. A later unused record field still
performs its division before any returned State is published.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/guarded_scalar_join_cases.rs)
tracks live State as 64 -> 18 -> -2, skipped callee work, zero-width work and
subsequent healthy work. Ordinary add/subtract/multiply follow Nuis wrapping
semantics. Six failure categories cover selected division/remainder by zero,
signed division/remainder overflow, the unused field and a call-argument check.
The first oracle draft incorrectly treated ordinary addition as checked; that
expectation was corrected against the established scalar contract, not by
changing language semantics or weakening the failure tests.
An adjacent frontend limitation remains: a direct unary `-9223372036854775808`
literal is rejected. Compiler arithmetic controls spell it as `(-9223372036854775807 - 1)`;
native word inputs still exercise the actual minimum value. This checkpoint
does not repair that literal parser boundary.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/guarded_scalar_join.rs)
check normal/reversed YIR, exact selected failure prefixes, raw state words,
output canaries and last-complete-State retention. Native fixture budgets are
4 loop-work units and 64 helper entries, within the unchanged 64-function
profile. These are this compact witness's limits, not a relaxation of the
preceding full fixture's 18/192 limits.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_guarded_tests.rs)
cover dependency removal, function-order independence, idempotence, scalar type
agreement, source scope, computed-gate/effect vetoes, combined expression-tree
budgets, depth limits, selected arithmetic and skipped poisoned arithmetic.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/guarded_scalar_join.rs)
checks fresh/cache and two independent source-free restorations. Each image runs
healthy sessions, all six failures and a further healthy session. Binary,
host-object, envelope, lifecycle and packaging identities must remain equal.
Trapping CLI children need not flush stdout. CLI checks published State, trap
status and any flushed prefix; the direct native probe flushes every print and
checks the complete selected failure prefix.

## Validation

Bounded local M2 CPU validation passed 119 distinct tests, including seven new
tests. Repeated checks are counted once:

- Compiler: 39 focused/adjacent checks passed in 10.20 seconds, including four
  new guarded-expression controls and the preceding ready-declaration controls.
- Native/reference: six checks passed in 6.64 seconds. The two new witnesses
  cover normal/reversed execution, exact flushed failure prefixes, canaries and
  last-State retention. Four adjacent controls retain full-fixture function
  bounds, recursion/depth checks and the 160-success/100-rejection entry limit.
- CLI delivery: one new check passed in 36.75 seconds. Fresh/cache and two
  independent source-free restorations each run healthy sessions, all six
  failures and another healthy session, retaining artifact identities.
- Borrowed-buffer integration: 17 checks passed in 37.45 seconds; host failure
  state: three passed; repository maintenance: 24 passed; file-line limits:
  two passed; status tensor: 27 passed.

The ordinary CLI rebuild passed in 8.89 seconds. Formatting, whitespace, UTF-8
across 5,647 files, 4,188 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,608 checks and 24,678 required
patterns. The rebuilt CLI matches all six source fields; complete preceding
hashes and inert historical commands are verified. Coverage, hierarchy and
lineage are clean, the next task card is ready and the coordinate stays active/99.

These are targeted local CPU checks. The full workspace, preceding four-variant
independent/shared matrix and 31-failure delivery, other historical delivery
families, GPU/Linux execution and physical performance were not rerun. Their
earlier receipts remain historical evidence, not new guarded-expression results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. A computed or effectful predicate needs its own once-only guarded
execution proof. Logical value arms require composition with the existing
short-circuit normalizer, not permission from this non-logical expression rule.
Branch prefixes, outer writes, resources, I/O/async captures, stable ABI,
GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- guarded_scalar_join:: --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- guarded_scalar_join --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding typed-scalar fields are retained in the status
tensor. Historical commands remain inert inside a quoted here-document.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `d3d8e49211d1095a8f46f85a8579ec2ef1dd6d26464c05080319efe096ba542b` |
| `next_step` | `6a718bd1946d084987d97bdd4ab470c88d152ab521fc967262c52ac9b2fe951c` |
| `blocker` | `548bd49f52ff1b644c3e7bbe0f1642808c660fc9da25cfb9b2ff19eb43031dc8` |
| `next_action` | `e91ff7630e6c733a59776650e3a05cb173100ce6f663436034e819b6adac3ef6` |
| `validation_command` | `904328c8626628ced1e613ee676a79a682608467bf52d5e4da0495cb3e9f2be2` |
| `expected_artifact` | `346f534190d47ed743b6c2016248095e034342fd1ed811fa4d53489a7d3c400f` |

