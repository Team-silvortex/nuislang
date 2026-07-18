#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

static int fail(NSString *message) {
    fprintf(stderr, "%s\n", message.UTF8String);
    return 1;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3) {
            return fail(@"usage: metal_u32_add <input> <delta>");
        }
        uint32_t input = (uint32_t)strtoul(argv[1], NULL, 10);
        uint32_t delta = (uint32_t)strtoul(argv[2], NULL, 10);
        uint32_t output = 0;

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            return fail(@"Metal device unavailable");
        }

        NSString *source =
            @"#include <metal_stdlib>\n"
             "using namespace metal;\n"
             "kernel void nuis_u32_add(device const uint *input [[buffer(0)]], "
             "device uint *output [[buffer(1)]], constant uint &delta [[buffer(2)]], "
             "uint gid [[thread_position_in_grid]]) { "
             "if (gid == 0) { output[0] = input[0] + delta; } }\n";
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        if (library == nil) {
            return fail([NSString stringWithFormat:@"Metal library compilation failed: %@", error]);
        }
        id<MTLFunction> function = [library newFunctionWithName:@"nuis_u32_add"];
        if (function == nil) {
            return fail(@"Metal function nuis_u32_add unavailable");
        }
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal pipeline creation failed: %@", error]);
        }

        MTLResourceOptions options = MTLResourceStorageModeShared;
        id<MTLBuffer> inputBuffer = [device newBufferWithBytes:&input length:sizeof(input) options:options];
        id<MTLBuffer> outputBuffer = [device newBufferWithBytes:&output length:sizeof(output) options:options];
        id<MTLBuffer> deltaBuffer = [device newBufferWithBytes:&delta length:sizeof(delta) options:options];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> commandBuffer = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [commandBuffer computeCommandEncoder];
        if (inputBuffer == nil || outputBuffer == nil || deltaBuffer == nil || queue == nil ||
            commandBuffer == nil || encoder == nil) {
            return fail(@"Metal command resources unavailable");
        }

        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:inputBuffer offset:0 atIndex:0];
        [encoder setBuffer:outputBuffer offset:0 atIndex:1];
        [encoder setBuffer:deltaBuffer offset:0 atIndex:2];
        [encoder dispatchThreads:MTLSizeMake(1, 1, 1)
            threadsPerThreadgroup:MTLSizeMake(1, 1, 1)];
        [encoder endEncoding];
        [commandBuffer commit];
        [commandBuffer waitUntilCompleted];
        if (commandBuffer.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal command failed: %@", commandBuffer.error]);
        }

        memcpy(&output, outputBuffer.contents, sizeof(output));
        printf("protocol=nuis-metal-provider-runner-v1\n");
        printf("status=ready\n");
        printf("device=%s\n", device.name.UTF8String);
        printf("output=%u\n", output);
        return 0;
    }
}
