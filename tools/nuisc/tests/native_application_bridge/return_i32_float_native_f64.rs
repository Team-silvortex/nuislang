use super::*;
const PAIR_PROBE: &str = "  %observed_pair_owner = sext i32 %arg0 to i64\n  call void @nuis_debug_print_i64(i64 %observed_pair_owner)\n  %observed_child_bits = bitcast double %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_child_bits)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_i32_float_private_pair_f64_native_exact_bits_and_order() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f64"),
        &oracle::cases("f64"),
        false,
        Some(("after_work", PAIR_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_i32_float_private_pair_f64_native_real_event_traps() {
    probe::check_with_open_words(
        &SOURCE.replace("f64", "f64"),
        &oracle::native_selected("f64"),
        true,
        Some(("after_work", PAIR_PROBE)),
        Case::open_words,
    );
}
