#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <limits.h>
#include <math.h>
#include <string.h>
#include <mach/mach_time.h>
#include <unistd.h>

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

static BOOL parseDimension(const char *raw, NSUInteger *value) {
    char *end = NULL;
    unsigned long long parsed = strtoull(raw, &end, 10);
    if (end == raw || *end != '\0' || parsed == 0 || parsed > NSUIntegerMax) return NO;
    *value = (NSUInteger)parsed;
    return YES;
}

static BOOL parseUniform(const char *raw, BOOL *present, NSUInteger *slot, float values[4]) {
    *present = strcmp(raw, "none") != 0;
    if (!*present) return YES;
    char *end = NULL;
    unsigned long parsed = strtoul(raw, &end, 10);
    if (end == raw || *end != ':' || parsed > 30 || strlen(end + 1) != 32) return NO;
    *slot = parsed;
    const char *hex = end + 1;
    uint8_t bytes[16];
    for (NSUInteger i = 0; i < 16; i++) {
        unsigned int value = 0;
        for (NSUInteger j = 0; j < 2; j++) {
            char c = hex[i * 2 + j];
            if (c >= '0' && c <= '9') value = value * 16 + c - '0';
            else if (c >= 'a' && c <= 'f') value = value * 16 + c - 'a' + 10;
            else return NO;
        }
        bytes[i] = value;
    }
    for (NSUInteger i = 0; i < 4; i++) {
        uint32_t bits = (uint32_t)bytes[i * 4] | ((uint32_t)bytes[i * 4 + 1] << 8) |
                        ((uint32_t)bytes[i * 4 + 2] << 16) | ((uint32_t)bytes[i * 4 + 3] << 24);
        memcpy(&values[i], &bits, sizeof(bits));
        if (!isfinite(values[i])) return NO;
    }
    return YES;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 9) {
            return fail(@"usage: metal_rgba8_render <msl-source> <vertex> <fragment> <width> <height> <vertices> <instances> <uniform-slot:hex|none>");
        }
        NSUInteger width = 0;
        NSUInteger height = 0;
        NSUInteger vertexCount = 0;
        NSUInteger instanceCount = 0;
        if (!parseDimension(argv[4], &width) || !parseDimension(argv[5], &height) ||
            width > NSUIntegerMax / 4 || height > NSUIntegerMax / (width * 4)) {
            return fail(@"invalid Metal RGBA8 render dimensions");
        }
        if (!parseDimension(argv[6], &vertexCount) || vertexCount > 4 ||
            !parseDimension(argv[7], &instanceCount) || instanceCount > 256) {
            return fail(@"invalid Metal unbound draw counts");
        }
        BOOL hasUniform = NO;
        NSUInteger uniformSlot = 0;
        float uniformValues[4] = {0};
        if (!parseUniform(argv[8], &hasUniform, &uniformSlot, uniformValues)) {
            return fail(@"invalid immutable fragment f32x4 uniform");
        }

        NSError *error = nil;
        NSString *sourcePath = [NSString stringWithUTF8String:argv[1]];
        NSString *source = [NSString stringWithContentsOfFile:sourcePath
                                                     encoding:NSUTF8StringEncoding
                                                        error:&error];
        if (source == nil) {
            return fail([NSString stringWithFormat:@"Metal render source unavailable: %@", error]);
        }

        id<MTLDevice> device = MTLCreateSystemDefaultDevice();
        if (device == nil) return fail(@"Metal device unavailable");
        id<MTLLibrary> library = [device newLibraryWithSource:source options:nil error:&error];
        if (library == nil) {
            return fail([NSString stringWithFormat:@"Metal render library unavailable: %@", error]);
        }
        id<MTLFunction> vertex =
            [library newFunctionWithName:[NSString stringWithUTF8String:argv[2]]];
        id<MTLFunction> fragment =
            [library newFunctionWithName:[NSString stringWithUTF8String:argv[3]]];
        if (vertex == nil || fragment == nil) {
            return fail(@"Metal render stage entry unavailable");
        }

        MTLRenderPipelineDescriptor *pipelineDescriptor = [MTLRenderPipelineDescriptor new];
        pipelineDescriptor.vertexFunction = vertex;
        pipelineDescriptor.fragmentFunction = fragment;
        pipelineDescriptor.colorAttachments[0].pixelFormat = MTLPixelFormatRGBA8Unorm;
        MTLRenderPipelineReflection *reflection = nil;
        id<MTLRenderPipelineState> pipeline =
            [device newRenderPipelineStateWithDescriptor:pipelineDescriptor
                options:MTLPipelineOptionArgumentInfo | MTLPipelineOptionBufferTypeInfo
                reflection:&reflection error:&error];
        if (pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal render pipeline unavailable: %@", error]);
        }
        if (reflection == nil || reflection.vertexArguments.count != 0 ||
            reflection.fragmentArguments.count != (hasUniform ? 1 : 0)) {
            return fail(@"Metal render resource reflection differs from admitted binding count");
        }
        id<MTLBuffer> uniformBuffer = nil;
        if (hasUniform) {
            MTLArgument *argument = reflection.fragmentArguments[0];
            if (argument.type != MTLArgumentTypeBuffer || argument.index != uniformSlot ||
                argument.access != MTLArgumentAccessReadOnly || argument.bufferDataSize != 16 ||
                argument.bufferDataType != MTLDataTypeFloat4) {
                return fail(@"Metal render uniform reflection differs from f32x4 capability");
            }
            uniformBuffer = [device newBufferWithBytes:uniformValues length:sizeof(uniformValues)
                                             options:MTLResourceStorageModeShared];
            if (uniformBuffer == nil) return fail(@"Metal fragment uniform allocation failed");
        }

        MTLTextureDescriptor *textureDescriptor =
            [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                                               width:width
                                                              height:height
                                                           mipmapped:NO];
        textureDescriptor.storageMode = MTLStorageModeShared;
        textureDescriptor.usage = MTLTextureUsageRenderTarget | MTLTextureUsageShaderRead;
        id<MTLTexture> texture = [device newTextureWithDescriptor:textureDescriptor];
        id<MTLCommandQueue> queue = [device newCommandQueue];
        id<MTLCommandBuffer> command = [queue commandBuffer];
        if (texture == nil || queue == nil || command == nil) {
            return fail(@"Metal render command resources unavailable");
        }

        MTLRenderPassDescriptor *renderPass = [MTLRenderPassDescriptor renderPassDescriptor];
        renderPass.colorAttachments[0].texture = texture;
        renderPass.colorAttachments[0].loadAction = MTLLoadActionClear;
        renderPass.colorAttachments[0].storeAction = MTLStoreActionStore;
        renderPass.colorAttachments[0].clearColor = MTLClearColorMake(0.0, 0.0, 0.0, 1.0);
        id<MTLRenderCommandEncoder> encoder =
            [command renderCommandEncoderWithDescriptor:renderPass];
        if (encoder == nil) return fail(@"Metal render encoder unavailable");
        [encoder setRenderPipelineState:pipeline];
        if (hasUniform) [encoder setFragmentBuffer:uniformBuffer offset:0 atIndex:uniformSlot];
        [encoder drawPrimitives:MTLPrimitiveTypeTriangleStrip vertexStart:0
                   vertexCount:vertexCount instanceCount:instanceCount];
        [encoder endEncoding];
        [command commit];
        [command waitUntilCompleted];
        if (command.status != MTLCommandBufferStatusCompleted) {
            return fail([NSString stringWithFormat:@"Metal render command failed: %@", command.error]);
        }

        NSUInteger byteCount = width * height * 4;
        NSMutableData *output = [NSMutableData dataWithLength:byteCount];
        [texture getBytes:output.mutableBytes
              bytesPerRow:width * 4
               fromRegion:MTLRegionMake2D(0, 0, width, height)
              mipmapLevel:0];
        uint64_t completionClock = mach_continuous_time();

        printf("protocol=nuis-metal-rgba8-render-provider-runner-v3\n");
        printf("fragment_uniform_bytes=%u\n", hasUniform ? 16 : 0);
        printf("vertex_count=%lu\ninstance_count=%lu\n",
               (unsigned long)vertexCount, (unsigned long)instanceCount);
        printf("status=ready\ndevice=%s\noutput_bytes=%lu\n",
               device.name.UTF8String, (unsigned long)byteCount);
        printf("completion_contract=nuis-yir-provider-physical-completion-v1\n");
        printf("completion_status=fence-observed\n");
        printf("completion_target_clock_domain=shader.clock.frame.v1\n");
        printf("completion_source_clock_domain=apple.mach-continuous.v1\n");
        printf("completion_fence_source=metal.command-buffer.completed\n");
        printf("completion_source_clock=%llu\n", (unsigned long long)completionClock);
        if (!emitOutput(output.bytes, output.length)) {
            return fail(@"Metal RGBA8 output carrier write failed");
        }
        return 0;
    }
}
