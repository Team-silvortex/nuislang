# Native State Return Join Exit Argument Print V1

Audit date: 2026-11-11. Checkout baseline: beta-0.18.7, `a270a99e`.
This worktree follow-up extends the
[return-gate witness](nuis-native-state-return-gate-exit-argument-print-v1.md).
It creates no version bump, commit, tag or release.

## Joined Results And Later Work

The [Nuis source](../../tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.ns)
binds the selected conditional-call result to a fresh typed `joined` local.
Both the called and skipped paths then compute `encoded = joined ^ AFTER_TAG`,
print a marker and that value, perform a separate checked operation, print its
completion marker, and decode the result for State publication.

The first return still decides whether the second helper runs; the original
branch still chooses its helper family. Bit-32 first-return tags and bit-33
post-join tags distinguish original input, intermediate return, joined result
and subsequent computation. Native source contrasts change the second tag to
bit 41 and the sign bit. Earlier first-tag contrasts remain covered separately.
Neither computation may substitute an original input or truncate to 32 bits.

Open/event/close inputs are now 44/46/2 words: the predecessor's 43-word input
prefix is unchanged, with one post-join fault input appended. State remains two
words. Base child count two, the 12-work ceiling and the 192-helper-entry ceiling
are unchanged.

## Production Lowering

Raw helper admission initially rejected this witness. The frontend represents
a typed if-expression as matching typed declarations in its two arms, while the
iteration helper validator previously kept both declarations arm-private.

The [iteration join validator](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration.rs)
now recognizes only complete, fresh, same-name, same-type scalar declarations
observed by the suffix. It uses the completed iteration-call catalog, exact
value types and the shared expression budgets. Missing arms or callees,
mismatched types, unready reads, constants, outer scalar writes and branch-local
prefixes remain outside this new shape. Pure helper admission is not widened.

The [scalar control builder](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control.rs)
extracts a private value-returning boundary. Calls stay behind their original
predicate; only its selected result reaches subsequent work. It restores the
owner return type after creating the boundary, including owners whose return
type differs from the joined scalar.

A native execution attempt also exposed a renaming failure when arm and suffix
locals shared names. The [hygiene pass](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_control_hygiene.rs)
now gives a validated paired declaration one identity shared by its two arms
and later reads. It still evaluates both initializers in the preceding lexical
scope and does not export unrelated branch-private bindings.

The near-limit helper admission file was split into a
[control-block validator](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_helpers/control_block.rs).
[General regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_iteration_tests.rs)
cover five exact owned scalar types, colliding branch names, different owner
return types, refusal cases and 64 joins with linear helper growth.
The five-type tests verify admission and normalized NIR; this witness's native
joined result is i64, not a new claim of native execution parity for all types.

## Effects, Faults And State

The [independent transition model](../../tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print_cases.rs)
places post-join effects after the selected helper completion or skipped path.
The raw matrix has 21 healthy sequences and 33 fault sequences per float width.
Nine added fault sequences cover post-join checks at offsets 16, 17, 48 and 49,
four earlier-check collisions, and a post-join failure after a poisoned second
helper has correctly been skipped.

A post-join check failure preserves the already printed marker and encoded
value, but emits neither its completion marker nor a new State. Earlier caller,
float-update and leaf failures retain first-failure priority and cannot execute
the post-join work. Reference rejected events and faulted close retain both
accepted State words. Native traps terminate the process, not return a
recoverable pending State.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/state_return_join_exit_argument_print.rs)
withhold each of twelve leaf and six post-join print grants separately, in both
float widths and declaration orders. Neither successful admission nor runtime
skipping grants effect authority. Previous return-gate tests remain selected.

The [CLI delivery driver](../../tools/nuis/tests/native_session_workflow/state_return_join_exit_argument_print.rs)
uses nine finite healthy and fourteen fault sequences, with healthy runs before
and after faults across fresh builds, cache hits and two independent source-free
restorations in both widths: 256 process runs. It checks complete State, output,
completion and binary/container/host-object/envelope/lifecycle/packaging identity.
Source, manifest and project cache are deleted before restoration. This is a
selected delivery matrix, not all 33 raw fault rows. Native probes own exact
trapping prefixes; CLI output may remain buffered on a trap.

