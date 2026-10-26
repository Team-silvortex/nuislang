# Native Unary Float Field Epoch Print V1

This bounded CPU follow-up starts from Git beta-0.17.9 at `2275b6af`.
It extends the [exact f64 argument proof](nuis-native-f64-field-epoch-print-v1.md)
without changing release versions, source intrinsics, native ABI, print policy
or backend implementations. It is worktree evidence, not a new release.

## Exact Sign Operation

The [shared recognizer](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_unary_float.rs)
accepts only the frontend's width-correct private unpack/XOR/pack structure for
one float negation. f32 uses bit 31 and f64 uses bit 63. Operand ordering, exact
mask, matching codecs and unqualified expected kind are checked, not inferred
from a superficially similar bitwise expression. Its wrapper and original
projection nodes count against the existing expression-depth limit.

The [epoch certificate](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/print_epoch.rs)
checks the original owned leaf, initialized entry, nominal layout and field
path. The [argument validator](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/temporaries.rs)
still queries the completed pure-i64 signature at the exact parameter index.
The expression is neither replaced nor rewritten into a synthetic ready value.
No record root enters general availability. Fresh child entries use real ready
values; they do not inherit unavailable ancestor grants.

Ordinary initializers and conditions, whole records, references, stale layouts,
wrong kinds, arbitrary masks/casts, word prints, nested negation and arithmetic
over unavailable fields remain outside this grant. Existing ready expressions
keep their earlier independent admission routes. Print results remain i64;
the 4,096-work/64-depth print-root limits and source-order rendering are unchanged.

## Execution Evidence

Five [compiler controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/unary_float_print_epoch_tests.rs)
cover both widths, exact/malformed private operations, cumulative depth,
root/child entry, scope and signature vetoes. Raw NIR execution checks both
induction orders, selected/skipped work and zero-trip traps without relying on
source simplification. Existing frontend tests pin once-only unary operands.

The [Nuis program](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.ns)
is an f64 template; the test driver instantiates the same source with f32 types.
Parent and child carries are negated by updates, while printed helper arguments
are independently negated before and after those updates. The final checked
helper returns the unchanged current carry before State publication.

The [independent unsigned oracle](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print_cases.rs)
separates argument XOR from carry XOR. Seventeen healthy sequences at each
width cover signed zeros, finite/subnormal values, infinities, quiet/signaling
NaNs, marker-valued data and adjacent low-bit distinctions. Test-only probes
bitcast the actual callee arguments using the existing width-specific probes;
both raw arguments and returned State words must match. Classification alone
cannot establish payload or signed-zero identity. Call counts are structural,
not marker scans. Existing schedules and probe helpers are reused by the tests,
not consulted by production lowering.

The [native/reference matrix](../../tools/nuisc/tests/native_application_bridge/unary_float_field_epoch_print.rs)
keeps normal/reversed YIR, canaries, nine failure rows per width, exact flushed
prefixes, faulted-session State retention and subsequent-event rejection.
Success uses 12 loop work, 256 helper entries and at most 32 functions. Separate
128-entry NaN controls reject atomically at both widths; producer budget rules
are unchanged.

## Delivery And Validation

The [finite CLI delivery](../../tools/nuis/tests/native_session_workflow/unary_float_field_epoch_print.rs)
checks six healthy sequences per width before and after all nine failure rows.
Fresh/cache and two independent source-free images retain binary/container,
host-object, envelope, lifecycle and packaging identities. Decimal arguments
are not claimed to preserve arbitrary NaN payloads; raw native/reference probes
own that evidence. Trapping CLI children may flush only a prefix; direct native
probes flush each print and pin the exact reached prefix.

The serial, locked validation chain passed 415 distinct targeted tests, including
nine new controls: five compiler, three native/reference and one CLI delivery
test. This comprises 235 compiler, 92 native/reference, one CLI, eight LLVM,
three packaging, 17 buffer-loop, three host-failure, 26 maintenance-script, one
absolute-path policy, two file-limit and 27 tensor tests. Repeated focused runs
are counted only once; this is not a whole-workspace acceptance claim.

Both widths passed fresh/cache and two independent source-free deliveries with
six finite healthy sequences before and after the nine failure rows. The ordinary
CLI build, delivery-test compilation, formatting, whitespace, locked offline
metadata for 29 packages, 4,371 local documentation links and 5,826 UTF-8 files
also passed their checks. The rebuilt CLI reports all 3,852 drift checks clean,
alongside coverage, manifest, milestone, hierarchy and task-card lineage checks.
All six selected fields match the source exactly, including their complete
predecessor hashes and values. Historical validation commands are preserved
losslessly in a quoted here-document consumed by a no-op, checked without running
old validation chains. These are bounded CPU results, not GPU/Linux parity,
stable ABI, performance or self-hosting evidence.

## Remaining Frontier

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
One exact sign operation is proved; a bounded chain such as
`observe(-(-record.value), divisor)` still needs a compositional typed proof
that counts every wrapper and retains the original leaf/entry/path. General
arithmetic, effects/resources/I/O/async capture, stable ABI, GPU/Linux parity and
self-hosting remain separate gates. No broad completion or performance claim is
made by this bounded CPU witness.

The separate [nested unary proof](nuis-native-nested-unary-float-field-epoch-print-v1.md)
now closes the bounded chain gate without changing this checkpoint's recorded
single-operation scope or acceptance totals.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- unary_float_print_epoch unary_float_literals:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- unary_float_field_epoch_print --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- unary_float_field_epoch_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding f64-field checkpoint fields are retained by SHA-256.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `b3a136f51a570017042da0d54b5a5345401fee3285e05e2ff342bffcc563942a` |
| `next_step` | `8388bd7456f7de39a35aa39c963ca3f55f3eda7dfb24c97e42579af56a691f47` |
| `blocker` | `185861e3d7eecd1fbcacbe7e92e91e95029823acfa3c46c6d04497a81b6a2e65` |
| `next_action` | `cb5c3017019e1263de1fa0d9f65797faf6bc429f34bbb7156a6dca108c675ccc` |
| `validation_command` | `67e12b46d702fb41d9eef5e9f04cf23bdd9480741514791298608323a35a2440` |
| `expected_artifact` | `959abf7cf05b619bb341b83363496d249193daa8a5506f67a8751247928066c5` |
