# Nsdb YIR Debugger Frontdoor

`Nsdb` is the Nuis debugger frontdoor for YIR-layer debugging.

It is intentionally not an `lldb` clone. Native debuggers can still attach to
the host executable shell when the final binary is Mach-O, PE, or ELF, but that
view is expected to be low-level and incomplete for Nuis semantics.

`Nsdb` should instead consume the metadata that `Nsld` organizes:

* domain units
* clock protocol edges
* deterministic data segments
* artifact lowering units
* lowering IR sidecars

## Current Commands

```sh
cargo run -p nsdb -- status
cargo run -p nsdb -- inspect <artifact-output-dir>
cargo run -p nsdb -- inspect <artifact-output-dir> --json
```

When given an output directory, `Nsdb` resolves
`nuis.build.manifest.toml` inside that directory.

## Current Debug Model

Current `Nsdb` output is an inspectable metadata view, not an interactive
debugger yet.

It reports:

* `debug_model = yir-metadata`
* `native_debugger_visibility = host-shell-only`
* `nsdb_visibility = domains+clock+segments+lowering-units`
* `sidecar_count`
* `sidecars`
* `debug_readiness = yir-debug-ready` when the linker graph, clock protocol,
  hetero calculate plan, lowering units, and referenced lowering IR sidecars are
  all readable

`Nsdb` can now expose sidecar capability metadata such as the owning Nustar,
frontend IR, native IR, backend lowering model, validation contracts, and entry
symbol. The same view is used across current heterogeneous domains:

* shader sidecars can describe
  `shader-nustar -> nuis-yir.shader -> msl2.4` plus Metal/Vulkan/DirectX style
  dispatch and resource lowering contracts
* kernel sidecars can describe
  `kernel-nustar -> nuis-yir.kernel -> coreml-program` or SPIR-V/host-SIMD
  tensor dispatch contracts
* network sidecars can describe
  `network-nustar -> nuis-yir.network -> foundation-url-request`,
  `posix-socket`, or `winsock-overlapped` transport lowering contracts

This means `Nsdb` is pointed at the right layer and can see real lowering
capability metadata, but source-level stepping, breakpoints, value inspection,
and replay still need dedicated debug sidecars.

## Relationship To Nsld

`Nsld` freezes and validates the linker-facing contract graph.

`Nsdb` consumes that graph to answer debugger questions at the semantic layer:

```text
Nsld link metadata
  -> domain / clock / segment / lowering-unit map
  -> Nsdb YIR debug view
  -> future stepping / replay / value inspection
```

The debugger should not depend on LLDB being able to reconstruct Nuis
semantics from lowered LLVM code. LLDB remains useful for host crashes and
native wrapper failures; `Nsdb` is responsible for the YIR-level truth.
