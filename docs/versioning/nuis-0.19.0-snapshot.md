# `nuis` 0.19.0 Snapshot

This file is the history anchor for the `0.19.*` line.

It follows the `0.18.*` mainline-closure push.

If `0.18.*` was where the repository proved that control flow, async/task,
memory/address, and project-backed compile truth could be described as one
mainline, `0.19.*` is where that mainline needs to become easier to maintain,
teach, and trust.

It is not a “finished language” milestone.

It is an internalization and hardening milestone.

## What `0.19.0` Means Here

`0.19.0` is the point where `nuis` should be read less as:

* one newly unified mainline that still needs translation layer-by-layer

and more as:

* one clearer compiler workflow with fewer current/historical ambiguities
* one more internalized source-style contract across examples and `std`
* one more explicit regression story for control flow, generics, async/task,
  memory/address, shader, and network
* one cleaner separation between source-facing syntax truth and lowered
  implementation truth

Short rule:

`0.19.0` is where the mainline should stop feeling newly assembled and start feeling deliberately maintained`

## High-Signal Current Surface

The most important current truths for `0.19.0` are:

* the compile frontdoor is now a real shared story:
  - `nuis status` / `nuis help` / `nuis workflow` /
    `project-doctor` / `project-status` / `scheduler-view`
    now read like one family
  - `check/test/build/release-check` remain the action spine
* the source-facing address surface is now explicitly normalized:
  - checked-in `.ns` examples and `std` modules prefer
    `ptr.value`, `ptr.next`, `buffer.len`, and `buffer[index]`
  - lowering/NIR/YIR docs deliberately keep builtin names such as
    `load_value(...)` and `store_at(...)`
* project-backed compile truth now matters more than isolated local greens:
  - `state_compile`
  - `task_compile`
  - `memory_compile`
  - `shader_nova_contracts`
  - `network_compile`
  together describe the believable spine
* control-flow, generic-bound, async, and memory/address work are now expected
  to stay aligned instead of progressing as separate stories
* current documentation has started to reflect those boundaries more honestly:
  - source-facing docs explain the preferred `.ns` style
  - implementation-facing docs explain lowered builtin truth

## What Is Still Intentionally Narrow

`0.19.0` should still avoid overclaiming:

* checked-in gate scripts are still named `check-0.18-*`
  even though the current line has moved forward
* broader loop-memory carry generalization is still intentionally narrower than
  the full verifier story
* host-boundary pointer ABI is still value-only even though internal `ref`
  semantics are real
* network/http compile closure is still ahead of final runtime portability and
  ahead of a polished public `std net` surface
* generic inference and specialization are stronger, but still not a fully
  general type-inference story

## Best Current Reading Order

For the `0.19.0` line, the shortest practical route is:

1. [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
2. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
3. [nuis-0.19.0-mainline-goals.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-goals.md)
4. [nuis-0.19.0-snapshot.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-snapshot.md)
5. [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
6. [nuis-0.19.0-mainline-regression-matrix.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-mainline-regression-matrix.md)
7. [nuis-0.19.0-address-pointer-mainline.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-address-pointer-mainline.md)
8. [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
9. [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
10. [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)

## Concrete Anchor Cases

If you want the shortest “show me the real thing” route for this snapshot,
start with:

* workflow/frontdoor anchor:
  [nuis-0.19.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.19.0-compile-workflow.md)
* state/control-flow proof:
  [state_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/state_compile.rs)
* task/async/control-flow proof:
  [task_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/task_compile.rs)
* address/pointer proof:
  [memory_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/memory_compile.rs)
* network/http/session proof:
  [network_compile.rs](/Users/Shared/chroot/dev/nuislang/tools/nuisc/tests/network_compile.rs)

## Recommended Practical Commands

For mainline project work:

```bash
cargo run -p nuis -- status
cargo run -p nuis -- workflow <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- project-status <project-dir|nuis.toml>
cargo run -p nuis -- scheduler-view <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

For fast `0.19.0` regression confidence, use:

```bash
bash scripts/check-0.19-mainline.sh
```

For the heavier compiler-facing gate, use:

```bash
bash scripts/check-0.19-release.sh
```

## Rule Of Thumb

If `0.18.*` was about proving one mainline exists, `0.19.*` is about making
that mainline easier to keep coherent across source style, docs, regression
gates, and project-backed compile truth.
