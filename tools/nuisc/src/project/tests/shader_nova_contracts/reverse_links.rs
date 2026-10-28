use super::*;

pub(super) fn validates_reverse_shader_project_links_against_nir_via_data_bridge() {
    let fabric_plane = shader_contract_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_data_bridge_entry(),
        false,
        fabric_plane.as_str(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    validate_project_links_against_nir(&project, &nir).unwrap();
}

pub(super) fn validates_reverse_shader_project_links_against_yir_via_data_bridge() {
    let fabric_plane = shader_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_data_bridge_entry(),
        true,
        fabric_plane.as_str(),
    );

    let nir = lower_project_module_to_nir(&project, &project.modules[0]).unwrap();
    let lowering_manifest =
        crate::registry::load_manifest_for_domain(std::path::Path::new("nustar-packages"), "cpu")
            .unwrap();
    let mut yir = crate::lowering::lower_nir_to_yir(&nir, &lowering_manifest, None).unwrap();
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();
    apply_project_links_to_yir(&project, &mut yir).unwrap();
    validate_project_links_against_yir(&project, &yir).unwrap();

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
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_resources
                .get(&edge.from)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("shader")
            && node_resources
                .get(&edge.to)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("data")
    }));
    assert!(yir.edges.iter().any(|edge| {
        edge.kind == EdgeKind::CrossDomainExchange
            && node_resources
                .get(&edge.from)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("data")
            && node_resources
                .get(&edge.to)
                .and_then(|resource| resource_families.get(resource))
                .map(String::as_str)
                == Some("cpu")
    }));
}

pub(super) fn compiles_reverse_shader_project_via_data_bridge() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(true);
    let root = write_temp_shader_data_project(
        "reverse_shader_via_data_bridge",
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
        &["shader.SurfaceShader -> cpu.Main via data.FabricPlane"],
    );
    let artifacts = crate::pipeline::compile_source_path(&root).unwrap();
    let _ = fs::remove_dir_all(&root);

    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.shader"));
    assert!(artifacts
        .loaded_nustar
        .iter()
        .any(|package| package == "official.data"));
    assert!(artifacts
        .yir
        .nodes
        .iter()
        .any(|node| node.op.module == "shader" && node.op.instruction == "draw_instanced"));
}

pub(super) fn rejects_reverse_shader_project_via_data_bridge_when_shader_to_data_xfer_is_missing() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(false);
    let root = write_temp_shader_data_project(
        "reverse_shader_via_data_bridge_missing_xfer",
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
        &["shader.SurfaceShader -> cpu.Main via data.FabricPlane"],
    );
    let err = match crate::pipeline::compile_source_path(&root) {
        Ok(_) => panic!("expected reverse shader/data bridge compile to fail"),
        Err(err) => err,
    };
    let _ = fs::remove_dir_all(&root);
    assert!(err.contains("requires typed `Marker<Tag>` binding for marker `shader_to_cpu`"));
}

pub(super) fn rejects_reverse_shader_project_links_against_yir_when_shader_to_data_xfer_is_missing()
{
    let fabric_plane = shader_fabric_plane_module(true);
    let project = reverse_shader_data_bridge_project_with_link(
        reverse_shader_render_bridge_entry(),
        true,
        fabric_plane.as_str(),
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
        !(from_family == Some("shader") && to_family == Some("data"))
    });

    let err = validate_project_links_against_yir(&project, &yir).unwrap_err();
    assert!(err.contains("requires a `shader` -> `data` xfer segment"));
}
