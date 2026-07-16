# `std/tooling`

This directory is the reading router for the `std` command/workflow/tooling
lane.

Keep the current recipe sources in
[`stdlib/std`](../../../stdlib/std) for now; this
file exists to give the tooling surface one lane-shaped front door before we
do any higher-risk filesystem reshuffle.

Canonical companions:

* `std` refactor frontdoor:
  [docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md](../../../docs/versioning/nuis-0.20.0-std-refactor-frontdoor.md)
* tooling/workflow contract:
  [docs/reference/std-tooling-workflow-contract.md](../../../docs/reference/std-tooling-workflow-contract.md)
* auto-injected CLI/workflow helper surface:
  [lib/cli_contracts.ns](../../../stdlib/std/lib/cli_contracts.ns)
* tooling image-preprocess bridge:
  [docs/reference/tooling-image-preprocess-lane.md](../../../docs/reference/tooling-image-preprocess-lane.md)
* mainline layering rule:
  [docs/reference/std-mainline-layering-contract.md](../../../docs/reference/std-mainline-layering-contract.md)
* shortest repo-wide route:
  [docs/current-mainline-map.md](../../../docs/current-mainline-map.md)

## Current Lane Shape

Read the current tooling surface in this order:

```text
command runtime
-> subprocess runtime
-> workflow runtime
-> workflow recipe
-> cli/report/session companions
-> build/project/workflow frontdoors
```

Short rule:

* command/subprocess should stay the narrow request/result layer
* workflow should stay the first explicit gate/plan/report layer
* CLI/report/build/project recipes should reuse that gate shape rather than
  silently inventing a new one
* shared scoring/exit-code helpers should live in `StdCliContracts`, not in
  every launch-shaped recipe

## Current Semantic Split

The current tooling lane should now be read in two buckets, not one.

### Launch-Shaped Frontdoor Recipes

These are the recipes that should be read as current AOT/native-artifact
frontdoors. They are allowed to exercise host bridges internally, but their
process exit shape should summarize frontdoor success/failure rather than leak
raw internal counters.

* [cli_runtime_recipe.ns](../../../stdlib/std/cli_runtime_recipe.ns)
* [cli_session_recipe.ns](../../../stdlib/std/cli_session_recipe.ns)
* [cli_report_session_recipe.ns](../../../stdlib/std/cli_report_session_recipe.ns)
* [workflow_runtime_recipe.ns](../../../stdlib/std/workflow_runtime_recipe.ns)
* [command_runtime_recipe.ns](../../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../../stdlib/std/subprocess_runtime_recipe.ns)
* [workflow_frontdoor_runtime_recipe.ns](../../../stdlib/std/workflow_frontdoor_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](../../../stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](../../../stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](../../../stdlib/std/cli_project_build_report_recipe.ns)
* [cli_compile_workflow_recipe.ns](../../../stdlib/std/cli_compile_workflow_recipe.ns)

### Probe-Style Observation Recipes

These are still useful current recipes, but they should be read primarily as
host/runtime observation or shaping probes rather than as the default CLI
artifact frontdoor.

* [command_shell_recipe.ns](../../../stdlib/std/command_shell_recipe.ns)
* [command_text_builder_recipe.ns](../../../stdlib/std/command_text_builder_recipe.ns)
* [report_runtime_recipe.ns](../../../stdlib/std/report_runtime_recipe.ns)
* [automation_runtime_recipe.ns](../../../stdlib/std/automation_runtime_recipe.ns)
* [host_text_runtime_recipe.ns](../../../stdlib/std/host_text_runtime_recipe.ns)
* [text_pipeline_recipe.ns](../../../stdlib/std/text_pipeline_recipe.ns)
* [text_report_builder_recipe.ns](../../../stdlib/std/text_report_builder_recipe.ns)

Short rule:

* launch-shaped recipes are the current mainline candidates for
  `build -> artifact-doctor -> run-artifact`
* probe-style recipes are still current, but should not be mistaken for the
  default user-facing CLI frontdoor
* text observation recipes now include reusable length, concat, line-count, and
  word-count probes before feeding report/JSON shaping
* text pipeline probes now attach those statistics to the generated pipeline
  handle itself, not a detached sample literal
* format runtime probes now apply the same text statistics to generated pair
  handles, not only to report handles
