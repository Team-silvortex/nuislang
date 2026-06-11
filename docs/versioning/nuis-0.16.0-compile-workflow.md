# `nuis` Compile Workflow For `0.16.0`

This is the current recommended compile workflow to stabilize and document for
the `0.16.0` line.

The goal is simple:

* one front door: `nuis`
* one default validation path: `doctor -> check -> test -> build`
* one release gate: `release-check`
* one debug path when things go wrong: `dump-ast -> dump-nir -> dump-yir`

Use this file as the shortest operational route for day-to-day project work.

## Current `0.16.0` Validation Spine

The compile workflow is no longer just “CLI commands exist”.

For the current `0.16.0` line, the practical confidence stack is:

```text
frontend generic + async/task rewrite probes
  -> real project compile harnesses
  -> targeted diagnostic guardrails
  -> project-doctor / check / test / build / release-check
```

That means the current workflow is backed by three distinct validation layers:

* frontend crossover probes:
  generic expected-type propagation, alias-aware struct/payload inference,
  async/task `spawn` / `join` / `join_result`, higher-order specialization,
  expression-level generic specialization through helper call chains,
  and `if` / `match` control-flow routes
* real project compile harnesses:
  checked-in `examples/projects/...` compile through the actual project
  pipeline, not only through single-file frontend parsing
* diagnostic guardrails:
  common misuse routes such as sync code directly calling async helpers or
  `spawn(...)` being fed a sync builder now have explicit regression coverage

There is now a fourth practical truth worth saying out loud:

* loop-family lowering closure:
  executable counted/carry/flow/post-flow `while` lowering is no longer limited
  to straight-line branch shells; nested control predicates from lowered `if`
  and `match` routes now participate in the checked-in loop node families too

And there is now a fifth one that matters for the project-facing generic story:

* generic helper + higher-order bridge closure:
  explicit generic helper chains no longer stop at frontend-only probes;
  they now survive lambda lifting, higher-order specialization, async/task
  routes, and checked-in real project compile harnesses too

Use this as the current rule of thumb:

* if a route only works in a toy snippet, it is not enough
* if a route works in frontend probes and real project compile harnesses, it
  is part of the `0.16.0` story
* if a common misuse has no regression test, the workflow is not mature enough

## Lowering Support Matrix

For the current `0.16.0` line, the checked-in executable `while` lowering
truth is:

* `closed`:
  counted `while` loops
* `closed`:
  chained carry `while` loops
* `closed`:
  pre-carry flow control with `break` / `continue`
  via `loop_while_i64_flow_chain`
* `closed`:
  pre-carry flow control with branching carry updates
  via `loop_while_i64_flow_cond_chain`
* `closed`:
  post-carry flow control with `break` / `continue`
  via `loop_while_i64_post_flow_chain`
* `closed`:
  post-carry flow control with branching carry updates
  via `loop_while_i64_post_flow_cond_chain`
* `closed`:
  `match`-prefixed control-flow temps feeding loop control lowering
* `closed`:
  nested `if -> break/continue` folded into `and` control conditions
* `closed`:
  nested `match` / branch-local `continue` folded into `or` control conditions
* `partial`:
  general iterative/backedge `while` lowering outside the counted/carry/flow/post-flow subset

Read that matrix conservatively:

* when a loop shape maps onto the counted/carry/flow/post-flow families, we
  should test and lean on it
* when a loop shape escapes those families, the honest current answer is still
  “general iterative loop/backedge lowering is not implemented”

## Canonical Stages

```text
project or source input
  -> nuis project-doctor
  -> nuis check
  -> nuis test
  -> nuis build
  -> nuis verify-build-manifest
  -> nuis release-check
```

In practice:

* `project-doctor` tells you whether the project shape is healthy enough to
  work on.
* `check` validates parsing, frontend, project wiring, `NIR`, `YIR`, ABI
  selection, and verifier rules without producing the final build directory.
* `test` runs language-level `test(...)` functions and project-declared test
  inputs.
* `build` emits the AOT output bundle.
* `verify-build-manifest` checks the generated build manifest directly.
* `release-check` is the canonical final gate because it runs `check`, `build`,
  and manifest verification together.

The operational intent is:

* `doctor` catches project-shape and link-surface problems early
* `check` is the canonical compiler truth gate
* `test` confirms language/runtime-facing routes
* `build` proves the AOT bundle path
* `release-check` is the smallest honest “ready enough to cut” answer

## Default Project Workflow

For a normal multi-file project:

```bash
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
```

For `0.16.0`, this route is the one we should keep teaching by default.

If a feature requires a different everyday ritual, prefer fixing the toolchain
or narrowing the claim instead of teaching a more fragile route.

Then verify the emitted manifest if you want the build artifact checked as a
standalone package description:

```bash
cargo run -p nuis -- verify-build-manifest <output-dir>/nuis.build.manifest.toml
```

For a final pre-release pass, prefer:

