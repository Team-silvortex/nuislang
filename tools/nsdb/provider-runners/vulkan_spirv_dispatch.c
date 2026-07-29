#define _POSIX_C_SOURCE 200809L

/* Thin Vulkan compute adapter; Nuis owns request shape and SPIR-V identity. */
#include <dlfcn.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <unistd.h>

typedef uint32_t VkBool32;
typedef uint32_t VkFlags;
typedef uint64_t VkDeviceSize;
typedef int32_t VkResult;
typedef void* VkInstance;
typedef void* VkPhysicalDevice;
typedef void* VkDevice;
typedef void* VkQueue;
typedef void* VkBuffer;
typedef void* VkDeviceMemory;
typedef void* VkShaderModule;
typedef void* VkDescriptorSetLayout;
typedef void* VkDescriptorPool;
typedef void* VkDescriptorSet;
typedef void* VkPipelineLayout;
typedef void* VkPipeline;
typedef void* VkCommandPool;
typedef void* VkCommandBuffer;

enum {
    VK_SUCCESS = 0,
    VK_STRUCTURE_TYPE_APPLICATION_INFO = 0,
    VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO = 1,
    VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO = 2,
    VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO = 3,
    VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO = 5,
    VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO = 12,
    VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO = 16,
    VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO = 18,
    VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO = 28,
    VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO = 30,
    VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO = 32,
    VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO = 33,
    VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO = 34,
    VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET = 35,
    VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO = 39,
    VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO = 42,
    VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO = 43,
    VK_QUEUE_COMPUTE_BIT = 0x00000002,
    VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT = 0x00000002,
    VK_MEMORY_PROPERTY_HOST_COHERENT_BIT = 0x00000004,
    VK_BUFFER_USAGE_STORAGE_BUFFER_BIT = 0x00000020,
    VK_DESCRIPTOR_TYPE_STORAGE_BUFFER = 7,
    VK_SHADER_STAGE_COMPUTE_BIT = 0x00000020,
    VK_PIPELINE_BIND_POINT_COMPUTE = 1,
    VK_COMMAND_BUFFER_LEVEL_PRIMARY = 0,
    VK_SHARING_MODE_EXCLUSIVE = 0,
};

typedef struct {
    int fd;
    size_t payload_offset;
    size_t payload_length;
    size_t hash_offset;
} OutputDescriptor;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    const char* p_application_name;
    uint32_t application_version;
    const char* p_engine_name;
    uint32_t engine_version;
    uint32_t api_version;
} VkApplicationInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    const VkApplicationInfo* p_application_info;
    uint32_t enabled_layer_count;
    const char* const* pp_enabled_layer_names;
    uint32_t enabled_extension_count;
    const char* const* pp_enabled_extension_names;
} VkInstanceCreateInfo;

typedef struct {
    VkFlags queueFlags;
    uint32_t queueCount;
    uint32_t timestampValidBits;
    struct {
        uint32_t width;
        uint32_t height;
        uint32_t depth;
    } minImageTransferGranularity;
} VkQueueFamilyProperties;

typedef struct {
    VkFlags propertyFlags;
    uint32_t heapIndex;
} VkMemoryType;

typedef struct {
    VkDeviceSize size;
    VkFlags flags;
} VkMemoryHeap;

typedef struct {
    uint32_t memoryTypeCount;
    VkMemoryType memoryTypes[32];
    uint32_t memoryHeapCount;
    VkMemoryHeap memoryHeaps[16];
} VkPhysicalDeviceMemoryProperties;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t queueFamilyIndex;
    uint32_t queueCount;
    const float* pQueuePriorities;
} VkDeviceQueueCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t queue_create_info_count;
    const VkDeviceQueueCreateInfo* p_queue_create_infos;
    uint32_t enabled_layer_count;
    const char* const* pp_enabled_layer_names;
    uint32_t enabled_extension_count;
    const char* const* pp_enabled_extension_names;
    const void* p_enabled_features;
} VkDeviceCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    VkDeviceSize size;
    VkFlags usage;
    uint32_t sharingMode;
    uint32_t queueFamilyIndexCount;
    const uint32_t* pQueueFamilyIndices;
} VkBufferCreateInfo;

