use super::parse_nuis_module;
use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

#[path = "tests_control_flow/if_expressions.rs"]
mod if_expressions;
#[path = "tests_control_flow/loop_control_await.rs"]
mod loop_control_await;
#[path = "tests_control_flow/match_expressions.rs"]
mod match_expressions;
#[path = "tests_control_flow/mutability_conditions.rs"]
mod mutability_conditions;