* text+JSON probes now keep computed/measured length consistency alongside
  line-count and word-count statistics for merged text handles
* JSON runtime and report runtime recipes reuse the standard JSON shape helper
  instead of scoring pair/array/object lengths ad hoc
* report runtime recipes now keep filesystem, JSON, and stdout scoring routed
  through their own std contracts before combining the report summary
* IO runtime and terminal IO recipes now reuse the standard console/terminal
  status helpers instead of adding write/read/TTY probes ad hoc
* CLI session and report-session recipes now combine session progress through
  `StdCliContracts` and console status through `StdIoContracts`
* command text and shell session probes now observe generated argv/env command
  text with the same length, line-count, and word-count helpers
* report builder probes now apply those text statistics to generated report
  handles, not only to source literals
* report+JSON probes preserve the same generated-report statistics while also
  validating JSON shape and text length consistency
* benchmark report probes reuse the same generated-report statistics without
  folding text concerns into the time contract itself
* count-aware benchmark reports keep the same generated-report statistics while
  varying the active sample window
* file-output benchmark reports keep the same generated-report statistics while
  validating filesystem write/close behavior separately
* IO and filesystem IO reports reuse the same generated-report statistics while
  leaving console and filesystem validation in their own contracts
* time reports reuse the same generated-report statistics while preserving the
  time sample contract as a separate concern
* filesystem report recipes also expose generated-report statistics even when
  they are recipe-only rather than project-frontdoor entries

## Current Authoring Shape

Within the narrow tooling recipe layer, prefer one stable source pattern:

```text
Seed -> build_*_context -> capture_* -> summarize_*
```

What that means in practice:

* recipe seeds should carry the scenario knobs, not implicit magic constants
* context builders should decide inherit/default policy in one place
* `capture_*` should assemble typed request/result/report values
* `summarize_*` should be the only place that collapses those values into one
  current host-facing integer

Current exemplars:

* [command_runtime_recipe.ns](../../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../../stdlib/std/subprocess_runtime_recipe.ns)
* [workflow_runtime_recipe.ns](../../../stdlib/std/workflow_runtime_recipe.ns)

Short rule:

* if two tooling recipes are expressing the same gate shape, they should differ
  by seeds and reports before they differ by naming style

## Current High-Level Companion Shape

For the current project-form compile ladder companions, prefer one explicit
high-level pattern:

```text
Seed
-> capture context
-> build step bundle
-> run/skip ordered steps
-> build success/failure report
-> build summary
-> return one exit code
```

What that means in practice:

* context capture should happen before step assembly
* step assembly should be visible through one `*Steps` struct
* step execution should stay as ordered `run_step` / `skipped_step`
* success/failure should collapse through small shared helpers such as
  `build_success_report`, `build_failed_report`, and `should_stop`
* the final `main()` should remain a narrow terminal handoff instead of
  re-encoding workflow logic

Current checked-in companion exemplars:

* [cli_compile_workflow_demo](../../../examples/projects/tooling/cli_compile_workflow_demo)
* [cli_workflow_automation_demo](../../../examples/projects/tooling/cli_workflow_automation_demo)
* [cli_build_pipeline_demo](../../../examples/projects/tooling/cli_build_pipeline_demo)
* [cli_project_build_report_demo](../../../examples/projects/tooling/cli_project_build_report_demo)

## Source Router

### Runtime Edge

* [command_runtime.ns](../../../stdlib/std/command_runtime.ns)
* [subprocess_runtime.ns](../../../stdlib/std/subprocess_runtime.ns)
* [workflow_runtime.ns](../../../stdlib/std/workflow_runtime.ns)

### Narrow Recipe Layer

* [command_runtime_recipe.ns](../../../stdlib/std/command_runtime_recipe.ns)
* [subprocess_runtime_recipe.ns](../../../stdlib/std/subprocess_runtime_recipe.ns)
* [workflow_runtime_recipe.ns](../../../stdlib/std/workflow_runtime_recipe.ns)
* [command_shell_recipe.ns](../../../stdlib/std/command_shell_recipe.ns)
* [command_text_builder_recipe.ns](../../../stdlib/std/command_text_builder_recipe.ns)
* text/data builder route:
  [host_text_runtime_recipe.ns](../../../stdlib/std/host_text_runtime_recipe.ns)
  ->
  [text_pipeline_recipe.ns](../../../stdlib/std/text_pipeline_recipe.ns)
  ->
  [text_report_builder_recipe.ns](../../../stdlib/std/text_report_builder_recipe.ns)

