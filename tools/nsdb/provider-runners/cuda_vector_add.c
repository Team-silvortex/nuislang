#define _POSIX_C_SOURCE 200809L

#include <dlfcn.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

typedef int CUresult;
typedef int CUdevice;
typedef unsigned long long CUdeviceptr;
typedef struct CUctx_st* CUcontext;
typedef struct CUmod_st* CUmodule;
typedef struct CUfunc_st* CUfunction;
typedef struct CUstream_st* CUstream;

typedef CUresult (*cu_init_fn)(unsigned int);
typedef CUresult (*cu_device_get_fn)(CUdevice*, int);
typedef CUresult (*cu_ctx_create_fn)(CUcontext*, unsigned int, CUdevice);
typedef CUresult (*cu_ctx_destroy_fn)(CUcontext);
typedef CUresult (*cu_module_load_fn)(CUmodule*, const char*);
typedef CUresult (*cu_module_unload_fn)(CUmodule);
typedef CUresult (*cu_module_get_function_fn)(CUfunction*, CUmodule, const char*);
typedef CUresult (*cu_mem_alloc_fn)(CUdeviceptr*, size_t);
typedef CUresult (*cu_mem_free_fn)(CUdeviceptr);
typedef CUresult (*cu_memcpy_htod_fn)(CUdeviceptr, const void*, size_t);
typedef CUresult (*cu_memcpy_dtoh_fn)(void*, CUdeviceptr, size_t);
typedef CUresult (*cu_launch_kernel_fn)(
    CUfunction,
    unsigned int,
    unsigned int,
    unsigned int,
    unsigned int,
    unsigned int,
    unsigned int,
    unsigned int,
    CUstream,
    void**,
    void**);
typedef CUresult (*cu_ctx_synchronize_fn)(void);

typedef struct {
    void* library;
    cu_init_fn init;
    cu_device_get_fn device_get;
    cu_ctx_create_fn context_create;
    cu_ctx_destroy_fn context_destroy;
    cu_module_load_fn module_load;
    cu_module_unload_fn module_unload;
    cu_module_get_function_fn module_get_function;
    cu_mem_alloc_fn memory_allocate;
    cu_mem_free_fn memory_free;
    cu_memcpy_htod_fn copy_host_to_device;
    cu_memcpy_dtoh_fn copy_device_to_host;
    cu_launch_kernel_fn launch_kernel;
    cu_ctx_synchronize_fn context_synchronize;
} CudaDriver;

typedef struct {
    int fd;
    size_t payload_offset;
    size_t payload_length;
    size_t hash_offset;
} OutputDescriptor;

_Static_assert(
    SIZE_MAX > UINT32_MAX,
    "CUDA provider runner requires a 64-bit host ABI");

static uint64_t fnv1a64(const unsigned char* bytes, size_t length) {
    uint64_t hash = UINT64_C(0xcbf29ce484222325);
    for (size_t index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= UINT64_C(0x100000001b3);
    }
    return hash;
}

static int load_symbol(void* library, const char* name, void* output, size_t size) {
    void* symbol = dlsym(library, name);
    if (symbol == NULL || size != sizeof(symbol)) return 0;
    memcpy(output, &symbol, size);
    return 1;
}

#define LOAD_CUDA(driver, field, symbol) \
    load_symbol((driver)->library, symbol, &(driver)->field, sizeof((driver)->field))

static int load_cuda_driver(CudaDriver* driver) {
    memset(driver, 0, sizeof(*driver));
    driver->library = dlopen("libcuda.so.1", RTLD_NOW | RTLD_LOCAL);
    return driver->library != NULL
        && LOAD_CUDA(driver, init, "cuInit")
        && LOAD_CUDA(driver, device_get, "cuDeviceGet")
        && LOAD_CUDA(driver, context_create, "cuCtxCreate_v2")
        && LOAD_CUDA(driver, context_destroy, "cuCtxDestroy_v2")
        && LOAD_CUDA(driver, module_load, "cuModuleLoad")
        && LOAD_CUDA(driver, module_unload, "cuModuleUnload")
        && LOAD_CUDA(driver, module_get_function, "cuModuleGetFunction")
        && LOAD_CUDA(driver, memory_allocate, "cuMemAlloc_v2")
        && LOAD_CUDA(driver, memory_free, "cuMemFree_v2")
        && LOAD_CUDA(driver, copy_host_to_device, "cuMemcpyHtoD_v2")
        && LOAD_CUDA(driver, copy_device_to_host, "cuMemcpyDtoH_v2")
        && LOAD_CUDA(driver, launch_kernel, "cuLaunchKernel")
        && LOAD_CUDA(driver, context_synchronize, "cuCtxSynchronize");
}

