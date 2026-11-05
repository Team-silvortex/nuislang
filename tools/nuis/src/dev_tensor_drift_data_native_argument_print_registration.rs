use crate::dev_tensor_drift::DevTensorDriftCheckSpec;

#[path = "dev_tensor_drift_data_native_child_epoch_local_argument_print.rs"]
mod child_epoch_local_argument_print;
#[path = "dev_tensor_drift_data_native_child_progress_argument_print.rs"]
mod child_progress_argument_print;
#[path = "dev_tensor_drift_data_native_epoch_local_argument_print.rs"]
mod epoch_local_argument_print;
#[path = "dev_tensor_drift_data_native_epoch_local_plan.rs"]
mod epoch_local_plan;
#[path = "dev_tensor_drift_data_native_fallible_bound_argument_print.rs"]
mod fallible_bound_argument_print;
#[path = "dev_tensor_drift_data_native_i32_negation_argument_print.rs"]
mod i32_negation_argument_print;
#[path = "dev_tensor_drift_data_native_i32_widening_argument_print.rs"]
mod i32_widening_argument_print;
#[path = "dev_tensor_drift_data_native_i64_arithmetic_argument_print.rs"]
mod i64_arithmetic_argument_print;
#[path = "dev_tensor_drift_data_native_i64_xor_argument_print.rs"]
mod i64_xor_argument_print;
#[path = "dev_tensor_drift_data_native_local_scalar_argument_print.rs"]
mod local_scalar_argument_print;
#[path = "dev_tensor_drift_data_native_parent_bound_argument_print.rs"]
mod parent_bound_argument_print;
#[path = "dev_tensor_drift_data_native_parent_reentry_argument_print.rs"]
mod parent_reentry_argument_print;
#[path = "dev_tensor_drift_data_native_state_bound_argument_print.rs"]
mod state_bound_argument_print;
#[path = "dev_tensor_drift_data_native_state_break_argument_print.rs"]
mod state_break_argument_print;
#[path = "dev_tensor_drift_data_native_state_continue_argument_print.rs"]
mod state_continue_argument_print;
#[path = "dev_tensor_drift_data_native_state_division_exit_argument_print.rs"]
mod state_division_exit_argument_print;
#[path = "dev_tensor_drift_data_native_state_field_exit_argument_print.rs"]
mod state_field_exit_argument_print;
#[path = "dev_tensor_drift_data_native_state_grouped_argument_print.rs"]
mod state_grouped_argument_print;
#[path = "dev_tensor_drift_data_native_state_mixed_argument_print.rs"]
mod state_mixed_argument_print;
#[path = "dev_tensor_drift_data_native_state_predicate_argument_print.rs"]
mod state_predicate_argument_print;
#[path = "dev_tensor_drift_data_native_state_short_circuit_argument_print.rs"]
mod state_short_circuit_argument_print;

pub(super) const CHILD_EPOCH_LOCAL_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    child_epoch_local_argument_print::CHECKS;
pub(super) const CHILD_PROGRESS_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    child_progress_argument_print::CHECKS;
pub(super) const EPOCH_LOCAL_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    epoch_local_argument_print::CHECKS;
pub(super) const EPOCH_LOCAL_PLAN: &[DevTensorDriftCheckSpec] = epoch_local_plan::CHECKS;
pub(super) const FALLIBLE_BOUND_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    fallible_bound_argument_print::CHECKS;
pub(super) const I32_NEGATION_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    i32_negation_argument_print::CHECKS;
pub(super) const I32_WIDENING_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    i32_widening_argument_print::CHECKS;
pub(super) const I64_ARITHMETIC_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    i64_arithmetic_argument_print::CHECKS;
pub(super) const I64_XOR_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    i64_xor_argument_print::CHECKS;
pub(super) const LOCAL_SCALAR_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    local_scalar_argument_print::CHECKS;
pub(super) const PARENT_BOUND_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    parent_bound_argument_print::CHECKS;
pub(super) const PARENT_REENTRY_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    parent_reentry_argument_print::CHECKS;
pub(super) const STATE_BOUND_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_bound_argument_print::CHECKS;
pub(super) const STATE_BREAK_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_break_argument_print::CHECKS;
pub(super) const STATE_CONTINUE_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_continue_argument_print::CHECKS;
pub(super) const STATE_MIXED_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_mixed_argument_print::CHECKS;
pub(super) const STATE_PREDICATE_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_predicate_argument_print::CHECKS;
pub(super) const STATE_SHORT_CIRCUIT_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_short_circuit_argument_print::CHECKS;

pub(super) const STATE_GROUPED_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_grouped_argument_print::CHECKS;

pub(super) const STATE_FIELD_EXIT_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_field_exit_argument_print::CHECKS;

pub(super) const STATE_DIVISION_EXIT_ARGUMENT_PRINT: &[DevTensorDriftCheckSpec] =
    state_division_exit_argument_print::CHECKS;
