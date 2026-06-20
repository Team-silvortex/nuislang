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
  [module.toml](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/module.toml)
* the current first canonical source assets are
  [core/image_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_packet_recipe.ns)
  and
  [core/image_resource_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/image_resource_recipe.ns),
  plus
  [core/texture_binding_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/texture_binding_recipe.ns),
  and
  [core/sampling_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/sampling_recipe.ns),
  plus
  [core/shader_packet_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_packet_recipe.ns),
  plus
  [core/shader_consumer_recipe.ns](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/shader_consumer_recipe.ns)
* this is still an early package skeleton, not yet a full crate-style auto-imported library

Current first responsibility:

* make the image-resource handoff explicit
* establish a canonical `PixelMagicImagePacket` shape
* establish a canonical `PixelMagicImageResource` shape
* establish a canonical `PixelMagicTextureBinding` shape
* establish a canonical `PixelMagicSampleIntent` shape
* establish a canonical `PixelMagicShaderPacket` shape
* establish a canonical `PixelMagicShaderConsumer` shape
* provide a stable checked-in bridge from host-preprocessed image description to shader-facing resource metadata

See also:

* [core/README.md](/Users/Shared/chroot/dev/nuislang/stdlib/pixelmagic/core/README.md)
* [galaxy-frontdoor-prep-sketch.md](/Users/Shared/chroot/dev/nuislang/docs/reference/galaxy-frontdoor-prep-sketch.md)
* [galaxy-texture-handoff-contract.md](/Users/Shared/chroot/dev/nuislang/docs/reference/galaxy-texture-handoff-contract.md)
