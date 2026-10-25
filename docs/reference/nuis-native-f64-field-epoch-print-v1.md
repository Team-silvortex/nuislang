# Native F64 Field Epoch Print V1

This bounded CPU worktree follow-up continues the [f32 argument proof](nuis-native-f32-field-epoch-print-v1.md)
on Git beta-0.17.8 at `2db5c82c`. It changes no release version, native ABI,
source intrinsic, print policy or backend-specific implementation. Print results
remain exactly i64.

## Exact Typed Grant

The [shared epoch policy](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
now admits exact unqualified f64 parameters alongside bool/i32/f32. The completed
pure-i64 signature query, original parameter index, initialized root or genuinely
ready child entry and exact nominal shape/path are unchanged. No record root
enters general availability. A value of the wrong width, an unavailable word
cast or compound argument, a non-i64 or effectful callee, an ordinary initializer
or condition, or an unavailable ancestor gains no permission.

The 4,096-work/64-depth print-root budget, source-order renderer and existing
f64 word codecs are reused. Native scalar/loop inequality keeps the repaired
IEEE behavior from the preceding checkpoint; it is not a new read grant.

Five [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/f64_field_print_epoch_tests.rs)
cover exact signature/leaf kinds, stale and qualified roots, single-print depth
and scope, compound/word/whole-record vetoes and ancestor isolation. Raw NIR
executes both induction orders, selected/skipped print work and zero trips with
exact 64-bit IEEE words, without relying on source simplification.

## Full 64-Bit Words

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/f64_field_epoch_print.ns)
uses nested f64/i64 parent and child carries. Source negation flips only the
sign bit; the pure i64 observer performs classification and a checked division.
A final pure f64 helper performs an unused checked division before returning
the unchanged value, retaining failure before State publication.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/f64_field_epoch_print_cases.rs)
models unsigned 64-bit XOR and casts only when expressing raw words as signed
i64 probe/ABI output. Seventeen independent healthy sequences cover both zero
signs, finite/subnormal values, infinities, quiet/signaling NaNs, marker-valued
data and two high-word values differing only in a low bit. The latter would
expose accidental narrowing even if a numeric classification stayed unchanged.
The nine failure rows pin the last accepted State and exact reached prefixes.

Test-only probes bitcast the actual double arguments at `observe` and `finish`
entries directly to i64, preserving all 64 bits without numerical conversion.
Both callee arguments and returned State words must match the independent
oracle. Call counts are structural loop/branch counts, never marker scans over
data. Probes observe real source work rather than replacing its computation.

[Native/reference checks](../../tools/nuisc/tests/native_application_bridge/f64_field_epoch_print.rs)
cover normal/reversed YIR, all raw-word sequences, canary outputs, complete
flushed fault prefixes and faulted-session State retention. Success uses 12
total loop work, 256 helper entries and at most 32 functions; a separate
128-entry NaN control retains protective rejection before State publication.
Those are explicit fixture bounds, not widened producer budget rules.
Nine existing [f64 transport checks](../../tools/nuisc/tests/native_application_bridge/f64_scoped_record_carries.rs)
independently retain raw codecs, wide/nested backedges, snapshots, arithmetic,
all five scalar kinds, wrong-word rejection and atomic limits.

## Delivery Boundary

The [finite CLI delivery](../../tools/nuis/tests/native_session_workflow/f64_field_epoch_print.rs)
uses six healthy sequences with exact decimal round trips, including negative
zero, negative finite values and subnormals. Fresh/cache and two independent
source-free images each run them before and after all nine failures, keeping
binary/container, host-object, envelope, lifecycle and packaging identities.
State text and any flushed trap prefix are checked exactly.

Decimal CLI arguments are not claimed to preserve custom NaN payloads. Direct
native/reference raw-word controls provide that evidence. Classification alone
does not establish signed-zero or payload identity; raw callee/State words do.

## Validation

The serial, locked validation chain passed 403 distinct targeted tests, including
nine new controls: five compiler, three native/reference and one CLI delivery
test. This comprises 226 compiler, 89 native/reference, one CLI, eight LLVM,
three packaging, 17 buffer-loop, three host-failure, 26 maintenance-script, one
absolute-path policy, two file-limit and 27 tensor tests. Repeated focused runs
are counted only once; this is not a claim that the entire workspace suite ran.

Fresh/cache delivery and two independent source-free images passed the six
finite sequences before and after all nine failure rows. The ordinary CLI build,
delivery test compilation, formatting, whitespace, locked offline metadata,
4,355 local documentation links and 5,818 UTF-8 files also passed their checks.

The rebuilt CLI reports all 3,840 drift checks clean, with coverage, manifest,
milestone, hierarchy and task-card lineage statuses clean. Its six selected
fields match the source exactly; all complete predecessor hashes and values are
retained. Historical predecessor commands remain quoted inert data, checked
without executing old validation chains. These results cover bounded CPU
execution only, not GPU/Linux parity, stable ABI or performance targets.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. Direct
typed argument fields now cover the bounded scalar kinds, but a compound
argument such as `observe(-record.value, divisor)` before update remains outside
this grant. Next is a shared typed unary-float argument proof for f32/f64 sign
negation, retaining the original leaf/entry/path, exact expected kind, raw sign
bits, source ordering and scoped depth/work budget without ambient readiness.
Whole-record readiness, references/resources, general effects/I/O/async capture,
outer writes, stable ABI, GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- f64_field_epoch_print:: f32_field_epoch_print:: i32_field_epoch_print:: bool_field_epoch_print:: typed_f64_scoped_record_carries --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- f64_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete f32-field checkpoint fields are retained by SHA-256.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `be76ac512b12312bebf2bdd81a19def19ea4db6e1b82af503ee740267cf36d8c` |
| `next_step` | `33e46e6700a8f7278bf1315ffa5693e6f0b087674e2029e905f501b80f177fab` |
| `blocker` | `47cf11335cada6f71de162004cd097b23d91dfba61b45ee0b695006003cef1f0` |
| `next_action` | `7be7f91c39f037641d9ed7457d54473af57af5e47b5a87c851ad29f9ca4a9e7c` |
| `validation_command` | `4a494ae4103e2b7acc3f58f1105ee56d779c30013d0eeee2cd7524c8ad367360` |
| `expected_artifact` | `82897a89ab477fd8c001ba51fae45afd19926e643c3ffa37aa85377e85abdee2` |
