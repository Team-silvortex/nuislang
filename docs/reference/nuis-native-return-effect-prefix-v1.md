# Native Return Effect-Prefix Certificate V1

This follow-up to the
[mixed-parent-exit receipt](nuis-native-stateful-loop-return-parent-mixed-v1.md)
adds a structural certificate for the existing independent return rewrite.
It does not enable return-payload coalescing, widen source admission or change
the native ABI. Git identity remains beta-0.17.3, commit 81dc1669.

## Separate Source And Rewrite Checks

[Return normalization](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/returns.rs)
now invokes the
[effect-prefix checker](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix.rs)
both during effectful source admission and before outlining. The second use
does not substitute for the first: an oversized/unproved source must not enter
the iteration catalog and subsequently fail an assumed-success preparation.

The checker does not call the return rewriter. It compares the source and
prepared NIR trees, accepting only the existing administrative transformation:

- Fresh, distinct typed payload and i64 pending storage start at zero.
- Ordinary assignments, calls, ordered arguments, fallible checks, prints and
  local break/continue remain at the same selected source positions.
- Source branch conditions and both arms remain structurally identical apart
  from checked return administration. One returning arm cannot hide a changed
  continuation arm.
- Each loop retains its exact condition, induction step and leading/trailing
  placement. Only its effect region receives return normalization.
- A selected terminal loop return evaluates the complete original payload
  expression, then sets pending to one, then breaks. No intermediate work is
  inserted, removed, replayed or moved before pending publication.
- Every enclosing returning loop has an immediate, exact pending-equals-one
  propagation. Inner propagation breaks that loop; outer propagation returns
  the payload. There is no extra else work or skipped propagation level.

Payload truthiness never controls exit authority. False, zero and signed-zero
values remain data, not a replacement for pending. Exact expression identity
also keeps original float literal spellings and call arguments intact.

This is a deliberately narrow normalization certificate, not a general
semantic-equivalence engine, formal verifier or global memory-safety proof.
Type, scope, completed call dependencies, native reservations and effect grants
still require their independent existing validators. A structurally preserved
unknown call therefore does not become an admitted helper.

## Bounded And Fail-Closed

[The preflight](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix_bounds.rs)
walks both trees before recursive structural comparison, sharing a 65,536-node
budget and rejecting depth 64 or greater. Its supported vocabulary is the
existing scalar/typed-record expression subset, not arbitrary resources or
asynchronous operations. Budget exhaustion rejects normalization without
mutating source or falling back to unchecked admission.

[Mutation regressions](../../tools/nuisc/src/lowering/buffer_loop_outline/control_loops/return_effect_prefix_tests.rs)
cover all existing selected-return, parent-step, parent-continue, parent-break
and mixed-parent-exit fixture matrices. Negative controls alter prints and
unused checks, bound calls, ordered return arguments, pending timing, storage
freshness/types, each loop step, local exits and propagation structure.
Oversized/deep/resource/async trees fail closed. A separate control demonstrates
that structural identity alone grants neither call typing nor purity.

## Remaining Optimization Boundary

Independent payload and pending storage remain the production baseline.
This certificate explicitly rejects reused source storage and cannot authorize
coalescing it. The next optimization needs a separate bounded storage-sharing
certificate, including original read/write authority, evaluation before
publication, local exits, subsequent snapshots and unchanged native failure
prefixes/reservations. Reusing this checker as a blanket optimization grant
would be incorrect.

General resource/I/O/asynchronous capture, broader entering-State writes,
stable ABI, full heterogeneous closure and completed self-hosting remain
separate. No performance, Linux/Windows/GPU or full-workspace semantic claim
follows from this CPU normalization guard.

## Scoped Acceptance

This follow-up passes 277 distinct focused tests, eight new: 216 compiler
units, four mixed-exit native/reference/reservation/preceding-break compatibility
tests, one fresh/cache/source-free CLI test, three host fault tests,
24 repository-maintenance tests, two line-budget tests and 27 tensor tests.
The earlier 179-test subset and first eight-test probe are included in the
216-unit result, not added again. Earlier maintenance/feature receipts retain
their own historical counts.

The native and CLI routes rerun the existing
[mixed-exit execution matrices](nuis-native-stateful-loop-return-parent-mixed-v1.md),
including raw float words, actual prior-State handoff, selected failures,
normal/reversed YIR storage and full reached-bound reservations.
The source-free test reports 523.13 seconds on this M2; this is scoped regression
acceptance, not a new performance comparison or speedup claim. Native aborts
remain effect-prefix observations, not post-abort memory inspection.

All 3,427 drift checks, locked metadata for 29 packages, workspace Rust
formatting, UTF-8, documentation links and source/test/document budgets pass.
The ordinary CLI was rebuilt and matches all six current tensor fields and
complete predecessor hashes, with clean coverage and task lineage. The selected
coordinate remains `standard-library/ns-nova/persistent-application-session`,
active/99, with a ready task card for the separate storage-sharing proof.
All six complete predecessor values are retained and old commands remain inert.

| Predecessor Field | Complete SHA-256 |
| --- | --- |
| `evidence` | `6900f6979b6f6974c8f0f0e9edc33b0fe9daf7f65e8530f805f1f576f5fa24a1` |
| `next_step` | `45ac4d1629e57d06e6d102dc892a2a44d2b90844c65a263cb8a90bd193bc6f8a` |
| `blocker` | `a078ac9071242d82d3a8821807db1ba4782b2e2482ff0152ede46caaa9d72b5a` |
| `next_action` | `6f041668e172a524180d9c83896be506653f74fec7f79613d1c5d5668c4e384a` |
| `validation_command` | `11fc7dee655338f4e42e89d321279e9a37587a951e31f5b5b7d77b6cf9dfe569` |
| `expected_artifact` | `5f5528a32d211c33a03d2cbf765397ff605f2c67246545134c71f23eedfb7407` |

No Git commit, push, tag or version bump is performed. Existing uncommitted
mainline work and prior regression registrations are preserved. No new Linux,
Windows, GPU, full-workspace semantic or completed self-hosting acceptance is
claimed.

