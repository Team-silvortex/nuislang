#include <metal_stdlib>
using namespace metal;

kernel void nuis_witsage_vector_bias_f32(
    device const float* input [[buffer(0)]],
    device float* output [[buffer(1)]],
    constant float& bias [[buffer(2)]],
    uint index [[thread_position_in_grid]]) {
    output[index] = input[index] + bias;
}
