#!/usr/bin/env bash
set -u

printf 'protocol=nuis-linux-cuda-host-probe-v1\n'
printf 'os_kernel=%s\n' "$(uname -s)"
printf 'os_kernel_release=%s\n' "$(uname -r)"
printf 'architecture=%s\n' "$(uname -m)"

if command -v nvcc >/dev/null 2>&1; then
  nvcc_release="$(nvcc --version | sed -n 's/.*release \([^,]*\).*/\1/p' | tail -1)"
  printf 'nvcc_status=available\n'
  printf 'nvcc_release=%s\n' "${nvcc_release:-unknown}"
else
  printf 'nvcc_status=missing\n'
  printf 'nvcc_release=none\n'
fi

loaded_driver_version="$(
  awk '/Kernel Module/{for (i = 1; i <= NF; i++) if ($i == "Module") {print $(i + 1); exit}}' \
    /proc/driver/nvidia/version 2>/dev/null
)"
installed_driver_version="$(modinfo -F version nvidia 2>/dev/null | head -1)"
printf 'nvidia_loaded_driver_version=%s\n' "${loaded_driver_version:-none}"
printf 'nvidia_installed_driver_version=%s\n' "${installed_driver_version:-none}"

if [[ -n "$loaded_driver_version" && "$loaded_driver_version" == "$installed_driver_version" ]]; then
  printf 'nvidia_driver_consistency_status=consistent\n'
else
  printf 'nvidia_driver_consistency_status=mismatch\n'
fi

nvidia_smi_output="$(nvidia-smi --query-gpu=index,name,compute_cap,driver_version,memory.total --format=csv,noheader 2>&1)"
nvidia_smi_rc=$?
if [[ "$nvidia_smi_rc" -eq 0 ]]; then
  printf 'nvidia_smi_status=ready\n'
  printf 'nvidia_gpu_count=%s\n' "$(printf '%s\n' "$nvidia_smi_output" | wc -l | tr -d ' ')"
  printf 'nvidia_gpu_records=%s\n' "$(printf '%s\n' "$nvidia_smi_output" | paste -sd ';' -)"
else
  printf 'nvidia_smi_status=blocked\n'
  printf 'nvidia_gpu_count=unknown\n'
  if printf '%s\n' "$nvidia_smi_output" | grep -q 'Driver/library version mismatch'; then
    printf 'nvidia_smi_blocker=driver-library-version-mismatch\n'
  else
    printf 'nvidia_smi_blocker=probe-failed\n'
  fi
fi

ptx_arch="${NUIS_CUDA_PROBE_ARCH:-compute_80}"
ptx_output="$(
  printf '%s\n' \
    'extern "C" __global__ void nuis_probe_add(const float* lhs, const float* rhs, float* out) {' \
    '  unsigned int i = blockIdx.x * blockDim.x + threadIdx.x;' \
    '  out[i] = lhs[i] + rhs[i];' \
    '}' |
    nvcc -x cu --ptx "-arch=$ptx_arch" -o - - 2>&1
)"
ptx_rc=$?
if [[ "$ptx_rc" -eq 0 ]] && printf '%s\n' "$ptx_output" | grep -q 'nuis_probe_add'; then
  printf 'ptx_compile_status=ready\n'
  printf 'ptx_requested_arch=%s\n' "$ptx_arch"
  printf 'ptx_version=%s\n' "$(printf '%s\n' "$ptx_output" | awk '/^\.version/{print $2; exit}')"
  printf 'ptx_target=%s\n' "$(printf '%s\n' "$ptx_output" | awk '/^\.target/{print $2; exit}')"
  printf 'ptx_entry=nuis_probe_add\n'
else
  printf 'ptx_compile_status=blocked\n'
  printf 'ptx_requested_arch=%s\n' "$ptx_arch"
fi

if [[ "$nvidia_smi_rc" -eq 0 && "$ptx_rc" -eq 0 ]]; then
  printf 'cuda_host_status=launch-candidate\n'
  printf 'cuda_host_next_action=run-provider-kernel-smoke\n'
elif [[ "$ptx_rc" -eq 0 ]]; then
  printf 'cuda_host_status=ptx-ready-launch-blocked\n'
  printf 'cuda_host_next_action=restore-driver-library-consistency\n'
else
  printf 'cuda_host_status=blocked\n'
  printf 'cuda_host_next_action=restore-cuda-toolchain\n'
fi
