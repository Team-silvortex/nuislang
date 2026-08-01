# `PixelMagic`

`PixelMagic` is an official `Galaxy` in the `nuis` standard-library family.

Its role is to hold the image-processing and texture-resource side of the
heterogeneous stack without forcing those semantics into `ns-nova` itself.

Target character:

* GPU-oriented image-processing package
* texture/resource handoff layer between host-side preprocess work and shader-facing consumption
* future home for image packet, image resource, and shader-ready sampling preparation contracts

Intended scope:

* host-side image description shaping
* narrow image packet/resource contracts
* texture/resource lowering helpers that feed shader-facing consumers
* render-plan summaries that connect filter chains, image analysis, texture
  handoff, and shader-facing consumption
* future filter/transform/image-kernel families once the frontdoor is stable

Relationship:

* `core`
  smallest semantic base
* `std`
  host/runtime helpers and preprocess scaffolding
* `pixelmagic`
  image/resource Galaxy built on top of `core + std`
* `ns-nova`
  GUI/render Galaxy that may consume `PixelMagic` contracts without becoming the image package itself

Current source-asset status:

* `PixelMagic` is now a checked-in stdlib package skeleton through
  [module.toml](module.toml)
* the current first auto-injectable library module is
  [lib/image_contracts.ns](lib/image_contracts.ns)
  which exposes a small `PixelMagicContracts` helper surface for project-level `galaxy = ["pixelmagic=workspace"]` resolution
* `PixelMagic` now also exposes an auto-injectable shader-side library module
  through
  [lib/shader_contracts.ns](lib/shader_contracts.ns),
  so project galaxy resolution can surface a canonical `PixelMagicSurfaceContracts`
  shader profile alongside the CPU helper layer
* `PixelMagic` now also ships checked-in official shader demo surfaces through
  [lib/packet_bridge_surface.ns](lib/packet_bridge_surface.ns),
  [lib/render_surface.ns](lib/render_surface.ns),
  [lib/texture_surface.ns](lib/texture_surface.ns),
  and
  [lib/pipeline_surface.ns](lib/pipeline_surface.ns),
  so the domain demos can consume stdlib-owned shader profiles instead of
  carrying project-local `surface_shader.ns` copies
* that helper surface now covers both image-op packet shaping and the first
  shader-facing packet / consumer / pipeline scoring helpers, so projects can
  depend on one stable auto-injected entry point while deeper recipe modules
  continue to evolve
* that helper surface now also covers compact filter-chain, analysis-quality,
  and texture-handoff summaries, so examples can express a fuller
  CPU-to-shader image pipeline through one stable `PixelMagicContracts` module
* the current first canonical source assets are
  [core/image_packet_recipe.ns](core/image_packet_recipe.ns)
  and
  [core/image_op_contract_recipe.ns](core/image_op_contract_recipe.ns),
  plus
  [core/image_resource_recipe.ns](core/image_resource_recipe.ns),
  and
  [core/texture_binding_recipe.ns](core/texture_binding_recipe.ns),
  and
  [core/sampling_recipe.ns](core/sampling_recipe.ns),
  plus
  [core/shader_packet_recipe.ns](core/shader_packet_recipe.ns),
  plus
  [core/shader_consumer_recipe.ns](core/shader_consumer_recipe.ns),
  plus
  [core/pixelmagic_pipeline_recipe.ns](core/pixelmagic_pipeline_recipe.ns),
  plus
  [core/render_plan_recipe.ns](core/render_plan_recipe.ns),
  plus the first image-op family:
  [core/grayscale_recipe.ns](core/grayscale_recipe.ns),
  [core/invert_recipe.ns](core/invert_recipe.ns),
  [core/threshold_recipe.ns](core/threshold_recipe.ns),
  and the next foundational filter family:
  [core/brightness_recipe.ns](core/brightness_recipe.ns),
  [core/contrast_recipe.ns](core/contrast_recipe.ns),
  [core/blur_recipe.ns](core/blur_recipe.ns),
  [core/edge_recipe.ns](core/edge_recipe.ns),
  [core/sharpen_recipe.ns](core/sharpen_recipe.ns),
  plus the first analysis family:
  [core/histogram_recipe.ns](core/histogram_recipe.ns),
  [core/image_stats_recipe.ns](core/image_stats_recipe.ns)
* this is still an early package skeleton, not yet a full crate-style auto-imported library

Current first responsibility:

