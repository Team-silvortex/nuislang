# Native Local Scalar Argument Print V1

Audit date: 2026-10-28. The committed baseline is Git beta-0.18.4,
`ceaa2caa`. This follows the [restricted i64 XOR proof](nuis-native-i64-xor-argument-print-v1.md)
without rewriting its historical baseline or executed totals. No release is
created by this follow-up.

## Existing Support, Not A New Grant

The source-local scalar audit confirms that ordinary ready locals already
compose signed i32 widening, i64 arithmetic/XOR and exact scalar helper calls.
No production admission rule, opcode, ABI, runtime protocol or backend-specific
permission is added. The only existing compiler-module change registers tests.

The [Nuis fixture](../../tools/nuisc/tests/native_application_bridge/local_scalar_argument_print.ns)
uses three real binding sites: a parameter-derived word before the record update,
a word derived from the updated record's i32 field, and a branch-local wrapping
arithmetic result. The words are printed and consumed by subsequent computations,
including a comparison in a float-helper argument. Both f32 and f64 State words
remain observable across open/event/close callbacks.

Ready scalar values and print-only original-field reads are different authorities.
A completed preceding print does not make an unupdated record ordinarily ready.
Factoring that field's widening/XOR/helper expression into an earlier ordinary
initializer is still rejected by this bounded iteration-helper profile. This is
an identified permission boundary, not proof that all local declarations are
missing from the language or other execution routes.

## Source Boundaries

Four [profile controls](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/local_scalar_argument_print_tests.rs)
check inferred/explicit i64 bindings, both float widths and normal/reversed
function declarations. They retain pre-update initializer vetoes, including after
a print has borrowed its read lens. Wrong declared kinds remain rejected.
Forward reads and branch-local escapes are separately pinned as frontend unknown
values, rather than misreported as lowering failures.

A ready-child control accepts the original-field print in a child iteration,
but rejects moving the same field into an ordinary initializer before its child
update. Root/child epoch permissions do not become ambient local readiness.
The original comparison-only, effect, call, capture and budget rules are unchanged.

## Actual Native And Reference Execution

The [independent oracle](../../tools/nuisc/tests/native_application_bridge/local_scalar_argument_print_cases.rs)
reconstructs XOR bit by bit and uses signed modulo arithmetic, rather than the
backend operations under test. Fourteen raw sequences per float width include
signed zero, a subnormal, infinity, a quiet NaN, high-bit masks and MIN/MAX branch
wrapping. Ten sequences per width use finite decimal CLI transport. Event input
contains a decoy initial float; actual execution must retain the preceding State.

Eight fault rows per width distinguish eager parameter-local initialization,
ready-field local initialization, reached lazy operands and tail completion.
A later false lazy gate cannot suppress a preceding local initializer's fault.
Zero trips skip body-local poisons, but not the reached tail. The dedicated native
probe observes real checked64 entry arguments and flushes every printed word;
it neither replaces the computation nor substitutes a reference result.

The [native/reference tests](../../tools/nuisc/tests/native_application_bridge/local_scalar_argument_print.rs)
check normal/reversed YIR, exact native fault prefixes, output canaries,
full State words, once-only helper counts, last accepted State and faulted close.
Separate well-typed negative sources are also rejected by the actual native
compile/emit route, not only by inspecting helper-catalog membership.
The fixture has a three-iteration work limit and 64 helper entries; these are
bounded witness settings, not increased production defaults or an exhaustion proof.

## Artifact Delivery

The [CLI matrix](../../tools/nuis/tests/native_session_workflow/local_scalar_argument_print.rs)
checks fresh and cache images, then two independent source-free restores for each
float width. Source, project manifest and project cache are removed before
restoration. Binary/container, host-object, lifecycle, envelope and packaging
identities are checked; healthy execution is repeated after the fault rows.
The CLI may buffer a trapping child's output; exact flushed fault prefixes belong
to the direct native probe, not an assumption about CLI buffering.

## Validation

289 distinct selected checks pass: 206 lowering/print/temporary controls, 12
native/reference/oracle controls, one CLI delivery matrix, 40 Python maintenance
tests, two file-budget tests, one host-path integration and 27 tensor tests.
The native set includes unchanged arithmetic/XOR neighbors and the independently
registered local witness; its four controls are rerun after the final changes.
The finite CLI matrix passes fresh/cache and two independent source-free images
for both float widths. Nine controls are new: four profile, four
native/reference/oracle and one CLI matrix. Repeated runs and matrix rows are
not additional tests, and earlier receipts are not summed.

Formatting, whitespace, UTF-8 across 5,985 tracked/unignored text files, 4,656
local documentation links and locked offline metadata pass. The actual workspace
graph remains 29 packages, six domain crates and 74 normal/build edges, with no
architecture policy violations. No file-budget exception is added; the proof
index remains within 300 lines.

The ordinary CLI is rebuilt with incremental environment overrides removed and
reports 4,147/4,147 drift checks passing. Coverage, manifest, milestone, hierarchy
and task lineage are clean; the selected active/99 task is ready. All six
source/runtime fields match, including complete predecessor values and hashes.
The retained original evidence headline and historical command syntax/inertness
contracts remain intact. Historical commands are checked through stdin, not
replayed. No commit, push, tag or version bump is performed.

This is bounded host CPU execution, not physical GPU/provider parity,
full-workspace acceptance, stable ABI, whole-engine completion or self-hosting.

## Next Boundary

The selected coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
The next implementation gate is a scoped epoch-local initializer plan: one
immutable i64 local initialized from certified pre-update fields, with original
field identity, statement position and comparison-only provenance retained.
Prove consumption, once-only evaluation and shared budgets before any admission.
Ordinary reads, assignments, escaping aliases, child inheritance and arbitrary
resources/effects remain closed; this receipt does not claim that plan exists.

## Reproduce

The subsequent [epoch-local plan](nuis-native-epoch-local-plan-v1.md) records a
private analysis-only step and its own gates. It does not rewrite this audit's
executed delivery results or claim that ordinary initializer admission is open.

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- temporaries:: print_epoch:: iteration_prints:: iteration_effects:: --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- local_scalar_argument_print --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- local_scalar_argument_print --test-threads=1 --nocapture
```

## Predecessor Lineage

All six complete preceding functional tensor fields are preserved losslessly
alongside SHA-256 identities. Historical commands remain quoted inert here-document
data; syntax/inertness checks use stdin without replaying them.

| Field | Complete Preceding SHA-256 |
| --- | --- |
| `evidence` | `84f97fe616c0ead3324a28bcb31fa7446183e2b93d80fa7d49dc8312964507fa` |
| `next_step` | `df2148170098b26bf5950c763a4843923a75ae98ffc87eb761212612f3a4b7e7` |
| `blocker` | `50ef761c1a24e7775a06543b1e273739d68da83f679c9b8a3b95ecc88a29dbce` |
| `next_action` | `adfcbff1688b38fff828dfe9e8ecdeca4e9f75db1aafb8b815dea79d34c493cc` |
| `validation_command` | `3a0534058a7b8e4a79d823aaa6e96f0f59e973e692c5896b85c63a726f2588e0` |
| `expected_artifact` | `d376881a392b624e3a48ea64cf099e04c84ba454eb892cb54c1b66deb722b602` |
