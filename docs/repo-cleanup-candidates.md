# Repo Cleanup Candidates

This file is the first-pass cleanup inventory for reducing repository noise
without accidentally deleting still-useful coverage.

The current goal is:

* keep the active `nuis -> NIR -> YIR -> LLVM/AOT` spine obvious
* reduce duplicated doc entrypoints
* separate narrow canonical examples from wider exploratory examples
* avoid deleting files that still carry regression, bridge, or packaging value

## Current Audit Snapshot

Latest audit result:

* `examples/projects/state/` currently has no orphaned project companion
  directories with zero doc/test/source references
* `examples/projects/task/` currently has no orphaned project companion
  directories with zero doc/test/source references
* current cleanup pressure is therefore no longer “delete obviously unused
  project demos”
* current cleanup pressure is now mostly:
  - duplicate README entrypoints
  - outdated semantic wording after async/GLM rule tightening
  - overlong local inventory READMEs that repeat routes already covered by
    `current-mainline-map.md`

Practical reading:

* if a demo still appears in a local README, `current-mainline-map.md`,
  reference docs, or focused tests, treat it as live until a narrower
  replacement path is chosen
* prefer deleting duplicate routes before deleting the underlying probe/demo
  itself

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

### Long local README inventories

These are not deletion targets yet, but they are the clearest next
documentation-trim targets whenever a folder README still repeats a large local
index that is already covered by:

* [docs/current-mainline-map.md](/Users/Shared/chroot/dev/nuislang/docs/current-mainline-map.md)
* [stdlib/std/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/std/README.md)
* local subdirectory READMEs

Strong current candidates:

* `examples/ns/memory/README.md`
  still mixes a useful small anchor set with a longer command-and-companion
  inventory that can likely be shortened into a tighter `first anchors + local
  detail lives here` structure
* `examples/invalid/README.md`
  small already, but still a plausible candidate to become even more of a pure
  router into `invalid/ns/*` and `invalid/yir/*` without maintaining its own
  recommended-check list

Cleanup now completed:

* `examples/ns/memory/README.md`
  has been reduced to a short router centered on ownership/task anchors plus
  direct links to `nir-memory-model`, `cpu-task-memory-contract`,
  `cpu-task-glm-contract`, and `cpu-task-payload-matrix`
* `examples/invalid/README.md`
  has been reduced to a pure invalid-example router with local subdirectory
  links and direct links back to the task payload matrix and cleanup policy

## Policy Chosen

### Checked-in generated bundles

Keep exactly two canonical checked-in bundles for now:

* `examples/bins/window_controls_demo_project/`
* `examples/bins/kernel_tensor_demo_project/`

Reason:

* they still anchor the current documented `nuis.toml` project workflow
* they give the repo one stable place to inspect emitted
  `nuis.build.manifest.toml`, `nuis.project.host_ffi.txt`,
  `nuis.project.abi.txt`, the manifest-level `abi_graph` summary, and related
  build artifacts
* current top-level docs and project docs still intentionally use them as
  concrete build targets rather than purely ephemeral `/private/tmp` outputs

Practical rule:

* keep only these two checked-in bundles
* do not re-introduce checked-in bundles for single-file `.ns` or handwritten
  `YIR` demos
* continue treating per-project `.nuis/` caches as local-only generated state

## Decision Needed

These can be cleaned, but only with an explicit repository-policy decision.

### Physical directory reshaping

These are no longer documentation-only questions. Changing them would affect
path stability, commands, or the mental model of the repository tree.

Current high-signal candidates:

* `examples/projects/`
  first-phase reshaping is now complete:
  - true end-to-end showcase projects such as `window_controls_demo` and
    `kernel_tensor_demo` remain at the root
  - narrow one-file companions now live under
    `examples/projects/task/`,
    `examples/projects/tooling/`,
    `examples/projects/state/`, and
    `examples/projects/filesystem/`
  Remaining future question:
  - whether the top-level directory name should stay `projects/`
    now that it also contains grouped companion subtrees
  Why this still needs a decision:
  - the first semantic split is done
  - but a later rename such as `project-companions/` would still be path churn
    with little immediate value unless the current grouped layout proves
    insufficient

* `examples/bins/`
  the current policy is now clear, but the directory name still reads like a
  generic dump of binaries rather than “checked-in canonical project bundles”.
  Future direction worth considering:
  - keep the current contents
  - but later rename or alias the area to something closer to
    `examples/project_builds/` or `examples/canonical-bundles/`
  Why this needs a decision:
  - many current build commands and docs point here directly
  - the current name is familiar but semantically weak

* `docs/fabric-spec/`, `docs/glm-spec/`, `docs/yir-spec/`, `docs/versioning/`
  these are design/spec directories, while `docs/reference/` is current truth.
  Future direction worth considering:
  - keep them separate
  - but possibly regroup them under a clearer second-level bucket such as
    `docs/design/` if we later want the top-level docs tree to read more
    cleanly
  Why this needs a decision:
  - large path churn
  - risk of breaking stable historical/spec references

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

1. keep shrinking front-door and local READMEs that still duplicate long
   companion inventories
2. observe whether the new grouped `examples/projects/{task,tooling,state,filesystem}`
   layout is already enough before considering any further rename
3. only revisit `examples/bins/` if we want to change the documented workflow
4. only then move or delete more source/example files

## Current Recommendation

Right now the safest immediate next deletion-like action is not source deletion.
It is:

* keep shrinking front-door docs
* keep converting long local README inventories into short routers
* make a conscious policy decision for checked-in generated bundles
* avoid more physical directory churn unless the new grouped
  `examples/projects/` layout still proves too noisy
