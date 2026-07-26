#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <limits.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static NSData *carrierPacketOwner = nil;

static int fail(NSString *message) {
    fprintf(stderr, "%s\n", message.UTF8String);
    return 1;
}

static uint64_t fnv1a64(const uint8_t *bytes, NSUInteger length) {
    uint64_t hash = 0xcbf29ce484222325ULL;
    for (NSUInteger index = 0; index < length; index++) {
        hash ^= bytes[index];
        hash *= 0x100000001b3ULL;
    }
    return hash;
}

static uint64_t readLittle(const uint8_t *bytes, NSUInteger width) {
    uint64_t value = 0;
    for (NSUInteger index = 0; index < width; index++) {
        value |= (uint64_t)bytes[index] << (index * 8);
    }
    return value;
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
    if (fnv1a64(packet.bytes, packet.length) != expectedHash) return nil;
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
    return fnv1a64(payload.bytes, payload.length) == expectedHash ? payload : nil;
}

static NSData *carrierFrame(const char *argument) {
    NSString *value = @(argument);
    if (![value hasPrefix:@"fd:"]) return [NSData dataWithContentsOfFile:value];
    int fd = -1;
    uint64_t frame = 0;
    uint64_t length = 0;
    uint64_t expectedHash = 0;
    if (!fdDescriptor(value, &fd, &frame, &length, &expectedHash)) return nil;
    NSData *packet = mappedCarrierPacket(fd, length, expectedHash);
    return packet == nil ? nil : alignedCarrierFrame(packet, frame);
}

static BOOL emitOutput(const uint8_t *bytes, NSUInteger length) {
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
            ssize_t count = pwrite((int)values[0], bytes + written, length - written,
                                   (off_t)(values[1] + written));
            if (count <= 0) return NO;
            written += (NSUInteger)count;
        }
        uint64_t hash = fnv1a64(bytes, length);
        uint8_t littleHash[8];
        for (NSUInteger index = 0; index < 8; index++) littleHash[index] = hash >> (index * 8);
        if (pwrite((int)values[0], littleHash, 8, (off_t)values[3]) != 8) return NO;
        printf("output_channel=inherited-fd\noutput_hash=%llu\n", hash);
        return YES;
    }
    NSMutableString *hex = [NSMutableString stringWithCapacity:length * 2];
    for (NSUInteger index = 0; index < length; index++) {
        [hex appendFormat:@"%02x", bytes[index]];
    }
    printf("output_channel=hex-stdout\noutput_hex=%s\n", hex.UTF8String);
    return YES;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 5) {
            return fail(@"usage: metal_gray8_unary <input-path> <operation> <scalar> <max-value>");
        }
        NSString *operation = [NSString stringWithUTF8String:argv[2]];
        uint8_t scalar = (uint8_t)strtoul(argv[3], NULL, 10);
        uint8_t maxValue = (uint8_t)strtoul(argv[4], NULL, 10);
        uint8_t operationCode = [operation isEqualToString:@"invert"] ? 0 : 1;
        if (![operation isEqualToString:@"invert"] &&
            ![operation isEqualToString:@"threshold"]) {
            return fail(@"unsupported Metal gray8 unary operation");
        }
        NSData *inputData = carrierFrame(argv[1]);
        if (inputData == nil || inputData.length == 0) {
            return fail(@"Metal input pixel payload unavailable");
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) return fail(@"Metal device unavailable");
        NSString *source =
            @"#include <metal_stdlib>\n"
             "using namespace metal;\n"
             "kernel void nuis_gray8_unary(device const uchar *input [[buffer(0)]], "
             "device uchar *output [[buffer(1)]], constant uchar &operation [[buffer(2)]], "
             "constant uchar &scalar [[buffer(3)]], constant uchar &maxValue [[buffer(4)]], "
             "constant uint &count [[buffer(5)]], uint gid [[thread_position_in_grid]]) { "
             "if (gid < count) { uchar value = min(input[gid], maxValue); "
             "output[gid] = operation == 0 ? maxValue - value : "
             "(value >= scalar ? maxValue : 0); } }\n";
        NSError *error = nil;
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        id<MTLFunction> function = [library newFunctionWithName:@"nuis_gray8_unary"];
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (library == nil || function == nil || pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal gray8 pipeline unavailable: %@", error]);
        }

        NSUInteger byteCount = inputData.length;
        uint32_t count = (uint32_t)byteCount;
        MTLResourceOptions options = MTLResourceStorageModeShared;
        id<MTLBuffer> input =
            [device newBufferWithBytes:inputData.bytes length:byteCount options:options];
        id<MTLBuffer> output = [device newBufferWithLength:byteCount options:options];
        id<MTLBuffer> op = [device newBufferWithBytes:&operationCode length:1 options:options];
        id<MTLBuffer> scalarBuffer = [device newBufferWithBytes:&scalar length:1 options:options];
        id<MTLBuffer> maxBuffer = [device newBufferWithBytes:&maxValue length:1 options:options];
        id<MTLBuffer> countBuffer = [device newBufferWithBytes:&count length:4 options:options];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> command = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
        if (input == nil || output == nil || op == nil || scalarBuffer == nil ||
            maxBuffer == nil || countBuffer == nil || encoder == nil) {
            return fail(@"Metal command resources unavailable");
        }
        [encoder setComputePipelineState:pipeline];
        [encoder setBuffer:input offset:0 atIndex:0];
        [encoder setBuffer:output offset:0 atIndex:1];
        [encoder setBuffer:op offset:0 atIndex:2];
        [encoder setBuffer:scalarBuffer offset:0 atIndex:3];
        [encoder setBuffer:maxBuffer offset:0 atIndex:4];
        [encoder setBuffer:countBuffer offset:0 atIndex:5];
        NSUInteger width = MIN(pipeline.maxTotalThreadsPerThreadgroup, byteCount);
        [encoder dispatchThreads:MTLSizeMake(byteCount, 1, 1)
            threadsPerThreadgroup:MTLSizeMake(MAX(width, 1), 1, 1)];
        [encoder endEncoding];
        [command commit];
        [command waitUntilCompleted];
        if (command.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal command failed: %@", command.error]);
        }

        const char *protocol = operationCode == 0
            ? "nuis-metal-gray8-provider-runner-v1"
            : "nuis-metal-gray8-threshold-provider-runner-v1";
        printf("protocol=%s\nstatus=ready\ndevice=%s\noutput_bytes=%lu\n",
               protocol, device.name.UTF8String, (unsigned long)byteCount);
        if (!emitOutput(output.contents, byteCount)) {
            return fail(@"Metal output carrier write failed");
        }
        return 0;
    }
}
