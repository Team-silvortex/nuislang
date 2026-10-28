use super::*;

pub(super) fn project_support_modules_accept_darwin_x86_64_shader_and_data_abis() {
    let surface_shader = standard_surface_shader_profile_module(true);
    let fabric_plane = shader_fabric_plane_module(true);
    let project = test_support::loaded_project_fixture(
        "shader_data_darwin_x86_64",
        darwin_x86_64_shader_project_abis(),
        reverse_shader_data_bridge_entry(),
        vec![
            ("surface_shader.ns", surface_shader.as_str()),
            ("fabric_plane.ns", fabric_plane.as_str()),
        ],
    );

    let plan = build_project_compilation_plan(&project).unwrap();
    let checks = validate_project_abi_selections(&project, &plan.abi_resolution).unwrap();
    let mut yir = YirModule::new("0.1");
    apply_project_support_modules_to_yir(&project, &mut yir).unwrap();

    assert!(checks.iter().all(|check| check.ok));
    assert!(checks.iter().any(|check| {
        check.domain == "shader" && check.abi.as_deref() == Some("shader.metal.x86_64.msl2_4")
    }));
    assert!(checks.iter().any(|check| {
        check.domain == "data" && check.abi.as_deref() == Some("data.fabric.macos.x86_64.v1")
    }));
    assert!(yir.nodes.iter().any(|node| {
        node.name == "project_profile_shader_SurfaceShader_shader_target_config_auto"
            && node.op.module == "shader"
            && node.op.instruction == "target_config"
            && node.op.args
                == vec![
                    "x86_64".to_owned(),
                    "metal".to_owned(),
                    "1".to_owned(),
                    "argument-buffer,device.mac-discrete-or-integrated-gpu,metal,msl,resource-binding,shader-ir,vendor.apple"
                        .to_owned()
                ]
    }));
}
