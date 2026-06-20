# `pixelmagic-core`

`pixelmagic-core` is the smallest checked-in source layer of `PixelMagic`.

Its job is to define the narrow image/resource handoff shapes that later
shader-facing consumers can build on.

Current intended responsibility:

* image packet description
* image resource description
* texture binding description
* sample intent description
* shader packet description
* shader consumer description
* resource-set lowering shape
* shader-facing seed preparation

Current source anchor:

* [image_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_packet_recipe.ns)
* [image_resource_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_resource_recipe.ns)
* [texture_binding_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/texture_binding_recipe.ns)
* [sampling_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/sampling_recipe.ns)
* [shader_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_packet_recipe.ns)
* [shader_consumer_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_consumer_recipe.ns)

`image_packet_recipe.ns` currently provides:

* `PixelMagicImagePacket`
* `PixelMagicPacketSeeds`
* a narrow packet summary path matching the current checked-in bridge demos

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
