# Repo Cleanup Candidates

This file is the first-pass cleanup inventory for reducing repository noise
without accidentally deleting still-useful coverage.

The current goal is:

* keep the active `nuis -> NIR -> YIR -> LLVM/AOT` spine obvious
* reduce duplicated doc entrypoints
* separate narrow canonical examples from wider exploratory examples
* avoid deleting files that still carry regression, bridge, or packaging value

## Cleanup Rule

Use three buckets:

* `keep`
  still active in the current mainline map or still carrying narrow regression
  value
* `archive next`
  still useful historically, but now clearly overshadowed by narrower canonical
  chains
* `decision needed`
  plausible cleanup targets, but deleting them changes workflow expectations,
  generated-artifact policy, or reference commands

## Keep

These should stay in the mainline for now.

### Core current examples

* `examples/projects/`
  recipe-companion project demos for task, host I/O, persistence, path, and
  filesystem surfaces
* `examples/ns/ffi/`
  narrow facade mirrors aligned with current `std/*_recipe.ns`
* `examples/ns/memory/`
  current task/GLM and payload-boundary examples
* `examples/yir/`
  current handwritten `YIR` probes and domain anchors

### Current `std` shape

* `stdlib/std/*_recipe.ns`
  these are now the real canonical `std` growth path
* narrow runtime helpers still used as raw facade surfaces:
  - `argv_runtime.ns`
  - `env_runtime.ns`
  - `process_runtime.ns`
  - `stdin_runtime.ns`
  - `tty_runtime.ns`
  - `cwd_runtime.ns`
  - `temp_runtime.ns`
  - `home_runtime.ns`
  - `config_runtime.ns`
  - `kv_runtime.ns`
  - `cache_runtime.ns`

### Explicit archive areas that already exist

* `docs/historical/`
* `examples/legacy/`

## Removed In Current Cleanup

These were previously secondary wider native examples. They have now been
removed from the repository after being demoted out of the mainline docs.

* `examples/ns/ffi/hello_native_input_tool.ns`
* `examples/ns/ffi/hello_native_cli_pipeline.ns`
* `examples/ns/ffi/hello_native_tool_runner.ns`
* `examples/ns/ffi/hello_native_cli_runtime.ns`
* `examples/projects/native_cli_pipeline_demo/`
* `examples/projects/native_tool_runner_demo/`
* `examples/projects/native_branch_cli_demo/`

## Archive Next

These are the strongest candidates to move out of the shortest-path docs and,
if desired, relocate under a more explicit archive bucket after references are
updated.

### Wider native CLI / workflow examples

These are not necessarily bad examples, but they are now overshadowed by
narrower `recipe -> facade -> project` chains.

Previous targets that motivated this cleanup:

* `examples/ns/ffi/hello_native_input_tool.ns`
* `examples/ns/ffi/hello_native_cli_pipeline.ns`
* `examples/ns/ffi/hello_native_tool_runner.ns`
* `examples/ns/ffi/hello_native_cli_runtime.ns`
* `examples/projects/native_cli_pipeline_demo/`
* `examples/projects/native_tool_runner_demo/`
* `examples/projects/native_branch_cli_demo/`

Cleanup just completed:

* they were first removed from recommended routes
* then isolated as secondary examples
* finally deleted once references were quiet

## Policy Chosen

### Checked-in generated bundles

Keep exactly two canonical checked-in bundles for now:

* `examples/bins/window_controls_demo_project/`
* `examples/bins/kernel_tensor_demo_project/`

Reason:

* they still anchor the current documented `nuis.toml` project workflow
* they give the repo one stable place to inspect emitted
  `nuis.build.manifest.toml`, `nuis.project.host_ffi.txt`,
  `nuis.project.abi.txt`, and related build artifacts
* current top-level docs and project docs still intentionally use them as
  concrete build targets rather than purely ephemeral `/private/tmp` outputs

Practical rule:

* keep only these two checked-in bundles
* do not re-introduce checked-in bundles for single-file `.ns` or handwritten
  `YIR` demos
* continue treating per-project `.nuis/` caches as local-only generated state

## Decision Needed

These can be cleaned, but only with an explicit repository-policy decision.

### Future re-evaluation of generated bundle outputs

* `examples/bins/window_controls_demo_project/`
* `examples/bins/kernel_tensor_demo_project/`

Why this needs a decision:

* they are generated outputs, not handwritten source
* but current top-level docs and commands still point at them
* they are also serving as checked-in example build artifacts and manifest
  snapshots

Current status:

* policy 1 is now the active repository policy
* revisit only if we later want to make `/private/tmp` rebuilds the only
  supported artifact inspection route

### Raw runtime helper modules

Files like `stdin_runtime.ns` and `tty_runtime.ns` are still useful as thin raw
facade surfaces, but the repository now increasingly teaches the recipe layer
first.

Possible future direction:

* keep raw runtime helpers, but stop surfacing most of them in first-read docs
* or move them under a more explicit “raw host facades” section inside
  `stdlib/std/README.md`

## Suggested Next Cleanup Pass

If we want a low-risk second pass, do it in this order:

1. remove archived-native examples from recommended-reading sections
2. add a single “wider native examples” section for the files listed above
3. only revisit `examples/bins/` if we want to change the documented workflow
4. only then move or delete files

## Current Recommendation

Right now the safest immediate next deletion-like action is not source deletion.
It is:

* keep shrinking front-door docs
* demote wider native examples out of the mainline route
* make a conscious policy decision for checked-in generated bundles
