# Nuis Long-Range Heterogeneous OS Roadmap

This file records the long-range direction behind the current alpha work.

It is intentionally not a current feature checklist.

It exists so future implementation work does not accidentally overfit `nuis`
to the temporary Linux/libc/host-toolchain bridge used during early bootstrap.

## Short Rule

`nuis is using the traditional host stack as a bridge, not as its final machine model`

The current toolchain still relies on host operating systems, LLVM, native
linkers, and carefully whitelisted FFI surfaces where they are useful. That is
practical and intentional.

The long-range architecture should still treat those pieces as compatibility
and bootstrap layers rather than as the semantic center of the system.

The compiler itself is the current self-hosting exception. Everywhere else,
implementation should move into Nuis as soon as the language and lowering are
capable: standard-library behavior, lifecycle loops, runtime orchestration,
workers, schedulers, and official Galaxy logic should not remain in a host
language merely because a bootstrap prototype began there. Foreign-language
code should converge toward thin registered ABI adapters with no product
policy.

## Architecture North Star

The eventual target is a `nuis`-native computing base for heterogeneous XR and
mobile-workstation style hardware.

In that direction, `nuis` should be read less like a conventional
von-Neumann-centered language and more like a contract-driven heterogeneous
compute system:

```text
nuis source / galaxy
  -> NIR
  -> YIR contract graph
  -> registered nustar capability domains
  -> multi-domain lowering
  -> lifecycle / scheduling / verification
  -> native nuis binary container
  -> host shell today, nuis OS loader later
```

The CPU remains important, but it should not be the only semantic center.

Long-range first-class participants include:

* CPU domains
* shader domains
* kernel / accelerator domains
* network domains
* host and C FFI compatibility domains
* future `nuis OS` system domains
* future `yalivia` runtime/JIT communication domains
* future `vulpoya` analyzer/verifier review domains

## What This Means For C, libc, And Linux

The C ABI and libc model should stay useful, but bounded.

Current role:

* bootstrap route for host interop
* portable smoke-test surface
* compatibility bridge for existing OS services
* constrained FFI lane for testing toolchain behavior early

Long-range role:

* one registered `nustar` capability family among others
* whitelisted and hash-signature checked where memory safety matters
* not the default semantic model for memory, scheduling, or device ownership
* not the thing that defines what a `nuis` program fundamentally is

The stronger design reading is that the whole C / libc / classic
von-Neumann-host world is a hardware and execution paradigm wrapped by Nuis.
It is not merely a bag of foreign functions. Today that paradigm maps to
ordinary CPUs, process loaders, host kernels, and native linkers. In a later
Nuis-native machine it can become one explicit compatibility fabric alongside
shader, kernel, network, and data-fabric domains.

Short rule:

`C ABI is an interop domain; it should not become the ontology of nuis`

Implementation-facing anchor:
[../reference/cffi-von-neumann-domain-contract.md](../../docs/reference/cffi-von-neumann-domain-contract.md)

## Nuis OS Direction

Future `nuis OS` work is still design-stage.

The long-range intent is that a native OS/runtime environment should eventually
replace the current dependence on traditional Linux/POSIX/libc assumptions for
core program lifecycle.

Likely responsibility areas:

* native loader support for the `nuis` binary/container family
* direct lifecycle-loop ownership instead of only host `main` handoff
* native task, clock, memory, and device scheduling contracts
* GLM-aware resource and capability validation
* first-class heterogeneous domain orchestration
* OS-level support for XR-facing input/output, sensors, graphics, compute, and
  local network/session surfaces

This should stay aligned with current binary and lifecycle docs:

* [../reference/nuis-binary-format-protocol.md](../../docs/reference/nuis-binary-format-protocol.md)
* [../reference/nuis-aot-lifecycle-loop-sketch.md](../../docs/reference/nuis-aot-lifecycle-loop-sketch.md)
* [../reference/nuis-launcher-container-linker-sketch.md](../../docs/reference/nuis-launcher-container-linker-sketch.md)

## Hardware Paradigm Direction

The long-range target hardware class is not just "a faster CPU machine."

The intended pressure is closer to:

