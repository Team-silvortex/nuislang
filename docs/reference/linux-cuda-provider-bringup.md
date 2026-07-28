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

* an `sm_80` virtual compatibility baseline and minimum compute capability
  rather than a host-specific target
* visible `nuis_kernel_vector_add_f32` and `nuis_kernel_scale_f32` entries
* an internal FNV-1a source hash
* the existing NDPB payload and artifact hash envelope

The exact two-entry sidecar source passes `ptxas` for `sm_89`. The repository-owned
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
contract. Its bytes are no longer a handwritten PTX module:
`nuis-kernel-ptx-emitter-registry-v1` consumes
`nuis-kernel-yir-codegen-function-v1` functions built from ordinary YIR
`Node`/`Operation` values. The registered vector-add and scale bodies contain
`kernel.add_f32` and `kernel.mul_f32`; the CUDA emitter supplies parameter ABI,
thread indexing, bounds checks, global loads/stores, and PTX arithmetic.
Unknown instructions fail closed. AOT caches and writes the generated
`nuis.domain.kernel.cuda.ptx` without invoking an external compiler. The
optional server-side `ptxas -arch=sm_89` differential check accepts this
module, but neither build nor execution depends on its cubin.

The `official.kernel` device-sample registration verifies the emitted bytes
before persisting two f32 input payloads and one expected output. Its generated
collection is accepted by the Nsdb parser and binds a vector-add request
followed by an f32 scale request. The second request consumes the first
request's transferable `output.values` carrier through one GLM/time-bound
dependency edge; no CUDA branch was added to the generic request parser or
scheduler.

The CUDA execution registration now consumes that fixture beneath the normal
persistent Nuis worker. The worker validates the adapter executable, PTX path
and hash, ordered input descriptors, output role, output length, and returned
hash. Its thin 64-bit Linux adapter opens `libcuda.so.1` dynamically, creates a
Driver context, loads the Nuis-emitted PTX, performs H2D/launch/D2H, and writes
the result into the worker-owned `NUISPFD1` output descriptor. The same adapter
validates and reads a transferred `NUISPFD1` frame directly for the dependent
scale launch. It uses no CUDA headers, CUDA SDK link dependency, `nvcc`, or
`ptxas`.

Device choice is provider-owned. `nuis-cuda-device-selection-registry-v1`
registers the `capability-ranked-lowest-ordinal` policy beside the CUDA runner
profile. Each request binds that policy and the code asset's minimum
capability, not a concrete device. The adapter builds a
`nuis-cuda-device-inventory-v1` inventory through `cuDeviceGetCount`,
`cuDeviceGet`, and `cuDeviceComputeCapability`; it selects the highest capable
device and uses the lowest ordinal as the deterministic tie-breaker. It emits
the inventory count, policy code, selected ordinal, minimum, and actual
capability as hash-bound `nuis-cuda-device-selection-v1` evidence. Nsdb
independently verifies those fields before publishing output. Synthetic
multi-device tests prove ordering and rejection behavior; on the current
single-GPU host both requests still record
`cuda:nvidia-gpu:ordinal-0:sm_89` against the portable `sm_80` floor.

The real RTX 4050 integration test reaches this route through
`execute_provider_samples`, compares exact `[11, 22, 33, 44]` bytes, and closes
the graph-owned output. Graph close additionally requires equal session and
worker sequence clocks, a positive Nuis dispatch receipt, matching released
output roles, and the original GLM ownership tokens. It then emits stable
`nuis-provider-completion-evidence-v1` and
`nuis-provider-glm-release-evidence-v1` records whose tokens participate in
the native-output collection hash. Nsdb now rereads the actual post-execution
payload, verifies its outer evidence hash, reconstructs the collection hash,
and independently recomputes every completion and GLM release token before
persisting the structured completion handoff. Nsld final-output replay exposes
the same verified fields beneath the existing final-image binding proof and
immutable provider-dispatch authority.

Completion is a runtime fact, so its token is not frozen into the image before
execution. Instead, the completion handoff is append-bound to the sealed image
lineage. The official post-seal heterogeneous smoke proves this boundary by
sealing first, executing only after final-image dispatch authorization, then
rerunning Nsld final-output inspection over the verified completion evidence.

The same boundary now runs on Linux. Nsld emits its own ELF64-amd64 relocatable
object; `file` and `readelf` accept the x86-64 header, 16-section table,
`__nuis_entry` symbol table, and six `R_X86_64_64` relocation records. The
generic seal path produces `kernel_cuda_provider_demo.nsb`, verifies one
registered CUDA dispatch entry, launches the Nuis-emitted PTX on the real RTX
4050 after sealing, receives exact output hashes `0xdc51cb8047a381e1` and
`0x40372e9bd3b02048`, refreshes the materialized output evidence, and exposes
two verified completion and GLM release tokens through Nsdb replay and Nsld
final output.

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
8. Verify output bytes and graph-close release. Complete.
9. Bind session/worker completion and GLM release into backend-neutral evidence.
   Complete.
10. Carry the provider-neutral payload, dispatch identity, completion token,
    and release token through Nsld. Complete.
11. Launch the sealed final image on Linux and replay its CUDA completion
    evidence through the same generic boundary. Complete.
12. Execute two ordered registered CUDA entries with direct transferable-output
    input and two replayed completion records. Complete.
13. Register ordinal/minimum-capability device selection and bind the Driver's
    actual compute capability into output evidence. Complete.

The first multi-request target is vector addition followed by f32 scaling.
An inventory-backed deterministic multi-device policy, wider tensor operations,
CUDA graphs, native cubin generation, and shader interop remain later work.
