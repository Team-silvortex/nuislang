use super::*;
const OWNER_PROBE: &str = "  %observed_owner = bitcast double %arg0 to i64\n  call void @nuis_debug_print_i64(i64 %observed_owner)\n";
const FIELD_PROBE: &str = "  %observed_owner = bitcast double %arg0 to i64\n  call void @nuis_debug_print_i64(i64 %observed_owner)\n  %observed_small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %observed_small)\n  %raw_float = bitcast float %arg2 to i32\n  %observed_float = zext i32 %raw_float to i64\n  call void @nuis_debug_print_i64(i64 %observed_float)\n  %observed_bool = zext i1 %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %observed_bool)\n  %observed_peer = zext i1 %arg4 to i64\n  call void @nuis_debug_print_i64(i64 %observed_peer)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_cross_width_field_vector_f64_native_stale_mutant_with_equality_masking() {
    let cases = oracle::masked_cases("f64");
    let stale = cases.iter().map(Case::stale_sibling).collect::<Vec<_>>();
    for (current, wrong) in cases.iter().zip(&stale) {
        assert_ne!(current.owner, wrong.owner);
        assert_eq!(current.result, wrong.result);
        assert_eq!(current.trace, wrong.trace);
    }
    let source = oracle::source("f64")
        .replace("d && peer_work(c,", "d && peer_work(bool_seed,")
        .replace("c || bool_work(d,", "c || bool_work(peer_seed,");
    probe::check_with_open_words(
        &source,
        &stale,
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_cross_width_field_vector_f64_native_exact_bits_and_order() {
    probe::check_with_open_words(
        &oracle::source("f64"),
        &oracle::cases("f64"),
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_cross_width_field_vector_f64_native_real_event_traps() {
    probe::check_with_open_words(
        &oracle::source("f64"),
        &oracle::native_selected("f64"),
        true,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