typedef struct {
    VkDeviceSize size;
    VkDeviceSize alignment;
    uint32_t memoryTypeBits;
} VkMemoryRequirements;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkDeviceSize allocationSize;
    uint32_t memoryTypeIndex;
} VkMemoryAllocateInfo;

typedef struct {
    VkBuffer buffer;
    VkDeviceSize offset;
    VkDeviceSize range;
} VkDescriptorBufferInfo;

typedef struct {
    uint32_t binding;
    uint32_t descriptorType;
    uint32_t descriptorCount;
    VkFlags stageFlags;
    const void* pImmutableSamplers;
} VkDescriptorSetLayoutBinding;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t bindingCount;
    const VkDescriptorSetLayoutBinding* pBindings;
} VkDescriptorSetLayoutCreateInfo;

typedef struct {
    uint32_t type;
    uint32_t descriptorCount;
} VkDescriptorPoolSize;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t maxSets;
    uint32_t poolSizeCount;
    const VkDescriptorPoolSize* pPoolSizes;
} VkDescriptorPoolCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkDescriptorPool descriptorPool;
    uint32_t descriptorSetCount;
    const VkDescriptorSetLayout* pSetLayouts;
} VkDescriptorSetAllocateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkDescriptorSet dstSet;
    uint32_t dstBinding;
    uint32_t dstArrayElement;
    uint32_t descriptorCount;
    uint32_t descriptorType;
    const void* pImageInfo;
    const VkDescriptorBufferInfo* pBufferInfo;
    const void* pTexelBufferView;
} VkWriteDescriptorSet;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    size_t codeSize;
    const uint32_t* pCode;
} VkShaderModuleCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t stage;
    VkShaderModule module;
    const char* pName;
    const void* pSpecializationInfo;
} VkPipelineShaderStageCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t setLayoutCount;
    const VkDescriptorSetLayout* pSetLayouts;
    uint32_t pushConstantRangeCount;
    const void* pPushConstantRanges;
} VkPipelineLayoutCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    VkPipelineShaderStageCreateInfo stage;
    VkPipelineLayout layout;
    VkPipeline basePipelineHandle;
    int32_t basePipelineIndex;
} VkComputePipelineCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    uint32_t queueFamilyIndex;
} VkCommandPoolCreateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkCommandPool commandPool;
    uint32_t level;
    uint32_t commandBufferCount;
} VkCommandBufferAllocateInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    VkFlags flags;
    const void* pInheritanceInfo;
} VkCommandBufferBeginInfo;

typedef struct {
    uint32_t s_type;
    const void* p_next;
    uint32_t waitSemaphoreCount;
    const void* pWaitSemaphores;
    const void* pWaitDstStageMask;
    uint32_t commandBufferCount;
    const VkCommandBuffer* pCommandBuffers;
    uint32_t signalSemaphoreCount;
    const void* pSignalSemaphores;
} VkSubmitInfo;

