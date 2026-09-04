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
        functions: Vec::new(),
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
        functions: Vec::new(),
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
        functions: Vec::new(),
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
        functions: Vec::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expects `cpu.join_result` input"));
}

#[test]
fn accepts_clocked_provider_completion_receipt_projection() {
    let module = data_completion_receipt_module(true);
    verify_module(&module).unwrap();
}

#[test]
fn rejects_completion_projection_from_receipt_less_observe() {
    let module = data_completion_receipt_module(false);
    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("receipt-less"));
}

#[test]
fn accepts_completion_projection_from_registered_shader_provider() {
    let module = shader_completion_receipt_module();
    verify_module(&module).unwrap();
}

fn shader_completion_receipt_module() -> YirModule {
    YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "shader0".to_owned(),
            kind: ResourceKind::parse("shader.reference"),
        }],
        nodes: vec![
            node("target", "shader0", "shader.target", &["rgba8", "8", "8"]),
            node(
                "pipeline",
                "shader0",
                "shader.pipeline",
                &["flat", "triangle"],
            ),
            node("viewport", "shader0", "shader.viewport", &["8", "8"]),
            node(
                "pass",
                "shader0",
                "shader.begin_pass",
                &["target", "pipeline", "viewport"],
            ),
            node(
                "result",
                "shader0",
                "shader.observe",
                &["pass", "pass_ready"],
            ),
            node("token", "shader0", "shader.completion_token", &["result"]),
        ],
        edges: vec![
            dep("target", "pass"),
            dep("pipeline", "pass"),
            dep("viewport", "pass"),
            dep("pass", "result"),
            dep("result", "token"),
        ],
        node_lanes: BTreeMap::new(),
        functions: Vec::new(),
    }
}

fn data_completion_receipt_module(with_clock: bool) -> YirModule {
    let mut observe_args = vec!["pipe", "moved"];
    if with_clock {
        observe_args.push("clock");
    }
    let mut nodes = vec![
        node("value", "cpu0", "cpu.const_i64", &["7"]),
        node("clock", "cpu0", "cpu.const_i64", &["23"]),
        node("pipe", "fabric0", "data.output_pipe", &["value"]),
        node("result", "fabric0", "data.observe", &observe_args),
        node("token", "fabric0", "data.completion_token", &["result"]),
    ];
    if !with_clock {
        nodes.remove(1);
    }
    let mut edges = vec![
        xfer("value", "pipe"),
        dep("pipe", "result"),
        dep("result", "token"),
    ];
    if with_clock {
        edges.push(xfer("clock", "result"));
    }
    YirModule {
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
        nodes,
        edges,
        node_lanes: BTreeMap::new(),
        functions: Vec::new(),
    }
}

#[path = "result_network_state.rs"]
mod result_network_state;
