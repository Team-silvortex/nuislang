#include <stdint.h>
#include <stdio.h>

extern int64_t nuis_yir_entry(void);

void nuis_debug_print_i64(int64_t value) {
    printf("%lld\n", (long long)value);
}

void nuis_debug_print_bool(int32_t value) {
    printf("%s\n", value ? "true" : "false");
}

void nuis_debug_print_i32(int32_t value) {
    printf("%d\n", value);
}

void nuis_debug_print_f32(float value) {
    printf("%g\n", value);
}

void nuis_debug_print_f64(double value) {
    printf("%g\n", value);
}

int64_t host_color_bias(int64_t value) {
    int64_t biased = value + 12;
    if (biased < 0) return 0;
    if (biased > 255) return 255;
    return biased;
}

int64_t host_speed_curve(int64_t value) {
    return value * 2 + 3;
}

int64_t host_radius_curve(int64_t value) {
    return (value * 3) / 2 + 8;
}

int64_t host_mix_tick(int64_t base, int64_t tick) {
    return base + tick;
}

int64_t HostRenderCurves__color_bias(int64_t value) {
    return host_color_bias(value);
}

int64_t HostRenderCurves__speed_curve(int64_t value) {
    return host_speed_curve(value);
}

int64_t HostRenderCurves__radius_curve(int64_t value) {
    return host_radius_curve(value);
}

int64_t HostRenderCurves__mix_tick(int64_t base, int64_t tick) {
    return host_mix_tick(base, tick);
}

int64_t HostMath__speed_curve(int64_t value) {
    return host_speed_curve(value);
}

int main(void) {
    return (int)nuis_yir_entry();
}
