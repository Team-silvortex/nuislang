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
* `nsld_final_executable_output_ready`
  whether the visible final executable output is currently Nsld-owned and has
  no lightweight final-output boundary blockers. This is intentionally
  narrower than `ready_to_run`, which can still describe the current
  host-toolchain AOT binary launch path.
* `nsld_final_executable_output_boundary_status`
  the normalized final-output boundary state for scripts and release gates:
  `ready`, `missing`, `not-nsld-owned`, `unreadable`, or `invalid`
* `nsld_final_executable_output_materialization_status`
  the normalized materialization layer for the visible output:
  `host-native-ready`, `self-contained-image-ready`, or `blocked`
* `nsld_final_executable_output_execution_handoff_contract`
  the contract family/version for the handoff field group, currently
  `nsld-final-output-handoff-v1`
* `nsld_final_executable_output_execution_handoff_ready`
  whether the current final output has enough verified boundary evidence to
  hand off to its next execution owner
* `nsld_final_executable_output_execution_handoff_status`
  the normalized execution handoff layer for the visible output:
  `runner-ready`, `entrypoint-materializer-required`, or `blocked`
* `nsld_final_executable_output_execution_handoff_target`
  the abstract component that owns the next execution handoff:
  `host-runner`, `entrypoint-materializer`, or `none`
* `nsld_final_executable_output_execution_handoff_evidence_status`
  the normalized evidence source for that handoff:
  `host-invoke-plan-ready`, `image-header-and-hash-ready`, or `blocked`
* `nsld_final_executable_output_execution_handoff_first_blocker`
  the first machine-readable blocker preventing execution handoff, or `null`
  when the handoff gate is ready
* `nsld_final_executable_output_execution_handoff_decision_code`
  the compact machine branch for CI and future debugger/linker routing, such as
  `handoff-host-runner`, `handoff-entrypoint-materializer`,
  `emit-final-executable`, or `inspect-output-diagnostics`
  Nsld launcher manifest and launcher dry-run artifacts carry the same
  `nsld-final-output-handoff-v1` decision group forward; they do not define a
  separate launch-readiness protocol. The final-executable pipeline summary
  also carries the same group for top-level automation routing.
* `nsld_final_executable_pipeline_execution_handoff_contract`
  pipeline-level mirror of the same handoff contract, or `null` when the
  pipeline TOML is not present
* `nsld_final_executable_pipeline_execution_handoff_ready`
  pipeline-level handoff gate copied from the final-executable pipeline summary
* `nsld_final_executable_pipeline_execution_handoff_status`
  pipeline-level handoff state for routing without opening launcher TOML
* `nsld_final_executable_pipeline_execution_handoff_target`
  pipeline-level target owner such as `host-runner` or `entrypoint-materializer`
* `nsld_final_executable_pipeline_execution_handoff_evidence_status`
  pipeline-level proof class backing that route
* `nsld_final_executable_pipeline_execution_handoff_first_blocker`
  first pipeline-level blocker for that handoff, or `null` / `<none>`
* `nsld_final_executable_pipeline_execution_handoff_decision_code`
  compact pipeline-level route code for CI and future runner/materializer tools
* `nsld_final_executable_pipeline_entrypoint_materialization_kind`
  pipeline-level materializer plan kind, currently `host-shell-entrypoint-plan`
  when the self-contained image route can hand off to a host entrypoint plan
* `nsld_final_executable_pipeline_entrypoint_materialization_path`
  planned materializer output path, or `null` until the pipeline TOML is present
* `nsld_final_executable_pipeline_entrypoint_materialization_ready`
  script-friendly gate for whether the entrypoint materialization plan is ready
* `nsld_final_executable_pipeline_entrypoint_materialization_first_blocker`
  first machine-readable blocker for that plan, or `null` / `<none>` when ready
* `nsld_final_executable_pipeline_entrypoint_materialization_present`
  whether the planned host entrypoint artifact is present on disk
* `nsld_final_executable_pipeline_entrypoint_materialization_hash`
  content hash for the generated host entrypoint handoff stub, or `null`
* `nsld_final_executable_pipeline_entrypoint_materialization_runner_command`
  script-facing summary of the host-runner handoff command
* `nsld_final_executable_output_recommended_next_action`
  the script-facing next action for the current boundary, such as
  `emit-final-executable-pipeline`,
  `materialize-host-shell-or-os-entrypoint`, or `handoff-to-runner`
* `nsld_final_executable_output_path_present`
  whether the current final-stage output path exists on disk
* `nsld_final_executable_output_nsld_owned`
  the lightweight `nuis` mirror of final output ownership when
  `nuis.nsld.final-executable.blocked.toml` exposes an `emitted` value; `null`
  means the frontdoor is not guessing ownership
* `nsld_final_executable_output_blocker_count`
  the number of lightweight final-output boundary blockers visible from the
  `nuis` frontdoor
* `nsld_final_executable_output_blockers`
  lightweight final-output boundary blockers such as
  `final-executable-output:missing`,
  `final-executable-output:ownership-unknown`, or
  `final-executable-output:not-nsld-owned`; `nsld check` remains the
  authoritative deep verifier. `ownership-unknown` means the path may already
  exist as a host/compiler output, but the `nuis` frontdoor has not seen Nsld
  emitted metadata proving ownership.
  Text output mirrors this array with repeated
  `nsld_final_executable_output_blocker` lines.
  `nuis release-check` prints the same boundary in its `release-check:
  nsld-drive` block with the shorter `final_executable_output_*` field prefix.
* `nsld_self_owned_image_ready`
  whether the current Nsld launcher manifest exposes a present `.nsb` image
  with a valid Nuis image header
* `nsld_self_owned_image_status`
  the normalized self-owned image state for scripts: `ready`,
  `manifest-missing`, `path-missing`, `missing`, `header-invalid`,
  `hash-missing`, or `unknown`
* `nsld_entrypoint_materialization_status`
  the normalized next entrypoint layer state derived from the Nsld final
  executable pipeline: `host-launcher-ready`,
  `image-ready-entrypoint-pending`, or `blocked`
* `nsld_self_owned_image_path`
  the self-owned `.nsb` image path from the Nsld launcher manifest when
  available
* `nsld_self_owned_image_present`
  whether the launcher manifest sees the `.nsb` image bytes on disk
* `nsld_self_owned_image_hash`
  the launcher-manifest hash for the self-owned `.nsb` image when available
* `nsld_self_owned_image_header_valid`
  whether the `.nsb` image header validates under the current Nsld image
  protocol
* `nsld_final_executable_output_first_blocker`
  the first lightweight final-output boundary blocker, or `null` / `<none>`
  when no blocker is currently visible

Short rule:

`these fields do not mean nuis already owns a finished self-hosted linker; they mean the current final-link route is now visible as a first-class modeled surface`

Boundary rule:

`missing` means there is no final-stage output path on disk; `ownership-unknown`
means a path exists but no Nsld emitted marker has been observed by the
frontdoor; `not-nsld-owned` means Nsld has explicitly reported that the visible
host-native output exists outside Nsld ownership.

Next-action rule:

`nsld_next_action` only reports `ready` when the prepared chain, final
executable tail, and final output boundary are all ready. If the tail is ready
but the final output boundary is not, the frontdoor recommends
`nsld final-executable-output <manifest>` instead of treating host-native launch
readiness as Nsld-owned final-output readiness.
In that state, `nsld_drive_recommended_mode` can still be `dry-run`: that means
there is no remaining mutating artifact-chain action, not that the final output
boundary is complete.

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
