use super::*;
const OWNER_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n";
const FIELD_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %small)\n  %narrow = bitcast float %arg2 to i32\n  %narrow_word = zext i32 %narrow to i64\n  call void @nuis_debug_print_i64(i64 %narrow_word)\n  %wide_word = bitcast double %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %wide_word)\n  %flag_word = zext i1 %arg4 to i64\n  call void @nuis_debug_print_i64(i64 %flag_word)\n  %peer_word = zext i1 %arg5 to i64\n  call void @nuis_debug_print_i64(i64 %peer_word)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_multi_float_field_vector_native_exact_bits_and_order() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::cases(),
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_multi_float_field_vector_native_real_event_traps() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::native_selected(),
        true,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_multi_float_field_vector_native_stale_mutant_with_masked_wide_signs() {
    let cases = oracle::masked_cases();
    let stale = cases.iter().map(Case::stale_narrow).collect::<Vec<_>>();
    for (current, wrong) in cases.iter().zip(&stale) {
        assert_ne!(current.fields, wrong.fields);
        assert_eq!(current.owner, wrong.owner);
        assert_eq!(current.result, wrong.result);
        assert_eq!(current.trace, wrong.trace);
    }
    let source = SOURCE.replace("wide_work(c, b,", "wide_work(c, narrow_seed,");
    probe::check_with_open_words(
        &source,
        &stale,
        false,
        Some(("after_work", FIELD_PROBE)),
        Case::open_words,
    );
}