typedef void* (*vk_get_instance_proc_addr_fn)(VkInstance, const char*);
typedef VkResult (*vk_enumerate_instance_version_fn)(uint32_t*);
typedef VkResult (*vk_create_instance_fn)(const VkInstanceCreateInfo*, const void*, VkInstance*);
typedef void (*vk_destroy_instance_fn)(VkInstance, const void*);
typedef VkResult (*vk_enumerate_physical_devices_fn)(VkInstance, uint32_t*, VkPhysicalDevice*);
typedef void (*vk_get_queue_family_properties_fn)(VkPhysicalDevice, uint32_t*, VkQueueFamilyProperties*);
typedef void (*vk_get_memory_properties_fn)(VkPhysicalDevice, VkPhysicalDeviceMemoryProperties*);
typedef VkResult (*vk_create_device_fn)(VkPhysicalDevice, const VkDeviceCreateInfo*, const void*, VkDevice*);
typedef void* (*vk_get_device_proc_addr_fn)(VkDevice, const char*);
typedef void (*vk_destroy_device_fn)(VkDevice, const void*);
typedef void (*vk_get_device_queue_fn)(VkDevice, uint32_t, uint32_t, VkQueue*);
typedef VkResult (*vk_create_buffer_fn)(VkDevice, const VkBufferCreateInfo*, const void*, VkBuffer*);
typedef void (*vk_destroy_buffer_fn)(VkDevice, VkBuffer, const void*);
typedef void (*vk_get_buffer_memory_requirements_fn)(VkDevice, VkBuffer, VkMemoryRequirements*);
typedef VkResult (*vk_allocate_memory_fn)(VkDevice, const VkMemoryAllocateInfo*, const void*, VkDeviceMemory*);
typedef void (*vk_free_memory_fn)(VkDevice, VkDeviceMemory, const void*);
typedef VkResult (*vk_bind_buffer_memory_fn)(VkDevice, VkBuffer, VkDeviceMemory, VkDeviceSize);
typedef VkResult (*vk_map_memory_fn)(VkDevice, VkDeviceMemory, VkDeviceSize, VkDeviceSize, VkFlags, void**);
typedef void (*vk_unmap_memory_fn)(VkDevice, VkDeviceMemory);
typedef VkResult (*vk_create_descriptor_set_layout_fn)(VkDevice, const VkDescriptorSetLayoutCreateInfo*, const void*, VkDescriptorSetLayout*);
typedef void (*vk_destroy_descriptor_set_layout_fn)(VkDevice, VkDescriptorSetLayout, const void*);
typedef VkResult (*vk_create_descriptor_pool_fn)(VkDevice, const VkDescriptorPoolCreateInfo*, const void*, VkDescriptorPool*);
typedef void (*vk_destroy_descriptor_pool_fn)(VkDevice, VkDescriptorPool, const void*);
typedef VkResult (*vk_allocate_descriptor_sets_fn)(VkDevice, const VkDescriptorSetAllocateInfo*, VkDescriptorSet*);
typedef void (*vk_update_descriptor_sets_fn)(VkDevice, uint32_t, const VkWriteDescriptorSet*, uint32_t, const void*);
typedef VkResult (*vk_create_shader_module_fn)(VkDevice, const VkShaderModuleCreateInfo*, const void*, VkShaderModule*);
typedef void (*vk_destroy_shader_module_fn)(VkDevice, VkShaderModule, const void*);
typedef VkResult (*vk_create_pipeline_layout_fn)(VkDevice, const VkPipelineLayoutCreateInfo*, const void*, VkPipelineLayout*);
typedef void (*vk_destroy_pipeline_layout_fn)(VkDevice, VkPipelineLayout, const void*);
typedef VkResult (*vk_create_compute_pipelines_fn)(VkDevice, void*, uint32_t, const VkComputePipelineCreateInfo*, const void*, VkPipeline*);
typedef void (*vk_destroy_pipeline_fn)(VkDevice, VkPipeline, const void*);
typedef VkResult (*vk_create_command_pool_fn)(VkDevice, const VkCommandPoolCreateInfo*, const void*, VkCommandPool*);
typedef void (*vk_destroy_command_pool_fn)(VkDevice, VkCommandPool, const void*);
typedef VkResult (*vk_allocate_command_buffers_fn)(VkDevice, const VkCommandBufferAllocateInfo*, VkCommandBuffer*);
typedef VkResult (*vk_begin_command_buffer_fn)(VkCommandBuffer, const VkCommandBufferBeginInfo*);
typedef VkResult (*vk_end_command_buffer_fn)(VkCommandBuffer);
typedef void (*vk_cmd_bind_pipeline_fn)(VkCommandBuffer, uint32_t, VkPipeline);
typedef void (*vk_cmd_bind_descriptor_sets_fn)(VkCommandBuffer, uint32_t, VkPipelineLayout, uint32_t, uint32_t, const VkDescriptorSet*, uint32_t, const uint32_t*);
typedef void (*vk_cmd_dispatch_fn)(VkCommandBuffer, uint32_t, uint32_t, uint32_t);
typedef VkResult (*vk_queue_submit_fn)(VkQueue, uint32_t, const VkSubmitInfo*, void*);
typedef VkResult (*vk_queue_wait_idle_fn)(VkQueue);

