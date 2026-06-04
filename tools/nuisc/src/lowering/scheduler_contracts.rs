use super::*;

pub(crate) fn assign_default_lanes(module: &mut YirModule) {
    let lane_policy = load_declared_lane_policy(module);
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource.kind.family()))
        .collect::<BTreeMap<_, _>>();

    module.node_lanes.retain(|_, lane| lane.starts_with("fn:"));
    for node in &module.nodes {
        if module.node_lanes.contains_key(&node.name) {
            continue;
        }
        let family = resource_families
            .get(node.resource.as_str())
            .copied()
            .unwrap_or("unknown");
        let lane = default_lane_for_node(&lane_policy, family, node);
        module.node_lanes.insert(node.name.clone(), lane.to_owned());
    }
}

pub(crate) fn materialize_registered_scheduler_contract_nodes(module: &mut YirModule) {
    let resource_families = module
        .resources
        .iter()
        .map(|resource| (resource.name.as_str(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let mut representative_by_family = BTreeMap::<String, String>::new();
    for node in &module.nodes {
        let Some(family) = resource_families.get(node.resource.as_str()) else {
            continue;
        };
        representative_by_family
            .entry(family.clone())
            .or_insert_with(|| node.name.clone());
    }
    let cpu_resource = module
        .resources
        .iter()
        .find(|resource| resource.kind.family() == "cpu")
        .map(|resource| resource.name.clone())
        .unwrap_or_else(|| "cpu0".to_owned());

    for (family, target) in representative_by_family {
        let Ok(manifest) =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &family)
        else {
            continue;
        };
        let lane_contract_name = format!("scheduler_contract_{family}_lane_policy_type");
        let lane_capability_contract_name =
            format!("scheduler_contract_{family}_lane_capability_type");
        let bridge_capability_contract_name =
            format!("scheduler_contract_{family}_bridge_capability_type");
        let clock_contract_name = format!("scheduler_contract_{family}_clock_type");
        let result_lane_contract_name = format!("scheduler_contract_{family}_result_lane_type");
        let result_capability_contract_name =
            format!("scheduler_contract_{family}_result_capability_type");
        let observer_role_variant_contract_name =
            format!("scheduler_contract_{family}_observer_role_variant_type");
        let summary_capability_contract_name =
            format!("scheduler_contract_{family}_summary_capability_type");
        let summary_class_contract_name = format!("scheduler_contract_{family}_summary_class_type");
        let observer_source_class_contract_name =
            format!("scheduler_contract_{family}_observer_source_class_type");
        let observer_stage_class_contract_name =
            format!("scheduler_contract_{family}_observer_stage_class_type");
        let observer_scope_class_contract_name =
            format!("scheduler_contract_{family}_observer_scope_class_type");
        let observer_branch_class_contract_name =
            format!("scheduler_contract_{family}_observer_branch_class_type");
        let lane_contract_value = render_lane_policy_contract(&family, &manifest.default_lanes);
        let lane_capability_contract_value =
            render_lane_capability_contract(&family, &manifest.default_lanes);
        let bridge_capability_contract_value =
            render_bridge_capability_contract(&family, &manifest);
        let clock_contract_value = render_clock_contract(&family, &manifest);
        let result_lane_contract_value = render_result_lane_contract(&family);
        let result_capability_contract_value = render_result_capability_contract(&family);
        let observer_role_variant_contract_value = render_observer_role_variant_contract(&family);
        let summary_capability_contract_value = render_summary_capability_contract(&family);
        let summary_class_contract_value = render_summary_class_contract(&family);
        let observer_source_class_contract_value = render_observer_source_class_contract(&family);
        let observer_stage_class_contract_value = render_observer_stage_class_contract(&family);
        let observer_scope_class_contract_value = render_observer_scope_class_contract(&family);
        let observer_branch_class_contract_value = render_observer_branch_class_contract(&family);

        push_scheduler_contract_text_node(
            module,
            &lane_contract_name,
            &cpu_resource,
            lane_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &lane_capability_contract_name,
            &cpu_resource,
            lane_capability_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &bridge_capability_contract_name,
            &cpu_resource,
            bridge_capability_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &clock_contract_name,
            &cpu_resource,
            clock_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &result_lane_contract_name,
            &cpu_resource,
            result_lane_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &result_capability_contract_name,
            &cpu_resource,
            result_capability_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &observer_role_variant_contract_name,
            &cpu_resource,
            observer_role_variant_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &summary_capability_contract_name,
            &cpu_resource,
            summary_capability_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &summary_class_contract_name,
            &cpu_resource,
            summary_class_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &observer_source_class_contract_name,
            &cpu_resource,
            observer_source_class_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &observer_stage_class_contract_name,
            &cpu_resource,
            observer_stage_class_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &observer_scope_class_contract_name,
            &cpu_resource,
            observer_scope_class_contract_value,
        );
        push_scheduler_contract_text_node(
            module,
            &observer_branch_class_contract_name,
            &cpu_resource,
            observer_branch_class_contract_value,
        );
        push_scheduler_contract_edge_if_missing(module, &lane_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &lane_capability_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &bridge_capability_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &clock_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &result_lane_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &result_capability_contract_name, &target);
        push_scheduler_contract_edge_if_missing(
            module,
            &observer_role_variant_contract_name,
            &target,
        );
        push_scheduler_contract_edge_if_missing(module, &summary_capability_contract_name, &target);
        push_scheduler_contract_edge_if_missing(module, &summary_class_contract_name, &target);
        push_scheduler_contract_edge_if_missing(
            module,
            &observer_source_class_contract_name,
            &target,
        );
        push_scheduler_contract_edge_if_missing(
            module,
            &observer_stage_class_contract_name,
            &target,
        );
        push_scheduler_contract_edge_if_missing(
            module,
            &observer_scope_class_contract_name,
            &target,
        );
        push_scheduler_contract_edge_if_missing(
            module,
            &observer_branch_class_contract_name,
            &target,
        );
    }
}

fn render_lane_policy_contract(family: &str, default_lanes: &[String]) -> String {
    let mut lanes = BTreeSet::<String>::new();
    let mut defaults = Vec::<String>::new();
    for entry in default_lanes {
        let Some((pattern, lane)) = entry.split_once('=') else {
            continue;
        };
        let pattern = pattern.trim();
        let lane = lane.trim();
        if pattern.is_empty() || lane.is_empty() {
            continue;
        }
        lanes.insert(lane.to_owned());
        defaults.push(format!("{pattern}={lane}"));
    }
    format!(
        "family={family};lanes={};defaults={}",
        lanes.into_iter().collect::<Vec<_>>().join(","),
        defaults.join("|")
    )
}

fn render_clock_contract(family: &str, manifest: &NustarPackageManifest) -> String {
    format!(
        "family={family};domain={};kind={};epoch={};resolution={};bridge={}",
        manifest.clock_domain_id,
        manifest.clock_kind,
        manifest.clock_epoch_kind,
        manifest.clock_resolution,
        manifest.clock_bridge_default
    )
}

fn render_lane_capability_contract(family: &str, default_lanes: &[String]) -> String {
    let lanes = default_lanes
        .iter()
        .filter_map(|entry| entry.split_once('='))
        .map(|(_, lane)| lane.trim())
        .filter(|lane| !lane.is_empty())
        .collect::<BTreeSet<_>>();
    let mut fields = vec![format!("family={family}")];
    for lane in lanes {
        let capability = lane_capability_for(family, lane);
        fields.push(format!("{lane}={capability}"));
    }
    fields.join(";")
}

fn render_bridge_capability_contract(family: &str, manifest: &NustarPackageManifest) -> String {
    let lane_bridge = match family {
        "cpu" => "cpu_bind_core_lane:host_main_lane|worker_lane",
        _ => "none",
    };
    format!(
        "family={family};lane_bridge={lane_bridge};clock_bridge={}",
        manifest.clock_bridge_default
    )
}

fn lane_capability_for(family: &str, lane: &str) -> &'static str {
    match (family, lane) {
        ("cpu", "main") => "host-entry",
        ("cpu", "mem") => "memory-ownership",
        ("data", "control") => "control-plane",
        ("data", "uplink") => "uplink-window",
        ("data", "downlink") => "downlink-window",
        ("data", "fabric") => "fabric-transfer",
        ("shader", "setup") => "render-setup",
        ("shader", "render") => "render-pass",
        ("kernel", "compute") | ("npu", "compute") => "compute-dispatch",
        (_, "contract") => "contract-metadata",
        _ => "general",
    }
}

fn render_result_lane_contract(family: &str) -> String {
    let lane = match family {
        "cpu" => "main",
        "data" => "fabric",
        "shader" => "setup",
        "network" => "control",
        "kernel" | "npu" => "compute",
        _ => "main",
    };
    format!("family={family};entry={lane};probe={lane};value={lane}")
}

fn render_result_capability_contract(family: &str) -> String {
    format!(
        "family={family};entry=result-entry;probe=result-ready-probe;value=result-payload-value"
    )
}

fn render_observer_role_variant_contract(family: &str) -> String {
    format!(
        "family={family};config_ready=config-ready-observer;send_ready=send-ready-observer;recv_ready=recv-ready-observer;connect_ready=connect-ready-observer;accept_ready=accept-ready-observer;closed=closed-observer"
    )
}

fn render_summary_capability_contract(family: &str) -> String {
    format!(
        "family={family};policy=async-policy-summary;batch=async-batch-summary;windowed=async-windowed-summary"
    )
}

fn render_summary_class_contract(family: &str) -> String {
    format!(
        "family={family};transport_split=transport-split-summary;transport_windowed_split=transport-windowed-split-summary;transport_session_bridge_split=transport-session-bridge-split-summary;control_split=control-split-summary;control_windowed=control-windowed-summary;control_session_bridge=control-session-bridge-summary"
    )
}

fn render_observer_source_class_contract(family: &str) -> String {
    format!("family={family};profile=profile-backed;result=result-backed;summary=summary-backed")
}

fn render_observer_stage_class_contract(family: &str) -> String {
    format!(
        "family={family};entry=observer-entry-stage;ready=observer-ready-stage;payload=observer-payload-stage;policy=observer-policy-stage;batch=observer-batch-stage;windowed=observer-windowed-stage"
    )
}

fn render_observer_scope_class_contract(family: &str) -> String {
    format!(
        "family={family};local=local-scope;cross_lane=cross-lane-scope;cross_domain=cross-domain-scope;bridge_visible=bridge-visible-scope"
    )
}

fn render_observer_branch_class_contract(family: &str) -> String {
    format!(
        "family={family};primary=primary-branch;secondary=secondary-branch;fallback=fallback-branch;send=send-branch;recv=recv-branch"
    )
}

fn push_scheduler_contract_text_node(
    module: &mut YirModule,
    name: &str,
    resource: &str,
    value: String,
) {
    if let Some(node) = module.nodes.iter_mut().find(|node| node.name == name) {
        node.resource = resource.to_owned();
        node.op = Operation {
            module: "cpu".to_owned(),
            instruction: "text".to_owned(),
            args: vec![value],
        };
        return;
    }
    module.nodes.push(Node {
        name: name.to_owned(),
        resource: resource.to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "text".to_owned(),
            args: vec![value],
        },
    });
}

