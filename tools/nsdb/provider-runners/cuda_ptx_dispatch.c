#define _POSIX_C_SOURCE 200809L

/* Thin CUDA Driver adapter; request ordering remains owned by the Nuis runtime. */
#include <dlfcn.h>
#include <inttypes.h>
#include <limits.h>
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
typedef CUresult (*cu_device_get_count_fn)(int*);
typedef CUresult (*cu_device_compute_capability_fn)(int*, int*, CUdevice);
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
    cu_device_get_count_fn device_get_count;
    cu_device_compute_capability_fn device_compute_capability;
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

typedef struct {
    CUdevice device;
    uint32_t inventory_count;
    uint32_t ordinal;
    uint32_t compute_capability;
} CudaDeviceSelection;

enum {
    CUDA_CAPABILITY_RANKED_POLICY_CODE = 1,
};

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
        && LOAD_CUDA(driver, device_get_count, "cuDeviceGetCount")
        && LOAD_CUDA(
            driver,
            device_compute_capability,
            "cuDeviceComputeCapability")
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

static int select_cuda_device(
    CudaDriver* driver,
    uint32_t policy_code,
    uint32_t minimum_compute_capability,
    CudaDeviceSelection* selection) {
    int count = 0;
    if (policy_code != CUDA_CAPABILITY_RANKED_POLICY_CODE
        || driver->device_get_count(&count) != 0
        || count <= 0) return 0;
    memset(selection, 0, sizeof(*selection));
    selection->inventory_count = (uint32_t)count;
    int found = 0;
    for (int ordinal = 0; ordinal < count; ordinal++) {
        CUdevice device = 0;
        int major = 0;
        int minor = 0;
        if (driver->device_get(&device, ordinal) != 0
            || driver->device_compute_capability(&major, &minor, device) != 0
            || major <= 0
            || minor < 0
            || minor >= 10) return 0;
        uint32_t capability = (uint32_t)(major * 10 + minor);
        if (capability < minimum_compute_capability) continue;
        if (!found
            || capability > selection->compute_capability
            || (capability == selection->compute_capability
                && (uint32_t)ordinal < selection->ordinal)) {
            selection->device = device;
            selection->ordinal = (uint32_t)ordinal;
            selection->compute_capability = capability;
            found = 1;
        }
    }
    return found;
}

static int read_exact(const char* path, unsigned char* output, size_t length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    size_t read = fread(output, 1, length, file);
    int trailing = fgetc(file);
    int closed = fclose(file);
    return read == length && trailing == EOF && closed == 0;
}

static uint32_t read_u32_le(const unsigned char* bytes) {
    uint32_t value = 0;
    for (size_t index = 0; index < 4; index++) {
        value |= (uint32_t)bytes[index] << (index * 8);
    }
    return value;
}

static uint64_t read_u64_le(const unsigned char* bytes) {
    uint64_t value = 0;
    for (size_t index = 0; index < 8; index++) {
        value |= (uint64_t)bytes[index] << (index * 8);
    }
    return value;
}

static int pread_exact(int fd, unsigned char* output, size_t length, size_t offset) {
    size_t read = 0;
    while (read < length) {
        ssize_t count = pread(fd, output + read, length - read, (off_t)(offset + read));
        if (count <= 0) return 0;
        read += (size_t)count;
    }
    return 1;
}

static int read_carrier_frame(
    const char* descriptor,
    unsigned char* output,
    size_t output_length) {
    int fd = -1;
    unsigned long long frame = 0;
    unsigned long long packet_length_raw = 0;
    unsigned long long packet_hash = 0;
    char tail = '\0';
    if (sscanf(
            descriptor,
            "fd:%d:%llu:%llu:%llu%c",
            &fd,
            &frame,
            &packet_length_raw,
            &packet_hash,
            &tail) != 4
        || fd < 0
        || frame > UINT32_MAX
        || packet_length_raw > SIZE_MAX) return 0;
    size_t packet_length = (size_t)packet_length_raw;
    if (packet_length < 56) return 0;
    unsigned char* packet = malloc(packet_length);
    if (packet == NULL || !pread_exact(fd, packet, packet_length, 0)) {
        free(packet);
        return 0;
    }
    uint32_t frame_count = read_u32_le(packet + 8);
    uint32_t page_size = read_u32_le(packet + 12);
    uint32_t frame_index = read_u32_le(packet + 16);
    uint64_t payload_offset_raw = read_u64_le(packet + 24);
    uint64_t payload_length_raw = read_u64_le(packet + 32);
    uint64_t mapped_length_raw = read_u64_le(packet + 40);
    uint64_t payload_hash = read_u64_le(packet + 48);
    int valid = memcmp(packet, "NUISPFD1", 8) == 0
        && fnv1a64(packet, packet_length) == (uint64_t)packet_hash
        && frame_count == 1
        && page_size != 0
        && (page_size & (page_size - 1)) == 0
        && frame_index == (uint32_t)frame
        && payload_offset_raw <= SIZE_MAX
        && payload_length_raw == output_length
        && mapped_length_raw <= SIZE_MAX;
    size_t payload_offset = valid ? (size_t)payload_offset_raw : 0;
    size_t mapped_length = valid ? (size_t)mapped_length_raw : 0;
    valid = valid
        && payload_offset >= 56
        && payload_offset % page_size == 0
        && mapped_length % page_size == 0
        && mapped_length >= output_length
        && payload_offset <= packet_length
        && mapped_length <= packet_length - payload_offset
        && fnv1a64(packet + payload_offset, output_length) == payload_hash;
    if (valid) memcpy(output, packet + payload_offset, output_length);
    free(packet);
    return valid;
}

