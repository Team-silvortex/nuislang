use super::*;

#[path = "loop_carries_branch.rs"]
mod loop_carries_branch;
#[path = "loop_carries_linear.rs"]
mod loop_carries_linear;
#[path = "loop_carries_readable.rs"]
mod loop_carries_readable;
#[path = "loop_carries_refs.rs"]
mod loop_carries_refs;

#[cfg(test)]
pub(super) use loop_carries_branch::parse_loop_carry_keep_source;
pub(super) use loop_carries_branch::{
    parse_loop_carry_branch_source, parse_loop_carry_linear, tail_recursive_prev_carry_binding,
};
pub(super) use loop_carries_linear::parse_additive_carry_source;
#[cfg(test)]
pub(super) use loop_carries_readable::parse_prepared_readable_carry_source_candidate;
pub(super) use loop_carries_readable::{
    diagnose_unsupported_loop_carry_expr, encode_loop_carry_branch_source_args,
    encode_loop_carry_source_args, parse_prepared_dynamic_read_carry_source,
    parse_prepared_fixed_read_carry_source, unsupported_loop_carry_branch_source_message,
};
#[cfg(test)]
pub(super) use loop_carries_refs::parse_prepared_loop_state_ref_name_from_carry_names;
pub(super) use loop_carries_refs::{
    loop_compare_from_binary_op, loop_state_ref_into_carry_source, loop_state_ref_into_cond_source,
    parse_prepared_loop_state_ref_expr, parse_prepared_loop_state_ref_name, render_loop_compare,
    render_loop_cond_kind, render_loop_logic_op,
};

#[cfg(test)]
#[path = "loop_carries_tests.rs"]
mod tests;
