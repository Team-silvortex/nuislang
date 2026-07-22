#import <CoreML/CoreML.h>
#import <Foundation/Foundation.h>

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

int main(int argc, const char *argv[]) {
    @autoreleasepool {
        if (argc != 6) {
            return fail(@"usage: coreml_vector_affine <model-path> <input-path> <input-feature> <output-feature> <shape>");
        }
        NSURL *modelURL = [NSURL fileURLWithPath:@(argv[1])];
        NSData *input = [NSData dataWithContentsOfFile:@(argv[2])];
        if (input == nil || input.length == 0 || input.length % sizeof(float) != 0) {
            return fail(@"CoreML input must contain contiguous f32 values");
        }
        NSError *error = nil;
        NSUInteger count = input.length / sizeof(float);
        NSArray<NSString *> *dimensionTexts = [@(argv[5]) componentsSeparatedByString:@"x"];
        NSMutableArray<NSNumber *> *shape = [NSMutableArray arrayWithCapacity:dimensionTexts.count];
        NSUInteger shapeCount = 1;
        for (NSString *text in dimensionTexts) {
            NSInteger dimension = text.integerValue;
            if (dimension <= 0 || shapeCount > NSUIntegerMax / (NSUInteger)dimension) {
                return fail(@"CoreML input shape contains an invalid dimension");
            }
            shapeCount *= (NSUInteger)dimension;
            [shape addObject:@(dimension)];
        }
        if (shape.count == 0 || shapeCount != count) {
            return fail(@"CoreML input shape does not match the input element count");
        }
        MLMultiArray *tensor = [[MLMultiArray alloc]
            initWithShape:shape
                 dataType:MLMultiArrayDataTypeFloat32
                    error:&error];
        if (tensor == nil) {
            return fail([NSString stringWithFormat:@"CoreML MLMultiArray unavailable: %@", error]);
        }
        const float *values = (const float *)input.bytes;
        for (NSUInteger index = 0; index < count; index++) {
            tensor[index] = @(values[index]);
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
        NSString *inputFeature = @(argv[3]);
        NSString *outputFeature = @(argv[4]);
        MLDictionaryFeatureProvider *features = [[MLDictionaryFeatureProvider alloc]
            initWithDictionary:@{ inputFeature: [MLFeatureValue featureValueWithMultiArray:tensor] }
                         error:&error];
        if (features == nil) {
            return fail([NSString stringWithFormat:@"CoreML input feature creation failed: %@", error]);
        }
        id<MLFeatureProvider> prediction = [model predictionFromFeatures:features error:&error];
        if (prediction == nil) {
            return fail([NSString stringWithFormat:@"CoreML prediction failed: %@", error]);
        }
        MLMultiArray *predictionTensor = [prediction featureValueForName:outputFeature].multiArrayValue;
        if (predictionTensor == nil || predictionTensor.count != count) {
            return fail(@"CoreML prediction returned an invalid output tensor");
        }
        NSMutableData *output = [NSMutableData dataWithLength:input.length];
        float *result = (float *)output.mutableBytes;
        for (NSUInteger index = 0; index < count; index++) {
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
        const unsigned char *bytes = output.bytes;
        NSMutableString *hex = [NSMutableString stringWithCapacity:output.length * 2];
        for (NSUInteger index = 0; index < output.length; index++) {
            [hex appendFormat:@"%02x", bytes[index]];
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
        printf("output_hex=%s\n", hex.UTF8String);
        return 0;
    }
}