```bash
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

## Default Single-Source Workflow

For a single `.ns` file, the default route is shorter:

```bash
cargo run -p nuis -- check <input.ns>
cargo run -p nuis -- test <input.ns>
cargo run -p nuis -- build <input.ns> <output-dir>
```

Use this when you are iterating on language behavior or compiler contracts
without needing a full `nuis.toml` project.

## Debug Workflow

When `check` fails, use the compiler IR dumps in this order:

```bash
cargo run -p nuis -- dump-ast <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-nir <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-yir <input.ns|project-dir|nuis.toml>
```

Use:

* `dump-ast` for parser, annotations, surface syntax, and binding shape issues
* `dump-nir` for frontend typing, generic rewrite, pattern lowering, and
  project validation issues
* `dump-yir` for lowering, scheduling, result-family, and verifier-adjacent
  issues

If the problem looks scheduling-specific, add:

```bash
cargo run -p nuis -- scheduler-view <input.ns|project-dir|nuis.toml>
```

When debugging compiler regressions, the shortest current drill is:

1. reproduce with the smallest project or source input
2. inspect `dump-nir`
3. check whether the failure belongs to:
   generic rewrite / expected-type propagation,
   async/task lowering,
   project validation,
   or later `YIR` / verifier stages
4. confirm the route against the closest checked-in probe before widening any fix

For generic-heavy routes, the best current anchors are:

* [nuis-0.16.0-generic-surface-audit.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-surface-audit.md)
* [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)

For real project generic/higher-order bridge routes, the best current anchor is:

* `examples/projects/domains/net_http_session_loop_bridge_recipe_demo`
  through [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

## Project Triage Workflow

When a project feels unhealthy before you even get to compile errors, use:

```bash
cargo run -p nuis -- project-status <project-dir|nuis.toml>
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
```

Use:

* `project-status` to inspect the current visible project plan and package
  shape
* `project-doctor` to get fix-oriented project guidance

If you want to freeze today’s recommended ABI choices into the project:

```bash
cargo run -p nuis -- project-lock-abi <project-dir|nuis.toml>
```

## Test Workflow

The default test command is:

```bash
cargo run -p nuis -- test <project-dir|nuis.toml>
```

Useful variants:

```bash
cargo run -p nuis -- test --list <project-dir|nuis.toml>
cargo run -p nuis -- test --ignored <project-dir|nuis.toml>
cargo run -p nuis -- test --include-ignored <project-dir|nuis.toml>
cargo run -p nuis -- test --exact <project-dir|nuis.toml> <test-name>
```

Use these when:

* you want to inspect available language tests without running them
* you want to run ignored probes intentionally
* you want to pin one exact test during compiler work

## Build Output Workflow

The default build command is:

```bash
cargo run -p nuis -- build <input.ns|project-dir|nuis.toml> <output-dir>
```

Use explicit CPU/target overrides only when you are intentionally freezing the
output ABI:

```bash
cargo run -p nuis -- build --cpu-abi <abi> <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- build --target <triple> <project-dir|nuis.toml> <output-dir>
```

The `0.16.0` rule should be:

* default to auto ABI selection while iterating
* only lock ABI/target when release packaging or reproducibility matters

## Cache Workflow

Cache commands are part of the compile workflow, not a side channel:

```bash
cargo run -p nuis -- cache-status <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- clean-cache <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- cache-prune <input.ns|project-dir|nuis.toml>
```

Use them when:

* build reuse looks suspicious
* you want to confirm cache hits/misses while iterating
* you want to keep the compile surface reproducible during release prep

## Compiler Maintenance Workflow

When the task is “stabilize the compiler” rather than “compile one project”,
the current `0.16.0` maintenance loop should be:

```text
add or tighten frontend probe
  -> run focused `cargo test -q -p nuisc ...`
  -> run relevant real-project compile harness
  -> update the versioning docs that describe the route
```

In concrete terms:

* use [tests_generics.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_generics.rs)
  when the route is about generic propagation, alias-aware expectation, async/task crossover,
  or control-flow-local specialization
* use [tests_loop_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_loop_flow.rs)
  and [tests_loop_post_flow.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/lowering/tests_loop_post_flow.rs)
  when the route is about executable loop-family lowering truth
* use [tests_higher_order.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/tests_higher_order.rs)
  when the route includes lambda or higher-order specialization
* use [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)
  when the route should survive real `examples/projects/domains` compile entrypoints
* use [tools/nuisc/src/frontend/mod.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/src/frontend/mod.rs)
  diagnostic tests when the goal is stable misuse reporting rather than successful lowering

The shortest honest success criterion for a compiler change is now:

* the focused probe passes
* the closest real project compile harness still passes
* the versioning docs still describe reality

## Recommended `0.16.0` Reading Rule

If someone asks “how do I compile a `nuis` project today?”, the shortest answer
should be:

1. run `nuis project-doctor`
2. run `nuis check`
3. run `nuis test`
4. run `nuis build`
5. run `nuis release-check` before calling it release-ready

If that answer starts growing caveats, we should tighten the toolchain surface
instead of teaching a more complicated ritual.
