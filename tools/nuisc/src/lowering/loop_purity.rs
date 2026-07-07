use super::*;

#[path = "loop_purity_collect.rs"]
mod loop_purity_collect;
#[path = "loop_purity_expr.rs"]
mod loop_purity_expr;
#[path = "loop_purity_host_calls.rs"]
mod loop_purity_host_calls;
#[path = "loop_purity_normalize.rs"]
mod loop_purity_normalize;
#[path = "loop_purity_refs.rs"]
mod loop_purity_refs;
#[path = "loop_purity_substitute.rs"]
mod loop_purity_substitute;

pub(super) use loop_purity_collect::{
    collect_inlineable_pure_helper_exprs, collect_pure_helper_blocks,
    collect_pure_helper_functions, inline_pure_helper_calls,
};
pub(super) use loop_purity_expr::{extract_pure_branch_binding, is_terminal_branch_pure_expr};
pub(in crate::lowering) use loop_purity_host_calls::{
    prepare_guard_host_call_stmt, prepare_host_call_computed_return,
};
pub(super) use loop_purity_normalize::normalize_pure_bool_test_expr;
pub(in crate::lowering) use loop_purity_refs::expr_references_names;
pub(super) use loop_purity_substitute::{
    prepare_terminal_branch, substitute_branch_binding, substitute_prepared_loop_body,
    substitute_stmt_bindings,
};