* make the image-resource handoff explicit
* establish a canonical `PixelMagicImagePacket` shape
* establish a first actually auto-injectable `PixelMagicContracts` helper module
* establish a canonical `PixelMagicImageOpProfile` shape
* establish a canonical `PixelMagicImageOpSummary` shape
* establish a canonical `PixelMagicImageResource` shape
* establish a canonical `PixelMagicTextureBinding` shape
* establish a canonical `PixelMagicSampleIntent` shape
* establish a canonical `PixelMagicShaderPacket` shape
* establish a canonical `PixelMagicShaderConsumer` shape
* establish a canonical `PixelMagic` project-shaped pipeline recipe
* establish a canonical render-plan summary that ties filter chains,
  analysis, texture handoff, and shader consumption into one CPU-visible
  contract
* establish the first checked-in image-op family for grayscale / invert / threshold style work
* establish the next checked-in filter family for brightness / contrast / blur / edge / sharpen style work
* establish the first checked-in image-analysis family for histogram / image-stats style work
* establish one explicit shared image-op contract that all checked-in filter recipes can align to
* provide a stable checked-in bridge from host-preprocessed image description to shader-facing resource metadata
* provide first reusable helper totals for chained filters, image analysis
  quality, and texture handoff scoring
* persist shape/hash-bound raw `gray8` data plus invert and chained-threshold
  expected outputs, then execute both upload/dispatch/readback paths through a
  generic registered Metal unary runner with exact comparisons
* own `nuis.pixelmagic` registration of gray8 shape, payload, two-kernel
  collection, and persistence metadata through the provider-neutral
  `nuis-device-sample-input-registration-v1` table; std only supplies the host
  preprocessing evidence and does not construct image requests
* own the checked-in
  [gray8-invert-threshold.nspf](provider-plans/gray8-invert-threshold.nspf)
  through `nuis-pixelmagic-filter-plan-v1`; the plan declares input bytes,
  ordered stages, scalar bindings, expected outputs, and dependency references,
  while its AOT parser derives only provider-neutral request contracts
* declare a `nuis-pixelmagic-filter-plan-catalog-v1` in `module.toml`, with the
  two-stage graph as the deterministic default and
  [gray8-threshold.nspf](provider-plans/gray8-threshold.nspf) as a second
  independently validated package plan
* consume optional `nuis.pixelmagic:filter-plan=<declared-plan-id>` requests
  carried by the generic `nuis-artifact-provider-metadata-v1` table; the
  compiler and provider registry preserve opaque entries, while PixelMagic
  alone authorizes the requested catalog identity
* consume trace-projected requests from the generic
  `nuis-artifact-provider-metadata-scope-v1` envelope, allowing one artifact
  metadata table to select different plans for different domain/trace pairs
  while legacy unscoped entries remain global
* execute both catalog entries in official native fixtures: the existing
  dependency graph retains two exact outputs and one GLM/time-bound transfer,
  while `pixelmagic_threshold_provider_demo` runs threshold-only as one Metal
  request with output `[0,0,15,15]` and zero dependency edges
* connect invert to threshold through package-authored provider dependency,
  input-binding, GLM ownership, and clock evidence while Nsdb remains unaware of
  PixelMagic operation names
* participate in the persistent Nuis worker route through a registered Metal
  provider/adapter/operation identity, worker-issued dispatch permit, inherited
  input carrier, verified output carrier, and graph-close release evidence

Current early-beta execution boundary:

* real Metal upload, dispatch, and readback are regression-backed on supported
  Apple hosts
* the Nuis worker owns lifecycle, request ingress, and operation authorization
* the Metal ABI runner remains a thin registered platform adapter
* moving the actual adapter invocation and output-carrier creation behind a
  provider-neutral worker execution capsule is the next closure step

Current official surface registry:

* `contract.pixelmagic.image-resource-shaping.v1`
* `contract.pixelmagic.texture-handoff.v1`
* `contract.pixelmagic.shader-facing-image-prep.v1`
* `contract.pixelmagic.render-plan.v1`
* `contract.pixelmagic.provider-sample-input-registration.v1`
* `contract.pixelmagic.filter-plan.v1`
* `surface.pixelmagic.shader.contracts.v1`
* `surface.pixelmagic.shader.packet-bridge.v1`
* `surface.pixelmagic.shader.render.v1`
* `surface.pixelmagic.shader.texture.v1`
* `surface.pixelmagic.shader.pipeline.v1`

See also:

* [core/README.md](core/README.md)
* [pixelmagic-mainline-contract.md](../../docs/reference/pixelmagic-mainline-contract.md)
* [galaxy-frontdoor-prep-sketch.md](../../docs/reference/galaxy-frontdoor-prep-sketch.md)
* [galaxy-texture-handoff-contract.md](../../docs/reference/galaxy-texture-handoff-contract.md)
