# Native Computed Logical Gates

The original `active/92` bounded CPU/source checkpoint extends the
[single-edge outer-entry proof](nuis-native-scalar-loop-snapshots-v1.md#single-edge-logical-outer-entries)
and shared logical value-root route. At that checkpoint it did not extend the
[internal return-arm grammar](nuis-native-scalar-loop-snapshots-v1.md#guarded-logical-roots-in-return-arms).
The original admission/evaluation/test sections below describe that checkpoint;
the later return-arm and nested-tree sections supersede only matching exclusions.

## Admission

One `lhs && rhs` or `lhs || rhs` root may now use a bounded pure, nonlogical,
exact owned-bool computation on the left. Calls, checked field producers,
division/comparisons and derived total comparisons use the existing pure helper
catalog and exact typed layouts. The RHS remains bounded, pure, nonlogical and
exact owned bool. Nested logical operands and effectful/unknown calls remain out.

`computed_logical_root` shares the existing iterative work/depth walker, using a
separate computed-gate mode. The entire root must fit 4096 nodes and depth below
64 before recursive inference or cloning. Both operands are queued in nonlogical
mode; enabling a computed root cannot grant nested logical authority. Existing
`expression_roots` still selects atom-only gates, and ordinary selection prefixes
remain nonlogical. The original internal return conditions, fresh return-arm
bindings and return-expression eligibility retained their atom-only contracts;
the independently validated computed return-arm extension is described below.
Computed if gates are not newly admitted inside source loops; the shared condition
visitor retains its existing atom-gate loop route. Paired loop probes check both
unchanged computed-gate rejection and preserved atom-gate admission.

## Evaluation

Shared value-root lowering passes the original left expression as the first
guarded helper argument. It is evaluated exactly once at the original source
position before the RHS decision. A left constructor/division check is mandatory,
even if a total Boolean result would otherwise appear to decide the expression.
The complete selected RHS remains inside its chosen helper arm; unselected calls,
unused constructor fields and checked math stay skipped. A total RHS needs no
extra guard, but cannot erase or defer the left computation. Current binding
versions are captured before a destination is rebound.

Outer return entries continue to save the original expression in an owned-bool
value root at its parent statement site. Return helpers and exit readiness consume
only that completed bool. Parent-local record operands stay at that site and do
not become new return-helper captures. Complete returns, replayable partial exits,
private stored exit signals and bounded intermediate suffixes keep their existing
proofs. Real false/zero exits skip the original parent continuation; earlier exits
skip both operands. Parent effects and selected local checks retain source order.

## Tests

Eight new compiler tests cover 1040 runtime source cases and 2080 ordinary/reversed
reference executions: 256 binding/return-root cases, 720 entry/arm combinations and
64 parent-record, rebinding, total-RHS and real-zero cases. Structural probes check
the original first argument, complete selected RHS, exact scalar helper captures,
parent-only record transport and idempotence with/without full control roots.
Exact 4096/4097-node and depth-63/64 probes preserve the separate atom-only mode.
Effect/type/unknown-call, nested-root and loop vetoes remain tested. The previously
vetoed computed internal-arm fixture is retained as a positive follow-up check.
Two default-source AOT tests cover eighteen binary variants through the existing
deadline, kill/reap, reached-trap and temporary cleanup harness.

Three older suites retain their now-admitted computed gate fixtures as positive
checks: the computed outer-call gate, derived outer comparison and derived bool
value roots. Other vetoes are not removed. An initial nested-root rejection fixture
also contained an independent valid return root; value-root rejection is now
tested in isolation instead of forbidding unrelated legal statement lowering.

Run sequentially from the repository root:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_computed_gates --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_entry_predicates_retain conditional_return_logical_entries_retain conditional_root_values_keep_kind --test-threads=1 --quiet
```

## Original Acceptance

Final selected acceptance passes 2046 distinct tests on the loop-restricted code:
1950 compiler units, the two new native unit tests, 58 native bridge tests,
28 tensor checks, two source-free native-session workflows, five application-session
tests and one host-absolute-path test. The workflows cover six artifact variants;
the two new native tests cover eighteen default-source AOT variants. Repeated runs,
runtime source cases and artifact/binary variants are not extra distinct tests.

All eight focused computed-gate tests pass, including the isolated veto and paired
loop probes. A boundary audit found that the shared visitor also reaches loop
bodies; computed gates now stay excluded there while the old atom-gate path remains
admitted. The initial wider run was deliberately stopped for this correction and
is not part of the receipt; the 1950-unit/58-bridge run is the completed rerun on
the restricted implementation. The three retained legacy fixture suites pass too.

The rebuilt CLI reports `active/92`, 1916/1916 live drift checks and clean coverage,
hierarchy and task-card lineage. Static checks pass 1914 definitions/12941 required
patterns, 4704 UTF-8 text files, 3053 relative documentation links and all 114
changed/new file caps; formatting and diff-whitespace checks pass. Earlier blocked
Unix-socket acceptance and strict-Clippy findings remain pending, not cleared by
this selected receipt.

The wider selected commands are:

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --test native_application_bridge --locked -j1 -- frontend lowering optimize:: typed_sparse_ typed_nested_preheader_snapshots typed_materialized_ gain_guarded_lowering gain_shared_keep_lowering --test-threads=1 --skip ::native:: --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --test native_session_workflow --locked -j1 -- dev_tensor closure_then_tensor native_stored_projections native_materialized_record_arguments --test-threads=1 --quiet
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test ns_nova_application_session --test examples_mainline_compile --locked -j1 -- compiled_nuis_ compiled_window_selection compiled_registration registration_preserves_uncalled_helpers host_absolute_paths --test-threads=1 --quiet
```

The wider compiler selector skips older `::native::` unit tests; only the two new
native tests were rerun separately for this checkpoint. No full-workspace or fresh
Linux/GPU execution is implied.

## Computed Return-Arm Gates

The `active/93` follow-up separately admits a single computed `&&`/`||` root in
complete or partial pure return trees, fresh bool `let`/inferred `let`/`const`
bindings, direct return values and bounded continuation/suffix bindings. Both
operands must remain pure, nonlogical and exact owned bool. This does not admit
nested logical operands, effects, loops or new parent capture transport.

`computed_expression_roots` reuses the iterative walker with one shared 4096-node
budget and depth below 64 across every return-arm expression. The original tree
and borrowed expanded suffix plan each retain the 32-statement bound; expanded
tails cannot purchase extra budget by checking roots separately. Preflight still
precedes recursive type inference, expression cloning and materialization.
Generic `expression_roots` retains its atom-only mode, while ordinary selection
prefixes remain nonlogical. Exact types and fresh lexical scopes remain separate
requirements, not properties inferred from structural admission.

Shared value lowering keeps the complete original left computation at its selected
statement site and guards the complete RHS. Non-total partial exit decisions use
the existing private `{ exited: bool, value: bool/i64 }` snapshot, never replaying
calls or checked math. Independently replayable parent bool decisions keep their
existing route. Real false/zero returns stop parent tails. Helper-local records
and earlier unused constructor/division checks remain inside the selected helper;
parent records cannot become new captures. Existing effect order, return authority,
name hygiene, helper eligibility and native/public limits remain unchanged.

Nine new compiler tests cover 1120 runtime source cases and 2240 ordinary/reversed
reference executions: 864 condition/binding/return cases, 192 continuation/inferred
and total-comparison cases, 48 local-record/prefix cases and 16 actual-zero exits.
Structure probes check signals, scalar captures, preserved parent statements and
idempotence. Original/expanded statement limits, exact 4096/4097 nodes and
depth-63/64 boundaries are paired with generic-mode, scope, effect, wrong-kind,
unknown-call, loop and parent-record capture rejection. Two default-source AOT
tests cover nineteen binary variants under the existing timeout/trap/cleanup
harness. Three older suites keep their newly admitted fixtures as positive tests.

The focused nine-test run passes. Its first run had a missing module terminator
in the test generator; later fixture corrections gave an unused prefix check a
fresh name, limited a signal assertion to non-total decisions and moved unknown
calls/wrong kinds into NIR rejection probes. These corrections did not weaken
production scope, capture or readiness contracts. The completed wider receipt
follows; unsuccessful or repeated runs are not additional acceptance.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_return_computed_arms --test-threads=1 --quiet
```

## Return-Arm Acceptance

Final selected acceptance passes 2055 distinct tests: 1957 compiler units, four
independently selected native unit tests (the two new return-arm tests and the two
previous computed-gate tests), 58 native bridge tests, 28 tensor units, two
source-free workflows, five application-session tests and one host-absolute-path
test. The two focused suites pass all nine new and eight previous tests; their
37 AOT variants, runtime cases, reference permutations and repeated runs are not
extra distinct tests. The workflows retain six artifact variants. The three
previous fixture suites also pass in the completed wider compiler rerun.

The rebuilt CLI reports `active/93`, 1924/1924 live drift checks and clean coverage,
hierarchy and task-card lineage, while retaining the prior evidence. Static checks
pass 1922 definitions/13019 required patterns, 4707 UTF-8 text files, 3055 relative
documentation links and all 117 changed/new file caps; formatting and
diff-whitespace checks pass. Post-receipt tensor reruns are not extra acceptance.
The wider command selectors above remain applicable; older unrelated native unit
tests, earlier blocked Unix-socket acceptance and strict-Clippy findings are not
certified by this receipt. No full-workspace, fresh Linux/GPU, benchmark or formal
safety acceptance is claimed.

## Nested Logical Trees

The `active/94` follow-up admits bounded trees of direct `&&`/`||` children at
the already proved source bool roots, outer return entries and internal return
conditions/bindings/returns. Ordinary leaves must still be pure and exact typed
owned bool computations. This is not permission for
logical gates hidden inside ordinary leaves: call arguments, comparisons, constructors and casts keep their
nonlogical admission. Effectful prefixes and new capture transports remain out.

The iterative walker checks the complete original root or return tree with
one shared 32-logical-edge budget, 4096 nodes and depth below 64. Original arms and
expanded suffix plans each keep their separate 32-statement bound and share the
edge budget across every root/copy. This bounds generated helper fanout before
recursive inference, cloning or materialization, without changing native limits.
Generic atom-only `expression_roots` and ordinary nonlogical prefixes keep their
old permissions. Source-loop if gates explicitly use the former preflight before
the shared visitor: old single atom gates stay admitted, nested gates stay out.

`conditional_values_logical.rs` verifies exact type/purity for the entire original
tree before changing helper names or functions. It then recursively lowers only
direct logical children. Each edge with a call-backed or checked RHS contributes
one helper; its lowered LHS is argument zero, evaluated once, and its complete
lowered RHS stays in the selected arm. Captures come from the original RHS and
existing scope. Generated helper calls are not re-inferred through the original
catalog. A total outer RHS still preserves guards needed in its nested LHS.
There is no flattening, distributing arms, condition replay or special AOT path.
Partial exits retain the independent stored-signal proof and real false/zero exits
still stop the original parent continuation.

Eight new tests cover 672 runtime source cases and 1344 ordinary/reversed
executions: 640 nested-left/right, total-outer-RHS, binding and partial-continuation
cases plus 32 balanced/repeated-call, outer-entry and real-zero cases. A separate
default-source native test covers fifteen default-source AOT variants, including
skipped checks, mandatory left/middle checks, partial signals and early exits.
Structure/veto tests check linear helper counts, scalar capture scopes, unchanged
rejection/idempotence, loop-route separation, exact 32/33-edge, 4096/4097-node and
depth-63/64 limits, shared multi-root budgets and 32/36-edge expanded suffixes.
Eight older suites retain nested fixtures as positive checks rather than deleting
them; all unrelated type/effect/scope/loop/capture vetoes remain.

The first focused build required the test-only `StructLiteral` fixture to include
its empty generic argument list. The completed eight-test rerun passes; this did
not change production admission. The completed wider receipt follows.

```sh
CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- conditional_logical_trees --test-threads=1 --quiet
```

## Nested-Tree Acceptance

Final selected acceptance passes 2063 distinct tests: 1964 compiler units, five
separately selected native units (one new nested-tree test and four previous
computed-gate/return-arm tests), 58 native bridge tests, 28 tensor units, two
source-free workflows, five application-session tests and one host-absolute-path
test. The workflows retain six artifact variants. All eight new and seventeen
previous focused tests pass; runtime cases, reference permutations, fifteen new
and thirty-seven previous AOT variants and repeated runs are not extra tests.
The wider selectors under Original Acceptance remain applicable.

The rebuilt CLI reports `active/94`, 1932/1932 live drift checks and clean coverage,
hierarchy and task-card lineage. Static checks pass 1930 definitions/13082 required
patterns, 4712 UTF-8 text files, 3058 relative documentation links and all 122
changed/new file caps; formatting and diff-whitespace checks pass. Prior evidence
and receipts are retained; post-receipt tensor reruns are not extra acceptance.
The earlier three blocked Unix-socket tests and eleven strict-Clippy findings
remain pending and uncounted. This is selected local CPU/source acceptance, not
full-workspace, fresh Linux/GPU, benchmark or formal-safety acceptance.

## Boundaries

The original checkpoint recorded `active/92`.
The return-arm checkpoint recorded `active/93`; the nested-tree checkpoint's
current coordinate is `active/94`. Subsequent work is `active/95` under the
[leading print-prefix proof](nuis-native-return-print-prefixes-v1.md), followed by
`active/96` under the [computed print-argument proof](nuis-native-computed-return-prints-v1.md).
The [interleaved atom-alias proof](nuis-native-return-print-aliases-v1.md) follows at
`active/97`. None is completed application closure. Logical gates hidden inside ordinary leaves,
effectful prefixes, loops, changing capture transports and borrowed/resource/aggregate parent
return-helper captures remain outside this proof. There are no public ABI, native
limit or ownership/GLM exemptions. This is selected local CPU/source evidence,
not full-workspace, fresh Linux/GPU, performance or formal-safety acceptance.
