#include <cuda_runtime.h>
#include <cstdio>

namespace {

constexpr int kElementCount = 4;

__global__ void nuis_vector_add(
    const float* lhs,
    const float* rhs,
    float* output,
    int element_count) {
  const int index = static_cast<int>(blockIdx.x * blockDim.x + threadIdx.x);
  if (index < element_count) {
    output[index] = lhs[index] + rhs[index];
  }
}

bool cuda_ok(cudaError_t status, const char* step) {
  if (status == cudaSuccess) {
    return true;
  }
  std::printf("status=blocked\n");
  std::printf("failed_step=%s\n", step);
  std::printf("cuda_error=%s\n", cudaGetErrorString(status));
  return false;
}

}  // namespace

int main() {
  std::printf("protocol=nuis-cuda-runtime-smoke-v1\n");

  int device = 0;
  cudaDeviceProp properties{};
  if (!cuda_ok(cudaGetDevice(&device), "get-device") ||
      !cuda_ok(cudaGetDeviceProperties(&properties, device), "get-device-properties")) {
    return 1;
  }

  const float lhs[kElementCount] = {1.0F, 2.0F, 3.0F, 4.0F};
  const float rhs[kElementCount] = {10.0F, 20.0F, 30.0F, 40.0F};
  const float expected[kElementCount] = {11.0F, 22.0F, 33.0F, 44.0F};
  float output[kElementCount] = {};
  const size_t byte_length = sizeof(lhs);

  float* device_lhs = nullptr;
  float* device_rhs = nullptr;
  float* device_output = nullptr;
  if (!cuda_ok(cudaMalloc(&device_lhs, byte_length), "allocate-lhs") ||
      !cuda_ok(cudaMalloc(&device_rhs, byte_length), "allocate-rhs") ||
      !cuda_ok(cudaMalloc(&device_output, byte_length), "allocate-output")) {
    cudaFree(device_lhs);
    cudaFree(device_rhs);
    cudaFree(device_output);
    return 1;
  }

  bool ready = cuda_ok(
                   cudaMemcpy(device_lhs, lhs, byte_length, cudaMemcpyHostToDevice),
                   "copy-lhs") &&
               cuda_ok(
                   cudaMemcpy(device_rhs, rhs, byte_length, cudaMemcpyHostToDevice),
                   "copy-rhs");
  if (ready) {
    nuis_vector_add<<<1, kElementCount>>>(
        device_lhs,
        device_rhs,
        device_output,
        kElementCount);
    ready = cuda_ok(cudaGetLastError(), "launch") &&
            cuda_ok(cudaDeviceSynchronize(), "synchronize") &&
            cuda_ok(
                cudaMemcpy(output, device_output, byte_length, cudaMemcpyDeviceToHost),
                "copy-output");
  }

  cudaFree(device_lhs);
  cudaFree(device_rhs);
  cudaFree(device_output);
  if (!ready) {
    return 1;
  }

  for (int index = 0; index < kElementCount; ++index) {
    if (output[index] != expected[index]) {
      std::printf("status=blocked\n");
      std::printf("failed_step=compare-output\n");
      std::printf("mismatch_index=%d\n", index);
      return 1;
    }
  }

  std::printf("status=ready\n");
  std::printf("device_name=%s\n", properties.name);
  std::printf(
      "compute_capability=%d.%d\n",
      properties.major,
      properties.minor);
  std::printf("element_count=%d\n", kElementCount);
  std::printf("output=11,22,33,44\n");
  return 0;
}
