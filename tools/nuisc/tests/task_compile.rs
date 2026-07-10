use std::path::Path;

use nuis_semantics::model::{NirBinaryOp, NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

#[path = "task_compile/expression_call_and_field.rs"]
mod expression_call_and_field;
#[path = "task_compile/expression_if.rs"]
mod expression_if;
#[path = "task_compile/expression_match_operand.rs"]
mod expression_match_operand;
#[path = "task_compile/expression_receiver_helper.rs"]
mod expression_receiver_helper;
#[path = "task_compile/httpish.rs"]
mod httpish;
#[path = "task_compile/memory_policy.rs"]
mod memory_policy;
#[path = "task_compile/recursive.rs"]
mod recursive;
#[path = "task_compile/result_branches.rs"]
mod result_branches;
#[path = "task_compile/runtime_observers.rs"]
mod runtime_observers;
#[path = "task_compile/scheduler_clock.rs"]
mod scheduler_clock;
#[path = "task_compile/tooling_thread.rs"]
mod tooling_thread;
#[path = "task_compile/while_post_flow.rs"]
mod while_post_flow;
