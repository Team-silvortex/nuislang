# Control-Flow Lowering Contract

This file is the current implementation-facing summary for the small
control-flow shapes that the checked-in `nuisc` lowering path treats as stable
today.

It is intentionally narrower than a full language design note. The goal is to
answer:

* which branch/loop/recursion shapes are already part of the current mainline
* which project examples and regression tests prove those shapes
* which nearby source forms are still intentionally rejected

## Short Rule

Today’s mainline is strongest when control flow stays in one of these families:

* branch-local value selection that collapses into `select`
* branch-local work followed by a small shared suffix
* structured `while` bodies that match counted/carry/flow/post-flow contracts
* async recursion that still reduces to a helper-lowered call/await spine

The compiler is not yet promising general “arbitrary CFG lowering”.

## Supported Today

### 1. Shared branch value plus shared suffix in sync/state paths

Current truth:

* `if` / `match` can produce a branch-local value
* a straight-line shared suffix can run after that value is merged
* the merged value lowers through `cpu.select`, then the shared suffix continues

Project anchors:

* [if_borrow_end_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/if_borrow_end_state_demo)
* [match_borrow_end_shared_suffix_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/match_borrow_end_shared_suffix_state_demo)
* [task_result_shared_suffix_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/task_result_shared_suffix_state_demo)
* [buffer_shared_suffix_state_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/buffer_shared_suffix_state_demo)

Regression anchors:

* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)
* [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)

Working rule:

* the shared prefix/suffix must stay straight-line
* today that means `let`, `const`, or expression statements that lowering can
  keep linear

### 2. Shared branch value plus shared suffix in async recursion

Current truth:

* async recursion can still carry a branch-selected value through a shared
  suffix before the recursive call
* the stable YIR shape is `select -> add -> async_call/call_i64 -> await`

Project anchor:

* [task_recursive_async_shared_suffix_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_recursive_async_shared_suffix_demo)

Regression anchor:

* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

Working rule:

* the recursive step remains explicit after the shared suffix
* this is a supported combination of branch merging and helper-lowered async
  recursion

### 3. Branch-local runtime observers inside `if` / lowered `match`

Current truth:

* branch-local runtime observation is now a supported sub-family of value
  selection
* this support is intentionally narrow and currently means observer-shaped task
  or mutex reads that still collapse into branch-local values
* today the stable checked-in observer family is:
  * `task_completed(...)`
  * `task_timed_out(...)`
  * `task_cancelled(...)`
  * `task_value(...)`
  * `mutex_value(...)`

Regression anchor:

* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)
* [hello_thread_mutex_if_lock_branch_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_thread_mutex_if_lock_branch_invalid.ns)
* [hello_thread_mutex_match_join_result_branch_invalid.ns](/Users/Shared/chroot/dev/nuislang/examples/invalid/ns/memory/hello_thread_mutex_match_join_result_branch_invalid.ns)

Working rule:

* observer-shaped branch-local reads are allowed when each branch still reduces
  to a select-compatible value path
* this is not a general promise for arbitrary branch-local runtime work

### 4. Structured async `while` with branch-local carry updates

Current truth:

* structured async `while` lowering accepts recognized post-flow loop shapes
* branch-local carry updates are accepted when the eventual loop-control test is
  still reducible to the supported carry/loop-state boolean family
* a branch-selected temporary value can feed a later carry update before the
  loop-control test, as long as that shared suffix still collapses into the
  existing carry-condition vocabulary
* a branch-selected value can now flow through a shared suffix that re-mixes the
  current loop state repeatedly, and that additive family can also be scaled by
  either a loop-invariant non-negative factor or a direct loop-state factor
  before the carry update

Project anchors:

* [task_async_while_post_flow_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_demo)
* [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo)
* [task_async_while_post_flow_compound_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_compound_demo)
* [task_async_post_flow_shared_suffix_loop_control_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo)

Regression anchors:

* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)
* [tests_loop_post_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_loop_post_flow.rs)

Working rule:

* the loop body still has to match the structured post-flow recognizer
* accepted branch work can update recognized carries
* the final `break` / `continue` condition still has to collapse into a known
  loop-state/carry test family

## Native Lowering Gate

The current control-flow contract is no longer checked only at the frontend or
YIR text level. Representative flow/post-flow shapes now also have a native
compile-and-launch smoke gate.

Current native smoke anchors:

