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

To select the pure Nsld self-contained image route at the `nuis` frontdoor,
pass `--packaging-mode nuis-self-contained-image` during build:

```bash
cargo run -p nuis -- build \
  --packaging-mode nuis-self-contained-image \
  examples/projects/tooling/native_artifact_closure_demo \
  examples/bins/native_artifact_closure_demo_project
```

That produces a build manifest whose link plan selects
`final_stage_kind = "nuis-self-contained-image"`,
`final_stage_driver = "nsld-internal-image-writer"`,
`final_stage_link_mode = "self-contained"`, and a `.nsb` final output path.
The default build route remains `native-cpu-llvm`, which keeps the current
host-native compatibility path available while Nsld's own binary route matures.

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

When the manifest was built with `--packaging-mode nuis-self-contained-image`,
the same `nsld drive --apply --until-clean` route stays inside Nsld's own
finalizer and materializes the `.nsb` output selected by the link plan. This is
the current protocol-level smoke path for the Nuis-native binary image before a
full standalone `nsld` linker replaces the remaining host-native tail.
Before that drive step, `nuis artifact-doctor --json` and
`nuis run-artifact --json` should report
`self-contained-image-awaiting-nsld-handoff` rather than falling back to the
legacy host binary. After the drive step, they should report
`nsld-host-entrypoint` / `ready` / `entrypoint-ready`.

If you want the CLI to classify the route before you build, use:

```bash
cargo run -p nuis -- status

cargo run -p nuis -- workflow \
  examples/projects/tooling/native_artifact_closure_demo

cargo run -p nuis -- project-status \
  examples/projects/tooling/native_artifact_closure_demo

cargo run -p nuis -- project-doctor \
  examples/projects/tooling/native_artifact_closure_demo
```

Read `status` as the stable top-level transcript sample:

```text
frontdoor_reading_order_contract: nuis-frontdoor-reading-order-v1
frontdoor_reading_order: closure_summary -> dev_tensor_weakest_task_card_handoff
frontdoor_sample_closure_summary: closure_summary_status -> closure_summary_next_action -> closure_summary_next_command
frontdoor_sample_tensor_handoff: dev_tensor_weakest_task_card_coordinate -> dev_tensor_weakest_task_card_handoff_coordinate -> dev_tensor_weakest_task_card_handoff_command
```

