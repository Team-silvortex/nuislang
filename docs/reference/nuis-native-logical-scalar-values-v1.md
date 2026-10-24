# Native Logical Scalar Values V1

This bounded CPU follow-up continues the [logical predicate checkpoint](nuis-native-logical-scalar-predicates-v1.md)
on the Git beta-0.17.6 base at 1feb6761, without a version bump.

The [pure prefix follow-up](nuis-native-prefixed-scalar-joins-v1.md) adds a separate
branch-local declaration proof. This receipt retains its original expression-arm
boundary and validation results as historical evidence.

## Logical Value Contract

[Scalar join analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs)
adds an independent logical-value proof. The destination must be an explicit,
fresh bool declared by both arms and observed outside the branch. At least one
value arm must be a direct logical root. Ready, non-logical computed and logical
pure conditions can compose with either or both logical bool value arms.

All three original roots share one 4,096-work/64-depth/32-logical-edge budget
before recursive inference, cloning or helper allocation. Completed pure-value
dependencies prove the condition and both arms. Logical trees hidden in call
arguments, comparisons or record fields remain excluded. Branch prefixes,
outer writes, incomplete/untyped arms and effectful callees are not admitted.
The four preceding declaration proofs are unchanged, including the preceding
logical-predicate proof's direct veto of logical value arms.

The [shared factory](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
normalizes the original bool arm roots only after the complete join proof.
Generated calls are not re-inferred against the original catalog. Each arm's
normalized return expression stays inside the selected value helper branch;
only the final predicate and already-bound captures become call arguments.
The [existing logical normalizer](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_logical.rs)
preserves each guarded LHS once and its selected RHS lazily, including nested
checked calls under a total outer logical edge. Total logical arms need no
additional gate helper. Strict scalar-write, pure/effect authority and public
ABI remain unchanged; no engine-specific or backend-specific case is added.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/logical_value_join.ns)
declares a bool from `(yes_head(yes) && yes_tail(rhs)) || false` or
`(no_head(no) || no_tail(other)) && true`, selected by a fallible pure condition.
An ordinary Nuis `encode` helper uses that bool in a record field; a later unused
field remains fallible. An effectful loop returns early through the record
call's direct field projection. No foreign-language computation replaces it.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/logical_value_join_cases.rs)
uses explicit Rust branches rather than lowering helpers. It distinguishes
ordinary wrapping addition from checked division, true/false value results,
unselected poisoned arms, selected-but-skipped RHS poison and signed raw words.
The accepted prefix retains State 64 -> 72 -> -2. Ten failure cases cover
caller arguments, the predicate, all four arm callee entries and a late field
failure after both true and false logical results. Failure never publishes a
partial State; later events are rejected and failed completion remains visible.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/logical_value_join.rs)
run normal and reversed YIR. Test-only markers 83-87 observe the actual entries
of decide/yes_head/yes_tail/no_head/no_tail without granting source effects.
Exact flushed native traces pin the chosen arm, its RHS laziness and each fault
prefix. Reference events independently count each original callee and retain
the last completed State. The compact profile remains 4 work, 64 entries and
the existing 64-function bound.

[Compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_logical_values_tests.rs)
pin lazy-helper placement, either/both value arms, all three condition forms,
total arms without extra helpers, order independence, idempotence, completed
dependency/effect revocation, hidden-tree/type/scope vetoes and atomic rejection.
Inline division RHS checks also cover skipped poison and reached failures for
both logical operators, independently of call-based checks.
The 32-edge positive and 33-edge negative share their budget across all three
roots; individually valid expression-work roots can still exceed the joint
budget. The preceding whole-pipeline negative now rejects hidden logical
comparison operands, while its direct proof veto remains separately tested.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/logical_value_join.rs)
checks fresh/cache and two independent source-free restorations. Every image
runs healthy sessions, all ten failures and another healthy session. Binary,
host-object, envelope, lifecycle, packaging and source artifact identities stay
fixed. Trapping CLI children need not flush stdout: State, trap status and any
flushed prefix are pinned there; the native probe pins the complete prefix.

## Validation

The completed local M2 CPU batch passed 140 distinct tests, including eight new
tests. This combines the confirmed pre-interruption results with resumed status
validation; repeated invocations are counted once:

- Compiler: 52 focused/adjacent checks in 12.66 seconds, including five new
  logical-value controls and the four preceding declaration contracts.
- Native/reference: 12 checks in 12.12 seconds, including two new witnesses,
  the three preceding declaration families and four full-fixture bounds controls.
- CLI delivery: two checks in 177.99 seconds. The value and preceding predicate
  families each pass fresh/cache and two independent source-free images, with
  healthy sessions before and after respectively ten and eleven failures.
- Borrowed-buffer integration: 17 checks in 40.33 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; resumed status tensor: 27 in 5.84 seconds.

The interrupted status-test session could not be recovered, so all 27 status
tests were rerun rather than treating an unconfirmed result as success. The
ordinary CLI rebuild then passed in 9.23 seconds. Formatting, whitespace, UTF-8
across 5,668 files, 4,232 local documentation links and locked offline metadata
for 29 packages passed. Drift matches all 3,635 checks and 24,837 required
patterns. The rebuilt CLI matches all six source fields, complete predecessor
hashes and inert historical commands. Coverage, hierarchy and lineage are clean;
the next task card is ready and the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. The earlier
four-variant independent/shared matrix and 31-failure delivery, the guarded and
non-logical computed CLI delivery families, other historical delivery families,
GPU/Linux execution and physical performance were not rerun for this checkpoint.
Their receipts remain historical evidence, not new logical-value results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. The next bounded task is a fresh pure branch-local prefix before a
typed value, keeping local dependency order, scope and laziness in its helper.
Effectful predicates/arms, outer writes, resource/I/O/async captures, stable ABI,
full GPU/Linux parity and self-hosting retain separate proof gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- logical_value_join:: logical_scalar_join:: computed_scalar_join:: guarded_scalar_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- logical_value_join logical_scalar_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points. The coordinate's `validation_command` field
retains the complete focused/adjacent command chain; predecessor commands remain
inert history, not additional commands to execute.

## Predecessor Lineage

All six complete logical-predicate fields are preserved by SHA-256, with quoted
inert historical commands. Historical evidence is not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `f5043e44eeedacd07e12a02cb144f6d5c1614b8332a882fba19f3d4294ebc968` |
| `next_step` | `378cd65ca5d8580547d948ca21c415283becfe38249d620f82d7d75f5ad58ebe` |
| `blocker` | `baa5631141086185fa8cd5c11f9f67a4afee0f1bea0cb30fe8a558a291982f70` |
| `next_action` | `95c88c8277ce6955c1cf3b223c15dca8838dc5820b9aa28d19302bb40d179194` |
| `validation_command` | `9a4def18df608def222c81704f85a84fe768b49a66b1867e118399282b7eab04` |
| `expected_artifact` | `d70096cb66c7cceba3895cc8d8738fa4875bb432f31f52b1a9e76e114c35095e` |
