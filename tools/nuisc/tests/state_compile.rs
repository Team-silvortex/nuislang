use std::path::Path;

use nuis_semantics::model::{NirExpr, NirStmt};

fn compiled_project(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_project(Path::new(path))
        .unwrap_or_else(|error| panic!("project `{path}` should compile: {error}"))
}

fn expect_const_i64_value(
    artifacts: &nuisc::pipeline::PipelineArtifacts,
    node_name: &str,
    value: &str,
) {
    assert!(
        artifacts.yir.nodes.iter().any(|node| {
            node.name == node_name
                && node.op.module == "cpu"
                && node.op.instruction == "const_i64"
                && node.op.args.last().is_some_and(|arg| arg == value)
        }),
        "expected const node `{node_name}` with value `{value}`"
    );
}

#[path = "state_compile/hof_and_glm.rs"]
mod hof_and_glm;
#[path = "state_compile/match_guards.rs"]
mod match_guards;
#[path = "state_compile/ordinary_recursion_a.rs"]
mod ordinary_recursion_a;
#[path = "state_compile/ordinary_recursion_b.rs"]
mod ordinary_recursion_b;
#[path = "state_compile/post_flow_a.rs"]
mod post_flow_a;
#[path = "state_compile/post_flow_b.rs"]
mod post_flow_b;
#[path = "state_compile/tail_recursion.rs"]
mod tail_recursion;
#[path = "state_compile/while_loops.rs"]
mod while_loops;