typedef struct {
    void* library;
    vk_destroy_instance_fn destroy_instance;
    vk_enumerate_physical_devices_fn enumerate_physical_devices;
    vk_get_queue_family_properties_fn get_queue_family_properties;
    vk_get_memory_properties_fn get_memory_properties;
    vk_create_device_fn create_device;
    vk_get_device_proc_addr_fn get_device_proc_addr;
    vk_destroy_device_fn destroy_device;
    vk_get_device_queue_fn get_device_queue;
    vk_create_buffer_fn create_buffer;
    vk_destroy_buffer_fn destroy_buffer;
    vk_get_buffer_memory_requirements_fn get_buffer_memory_requirements;
    vk_allocate_memory_fn allocate_memory;
    vk_free_memory_fn free_memory;
    vk_bind_buffer_memory_fn bind_buffer_memory;
    vk_map_memory_fn map_memory;
    vk_unmap_memory_fn unmap_memory;
    vk_create_descriptor_set_layout_fn create_descriptor_set_layout;
    vk_destroy_descriptor_set_layout_fn destroy_descriptor_set_layout;
    vk_create_descriptor_pool_fn create_descriptor_pool;
    vk_destroy_descriptor_pool_fn destroy_descriptor_pool;
    vk_allocate_descriptor_sets_fn allocate_descriptor_sets;
    vk_update_descriptor_sets_fn update_descriptor_sets;
    vk_create_shader_module_fn create_shader_module;
    vk_destroy_shader_module_fn destroy_shader_module;
    vk_create_pipeline_layout_fn create_pipeline_layout;
    vk_destroy_pipeline_layout_fn destroy_pipeline_layout;
    vk_create_compute_pipelines_fn create_compute_pipelines;
    vk_destroy_pipeline_fn destroy_pipeline;
    vk_create_command_pool_fn create_command_pool;
    vk_destroy_command_pool_fn destroy_command_pool;
    vk_allocate_command_buffers_fn allocate_command_buffers;
    vk_begin_command_buffer_fn begin_command_buffer;
    vk_end_command_buffer_fn end_command_buffer;
    vk_cmd_bind_pipeline_fn cmd_bind_pipeline;
    vk_cmd_bind_descriptor_sets_fn cmd_bind_descriptor_sets;
    vk_cmd_dispatch_fn cmd_dispatch;
    vk_queue_submit_fn queue_submit;
    vk_queue_wait_idle_fn queue_wait_idle;
} VulkanApi;

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

static int parse_output_descriptor(OutputDescriptor* output) {
    const char* text = getenv("NUIS_PROVIDER_OUTPUT_FD");
    char tail = '\0';
    return text != NULL
        && sscanf(text, "fd:%d:%zu:%zu:%zu%c", &output->fd, &output->payload_offset, &output->payload_length, &output->hash_offset, &tail) == 4;
}

static int parse_u32(const char* text, uint32_t* value) {
    char* end = NULL;
    unsigned long parsed = strtoul(text, &end, 10);
    if (text == end || *end != '\0' || parsed == 0 || parsed > UINT32_MAX) return 0;
    *value = (uint32_t)parsed;
    return 1;
}

static int read_exact_file(const char* path, unsigned char* output, size_t length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL) return 0;
    size_t read = fread(output, 1, length, file);
    int trailing = fgetc(file);
    int closed = fclose(file);
    return read == length && trailing == EOF && closed == 0;
}

static int read_file_alloc(const char* path, unsigned char** output, size_t* length) {
    FILE* file = fopen(path, "rb");
    if (file == NULL || fseek(file, 0, SEEK_END) != 0) return 0;
    long end = ftell(file);
    if (end <= 0 || fseek(file, 0, SEEK_SET) != 0) {
        fclose(file);
        return 0;
    }
    unsigned char* bytes = malloc((size_t)end);
    if (bytes == NULL) {
        fclose(file);
        return 0;
    }
    size_t read = fread(bytes, 1, (size_t)end, file);
    int closed = fclose(file);
    if (read != (size_t)end || closed != 0) {
        free(bytes);
        return 0;
    }
    *output = bytes;
    *length = (size_t)end;
    return 1;
}

static int choose_compute_device(VulkanApi* vk, VkInstance instance, VkPhysicalDevice* device, uint32_t* queue_family, uint32_t* device_count) {
    if (vk->enumerate_physical_devices(instance, device_count, NULL) != VK_SUCCESS || *device_count == 0 || *device_count > 64) return 0;
    VkPhysicalDevice devices[64];
    if (vk->enumerate_physical_devices(instance, device_count, devices) != VK_SUCCESS) return 0;
    for (uint32_t index = 0; index < *device_count; index++) {
        uint32_t queue_count = 0;
        vk->get_queue_family_properties(devices[index], &queue_count, NULL);
        if (queue_count == 0 || queue_count > 128) continue;
        VkQueueFamilyProperties queues[128];
        vk->get_queue_family_properties(devices[index], &queue_count, queues);
        for (uint32_t queue = 0; queue < queue_count; queue++) {
            if ((queues[queue].queueFlags & VK_QUEUE_COMPUTE_BIT) != 0 && queues[queue].queueCount > 0) {
                *device = devices[index];
                *queue_family = queue;
                return 1;
            }
        }
    }
    return 0;
}

