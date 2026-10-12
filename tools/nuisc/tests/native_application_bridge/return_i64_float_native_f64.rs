use super::*;
const PAIR_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %observed_child_bits = bitcast double %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_child_bits)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_i64_float_private_pair_f64_native_exact_bits_and_order() {
    probe::check(
        &SOURCE.replace("f64", "f64"),
        &oracle::cases("f64"),
        false,
        Some(("after_work", PAIR_PROBE)),
    );
}
#[test]
fn return_i64_float_private_pair_f64_native_real_event_traps() {
    probe::check(
        &SOURCE.replace("f64", "f64"),
        &oracle::selected("f64"),
        true,
        Some(("after_work", PAIR_PROBE)),
    );
}
