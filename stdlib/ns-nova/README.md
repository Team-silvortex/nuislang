# `ns-nova`

`ns-nova` is the third major standard-library module of `nuis`.

Its ecosystem role is the Nuis counterpart to Unreal Engine: a native real-time
world engine that turns heterogeneous execution, data-plane orchestration, and
inline shader capability into applications, simulations, spatial interfaces,
and games. This describes its architectural role, not API or asset compatibility
with Unreal Engine.

Target character:

* native GPU cross-platform 2D/3D real-time world engine
* world loop, scene, rendering, control/input, ML, audio workflows, physics,
  and resource orchestration rather than only a widget kit
* built on `nuis` domain composition:
  `cpu` for orchestration,
  `data` for exchange,
  `shader` for rendering,
  `kernel` for model and compute-heavy scene or simulation workflows;
  concrete device placement remains a registered capability, not an engine assumption

Intended scope:

* renderer and scene/frame orchestration
* material/pipeline/shader packaging helpers
* window/input/frame lifecycle abstractions
* 2D UI, 3D scene, and game-style application driving built on the same GPU-native core
* ML task/result integration with independent WitSage algorithms and registered providers
* audio capture/playback, processing and mixing workflows with independent processing/device modules
* cross-workflow data, time and resource coordination in one Nuis-owned application,
  not a collection of isolated rendering, ML and audio demos

