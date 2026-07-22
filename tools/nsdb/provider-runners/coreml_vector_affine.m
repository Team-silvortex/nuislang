#import <CoreML/CoreML.h>
#import <Foundation/Foundation.h>
#include <limits.h>
#include <string.h>
#include <sys/mman.h>
#include <unistd.h>

static NSMutableArray<NSData *> *carrierPacketOwners = nil;

static NSString *deviceKind(id<MLComputeDeviceProtocol> device) {
    if ([device isKindOfClass:[MLNeuralEngineComputeDevice class]]) {
        return @"neural-engine";
    }
    if ([device isKindOfClass:[MLGPUComputeDevice class]]) {
        return @"gpu";
    }
    if ([device isKindOfClass:[MLCPUComputeDevice class]]) {
        return @"cpu";
    }
    return @"unknown";
}

static void addDevice(NSMutableOrderedSet<NSString *> *devices,
                      id<MLComputeDeviceProtocol> device) {
    if (device != nil) {
        [devices addObject:deviceKind(device)];
    }
}

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

static BOOL emitOutput(NSData *output) {
    const char *descriptorText = getenv("NUIS_PROVIDER_OUTPUT_FD");
    if (descriptorText != NULL) {
        NSArray<NSString *> *parts = [@(descriptorText) componentsSeparatedByString:@":"];
        if (parts.count != 5 || ![parts[0] isEqualToString:@"fd"]) return NO;
        unsigned long long values[4] = {0};
        for (NSUInteger index = 0; index < 4; index++) {
            NSScanner *scanner = [NSScanner scannerWithString:parts[index + 1]];
            if (![scanner scanUnsignedLongLong:&values[index]] || !scanner.isAtEnd) return NO;
        }
        if (values[0] > INT_MAX || values[2] != output.length) return NO;
        NSUInteger written = 0;
        while (written < output.length) {
            ssize_t count = pwrite((int)values[0], (const uint8_t *)output.bytes + written,
                                   output.length - written, (off_t)(values[1] + written));
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
    const unsigned char *bytes = output.bytes;
    NSMutableString *hex = [NSMutableString stringWithCapacity:output.length * 2];
    for (NSUInteger index = 0; index < output.length; index++) {
        [hex appendFormat:@"%02x", bytes[index]];
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
    if (carrierPacketOwners == nil) carrierPacketOwners = [NSMutableArray array];
    [carrierPacketOwners addObject:packet];
    return packet;
}

static NSDictionary<NSNumber *, NSData *> *alignedCarrierFrames(NSData *packet) {
    const uint8_t *bytes = packet.bytes;
    if (packet.length < 16 || memcmp(bytes, "NUISPFD1", 8) != 0) return nil;
    uint64_t frameCount = readLittle(bytes + 8, 4);
    uint64_t pageSize = readLittle(bytes + 12, 4);
    if (pageSize == 0 || (pageSize & (pageSize - 1)) != 0 || frameCount > NSUIntegerMax) return nil;
    if (frameCount > (packet.length - 16) / 40) return nil;
    uint64_t previousEnd = (16 + frameCount * 40 + pageSize - 1) & ~(pageSize - 1);
    NSMutableDictionary<NSNumber *, NSData *> *frames = [NSMutableDictionary dictionary];
    for (uint64_t frame = 0; frame < frameCount; frame++) {
        NSUInteger cursor = 16 + (NSUInteger)frame * 40;
        uint64_t index = readLittle(bytes + cursor, 4);
        uint64_t offset = readLittle(bytes + cursor + 8, 8);
        uint64_t length = readLittle(bytes + cursor + 16, 8);
        uint64_t mappedLength = readLittle(bytes + cursor + 24, 8);
        uint64_t expectedHash = readLittle(bytes + cursor + 32, 8);
        if (index != frame || offset > NSUIntegerMax || length > NSUIntegerMax ||
            mappedLength > NSUIntegerMax || offset < previousEnd ||
            offset % pageSize != 0 || mappedLength % pageSize != 0 || mappedLength < length ||
            offset > packet.length || mappedLength > packet.length - (NSUInteger)offset) return nil;
        NSData *payload = [NSData dataWithBytesNoCopy:(void *)(bytes + (NSUInteger)offset)
                                               length:(NSUInteger)length
                                         freeWhenDone:NO];
        NSNumber *key = @(index);
        if (frames[key] != nil || fnv1a64(payload) != expectedHash) return nil;
        frames[key] = payload;
        previousEnd = offset + mappedLength;
    }
    return frames;
}

static NSDictionary<NSNumber *, NSData *> *carrierFrames(int argc, const char *argv[]) {
    BOOL usesStdin = NO;
    for (int index = 1; index < argc; index++) {
        NSString *value = @(argv[index]);
        if ([value hasPrefix:@"frame:"]) usesStdin = YES;
    }
    if (!usesStdin) return @{};
    NSData *packet = [[NSFileHandle fileHandleWithStandardInput] readDataToEndOfFile];
    const uint8_t *bytes = packet.bytes;
    if (packet.length < 12 || memcmp(bytes, "NUISPCV1", 8) != 0) return nil;
    NSUInteger cursor = 8;
    uint64_t frameCount = readLittle(bytes + cursor, 4);
    cursor += 4;
    NSMutableDictionary<NSNumber *, NSData *> *frames = [NSMutableDictionary dictionary];
    for (uint64_t frame = 0; frame < frameCount; frame++) {
        if (cursor > packet.length || packet.length - cursor < 20) return nil;
        uint64_t index = readLittle(bytes + cursor, 4);
        uint64_t length = readLittle(bytes + cursor + 4, 8);
        uint64_t expectedHash = readLittle(bytes + cursor + 12, 8);
        cursor += 20;
        if (length > NSUIntegerMax || cursor > packet.length || length > packet.length - cursor) return nil;
        NSData *payload = [packet subdataWithRange:NSMakeRange(cursor, (NSUInteger)length)];
        NSNumber *key = @(index);
        if (frames[key] != nil || fnv1a64(payload) != expectedHash) return nil;
        frames[key] = payload;
        cursor += (NSUInteger)length;
    }
    return cursor == packet.length ? frames : nil;
}

static NSData *inputData(NSString *value, NSDictionary<NSNumber *, NSData *> *frames) {
    if (![value hasPrefix:@"frame:"] && ![value hasPrefix:@"fd:"]) {
        return [NSData dataWithContentsOfFile:value];
    }
    if ([value hasPrefix:@"fd:"]) {
        int fd = -1;
        uint64_t frame = 0;
        uint64_t length = 0;
        uint64_t hash = 0;
        if (!fdDescriptor(value, &fd, &frame, &length, &hash)) return nil;
        NSData *packet = mappedCarrierPacket(fd, length, hash);
        if (packet == nil) return nil;
        NSDictionary<NSNumber *, NSData *> *mappedFrames = alignedCarrierFrames(packet);
        return mappedFrames[@(frame)];
    }
    NSString *indexText = [value substringFromIndex:6];
    NSScanner *scanner = [NSScanner scannerWithString:indexText];
    unsigned long long index = 0;
    if (![scanner scanUnsignedLongLong:&index] || !scanner.isAtEnd) return nil;
    return frames[@(index)];
}

static MLMultiArray *tensorFromInput(NSString *value, NSString *shapeText,
                                     NSDictionary<NSNumber *, NSData *> *frames, NSError **error) {
    NSData *input = inputData(value, frames);
    if (input == nil || input.length == 0 || input.length % sizeof(float) != 0) {
        return nil;
    }
    NSArray<NSString *> *dimensionTexts = [shapeText componentsSeparatedByString:@"x"];
    NSMutableArray<NSNumber *> *shape = [NSMutableArray arrayWithCapacity:dimensionTexts.count];
    NSUInteger shapeCount = 1;
    for (NSString *text in dimensionTexts) {
        NSInteger dimension = text.integerValue;
        if (dimension <= 0 || shapeCount > NSUIntegerMax / (NSUInteger)dimension) {
            return nil;
        }
        shapeCount *= (NSUInteger)dimension;
        [shape addObject:@(dimension)];
    }
    NSUInteger count = input.length / sizeof(float);
    if (shape.count == 0 || shapeCount != count) {
        return nil;
    }
    BOOL carrierBacked = [value hasPrefix:@"frame:"] || [value hasPrefix:@"fd:"];
    MLMultiArray *tensor = nil;
    if (carrierBacked) {
        NSMutableArray<NSNumber *> *strides = [NSMutableArray arrayWithCapacity:shape.count];
        NSUInteger stride = 1;
        for (NSInteger index = (NSInteger)shape.count - 1; index >= 0; index--) {
            [strides insertObject:@(stride) atIndex:0];
            stride *= shape[(NSUInteger)index].unsignedIntegerValue;
        }
        tensor = [[MLMultiArray alloc] initWithDataPointer:(void *)input.bytes
                                                    shape:shape
                                                 dataType:MLMultiArrayDataTypeFloat32
                                                  strides:strides
                                              deallocator:nil
                                                    error:error];
    } else {
        tensor = [[MLMultiArray alloc] initWithShape:shape
                                           dataType:MLMultiArrayDataTypeFloat32
                                              error:error];
    }
    if (tensor == nil) {
        return nil;
    }
    if (!carrierBacked) {
        const float *values = (const float *)input.bytes;
        for (NSUInteger index = 0; index < count; index++) {
            tensor[index] = @(values[index]);
        }
    }
    return tensor;
}

static NSUInteger elementCount(NSString *shapeText) {
    NSUInteger count = 1;
    NSArray<NSString *> *dimensions = [shapeText componentsSeparatedByString:@"x"];
    if (dimensions.count == 0) return 0;
    for (NSString *text in dimensions) {
        NSInteger dimension = text.integerValue;
        if (dimension <= 0 || count > NSUIntegerMax / (NSUInteger)dimension) return 0;
        count *= (NSUInteger)dimension;
    }
    return count;
}

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        BOOL multi = argc >= 8 && strcmp(argv[2], "--multi") == 0 && (argc - 5) % 3 == 0;
        if (argc != 6 && !multi) {
            return fail(@"usage: coreml_runner <model> <input-path> <input-feature> <output-feature> <shape> | <model> --multi <output-feature> <output-shape> (<input-feature> <input-path> <shape>)+");
        }
        NSURL *modelURL = [NSURL fileURLWithPath:@(argv[1])];
        NSError *error = nil;
        NSString *outputFeature = multi ? @(argv[3]) : @(argv[4]);
        NSString *outputShape = multi ? @(argv[4]) : @(argv[5]);
        NSUInteger outputCount = elementCount(outputShape);
        if (outputCount == 0) return fail(@"CoreML output shape is invalid");
        NSMutableDictionary<NSString *, MLFeatureValue *> *inputFeatures =
            [NSMutableDictionary dictionary];
        NSDictionary<NSNumber *, NSData *> *frames = carrierFrames(argc, argv);
        if (frames == nil) return fail(@"CoreML carrier frame packet is invalid");
        if (multi) {
            for (int index = 5; index < argc; index += 3) {
                NSString *feature = @(argv[index]);
                MLMultiArray *tensor = tensorFromInput(@(argv[index + 1]), @(argv[index + 2]), frames, &error);
                if (tensor == nil || inputFeatures[feature] != nil) {
                    return fail(@"CoreML named input is invalid or duplicated");
                }
                inputFeatures[feature] = [MLFeatureValue featureValueWithMultiArray:tensor];
            }
        } else {
            MLMultiArray *tensor = tensorFromInput(@(argv[2]), @(argv[5]), frames, &error);
            if (tensor == nil) return fail(@"CoreML input must match its contiguous f32 shape");
            inputFeatures[@(argv[3])] = [MLFeatureValue featureValueWithMultiArray:tensor];
        }
        MLModelConfiguration *configuration = [[MLModelConfiguration alloc] init];
        configuration.computeUnits = MLComputeUnitsCPUAndNeuralEngine;

        NSURL *compiledURL = [MLModel compileModelAtURL:modelURL error:&error];
        if (compiledURL == nil) {
            return fail([NSString stringWithFormat:@"CoreML model compilation failed: %@", error]);
        }
        MLModel *model = [MLModel modelWithContentsOfURL:compiledURL
                                           configuration:configuration
                                                   error:&error];
        if (model == nil) {
            return fail([NSString stringWithFormat:@"CoreML model loading failed: %@", error]);
        }
        MLDictionaryFeatureProvider *features = [[MLDictionaryFeatureProvider alloc]
            initWithDictionary:inputFeatures
                         error:&error];
        if (features == nil) {
            return fail([NSString stringWithFormat:@"CoreML input feature creation failed: %@", error]);
        }
        id<MLFeatureProvider> prediction = [model predictionFromFeatures:features error:&error];
        if (prediction == nil) {
            return fail([NSString stringWithFormat:@"CoreML prediction failed: %@", error]);
        }
        MLMultiArray *predictionTensor = [prediction featureValueForName:outputFeature].multiArrayValue;
        if (predictionTensor == nil || predictionTensor.count != outputCount) {
            return fail(@"CoreML prediction returned an invalid output tensor");
        }
        NSMutableData *output = [NSMutableData dataWithLength:outputCount * sizeof(float)];
        float *result = (float *)output.mutableBytes;
        for (NSUInteger index = 0; index < outputCount; index++) {
            result[index] = predictionTensor[index].floatValue;
        }

        NSString *computePlanStatus = @"unavailable";
        NSUInteger computePlanLayerCount = 0;
        NSString *preferredDevices = @"none";
        NSString *supportedDevices = @"none";
        if (@available(macOS 14.4, *)) {
            __block MLComputePlan *computePlan = nil;
            __block NSError *computePlanError = nil;
            dispatch_semaphore_t semaphore = dispatch_semaphore_create(0);
            [MLComputePlan loadContentsOfURL:compiledURL
                              configuration:configuration
                          completionHandler:^(MLComputePlan *plan, NSError *planError) {
                              computePlan = plan;
                              computePlanError = planError;
                              dispatch_semaphore_signal(semaphore);
                          }];
            if (dispatch_semaphore_wait(semaphore,
                    dispatch_time(DISPATCH_TIME_NOW, 30 * NSEC_PER_SEC)) != 0) {
                return fail(@"CoreML compute-plan loading timed out");
            }
            if (computePlan == nil) {
                return fail([NSString stringWithFormat:@"CoreML compute-plan loading failed: %@",
                                                       computePlanError]);
            }
            NSArray<MLModelStructureNeuralNetworkLayer *> *layers =
                computePlan.modelStructure.neuralNetwork.layers;
            NSMutableOrderedSet<NSString *> *preferred = [NSMutableOrderedSet orderedSet];
            NSMutableOrderedSet<NSString *> *supported = [NSMutableOrderedSet orderedSet];
            for (MLModelStructureNeuralNetworkLayer *layer in layers) {
                MLComputePlanDeviceUsage *usage =
                    [computePlan computeDeviceUsageForNeuralNetworkLayer:layer];
                addDevice(preferred, usage.preferredComputeDevice);
                for (id<MLComputeDeviceProtocol> device in usage.supportedComputeDevices) {
                    addDevice(supported, device);
                }
            }
            computePlanStatus = @"ready";
            computePlanLayerCount = layers.count;
            preferredDevices = [preferred.array componentsJoinedByString:@","];
            supportedDevices = [supported.array componentsJoinedByString:@","];
            if (preferredDevices.length == 0) preferredDevices = @"none";
            if (supportedDevices.length == 0) supportedDevices = @"none";
        }
        printf("protocol=nuis-coreml-model-prediction-provider-runner-v1\n");
        printf("status=ready\n");
        printf("device=CoreML.framework:MLModel:CPUAndNeuralEngine-requested\n");
        printf("compute_plan_contract=nuis-coreml-compute-plan-evidence-v1\n");
        printf("compute_plan_status=%s\n", computePlanStatus.UTF8String);
        printf("compute_plan_layer_count=%lu\n", (unsigned long)computePlanLayerCount);
        printf("compute_plan_preferred_devices=%s\n", preferredDevices.UTF8String);
        printf("compute_plan_supported_devices=%s\n", supportedDevices.UTF8String);
        printf("output_bytes=%lu\n", (unsigned long)output.length);
        if (!emitOutput(output)) return fail(@"CoreML output carrier write failed");
        return 0;
    }
}
