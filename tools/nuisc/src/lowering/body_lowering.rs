use super::*;

#[path = "body_calls.rs"]
mod body_calls;
#[path = "body_control.rs"]
mod body_control;
#[path = "body_effects.rs"]
mod body_effects;
#[path = "body_linear.rs"]
mod body_linear;

pub(super) use body_calls::{lower_async_call_boundary, lower_call_expr, lower_unary_cpu_expr};
pub(super) use body_control::{lower_function_body, lower_if_stmt, lower_while_stmt};
pub(super) use body_effects::{chain_statement_effect, eval_const_i64_with_env};
pub(super) use body_linear::lower_linear_stmts;

use body_control::unsupported_loop_control_stmt_message;
use body_effects::{chain_nonpure_expr_stmt, eval_const_bool_with_env, refresh_const_binding};
use body_linear::lower_inline_stmts;
