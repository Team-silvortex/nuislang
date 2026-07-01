# Examples

This directory is no longer one flat “read everything here” area.

It contains several different layers:

* current source-level `.ns` examples
* current multi-file project examples
* handwritten `YIR` examples
* intentionally invalid verifier examples
* historical bridge material
* a very small set of checked-in build artifacts

If the example tree feels noisy, that is real. The current rule is:

* use this file only as a short router
* use [docs/current-mainline-map.md](../docs/current-mainline-map.md)
  for the current shortest path
* do not treat every example here as equally current or equally recommended

## Start Here

If you want the fastest route into the current repository spine:

* [docs/current-mainline-map.md](../docs/current-mainline-map.md)
* [docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md](../docs/versioning/nuis-alpha-0.4-doc-sync-inventory.md)
* [docs/examples-freshness-audit.md](../docs/examples-freshness-audit.md)
* [examples/projects/README.md](projects/README.md)
* [examples/ns/README.md](ns/README.md)

## Pick By Layer

* source-language examples:
  [examples/ns](ns)
  Start with:
  [hello_world.ns](ns/core/hello_world.ns),
  [hello_ref_struct.ns](ns/types/hello_ref_struct.ns),
  [hello_task_glm_value_path.ns](ns/memory/hello_task_glm_value_path.ns)
* multi-file project examples:
  [examples/projects](projects)
  Start with:
  [window_controls_demo](projects/window_controls_demo),
  [kernel_tensor_demo](projects/kernel_tensor_demo)
* handwritten `YIR` anchors:
  [examples/yir](yir)
  Start with:
  [hello_yir.yir](yir/demos/hello_yir.yir),
  [data_fabric_demo.yir](yir/data/data_fabric_demo.yir),
  [kernel_tensor_demo.yir](yir/kernel/kernel_tensor_demo.yir)
* invalid/verifier examples:
  [examples/invalid](invalid)
* retired historical material summaries

## Freshness Rule

Examples in this repository are now best read in four roles:

* frontdoor:
  the shortest current entrypoints called out by README and mainline-map routes
* companion:
  still useful, but mainly there to cover one feature, contract, or regression
* probe:
  validation, experiment, or future-facing routes that should not be mistaken
  for first-stop reading
* legacy:
  historical material that is better summarized by docs than kept as active
  checked-in example source

That means:

* age alone is not a reason to delete an example
* but old companions should stop being homepage material once better anchors
  exist
* probe routes can stay in-tree when they still support active design or host
  validation docs, even if they are not current onboarding material

## Local Guides

Drill into the area you are actually touching:

* [examples/ns/README.md](ns/README.md)
* [examples/projects/README.md](projects/README.md)
* [examples/yir/README.md](yir/README.md)
* [examples/invalid/README.md](invalid/README.md)
* [examples/bins/README.md](bins/README.md)
