# Native Nested Unary Float Field Epoch Print V1

This bounded CPU worktree follow-up continues the
[single-operation proof](nuis-native-unary-float-field-epoch-print-v1.md) on Git
beta-0.17.9 at `2275b6af`. Neither worktree proof is a new release. Native ABI,
source intrinsics, backend implementations and print policy are unchanged.

## Compositional Sign Proof

The [shared recognizer](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_unary_float.rs)
now peels a nonempty chain iteratively. Every layer must be the exact
width-correct private unpack/XOR/pack operation: bit 31 for f32, bit 63 for f64,
the expected operand order and matching codecs. Each layer consumes three
expression-depth edges and the original projection consumes its own edges.
Depth overflow or an exhausted 64-depth budget rejects before further traversal.
There is no list of supported chain lengths or recursive compiler-stack growth.
The source parser retains its independent nesting limit; raw-NIR admission does
not promise that every admitted IR depth can be written as nested source text.

Even chains are not folded into ready variables. The recognizer returns the
original leaf to the [existing epoch certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs),
which retains initialized root/child entry, original nominal layout and path.
The completed pure-i64 parameter query still fixes the exact argument position.
Rendering uses the original expression once in source order; no record root
enters general availability. The existing shared 4,096-work print-root budget
still bounds total print expression work rather than resetting per argument.

Mixed-width layers, bad masks/operators/order at any depth, casts, computed or
whole-record leaves, ordinary initializers/conditions, unavailable ancestors,
stale layouts and nonowned/reference roots do not gain permission. Existing
ready-value routes and direct typed fields retain their independent admission.

## Layer And Runtime Controls

Six [compiler tests](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/nested_unary_float_print_epoch_tests.rs)
sweep all accepted lengths and nearby rejections, count cumulative caller/leaf
depth and retain overflow rejection. Mutations at every position of multiple
chain lengths exercise width, mask, operator and operand-order vetoes. They also
check root/child scope, exact parameter positions, ordinary-read/ancestor vetoes
and the shared aggregate print-work limit. Raw NIR uses both induction orders,
selected/skipped work and zero-trip traps without depending on source folding.

The existing [Nuis template](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.ns)
and [test driver](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print_cases.rs)
instantiate source chains without changing carry updates or the final checked
State publication. The independent oracle uses even/odd parity only for the
argument's unsigned sign mask; carry XOR remains separate. Tests reuse schedules
and raw probes, not production special cases. The production proof validates
each layer regardless of parity.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs)
retain the single-layer matrix and add two, three and six layers at both widths.
Each profile runs 17 independent healthy sequences, including signed zeros,
subnormals, infinities, quiet/signaling NaNs, marker-valued words and adjacent
low-bit distinctions. Actual callee arguments and State words are checked, not
just numeric classification. Normal/reversed YIR, structural call counts,
canaries and nine failure rows per profile retain exact flushed prefixes,
last accepted State, faulted-session and subsequent-event rejection behavior.

Success uses 12 loop work, 256 helper entries and at most 32 functions. Separate
128-entry NaN controls retain atomic rejection for every nested profile. Neither
producer budgets nor the protected failure path are widened to fit the tests.

## Delivery And Validation

The [finite CLI test](../../tools/nuis/tests/native_session_workflow/unary_float_field_epoch_print.rs)
runs two- and three-layer profiles at both widths. Each uses fresh/cache and two
independent source-free images, running six finite healthy sequences before
and after all nine failure rows. Binary/container, host objects, envelope,
lifecycle and packaging identities remain pinned. Decimal CLI text is not
evidence for arbitrary NaN payloads; raw native/reference probes own that proof.

The serial, locked validation chain passed 424 distinct targeted tests, including
ten new controls: six compiler, three native/reference and one nested CLI delivery
test. The total comprises 241 compiler, 95 native/reference, one CLI, eight LLVM,
three packaging, 17 buffer-loop, three host-failure, 26 maintenance-script, one
absolute-path policy, two file-limit and 27 tensor tests. Repeated focused runs
are counted only once. The single-layer CLI test belongs to its preceding
receipt and is not counted as a new run here; this is not whole-workspace acceptance.

All four nested CLI configurations passed fresh/cache and two independent
source-free images, including the repeated healthy and nine-failure matrices.
The ordinary CLI build, delivery-test compilation, formatting, whitespace,
locked offline metadata for 29 packages, 4,382 local documentation links and
5,829 UTF-8 files also passed. The rebuilt CLI reports 3,864 drift checks with
zero failures, and clean coverage, manifest, milestone, hierarchy and task-card
lineage statuses. All six selected fields exactly match source, retaining the
complete predecessor values and SHA-256 digests. Historical validation commands
remain lossless quoted here-document data consumed by a no-op, checked without
executing prior validation chains. These are bounded CPU results, not GPU/Linux
parity, stable ABI, performance or self-hosting evidence.

## Remaining Frontier

The coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99. The next
missing argument proof is same-kind binary float computation over original
fields and already-ready scalar values, such as `observe(record.value + ready,
divisor)`. A unary chain is not that permission. Binary operand order, typed
read readiness, IEEE rounding/failure behavior and explicit work/depth bounds
need their own shared proof; arithmetic must not inherit a bitwise NaN-payload
preservation promise. General calls/effects/resources/I/O/async capture, stable
ABI, GPU/Linux parity and self-hosting remain separate gates.

The separate [binary argument proof](nuis-native-binary-float-field-epoch-print-v1.md)
now follows this checkpoint; its checked division and arithmetic-read grant do
not retroactively change this sign-only receipt's scope or totals.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- nested_unary_float_print_epoch unary_float_print_epoch --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- unary_float_field_epoch_print --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- nested_unary_float_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `4ebd7469c105e33c8b48eb4d2d748736a4866070511dd274415109ef8acc987c` |
| `next_step` | `a319691f01c46af6cb93c4ae3fbf3b4f542108205ecca8f63923afc6db01ad93` |
| `blocker` | `b9e379fcb51e9f3b27bb7d73459e0853aea8e1ef197d71419d75c3d08e6a4567` |
| `next_action` | `9924f5b4320300c77158bb6b58c55fd9aa228f91c861af9c56b9702357f7d1bb` |
| `validation_command` | `72ebb5d4923dfce973d4fc0b03f849dafb290f367451e4b81a0356debb764d08` |
| `expected_artifact` | `b01d2b2c82513a1800865452caf3ee1aa25efa7f0965ca8941ea8a089b104070` |