static uint32_t memory_type_index(VulkanApi* vk, VkPhysicalDevice device, uint32_t bits) {
    VkPhysicalDeviceMemoryProperties props;
    vk->get_memory_properties(device, &props);
    for (uint32_t index = 0; index < props.memoryTypeCount; index++) {
        VkFlags wanted = VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT;
        if ((bits & (1u << index)) != 0 && (props.memoryTypes[index].propertyFlags & wanted) == wanted) return index;
    }
    return UINT32_MAX;
}

static int create_host_buffer(VulkanApi* vk, VkPhysicalDevice physical, VkDevice device, VkDeviceSize length, VkBuffer* buffer, VkDeviceMemory* memory) {
    VkBufferCreateInfo buffer_info = {VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO, NULL, 0, length, VK_BUFFER_USAGE_STORAGE_BUFFER_BIT, VK_SHARING_MODE_EXCLUSIVE, 0, NULL};
    if (vk->create_buffer(device, &buffer_info, NULL, buffer) != VK_SUCCESS) return 0;
    VkMemoryRequirements req;
    vk->get_buffer_memory_requirements(device, *buffer, &req);
    uint32_t type = memory_type_index(vk, physical, req.memoryTypeBits);
    if (type == UINT32_MAX) return 0;
    VkMemoryAllocateInfo alloc = {VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO, NULL, req.size, type};
    return vk->allocate_memory(device, &alloc, NULL, memory) == VK_SUCCESS
        && vk->bind_buffer_memory(device, *buffer, *memory, 0) == VK_SUCCESS;
}

static int load_vulkan(VulkanApi* vk, VkInstance* instance, uint32_t* api_version) {
    memset(vk, 0, sizeof(*vk));
    vk->library = dlopen("libvulkan.so.1", RTLD_NOW | RTLD_LOCAL);
    vk_create_instance_fn create_instance = NULL;
    vk_enumerate_instance_version_fn enumerate_version = NULL;
    if (vk->library == NULL
        || !load_symbol(vk->library, "vkCreateInstance", &create_instance, sizeof(create_instance))
        || !load_symbol(vk->library, "vkEnumerateInstanceVersion", &enumerate_version, sizeof(enumerate_version))) return 0;
    *api_version = (1u << 22) | (3u << 12);
    if (enumerate_version(api_version) != VK_SUCCESS || *api_version < ((1u << 22) | (1u << 12))) return 0;
    VkApplicationInfo app = {VK_STRUCTURE_TYPE_APPLICATION_INFO, NULL, "nuis-vulkan-spirv-dispatch", 1, "nuis", 1, *api_version};
    VkInstanceCreateInfo info = {VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO, NULL, 0, &app, 0, NULL, 0, NULL};
    return create_instance(&info, NULL, instance) == VK_SUCCESS && *instance != NULL;
}

#define LOAD_INSTANCE(vk, instance, gip, field, name) \
    ((vk)->field = (void*)gip(instance, name), (vk)->field != NULL)
#define LOAD_DEVICE(vk, device, field, name) \
    ((vk)->field = (void*)(vk)->get_device_proc_addr(device, name), (vk)->field != NULL)

static int load_instance_api(VulkanApi* vk, VkInstance instance) {
    vk_get_instance_proc_addr_fn gip = NULL;
    return load_symbol(vk->library, "vkGetInstanceProcAddr", &gip, sizeof(gip))
        && LOAD_INSTANCE(vk, instance, gip, destroy_instance, "vkDestroyInstance")
        && LOAD_INSTANCE(vk, instance, gip, enumerate_physical_devices, "vkEnumeratePhysicalDevices")
        && LOAD_INSTANCE(vk, instance, gip, get_queue_family_properties, "vkGetPhysicalDeviceQueueFamilyProperties")
        && LOAD_INSTANCE(vk, instance, gip, get_memory_properties, "vkGetPhysicalDeviceMemoryProperties")
        && LOAD_INSTANCE(vk, instance, gip, create_device, "vkCreateDevice")
        && LOAD_INSTANCE(vk, instance, gip, get_device_proc_addr, "vkGetDeviceProcAddr");
}

