#import <Foundation/Foundation.h>
#import <Metal/Metal.h>
#include <limits.h>
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

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 8) {
            return fail(@"usage: metal_rgba8_render <msl-source> <vertex> <fragment> <width> <height> <vertices> <instances>");
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
        id<MTLRenderPipelineState> pipeline =
            [device newRenderPipelineStateWithDescriptor:pipelineDescriptor error:&error];
        if (pipeline == nil) {
            return fail([NSString stringWithFormat:@"Metal render pipeline unavailable: %@", error]);
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

        printf("protocol=nuis-metal-rgba8-render-provider-runner-v2\n");
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