That order keeps artifact closure work and tensor-driven mainline planning from
fighting each other: first close the current `closure_summary_*` blocker, then
use `dev_tensor_weakest_task_card_*` and
`dev_tensor_weakest_task_card_handoff_*` to choose the next bootstrap
coordinate.

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
* `link_plan_heterogeneous_backend_artifact_units`
* `link_plan_heterogeneous_backend_artifact_ready_units`
* `link_plan_heterogeneous_domain_readiness_ready`
* `link_plan_heterogeneous_domain_families`
* `link_plan_heterogeneous_backend_families`
* `link_plan_heterogeneous_target_devices`
* `link_plan_heterogeneous_domain_first_unready`
* `link_plan_heterogeneous_backend_artifact_first_unready`
* `workflow_run_artifact_prelaunch_kind`
* `workflow_run_artifact_prelaunch_status`
* `workflow_run_artifact_prelaunch_evidence_status`
* `workflow_run_artifact_prelaunch_command`
* `workflow_run_artifact_prelaunch_reason`
* `workflow_launch_evidence_protocol`
* `workflow_launch_evidence_status`
* `workflow_launch_evidence_route`
* `workflow_launch_evidence_status_code`
* `workflow_launch_evidence_debugger_contract`
* `workflow_launch_evidence_host_runner_probe_status`
* `workflow_launch_evidence_first_blocker`
* `nsld_final_executable_output_ready`
* `nsld_final_executable_output_boundary_status`
* `nsld_final_executable_output_materialization_status`
* `nsld_final_executable_output_execution_handoff_contract`
* `nsld_final_executable_output_execution_handoff_ready`
* `nsld_final_executable_output_execution_handoff_status`
* `nsld_final_executable_output_execution_handoff_target`
* `nsld_final_executable_output_execution_handoff_evidence_status`
* `nsld_final_executable_output_execution_handoff_first_blocker`
* `nsld_final_executable_output_execution_handoff_decision_code`
* `nsld_final_executable_output_payload_execution_trace_protocol`
* `nsld_final_executable_output_payload_execution_trace_available`
* `nsld_final_executable_output_payload_execution_trace_record_count`
* `nsld_final_executable_output_payload_execution_trace_ready_record_count`
* `nsld_final_executable_output_nsdb_replay_contract`
* `nsld_final_executable_output_nsdb_replay_ready`
* `nsld_final_executable_output_nsdb_replay_status`
* `nsld_final_executable_output_nsdb_replay_command`
* `nsld_final_executable_output_nsdb_replay_first_blocker`
* `nsld_final_executable_pipeline_execution_handoff_contract`
* `nsld_final_executable_pipeline_execution_handoff_ready`
* `nsld_final_executable_pipeline_execution_handoff_status`
* `nsld_final_executable_pipeline_execution_handoff_target`
* `nsld_final_executable_pipeline_execution_handoff_evidence_status`
* `nsld_final_executable_pipeline_execution_handoff_first_blocker`
* `nsld_final_executable_pipeline_execution_handoff_decision_code`
* `nsld_final_executable_pipeline_entrypoint_materialization_kind`
* `nsld_final_executable_pipeline_entrypoint_materialization_path`
* `nsld_final_executable_pipeline_entrypoint_materialization_ready`
* `nsld_final_executable_pipeline_entrypoint_materialization_first_blocker`
* `nsld_final_executable_pipeline_entrypoint_materialization_present`
* `nsld_final_executable_pipeline_entrypoint_materialization_hash`
* `nsld_final_executable_pipeline_entrypoint_materialization_runner_command`
* `run_artifact_prelaunch_kind`
* `run_artifact_prelaunch_status`
* `run_artifact_prelaunch_evidence_status`
* `run_artifact_prelaunch_command`
* `run_artifact_prelaunch_runner_command_present`
* `run_artifact_prelaunch_entrypoint_path`
* `run_artifact_prelaunch_entrypoint_present`
* `run_artifact_prelaunch_entrypoint_protocol`
* `run_artifact_prelaunch_entrypoint_protocol_valid`
* `run_artifact_prelaunch_reason`
* `host_runner_invoked`
* `host_runner_status`
* `host_runner_program`
* `host_runner_exit_status`
* `host_runner_error`
* `host_runner_ready`
* `host_runner_would_enter_lifecycle_hook`
* `host_runner_nsb_readable`
* `host_runner_nsb_hash_matches`
* `host_runner_nsb_payload_region_mapped`
* `host_runner_nsb_payload_scan_kind`
* `host_runner_container_loader_status`
* `host_runner_container_ready`
* `host_runner_container_loader_entry_kind`
* `host_runner_container_loader_entry_symbol`
* `host_runner_container_loader_entry_section_id`
* `host_runner_container_loader_handoff_ready`
* `host_runner_container_loader_handoff_status`
* `launch_evidence_protocol`
* `launch_evidence_status`
* `launch_evidence_route`
* `launch_evidence_status_code`
* `launch_evidence_debugger_contract`
* `launch_evidence_command`
* `launch_evidence_host_runner_probe_status`
* `launch_evidence_host_runner_probe_ready`
* `launch_evidence_first_payload_status`
* `launch_evidence_first_payload_ready`
* `launch_evidence_first_payload_target`
* `launch_evidence_first_payload_entry_symbol`
* `launch_evidence_first_payload_entry_kind`
* `launch_evidence_first_payload_entry_section_id`
* `launch_evidence_first_payload_first_blocker`
* `launch_evidence_payload_execution_trace_protocol`
* `launch_evidence_payload_execution_trace_record_count`
* `launch_evidence_first_blocker`
* `launch_evidence_reason`
* `artifact_closure_kind`
* `artifact_closure_status`
* `artifact_closure_evidence_status`
* `artifact_launch_evidence_protocol`
* `artifact_launch_evidence_status`
* `artifact_launch_evidence_route`
* `artifact_launch_evidence_status_code`
* `artifact_launch_evidence_debugger_contract`
* `artifact_launch_evidence_host_runner_probe_status`
* `artifact_launch_evidence_first_blocker`
* `artifact_closure_command`
* `artifact_closure_runner_command_present`
* `artifact_closure_entrypoint_path`
* `artifact_closure_entrypoint_present`
* `artifact_closure_entrypoint_protocol`
* `artifact_closure_entrypoint_protocol_valid`
* `artifact_closure_reason`
* `nsld_final_executable_output_recommended_next_action`
* `nsld_final_executable_output_artifact_chain_safe_next_contract`
* `nsld_final_executable_output_artifact_chain_safe_next_probe_command`
* `nsld_final_executable_output_path_present`
* `nsld_final_executable_output_nsld_owned`
* `nsld_final_executable_output_blockers`
* `nsld_self_owned_image_ready`
* `nsld_self_owned_image_status`
* `nsld_entrypoint_materialization_status`
* `nsld_self_owned_image_path`
* `nsld_self_owned_image_present`
* `nsld_self_owned_image_hash`
* `nsld_self_owned_image_header_valid`