static int load_device_api(VulkanApi* vk, VkDevice device) {
    return LOAD_DEVICE(vk, device, destroy_device, "vkDestroyDevice")
        && LOAD_DEVICE(vk, device, get_device_queue, "vkGetDeviceQueue")
        && LOAD_DEVICE(vk, device, create_buffer, "vkCreateBuffer")
        && LOAD_DEVICE(vk, device, destroy_buffer, "vkDestroyBuffer")
        && LOAD_DEVICE(vk, device, get_buffer_memory_requirements, "vkGetBufferMemoryRequirements")
        && LOAD_DEVICE(vk, device, allocate_memory, "vkAllocateMemory")
        && LOAD_DEVICE(vk, device, free_memory, "vkFreeMemory")
        && LOAD_DEVICE(vk, device, bind_buffer_memory, "vkBindBufferMemory")
        && LOAD_DEVICE(vk, device, map_memory, "vkMapMemory")
        && LOAD_DEVICE(vk, device, unmap_memory, "vkUnmapMemory")
        && LOAD_DEVICE(vk, device, create_descriptor_set_layout, "vkCreateDescriptorSetLayout")
        && LOAD_DEVICE(vk, device, destroy_descriptor_set_layout, "vkDestroyDescriptorSetLayout")
        && LOAD_DEVICE(vk, device, create_descriptor_pool, "vkCreateDescriptorPool")
        && LOAD_DEVICE(vk, device, destroy_descriptor_pool, "vkDestroyDescriptorPool")
        && LOAD_DEVICE(vk, device, allocate_descriptor_sets, "vkAllocateDescriptorSets")
        && LOAD_DEVICE(vk, device, update_descriptor_sets, "vkUpdateDescriptorSets")
        && LOAD_DEVICE(vk, device, create_shader_module, "vkCreateShaderModule")
        && LOAD_DEVICE(vk, device, destroy_shader_module, "vkDestroyShaderModule")
        && LOAD_DEVICE(vk, device, create_pipeline_layout, "vkCreatePipelineLayout")
        && LOAD_DEVICE(vk, device, destroy_pipeline_layout, "vkDestroyPipelineLayout")
        && LOAD_DEVICE(vk, device, create_compute_pipelines, "vkCreateComputePipelines")
        && LOAD_DEVICE(vk, device, destroy_pipeline, "vkDestroyPipeline")
        && LOAD_DEVICE(vk, device, create_command_pool, "vkCreateCommandPool")
        && LOAD_DEVICE(vk, device, destroy_command_pool, "vkDestroyCommandPool")
        && LOAD_DEVICE(vk, device, allocate_command_buffers, "vkAllocateCommandBuffers")
        && LOAD_DEVICE(vk, device, begin_command_buffer, "vkBeginCommandBuffer")
        && LOAD_DEVICE(vk, device, end_command_buffer, "vkEndCommandBuffer")
        && LOAD_DEVICE(vk, device, cmd_bind_pipeline, "vkCmdBindPipeline")
        && LOAD_DEVICE(vk, device, cmd_bind_descriptor_sets, "vkCmdBindDescriptorSets")
        && LOAD_DEVICE(vk, device, cmd_dispatch, "vkCmdDispatch")
        && LOAD_DEVICE(vk, device, queue_submit, "vkQueueSubmit")
        && LOAD_DEVICE(vk, device, queue_wait_idle, "vkQueueWaitIdle");
}

