#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]

const SOURCE: &str = include_str!("child_epoch_local_argument_print.ns");
// One outer iteration plus at most three child iterations share this budget.
const LOOP_LIMIT: u64 = 4;
const ENTRY_LIMIT: u64 = 64;

#[test]
fn child_epoch_local_native_preserves_fresh_entries_and_eager_faults() {
    super::epoch_local_argument_print::native_sequences(SOURCE, LOOP_LIMIT, ENTRY_LIMIT);
}

#[test]
fn child_epoch_local_reference_preserves_counts_state_and_faulted_close() {
    super::epoch_local_argument_print::reference_sequences(SOURCE, 1);
}

#[test]
fn child_epoch_local_native_keeps_entry_exhaustion_before_publication() {
    super::epoch_local_argument_print::entry_exhaustion(SOURCE, LOOP_LIMIT);
}

#[test]
fn child_epoch_local_native_rejects_branch_alias_and_duplicate_uses() {
    super::epoch_local_argument_print::reject_unproved_uses(SOURCE, LOOP_LIMIT, ENTRY_LIMIT);
}