static int read_input(const char* source, unsigned char* output, size_t length) {
    return strncmp(source, "fd:", 3) == 0
        ? read_carrier_frame(source, output, length)
        : read_exact(source, output, length);
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

static int parse_u32(const char* text, uint32_t* value) {
    char* end = NULL;
    unsigned long parsed = strtoul(text, &end, 10);
    if (text == end || *end != '\0' || parsed > UINT32_MAX) return 0;
    *value = (uint32_t)parsed;
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
    if (argc != 8) return 2;
    int vector_add = strcmp(argv[2], "nuis_kernel_vector_add_f32") == 0;
    int scale = strcmp(argv[2], "nuis_kernel_scale_f32") == 0;
    char* scale_end = NULL;
    float scale_value = scale ? strtof(argv[4], &scale_end) : 0.0f;
    uint32_t element_count = 0;
    uint32_t device_selection_policy = 0;
    uint32_t minimum_compute_capability = 0;
    OutputDescriptor output_descriptor = {0};
    if ((!vector_add && !scale)
        || (scale && (argv[4] == scale_end || *scale_end != '\0'))
        || !parse_element_count(argv[5], &element_count)
        || !parse_u32(argv[6], &device_selection_policy)
        || device_selection_policy != CUDA_CAPABILITY_RANKED_POLICY_CODE
        || !parse_element_count(argv[7], &minimum_compute_capability)
        || !parse_output_descriptor(&output_descriptor)) return 3;
    size_t byte_length = (size_t)element_count * sizeof(float);
    if (output_descriptor.payload_length != byte_length) return 4;

    unsigned char* left = malloc(byte_length);
    unsigned char* right = vector_add ? malloc(byte_length) : NULL;
    unsigned char* output = malloc(byte_length);
    if (left == NULL || (vector_add && right == NULL) || output == NULL
        || !read_input(argv[3], left, byte_length)
        || (vector_add && !read_input(argv[4], right, byte_length))) {
        free(left);
        free(right);
        free(output);
        return 5;
    }

    CudaDriver driver;
    CudaDeviceSelection selection = {0};
    CUcontext context = NULL;
    CUmodule module = NULL;
    CUfunction function = NULL;
    CUdeviceptr device_left = 0;
    CUdeviceptr device_right = 0;
    CUdeviceptr device_output = 0;
    int ready = load_cuda_driver(&driver)
        && driver.init(0) == 0
        && select_cuda_device(
               &driver,
               device_selection_policy,
               minimum_compute_capability,
               &selection);
    ready = ready
        && driver.context_create(&context, 0, selection.device) == 0
        && driver.module_load(&module, argv[1]) == 0
        && driver.module_get_function(&function, module, argv[2]) == 0
        && driver.memory_allocate(&device_left, byte_length) == 0
        && (!vector_add || driver.memory_allocate(&device_right, byte_length) == 0)
        && driver.memory_allocate(&device_output, byte_length) == 0
        && driver.copy_host_to_device(device_left, left, byte_length) == 0
        && (!vector_add
            || driver.copy_host_to_device(device_right, right, byte_length) == 0);
    unsigned int block_size = 256;
    unsigned int grid_size = (element_count + block_size - 1) / block_size;
    void* vector_add_parameters[] = {
        &device_left,
        &device_right,
        &device_output,
        &element_count,
    };
    void* scale_parameters[] = {
        &device_left,
        &device_output,
        &element_count,
        &scale_value,
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
               vector_add ? vector_add_parameters : scale_parameters,
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
        "device_inventory_contract=nuis-cuda-device-inventory-v1\n"
        "device_inventory_count=%" PRIu32 "\n"
        "device_selection_contract=nuis-cuda-device-selection-v1\n"
        "device_selection_policy=capability-ranked-lowest-ordinal\n"
        "device_selection_policy_code=%" PRIu32 "\n"
        "device_selection_status=verified\n"
        "selected_device_ordinal=%" PRIu32 "\n"
        "minimum_compute_capability=%" PRIu32 "\n"
        "selected_compute_capability=%" PRIu32 "\n"
        "output_bytes=%zu\n"
        "output_hash=%" PRIu64 "\n",
        selection.inventory_count,
        device_selection_policy,
        selection.ordinal,
        minimum_compute_capability,
        selection.compute_capability,
        byte_length,
        hash);
    return fflush(stdout) == 0 ? 0 : 8;
}