int main(int argc, char** argv) {
    if (argc != 5 || strcmp(argv[2], "nuis_vulkan_copy_u32") != 0) return 2;
    uint32_t element_count = 0;
    OutputDescriptor output_descriptor = {0};
    if (!parse_u32(argv[4], &element_count) || !parse_output_descriptor(&output_descriptor)) return 3;
    size_t byte_length = (size_t)element_count * sizeof(uint32_t);
    if (element_count > SIZE_MAX / sizeof(uint32_t) || output_descriptor.payload_length != byte_length) return 4;
    unsigned char* input = malloc(byte_length);
    unsigned char* spirv = NULL;
    size_t spirv_length = 0;
    if (input == NULL || !read_exact_file(argv[3], input, byte_length) || !read_file_alloc(argv[1], &spirv, &spirv_length) || spirv_length % 4 != 0) return 5;

    VulkanApi vk;
    VkInstance instance = NULL;
    uint32_t api_version = 0;
    VkPhysicalDevice physical = NULL;
    uint32_t queue_family = 0;
    uint32_t device_count = 0;
    VkDevice device = NULL;
    VkQueue queue = NULL;
    VkBuffer input_buffer = NULL, output_buffer = NULL;
    VkDeviceMemory input_memory = NULL, output_memory = NULL;
    VkDescriptorSetLayout set_layout = NULL;
    VkDescriptorPool pool = NULL;
    VkDescriptorSet set = NULL;
    VkShaderModule shader = NULL;
    VkPipelineLayout pipeline_layout = NULL;
    VkPipeline pipeline = NULL;
    VkCommandPool command_pool = NULL;
    VkCommandBuffer command = NULL;
    float priority = 1.0f;
    int ready = load_vulkan(&vk, &instance, &api_version)
        && load_instance_api(&vk, instance)
        && choose_compute_device(&vk, instance, &physical, &queue_family, &device_count);
    VkDeviceQueueCreateInfo queue_info = {VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO, NULL, 0, queue_family, 1, &priority};
    VkDeviceCreateInfo device_info = {VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO, NULL, 0, 1, &queue_info, 0, NULL, 0, NULL, NULL};
    ready = ready && vk.create_device(physical, &device_info, NULL, &device) == VK_SUCCESS && load_device_api(&vk, device);
    if (ready) vk.get_device_queue(device, queue_family, 0, &queue);
    ready = ready
        && create_host_buffer(&vk, physical, device, byte_length, &input_buffer, &input_memory)
        && create_host_buffer(&vk, physical, device, byte_length, &output_buffer, &output_memory);
    void* mapped = NULL;
    ready = ready && vk.map_memory(device, input_memory, 0, byte_length, 0, &mapped) == VK_SUCCESS;
    if (ready) {
        memcpy(mapped, input, byte_length);
        vk.unmap_memory(device, input_memory);
    }
    VkDescriptorSetLayoutBinding bindings[2] = {
        {0, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, 1, VK_SHADER_STAGE_COMPUTE_BIT, NULL},
        {1, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, 1, VK_SHADER_STAGE_COMPUTE_BIT, NULL},
    };
    VkDescriptorSetLayoutCreateInfo set_layout_info = {VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO, NULL, 0, 2, bindings};
    VkDescriptorPoolSize pool_size = {VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, 2};
    VkDescriptorPoolCreateInfo pool_info = {VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO, NULL, 0, 1, 1, &pool_size};
    ready = ready
        && vk.create_descriptor_set_layout(device, &set_layout_info, NULL, &set_layout) == VK_SUCCESS
        && vk.create_descriptor_pool(device, &pool_info, NULL, &pool) == VK_SUCCESS;
    VkDescriptorSetAllocateInfo set_info = {VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO, NULL, pool, 1, &set_layout};
    ready = ready && vk.allocate_descriptor_sets(device, &set_info, &set) == VK_SUCCESS;
    VkDescriptorBufferInfo input_info = {input_buffer, 0, byte_length};
    VkDescriptorBufferInfo output_info = {output_buffer, 0, byte_length};
    VkWriteDescriptorSet writes[2] = {
        {VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET, NULL, set, 0, 0, 1, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, NULL, &input_info, NULL},
        {VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET, NULL, set, 1, 0, 1, VK_DESCRIPTOR_TYPE_STORAGE_BUFFER, NULL, &output_info, NULL},
    };
    if (ready) vk.update_descriptor_sets(device, 2, writes, 0, NULL);
    VkShaderModuleCreateInfo shader_info = {VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO, NULL, 0, spirv_length, (const uint32_t*)spirv};
    VkPipelineLayoutCreateInfo layout_info = {VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO, NULL, 0, 1, &set_layout, 0, NULL};
    ready = ready
        && vk.create_shader_module(device, &shader_info, NULL, &shader) == VK_SUCCESS
        && vk.create_pipeline_layout(device, &layout_info, NULL, &pipeline_layout) == VK_SUCCESS;
    VkPipelineShaderStageCreateInfo stage = {VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO, NULL, 0, VK_SHADER_STAGE_COMPUTE_BIT, shader, argv[2], NULL};
    VkComputePipelineCreateInfo pipeline_info = {VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO, NULL, 0, stage, pipeline_layout, NULL, -1};
    ready = ready && vk.create_compute_pipelines(device, NULL, 1, &pipeline_info, NULL, &pipeline) == VK_SUCCESS;
    VkCommandPoolCreateInfo command_pool_info = {VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO, NULL, 0, queue_family};
    ready = ready && vk.create_command_pool(device, &command_pool_info, NULL, &command_pool) == VK_SUCCESS;
    VkCommandBufferAllocateInfo command_info = {VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO, NULL, command_pool, VK_COMMAND_BUFFER_LEVEL_PRIMARY, 1};
    ready = ready && vk.allocate_command_buffers(device, &command_info, &command) == VK_SUCCESS;
    VkCommandBufferBeginInfo begin = {VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO, NULL, 0, NULL};
    ready = ready && vk.begin_command_buffer(command, &begin) == VK_SUCCESS;
    if (ready) {
        vk.cmd_bind_pipeline(command, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
        vk.cmd_bind_descriptor_sets(command, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline_layout, 0, 1, &set, 0, NULL);
        vk.cmd_dispatch(command, element_count, 1, 1);
    }
    VkSubmitInfo submit = {4, NULL, 0, NULL, NULL, 1, &command, 0, NULL};
    ready = ready
        && vk.end_command_buffer(command) == VK_SUCCESS
        && vk.queue_submit(queue, 1, &submit, NULL) == VK_SUCCESS
        && vk.queue_wait_idle(queue) == VK_SUCCESS
        && vk.map_memory(device, output_memory, 0, byte_length, 0, &mapped) == VK_SUCCESS;
    uint64_t hash = 0;
    int persisted = 0;
    if (ready) {
        hash = fnv1a64(mapped, byte_length);
        persisted = pwrite(output_descriptor.fd, mapped, byte_length, (off_t)output_descriptor.payload_offset) == (ssize_t)byte_length
            && pwrite(output_descriptor.fd, &hash, sizeof(hash), (off_t)output_descriptor.hash_offset) == (ssize_t)sizeof(hash);
        vk.unmap_memory(device, output_memory);
    }
    if (command_pool) vk.destroy_command_pool(device, command_pool, NULL);
    if (pipeline) vk.destroy_pipeline(device, pipeline, NULL);
    if (pipeline_layout) vk.destroy_pipeline_layout(device, pipeline_layout, NULL);
    if (shader) vk.destroy_shader_module(device, shader, NULL);
    if (pool) vk.destroy_descriptor_pool(device, pool, NULL);
    if (set_layout) vk.destroy_descriptor_set_layout(device, set_layout, NULL);
    if (input_memory) vk.free_memory(device, input_memory, NULL);
    if (output_memory) vk.free_memory(device, output_memory, NULL);
    if (input_buffer) vk.destroy_buffer(device, input_buffer, NULL);
    if (output_buffer) vk.destroy_buffer(device, output_buffer, NULL);
    if (device) vk.destroy_device(device, NULL);
    if (instance) vk.destroy_instance(instance, NULL);
    if (vk.library) dlclose(vk.library);
    free(input);
    free(spirv);
    if (!ready || !persisted) return 6;
    printf(
        "protocol=nuis-vulkan-spirv-provider-runner-v1\n"
        "status=ready\n"
        "device_inventory_contract=nuis-vulkan-device-inventory-v1\n"
        "device_inventory_count=%" PRIu32 "\n"
        "device_selection_contract=nuis-vulkan-device-selection-v1\n"
        "device_selection_status=verified\n"
        "selected_device_index=0\n"
        "selected_queue_family_index=%" PRIu32 "\n"
        "instance_api_version=%" PRIu32 "\n"
        "output_bytes=%zu\n"
        "output_hash=%" PRIu64 "\n",
        device_count,
        queue_family,
        api_version,
        byte_length,
        hash);
    return fflush(stdout) == 0 ? 0 : 7;
}
