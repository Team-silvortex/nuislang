use super::*;

#[test]
fn infers_kernel_data_route_payload_types_through_shared_cpu_helper() {
    let project = kernel_task_async_shapes_project();

    let uplink = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", true)
        .unwrap()
        .expect("expected uplink payload");
    assert_eq!(uplink.render(), "Window<i64>");

    let downlink = infer_project_route_payload_type(&project, "cpu.Main", "FabricPlane", false)
        .unwrap()
        .expect("expected downlink payload");
    assert_eq!(downlink.render(), "Window<Window<i64>>");
}

#[test]
fn validates_kernel_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = kernel_task_async_shapes_project_with_link();

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_network_project_links_against_nir_with_shared_cpu_helper_indirection() {
    let project = network_task_async_project_with_link(
        network_task_async_probe_entry(),
        network_task_async_probe_module(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn validates_network_project_links_against_nir_via_data_bridge() {
    let project =
        forward_network_data_bridge_project_with_link(reverse_network_data_bridge_module());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_network_project_links_via_data_bridge_missing_downlink_usage() {
    let project = forward_network_data_bridge_project_with_link(
        forward_network_data_bridge_missing_downlink_module(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("data_send_downlink(\"FabricPlane\""));
}

#[test]
fn validates_network_project_links_against_yir_via_data_bridge() {
    let project =
        forward_network_data_bridge_project_with_link(reverse_network_data_bridge_module());

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();
    validate_project_links_against_yir(&project, &yir).unwrap();
}

#[test]
fn rejects_network_project_links_against_yir_when_data_to_network_xfer_is_missing() {
    let project =
        forward_network_data_bridge_project_with_link(reverse_network_data_bridge_module());

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();

    let resource_families = yir
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = yir
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    yir.edges.retain(|edge| {
        if edge.kind != EdgeKind::CrossDomainExchange {
            return true;
        }
        let from_family = node_resources
            .get(&edge.from)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        let to_family = node_resources
            .get(&edge.to)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        !(from_family == Some("data") && to_family == Some("network"))
    });

    let err = validate_project_links_against_yir(&project, &yir).unwrap_err();
    assert!(err.contains("requires a `data` -> `network` xfer segment"));
}

#[test]
fn validates_reverse_network_project_links_against_nir_via_data_bridge() {
    let project = reverse_network_data_bridge_project_with_link(
        reverse_network_data_bridge_module(),
        "network.NetworkUnit",
        "cpu.Main",
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_reverse_network_project_links_missing_endpoint_kind_usage() {
    let project = network_data_project_with_entry(
        r#"
        use cpu NetworkDataBridge;
        use network NetworkUnit;
        use data FabricPlane;

        mod cpu Main {
          fn main() -> i64 {
            return NetworkDataBridge.probe_roundtrip();
          }
        }
        "#,
        {
            let mut modules = network_data_support_modules();
            modules.push((
                "network_data_bridge.ns",
                r#"
                use network NetworkUnit;
                use data FabricPlane;

                mod cpu NetworkDataBridge {
                  pub fn probe_roundtrip() -> i64 {
                    let bind_core: NetworkResult<i64> =
                      network_result(network_profile_bind_core("NetworkUnit"));
                    let send_window: NetworkResult<i64> =
                      network_result(network_profile_send_window("NetworkUnit"));
                    let value: i64 =
                      network_value(bind_core) + network_value(send_window);
                    data_profile_bind_core("FabricPlane");
                    let handles: HandleTable<FabricPlaneBindings> =
                      data_profile_handle_table("FabricPlane");
                    let uplink: Window<i64> =
                      data_profile_send_uplink("FabricPlane", value);
                    let downlink: Window<Window<i64>> =
                      data_profile_send_downlink("FabricPlane", uplink);
                    print(handles);
                    print(downlink);
                    return value;
                  }
                }
                "#,
            ));
            modules
        },
    );
    let mut project = project;
    project.manifest.links = vec![ProjectLink {
        from: "network.NetworkUnit".to_owned(),
        to: "cpu.Main".to_owned(),
        via: Some("data.FabricPlane".to_owned()),
    }];

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("network_profile_endpoint_kind(\"NetworkUnit\")"));
}

#[test]
fn validates_reverse_network_project_links_against_yir_via_data_bridge() {
    let project = reverse_network_data_bridge_project_with_link(
        reverse_network_data_bridge_module(),
        "network.NetworkUnit",
        "cpu.Main",
    );

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();
    validate_project_links_against_yir(&project, &yir).unwrap();
}

#[test]
fn rejects_reverse_network_project_links_against_yir_when_network_to_data_xfer_is_missing() {
    let project = reverse_network_data_bridge_project_with_link(
        reverse_network_data_bridge_module(),
        "network.NetworkUnit",
        "cpu.Main",
    );

    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();

    let resource_families = yir
        .resources
        .iter()
        .map(|resource| (resource.name.clone(), resource.kind.family().to_owned()))
        .collect::<BTreeMap<_, _>>();
    let node_resources = yir
        .nodes
        .iter()
        .map(|node| (node.name.clone(), node.resource.clone()))
        .collect::<BTreeMap<_, _>>();
    yir.edges.retain(|edge| {
        if edge.kind != EdgeKind::CrossDomainExchange {
            return true;
        }
        let from_family = node_resources
            .get(&edge.from)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        let to_family = node_resources
            .get(&edge.to)
            .and_then(|resource| resource_families.get(resource))
            .map(String::as_str);
        !(from_family == Some("network") && to_family == Some("data"))
    });

    let err = validate_project_links_against_yir(&project, &yir).unwrap_err();
    assert!(err.contains("requires a `network` -> `data` xfer segment"));
}

// Direct network profile and transport/ownership validation matrix.
#[test]
fn rejects_network_project_links_missing_endpoint_kind_usage() {
    let project = direct_network_default_project_with_link(direct_network_bind_core_entry());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("network_profile_endpoint_kind(\"NetworkUnit\")"));
}

#[test]
fn validates_network_project_links_for_transport_and_protocol_profile_usage() {
    let project = network_task_async_project_with_link(
        network_task_async_transport_entry(),
        network_task_async_transport_module(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

#[test]
fn rejects_network_project_links_when_protocol_profile_const_is_missing() {
    let project = direct_network_project_with_link(
        direct_network_protocol_kind_entry(),
        network_support_modules_without_protocol_kind(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let err = validate_project_links_against_nir(&project, &nir).unwrap_err();
    assert!(err.contains("requires `protocol_kind` profile const"));
}

#[test]
fn validates_network_project_links_for_host_transport_calls() {
    let project = direct_network_default_project_with_link(network_host_transport_entry());

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}