static int read_exact(const char* path, unsigned char* output, size_t length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    size_t read = fread(output, 1, length, file);
    int trailing = fgetc(file);
    int closed = fclose(file);
    return read == length && trailing == EOF && closed == 0;
}

static int parse_output_descriptor(OutputDescriptor* output) {
    const char* text = getenv("NUIS_PROVIDER_OUTPUT_FD");
    char tail = '\0';
    return text != NULL
        && sscanf(
            text,
            "fd:%d:%zu:%zu:%zu%c",
            &output->fd,
            &output->payload_offset,
            &output->payload_length,
            &output->hash_offset,
            &tail) == 4;
}

static int parse_element_count(const char* text, uint32_t* count) {
    char* end = NULL;
    unsigned long value = strtoul(text, &end, 10);
    if (text == end || *end != '\0' || value == 0 || value > UINT32_MAX) return 0;
    *count = (uint32_t)value;
    return 1;
}

static void release_cuda(
    CudaDriver* driver,
    CUdeviceptr left,
    CUdeviceptr right,
    CUdeviceptr output,
    CUmodule module,
    CUcontext context) {
    if (output != 0) driver->memory_free(output);
    if (right != 0) driver->memory_free(right);
    if (left != 0) driver->memory_free(left);
    if (module != NULL) driver->module_unload(module);
    if (context != NULL) driver->context_destroy(context);
    if (driver->library != NULL) dlclose(driver->library);
}

int main(int argc, char** argv) {
    if (argc != 6) return 2;
    uint32_t element_count = 0;
    OutputDescriptor output_descriptor = {0};
    if (!parse_element_count(argv[5], &element_count)
        || !parse_output_descriptor(&output_descriptor)) return 3;
    size_t byte_length = (size_t)element_count * sizeof(float);
    if (output_descriptor.payload_length != byte_length) return 4;

    unsigned char* left = malloc(byte_length);
    unsigned char* right = malloc(byte_length);
    unsigned char* output = malloc(byte_length);
    if (left == NULL || right == NULL || output == NULL
        || !read_exact(argv[3], left, byte_length)
        || !read_exact(argv[4], right, byte_length)) {
        free(left);
        free(right);
        free(output);
        return 5;
    }

    CudaDriver driver;
    CUdevice device = 0;
    CUcontext context = NULL;
    CUmodule module = NULL;
    CUfunction function = NULL;
    CUdeviceptr device_left = 0;
    CUdeviceptr device_right = 0;
    CUdeviceptr device_output = 0;
    int ready = load_cuda_driver(&driver)
        && driver.init(0) == 0
        && driver.device_get(&device, 0) == 0
        && driver.context_create(&context, 0, device) == 0
        && driver.module_load(&module, argv[1]) == 0
        && driver.module_get_function(&function, module, argv[2]) == 0
        && driver.memory_allocate(&device_left, byte_length) == 0
        && driver.memory_allocate(&device_right, byte_length) == 0
        && driver.memory_allocate(&device_output, byte_length) == 0
        && driver.copy_host_to_device(device_left, left, byte_length) == 0
        && driver.copy_host_to_device(device_right, right, byte_length) == 0;
    unsigned int block_size = 256;
    unsigned int grid_size = (element_count + block_size - 1) / block_size;
    void* parameters[] = {
        &device_left,
        &device_right,
        &device_output,
        &element_count,
    };
    ready = ready
        && driver.launch_kernel(
               function,
               grid_size,
               1,
               1,
               block_size,
               1,
               1,
               0,
               NULL,
               parameters,
               NULL) == 0
        && driver.context_synchronize() == 0
        && driver.copy_device_to_host(output, device_output, byte_length) == 0;
    if (!ready) {
        release_cuda(
            &driver,
            device_left,
            device_right,
            device_output,
            module,
            context);
        free(left);
        free(right);
        free(output);
        return 6;
    }

    uint64_t hash = fnv1a64(output, byte_length);
    int persisted =
        pwrite(
            output_descriptor.fd,
            output,
            byte_length,
            (off_t)output_descriptor.payload_offset) == (ssize_t)byte_length
        && pwrite(
            output_descriptor.fd,
            &hash,
            sizeof(hash),
            (off_t)output_descriptor.hash_offset) == (ssize_t)sizeof(hash);
    release_cuda(
        &driver,
        device_left,
        device_right,
        device_output,
        module,
        context);
    free(left);
    free(right);
    free(output);
    if (!persisted) return 7;
    printf(
        "protocol=nuis-cuda-ptx-driver-provider-runner-v1\n"
        "status=ready\n"
        "device=cuda-driver-device-0\n"
        "output_bytes=%zu\n"
        "output_hash=%" PRIu64 "\n",
        byte_length,
        hash);
    return fflush(stdout) == 0 ? 0 : 8;
}