* XR-first interaction
* mobile-workstation constraints
* heterogeneous local compute
* GPU / NPU / accelerator participation
* low-latency local scheduling
* structured local networking
* explicit memory and ownership boundaries
* static packaging where possible, runtime adaptation where necessary

This is why the language/toolchain should keep investing in:

* `nustar` registration instead of hard-coded backend assumptions
* YIR contract graphs instead of one backend-specific IR worldview
* GLM and global-time contracts across synchronous and asynchronous work
* native binary protocol evolution instead of only host-object emission
* official galaxies such as `std`, PixelMagic, and WitSage as real pressure
  tests for CPU/shader/kernel/network cooperation

## Performance Posture

The bootstrap path can accept a CFFI / host-compat performance tax, as long as
the tax is visible and structurally removable.

Early Nuis programs may depend heavily on CFFI Nustar compatibility lanes for
I/O, filesystem, networking, platform services, and native wrappers. Compared
with direct C/C++ on the same host, those lanes can reasonably be slower while
the project is still prioritizing explicit contracts, whitelist validation,
GLM/YIR visibility, and deterministic lifecycle metadata.

The long-range goal is staged:

* alpha/toolchain phase: make overhead measurable with std, benchmark, and
  hetero-proxy examples instead of hiding it behind claims
* Nuis OS phase: move common compatibility costs into kernel/runtime-owned
  services so fewer calls pay full process-level wrapper overhead
* hardware-paradigm phase: use heterogeneous scheduling and data placement to
  win on workloads where classic CPU-centered execution is not the natural fit

Working target language:

* short-term compatibility overhead may be in a rough 20% class for some
  CFFI-heavy paths, especially against tight hand-written C/C++ baselines
* Nuis OS compatibility services should aim for a lower 5-10% class overhead
  on common host-compat paths when measured workload shape permits
* true heterogeneous workloads may eventually outperform classic pipelines by
  avoiding unnecessary CPU-centered data motion and scheduling assumptions
* these are roadmap targets, not current benchmark claims

## Roadmap Posture

This direction should influence design decisions now, but it should not be
misrepresented as current product maturity.

The current long-range version posture is:

* `alpha`
  complete and harden the executable-artifact route, std/tooling base, Nustar
  registration discipline, and native binary/linker contracts
* early `beta`
  stabilize enough of that base that compiler/tooling self-use can stop being
  a sketch and become a repeated engineering pressure
* around `beta-0.10.0`
  treat self-hosting as a serious central gate rather than a distant direction
* late `beta` into `gamma`
  begin substantial Vulpoya and Yalivia work, mature Nsld/Nsdb/Nsbdr-style
  toolchain coordination, and grow native framework surfaces
* `gamma`
  absorb whole-stack coordination, analyzer/runtime cooperation, and
  native-framework maturity so `1.0.0` is delayed until the ecosystem shape is
  honest, not merely until the compiler can emit binaries

Current alpha work should keep prioritizing:

* source-to-YIR-to-AOT repeatability
* standard library contract surfaces
* project/package registration discipline
* FFI whitelist and pointer-safety boundaries
* native artifact inspection, verification, and launch loops
* PixelMagic and WitSage as proving grounds for official galaxy behavior

Later work can broaden into:

* richer `NART` section-table defaults
* self-owned linker experiments
* `nuis OS` loader sketches
* device/domain scheduling contracts
* deeper `yalivia` runtime integration
* deeper `vulpoya` YIR review integration
* XR-oriented galaxy/runtime surfaces

## Design Guardrails

When future work faces a tradeoff, prefer the option that:

* keeps domain capability behind registered `nustar` contracts
* keeps the compiler aware of contract shape, not every backend implementation
* keeps C/libc compatibility as a bridge rather than the default model
* keeps native binary metadata inspectable and verifiable
* keeps GLM/global-time semantics applicable across CPU, shader, kernel,
  network, and future OS/runtime domains
* keeps official galaxies simple enough to become real examples instead of
  ornamental demos

## What Not To Claim Yet

This roadmap does not claim that the repository already has:

* a native `nuis OS`
* a final self-owned linker
* a final XR workstation runtime
* a stable kernel/device driver model
* full replacement of Linux/libc/host linker dependencies
* final GLM treatment for every raw pointer or FFI edge

Short rule:

`write the future into the architecture, but keep the status reports honest`
