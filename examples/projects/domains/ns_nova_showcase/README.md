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
* profile-derived runtime vertex counts `3/2/3` and instance counts `1/2/3`
* a Nuis-owned `(f32, f32, f32, f32)` tint uploaded as a read-only fragment uniform
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
The current helper carries aggregate application state through every iteration.
The separate-process M2 regression sends each validated draw through registered
provider IPC v2, observes a same-command Metal completion, and binds canonical
runtime scalars and immutable resource bytes into result-stream v2 replay. The
three-vertex frames have matching coverage but red/blue pixels selected by Nuis
tint values; the two-vertex frame stays cleared because no triangle is formed.
Replay matches live output. Noncanonical vertex-body changes are rejected by
the bounded emitter rather than silently replaced with fullscreen geometry.

This is a procedural `rgba8_unorm` / `triangle_strip` projection with fixed admitted
dimensions. The current Metal adapter admits 1-4 vertices and 1-256 instances;
one group-zero `vec4<f32>` fragment uniform is admitted at its compiled slot.
`shader_draw_instanced(pass, packet, vertices, instances, bindings)` carries an
optional `BindingSet`; `shader_uniform_binding(2, tint)` matches the PixelMagic
profile's `binding(0, 2)` declaration. Only four finite f32 values are accepted,
copied as 16 little-endian bytes, with no implicit casts, pointers, or file paths.
Code-asset identity and native Metal reflection both constrain the binding.
Type, shape, slot, bytes, and content hash are part of dispatch/replay identity.
The unchanged 256-byte argument budget is not a bulk-resource transport.

Legacy packet values are validated for the structural CPU reference preview, not
uploaded as GPU uniforms; that preview is not a WGSL pixel-equivalence oracle.
The real-pixel proof executes a runtime child after an AOT build, not a standalone
interactive window binary with self-contained provider injection. The existing
Metal OS adapter remains a native compatibility bridge. Continuous event dispatch,
large buffers/textures, self-contained runner injection, and cross-host window
adapters remain active work. Rebuild artifacts after the profile and Metal runner
v3 changes. Offline rendering cannot invent required uniform data, and unsupported
resource contracts never fall back to an unbound replay.
