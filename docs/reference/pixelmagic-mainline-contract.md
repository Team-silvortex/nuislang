# `PixelMagic` Mainline Contract

This file is the shortest current contract for reading `PixelMagic` as a real
checked-in standard-library `Galaxy`, not just as a future package name.

## Current Position

Today `PixelMagic` already exists in two complementary forms:

* stdlib canonical source modules
* project-shaped domain companions

The important current rule is:

`stdlib defines the canonical chain; project demos prove the chain survives as one compiled domain route`

## Stdlib Chain

The current canonical source chain in
[stdlib/pixelmagic](../../stdlib/pixelmagic/README.md)
is:

```text
image packet
-> image resource
-> texture binding
-> sample intent
-> shader packet
-> shader consumer
-> project-shaped pipeline
```

Concrete current anchors:

1. [image_packet_recipe.ns](../../stdlib/pixelmagic/core/image_packet_recipe.ns)
2. [image_op_contract_recipe.ns](../../stdlib/pixelmagic/core/image_op_contract_recipe.ns)
3. [image_resource_recipe.ns](../../stdlib/pixelmagic/core/image_resource_recipe.ns)
4. [texture_binding_recipe.ns](../../stdlib/pixelmagic/core/texture_binding_recipe.ns)
5. [sampling_recipe.ns](../../stdlib/pixelmagic/core/sampling_recipe.ns)
6. [shader_packet_recipe.ns](../../stdlib/pixelmagic/core/shader_packet_recipe.ns)
7. [shader_consumer_recipe.ns](../../stdlib/pixelmagic/core/shader_consumer_recipe.ns)
8. [pixelmagic_pipeline_recipe.ns](../../stdlib/pixelmagic/core/pixelmagic_pipeline_recipe.ns)

## Domain Route

The current checked-in project route reads like:

```text
tooling preprocess
-> PixelMagic packet bridge
-> PixelMagic texture-resource handoff
-> PixelMagic project-shaped pipeline
-> PixelMagic render
```

Concrete current anchors:

