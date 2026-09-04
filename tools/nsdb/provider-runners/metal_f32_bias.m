#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <limits.h>
#include <mach/mach_time.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static NSMutableArray<NSData *> *carrierPacketOwners = nil;

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

static BOOL endsWith(const char *value, const char *suffix) {
    size_t valueLength = strlen(value);
    size_t suffixLength = strlen(suffix);
    return valueLength >= suffixLength &&
           strcmp(value + valueLength - suffixLength, suffix) == 0;
}

static BOOL writeOutputDescriptor(const void *bytes, NSUInteger length,
                                  NSString *descriptorText, uint64_t *hashOut) {
    NSData *output = [NSData dataWithBytesNoCopy:(void *)bytes length:length freeWhenDone:NO];
    NSArray<NSString *> *parts = [descriptorText componentsSeparatedByString:@":"];
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
    *hashOut = fnv1a64(output);
    uint8_t littleHash[8];
    for (NSUInteger index = 0; index < 8; index++) littleHash[index] = *hashOut >> (index * 8);
    return pwrite((int)values[0], littleHash, 8, (off_t)values[3]) == 8;
}

static NSArray<NSString *> *outputDescriptors(void) {
    const char *manifestText = getenv("NUIS_PROVIDER_OUTPUT_FDS");
    if (manifestText != NULL) {
        NSMutableArray<NSString *> *descriptors = [NSMutableArray array];
        for (NSString *field in [@(manifestText) componentsSeparatedByString:@","]) {
            NSRange separator = [field rangeOfString:@"="];
            if (separator.location == NSNotFound || separator.location == 0) return nil;
            [descriptors addObject:[field substringFromIndex:separator.location + 1]];
        }
        return descriptors.count > 0 ? descriptors : nil;
    }
    const char *single = getenv("NUIS_PROVIDER_OUTPUT_FD");
    return single == NULL ? @[] : @[@(single)];
}

static BOOL emitOutputs(NSArray<id<MTLBuffer>> *buffers, NSUInteger length,
                        NSArray<NSString *> *descriptors) {
    if (descriptors.count == 0) {
        if (buffers.count != 1) return NO;
        const uint8_t *bytes = buffers[0].contents;
        NSMutableString *hex = [NSMutableString stringWithCapacity:length * 2];
        for (NSUInteger index = 0; index < length; index++) {
            [hex appendFormat:@"%02x", bytes[index]];
        }
        printf("output_channel=hex-stdout\noutput_hex=%s\n", hex.UTF8String);
        return YES;
    }
    if (descriptors.count != buffers.count) return NO;
    NSMutableArray<NSNumber *> *hashes = [NSMutableArray arrayWithCapacity:buffers.count];
    for (NSUInteger index = 0; index < buffers.count; index++) {
        uint64_t hash = 0;
        if (!writeOutputDescriptor(buffers[index].contents, length, descriptors[index], &hash)) {
            return NO;
        }
        [hashes addObject:@(hash)];
    }
    if (hashes.count == 1) {
        printf("output_channel=inherited-fd\noutput_hash=%llu\n",
               hashes[0].unsignedLongLongValue);
    } else {
        printf("output_channel=inherited-fds\noutput_hashes=");
        for (NSUInteger index = 0; index < hashes.count; index++) {
            printf("%s%llu", index == 0 ? "" : ",", hashes[index].unsignedLongLongValue);
        }
        printf("\n");
    }
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
    [carrierPacketOwners addObject:packet];
    return packet;
}

static NSData *alignedCarrierFrame(NSData *packet, uint64_t requestedFrame,
                                   NSUInteger *mappedLengthOut) {
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
    *mappedLengthOut = (NSUInteger)mappedLength;
    return payload;
}

