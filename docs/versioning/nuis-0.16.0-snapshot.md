# `nuis` 0.16.0 Snapshot

This file is the history anchor for the `0.16.*` line.

It is the point where the repository stopped looking like only a collection of
compiler experiments and started looking much more like a coherent binary
compile toolchain with one visible workflow.

It is still not a “language complete” milestone.

It is a maturity-line milestone.

## What `0.16.0` Means Here

`0.16.0` is the point where the repository gained a much clearer shared spine
across:

* one canonical `nuis` compile workflow
* one stronger binary-compile maturity target
* modularized frontend / lowering orchestration
* practical generic struct and payload syntax
* stronger generic constraint and method-bound validation
* tightened async ownership and task-result verification
* task + memory + packet-like examples that look like real higher-level
  protocol groundwork
* `std net` and domains-side network routes that are real compile truth, plus a
  clearer separation between compile truth and runtime truth

This is the first line where “how to compile a project” and “what the compiler
can honestly stand on” became much easier to explain in one route.

## High-Signal Current Surface

The most important current truths for `0.16.0` are:

* `nuis` now has a deliberately singular project route:
  `project-doctor -> check -> test -> build -> release-check`
* `release-check` is the explicit final gate for the current binary compile
  story.
* the project layer is real compiler truth:
  plan, ABI mode, packets, contracts, support surfaces, and manifest
  verification all sit on one clearer route.
* frontend generic support is now much more practical:
  - `Fn1`, `Fn2`, `Fn3`
  - alias-aware generic callable families
  - generic structs
  - explicit generic struct literals
  - explicit generic payload constructors
  - inferred single-field generic payload constructors
  - inferred transparent generic alias payload constructors
  - shorthand destructuring
  - shorthand `match`
  - nested shorthand
  - alias-aware generic struct patterns
* generic constraint validation and generic method-bound diagnostics now behave
  like real user-facing compiler surfaces, including:
  - alias-chain validation
  - nested alias-chain context
  - control-flow-local binding environments
  - guarded `match` and nested `match`
  - lambda / higher-order helper context restoration
  - call-inferred locals and call receivers
  - call-root destructuring validation
* async/task ownership rules are much tighter:
  - `join(...)`
  - `join_result(...)`
  - `cancel(...)`
  - `timeout(...)`
  are all now explicit task-handle boundaries
* task-result verification is stronger:
  `task_value(...)` is now constrained by completed-path facts instead of being
  only a documentation convention.
* task + memory examples now form a meaningful protocol groundwork:
  request-plan / response-packet / session-policy / staged readback shapes are
  real compile examples, not just sketches.
* `std net` and domains network routes now have compile-level ladders that
  reach:
  - client/session packet recipes
  - service/session packet recipes
  - exchange contract demos
  - request/response line-block demos

## What Is Still Intentionally Narrow

`0.16.0` still has clear boundaries:

* pattern matching is now practical, but it is not yet a full ADT-pattern
  system.
* generic payload constructors are much better, including direct and
  transparent-alias single-field inference, but they still stop short of full
  unconstrained inference across arbitrary constructor/alias shapes.
* async/task verification is much stronger, but the runtime story is still
  narrower than the compile story.
* network compile truth is ahead of network runtime truth:
  - compile routes for host-network and httpish lanes are real
  - runtime probes show that socket-enabled host behavior still depends on the
    actual execution environment
* `std net` is now easier to route and reason about, but its highest layers are
  still recipe-heavy rather than a final polished builder/API surface.

## Best Current Reading Order

For `0.16.0`, the shortest practical route is:

1. [README.md](/Users/Shared/chroot/dev/nuislang/README.md)
2. [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
3. [nuis-0.16.0-compile-workflow.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-compile-workflow.md)
4. [nuis-0.16.0-binary-compile-maturity.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-binary-compile-maturity.md)
5. [nuis-0.16.0-generic-constraint-coverage.md](/Users/Shared/chroot/dev/nuislang/docs/versioning/nuis-0.16.0-generic-constraint-coverage.md)
6. [docs/reference/README.md](/Users/Shared/chroot/dev/nuislang/docs/reference/README.md)
7. [examples/projects/README.md](/Users/Shared/chroot/dev/nuislang/examples/projects/README.md)
8. [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)

## `0.16.0` Focus Areas

If you want the shortest thematic picture of this line:

* compile workflow:
  `one front door, one default route, one final gate`
* binary maturity:
  `frontend -> project plan -> YIR verify -> AOT -> manifest -> release-check`
* generics:
  `practical generic structs + stronger alias-aware validation`
* async truth:
  `task ownership tightened enough to treat async verification as real compiler truth`
* protocol groundwork:
  `task + memory + packet/session shapes that higher network/http layers can stand on`
* network truth:
  `compile ladders are real; runtime must still be probed honestly per host`

## Recommended Practical Commands

For project work:

```bash
cargo run -p nuis -- project-doctor <project-dir|nuis.toml>
cargo run -p nuis -- check <project-dir|nuis.toml>
cargo run -p nuis -- test <project-dir|nuis.toml>
cargo run -p nuis -- build <project-dir|nuis.toml> <output-dir>
cargo run -p nuis -- release-check <project-dir|nuis.toml> <output-dir>
```

For compiler debugging:

```bash
cargo run -p nuis -- dump-ast <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-nir <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- dump-yir <input.ns|project-dir|nuis.toml>
cargo run -p nuis -- scheduler-view <input.ns|project-dir|nuis.toml>
```

For network runtime reality checks:

```bash
cargo run -p nuis -- build <network-probe-project> <output-dir>
<output-dir>/<binary>
```

and then compare the result against
[../reference/network-runtime-host-validation.md](/Users/Shared/chroot/dev/nuislang/docs/reference/network-runtime-host-validation.md).

## Rule Of Thumb

If `0.16.0` has to be summarized in one sentence, it is this:

`0.16.0` is where `nuis` became much easier to teach as a real compile toolchain, even though some runtime and high-level library surfaces are still intentionally maturing.
