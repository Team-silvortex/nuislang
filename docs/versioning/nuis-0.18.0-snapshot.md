# `nuis` 0.18.0 Snapshot

This file is the history anchor for the `0.18.*` line.

It follows the `0.17.*` integration push and marks the point where the
repository started treating three previously separate stories as one shared
mainline:

* control flow
* async/task + memory/address composition
* project-backed compile truth across state/shader/network routes

It is still not a “finished language” milestone.

It is a mainline-closure milestone.

## What `0.18.0` Means Here

`0.18.0` is the point where `nuis` should be read less as
“many supported islands” and more as:

* one clearer compiler workflow
* one clearer control-flow lowering spine
* one real internal address/pointer baseline
* one stronger generic-constraint validation surface
* one more believable async/task/session composition story
* one more honest set of project-backed anchors for shader/network/http-shaped
  examples

Short rule:

`0.18.0` is where ordinary source patterns should increasingly either survive into named lowering families or fail locally and honestly`

## High-Signal Current Surface

The most important current truths for `0.18.0` are:

* control flow is now a first-class mainline concern:
  - `if`, `match`, and `while` are no longer only frontend-local wins
  - checked-in state projects now defend named loop families through the real
    project pipeline
  - branch-local carry updates, flow/post-flow routes, and guarded `match`
    routes are much easier to point to concretely
* generic validation is stronger and more local:
  - alias-aware method-bound diagnostics are now test-backed across ordinary
    control-flow routes
  - lambda bodies, destructuring locals, guarded `match`, and nested
    binding environments now behave more like one user-facing diagnostic system
* async/task routes now connect back to the same mainline:
  - result/policy/fallback/batch/windowed task summaries are project-backed
  - recursive/generic async routes increasingly count as real compile truth,
    not only toy frontend probes
* the address model is now explicit enough to call real:
  - `ref Node` and `ref Buffer` are the current pointer core
  - `borrow(...)` / `borrow_end(...)` are part of the verified ownership story
  - read-vs-write authority is intentionally conservative and test-backed
* compile truth increasingly means project truth:
  - state/control-flow anchors
  - task/control-flow anchors
  - shader/helper-mediated anchors
  - network/http/session anchors
  now all contribute to the current believable mainline instead of living as
  unrelated demos
* repo-level stdlib smoke now agrees with the mainline story again:
  - `stdlib/std/net_session_recipe.ns` no longer wedges YIR cycle verification
  - helper-heavy network/session recipes survive the checked-in stdlib sweep
  - the narrower compiler release gate and the broader stdlib smoke once again
    point in the same direction

## What Is Still Intentionally Narrow

`0.18.0` should still avoid overclaiming:

* general loop/backedge lowering outside the current counted/carry/flow/post-flow
  families is still intentionally narrower
* host-boundary pointer ABI is still value-only even though internal `ref`
  semantics are real
* network/http compile ladders are ahead of full runtime portability and ahead
  of a final polished public `std` API
* generic inference is stronger and more coherent, but still not a fully
  general HM-style type system
* control-flow support is much broader, but not yet “all ordinary programs
  lower without caveats”

## Best Current Reading Order

For the `0.18.0` line, the shortest practical route is:

1. [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
2. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
3. [nuis-0.18.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-goals.md)
4. [nuis-0.18.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-snapshot.md)
5. [nuis-0.18.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-compile-workflow.md)
6. [nuis-0.18.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-mainline-regression-matrix.md)
7. [nuis-0.18.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.18.0-address-pointer-mainline.md)
8. [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
9. [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
10. [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)

## Concrete Anchor Cases

If you want the shortest “show me the real thing” route for this snapshot,
start with:

* state/control-flow proof:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)
* task/async/control-flow proof:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)
* address/pointer proof:
  [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* network/http/session proof:
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)
* generic-bound diagnostic proof:
  [tests_generic_method_bounds.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds.rs),
  [tests_generic_method_bounds_control_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_control_flow.rs),
  and
  [tests_generic_method_bounds_lambda_bindings.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generic_method_bounds_lambda_bindings.rs)

## Recommended Practical Commands

For mainline project work:

```bash
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

For fast `0.18.0` regression confidence:

```bash
cargo test -q -p nuisc tests_control_flow
cargo test -q -p nuisc tests_loop_flow
cargo test -q -p nuisc tests_loop_post_flow
cargo test -q -p nuisc generic_method_bounds
cargo test -q -p nuisc --test state_compile
cargo test -q -p nuisc --test task_compile
cargo test -q -p nuisc --test memory_compile
cargo test -q -p nuisc shader_nova_contracts
cargo test -q -p nuisc --test network_compile
```

For broader repository confidence:

```bash
cargo test -q -p nuis stdlib_source_modules -- --nocapture
bash scripts/check-0.18-release.sh
```

## Rule Of Thumb

If `0.17.*` was about making more subsystems line up, `0.18.0` is about making
the resulting mainline easier to describe as one real compiler story:
control flow, generics, async/task, memory/address, and project-backed lowering
should increasingly reinforce each other instead of drifting apart.
