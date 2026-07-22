#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <limits.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static NSData *carrierPacketOwner = nil;
static NSUInteger carrierMappedLength = 0;

static int fail(NSString *message) {
    fprintf(stderr, "%s\n", message.UTF8String);
    return 1;
}

static uint64_t readLittle(const uint8_t *bytes, NSUInteger width) {
    uint64_t value = 0;
    for (NSUInteger index = 0; index < width; index++) value |= (uint64_t)bytes[index] << (index * 8);
    return value;
}

static uint64_t fnv1a64(NSData *data) {
    uint64_t hash = 0xcbf29ce484222325ULL;
    const uint8_t *bytes = data.bytes;
    for (NSUInteger index = 0; index < data.length; index++) {
        hash ^= bytes[index];
        hash *= 0x100000001b3ULL;
    }
    return hash;
}

static BOOL emitOutput(const void *bytes, NSUInteger length) {
    NSData *output = [NSData dataWithBytesNoCopy:(void *)bytes length:length freeWhenDone:NO];
    const char *descriptorText = getenv("NUIS_PROVIDER_OUTPUT_FD");
    if (descriptorText != NULL) {
        NSArray<NSString *> *parts = [@(descriptorText) componentsSeparatedByString:@":"];
        if (parts.count != 5 || ![parts[0] isEqualToString:@"fd"]) return NO;
        unsigned long long values[4] = {0};
        for (NSUInteger index = 0; index < 4; index++) {
            NSScanner *scanner = [NSScanner scannerWithString:parts[index + 1]];
            if (![scanner scanUnsignedLongLong:&values[index]] || !scanner.isAtEnd) return NO;
        }
        if (values[0] > INT_MAX || values[2] != length) return NO;
        NSUInteger written = 0;
        while (written < length) {
            ssize_t count = pwrite((int)values[0], (const uint8_t *)bytes + written,
                                   length - written, (off_t)(values[1] + written));
            if (count <= 0) return NO;
            written += (NSUInteger)count;
        }
        uint64_t hash = fnv1a64(output);
        uint8_t littleHash[8];
        for (NSUInteger index = 0; index < 8; index++) littleHash[index] = hash >> (index * 8);
        if (pwrite((int)values[0], littleHash, 8, (off_t)values[3]) != 8) return NO;
        printf("output_channel=inherited-fd\noutput_hash=%llu\n", hash);
        return YES;
    }
    NSMutableString *hex = [NSMutableString stringWithCapacity:length * 2];
    for (NSUInteger index = 0; index < length; index++) {
        [hex appendFormat:@"%02x", ((const uint8_t *)bytes)[index]];
    }
    printf("output_channel=hex-stdout\noutput_hex=%s\n", hex.UTF8String);
    return YES;
}

static BOOL fdDescriptor(NSString *value, int *fd, uint64_t *frame,
                         uint64_t *length, uint64_t *hash) {
    NSArray<NSString *> *parts = [value componentsSeparatedByString:@":"];
    if (parts.count != 5 || ![parts[0] isEqualToString:@"fd"]) return NO;
    unsigned long long values[4] = {0};
    for (NSUInteger index = 0; index < 4; index++) {
        NSScanner *scanner = [NSScanner scannerWithString:parts[index + 1]];
        if (![scanner scanUnsignedLongLong:&values[index]] || !scanner.isAtEnd) return NO;
    }
    if (values[0] > INT_MAX) return NO;
    *fd = (int)values[0];
    *frame = values[1];
    *length = values[2];
    *hash = values[3];
    return YES;
}

static NSData *mappedCarrierPacket(int fd, uint64_t length, uint64_t expectedHash) {
    if (length == 0 || length > NSUIntegerMax) return nil;
    void *mapping = mmap(NULL, (size_t)length, PROT_READ, MAP_PRIVATE, fd, 0);
    close(fd);
    if (mapping == MAP_FAILED) return nil;
    NSData *packet = [[NSData alloc]
        initWithBytesNoCopy:mapping
                     length:(NSUInteger)length
                deallocator:^(void *bytes, NSUInteger mappedLength) {
                    munmap(bytes, mappedLength);
                }];
    if (fnv1a64(packet) != expectedHash) return nil;
    carrierPacketOwner = packet;
    return packet;
}