Short reading rule:

* `workflow` tells you the shortest branch for the current input shape
* `project-status` tells you the current project/build surface summary
* `project-doctor` tells you the same route with more preflight/health detail
* `closure_summary_*` is the canonical human closure line shared by
  `workflow`, `project-status`, and `project-doctor`; read it before drilling
  into the detailed Nsld, artifact, runtime, or project-health mirrors
* after the closure summary is understood, `dev_tensor_weakest_task_card_*`
  and `dev_tensor_weakest_task_card_handoff_*` identify the next bootstrap
  coordinate to push; this keeps artifact closure work and tensor-driven
  planning on one reading path
* `artifact-doctor` tells you whether the emitted native bundle is actually
  closed enough to run
* final-output ownership fields tell you whether the visible host-native output
  is missing, merely host-produced, or explicitly Nsld-owned
* `nsld_final_executable_output_boundary_status` is the normalized script-facing
  state for the final-output boundary
* `nsld_final_executable_output_materialization_status` distinguishes host-native
  readiness from the self-contained internal image route
* `nsld_final_executable_output_execution_handoff_contract` versions the
  handoff field group so runner, materializer, and debugger consumers can branch
  on an explicit protocol
* `nsld_final_executable_output_execution_handoff_ready` is the script-friendly
  boolean mirror for whether the verified output boundary can hand off to the
  next execution owner
* `nsld_final_executable_output_nsdb_replay_ready` says whether the persisted
  final-output nsdb handoff has passed `nsdb replay-plan` and can be consumed by
  `nsdb replay`; the matching command enters deterministic YIR transcript
  consumption, not native instruction execution
* `nsld_final_executable_output_execution_handoff_status` distinguishes whether
  the output can hand off directly to a runner, still needs entrypoint
  materialization, or is blocked
* `nsld_final_executable_output_execution_handoff_target` names the abstract
  component that owns that handoff without binding the artifact to Mach-O, ELF,
  PE, or a future Nuis-native shell
* `nsld_final_executable_output_execution_handoff_evidence_status` names the
  proof class backing the handoff, such as the host invoke plan or the internal
  image header/hash evidence
* `nsld_final_executable_output_execution_handoff_first_blocker` mirrors the
  first blocker that prevents that handoff, so scripts do not need to parse the
  full blocker list for the common branch