fn push_scheduler_contract_edge_if_missing(module: &mut YirModule, from: &str, to: &str) {
    let exists = module.edges.iter().any(|edge| {
        edge.from == from
            && edge.to == to
            && matches!(edge.kind, EdgeKind::Dep | EdgeKind::CrossDomainExchange)
    });
    if !exists {
        module.edges.push(Edge {
            kind: EdgeKind::Dep,
            from: from.to_owned(),
            to: to.to_owned(),
        });
    }
}

fn load_declared_lane_policy(module: &YirModule) -> BTreeMap<String, String> {
    let mut policy = BTreeMap::<String, String>::new();
    for family in module
        .resources
        .iter()
        .map(|resource| resource.kind.family().to_owned())
        .collect::<std::collections::BTreeSet<_>>()
    {
        let Ok(manifest) =
            crate::registry::load_manifest_for_domain(Path::new("nustar-packages"), &family)
        else {
            continue;
        };
        for entry in manifest.default_lanes {
            let Some((pattern, lane)) = entry.split_once('=') else {
                continue;
            };
            if !pattern.is_empty() && !lane.is_empty() {
                policy.insert(pattern.trim().to_owned(), lane.trim().to_owned());
            }
        }
    }
    policy
}

fn default_lane_for_node<'a>(
    lane_policy: &'a BTreeMap<String, String>,
    family: &str,
    node: &'a Node,
) -> &'a str {
    if node.name.starts_with("scheduler_contract_") {
        return "contract";
    }
    if node.name.starts_with("project_link_") {
        return "contract";
    }
    if node.name.starts_with("project_profile_") {
        if family == "cpu" && node.op.instruction == "text" {
            return "contract";
        }
        if family == "cpu" {
            return "profile";
        }
        if family == "data" {
            return match node.op.semantic_op() {
                SemanticOp::DataImmutableWindow => "profile_uplink",
                SemanticOp::DataCopyWindow | SemanticOp::DataInputPipe => "profile_downlink",
                SemanticOp::DataHandleTable | SemanticOp::DataBindCore | SemanticOp::DataMarker => {
                    "profile_control"
                }
                SemanticOp::DataMove => "profile_fabric",
                _ => "profile_data",
            };
        }
        if family == "shader" {
            return "profile_setup";
        }
        if family == "kernel" || family == "npu" {
            return "profile_compute";
        }
    }
    if let Some(lane) = lane_policy.get(&node.op.full_name()) {
        return lane.as_str();
    }
    match family {
        "cpu" => match node.op.semantic_op() {
            SemanticOp::CpuAllocNode
            | SemanticOp::CpuAllocBuffer
            | SemanticOp::CpuBorrow
            | SemanticOp::CpuBorrowEnd
            | SemanticOp::CpuMovePtr
            | SemanticOp::CpuLoadValue
            | SemanticOp::CpuLoadNext
            | SemanticOp::CpuBufferLen
            | SemanticOp::CpuLoadAt
            | SemanticOp::CpuStoreValue
            | SemanticOp::CpuStoreNext
            | SemanticOp::CpuStoreAt
            | SemanticOp::CpuFree => "mem",
            _ => match node.op.instruction.as_str() {
                "window" | "input_i64" | "tick_i64" | "extern_call_i64" | "present_frame"
                | "print" | "bind_core" | "instantiate_unit" => "main",
                _ => "main",
            },
        },
        "data" => match node.op.semantic_op() {
            SemanticOp::DataImmutableWindow | SemanticOp::DataOutputPipe => "uplink",
            SemanticOp::DataCopyWindow | SemanticOp::DataInputPipe => "downlink",
            SemanticOp::DataHandleTable | SemanticOp::DataBindCore | SemanticOp::DataMarker => {
                "control"
            }
            SemanticOp::DataMove => "fabric",
            _ => "fabric",
        },
        "shader" => match node.op.semantic_op() {
            SemanticOp::ShaderBeginPass | SemanticOp::ShaderDrawInstanced => "render",
            SemanticOp::ShaderPipeline | SemanticOp::ShaderInlineWgsl => "setup",
            _ => "setup",
        },
        "kernel" | "npu" => "compute",
        _ => "main",
    }
}
