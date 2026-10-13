use super::*;
const OWNER_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n";
const FIELD_PROBE: &str = "  call void @nuis_debug_print_i64(i64 %arg0)\n  %small = sext i32 %arg1 to i64\n  call void @nuis_debug_print_i64(i64 %small)\n  %narrow = bitcast float %arg2 to i32\n  %narrow_word = zext i32 %narrow to i64\n  call void @nuis_debug_print_i64(i64 %narrow_word)\n  %wide_word = bitcast double %arg3 to i64\n  call void @nuis_debug_print_i64(i64 %wide_word)\n  %flag_word = zext i1 %arg4 to i64\n  call void @nuis_debug_print_i64(i64 %flag_word)\n  %peer_word = zext i1 %arg5 to i64\n  call void @nuis_debug_print_i64(i64 %peer_word)\n";
#[path = "return_i32_ancestor_probe.rs"]
mod probe;
#[test]
fn return_early_fields_native_exact_bits_and_order() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::cases(),
        false,
        Some(("exit_work", FIELD_PROBE)),
        Case::open_words,
    );
}
#[test]
fn return_early_fields_native_real_event_traps() {
    probe::check_with_open_words(
        SOURCE,
        &oracle::selected(),
        true,
        Some(("exit_work", FIELD_PROBE)),
        Case::open_words,
    );
}
