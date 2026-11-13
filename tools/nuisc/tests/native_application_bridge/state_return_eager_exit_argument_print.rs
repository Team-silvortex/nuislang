#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::state_return_logical_exit_argument_print as checks;

const SOURCE: &str = include_str!("state_return_eager_exit_argument_print.ns");

// The computation and trace are identical to the field witness. Reuse its
// independent model, not a second expected-output implementation.

#[test]
fn state_return_eager_exit_requires_each_leaf_and_join_effect_site_grant() {
    checks::check_grants(SOURCE);
}

#[test]
fn state_return_eager_exit_native_executes_only_selected_checks() {
    checks::check_native(SOURCE);
}

#[test]
fn state_return_eager_exit_reference_keeps_complete_accepted_state() {
    checks::check_reference(SOURCE);
}

#[test]
fn state_return_eager_exit_keeps_work_and_helper_entry_limits() {
    checks::check_limits(SOURCE);
}

#[test]
fn state_return_eager_exit_keeps_unused_local_effects_and_preceding_statement_failures() {
    checks::check_unused(SOURCE);
}

#[test]
fn state_return_eager_exit_consumes_full_width_arm_local_results() {
    checks::check_width(SOURCE);
}