1. [cli_pgm_info_demo](../../examples/projects/tooling/cli_pgm_info_demo)
2. [cli_pgm_invert_demo](../../examples/projects/tooling/cli_pgm_invert_demo)
3. [cli_pgm_threshold_demo](../../examples/projects/tooling/cli_pgm_threshold_demo)
4. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
5. [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
6. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
7. [pixelmagic_threshold_provider_demo](../../examples/projects/domains/pixelmagic_threshold_provider_demo)
8. [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)

## Relationship Rule

The current intended relationship is:

* stdlib recipes should name the stable semantic chain
* project demos should pressure-test the same chain in one domain-shaped closure
* demos may still repeat logic that stdlib already models, but that duplication
  should shrink over time

Short rule:

`recipe first for contract truth; demo second for end-to-end lowering truth`

## Official Surface Registry

The current registry-facing `PixelMagic` surface ids are:

1. `contract.pixelmagic.image-resource-shaping.v1`
2. `contract.pixelmagic.texture-handoff.v1`
3. `contract.pixelmagic.shader-facing-image-prep.v1`
4. `contract.pixelmagic.render-plan.v1`
5. `contract.pixelmagic.provider-sample-input-registration.v1`
6. `contract.pixelmagic.filter-plan.v1`
7. `surface.pixelmagic.shader.contracts.v1`
8. `surface.pixelmagic.shader.packet-bridge.v1`
9. `surface.pixelmagic.shader.render.v1`
10. `surface.pixelmagic.shader.texture.v1`
11. `surface.pixelmagic.shader.pipeline.v1`

The intended rule is:

* `contract.*` ids name semantic lowering/bridge commitments
* `surface.*` ids name checked-in shader-facing public units
* library module filenames may evolve, but these registry ids should remain the stable discovery vocabulary

## What Is Already Real

At the current repository stage, `PixelMagic` already has:

* an official stdlib package identity
* a canonical recipe chain in `stdlib/pixelmagic/core`
* an explicit shared image-op contract in stdlib form
* a checked-in project-shaped domain pipeline
* shader/data/cpu cooperation through the current packet/resource/render route
* a registered Metal provider runner that submits a real compute command buffer
  on macOS, uploads a shape/hash-validated raw `gray8` payload, dispatches an
  invert kernel, reads the transformed bytes back, and records the Metal device
  plus output byte/hash evidence
* package-independent `nuis-provider-buffer-descriptor-v1` and
  `nuis-provider-kernel-descriptor-v1` requests carrying buffer identity,
  element/layout/shape/stride, payload integrity, kernel bindings, dispatch,
  and typed scalar arguments across the Nuis-to-Nsdb boundary
* a checked-in package-owned
  [`nuis-pixelmagic-filter-plan-v1`](../../stdlib/pixelmagic/provider-plans/gray8-invert-threshold.nspf)
  plan that declares the ordered gray8 `invert -> threshold` graph, its raw
  input and exact baselines, scalar bindings, and producer dependency
* a strict AOT filter-plan parser that hash-binds those package assets and
  derives the existing provider-neutral request, input-binding, GLM ownership,
  and clock evidence without teaching Nsdb any PixelMagic operation names
* a package-owned `nuis-pixelmagic-filter-plan-catalog-v1` that declares two
  AOT plan paths, selects the two-stage graph as its default, rejects incomplete
  source coverage or duplicate identities, and binds ordered path/id/source
  hashes into one catalog hash
* an open `nuis-artifact-provider-metadata-v1` path that carries ordered opaque
  provider requests from `nuis.toml` through the build manifest and LinkPlan;
  only the PixelMagic adapter interprets
  `nuis.pixelmagic:filter-plan=<declared-plan-id>`
* an optional `nuis-artifact-provider-metadata-scope-v1` envelope that projects
  requests by domain and trace, strips scope syntax before package dispatch,
  and keeps unscoped entries globally visible for compatibility

The current native sample is a complete narrow data path rather than a scalar
proxy: std preprocessing persists a 2 x 2 PGM-derived payload, provider evidence
binds its format, dimensions, stride, maximum value, operation, size, path, and
hash, and Nsdb validates the registered buffer/kernel descriptors before Metal
execution. The first output is transferred into the threshold stage through
the generic edge transport; both native outputs are compared exactly. Legacy
`pixel_*` evidence remains readable through a compatibility conversion, but
native execution consumes only the common provider request collection. The
scope remains deliberately small: the package declares one two-stage plan and
one threshold-only plan. Normal artifact execution now requests the two-stage
entry through an exact shader/trace scope. A separate official shader artifact
retains the unscoped compatibility form, requests the non-default threshold-only
entry, and executes it as one real Metal request with no dependency edge, exact
output `[0,0,15,15]`, and hash `0x4d00177f9dae564b`. Independent trace
projection and multi-plan persistence are covered together; undeclared,
malformed, duplicate, or hash-drifting requests fail closed.

## What Is Not Done Yet

`PixelMagic` does not yet claim:

* a stable public image asset ABI
* a finished source-level texture sampling surface
* a real import-based package workflow
* a finished public filter family API
* a backend-complete texture upload/runtime contract
* additional pixel formats beyond the first `gray8` plan catalog

## Reading Order

If you only want the shortest current reading route, use:

1. [stdlib/pixelmagic/README.md](../../stdlib/pixelmagic/README.md)
2. [artifact-provider-metadata-scope.md](artifact-provider-metadata-scope.md)
3. [stdlib/pixelmagic/core/README.md](../../stdlib/pixelmagic/core/README.md)
4. [pixelmagic_pipeline_recipe.ns](../../stdlib/pixelmagic/core/pixelmagic_pipeline_recipe.ns)
5. [pixelmagic_packet_bridge_demo](../../examples/projects/domains/pixelmagic_packet_bridge_demo)
6. [pixelmagic_texture_resource_demo](../../examples/projects/domains/pixelmagic_texture_resource_demo)
7. [pixelmagic_pipeline_demo](../../examples/projects/domains/pixelmagic_pipeline_demo)
8. [pixelmagic_threshold_provider_demo](../../examples/projects/domains/pixelmagic_threshold_provider_demo)
9. [pixelmagic_render_demo](../../examples/projects/domains/pixelmagic_render_demo)