* `nsld_final_executable_output_execution_handoff_decision_code` is the compact
  branch code for CI, nsdb, and future runner/materializer routing
* `nsld_final_executable_output_entrypoint_materialization_evidence_status`
  reports whether a ready self-contained output still lacks launcher evidence,
  has a ready launcher manifest, or has a launcher dry-run that would enter the
  lifecycle hook
* `nsld_final_executable_output_launcher_manifest_*` and
  `nsld_final_executable_output_launcher_dry_run_*` mirror the materialized
  launcher evidence from the final-output boundary, so scripts can distinguish
  "output ready" from "entrypoint handoff evidence ready"
* Nsld launcher manifest and launcher dry-run artifacts preserve the same
  `nsld-final-output-handoff-v1` decision group instead of inventing a second
  launch-readiness model
* the final-executable pipeline summary preserves that same handoff group so
  automation can route from the pipeline report first
* the `nsld_final_executable_pipeline_execution_handoff_*` fields are the
  `nuis` frontdoor mirror of that pipeline route; they are `null` until the
  pipeline artifact exists
* `nsld_final_executable_output_recommended_next_action` gives scripts the next
  boundary action without forcing them to interpret every blocker string
* `nsld_final_executable_output_artifact_chain_safe_next_contract` and
  `nsld_final_executable_output_artifact_chain_safe_next_probe_command` let
  object-package and replay consumers return to the shared
  `nsld-drive-safe-next-v1` probe before mutating linker state; replay itself
  remains read-only and does not acquire linker transition semantics
* `nsld_self_owned_image_status` is the normalized script-facing state for the
  internal `.nsb` image layer before host-shell or OS-native materialization
* `nsld_entrypoint_materialization_status` separates the next entrypoint layer
  from image readiness: `host-launcher-ready`,
  `image-ready-entrypoint-pending`, or `blocked`
* on the current ready self-contained route, `nuis.host-entrypoint.sh` is a
  generated host-runner handoff stub, not an OS package or embedded runner; the
  pipeline exposes its presence, hash, and runner command for automation, and
  verifier/check reports fail if the stub is deleted or its content no longer
  matches the emitted pipeline snapshot
* that host entrypoint stub declares
  `NUIS_HOST_ENTRYPOINT_STUB_PROTOCOL=nuis-nsld-host-entrypoint-v1` and exports
  it before delegating to `NUIS_HOST_RUNNER`, so future runner, debugger, and
  bundler layers can recognize the stub as an Nsld protocol artifact rather
  than treating it as an anonymous shell script
* `run-artifact` treats the protocol marker as part of the entrypoint closure:
  a reported stub path that exists but does not declare/export
  `nuis-nsld-host-entrypoint-v1` is still `blocked`, preventing an arbitrary
  host shell script from being mistaken for a verified Nsld entrypoint
* when the legacy host binary is absent or intentionally ignored by the
  self-contained route, a verified Nsld host-entrypoint makes non-JSON
  `run-artifact` invoke `nuis-host-runner` instead of falling back to the older
  binary-only path. The runner validates the image handoff and reports that it
  would enter the lifecycle hook; this is still a loader/handoff step, not a
  claim that payload execution has completed.
