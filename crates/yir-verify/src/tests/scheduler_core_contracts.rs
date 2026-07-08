use super::*;

#[test]
fn scheduler_contract_nodes_validate_lane_and_clock_registration() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_lane_capability_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;render=render-pass;setup=render-setup"#],
            ),
            node(
                "scheduler_contract_shader_bridge_capability_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;lane_bridge=none;clock_bridge=global->frame:bridge"#],
            ),
            node(
                "scheduler_contract_shader_clock_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;domain=shader.clock.frame.v1;kind=frame-monotonic;epoch=frame-epoch;resolution=render-pass-step;bridge=global->frame:bridge"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
            ),
            node(
                "scheduler_contract_shader_result_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_role_variant_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-ready-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"#,
                ],
            ),
            node(
                "scheduler_contract_shader_summary_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_source_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;profile=profile-backed;result=result-backed;summary=summary-backed"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_stage_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_scope_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;local=local-scope;cross_lane=cross-lane-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"#,
                ],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_lane_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_bridge_capability_type",
                "shader_target",
            ),
            dep("scheduler_contract_shader_clock_type", "shader_target"),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_role_variant_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_summary_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_source_class_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_stage_class_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_scope_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    verify_module(&module).unwrap();
}

#[test]
fn scheduler_contract_nodes_reject_lane_defaults_outside_declared_set() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "kernel0".to_owned(),
                kind: ResourceKind::parse("kernel.apple"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_kernel_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=kernel;lanes=compute;defaults=kernel.tensor=compute|kernel.print=main"#,
                ],
            ),
            node(
                "kernel_entry",
                "kernel0",
                "kernel.target_config",
                &["apple_ane", "coreml", "16"],
            ),
        ],
        edges: vec![dep(
            "scheduler_contract_kernel_lane_policy_type",
            "kernel_entry",
        )],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("declares default lane `main` outside"));
}

#[test]
fn scheduler_contract_nodes_reject_result_lane_outside_declared_set() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=render;value=main"#],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("declares result lane `main`"));
}

#[test]
fn scheduler_contract_nodes_reject_invalid_result_capability_label() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
            ),
            node(
                "scheduler_contract_shader_result_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=result-entry;probe=result-state-probe;value=result-payload-value"#,
                ],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_capability_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `result-ready-probe`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_observer_role_variant_label() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
            ),
            node(
                "scheduler_contract_shader_result_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_role_variant_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"#,
                ],
            ),
            node(
                "scheduler_contract_shader_summary_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                ],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_role_variant_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_summary_capability_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `recv-ready-observer`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_summary_capability_label() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
            ),
            node(
                "scheduler_contract_shader_result_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                ],
            ),
            node(
                "scheduler_contract_shader_summary_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;policy=async-policy-summary;batch=async-fan-in-summary;windowed=async-windowed-summary"#,
                ],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_summary_capability_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `async-batch-summary`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_summary_class_label() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![
            Resource {
                name: "cpu0".to_owned(),
                kind: ResourceKind::parse("cpu.arm64"),
            },
            Resource {
                name: "shader0".to_owned(),
                kind: ResourceKind::parse("shader.metal"),
            },
        ],
        nodes: vec![
            node(
                "scheduler_contract_shader_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;lanes=render,setup;defaults=shader.target=setup|shader.begin_pass=render"#,
                ],
            ),
            node(
                "scheduler_contract_shader_result_lane_type",
                "cpu0",
                "cpu.text",
                &[r#"family=shader;entry=setup;probe=setup;value=setup"#],
            ),
            node(
                "scheduler_contract_shader_result_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;entry=result-entry;probe=result-ready-probe;value=result-payload-value"#,
                ],
            ),
            node(
                "scheduler_contract_shader_summary_capability_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"#,
                ],
            ),
            node(
                "scheduler_contract_shader_summary_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;transport_split=transport-summary;transport_windowed_split=transport-windowed-split-summary;transport_session_bridge_split=transport-session-bridge-split-summary;control_split=control-split-summary;control_windowed=control-windowed-summary;control_session_bridge=control-session-bridge-summary"#,
                ],
            ),
            node(
                "shader_target",
                "shader0",
                "shader.target",
                &["rgba8_unorm", "160", "120"],
            ),
        ],
        edges: vec![
            dep(
                "scheduler_contract_shader_lane_policy_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_lane_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_result_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_summary_capability_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_summary_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(
        error.contains("expected `transport-split-summary`"),
        "{error}"
    );
}
