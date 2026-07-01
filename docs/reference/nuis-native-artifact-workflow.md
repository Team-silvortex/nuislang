# `nuis` Native Artifact Workflow

This file is the shortest current reference for the native artifact closure
that `nuis` can already express today.

It answers one narrow question:

`what is the clearest current build -> inspect -> verify -> launch route for a real nuis-produced native binary bundle?`

## Current Frontdoor

Use this checked-in example first:

* [native_artifact_closure_demo](../../examples/projects/tooling/native_artifact_closure_demo)

Use this command chain:

```bash
cargo run -p nuis -- build \
  examples/projects/tooling/native_artifact_closure_demo \
  examples/bins/native_artifact_closure_demo_project

cargo run -p nuis -- inspect-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml

cargo run -p nuis -- verify-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.compiled.artifact

cargo run -p nuis -- artifact-doctor \
  examples/bins/native_artifact_closure_demo_project

cargo run -p nuis -- verify-build-manifest \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml

cargo run -p nuis -- run-artifact \
  examples/bins/native_artifact_closure_demo_project/nuis.build.manifest.toml
```

If you want the CLI to classify the route before you build, use:

```bash
cargo run -p nuis -- workflow \
  examples/projects/tooling/native_artifact_closure_demo

cargo run -p nuis -- project-status \
  examples/projects/tooling/native_artifact_closure_demo

cargo run -p nuis -- project-doctor \
  examples/projects/tooling/native_artifact_closure_demo
```

After a successful build, those frontdoors now also expose the current
artifact-follow-up state:

* `artifact_ready_to_run`
* `artifact_recommended_next_step`
* `link_plan_available`
* `link_plan_final_stage`
* `link_plan_final_driver`
* `link_plan_final_link_mode`
* `link_plan_final_output`
* `link_plan_domain_units`

Short reading rule:

* `workflow` tells you the shortest branch for the current input shape
* `project-status` tells you the current project/build surface summary
* `project-doctor` tells you the same route with more preflight/health detail
* `artifact-doctor` tells you whether the emitted native bundle is actually
  closed enough to run

## Current Link Truth

The current line should be described honestly:

* `LinkPlan` is now a visible current model of the final native-artifact link
  route
* `nuis` frontdoors can already surface that model from
  `nuis.build.manifest.toml`
* the current native CPU final stage still resolves to host-native linking
  through `clang`
* heterogeneous bundle packing is modeled separately from host-native final
  link
* this is not yet the final self-owned `nuis` linker architecture

Short rule:

`the current repository can already describe the native artifact closure clearly, even though the final linker implementation is still partly host-toolchain-backed`

## What This Proves

Today this route proves all of these together:

* project-form `nuis` source compiles through `nuisc`
* LLVM IR and native CPU outputs are emitted
* `nuis.build.manifest.toml` is written
* `nuis.compiled.artifact` is written
* the compiled artifact can be inspected from either the manifest path or the
  artifact path
* the current output directory can be summarized in one doctor-style view
* the current frontdoor can restate whether the artifact closure is ready to
  run and what the current final link stage looks like
* the compiled artifact and manifest both survive verifier checks
* the produced native binary actually launches successfully through the `nuis`
  frontdoor

## Current Checked-In Gates

The current checked-in coverage is split deliberately:

* checked-in project compile anchor:
  [tooling_compile.rs](../../tools/nuisc/tests/tooling_compile.rs)
* AOT compile/package/launch smoke:
  [lib.rs](../../tools/nuisc/src/lib.rs)
* representative native control-flow compile/launch smoke:
  [artifact_cli.rs](../../tools/nuisc/tests/artifact_cli.rs)

Short rule:

`the example gives the repository a visible frontdoor; the AOT smoke proves the emitted binary is not only packaged but launchable`

## Native Control-Flow Smoke

The native artifact route now also covers a small representative set of
structured control-flow examples. This matters because project/YIR compile
success alone is not enough to prove the final LLVM block graph and host-linked
binary are coherent.

Useful local commands:

```bash
cargo test -p yir-lower-llvm
cargo test -p nuisc --test state_compile --test task_compile
cargo test -p nuisc --test artifact_cli cli_compile_emits_runnable_native_control_flow_binaries
```

Current smoke families:

* state flow/post-flow branching loops
* async flow/post-flow loop-control conditions
* async post-flow shared-suffix loop-control carrying

Launch rule:

`the smoke requires the produced executable to launch and return an exit status; it does not require status 0 because these examples often return business values`

Current honest boundary:

* this proves a real `nuis -> nuisc -> native-cpu-llvm -> clang -> executable`
  route for representative control-flow shapes
* this does not replace the future self-owned `nuis` linker
* this does not mean every high-level source CFG can lower natively yet