### Session / Report / Automation Companions

* [cli_session_recipe.ns](../../../stdlib/std/cli_session_recipe.ns)
* [cli_shell_session_recipe.ns](../../../stdlib/std/cli_shell_session_recipe.ns)
* [cli_report_session_recipe.ns](../../../stdlib/std/cli_report_session_recipe.ns)
* [cli_runtime_recipe.ns](../../../stdlib/std/cli_runtime_recipe.ns)
* [report_runtime_recipe.ns](../../../stdlib/std/report_runtime_recipe.ns)
* [automation_runtime_recipe.ns](../../../stdlib/std/automation_runtime_recipe.ns)
* builder-to-report bridge:
  [text_report_builder_recipe.ns](../../../stdlib/std/text_report_builder_recipe.ns)
  ->
  [report_runtime_recipe.ns](../../../stdlib/std/report_runtime_recipe.ns)

### Workflow Frontdoor Recipes

* [workflow_frontdoor_runtime_recipe.ns](../../../stdlib/std/workflow_frontdoor_runtime_recipe.ns)
* [cli_workflow_automation_recipe.ns](../../../stdlib/std/cli_workflow_automation_recipe.ns)
* [cli_build_pipeline_recipe.ns](../../../stdlib/std/cli_build_pipeline_recipe.ns)
* [cli_project_build_report_recipe.ns](../../../stdlib/std/cli_project_build_report_recipe.ns)
* [cli_compile_workflow_recipe.ns](../../../stdlib/std/cli_compile_workflow_recipe.ns)

### Image Preprocess Companions

* [cli_pgm_info_demo](../../../examples/projects/tooling/cli_pgm_info_demo)
* [cli_pgm_invert_demo](../../../examples/projects/tooling/cli_pgm_invert_demo)
* [cli_pgm_threshold_demo](../../../examples/projects/tooling/cli_pgm_threshold_demo)

`cli_pgm_invert_demo` is now part of the observable std smoke lane: it builds a
real host binary, reads a tiny `P2` PGM file, writes an inverted output file, and
checks the generated image bytes. Treat it as the current bridge between
host-backed std filesystem/text tooling and future PixelMagic image pipelines.

## Current Reading Rule

If you only want one pass:

1. start with [command_runtime_recipe.ns](../../../stdlib/std/command_runtime_recipe.ns)
2. follow into [subprocess_runtime_recipe.ns](../../../stdlib/std/subprocess_runtime_recipe.ns)
3. then [workflow_runtime_recipe.ns](../../../stdlib/std/workflow_runtime_recipe.ns)
4. then one frontdoor recipe:
   [cli_workflow_automation_recipe.ns](../../../stdlib/std/cli_workflow_automation_recipe.ns)
   or
   [cli_compile_workflow_recipe.ns](../../../stdlib/std/cli_compile_workflow_recipe.ns)

## Companion Validation Router

Use [examples/projects/tooling/README.md](../../../examples/projects/tooling/README.md)
as the project-form companion set.

Shortest grouped route:

* [cli_runtime_demo](../../../examples/projects/tooling/cli_runtime_demo)
* [command_runtime_demo](../../../examples/projects/tooling/command_runtime_demo)
* [workflow_runtime_demo](../../../examples/projects/tooling/workflow_runtime_demo)

Current launch-shaped note:

* `cli_runtime_demo` is the shortest checked-in tooling project that should now
  survive the normal native-artifact launch path through `nuis run-artifact`
* `cli_session_demo` and `cli_report_session_demo` should be read as the
  session/report companions that also survive that launch path under the
  current host EOF/non-interactive runtime behavior
* `workflow_runtime_demo` should be read as the workflow-facing companion that
  also survives that same launch path
* `command_runtime_demo` and `subprocess_runtime_demo` should be read as the
  narrow command/process companions that also survive that same launch path

Short rule:

* prefer these three frontdoor project routes first
* only then drop into the broader tooling companion router

## Current Refactor Meaning

For the current `0.20.*` line, this router means:

* the tooling lane is now treated as one owned `std` cluster
* the next source-level cleanup should happen here before the richer net lane
* repeated command/workflow skeletons are now a lane-level cleanup target, not
  just a local style quirk