static NSData *carrierFrame(const char *argument, NSUInteger *mappedLengthOut) {
    *mappedLengthOut = 0;
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
            return alignedCarrierFrame(packet, frame, mappedLengthOut);
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
        carrierPacketOwners = [NSMutableArray array];
        BOOL argmax = argc == 4;
        BOOL u32Mode = argc >= 5 && endsWith(argv[argc - 1], "-u32");
        BOOL copyU32 = u32Mode && strcmp(argv[argc - 1], "copy-u32") == 0;
        BOOL biasMode = argc == 5 && !u32Mode;
        if (!argmax && !u32Mode && !biasMode) {
            return fail(@"usage: metal_f32_bias <input>... <metal-source> <entry> [bias|*-u32]");
        }
        NSUInteger inputCount = u32Mode ? (NSUInteger)argc - 4 : 1;
        NSMutableArray<NSData *> *inputs = [NSMutableArray arrayWithCapacity:inputCount];
        NSMutableArray<NSNumber *> *mappedLengths = [NSMutableArray arrayWithCapacity:inputCount];
        NSUInteger elementSize = u32Mode ? sizeof(uint32_t) : sizeof(float);
        NSUInteger inputLength = 0;
        for (NSUInteger index = 0; index < inputCount; index++) {
            NSUInteger mappedLength = 0;
            NSData *input = carrierFrame(argv[index + 1], &mappedLength);
            if (input == nil || input.length == 0 || input.length % elementSize != 0 ||
                (index > 0 && input.length != inputLength)) {
                return fail(u32Mode ? @"Metal u32 inputs are unavailable, misaligned, or unequal"
                                    : @"Metal f32 input unavailable or misaligned");
            }
            if (index == 0) inputLength = input.length;
            [inputs addObject:input];
            [mappedLengths addObject:@(mappedLength)];
        }
        NSUInteger sourceIndex = inputCount + 1;
        NSString *sourcePath = [NSString stringWithUTF8String:argv[sourceIndex]];
        NSString *entry = [NSString stringWithUTF8String:argv[sourceIndex + 1]];
        NSError *error = nil;
        NSString *source = [NSString stringWithContentsOfFile:sourcePath
                                                    encoding:NSUTF8StringEncoding
                                                       error:&error];
        if (source == nil || entry.length == 0) {
            return fail([NSString stringWithFormat:@"Metal code asset unavailable: %@", error]);
        }
        float bias = biasMode ? strtof(argv[argc - 1], NULL) : 0.0f;
        if (inputLength / elementSize > UINT32_MAX) return fail(@"Metal input is too large");
        uint32_t count = (uint32_t)(inputLength / elementSize);
        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) return fail(@"Metal device unavailable");
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        id<MTLFunction> function = [library newFunctionWithName:entry];
        id<MTLComputePipelineState> pipeline =
            [device newComputePipelineStateWithFunction:function error:&error];
        if (library == nil || function == nil || pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal f32 pipeline unavailable: %@", error]);
        }
        MTLResourceOptions options = MTLResourceStorageModeShared;
        NSMutableArray<id<MTLBuffer>> *inputBuffers =
            [NSMutableArray arrayWithCapacity:inputCount];
        for (NSUInteger index = 0; index < inputCount; index++) {
            NSData *input = inputs[index];
            NSUInteger mappedLength = mappedLengths[index].unsignedIntegerValue;
            id<MTLBuffer> buffer = mappedLength > 0
                ? [device newBufferWithBytesNoCopy:(void *)input.bytes
                                             length:mappedLength
                                            options:options
                                        deallocator:nil]
                : [device newBufferWithBytes:input.bytes length:input.length options:options];
            if (buffer == nil) return fail(@"Metal input buffer unavailable");
            [inputBuffers addObject:buffer];
        }
        NSArray<NSString *> *outputDescriptorList = outputDescriptors();
        if (outputDescriptorList == nil) return fail(@"Metal output descriptor manifest is invalid");
        NSUInteger outputCount = outputDescriptorList.count > 0 ? outputDescriptorList.count : 1;
        if (!u32Mode && outputCount != 1) return fail(@"Metal scalar kernels require one output");
        NSUInteger outputLength = argmax ? sizeof(uint32_t) : inputLength;
        NSMutableArray<id<MTLBuffer>> *outputBuffers =
            [NSMutableArray arrayWithCapacity:outputCount];
        for (NSUInteger index = 0; index < outputCount; index++) {
            id<MTLBuffer> buffer = [device newBufferWithLength:outputLength options:options];
            if (buffer == nil) return fail(@"Metal output buffer unavailable");
            [outputBuffers addObject:buffer];
        }
        id<MTLBuffer> biasBuffer = [device newBufferWithBytes:&bias length:sizeof(bias) options:options];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> command = [queue commandBuffer];
        id<MTLComputeCommandEncoder> encoder = [command computeCommandEncoder];
        if (biasBuffer == nil || queue == nil || command == nil || encoder == nil) {
            return fail(@"Metal f32 command resources unavailable");
        }
        [encoder setComputePipelineState:pipeline];
        for (NSUInteger index = 0; index < inputCount; index++) {
            [encoder setBuffer:inputBuffers[index] offset:0 atIndex:index];
        }
        for (NSUInteger index = 0; index < outputCount; index++) {
            [encoder setBuffer:outputBuffers[index] offset:0 atIndex:inputCount + index];
        }
        if (biasMode) {
            [encoder setBuffer:biasBuffer offset:0 atIndex:2];
        }
        NSUInteger dispatchCount = argmax ? 1 : count;
        NSUInteger width = MIN(pipeline.maxTotalThreadsPerThreadgroup, dispatchCount);
        [encoder dispatchThreads:MTLSizeMake(dispatchCount, 1, 1)
            threadsPerThreadgroup:MTLSizeMake(MAX(width, 1), 1, 1)];
        [encoder endEncoding];
        [command commit];
        [command waitUntilCompleted];
        if (command.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal command failed: %@", command.error]);
        }
        uint64_t completionClock = mach_continuous_time();
        const char *protocol = copyU32
            ? "nuis-metal-u32-copy-provider-runner-v1"
            : (u32Mode ? "nuis-metal-u32-canonical-provider-runner-v1"
                       : (argmax ? "nuis-metal-f32-argmax-provider-runner-v1"
                                 : "nuis-metal-f32-bias-provider-runner-v1"));
        printf("protocol=%s\nstatus=ready\n", protocol);
        printf("device=%s\n", device.name.UTF8String);
        printf("output_bytes=%lu\n", (unsigned long)outputLength);
        printf("completion_contract=nuis-yir-provider-physical-completion-v1\n");
        printf("completion_status=fence-observed\n");
        printf("completion_target_clock_domain=shader.clock.frame.v1\n");
        printf("completion_source_clock_domain=apple.mach-continuous.v1\n");
        printf("completion_fence_source=metal.command-buffer.completed\n");
        printf("completion_source_clock=%llu\n", (unsigned long long)completionClock);
        if (!emitOutputs(outputBuffers, outputLength, outputDescriptorList)) {
            return fail(@"Metal f32 output carrier write failed");
        }
        return 0;
    }
}
