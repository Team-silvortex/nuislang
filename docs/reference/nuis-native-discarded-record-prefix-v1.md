# Native Discarded Record Prefix V1

This bounded CPU follow-up continues the [complete-record checkpoint](nuis-native-record-prefix-joins-v1.md)
on Git beta-0.17.6 at 1feb6761 without changing the version.

## Discarded Record Contract

The [typed-plan proof](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_nested_prefix.rs)
adds an independent discarded-record entry. It admits a local record selection
whose unused total side was erased by dead-binding pruning. The retained side
must still have a complete explicit supported record type and a fully proved
pure body. The enclosing declaration remains an observed scalar result.

The old scalar and complete-record entries keep their own authority. The new
entry requires an actual discarded record somewhere in its plan; it cannot
claim an unrelated scalar or complete-record declaration. It exports no source
identity for the erased result. Later reads of that identity, hidden child locals,
resource/reference captures, effects, outer writes and incomplete types fail
before helpers or public scope changes. This is not general partial-record typing.

The [existing constructor](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_nested_prefix.rs)
still uses the shared extractor, logical normalizer, guarded selection and
`Selection::with_prefix`. A private discarded name receives the generated call.
The retained record expression and local initializers execute in original order;
the erased path receives a total neutral record, never an exported source value.
No engine/backend case, foreign source shim, public ABI or graph limit is added.

## Neutral Layout Preflight

The [preflight](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records.rs)
walks the complete unrewritten declaration after its existing source-syntax check
and before typed-plan recursion or any neutral record construction. All record
neutral expressions share a separate 4,096-node budget across every descendant
branch. Their expression depth must stay below 64. `i32` zero costs two nodes,
including its conversion operand; nested records are counted by expanded shape,
not merely by the number of distinct type names. Unknown/reference fields,
cycles and over-budget layouts are rejected before synthesis.

This does not narrow existing flat value-layout admission. A flat layout can
remain a supported ordinary value while being too large for this new neutral
synthesis path. Original conditions, initializers and fields retain their
separate shared 4,096-work/64-depth/32-logical-edge budget, 128 descendant
statements, 32 statements per block and eight nested-selection levels. These
limits apply to the input NIR after prior simplification, not the source AST.

[Budget controls](../../tools/nuisc/src/lowering/buffer_loop_outline/scalar_joins_discarded_records_tests.rs)
pin exact 4,096/4,097-node boundaries, conversion-node accounting, depth boundaries,
cycles and shared multi-root exhaustion. A 4,096-field flat layout remains
supported globally but is rejected for neutral synthesis. [Admission controls](../../tools/nuisc/src/lowering/buffer_loop_outline/conditional_values_discarded_record_tests.rs)
start from genuinely simplified Nuis input and separately prove authority,
idempotence, function-order independence, dependency/effect revocation, atomic
rejection and erased-name/child-local non-export.

## Executable Witness

The [Nuis application](../../tools/nuisc/tests/native_application_bridge/discarded_record_prefix.ns)
composes discarded nested records with an effectful record return, direct field
projection and loop early exit. One outer arm retains the child's true path,
the other its false path. Each retained path has two fallible field computations
in opposite source orders, including a child-local const and nested record fields.
The discarded record is genuinely unused; its selected computations must still
run. A fallible condition and later unused return-record field remain observable.

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/discarded_record_prefix_cases.rs)
models wrapping State arithmetic, checked division and both field orders without
using compiler output for expectations. Healthy sessions include skipped poison,
condition inversion, non-entered loops, live State and MIN/MAX arithmetic.
Twelve failure inputs cover the condition, both field orders with one or both
divisors poisoned, the later unused return field and pre-callee argument failure.

[Native/reference tests](../../tools/nuisc/tests/native_application_bridge/discarded_record_prefix.rs)
exercise normal/reversed YIR. Test-only markers 94, 96 and 97 observe the three
original pure callees; native probes flush exact failure prefixes and guard output
with canaries. Failure retains the accepted State sequence 64 -> 72 -> -2,
rejects later events and leaves completion faulted. The profile remains 4 work,
64 entries and at most 64 functions, with no owned-aggregate heap allocation/drop
in the native bridge.