* [flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/flow_branching_while_demo)
* [post_flow_branching_while_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/state/post_flow_branching_while_demo)
* [task_async_while_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_flow_cond_demo)
* [task_async_while_post_flow_cond_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_while_post_flow_cond_demo)
* [task_async_post_flow_shared_suffix_loop_control_demo](/Users/Shared/chroot/dev/nuislang/examples/projects/task/task_async_post_flow_shared_suffix_loop_control_demo)

Regression anchor:

* [artifact_cli.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/artifact_cli.rs)

Useful local gate:

```bash
cargo test -p nuisc --test artifact_cli cli_compile_emits_runnable_native_control_flow_binaries
```

What this gate is intended to catch:

* malformed LLVM block chains in flow/post-flow loop lowering
* missing terminators or accidental fallthrough between generated loop blocks
* unresolved YIR payload names leaking into LLVM carry formulas
* async loop steps that should recognize `let value = await step(value)` before
  the final loop-control decision

Short rule:

`a control-flow shape is stronger when it survives frontend compile, YIR lowering, LLVM lowering, and native launch smoke`

This is still not a promise of arbitrary CFG lowering or full self-hosting.
The current gate proves specific structured families that the compiler can
recognize and lower predictably.

## Not Yet Supported

### Branch-local consuming task/thread/mutex runtime primitives

Current rejection:

* branch-local consuming runtime primitives are still intentionally rejected in
  `if` / lowered `match`
* the current disallowed family includes shapes such as:
  * `join_result(...)`
  * `thread_join_result(...)`
  * `spawn(...)`
  * `thread_spawn(...)`
  * `join(...)`
  * `thread_join(...)`
  * `cancel(...)`
  * `timeout(...)`
  * `mutex_new(...)`
  * `mutex_lock(...)`
  * `mutex_unlock(...)`

Regression anchor:

* [tests_branch_helpers.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_branch_helpers.rs)

Current diagnostic contract:

* `conditional if/lowered-match lowering does not yet support branch-local consuming task/thread/mutex runtime primitives`
* `hoist those effects before the branch or reduce each branch to pure/select-compatible values`

What this means:

* branch-local runtime observation and branch-local runtime consumption are not
  the same support boundary
* observer-safe task/mutex reads are now part of the current control-flow
  mainline
* consuming task/thread/mutex operations still need to be hoisted or reduced to
  pre-branch values before lowering

### Mixed factor expressions after additive shared-suffix re-mix inside structured async `while`

Current rejection:

* if a structured async post-flow `while` first selects a branch-local value
* then runs a shared additive normalization suffix on top of that value
* and then scales that additive result by a non-additive or otherwise deeper
  mixed factor expression
* current lowering still rejects that shape

Boundary anchor:

* [bad_task_async_post_flow_shared_suffix_loop_control](/Users/Shared/chroot/dev/nuislang/examples/invalid/projects/bad_task_async_post_flow_shared_suffix_loop_control)

Regression anchor:

* [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)

Current diagnostic contract:

* `structured while lowering recognized loop state value and a loop-control if`
* `control condition is not reducible to supported loop-state/carry boolean tests`

What this means:

* branch-value merging plus shared suffix is supported in straight-line state
  paths
* the same idea is supported in async recursion
* a branch-selected temp plus repeated additive loop-state re-mix is now
  supported in structured async `while`
* scaling that additive family by a loop-invariant non-negative factor is
  supported
* scaling that additive family by a direct loop-state factor like `value` is
  also supported
* scaling that additive family by a one-step mixed factor like `value + scale`
  is also supported
* scaling that additive family by a nested additive factor like
  `((value + scale) + 1)` is also supported
* scaling that additive family by a multi-state additive factor like
  `(value + value)` is also supported
* scaling that additive family by the product of two additive factor groups like
  `((value + value) * (value + scale))` is also supported
* scaling that additive family by a factor-group product times an invariant like
  `(((value + value) * (value + scale)) * scale)` is also supported
* but the structured async `while` recognizer still rejects a deeper chain that
  multiplies an already-composed factor product again, such as
  `branch_value -> widened -> normalized = widened + value -> remixed = normalized + value -> stretched = remixed * ((((value + value) * (value + scale)) * scale) * (value + 1)) -> acc`,
  before the loop-control test

## Practical Reading Order

If you are debugging a current control-flow lowering issue, use this order:

1. read the nearest project anchor
2. run `dump-nir` to confirm the frontend/control-flow shape
3. run `dump-yir` if the project is expected to lower successfully
4. compare against the matching regression test
5. if the shape sits near the unsupported loop boundary above, expect a
   deliberate lowering rejection rather than a missing parser/frontend feature
