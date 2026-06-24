# `pixelmagic-core`

`pixelmagic-core` is the smallest checked-in source layer of `PixelMagic`.

Its job is to define the narrow image/resource handoff shapes that later
shader-facing consumers can build on.

The companion auto-injectable helper surface currently lives in
[../lib/image_contracts.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/lib/image_contracts.ns).

Current intended responsibility:

* image packet description
* image resource description
* texture binding description
* sample intent description
* shader packet description
* shader consumer description
* project-shaped pipeline composition
* first image-op family
* resource-set lowering shape
* shader-facing seed preparation

Current source anchor:

* [image_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_packet_recipe.ns)
* [image_op_contract_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_op_contract_recipe.ns)
* [image_resource_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_resource_recipe.ns)
* [texture_binding_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/texture_binding_recipe.ns)
* [sampling_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/sampling_recipe.ns)
* [shader_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_packet_recipe.ns)
* [shader_consumer_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_consumer_recipe.ns)
* [pixelmagic_pipeline_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/pixelmagic_pipeline_recipe.ns)
* [grayscale_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/grayscale_recipe.ns)
* [invert_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/invert_recipe.ns)
* [threshold_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/threshold_recipe.ns)
* [brightness_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/brightness_recipe.ns)
* [contrast_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/contrast_recipe.ns)
* [blur_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/blur_recipe.ns)
* [edge_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/edge_recipe.ns)
* [sharpen_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/sharpen_recipe.ns)
* [histogram_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/histogram_recipe.ns)
* [image_stats_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_stats_recipe.ns)

`image_packet_recipe.ns` currently provides:

* `PixelMagicImagePacket`
* `PixelMagicPacketSeeds`
* a narrow packet summary path matching the current checked-in bridge demos

`image_op_contract_recipe.ns` currently provides:

* `PixelMagicImageOpProfile`
* `PixelMagicImageOpSummary`
* one explicit generic `packet -> resource -> sample -> shader` image-op contract
* the shared semantic baseline that current checked-in filter recipes are expected to align with

`image_resource_recipe.ns` currently provides:

* `PixelMagicImageResource`
* `PixelMagicShaderSeeds`
* lowering into `NovaResourceSetPacket` / `NovaResourceSetState`
* a small compileable summary path that keeps the first contract narrow and inspectable

`texture_binding_recipe.ns` currently provides:

* `PixelMagicTextureBinding`
* `PixelMagicBindingSeeds`
* a narrow checked-in `resource -> binding` bridge before real source-level texture sampling builtins land

`sampling_recipe.ns` currently provides:

* `PixelMagicSampleIntent`
* `PixelMagicSampleSeeds`
* a narrow checked-in `binding -> sample intent` bridge before real source-level `sample_uv` style builtins land

`shader_packet_recipe.ns` currently provides:

* `PixelMagicShaderPacket`
* a narrow checked-in `sample intent -> shader packet` bridge
* a stable packet shape that later real shader-profile lowering can target

`shader_consumer_recipe.ns` currently provides:

* `PixelMagicShaderConsumer`
* `PixelMagicConsumerSummary`
* a narrow checked-in `shader packet -> shader consumer` bridge

`pixelmagic_pipeline_recipe.ns` currently provides:

* a checked-in `packet -> resource -> binding -> sample -> shader -> consumer` pipeline skeleton
* one project-shaped summary path for the whole canonical chain
* a stable composition target before real package/import and shader frontdoor work is finished

`grayscale_recipe.ns`, `invert_recipe.ns`, and `threshold_recipe.ns` currently provide:

* the first checked-in `PixelMagic` image-op family
* narrow operation-specific packet/resource/sample/shader summaries
* the first stdlib-side alignment with the current tooling `pgm` route

`brightness_recipe.ns`, `contrast_recipe.ns`, `blur_recipe.ns`, `edge_recipe.ns`,
and `sharpen_recipe.ns` currently provide:

* the next checked-in `PixelMagic` filter family
* narrow operation-specific packet/resource/sample/shader summaries
* a stable follow-up set for common image-adjust and image-kernel style work

`histogram_recipe.ns` and `image_stats_recipe.ns` currently provide:

* the first checked-in `PixelMagic` image-analysis family
* narrow plan/summary shapes for histogram and sampled image statistics work
* a CPU-visible contract that later shader/kernel lowering can share instead of inventing separate analysis packets
