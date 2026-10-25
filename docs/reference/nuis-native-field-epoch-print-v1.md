# Native Field Epoch Print V1

This bounded CPU follow-up continues the [child-epoch proof](nuis-native-child-epoch-print-v1.md)
on Git beta-0.17.8 at `2db5c82c`. It does not change the version, native ABI or
explicit i64-v1 print policy. It is a worktree follow-up until committed.

## Original Field Path

The [field certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_field_reads.rs)
reuses the shared bounded scalar-record shape. Entry roots must be initialized,
source-writable and updated by this loop. Child certificates also require real
ordered readiness before child availability is reset; parent print permissions
are not readiness. Unknown names, references, optional/generic types, resources,
duplicate fields and recursive or oversized nested layouts do not qualify.

The certificate retains exact nominal types and ordered layout structure. At
use, it checks the current root type and complete shape, then resolves the
original Var-rooted field path to an exact i64 leaf. Intermediate subrecords,
wrong paths, changed layouts and non-i64 leaves gain no permission. This is not
a field-index guess or a substitution with a synthetic ready value.

The [epoch owner](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
keeps scalar and record grants separate. Only scalar seeds enter the existing
copy-on-write print view's available set. Record roots never enter it. The
[expression checker](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
receives the field certificate only for one print value; ordinary assignments,
conditions, temporary initializers and child headers receive none. A whole-record
argument such as `print(read(live))` remains unavailable before update even if
`print(live.inner.total)` succeeded immediately before it.

Complete pure dependencies, exact argument types, the ordinary shared
4,096-work/64-depth print-root budget and existing source-order rendering are
unchanged. The certificate proves read authority, not a frozen value: repeated
projections observe the current source version before or after an update.
No backend-specific branch, foreign shim, new intrinsic or runtime allocation
is introduced.

## Execution Witnesses

Five [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/field_print_epoch_tests.rs)
cover exact nested paths and layout changes, non-owned/malformed roots, child
readiness, whole-record laundering and ordinary-read isolation. Raw NIR runs
both leading and trailing loops with selected or skipped checked print work,
without relying on simplification. Its lowering entry owns outlining once, so
record transport metadata is not discarded by a second manual outline pass.

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/field_epoch_print.ns)
keeps `Outer { inner: Pair { total, tag, flag } }` as parent and child loop
carries. It prints both i64 fields before and after updates. The total follows
the persisted State; the tag starts at 100 for each work invocation and advances
by different parent/child increments. This distinguishes paths and fresh entry
versions without changing the flat external session interface.

The [independent scalar oracle](../../tools/nuisc/tests/native_application_bridge/field_epoch_print_cases.rs)
models those values directly with wrapping arithmetic and checked division.
Healthy State follows 64 -> 72 -> -2. It covers zero/negative child trips,
repeated entries, skipped poison, negative deltas, MIN/MAX divisors and a full
three-by-three case. Its nine failure rows stop at selected child prints in
later entries or final checked work before State publication.

[Reference/native checks](../../tools/nuisc/tests/native_application_bridge/field_epoch_print.rs)
use normal/reversed YIR, real callee-entry markers, complete flushed native
failure prefixes, canary outputs and last-State/faulted-session checks. Bounds
remain 12 total loop work, 64 helper entries and at most 32 functions.
[CLI delivery](../../tools/nuis/tests/native_session_workflow/field_epoch_print.rs)
checks fresh/cache and two independent source-free images with healthy runs
before and after all failures. Binary/container, host-object, envelope,
lifecycle and packaging identities are retained. CLI fault checks pin any
flushed stdout rather than assuming trapping processes flush everything.

## Validation

Bounded local M2 CPU validation passed 340 distinct tests, including eight new
tests. Repeated focused runs are counted once:

- Compiler: 203 focused/adjacent tests in 61.44 seconds, including five new field
  controls, plus three existing shared shape tests in 0.07 seconds.
- Native/reference: 51 tests in 135.00 seconds, including two new field witnesses
  and the prior child/root/computed-print, record/join and literal-policy families.
- CLI delivery: one new test in 49.27 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after all nine failures.
- Producer policy: three LLVM build-policy and three packaging-option tests.
- Borrowed-buffer integration: 17 tests in 39.90 seconds; host failure state:
  three; repository maintenance: 26 in 21.546 seconds; host absolute-path policy:
  one; file-line limits: two; tensor checks: 27 in 7.24 seconds.

The final CLI-test compilation and ordinary CLI build passed without warnings,
after delivery finished. Formatting, whitespace, UTF-8 across 5,788 files,
4,294 canonical documentation links and locked offline metadata for 29 packages
passed. All 3,790 drift checks and 25,680 required patterns match.

The rebuilt CLI matches all six source fields. Complete predecessor hashes and
inert historical command syntax are verified. Coverage, hierarchy and lineage
are clean, the next task card is ready and the coordinate remains active/99.

These are targeted CPU checks, not a full workspace run or performance evidence.
Earlier CLI delivery families, the independent/shared storage matrix, physical
float/performance tests and GPU/Linux execution were not rerun. Their receipts
remain historical evidence, not new field-epoch acceptance. Use the active tensor
command for adjacent families, not commands inside its inert predecessor chain.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Next is
pre-update bool field projection as an exact bool argument inside a completed
pure-i64 print value. That requires a typed leaf certificate and independent
controls; this i64 proof does not grant it by analogy. Whole-record readiness,
reference/resource reads, general effects/I/O/async capture, outer writes,
stable ABI, GPU/Linux parity and self-hosting remain separate gates.

The subsequent [typed-bool argument proof](nuis-native-bool-field-epoch-print-v1.md)
advances that bool frontier without granting ambient bool expression readiness.
The test totals above remain this i64 checkpoint's own acceptance results.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- field_epoch_print:: child_epoch_print:: carry_epoch_print:: computed_iteration_print:: literal_print_policy:: literal_print_execution:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

The six complete child-epoch fields are preserved with their SHA-256 identities.
Historical validation commands are quoted inert data, not additional execution.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `858379f947c3f20a6b153f70b9820113e3cacf39355bcc4b82920f941de921ec` |
| `next_step` | `e1f92a35b372a432eb429fe07187ced960157427e97b19081e2a9a0ff69b1bc5` |
| `blocker` | `d294cbc23a6de50312c600e4bc4e35d09b4ae019c498c040585a247719e871c2` |
| `next_action` | `201270bbad5f196191b5ceda09ec4230fb024a240bb992ce1e858eac64d4cb4e` |
| `validation_command` | `812004648112a9787ab0144375809520471d968402dc0280f3c1759dbcb32ba3` |
| `expected_artifact` | `530e2a789c35fa19925175c892ae7c1dc27027f3b3f38c23843fd64cce068023` |
