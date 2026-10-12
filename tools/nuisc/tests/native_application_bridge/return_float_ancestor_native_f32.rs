use super::*;
const OWNER_PROBE: &str = "  %observed_bits = bitcast float %arg0 to i32\n  %observed_owner = zext i32 %observed_bits to i64\n  call void @nuis_debug_print_i64(i64 %observed_owner)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_float_ancestor_private_bool_f32_native_exact_bits_and_order() {
    probe::check(
        &SOURCE.replace("f64", "f32"),
        &oracle::cases("f32"),
        false,
        None,
    );
}
#[test]
fn return_float_ancestor_private_bool_f32_native_real_event_traps() {
    probe::check(
        &SOURCE.replace("f64", "f32"),
        &oracle::selected("f32"),
        true,
        None,
    );
}
