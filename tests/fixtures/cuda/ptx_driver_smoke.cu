#include <cuda.h>

#include <cstdio>

namespace {

constexpr unsigned int kElementCount = 4;

bool cuda_ok(CUresult status, const char* step) {
  if (status == CUDA_SUCCESS) {
    return true;
  }
  const char* name = "unknown";
  const char* message = "unknown";
  cuGetErrorName(status, &name);
  cuGetErrorString(status, &message);
  std::printf("status=blocked\n");
  std::printf("failed_step=%s\n", step);
  std::printf("cuda_error=%s:%s\n", name, message);
  return false;
}

}  // namespace

int main(int argc, char** argv) {
  std::printf("protocol=nuis-cuda-ptx-driver-smoke-v1\n");
  if (argc != 2) {
    std::printf("status=blocked\n");
    std::printf("failed_step=ptx-path\n");
    return 2;
  }

  CUdevice device = 0;
  CUcontext context = nullptr;
  CUmodule module = nullptr;
  CUfunction function = nullptr;
  CUdeviceptr device_lhs = 0;
  CUdeviceptr device_rhs = 0;
  CUdeviceptr device_output = 0;
  char device_name[128] = {};

  bool ready = cuda_ok(cuInit(0), "initialize") &&
               cuda_ok(cuDeviceGet(&device, 0), "get-device") &&
               cuda_ok(
                   cuDeviceGetName(device_name, sizeof(device_name), device),
                   "get-device-name") &&
               cuda_ok(cuCtxCreate(&context, 0, device), "create-context") &&
               cuda_ok(cuModuleLoad(&module, argv[1]), "load-ptx") &&
               cuda_ok(
                   cuModuleGetFunction(
                       &function,
                       module,
                       "nuis_kernel_vector_add_f32"),
                   "resolve-entry");

  const float lhs[kElementCount] = {1.0F, 2.0F, 3.0F, 4.0F};
  const float rhs[kElementCount] = {10.0F, 20.0F, 30.0F, 40.0F};
  const float expected[kElementCount] = {11.0F, 22.0F, 33.0F, 44.0F};
  float output[kElementCount] = {};
  unsigned int element_count = kElementCount;
  const size_t byte_length = sizeof(lhs);

  ready = ready && cuda_ok(cuMemAlloc(&device_lhs, byte_length), "allocate-lhs") &&
          cuda_ok(cuMemAlloc(&device_rhs, byte_length), "allocate-rhs") &&
          cuda_ok(cuMemAlloc(&device_output, byte_length), "allocate-output") &&
          cuda_ok(
              cuMemcpyHtoD(device_lhs, lhs, byte_length),
              "copy-lhs") &&
          cuda_ok(
              cuMemcpyHtoD(device_rhs, rhs, byte_length),
              "copy-rhs");

  if (ready) {
    void* arguments[] = {
        &device_lhs,
        &device_rhs,
        &device_output,
        &element_count,
    };
    ready = cuda_ok(
                cuLaunchKernel(
                    function,
                    1,
                    1,
                    1,
                    kElementCount,
                    1,
                    1,
                    0,
                    nullptr,
                    arguments,
                    nullptr),
                "launch") &&
            cuda_ok(cuCtxSynchronize(), "synchronize") &&
            cuda_ok(
                cuMemcpyDtoH(output, device_output, byte_length),
                "copy-output");
  }

  if (device_lhs != 0) {
    cuMemFree(device_lhs);
  }
  if (device_rhs != 0) {
    cuMemFree(device_rhs);
  }
  if (device_output != 0) {
    cuMemFree(device_output);
  }
  if (module != nullptr) {
    cuModuleUnload(module);
  }
  if (context != nullptr) {
    cuCtxDestroy(context);
  }
  if (!ready) {
    return 1;
  }

  for (unsigned int index = 0; index < kElementCount; ++index) {
    if (output[index] != expected[index]) {
      std::printf("status=blocked\n");
      std::printf("failed_step=compare-output\n");
      std::printf("mismatch_index=%u\n", index);
      return 1;
    }
  }

  std::printf("status=ready\n");
  std::printf("device_name=%s\n", device_name);
  std::printf("entry=nuis_kernel_vector_add_f32\n");
  std::printf("output=11,22,33,44\n");
  return 0;
}
