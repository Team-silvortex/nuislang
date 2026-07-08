use super::*;

#[test]
fn scheduler_contract_nodes_reject_invalid_observer_source_class_label() {
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
                "scheduler_contract_shader_observer_source_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;profile=profile-source;result=result-backed;summary=summary-backed"#,
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
                "scheduler_contract_shader_observer_source_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `profile-backed`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_observer_stage_class_label() {
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
                    r#"family=shader;entry=observer-entry-stage;ready=observer-state-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"#,
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
                "scheduler_contract_shader_observer_source_class_type",
                "shader_target",
            ),
            dep(
                "scheduler_contract_shader_observer_stage_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `observer-ready-stage`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_observer_scope_class_label() {
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
                    r#"family=shader;local=local-scope;cross_lane=lane-crossing-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"#,
                ],
            ),
            node(
                "scheduler_contract_shader_observer_branch_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;primary=primary-branch;secondary=secondary-branch;fallback=fallback-branch;send=send-branch;recv=recv-branch"#,
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
            dep(
                "scheduler_contract_shader_observer_branch_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `cross-lane-scope`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_invalid_observer_branch_class_label() {
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
                "scheduler_contract_shader_observer_branch_class_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=shader;primary=primary-branch;secondary=secondary-branch;fallback=default-branch;send=send-branch;recv=recv-branch"#,
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
            dep(
                "scheduler_contract_shader_observer_branch_class_type",
                "shader_target",
            ),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("expected `fallback-branch`"), "{error}");
}

#[test]
fn scheduler_contract_nodes_reject_lane_capability_outside_declared_set() {
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
                &[r#"family=shader;render=render-pass;setup=render-setup;main=host-entry"#],
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
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(error.contains("declares capability for lane `main`"));
}

#[test]
fn scheduler_contract_nodes_reject_invalid_cpu_bridge_capability() {
    let module = YirModule {
        version: "0.1".to_owned(),
        resources: vec![Resource {
            name: "cpu0".to_owned(),
            kind: ResourceKind::parse("cpu.arm64"),
        }],
        nodes: vec![
            node(
                "scheduler_contract_cpu_lane_policy_type",
                "cpu0",
                "cpu.text",
                &[r#"family=cpu;lanes=main,mem;defaults=cpu.print=main|cpu.alloc_node=mem"#],
            ),
            node(
                "scheduler_contract_cpu_clock_type",
                "cpu0",
                "cpu.text",
                &[
                    r#"family=cpu;domain=cpu.clock.host.v1;kind=host-monotonic;epoch=host-epoch;resolution=cpu.tick_i64;bridge=global->monotonic:bridge"#,
                ],
            ),
            node(
                "scheduler_contract_cpu_bridge_capability_type",
                "cpu0",
                "cpu.text",
                &[r#"family=cpu;lane_bridge=host_main_lane;clock_bridge=global->monotonic:bridge"#],
            ),
            node("seed", "cpu0", "cpu.const", &["7"]),
            node("cpu_entry", "cpu0", "cpu.print", &["seed"]),
        ],
        edges: vec![
            dep("scheduler_contract_cpu_lane_policy_type", "cpu_entry"),
            dep("scheduler_contract_cpu_clock_type", "cpu_entry"),
            dep("scheduler_contract_cpu_bridge_capability_type", "cpu_entry"),
            dep("seed", "cpu_entry"),
        ],
        node_lanes: BTreeMap::new(),
    };

    let error = verify_module(&module).unwrap_err();
    assert!(
        error.contains("currently expects CPU lane bridge"),
        "{error}"
    );
}
