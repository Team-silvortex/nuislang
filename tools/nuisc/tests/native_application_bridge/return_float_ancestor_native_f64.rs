use super::*;
const OWNER_PROBE: &str = "  %observed_owner = bitcast double %arg0 to i64\n  call void @nuis_debug_print_i64(i64 %observed_owner)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_float_ancestor_private_bool_f64_native_exact_bits_and_order() {
    probe::check(
        &SOURCE.replace("f64", "f64"),
        &oracle::cases("f64"),
        false,
        None,
    );
}
#[test]
fn return_float_ancestor_private_bool_f64_native_real_event_traps() {
    probe::check(
        &SOURCE.replace("f64", "f64"),
        &oracle::selected("f64"),
        true,
        None,
    );
}
