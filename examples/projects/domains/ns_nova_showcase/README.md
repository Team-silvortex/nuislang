# NS Nova Showcase

This is the shortest checked-in NS Nova application slice. It keeps framework
lifecycle, image rendering, data transport, and backend selection independently
owned while compiling them as one static Nuis project.

The slice currently proves:

* Nuis-owned `NovaAppState` and `NovaFrameTransaction` transitions
* an explicit NS Nova framework import
* PixelMagic-owned inline shader rendering
* registered Data transport through `FabricPlane`
* automatic host ABI selection without platform entries in `nuis.toml`
* a relocatable `galaxy.toml` plus an explicit `ns-nova.toml` framework profile

From the repository root:

```bash
cargo run -p nuis -- galaxy check examples/projects/domains/ns_nova_showcase
cargo run -p nuis -- check examples/projects/domains/ns_nova_showcase
cargo run -p nuis -- build examples/projects/domains/ns_nova_showcase build/ns-nova-showcase
```

This is still a one-frame bootstrap, not a claim of a stable interactive loop.
Continuous event dispatch, conditional presentation lowering, provider-neutral
window adapters, and full backend payload execution remain active work.
