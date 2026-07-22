#import <Foundation/Foundation.h>
#import <Metal/Metal.h>

static int fail(NSString *message) {
    fprintf(stderr, "%s\n", message.UTF8String);
    return 1;
}

static NSData *inputData(const char *argument) {
    NSString *value = @(argument);
    if (![value hasPrefix:@"hex:"]) return [NSData dataWithContentsOfFile:value];
    NSString *hex = [value substringFromIndex:4];
    if (hex.length == 0 || hex.length % 2 != 0) return nil;
    NSMutableData *data = [NSMutableData dataWithCapacity:hex.length / 2];
    for (NSUInteger index = 0; index < hex.length; index += 2) {
        unsigned int byte = 0;
        NSScanner *scanner = [NSScanner scannerWithString:[hex substringWithRange:NSMakeRange(index, 2)]];
        if (![scanner scanHexInt:&byte] || !scanner.isAtEnd) return nil;
        uint8_t value = (uint8_t)byte;
        [data appendBytes:&value length:1];
    }
    return data;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3) return fail(@"usage: metal_f32_bias <input-path|hex:bytes> <bias>");
        NSData *input = inputData(argv[1]);
        if (input == nil || input.length == 0 || input.length % sizeof(float) != 0) {
            return fail(@"Metal f32 input unavailable or misaligned");
        }
        float bias = strtof(argv[2], NULL);
        uint32_t count = (uint32_t)(input.length / sizeof(float));
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) return fail(@"Metal device unavailable");
        NSString *source =
            @"#include <metal_stdlib>\nusing namespace metal;\n"
             "kernel void nuis_f32_bias(device const float *input [[buffer(0)]], "
             "device float *output [[buffer(1)]], constant float &bias [[buffer(2)]], "
             "constant uint &count [[buffer(3)]], uint gid [[thread_position_in_grid]]) { "
             "if (gid < count) output[gid] = input[gid] + bias; }\n";
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        id<MTLFunction> function = [library newFunctionWithName:@"nuis_f32_bias"];
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (library == nil || function == nil || pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal f32 pipeline unavailable: %@", error]);
        }
        MTLResourceOptions options = MTLResourceStorageModeShared;
        id<MTLBuffer> inputBuffer =
            [device newBufferWithBytes:input.bytes length:input.length options:options];
        id<MTLBuffer> outputBuffer = [device newBufferWithLength:input.length options:options];
        id<MTLBuffer> biasBuffer = [device newBufferWithBytes:&bias length:sizeof(bias) options:options];
        id<MTLBuffer> countBuffer =
            [device newBufferWithBytes:&count length:sizeof(count) options:options];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> command = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
        if (inputBuffer == nil || outputBuffer == nil || biasBuffer == nil || countBuffer == nil ||
            queue == nil || command == nil || encoder == nil) {
            return fail(@"Metal f32 command resources unavailable");
        }
        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:inputBuffer offset:0 atIndex:0];
        [encoder setBuffer:outputBuffer offset:0 atIndex:1];
        [encoder setBuffer:biasBuffer offset:0 atIndex:2];
        [encoder setBuffer:countBuffer offset:0 atIndex:3];
        NSUInteger width = MIN(pipeline.maxTotalThreadsPerThreadgroup, count);
        [encoder dispatchThreads:MTLSizeMake(count, 1, 1)
            threadsPerThreadgroup:MTLSizeMake(MAX(width, 1), 1, 1)];
        [encoder endEncoding];
        [command commit];
        [command waitUntilCompleted];
        if (command.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal f32 command failed: %@", command.error]);
        }
        const uint8_t *bytes = outputBuffer.contents;
        NSMutableString *hex = [NSMutableString stringWithCapacity:input.length * 2];
        for (NSUInteger index = 0; index < input.length; index++) {
            [hex appendFormat:@"%02x", bytes[index]];
        }
        printf("protocol=nuis-metal-f32-bias-provider-runner-v1\nstatus=ready\n");
        printf("device=%s\n", device.name.UTF8String);
        printf("output_bytes=%lu\n", (unsigned long)input.length);
        printf("output_hex=%s\n", hex.UTF8String);
        return 0;
    }
}