static NSData *alignedCarrierFrame(NSData *packet, uint64_t requestedFrame) {
    const uint8_t *bytes = packet.bytes;
    if (packet.length < 56 || memcmp(bytes, "NUISPFD1", 8) != 0) return nil;
    uint64_t frameCount = readLittle(bytes + 8, 4);
    uint64_t pageSize = readLittle(bytes + 12, 4);
    if (frameCount != 1 || pageSize == 0 || (pageSize & (pageSize - 1)) != 0) return nil;
    uint64_t index = readLittle(bytes + 16, 4);
    uint64_t offset = readLittle(bytes + 24, 8);
    uint64_t length = readLittle(bytes + 32, 8);
    uint64_t mappedLength = readLittle(bytes + 40, 8);
    uint64_t expectedHash = readLittle(bytes + 48, 8);
    uint64_t headerEnd = (56 + pageSize - 1) & ~(pageSize - 1);
    if (index != requestedFrame || offset > NSUIntegerMax || length > NSUIntegerMax ||
        mappedLength > NSUIntegerMax || offset < headerEnd || offset % pageSize != 0 ||
        mappedLength % pageSize != 0 || mappedLength < length || offset > packet.length ||
        mappedLength > packet.length - (NSUInteger)offset) return nil;
    NSData *payload = [NSData dataWithBytesNoCopy:(void *)(bytes + (NSUInteger)offset)
                                           length:(NSUInteger)length
                                     freeWhenDone:NO];
    if (fnv1a64(payload) != expectedHash) return nil;
    carrierMappedLength = (NSUInteger)mappedLength;
    return payload;
}

static NSData *carrierFrame(const char *argument) {
    NSString *value = @(argument);
    if (![value hasPrefix:@"frame:"] && ![value hasPrefix:@"fd:"]) {
        return [NSData dataWithContentsOfFile:value];
    }
    NSData *packet = nil;
    uint64_t frame = 0;
    BOOL mappedPacket = NO;
    if ([value hasPrefix:@"fd:"]) {
        int fd = -1;
        uint64_t length = 0;
        uint64_t expectedHash = 0;
        if (!fdDescriptor(value, &fd, &frame, &length, &expectedHash)) return nil;
        packet = mappedCarrierPacket(fd, length, expectedHash);
        if (packet == nil) return nil;
        mappedPacket = YES;
        if (packet.length >= 8 && memcmp(packet.bytes, "NUISPFD1", 8) == 0) {
            return alignedCarrierFrame(packet, frame);
        }
    } else {
        if (![value isEqualToString:@"frame:0"]) return nil;
        packet = [[NSFileHandle fileHandleWithStandardInput] readDataToEndOfFile];
    }
    if (frame != 0) return nil;
    const uint8_t *bytes = packet.bytes;
    if (packet.length < 32 || memcmp(bytes, "NUISPCV1", 8) != 0) return nil;
    if (readLittle(bytes + 8, 4) != 1 || readLittle(bytes + 12, 4) != 0) return nil;
    uint64_t length = readLittle(bytes + 16, 8);
    uint64_t expectedHash = readLittle(bytes + 24, 8);
    if (length > NSUIntegerMax || length != packet.length - 32) return nil;
    NSData *payload = mappedPacket
        ? [NSData dataWithBytesNoCopy:(void *)(bytes + 32)
                               length:(NSUInteger)length
                         freeWhenDone:NO]
        : [packet subdataWithRange:NSMakeRange(32, (NSUInteger)length)];
    return fnv1a64(payload) == expectedHash ? payload : nil;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 3) return fail(@"usage: metal_f32_bias <input-path|frame:0> <bias>");
        NSData *input = carrierFrame(argv[1]);
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
        id<MTLBuffer> inputBuffer = carrierMappedLength > 0
            ? [device newBufferWithBytesNoCopy:(void *)input.bytes
                                         length:carrierMappedLength
                                        options:options
                                    deallocator:nil]
            : [device newBufferWithBytes:input.bytes length:input.length options:options];
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
        printf("protocol=nuis-metal-f32-bias-provider-runner-v1\nstatus=ready\n");
        printf("device=%s\n", device.name.UTF8String);
        printf("output_bytes=%lu\n", (unsigned long)input.length);
        if (!emitOutput(outputBuffer.contents, input.length)) {
            return fail(@"Metal f32 output carrier write failed");
        }
        return 0;
    }
}
