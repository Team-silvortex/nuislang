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

To hand the emitted manifest to the current linker frontdoor, use:

```bash
cargo run -p nsld -- check-next-action \
  examples/bins/native_artifact_closure_demo_project

cargo run -p nsld -- drive \
  examples/bins/native_artifact_closure_demo_project \
  --apply --until-clean --json
```

`check-next-action` is read-only. `drive --apply --until-clean` walks the
registered artifact chain with the internal Nsld whitelist until no next action
remains or it reaches a structured stop such as `not-applied`,
`repeated-next-action`, or `max-steps`. Host-assisted final executable blockers
should be read from the final pipeline/output reports rather than from a
repeated driver action.

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
* `link_plan_heterogeneous_domain_units`
* `link_plan_heterogeneous_domain_ready_units`
* `link_plan_heterogeneous_domain_readiness_ready`
* `link_plan_heterogeneous_domain_families`
* `link_plan_heterogeneous_domain_first_unready`
* `nsld_final_executable_output_ready`
* `nsld_final_executable_output_boundary_status`
* `nsld_final_executable_output_path_present`
* `nsld_final_executable_output_nsld_owned`
* `nsld_final_executable_output_blockers`
* `nsld_self_owned_image_ready`
* `nsld_self_owned_image_status`
* `nsld_self_owned_image_path`
* `nsld_self_owned_image_present`
* `nsld_self_owned_image_hash`
* `nsld_self_owned_image_header_valid`

Short reading rule:

* `workflow` tells you the shortest branch for the current input shape
* `project-status` tells you the current project/build surface summary
* `project-doctor` tells you the same route with more preflight/health detail
* `artifact-doctor` tells you whether the emitted native bundle is actually
  closed enough to run
* final-output ownership fields tell you whether the visible host-native output
  is missing, merely host-produced, or explicitly Nsld-owned
* `nsld_final_executable_output_boundary_status` is the normalized script-facing
  state for the final-output boundary
* `nsld_self_owned_image_status` is the normalized script-facing state for the
  internal `.nsb` image layer before host-shell or OS-native materialization
* heterogeneous-domain readiness fields summarize whether non-CPU domain units
  have the generic payload, lowering, sidecar, and bridge evidence needed by the
  current artifact route
* `ready_to_run` and `nsld_final_executable_output_ready` are deliberately
  separate: the first describes the current launchable AOT host binary path,
  while the second describes the stricter Nsld-owned final-output boundary
* self-contained Nsld `.nsb` image output is now a real Nsld-owned output
  boundary, but it is not yet the same thing as an OS-native executable

## Current Link Truth

The current line should be described honestly:

* `LinkPlan` is now a visible current model of the final native-artifact link
  route
* `nuis` frontdoors can already surface that model from
  `nuis.build.manifest.toml`
* the current native CPU final stage still resolves to host-native linking
  through `clang`
* the self-contained internal image route can produce an Nsld-owned `.nsb`
  image and launcher dry-run metadata
* heterogeneous bundle packing is modeled separately from host-native final
  link
* this is not yet the final host-shell / OS-native `nuis` linker architecture

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
* `nsld check-next-action` can expose the next linker artifact action without
  mutating the build directory
* `nsld drive --apply --until-clean` can materialize the current whitelisted
  Nsld artifact chain; host-assisted finalization blockers are carried by the
  emitted final pipeline/output metadata instead of by repeating the drive step
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
