use super::*;

#[test]
fn rejects_mismatched_data_observe_state() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "fabric0".to_owned(),
                kind: ResourceKind::parse("data.fabric"),
            },
        ],
        nodes: vec![
            node("value", "cpu0", "cpu.const", &["7"]),
            node("pipe", "fabric0", "data.output_pipe", &["value"]),
            node("result", "fabric0", "data.observe", &["pipe", "ready"]),
        ],
        edges: vec![xfer("value", "pipe"), dep("pipe", "result")],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("does not support that state"));
}

#[test]
fn accepts_kernel_result_observe_from_project_profile_ref() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.compute"),
            },
        ],
        nodes: vec![
            node(
                "queue_depth",
                "cpu0",
                "cpu.project_profile_ref",
                &["kernel", "KernelUnit", "queue_depth"],
            ),
            node(
                "kernel_result",
                "kernel0",
                "kernel.observe",
                &["queue_depth", "config_ready"],
            ),
            node(
                "kernel_ready",
                "kernel0",
                "kernel.is_config_ready",
                &["kernel_result"],
            ),
        ],
        edges: vec![
            xfer("queue_depth", "kernel_result"),
            dep("kernel_result", "kernel_ready"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn accepts_kernel_result_observe_from_resolved_project_profile_slot() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.compute"),
            },
        ],
        nodes: vec![
            node(
                "project_profile_kernel_KernelUnit_batch_lanes",
                "cpu0",
                "cpu.const_i64",
                &["16"],
            ),
            node(
                "kernel_result",
                "kernel0",
                "kernel.observe",
                &[
                    "project_profile_kernel_KernelUnit_batch_lanes",
                    "config_ready",
                ],
            ),
            node(
                "kernel_ready",
                "kernel0",
                "kernel.is_config_ready",
                &["kernel_result"],
            ),
        ],
        edges: vec![
            xfer(
                "project_profile_kernel_KernelUnit_batch_lanes",
                "kernel_result",
            ),
            dep("kernel_result", "kernel_ready"),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn rejects_task_value_without_join_result_source() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node("value", "cpu0", "cpu.const", &["7"]),
            node("task", "cpu0", "cpu.spawn_task", &["ping", "value"]),
            node("invalid", "cpu0", "cpu.task_value", &["task"]),
        ],
        edges: vec![dep("value", "task"), dep("task", "invalid")],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expects `cpu.join_result` input"));
}

#[path = "result_network_state.rs"]
mod result_network_state;