## Validation

Fresh selected validation on 2026-11-11:

| Selected Suite | Result |
| --- | --- |
| Source admission, scalar joins/control and iteration effects | 96 passed |
| Join/gate native, reference, return, grant and budget controls | 16 passed |
| Fresh, cached and source-free CLI delivery | 1 passed; 256 process runs |
| Tensor selection, coverage, drift and complete predecessor lineage | 39 passed |
| Development and repository scripts | 40 passed |
| Repository line budgets and exception ratchet | 2 passed |
| Host absolute-path policy | 1 passed |

These are 195 selected test functions, including Python tests. Repeated runs,
matrix rows and drift guards are not additional test functions. Native/reference
validation completed in 79.80 seconds; CLI delivery completed in 712.05 seconds,
including intended failures. These timings record this audit, not a performance
benchmark. This is selected host validation, not full workspace or CI acceptance.

The rebuilt ordinary CLI reports 4416 passing drift checks and zero failures.
All 4399 predecessor IDs retain their order. Thirteen feature guards and four
extracted control-block guards are added; every moved protection remains checked
at its actual implementation path. All six complete predecessor fields and
hashes match source and runtime. Coverage, hierarchy and task lineage are clean.
Historical commands pass syntax checks, and their quoted no-op suffix executes
without replaying old jobs.

UTF-8 validation covers 6142 tracked files; documentation checks cover 4917 local
links. Formatting, diff whitespace and architecture checks pass. The dependency
graph remains at 29 packages, six domains and 74 normal/build edges, excluding
dev-only edges. No line-budget exception or runtime quota was increased.

## Scope And Next Gate

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
Next admit ordered branch-local prefixes before producing a joined result:
a selected arm may bind/check intermediate values before its final expression.
Prove once-only evaluation, lexical isolation, shared budgets, selected-only
effects, first-failure priority and complete State publication before widening.

This is bounded host CPU evidence, not Linux/GPU parity, unlimited call depth,
resource or I/O/async capture, stable ABI, general memory safety or self-hosting.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- state_return_join_exit iteration_value_join scalar_control scalar_joins iteration_effects --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_join_exit state_return_gate_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_join_exit --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test file_line_limit --locked -j1 -- --test-threads=1
python3 -m unittest discover -s scripts/tests
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 bash scripts/check-host-absolute-paths.sh
python3 scripts/check-text-encoding.py
bash scripts/check-doc-links.sh
python3 scripts/check-workspace-architecture.py
cargo fmt --all -- --check
git diff --check
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --locked -j1
target/debug/nuis dev-tensor --json
```

## Predecessor Lineage

All six complete return-gate checkpoint fields and their SHA-256 hashes are
retained. Historical validation commands remain quoted no-op here-document
data. Syntax-check the full command through stdin and execute only its inert
history suffix, never the historical command chains.

| Complete predecessor field | SHA-256 |
| --- | --- |
| `evidence` | `4e7fe4eea5c966d31e342ac12a509deb2c901d37ef0ec3eb98b2a52b6748a433` |
| `next_step` | `5bc4c157d5943f64a5868bd5c9423e7f3b49df959cffaa3d9836e68004109efe` |
| `blocker` | `3b48706889da626e4700ce5b63a5d55d1f6064b81935b276ed670aa8485a40e4` |
| `next_action` | `ab8b23ed5594577e6ecd2568690197c20c2fc1164fa0389b58d27d59a1f2f7fe` |
| `validation_command` | `a9cd8d04ad65b7c7d71a0e881543b38a1caa49a1145e28421c4e05cb77801bde` |
| `expected_artifact` | `fcfc330ec58400e6f2fe31f395387ab699f9502fd33f36d3f7eb814f4d742a58` |
