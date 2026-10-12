use super::*;
const FIELD_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %observed_small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_small)\n  %observed_bits = bitcast float %arg2 to i32\n  %observed_float = zext i32 %observed_bits to i64\n  call void @nuis_debug_print_i64(i64 %observed_float)\n  call void @nuis_debug_print_i64(i64 %arg3)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_numeric_field_vector_f32_native_exact_bits_and_order() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f32"),
        &oracle::cases("f32"),
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_numeric_field_vector_f32_native_real_event_traps() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f32"),
        &oracle::native_selected("f32"),
        true,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