[CLI delivery](../../tools/nuis/tests/native_session_workflow/discarded_record_prefix.rs)
checks fresh/cache and two independent source-free images, with healthy sessions
before and after all twelve failures per image. Binary, host-object, envelope,
lifecycle, packaging and original artifact identities stay fixed. Trapping
children need not flush stdout: direct native probes pin full prefixes, while
CLI checks pin State, trap status and any flushed prefix.

## Validation

Bounded local M2 CPU validation passed 170 distinct tests, including nine new
tests. Repeated focused runs are counted once:

- Compiler: 74 focused/adjacent checks in 17.33 seconds, including six new
  admission and neutral-budget controls plus preceding scalar/record families.
- Native/reference: 21 checks in 33.72 seconds, including two new witnesses,
  previous join families and full-fixture bounds controls. The new witness
  exercises all twelve failures in both normal and reversed YIR order.
- CLI delivery: one new check in 137.70 seconds. Fresh/cache and two independent
  source-free images each run healthy sessions before and after all twelve failures.
- Borrowed-buffer integration: 17 checks in 40.72 seconds; host failure state:
  three; repository maintenance: 24; host absolute-path policy: one; file-line
  limits: two; status tensor: 27 in 7.24 seconds.

The ordinary CLI build was already current and passed in 0.05 seconds after
delivery finished. Formatting, whitespace, UTF-8 across 5,702 files, 4,291 local
documentation links and locked offline metadata for 29 packages passed. Drift
matches all 3,675 checks and 25,065 required patterns. All six source fields match
the CLI, complete predecessor hashes are verified and historical commands remain
inert. Coverage, hierarchy and lineage are clean; the next task card is ready and
the coordinate remains active/99.

These are targeted local CPU checks, not a full workspace run. Earlier
complete-record/nested-scalar CLI delivery families, the four-variant
independent/shared matrix and 31-failure delivery, other historical delivery
families, GPU/Linux execution and physical performance were not rerun here.
Their receipts remain historical evidence, not new discarded-record results.

## Remaining Frontier

The coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99. Next, prove a fresh observed record-valued outer declaration with
bounded pure prefixes and exact common layouts. This checkpoint keeps that
outer result scalar. Effectful prefixes, outer writes, resource/I/O/async captures,
stable ABI, full GPU/Linux parity and self-hosting remain separate gates.

## Reproduce

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- discarded_record --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- discarded_record_prefix:: record_prefix_join:: nested_prefix_join:: --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- discarded_record_prefix --test-threads=1 --nocapture
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --bin nuis --locked -j1 -- dev_tensor --test-threads=1 --quiet
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo build -p nuis --bin nuis --locked -j1
target/debug/nuis dev-tensor --json
```

These are focused entry points; the tensor retains the full adjacent chain.
Complete predecessor commands stay inert history, not commands to execute.

## Predecessor Lineage

All six complete record-prefix fields are preserved by SHA-256, with quoted inert
historical commands. Historical results are not counted as rerun tests.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `282ac801f6932fa3fe0a2a168148c2617872448c2b3a7dc2812c1a57c61b5bec` |
| `next_step` | `c800a453136cde43c1604930d6d652f31b3f7b8710f88cd2a3c4e9004ed0a36f` |
| `blocker` | `5e268fb4933d74015c2d50c4e8e23338a1306eeb54449ae553f327ad513311d8` |
| `next_action` | `6ede2e7a0c52c6e4a70b67809b872eb4e292bfee706516b196053f9b44f21cf8` |
| `validation_command` | `12f495847b77d75cae03d7b72c53ced511d7edcea463214815c01b9e105242de` |
| `expected_artifact` | `16ff8d7681bb8c59dd1a66f3b21f5d7f724bde1f32c197147d22e1575c964d54` |
