# Native Typed Scalar Joins V1

This bounded CPU follow-up continues the
[ready-value checkpoint](nuis-native-return-ready-values-v1.md), starting from
Git beta-0.17.5 at c3e6a14f without changing the version.

This records the preceding ready-declaration checkpoint. The
[guarded expression follow-up](nuis-native-guarded-scalar-joins-v1.md) extends its
arm contract without treating pure expressions as safe to speculate.

## Source Declaration Contract

The frontend represents `let x: i64 = if choice { left } else { right };`
as two typed branch declarations. The strict helper catalog keeps branch locals
scoped to their own arms, so the following read of `x` previously lost admission.

[Scalar join analysis](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins.rs)
recognizes a narrow source declaration: both arms declare the same fresh name
and explicit scalar type, the predicate is an already available bool, and both
values are already available scalars or literals of that type. Existing outer
bindings, branch-local prefixes, calls, projections and checked arithmetic do
not qualify. The continuation must read the joined identity before a later
declaration shadows it; nested branch-local declarations do not create that use.
The observation scan has work/depth bounds of 4,096/64.

The [source normalization phase](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values.rs)
runs before return/region helper generation and reuses the existing conditional
value helper factory. Both value arms become exact typed returns. The original
strict scalar-write validator is unchanged. Raw source joins remain outside
that catalog until normalization supplies a single value declaration.
The phase is idempotent and leaves iteration-local declarations to the loop
outliner's existing authority.

## Executable Witness

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/state_loop_return_scalar_join_helpers.ns)
replaces both `record_select` calls with inline typed conditional bindings in
effectful marker/total callees. The now-unused source selector is removed.
Both fields retain their original effectful short-circuit predicates and
unconditional checked work. This uses the same independent raw-State/effect
oracle as the preceding full fixture.
The standalone `.ns` fragment is byte-identical to the earlier generated matrix
input, SHA-256 `2275da10bceff432eef1b165508e99271c6480d4092ef0d1cc4783a5261f377f`.

All four full variants retain every local/parent exit mode, both record epochs,
six lazy truth paths and 31 failure categories. The selected result must remain
distinct from unselected poisoned work and partial record-field publication.
The generated closure fits the existing 64-function profile. Fixture budgets
remain 18 loop-work units and 192 helper entries.

[Unit controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_tests.rs)
check bool/i32/i64/f32/f64 values, including integer boundaries, signed zeros,
subnormals and NaN payload words under normal/reversed YIR. They also reject
outer writes, incomplete arms, untyped arms, calls, division and redeclaration
leaks. Dependency removal must still revoke the enclosing effectful call chain;
effectful callers never gain pure-value authority.

An adjacent nested-return test had a stale rejection expectation. Its input is
unchanged by source declaration normalization. The replacement checks that fact
and executes both logical operators over skipped, successful and failing paths,
checking selected prints and checked call counts in reference/native execution.
This is separate evidence for the existing nested-return route.

## Validation

Bounded local M2 CPU validation passed 117 distinct tests, including 14 new
tests. Repeated checks are counted once:

- Compiler: 35 focused/adjacent controls and the complete four-variant
  independent/shared execution matrix. The latter passed in 388.11 seconds.
- Native/reference: seven checks passed in 233.46 seconds, including exact
  fault prefixes, last-State retention, full loop reservations and unchanged
  function/entry bounds.
- CLI delivery: one check passed in 735.03 seconds. Fresh/cache and two
  source-free images each run healthy sessions, all 31 failure categories and
  another healthy session. Binary, host-object, envelope, lifecycle and
  packaging identities remain equal across cache/restoration.
- Borrowed-buffer integration: 17 checks passed in 35.84 seconds, covering
  scalar control, branch scope, selected failures and ownership rejection.
- Host failure-state checks: three passed; repository maintenance: 24 passed;
  file-line limits: two passed; status tensor: 27 passed.

The ordinary CLI rebuild passed in 6.85 seconds. Formatting, whitespace, UTF-8
across 5,640 files, 4,174 local documentation links and locked offline metadata
for 29 packages passed. Drift validation matches all 3,599 checks and 24,632
required patterns. The rebuilt CLI matches all six source fields; complete
predecessor hashes and inert historical commands are verified. Coverage,
hierarchy and lineage are clean, the next task card is ready, and the coordinate
remains active/99.

These are targeted local CPU checks. The full workspace, other historical
delivery families, GPU/Linux execution and physical performance were not rerun.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Computed or effectful predicates and non-atomic value arms need a
separate proof that composes their existing guarded execution with a fresh
typed result. This ready-value rule does not provide that authority. Nested
multi-binding joins, resources, I/O/async captures, stable ABI, GPU/cross-platform
parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- scalar_joins::tests effect_return_storage_scalar_join --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- scalar_join_payload --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- scalar_join_payload --test-threads=1 --nocapture
```

## Predecessor Lineage

The status tensor preserves the complete preceding ready-value fields, rather
than just their summaries. Their SHA-256 identities are recorded below.
Historical validation commands remain inert inside a quoted here-document.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `393246e5fa28bfdb31564e37959abcf94fa8159a5bdc3760090d6189b0d8eaf1` |
| `next_step` | `36176d2d49d7cbb516611b12070b197314db8ead73e86d58e717ffd978e1dd0b` |
| `blocker` | `d601ea4bfe45527f886895f894995ece3151c00f153dac1aeba2c1b239ac63d6` |
| `next_action` | `217b0cd20f2f06312126115da111cf2874c908a936f9c9c5890cf2e6ecc4c0a6` |
| `validation_command` | `1ff488efafe0505ed06e83e5b8d28761c0549f2e174219162526cb1f12489d5d` |
| `expected_artifact` | `2285eb5e19b90d2fbcd66a672e9972b416d82f608cfb6608f801799aef4d15ed` |
