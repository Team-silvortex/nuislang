use super::*;
const FIELD_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %observed_small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_small)\n  %observed_float = bitcast double %arg2 to i64\n  call void @nuis_debug_print_i64(i64 %observed_float)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_retained_private_fields_f64_native_exact_bits_and_order() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f64"),
        &oracle::cases("f64"),
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_retained_private_fields_f64_native_real_event_traps() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f64"),
        &oracle::native_selected("f64"),
        true,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
