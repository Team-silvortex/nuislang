# `nuis` Frontdoor Surface Reference

This file is the short current reference for the user-visible frontdoor fields
emitted by `nuis`.

It is intentionally narrower than the full CLI help and narrower than the
versioned workflow docs.

It answers one narrow question:

`which frontdoor fields are stable enough to treat as the current machine-readable workflow/artifact surface?`

## Scope

This reference only covers the current grouped frontdoor family:

* `nuis workflow`
* `nuis project-status`
* `nuis project-doctor`
* `nuis artifact-doctor`

It does not attempt to freeze:

* every diagnostic line
* every domain-specific helper field
* future launcher/container/linker architecture

## Shared Frontdoor Fields

These fields now form the current common frontdoor spine:

* `source_kind`
  classifies the input shape such as single-file, project, or output-dir-like
  artifact input
* `workflow_kind`
  names the current workflow branch such as `compile_workflow` or
  `project_compile_workflow`
* `workflow_brief`
  gives the shortest ordered route for that branch
* `workflow_samples`
  gives the shortest command-shaped examples for that branch
* `recommended_next_step`
  says what the CLI believes the next command should be right now
* `recommended_command`
  gives that next command as a concrete command string
* `recommended_reason`
  explains why that route is being recommended

Short rule:

`these fields tell you what branch you are on and what the CLI thinks should happen next`

## Artifact Closure Fields

These fields now form the current artifact-follow-up spine:

* `artifact_output_dir`
  the default or resolved output directory being discussed
* `artifact_ready_to_run`
  whether the visible artifact closure is currently complete enough to launch
* `artifact_recommended_next_step`
  the artifact-side next action such as `build`, `inspect_artifact`, or
  `run_artifact`
* `artifact_recommended_command`
  the concrete command string for that next action

Short rule:

`these fields answer whether the current build closure is missing, partial, inspectable, or launchable`

## LinkPlan Fields

These fields now form the current visible link summary:

* `link_plan_available`
  whether a `LinkPlan` could be recovered from the current build manifest
* `link_plan_final_stage`
  the current final-stage kind such as `host-native-link`
* `link_plan_final_driver`
  the current final-stage driver such as `clang`
* `link_plan_final_link_mode`
  the current link-mode summary such as `host-toolchain-finalize`
* `link_plan_final_output`
  the current final output path when available
* `link_plan_domain_units`
  the number of domain build units carried by the current build plan

Short rule:

`these fields do not mean nuis already owns a finished self-hosted linker; they mean the current final-link route is now visible as a first-class modeled surface`

## Command Surface By Entry Point

### `nuis workflow`

Current purpose:

* classify input shape first
* restate the shortest branch
* expose default build/release output paths
* expose current artifact-follow-up and link-plan summary when available

### `nuis project-status`

Current purpose:

* summarize project structure and public surface
* summarize current build-follow-up state
* expose current artifact closure and link-plan summary

### `nuis project-doctor`

Current purpose:

* summarize project health and next steps
* expose validation/preflight status
* expose current artifact closure and link-plan summary

### `nuis artifact-doctor`

Current purpose:

* summarize the emitted build closure directly
* answer whether manifest/artifact/binary are all visible and verified
* recommend the next artifact-facing command

## Current Honesty Boundary

The current line should be described carefully:

* these frontdoor fields are current implementation truth
* they are already suitable for repo-local tooling, documentation, and
  self-hosting direction
* they are not yet a promise of frozen long-term public schema
* the current native CPU final link is still host-toolchain-backed

Short rule:

`treat this as the current stable reading surface, not as the final forever CLI schema`
