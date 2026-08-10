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

fn assert_shared_scheduler_mutex_metadata(artifacts: &nuisc::pipeline::PipelineArtifacts) {
    for node in artifacts.yir.nodes.iter().filter(|node| {
        node.op.module == "cpu"
            && matches!(
                node.op.instruction.as_str(),
                "mutex_share"
                    | "mutex_shared_close"
                    | "mutex_permit"
                    | "mutex_permit_lock"
                    | "mutex_lease_value"
                    | "mutex_lease_replace"
                    | "mutex_lease_unlock"
            )
    }) {
        let dependency_count = match node.op.instruction.as_str() {
            "mutex_share" | "mutex_permit" | "mutex_lease_replace" => 2,
            _ => 1,
        };
        assert_eq!(
            &node.op.args[dependency_count..],
            yir_core::CPU_SHARED_MUTEX_RUNTIME_METADATA,
            "shared mutex contract metadata drifted for {}",
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

#[test]
fn lowers_branch_local_shared_mutex_capabilities_exactly_once() {
    let artifacts =
        compiled_source("../../examples/projects/task/task_shared_mutex_branch_demo/main.ns");

    for instruction in [
        "mutex_new",
        "mutex_share",
        "mutex_permit",
        "mutex_permit_lock",
        "mutex_lease_replace",
        "mutex_lease_value",
        "mutex_lease_unlock",
        "mutex_shared_close",
    ] {
        assert_eq!(
            cpu_op_count(&artifacts, instruction),
            1,
            "{instruction} should execute once after branch selection"
        );
    }
    assert_eq!(cpu_op_count(&artifacts, "select"), 2);
    assert_shared_scheduler_mutex_metadata(&artifacts);

    let share = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.op.module == "cpu" && node.op.instruction == "mutex_share")
        .expect("selected shared mutex share node");
    let cardinality = artifacts
        .yir
        .nodes
        .iter()
        .find(|node| node.name == share.op.args[1])
        .expect("static cardinality dependency");
    assert_eq!(cardinality.op.instruction, "const_i64");
    assert_eq!(cardinality.op.args, ["3"]);
}

#[test]
fn rejects_branch_selected_shared_mutex_static_contract_drift() {
    for (ty, lhs, rhs, finish, expected) in [
        (
            "SharedMutex<i64>",
            "mutex_share(mutex_new(11), 2)",
            "mutex_share(mutex_new(19), 3)",
            "return mutex_shared_close(selected);",
            "same static permit cardinality literal",
        ),
        (
            "MutexPermit<i64>",
            "mutex_permit(shared, 0)",
            "mutex_permit(shared, 1)",
            "let lease: MutexLease<i64> = mutex_permit_lock(selected); let released: i64 = mutex_lease_unlock(lease); let revoked: i64 = mutex_shared_close(shared); return released + revoked;",
            "same static lane literal",
        ),
    ] {
        let setup = if lhs.starts_with("mutex_permit") {
            "let shared: SharedMutex<i64> = mutex_share(mutex_new(11), 3);"
        } else {
            ""
        };
        let source = format!(
            r#"
            mod cffi Main {{
              extern "c" fn host_argv_count() -> i64;
              fn main() -> i64 {{
                {setup}
                let selected: {ty} = if host_argv_count() < 2 {{
                  let value: {ty} = {lhs};
                  value
                }} else {{
                  let value: {ty} = {rhs};
                  value
                }};
                {finish}
              }}
            }}
            "#
        );
        let error = match nuisc::pipeline::compile_source(&source) {
            Ok(_) => panic!("static shared mutex branch contract drift must fail"),
            Err(error) => error,
        };
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}
