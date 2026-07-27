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

Until the Nuis provider path consumes that fixture, it does not claim:

* Kernel Nustar-owned CUDA context creation
* provider-worker-owned kernel launch
* provider-managed device-memory transfer
* completion-event or GLM release evidence

## Bring-Up Order

1. Keep the host probe deterministic and read-only. Complete.
2. Register a Linux/NVIDIA CUDA ABI target in the Kernel Nustar. Complete.
3. Lower a minimal YIR kernel into a hash-bound PTX sidecar. Complete.
4. Register a CUDA provider bundle without changing generic bundle selection.
5. Execute the PTX kernel through the persistent Nuis worker.
6. Verify output bytes, clock evidence, GLM ownership, and graph-close release.
7. Carry the provider-neutral payload and dispatch identity through Nsld.

The first execution target is vector addition. Wider tensor operations,
multi-GPU selection, CUDA graphs, and shader interop remain later work.