* `nuis-host-runner` is the first thin runtime-side consumer for that handoff:
  it verifies the launcher manifest, `.nsb` path/header/hash, scheduler entry,
  and lifecycle hook before reporting that it would enter the lifecycle hook.
  Its report also exposes the parsed `.nsb` payload offset/span, mapped payload
  region byte count/hash, plus layout and byte-map hashes, giving the future
  runtime loop a concrete payload region to scan rather than treating the image
  as an opaque hashed blob. The first scanner layer reports payload scan status,
  a coarse payload kind such as `nsld-container-toml` / `toml-like` /
  `opaque-bytes`, and bounded hex/text prefixes for diagnostics. When the
  payload is an Nsld container TOML or TOML-like candidate, the runner also
  extracts the current container contract fields: schema/version,
  container-kind, producer, `ready`, top-level `blockers`, magic/version, and
  payload size/hash metadata, including the declared payload path plus the
  container/table hashes that are useful to nsdb and CI diagnostics. The runner
  reports those hashes but deliberately does not recompute them; deep hash
  verification remains the job of `nsld check` and the container verifier. It
  then extracts loader summary fields: readiness, declared loader blockers,
  entry kind, entry symbol, entry section id, and loader symbol count. It checks
  `section_count` against parsed `[[section]]` rows and requires the loader
  entry section to exist in that table. It reads the first `[[loader_symbol]]`
  row and checks that the bootstrap symbol kind, symbol name, and section match
  the loader entry summary. It also checks `relocation_count` and verifies that
  the first `[[relocation]]` binds the entry section to the first loader symbol.
  Compatibility-domain and external-import counts are checked as container
  tables; any required `[[external_import]]` becomes a runner blocker because
  the handoff still depends on host-side compatibility. Those container and
  loader fields now
  participate in runner readiness: a blocked container/loader, non-empty
  `blockers` or `loader_blockers`, required external imports, unsupported
  schema/version/kind/producer/magic/version, missing payload metadata, missing
  or mismatched section/relocation/compat/import table, missing entry
  kind/symbol/section, empty loader-symbol table, missing bootstrap row, or
  mismatched first loader symbol/relocation prevents the host handoff from being
  reported as ready
* `run-artifact --json` additionally emits the `run_artifact_prelaunch_*`
  aggregate fields so scripts can choose between a verified Nsld host
  entrypoint and the older host-binary launch path without re-interpreting every
  lower-level Nsld field. The group includes a compact evidence status
  (`host-binary-ready`, `entrypoint-ready`, `entrypoint-missing`,
  `entrypoint-protocol-invalid`,
  `self-contained-image-awaiting-nsld-handoff`, or `no-launch-surface`),
  runner-command presence, entrypoint presence, expected entrypoint protocol,
  and protocol-validity fields; if the pipeline snapshot claims an entrypoint
  but the stub is missing on disk, the aggregate prelaunch status is `blocked`
  instead of silently falling back to a different launch surface. A
  self-contained `.nsb` route is also blocked until Nsld materializes a verified
  runtime handoff, even if a legacy host artifact exists in the output folder.
  After `nsld drive --apply --until-clean` materializes that handoff, the same
  aggregate fields should move to `nsld-host-entrypoint` / `ready` /
  `entrypoint-ready`.
* on that ready self-contained route, `run-artifact --json` also performs a
  non-fatal `nuis-host-runner` probe and emits `host_runner_*` fields. These
  fields tell automation whether the runner was invoked, which runner program
  was selected, whether the runner itself reported ready, whether the `.nsb`
  was readable, whether the expected image hash matched, whether the payload
  region was mapped, and what coarse payload scan/container-loader boundary was
  observed. Runner probe failure is surfaced as `host_runner_status =
  "unavailable"` or `failed`; JSON classification still remains an inspection
  surface rather than a hard launch command.
* when payload execution trace records are available, `run-artifact` persists a
  debugger handoff file named `nuis.nsdb.payload-execution-handoff.toml` in the
  artifact output directory. The file uses
  `nuis-nsdb-payload-execution-handoff-v1`, mirrors the
  `nsdb-yir-payload-execution-trace-v1` debugger contract, and records the
  first container-loader handoff trace so future nsdb tooling can consume YIR
  payload execution metadata without re-running the host probe. Passive
  frontdoors such as `artifact-doctor --json` and `workflow --json` read the
  same file back as `artifact_nsdb_handoff_*` fields plus
  `workflow_nsdb_handoff_available`, `workflow_nsdb_handoff_protocol`, and
  `workflow_nsdb_handoff_record_count`, keeping debugger handoff state
  observable without forcing a runtime launch. `nsdb inspect` consumes that
  same file as `payload_execution_handoff_*` fields, validating the
  `nuis-nsdb-payload-execution-handoff-v1` /
  `nsdb-yir-payload-execution-trace-v1` pair before treating the first
  container-loader handoff as debugger metadata.
