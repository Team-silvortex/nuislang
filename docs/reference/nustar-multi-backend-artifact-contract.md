# Nustar Multi-Backend Artifact Contract

This file records the current implementation-facing contract for heterogeneous
`nustar` backend artifacts.

It answers one narrow question:

`when one YIR lowering contract can produce several backend artifacts, what
metadata must travel with every artifact so nsld can assemble it deterministically
without knowing domain-specific backend logic?`

## Short Rule

Every backend artifact variant should identify:

* `backend`: concrete backend lane such as `metal`, `vulkan`, `webgpu`,
  `coreml`, `mps-graph`, or `cpu-fallback`
* `backend_family`: broad execution family such as `gpu`, `npu`, `cpu`, or
  `reference`
* `target_os`: platform routing label such as `macos`, `windows`, `host`, or
  `cross-platform`
* `target_device`: device-class routing label such as `apple-gpu`,
  `apple-ane`, `vulkan-device`, `webgpu-device`, or `host-cpu`
* `ir_format`: artifact IR/source format such as `msl`, `glsl450`, `hlsl`,
  `wgsl`, `mlmodel`, `mlpackage`, `mps-graph-json`, `spirv`, or
  `llvm-bitcode`
* `dispatch_abi`: lifecycle/dispatch ABI name such as
  `metal-render-pipeline`, `vulkan-compute-pipeline`, `coreml-predict`, or
  `nuis-host-call`
* `kind`: storage kind such as `msl-source`, `wgsl-source`, `graph`,
  `mlmodel`, `mlpackage`, `spirv`, or `native`
* `priority`: deterministic fallback/order hint; smaller numbers are preferred
* `status`: current readiness label such as `active` or `planned`
* `verification`: current verification surface, presently `contract-only`
* `entry`: backend entry symbol or logical stage id
* `artifact`: relative artifact path under the AOT output directory
* `notes`: human-readable implementation note

Shorter version:

`backend` says what produced it; `backend_family` and `target_device` say where
it wants to run; `ir_format` and `dispatch_abi` say how nsld/runtime should
consume it.

## Current Shader Variants

Shader stage contracts currently emit these backend slots:

| backend | family | target OS | target device | IR format | dispatch ABI | priority | status |
| --- | --- | --- | --- | --- | --- | ---: | --- |
| `metal` | `gpu` | `macos` | `apple-gpu` | `msl` | `metal-render-pipeline` | 10 | `active` |
| `vulkan` | `gpu` | `cross-platform` | `vulkan-device` | `glsl450` | `vulkan-graphics-pipeline` | 20 | `active` |
| `directx` | `gpu` | `windows` | `d3d12-device` | `hlsl` | `d3d12-graphics-pipeline` | 30 | `active` |
| `webgpu` | `gpu` | `cross-platform` | `webgpu-device` | `wgsl` | `webgpu-render-pipeline` | 40 | `planned` |
| `opengl` | `gpu` | `cross-platform` | `opengl-device` | `glsl460` | `opengl-graphics-pipeline` | 80 | `active` |
| `reference` | `reference` | `host` | `host` | `ppm` | `prerender` | 900 | `active` |

`yir-pack-aot` now writes descriptor comments into each generated shader
artifact scaffold. It also writes a `webgpu/*.wgsl` scaffold instead of treating
WebGPU as an opaque fallback text artifact.

## Current Kernel Variants

Kernel stage and graph contracts currently emit these backend slots when a
stage/graph is backend-eligible:

| backend | family | target OS | target device | IR format | dispatch ABI | priority | status |
| --- | --- | --- | --- | --- | --- | ---: | --- |
| `coreml` | `npu` | `macos` | `apple-ane` | `mlmodel` / `mlpackage` | `coreml-predict` | 10 | `planned` |
| `mps-graph` | `gpu` | `macos` | `apple-gpu` | `mps-graph-json` | `mps-graph-dispatch` | 20 | `planned` |
| `vulkan` | `gpu` | `cross-platform` | `vulkan-device` | `spirv` | `vulkan-compute-pipeline` | 30 | `planned` |
| `cpu-fallback` | `cpu` | `cross-platform` | `host-cpu` | `llvm-bitcode` | `nuis-host-call` | 900 | `planned` / `active` |

`cpu-fallback` stays explicit instead of being hidden as an implementation
detail. That keeps the classic host/C world visible as one execution family
inside the Nuis artifact graph.

## Frontdoor Readiness

`nuis` link-plan frontdoors expose a generic heterogeneous readiness summary so
scripts do not need to understand every shader, kernel, data, or network
backend contract directly.

Current fields include:

* `link_plan_heterogeneous_domain_units`
* `link_plan_heterogeneous_domain_ready_units`
* `link_plan_heterogeneous_domain_readiness_ready`
* `link_plan_heterogeneous_domain_families`
* `link_plan_heterogeneous_domain_first_unready`
* `link_plan_heterogeneous_domain_readiness`

The current generic readiness check is intentionally about assembly evidence,
not domain-specific execution semantics. A non-CPU domain unit is ready when it
has:

* an artifact payload blob
* an artifact payload format
* a bridge stub

Domain-specific validation still belongs to the registered `nustar` contract.
For example, shader and kernel domains can additionally require selected
lowering targets and IR sidecars, while data-fabric contract units may be
assembly-ready without those execution-side fields. The readiness summary is a
cross-domain frontdoor signal for workflow routing, not a replacement for
shader/kernel/network contract validation.

## Nsld Boundary

`nsld` should not need to understand shader or kernel semantics directly.

For multi-backend artifacts, its intended job is:

* read backend descriptors from package manifests and generated artifact
  scaffolds
* order candidates by lifecycle, clock metadata, dependency edges, and
  `priority`
* reject artifacts whose `dispatch_abi`, `target_os`, or `target_device` cannot
  be satisfied by the current final executable plan
* keep C/native object compatibility behind explicit adapter lanes such as
  `cpu-fallback` or future `cffi` backend descriptors

The compiler may still bootstrap some domain knowledge in-tree today, but the
artifact contract should be explicit enough that future `nustar` registration can
own the backend matrix without turning nsld into a pile of special cases.
