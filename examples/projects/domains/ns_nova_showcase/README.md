# NS Nova Showcase

This is the shortest checked-in NS Nova application slice. It keeps framework
lifecycle, image rendering, data transport, and backend selection independently
owned while compiling them as one static Nuis project.

The slice currently proves:

* Nuis-owned `NovaAppState` and `NovaFrameTransaction` transitions
* an explicit NS Nova framework import
* PixelMagic-owned inline shader rendering
* registered Data transport through `FabricPlane`
* conditional presentation driven by Shader readiness through a shared YIR result-state projection
* a bounded three-frame Nuis update loop lowered through a scoped frame helper
* one aggregate `NovaAppState` carried and rebound through all three loop iterations
* runtime-owned `NovaFrameResultHandle` validation of Shader-issued token, clock,
  and root identity before submission
* acyclic YIR text round-tripping for the aggregate loop result edge
* automatic host ABI selection without platform entries in `nuis.toml`
* a relocatable `galaxy.toml` plus an explicit `ns-nova.toml` framework profile

From the repository root:

```bash
cargo run -p nuis -- galaxy check examples/projects/domains/ns_nova_showcase
cargo run -p nuis -- check examples/projects/domains/ns_nova_showcase
cargo run -p nuis -- build examples/projects/domains/ns_nova_showcase build/ns-nova-showcase
```

This is a bounded lifecycle proof, not a claim of a stable interactive world loop.
The current helper carries aggregate application state through every iteration,
but continuous event dispatch, live post-dispatch Shader completion clocks,
provider-neutral window adapters, and full backend payload execution remain active work.