* when heterogeneous runtime trace records are available, `run-artifact` also
  persists `nuis.nsdb.hetero-runtime-trace.toml` beside the build manifest. The
  file uses `nuis-nsdb-hetero-runtime-trace-v1`, mirrors the
  `nsdb-yir-hetero-runtime-trace-v1` debugger contract, and records domain
  metadata plus backend-artifact trace records so nsdb value-sample resolution
  can read runtime/device trace metadata without scraping JSON output.
* the current self-contained smoke proves the host-runner image boundary:
  `.nsb` readable, image hash matched, payload region mapped, and lifecycle
  hook handoff ready. The payload scanner now sees `nsld-container-toml`, the
  runner parses the container metadata prefix before the binary payload region,
  and host-assisted external-import declarations no longer block the host-runner
  handoff. They remain visible as compatibility evidence rather than being
  treated as a pure self-contained closure.
* `workflow` and LinkPlan JSON mirror that decision under
  `workflow_run_artifact_prelaunch_*`, so the main workflow surface can show the
  launch closure that `run-artifact` would prefer without forcing callers to run
  a second command just to classify the final handoff. It also emits
  `workflow_launch_evidence_*` as a non-executing mirror of the same nsdb-facing
  launch evidence contract; host-runner probe fields are intentionally marked as
  `workflow-mirror` until `run-artifact` performs the real probe
* `artifact-doctor --json` emits the matching `artifact_closure_*` aggregate
  fields. These describe the current runnable artifact closure before execution:
  `host-binary` for the older direct binary path, `nsld-host-entrypoint` for the
  self-contained Nsld entrypoint route, or `none` when no launch surface is
  available yet. The closure group mirrors evidence status, runner-command
  presence, entrypoint presence, expected entrypoint protocol, and protocol
  validity. It also emits `artifact_launch_evidence_*` as the artifact-doctor
  mirror of the nsdb-facing evidence contract, with host-runner probe fields
  marked as `artifact-doctor-mirror` until runtime handoff
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
* launcher metadata now carries the final-output handoff contract through to
  the non-executing launch preflight layer
* final-executable pipeline metadata carries the same handoff contract as its
  top-level route summary
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
  mutating the build directory, and now falls through to a read-only
  `final-output-boundary` action when the artifact chain is otherwise clean but
  the final executable output still needs diagnosis
* `nsld drive --apply --until-clean` can materialize the current whitelisted
  Nsld artifact chain; host-assisted finalization blockers are carried by the
  emitted final pipeline/output metadata instead of by repeating the drive step
* `nuis build --packaging-mode nuis-self-contained-image` followed by
  `nsld drive --apply --until-clean` can materialize the current pure Nsld
  `.nsb` image route from a manifest-selected link plan
* the compiled artifact and manifest both survive verifier checks
* host-native outputs can still launch directly through the `nuis` frontdoor
* self-contained `.nsb` outputs can move from Nsld-drive-required to verified
  `nsld-host-entrypoint` handoff through the same `nuis` frontdoor surfaces

## Current Checked-In Gates

The current checked-in coverage is split deliberately:

* checked-in project compile anchor:
  [tooling_compile.rs](../../tools/nuisc/tests/tooling_compile.rs)
* AOT compile/package/launch smoke:
  [lib.rs](../../tools/nuisc/src/lib.rs)
* representative native control-flow compile/launch smoke:
  [artifact_cli.rs](../../tools/nuisc/tests/artifact_cli.rs)
* self-contained Nsld image handoff smoke:
  [self_contained_nsb_smoke.rs](../../tools/nuis/tests/self_contained_nsb_smoke.rs)

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
