use super::*;
const FIELD_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %observed_small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_small)\n  %raw_float = bitcast float %arg2 to i32\n  %observed_float = zext i32 %raw_float to i64\n  call void @nuis_debug_print_i64(i64 %observed_float)\n  %observed_bool = zext i1 %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %observed_bool)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_bool_field_vector_f32_native_exact_bits_and_order() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f32"),
        &oracle::cases("f32"),
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_bool_field_vector_f32_native_real_event_traps() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f32"),
        &oracle::native_selected("f32"),
        true,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
