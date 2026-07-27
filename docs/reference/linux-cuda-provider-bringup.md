# Linux CUDA Provider Bring-Up

This document defines the first Linux/CUDA mainline boundary for Nuis. The
route extends the existing provider-bundle and heterogeneous graph contracts;
it does not add a CUDA special case to Nsld or the generic provider frontdoor.

## Host Probe

The host capability protocol is `nuis-linux-cuda-host-probe-v1`.

Run the repository-owned probe on a selected Linux host without embedding that
host in project configuration:

```sh
ssh <linux-cuda-host> 'bash -s' < scripts/probe-linux-cuda-host.sh
```

Select a PTX architecture explicitly when needed:

```sh
ssh <linux-cuda-host> 'NUIS_CUDA_PROBE_ARCH=compute_89 bash -s' \
  < scripts/probe-linux-cuda-host.sh
```

The probe is read-only. It reports separate states for:

* Linux kernel and machine architecture
* CUDA compiler availability
* loaded and installed NVIDIA driver versions
* NVML/device-query readiness
* PTX compilation readiness

PTX compilation and device launch are deliberately separate. A host may
produce valid PTX while a driver/library mismatch blocks real device execution.
Conversely, a deployment host may launch Nuis-emitted PTX with only the NVIDIA
driver runtime installed. `nvcc` and `ptxas` are development-time differential
validation oracles, not Nuis build, packaging, or runtime dependencies.

## Current Evidence

The first remote x86_64 Linux host has CUDA 12.0 and successfully emits PTX 8.0
for an `sm_89` `nuis_probe_add` kernel. Its first probe found a loaded-versus-
installed NVIDIA driver mismatch. A maintenance reboot restored version
consistency and NVML device discovery.

The independent `nuis-cuda-runtime-smoke-v1` fixture now proves a real CUDA
runtime path on that host: device discovery, device allocation, H2D transfer,
vector-add launch, synchronization, D2H transfer, and exact
`[11, 22, 33, 44]` comparison all succeed.

Kernel Nustar now owns `kernel.cuda.ptx8_0.v1` and the
`cuda.nvidia-gpu` lowering target. The same portable Kernel YIR path emits a
deterministic PTX 8.0 sidecar with:

* an `sm_80` virtual compatibility baseline rather than a host-specific target
* the visible `nuis_kernel_vector_add_f32` entry
* an internal FNV-1a source hash
* the existing NDPB payload and artifact hash envelope

The exact sidecar source passes `ptxas` for `sm_89`. The repository-owned
`nuis-cuda-ptx-driver-smoke-v1` fixture then loads that PTX through the CUDA
Driver API, launches it on the real device, and verifies
`[11, 22, 33, 44]`.

This use of NVIDIA tooling validates compatibility; it does not define the
production compiler route. The intended route is:

`Kernel YIR -> Nuis PTX lowering -> Nsld-bound PTX -> Nuis worker -> CUDA Driver ABI`

The later native backend may additionally replace driver JIT assembly with a
Nuis-owned PTX-to-cubin/device-code path. Until then, the driver consumes PTX
directly and no `nvcc` invocation belongs in a produced program.

Kernel Nustar now owns the static `cuda.nvidia-gpu.bundle.v1` registration.
The generated provider manifest cross-binds it to `official.kernel`,
`cuda:nvidia-gpu`, `cuda.nvidia-gpu.real-device`, and
`cuda-ptx-real-device-runner`. Generic runner and execution selectors continue
to discover it only through the generated bundle table. Non-Linux hosts report
the registered CUDA runner as unavailable. Linux hosts materialize a
content-addressed process adapter only after the registered CUDA probe
succeeds.

The provider request protocol now carries generated device code through
`nuis-provider-code-asset-descriptor-v1`. The descriptor is provider-neutral:
it binds format, target, visible entry, package-relative path, byte length,
digest contract, and content hash. It rejects partial descriptors, absolute or
traversing paths, and malformed hash bindings. CUDA-specific interpretation
remains owned by the CUDA execution registration rather than the generic
provider frontdoor.

`nuis-kernel-code-asset-registry-v1` is now the single authority for the PTX
bytes, `sm_80` target, visible entry, package-relative file name, and digest
contract. AOT writes `nuis.domain.kernel.cuda.ptx` directly without invoking an
external compiler. The `official.kernel` device-sample registration verifies
the emitted bytes before persisting two f32 input payloads and one expected
output. Its generated request is accepted by the Nsdb parser and binds
`input.left,input.right`, `element_count=4`, `output.result`, and
`cuda:nvidia-gpu` without adding a CUDA branch to the generic request parser.

The CUDA execution registration now consumes that fixture beneath the normal
persistent Nuis worker. The worker validates the adapter executable, PTX path
and hash, ordered input descriptors, output role, output length, and returned
hash. Its thin 64-bit Linux adapter opens `libcuda.so.1` dynamically, creates a
Driver context, loads the Nuis-emitted PTX, performs H2D/launch/D2H, and writes
the result into the worker-owned `NUISPFD1` output descriptor. It uses no CUDA
headers, CUDA SDK link dependency, `nvcc`, or `ptxas`.

The real RTX 4050 integration test reaches this route through
`execute_provider_samples`, compares exact `[11, 22, 33, 44]` bytes, and closes
the graph-owned output. This proves the first persistent-worker CUDA closure,
not yet a final-image heterogeneous executable closure.

## Bring-Up Order

1. Keep the host probe deterministic and read-only. Complete.
2. Register a Linux/NVIDIA CUDA ABI target in the Kernel Nustar. Complete.
3. Lower a minimal YIR kernel into a hash-bound PTX sidecar. Complete.
4. Register a CUDA provider bundle without changing generic bundle selection.
   Complete.
5. Carry hash-bound device code in a provider-neutral request. Complete.
6. Bind the PTX asset to the CUDA vector-add request's ordered buffer roles.
   Complete.
7. Execute Nuis-emitted PTX through a persistent worker and the CUDA Driver ABI.
   Complete.
8. Verify output bytes and graph-close release. Complete. Promote clock and GLM
   release evidence into independently inspectable CUDA records.
9. Carry the provider-neutral payload and dispatch identity through Nsld.

The first execution target is vector addition. Wider tensor operations,
multi-GPU selection, CUDA graphs, and shader interop remain later work.
