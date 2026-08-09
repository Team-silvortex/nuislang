use std::path::Path;

fn compiled_source(path: &str) -> nuisc::pipeline::PipelineArtifacts {
    nuisc::pipeline::compile_source_path(Path::new(path))
        .unwrap_or_else(|error| panic!("source `{path}` should compile: {error}"))
}

fn cpu_op_count(artifacts: &nuisc::pipeline::PipelineArtifacts, instruction: &str) -> usize {
    artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction == instruction)
        .count()
}

fn assert_scheduler_mutex_metadata(artifacts: &nuisc::pipeline::PipelineArtifacts) {
    let mutex_nodes = artifacts
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.module == "cpu" && node.op.instruction.starts_with("mutex_"))
        .collect::<Vec<_>>();
    assert!(!mutex_nodes.is_empty(), "expected mutex YIR nodes");
    for node in mutex_nodes {
        assert_eq!(
            &node.op.args[1..],
            yir_core::CPU_MUTEX_RUNTIME_METADATA,
            "mutex contract metadata drifted for {}",
            node.name
        );
    }
}

#[test]
fn lowers_branch_local_mutex_lock_through_one_selected_runtime_prefix() {
    let artifacts =
        compiled_source("../../examples/ns/memory/hello_thread_mutex_if_lock_branch.ns");

    assert_eq!(cpu_op_count(&artifacts, "mutex_lock"), 1);
    assert_eq!(cpu_op_count(&artifacts, "mutex_value"), 1);
    assert_eq!(cpu_op_count(&artifacts, "add"), 2);
    assert!(cpu_op_count(&artifacts, "select") >= 2);
    assert_scheduler_mutex_metadata(&artifacts);
}

#[test]
fn lowers_branch_local_thread_result_through_one_selected_runtime_prefix() {
    let artifacts =
        compiled_source("../../examples/ns/memory/hello_thread_mutex_match_join_result_branch.ns");

    assert_eq!(cpu_op_count(&artifacts, "async_call"), 1);
    assert_eq!(cpu_op_count(&artifacts, "spawn_thread"), 1);
    assert_eq!(cpu_op_count(&artifacts, "thread_join_result"), 1);
    assert_eq!(cpu_op_count(&artifacts, "task_completed"), 1);
    assert_eq!(cpu_op_count(&artifacts, "task_value"), 1);
    assert!(cpu_op_count(&artifacts, "select") >= 2);
}