These are target capabilities, not present API or execution claims. Shared YIR,
GLM and clock/lifecycle contracts allow different event, frame, inference and
audio rates; they do not require frame lockstep. Audio integration and ML-driven
application behavior are still planned. See the
[engine capability horizon](../../docs/versioning/nuis-beta-0.11-application-led-mainline.md#engine-capability-horizon)
for ownership boundaries and combined acceptance without expanding the current
image-session prerequisite.

Family structure:

* `ns-nova-core`
  shared render/runtime skeleton such as theme, surface, viewport, layer, and frame-facing contracts
* `ns-nova-ui`
  UI/widget/control framework built on top of the core render skeleton
* `ns-nova-scene`
  future 2D/3D scene/render-world framework built on the same core

Design principles:

* runtime-native and GPU-first, not a thin wrapper around software preview paths
* powered by `nuis` inline shader and heterogenous graph abilities rather than hiding them completely
* should remain mod-aware and ABI-aware so that packaged `nustar` capabilities stay visible

## Application-Led Mainline

The current mainline uses ns-nova to drive one growing interactive image
application, then foundation hardening, measured performance and incremental
compiler-module migration. The first open prerequisite is hardening the bounded
Nuis-owned session across host events, not whole-module timer replay.
See the [roadmap](../../docs/versioning/nuis-beta-0.11-application-led-mainline.md)
and [separate acceptance coordinates](../../docs/reference/nuis-development-tensor-mainline.md).
Bounded Metal output is established; engine maturity and compiler self-hosting
are not inferred from that evidence.

Current state:

* this repository now treats `ns-nova` as a standard-library/framework layer target, not as a separate future repository by default
* `lib/app_runtime.ns` now owns the first reusable application and frame lifecycle in Nuis source
* `examples/projects/domains/ns_nova_showcase` composes that lifecycle with PixelMagic, Data, and Shader as separate Galaxy/Nustar owners
* [ns_nova_image_showcase](../../examples/projects/domains/ns_nova_image_showcase)
  extends the same lifecycle with Nuis-generated image snapshots and real Metal
  RGB inversion. PixelMagic owns the image algorithm; shared IPC carries bounded
  immutable bytes, while the registered Shader adapter owns GPU resource admission.
  `nuis run-artifact --export-frame` now drives both examples through their compiled
  host binary, not a test child. The native shell embeds the YIR lifecycle runtime;
  fully native CPU lowering and self-contained provider injection remain separate
  milestones. Explicit `--window-session window` now retains Nuis state and one
  provider across AppKit events; the no-option legacy preview is not migrated
* [window profile v3](../../docs/reference/nuis-yir-window-session-v3.md) passes
  `NovaCloseReason` and `NovaFailureKind` to Nuis cleanup. `close_with_failure`
  preserves failed status, the first fault and accepted frame identity; the host distinguishes completed cleanup from
  successful execution. Compiled provider-failure injection retains the error
  and old replay files. Version 1/2 window bundles, state artifacts and close helpers require rebuilding
* [provider IPC v4](../../docs/reference/nuis-yir-provider-runtime-ipc-v4.md) admits
  rejection phase/sequence/code before projecting failure-v2 categories into
  Nuis. `ProviderBudget`, `ProviderExecution` and `ProviderFinalization` preserve
  producer boundaries without guessing device causes. IPC-v3 peers require
  rebuilding together; the window-v3 close signature is unchanged
* [terminal outcome v1](../../docs/reference/nuis-yir-application-outcome-v1.md)
  adds `NovaAppOutcome`, `terminal_outcome`, `outcome_succeeded` and `outcome_failed`.
  The Nuis decoder separates late Finish failure from completed cleanup, rejects
  contradictory/unknown fields and never rewrites the old application state.
  Runtime snapshots are readonly. Explicit owned delivery invokes an already
  running parent's Nuis event once under scoped executor fuel; it does not rerun
  global initialization or transfer child resources. Explicit
  [packaged parent wiring](../../docs/reference/nuis-yir-application-outcome-pump-v1.md)
  now initializes a registered CPU parent before the child and delivers once on
  a separate worker. Root-scoped globals avoid unrelated device initialization;
  native parent dispatch and general multi-child orchestration remain open
* lifecycle-gated `cpu_present_frame` now lowers through the generic registered branch-effect contract; ns-nova adds no compiler branch of its own
* Data, Shader, Kernel, and Network observers now share one YIR result-state projection into CPU CFG; absent provider payloads remain explicitly deferred
* the showcase owns a bounded three-frame loop in Nuis source and passes each
  frame through `NovaFrameResultHandle` before submission and conditional presentation
* that loop carries and rebinds one aggregate `NovaAppState` across all three
  frame-helper calls through the generic recursive-scalar aggregate backedge ABI
* Shader observe issues the shared YIR provider token/clock/root receipt; the
  runtime validates and preserves that identity without synthesizing its own token
* a pure-Nuis kernel receipt test crosses NIR, YIR, LLVM, system linking, and
  final native execution while using the same family-neutral contract
* the showcase now carries checked `galaxy.toml` and `ns-nova.toml` manifests whose package inputs are project-owned relative paths; Galaxy dependencies remain registry-resolved rather than embedded as host paths
* `nuis galaxy init --framework ns-nova` now emits an `ns-nova.toml` profile that carries framework-level assembly metadata, including the standard `ns-nova-selection-v1` selection contract for relational controls such as `list`, `table`, `tree`, `inspector`, and `outline`
* `ns-nova.toml` now also carries `ns-nova-family-v1` and `ns-nova-render-v1` scaffolding so projects can declare whether they currently lean toward `core`, `ui`, or future `scene` layers
* host ABI selection remains project-driven and automatic; the framework source does not select Metal, Vulkan, or a host OS

Current source-asset status:

* this is currently the only `stdlib` layer that already declares a canonical
  checked-in source set through
  [module.toml](module.toml)
* the initial score-oriented project library module is
  [lib/nova_contracts.ns](lib/nova_contracts.ns)
* the first executable lifecycle module is
  [lib/app_runtime.ns](lib/app_runtime.ns), which exposes owned
  `NovaAppState` and `NovaFrameTransaction` transitions plus `NovaCloseReason` and
  `NovaFailureKind`; the first observed fault survives subsequent cleanup
* both library modules currently use `library_import_policy = "manual-only"`
  so it is declared and discoverable through project metadata, but it is not
  auto-injected into project scope by default
* projects may still opt into it explicitly through
  `galaxy_imports = ["ns-nova:lib/app_runtime.ns"]`
* duplicate `galaxy_imports` entries are rejected during manifest loading, so
  this opt-in should be listed at most once
* that manifest currently lists `11` source modules
* `nuis` smoke tests and `project-doctor` now both inspect that asset set

See metadata:

* [module.toml](module.toml)
* [core/README.md](core/README.md)
* [ui/README.md](ui/README.md)
* [scene/README.md](scene/README.md)

First source modules:

* [core/theme_surface.ns](core/theme_surface.ns)
* [core/frame_runtime.ns](core/frame_runtime.ns)
* [core/texture_resource_recipe.ns](core/texture_resource_recipe.ns)
* [core/window_controls_runtime_recipe.ns](core/window_controls_runtime_recipe.ns)
* [ui/panel_selection.ns](ui/panel_selection.ns)
* [ui/panel_blueprint.ns](ui/panel_blueprint.ns)
* [ui/window_controls_recipe.ns](ui/window_controls_recipe.ns)
* [scene/scene_runtime.ns](scene/scene_runtime.ns)
* [scene/efficiency_runtime.ns](scene/efficiency_runtime.ns)
* [scene/scene_blueprint.ns](scene/scene_blueprint.ns)
* [scene/window_controls_scene_recipe.ns](scene/window_controls_scene_recipe.ns)

Current limitation:

* the native three-frame validation loop and the persistent embedded-YIR window
  are distinct routes. The latter remains bounded to 256 dispatches and 64 MiB
  replay, not a stable unlimited interactive world loop. Failure teardown is
  tested, but device-specific causes, general multi-child parent orchestration, cancellation
  and owned-resource retirement remain open
* conditional `cpu_present_frame` now consumes the Shader-derived
  `submitted.present_requested` predicate through a runtime-owned result handle;
  its receipt is provider-domain-issued, but its clock still comes from the planned
  frame deadline rather than a live post-dispatch renderer completion fence
* YIR text round-tripping preserves the aggregate loop result as an explicit output edge, so the checked window AOT graph remains acyclic
* the framework does not yet own a renderer; PixelMagic remains an independent
  project-level composition dependency in the showcase
* the current checked AOT window shell is the replaceable Apple bootstrap adapter;
  non-Apple window adapters and backend execution closure remain open

The machine-readable honesty boundary is
[nuis-ns-nova-application-lifecycle-v1.toml](../../docs/reference/nuis-ns-nova-application-lifecycle-v1.toml).

## First Executable Slice

The current shortest route is:

* [examples/projects/domains/ns_nova_showcase](../../examples/projects/domains/ns_nova_showcase)

It proves:

* explicit import of the reusable Nova lifecycle
* a bounded Nuis-owned update loop lowered to native LLVM control flow
* recursive scalar aggregate application state carried across every loop backedge
* typed frame-result capture, validation, submission, and conditional presentation
* provider-issued token/clock/root identity preserved into carried application state
* independent PixelMagic shader ownership
* registered Data transfer through `FabricPlane`
* automatic CPU, Data, and Shader ABI recommendation
* project compilation and Apple arm64 window AOT packaging

## Relationship To `window_controls_demo`

The older broad stress route remains:

* [examples/projects/window_controls_demo](../../examples/projects/window_controls_demo)

That project is not obsolete. It remains the detailed control/scene stress
fixture, while `ns_nova_showcase` is now the smaller framework front door.

The current migration split is:

* already extracted into stdlib recipes
  - render/runtime orchestration patterns:
    [core/texture_resource_recipe.ns](core/texture_resource_recipe.ns),
    [core/window_controls_runtime_recipe.ns](core/window_controls_runtime_recipe.ns)
  - UI/selection/control packing patterns:
    [ui/window_controls_recipe.ns](ui/window_controls_recipe.ns)
  - scene/runtime efficiency and assembly patterns:
    [scene/window_controls_scene_recipe.ns](scene/window_controls_scene_recipe.ns)
* still intentionally left in the project demo
  - full multi-domain assembly in one realistic project
  - host/window integration details
  - exact demo-oriented packet mixes used to pressure-test current lowering and
    shader fallback behavior

So the rule of thumb is:

* read `examples/projects/domains/ns_nova_showcase` for the current shortest workflow
* read `examples/projects/window_controls_demo` for the broad stress workflow
* read `stdlib/ns-nova/*recipe.ns` for the pieces that have already become
  reusable source assets
