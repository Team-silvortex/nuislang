#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

static int fail(NSString *message) {
    fprintf(stderr, "%s\n", message.UTF8String);
    return 1;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3) {
            return fail(@"usage: metal_gray8_invert <input-path> <max-value>");
        }
        NSString *inputPath = [NSString stringWithUTF8String:argv[1]];
        uint8_t maxValue = (uint8_t)strtoul(argv[2], NULL, 10);
        NSData *inputData = [NSData dataWithContentsOfFile:inputPath];
        if (inputData == nil || inputData.length == 0) {
            return fail(@"Metal input pixel payload unavailable");
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) {
            return fail(@"Metal device unavailable");
        }

        NSString *source =
            @"#include <metal_stdlib>\n"
             "using namespace metal;\n"
             "kernel void nuis_gray8_invert(device const uchar *input [[buffer(0)]], "
             "device uchar *output [[buffer(1)]], constant uchar &maxValue [[buffer(2)]], "
             "constant uint &count [[buffer(3)]], uint gid [[thread_position_in_grid]]) { "
             "if (gid < count) { output[gid] = maxValue - min(input[gid], maxValue); } }\n";
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        if (library == nil) {
            return fail([NSString stringWithFormat:@"Metal library compilation failed: %@", error]);
        }
        id<MTLFunction> function = [library newFunctionWithName:@"nuis_gray8_invert"];
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (function == nil || pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal gray8 pipeline unavailable: %@", error]);
        }

        NSUInteger byteCount = inputData.length;
        uint32_t count = (uint32_t)byteCount;
        MTLResourceOptions options = MTLResourceStorageModeShared;
        id<MTLBuffer> inputBuffer = [device newBufferWithBytes:inputData.bytes length:byteCount options:options];
        id<MTLBuffer> outputBuffer = [device newBufferWithLength:byteCount options:options];
        id<MTLBuffer> maxBuffer = [device newBufferWithBytes:&maxValue length:sizeof(maxValue) options:options];
        id<MTLBuffer> countBuffer = [device newBufferWithBytes:&count length:sizeof(count) options:options];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> commandBuffer = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [commandBuffer computeCommandEncoder];
        if (inputBuffer == nil || outputBuffer == nil || maxBuffer == nil || countBuffer == nil ||
            queue == nil || commandBuffer == nil || encoder == nil) {
            return fail(@"Metal command resources unavailable");
        }

        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:inputBuffer offset:0 atIndex:0];
        [encoder setBuffer:outputBuffer offset:0 atIndex:1];
        [encoder setBuffer:maxBuffer offset:0 atIndex:2];
        [encoder setBuffer:countBuffer offset:0 atIndex:3];
        NSUInteger width = MIN(pipeline.maxTotalThreadsPerThreadgroup, byteCount);
        [encoder dispatchThreads:MTLSizeMake(byteCount, 1, 1)
            threadsPerThreadgroup:MTLSizeMake(MAX(width, 1), 1, 1)];
        [encoder endEncoding];
        [commandBuffer commit];
        [commandBuffer waitUntilCompleted];
        if (commandBuffer.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal command failed: %@", commandBuffer.error]);
        }

        const uint8_t *output = outputBuffer.contents;
        NSMutableString *hex = [NSMutableString stringWithCapacity:byteCount * 2];
        for (NSUInteger index = 0; index < byteCount; index++) {
            [hex appendFormat:@"%02x", output[index]];
        }
        printf("protocol=nuis-metal-gray8-provider-runner-v1\n");
        printf("status=ready\n");
        printf("device=%s\n", device.name.UTF8String);
        printf("output_bytes=%lu\n", (unsigned long)byteCount);
        printf("output_hex=%s\n", hex.UTF8String);
        return 0;
    }
}
