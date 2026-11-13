# Native Direct Effectful Record Returns V1

Worktree follow-up: 2026-11-13, based on `283c7c28` (`beta-0.18.8`).
This does not change the version or announce a release.

## Direct Effectful Record Returns

The preceding [eager boolean initializer proof](nuis-native-state-return-eager-exit-argument-print-v1.md)
needed a loop wrapper to retain its effectful record-return helper. Removing
that wrapper reproduced `conditional fallible return requires guarded helper lowering`
in `work`. This follow-up fixes entry discovery rather than inserting a loop.

The isolated effectful control plan now considers admitted top-level conditional
record-return functions as well as counted-loop functions. Dependency completion, exact types,
scope and bounded source validation still gate admission. Straight-line wrappers
do not become normalization roots. Standalone scalar effect/print routes retain
their existing admission boundaries. Pure discovery, speculative value selection
and backend print grants remain separate.

Only actual loop roots enter loop-return/carry normalization. Direct roots use
the existing guarded scalar/record control builder. This distinction also keeps
branch-local nested records out of the loop-carry builder; the preceding local
and eager initializer witnesses are explicit regressions. Already-normalized
dependencies retain the existing freshness guard.

## Execution And Delivery

- The [source witness](../../tools/nuisc/tests/native_application_bridge/state_return_direct_eager_pair.ns)
  has no loop: `start` and `step` directly call `work`, which returns `State`.
- Native and reference runs cover `==`, `!=`, XOR, let/const and f32/f64 State.
  A shared independent model enumerates 16 gate/value combinations and poison
  cases; no safely skipped poison case is discarded. Both State words are checked,
  including a nonzero `1.5` floating-point payload in direct execution.
- Both eager operands retain left-to-right order. Their AND/OR children retain
  laziness. Left, right and suffix faults stop later prints and State publication.
  Native probes check exact output prefixes and memory canaries; reference sessions
  retain the last accepted State on failure and close.
- Direct execution uses zero loop-work budget, not an artificial one-trip loop.
  Explicit print-site grants and the helper-entry budget remain enforced.
- The CLI witness covers fresh builds, cache hits and two source-free restorations
  for both widths and all three eager operators: 120 program launches. It removes
  source, manifest and project cache before restoring. Successful traces, complete
  State, completion status, binary/container identity and fault publication are
  checked. Faulting CLI stdout may be buffered, so exact fault traces belong to
  native probes rather than the CLI assertion.

The relevant tests are
[admission](../../tools/nuisc/src/lowering/buffer_loop_outline/iteration_effects_direct_tests.rs),
[native/reference](../../tools/nuisc/tests/native_application_bridge/state_return_eager_pair.rs)
and [delivery](../../tools/nuis/tests/native_session_workflow/state_return_direct_eager_pair.rs).
Final host validation:

| Check | Result |
| --- | --- |
| Selected compiler regression suite | 224 passed, including the existing scalar-route rejection and computed-print ordering checks |
| Native/reference suite | 15 passed, including the prior eager/local loop witnesses |
| CLI direct-entry delivery | 1 test passed, 120 program launches, fresh/cache/two source-free restores |
| Tensor unit and lineage suite | 47 passed |
| Tensor drift | 4509 clean; 4501 prior IDs retained, 8 new IDs |
| Source/document policies | Formatting, UTF-8, relative documentation links, host-path policy, line budgets and architecture checks pass |

The initial direct-entry test was red with the error above. Two overly broad
intermediate routes were rejected by regressions: sending direct branches into
loop-carry normalization and reclassifying standalone scalar effect roots. The
final implementation separates these scopes; no old assertion was weakened.
This is a focused suite, not a completed all-workspace test run.

Reproduce the execution gates from the repository root:

```sh
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --lib --locked -j1 -- iteration_effects scalar_control scalar_joins scalar_helpers conditional_values control_values logical_conditions ready_values conditional_return_computed_prints_preserve_each_call --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuisc --test native_application_bridge --locked -j1 -- state_return_direct_eager_pair state_return_eager state_return_local_exit --test-threads=1
env CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo test -p nuis --test native_session_workflow --locked -j1 -- state_return_direct_eager_pair --test-threads=1 --nocapture
```

## Predecessor Lineage

The selected tensor coordinate remains
`standard-library/ns-nova/persistent-application-session`, active/99.
All six complete eager-initializer checkpoint fields are retained with SHA-256
lineage checks. The old validation command is stored in a quoted no-op heredoc,
not executed as part of the new validation command. Existing drift IDs remain
in order; eight checks are added for this follow-up. Three older loop-plan checks
now require the explicit loop-root subset instead of all effectful roots; their
IDs and the underlying admission/return-provenance requirements are retained.

This closes the reproduced direct-entry gap, not the entire application session
roadmap. The next audit is composition through direct callers with unused fields
and computed record inputs. Nested transport, arbitrary effects and unbounded
control are not newly authorized. Evidence here is host CPU, not Linux/GPU parity.
