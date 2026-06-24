use super::test_support::write_temp_project_fixture;
use super::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn resolves_project_galaxy_dependency_closure_from_stdlib() {
    let root = write_temp_project_fixture(
        "galaxy_resolution",
        r#"
name = "galaxy-resolution"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    let resolved_names = project
        .resolved_galaxies
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(resolved_names, vec!["core", "pixelmagic", "std"]);

    let pixelmagic = project
        .resolved_galaxies
        .iter()
        .find(|item| item.name == "pixelmagic")
        .unwrap();
    assert!(pixelmagic.direct);
    assert_eq!(pixelmagic.package_id, "nuis.pixelmagic");
    assert!(pixelmagic.auto_injectable);
    assert_eq!(pixelmagic.library_import_policy.as_str(), "project-auto");
    assert_eq!(
        pixelmagic.library_modules,
        vec![
            "lib/image_contracts.ns".to_owned(),
            "lib/shader_contracts.ns".to_owned(),
            "lib/packet_bridge_surface.ns".to_owned(),
            "lib/render_surface.ns".to_owned(),
            "lib/texture_surface.ns".to_owned(),
            "lib/pipeline_surface.ns".to_owned()
        ]
    );

    let std = project
        .resolved_galaxies
        .iter()
        .find(|item| item.name == "std")
        .unwrap();
    assert!(!std.direct);
    assert!(std.requested_by.iter().any(|item| item == "pixelmagic"));
    assert!(std.auto_injectable);
    assert_eq!(std.library_import_policy.as_str(), "project-auto");
    assert_eq!(
        std.library_modules,
        vec!["lib/task_contracts.ns".to_owned()]
    );

    let core = project
        .resolved_galaxies
        .iter()
        .find(|item| item.name == "core")
        .unwrap();
    assert!(core.auto_injectable);
    assert_eq!(core.library_import_policy.as_str(), "project-auto");
    assert_eq!(
        core.library_modules,
        vec!["lib/prelude_contracts.ns".to_owned()]
    );
}

#[test]
fn writes_project_galaxy_index_metadata() {
    let root = write_temp_project_fixture(
        "galaxy_metadata",
        r#"
name = "galaxy-metadata"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    let plan = build_project_compilation_plan(&project).unwrap();
    let output_dir = root.join("build");
    let metadata = write_project_metadata(&output_dir, &project, &plan).unwrap();
    let galaxy_index = fs::read_to_string(&metadata.galaxy_index_path).unwrap();
    let modules_index =
        fs::read_to_string(root.join("build").join("nuis.project.modules.txt")).unwrap();
    let docs_index = fs::read_to_string(root.join("build").join("nuis.project.docs.txt")).unwrap();
    let imports_index =
        fs::read_to_string(root.join("build").join("nuis.project.imports.txt")).unwrap();
    let organization_index =
        fs::read_to_string(root.join("build").join("nuis.project.organization.txt")).unwrap();

    assert!(galaxy_index
        .contains("summary\tgalaxies=3\tdocumented_galaxies=3\tdocumented_library_modules="));
    assert!(galaxy_index.contains("pixelmagic\tpackage=nuis.pixelmagic\tdirect=true"));
    assert!(galaxy_index.contains("library_modules=lib/image_contracts.ns, lib/shader_contracts.ns, lib/packet_bridge_surface.ns, lib/render_surface.ns, lib/texture_surface.ns, lib/pipeline_surface.ns"));
    assert!(galaxy_index.contains("core\tpackage=nuis.core\tdirect=false"));
    assert!(galaxy_index.contains("library_modules=lib/prelude_contracts.ns"));
    assert!(galaxy_index.contains("pixelmagic\tpackage=nuis.pixelmagic\tdirect=true\trequested_by=pixelmagic\tsource_modules=18\tauto_injectable=true"));
    assert!(galaxy_index.contains("documented_library_modules="));
    assert!(galaxy_index.contains("documented_items="));
    assert!(galaxy_index.contains("std\tpackage=nuis.std\tdirect=false"));
    assert!(galaxy_index.contains("library_modules=lib/task_contracts.ns"));
    assert!(galaxy_index.contains("library_import_policy=project-auto"));
    assert!(galaxy_index.contains("blockers=<none>"));
    assert!(modules_index.contains(
        "main.ns\tmod cpu Main\tentry=true\tsource_kind=project-local\tmanifest_spec=main.ns"
    ));
    assert!(docs_index.contains("summary\tmodules=9\tdocumented_modules=8\tdocumented_items=54"));
    assert!(docs_index.contains("module\tcpu.Main\titems=0\tsource_kind=project-local"));
    assert!(docs_index
        .contains("module\tcpu.PixelMagicContracts\titems=33\tsource_kind=galaxy-auto-inject"));
    assert!(imports_index.contains(
        "summary\tlibraries=8\tvisible_libraries=8\tvisible_modules=9\tdocumented_visible_modules=8\tdocumented_visible_items=54"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/image_contracts.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/shader_contracts.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/packet_bridge_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/render_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/texture_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/pipeline_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tcore\tlib/prelude_contracts.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "visible\tcpu\tMain\tdoc_items=0\tsource_kind=project-local\tmanifest_spec=main.ns"
    ));
    assert!(imports_index.contains("visible\tcpu\tStdTaskContracts\tdoc_items="));
    assert!(imports_index.contains("visible\tcpu\tPixelMagicContracts\tdoc_items="));
    assert!(imports_index.contains("visible\tshader\tPixelMagicSurfaceContracts\tdoc_items="));
    assert!(imports_index.contains("import_policy=project-auto"));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/image_contracts.ns\tmod cpu PixelMagicContracts\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/image_contracts.ns"
    ));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/shader_contracts.ns\tmod shader PixelMagicSurfaceContracts\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/shader_contracts.ns"
    ));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/packet_bridge_surface.ns\tmod shader PixelMagicPacketBridgeSurface\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/packet_bridge_surface.ns"
    ));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/render_surface.ns\tmod shader PixelMagicRenderSurface\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/render_surface.ns"
    ));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/texture_surface.ns\tmod shader PixelMagicTextureSurface\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/texture_surface.ns"
    ));
    assert!(modules_index.contains(
        "stdlib/pixelmagic/lib/pipeline_surface.ns\tmod shader PixelMagicPipelineSurface\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/pipeline_surface.ns"
    ));
    assert!(organization_index.contains(
        "cpu\tCorePrelude\tentry=false\tsource_kind=galaxy-auto-inject\tgalaxy=core\tpackage=nuis.core\tlibrary_module=lib/prelude_contracts.ns"
    ));
}

#[test]
fn injects_pixelmagic_library_modules_into_project_scope() {
    let root = write_temp_project_fixture(
        "galaxy_library_injection",
        r#"
name = "galaxy-library-injection"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
use cpu PixelMagicContracts;

mod cpu Main {
  fn main() -> i64 {
    return PixelMagicContracts.grayscale_packet_total(5101, 160, 120)
      + PixelMagicContracts.filter_packet_total(
        5102,
        160,
        120,
        PixelMagicContracts.blur_op_kind(),
        5
      );
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "PixelMagicContracts"));
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.domain == "shader"
            && module.ast.unit == "PixelMagicSurfaceContracts"));
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "CorePrelude"));
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "StdTaskContracts"));

    validate_project_modules(&project.modules).unwrap();
    validate_project_uses(&project.modules, &project.resolved_galaxies).unwrap();

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.contains(
        "use\tcpu.Main\tcpu.PixelMagicContracts\tresolution=local-visible:galaxy-auto-inject:galaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/image_contracts.ns"
    ));
    assert!(imports_index.contains("import_policy=project-auto"));

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert_eq!(artifacts.nir.unit, "Main");
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.grayscale_packet_total"));
}

#[test]
fn injects_pixelmagic_shader_library_module_into_project_scope() {
    let root = write_temp_project_fixture(
        "galaxy_shader_library_injection",
        r#"
name = "galaxy-shader-library-injection"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
use shader PixelMagicSurfaceContracts;

mod cpu Main {
  fn main() -> i64 {
    return shader_profile_vertex_count("PixelMagicSurfaceContracts")
      + shader_profile_instance_count("PixelMagicSurfaceContracts")
      + shader_profile_packet_tag("PixelMagicSurfaceContracts")
      + shader_profile_material_mode("PixelMagicSurfaceContracts")
      + shader_profile_pass_kind("PixelMagicSurfaceContracts");
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.domain == "shader"
            && module.ast.unit == "PixelMagicSurfaceContracts"));

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.contains(
        "use\tcpu.Main\tshader.PixelMagicSurfaceContracts\tresolution=local-visible:galaxy-auto-inject:galaxy=pixelmagic\tpackage=nuis.pixelmagic\tlibrary_module=lib/shader_contracts.ns"
    ));

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert_eq!(artifacts.nir.unit, "Main");
}

#[test]
fn injects_transitive_core_and_std_library_modules_from_pixelmagic() {
    let root = write_temp_project_fixture(
        "galaxy_transitive_library_injection",
        r#"
name = "galaxy-transitive-library-injection"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
use cpu CorePrelude;
use cpu PixelMagicContracts;
use cpu StdTaskContracts;

mod cpu Main {
  fn main() -> i64 {
    return CorePrelude.sum3_i64(
      PixelMagicContracts.grayscale_packet_total(5101, 160, 120),
      StdTaskContracts.add_bias(7, 5),
      CorePrelude.one_i64()
    );
  }
}
"#,
        vec![],
    );

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "CorePrelude.sum3_i64"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "StdTaskContracts.add_bias"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.grayscale_packet_total"));
}

#[test]
fn injects_pixelmagic_shader_facing_contract_helpers_into_project_scope() {
    let root = write_temp_project_fixture(
        "galaxy_pixelmagic_shader_helpers",
        r#"
name = "galaxy-pixelmagic-shader-helpers"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
use cpu PixelMagicContracts;

mod cpu Main {
  fn main() -> i64 {
    let residency: i64 = PixelMagicContracts.resource_residency_total(6101, 1, 1, 2, 0);
    let color_base: i64 = PixelMagicContracts.shader_color_seed_base(
      6101,
      PixelMagicContracts.blur_op_kind(),
      1
    );
    let speed_base: i64 = PixelMagicContracts.shader_speed_seed_base(320, 4);
    let radius_base: i64 = PixelMagicContracts.shader_radius_seed_base(200, 0, 0);
    let lowered: PixelMagicSurfaceContractsPacket = PixelMagicContracts.surface_contract_packet(
      6101,
      320,
      200,
      PixelMagicContracts.blur_op_kind(),
      1,
      4,
      0,
      0
    );
    return residency + color_base + speed_base + radius_base +
      PixelMagicContracts.surface_contract_profile_total() +
      PixelMagicContracts.shader_pipeline_total(
        6101,
        320,
        200,
        PixelMagicContracts.blur_op_kind(),
        1,
        4,
        0,
        0,
        PixelMagicContracts.sample_output_kind_color()
      );
  }
}
"#,
        vec![],
    );

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.surface_contract_profile_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.surface_contract_packet"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.resource_residency_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_color_seed_base"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_speed_seed_base"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_radius_seed_base"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_packet_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_consumer_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.shader_pipeline_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.filter_chain_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.analysis_quality_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "PixelMagicContracts.texture_handoff_total"));
}

#[test]
fn injects_witsage_cpu_and_kernel_library_modules_into_project_scope() {
    let root = write_temp_project_fixture(
        "galaxy_witsage_kernel_helpers",
        r#"
name = "galaxy-witsage-kernel-helpers"
entry = "main.ns"
galaxy = ["witsage=workspace"]
"#,
        r#"
use cpu WitSageContracts;
use kernel WitSageKernelSurface;

mod cpu Main {
  fn main() -> i64 {
    let features = kernel_tensor(2, 3, "2,4,6,1,3,5");
    let reduced = kernel_reduce_mean_axis(features, "cols");
    let feature_seed: i64 = kernel_element_at(reduced, 0, 1);
    return WitSageContracts.classifier_pipeline_total(
      6401,
      160,
      3,
      128,
      32,
      WitSageContracts.zscore_normalization_kind(),
      WitSageContracts.linear_model_kind(),
      feature_seed
    ) + WitSageContracts.kernel_pipeline_total(
      6401,
      160,
      3,
      WitSageContracts.linear_model_kind(),
      WitSageContracts.kernel_reduce_plan_kind(),
      16,
      feature_seed
    );
  }
}
"#,
        vec![],
    );

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "WitSageContracts.classifier_pipeline_total"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "WitSageContracts.kernel_pipeline_total"));

    let project = load_project(root.as_path()).unwrap();
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.domain == "kernel" && module.ast.unit == "WitSageKernelSurface"));
}

#[test]
fn keeps_ns_nova_library_module_available_but_not_visible_by_default() {
    let root = write_temp_project_fixture(
        "galaxy_ns_nova_library_injection",
        r#"
name = "galaxy-ns-nova-library-injection"
entry = "main.ns"
galaxy = ["ns-nova=workspace"]
"#,
        r#"
use cpu CorePrelude;
use cpu StdTaskContracts;

mod cpu Main {
  fn main() -> i64 {
    return CorePrelude.sum3_i64(
      StdTaskContracts.add_bias(10, 5),
      StdTaskContracts.add_bias(20, 6),
      CorePrelude.one_i64()
    );
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    let resolved_names = project
        .resolved_galaxies
        .iter()
        .map(|item| item.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(resolved_names, vec!["core", "ns-nova", "std"]);
    let ns_nova = project
        .resolved_galaxies
        .iter()
        .find(|item| item.name == "ns-nova")
        .unwrap();
    assert_eq!(ns_nova.library_import_policy.as_str(), "manual-only");
    assert!(!ns_nova.auto_injectable);
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "CorePrelude"));
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "StdTaskContracts"));
    assert!(!project
        .modules
        .iter()
        .any(|module| module.ast.unit == "NovaContracts"));

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.contains(
        "library\tns-nova\tlib/nova_contracts.ns\timport_policy=manual-only\tauto_injectable=false\tvisible=false"
    ));
    assert!(!imports_index.contains("visible\tcpu\tNovaContracts"));

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "CorePrelude.sum3_i64"));
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "StdTaskContracts.add_bias"));
}

#[test]
fn reports_duplicate_binding_between_local_module_and_galaxy_injection() {
    let root = write_temp_project_fixture(
        "galaxy_duplicate_binding",
        r#"
name = "galaxy-duplicate-binding"
entry = "main.ns"
modules = ["main.ns", "shadow.ns"]
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        vec![(
            "shadow.ns",
            r#"
mod cpu CorePrelude {
  fn local_value() -> i64 {
    return 7;
  }
}
"#,
        )],
    );

    let error = load_project(root.as_path()).unwrap_err();
    assert!(error.contains("duplicate project mod definition for `mod cpu CorePrelude`"));
    assert!(error.contains("manifest_spec=shadow.ns"));
    assert!(error.contains("galaxy=core"));
    assert!(error.contains("package=nuis.core"));
    assert!(error.contains("library_module=lib/prelude_contracts.ns"));
    assert!(error.contains("import_policy=project-auto"));
}

#[test]
fn manual_only_library_policy_disables_auto_injection_during_resolution() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let stdlib_root = std::env::temp_dir().join(format!("nuisc_stdlib_manual_only_{nonce}"));
    fs::create_dir_all(stdlib_root.join("manual")).unwrap();
    fs::write(
        stdlib_root.join("index.toml"),
        r#"
name = "test-stdlib"
default_entry = "core"

[[module]]
name = "manual"
kind = "galaxy"
path = "manual"
package_id = "nuis.manual"
depends_on = []
summary = "manual policy test module"
"#,
    )
    .unwrap();
    fs::write(
        stdlib_root.join("manual").join("module.toml"),
        r#"
module_schema = "nuis-stdlib-module-v1"
name = "manual"
package_id = "nuis.manual"
tier = "framework"
depends_on = []
summary = "manual policy test module"
library_modules = ["lib/manual_contracts.ns"]
library_import_policy = "manual-only"
"#,
    )
    .unwrap();
    fs::create_dir_all(stdlib_root.join("manual").join("lib")).unwrap();
    fs::write(
        stdlib_root
            .join("manual")
            .join("lib")
            .join("manual_contracts.ns"),
        r#"
mod cpu ManualContracts {
  pub fn value() -> i64 {
    return 42;
  }
}
"#,
    )
    .unwrap();

    let resolved = crate::stdlib_registry::resolve_galaxy_dependencies(
        &stdlib_root,
        &[ProjectGalaxyDependency {
            name: "manual".to_owned(),
            version: "workspace".to_owned(),
        }],
    )
    .unwrap();
    assert_eq!(resolved.len(), 1);
    let manual = &resolved[0];
    assert_eq!(manual.library_import_policy.as_str(), "manual-only");
    assert!(!manual.auto_injectable);
    assert_eq!(
        manual.auto_inject_blockers,
        vec!["library import policy `manual-only` disables automatic project injection".to_owned()]
    );

    let galaxy_index = crate::stdlib_registry::render_resolved_galaxy_index(&resolved);
    assert!(galaxy_index.contains("library_import_policy=manual-only"));
    assert!(galaxy_index.contains(
        "blockers=library import policy `manual-only` disables automatic project injection"
    ));
}

#[test]
fn import_index_reports_manual_only_library_as_not_visible() {
    let project = LoadedProject {
        root: PathBuf::from("."),
        manifest_path: PathBuf::from("nuis.toml"),
        manifest: NuisProjectManifest {
            name: "manual-only-import-index".to_owned(),
            entry: "main.ns".to_owned(),
            modules: vec!["main.ns".to_owned()],
            tests: vec![],
            links: vec![],
            abi_requirements: vec![],
            galaxy_dependencies: vec![ProjectGalaxyDependency {
                name: "manual".to_owned(),
                version: "workspace".to_owned(),
            }],
            galaxy_imports: vec![],
        },
        entry_path: PathBuf::from("main.ns"),
        entry_source: r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#
        .to_owned(),
        modules: vec![ProjectModule {
            path: PathBuf::from("main.ns"),
            ast: crate::frontend::parse_nuis_ast(
                r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
            )
            .unwrap(),
            origin: ProjectModuleOrigin::LocalProject {
                manifest_spec: "main.ns".to_owned(),
            },
        }],
        resolved_galaxies: vec![crate::stdlib_registry::ResolvedGalaxyDependency {
            name: "manual".to_owned(),
            version: "workspace".to_owned(),
            package_id: "nuis.manual".to_owned(),
            direct: true,
            requested_by: vec!["manual".to_owned()],
            module_dir: PathBuf::from("stdlib/manual"),
            manifest_path: PathBuf::from("stdlib/manual/module.toml"),
            depends_on: vec![],
            surfaces: vec!["surface.manual.contracts.v1".to_owned()],
            source_modules: vec![],
            resolved_source_paths: vec![],
            library_modules: vec!["lib/manual_contracts.ns".to_owned()],
            resolved_library_paths: vec![PathBuf::from("stdlib/manual/lib/manual_contracts.ns")],
            library_import_policy: crate::stdlib_registry::StdlibLibraryImportPolicy::ManualOnly,
            auto_injectable: false,
            auto_inject_blockers: vec![
                "library import policy `manual-only` disables automatic project injection"
                    .to_owned(),
            ],
        }],
    };

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.contains(
        "library\tmanual\tlib/manual_contracts.ns\timport_policy=manual-only\tauto_injectable=false\tvisible=false"
    ));
    assert!(!imports_index.contains("visible\tcpu\tManualContracts"));
}

#[test]
fn reports_manual_only_library_when_project_tries_to_use_it_directly() {
    let root = write_temp_project_fixture(
        "galaxy_ns_nova_manual_only_use",
        r#"
name = "galaxy-ns-nova-manual-only-use"
entry = "main.ns"
galaxy = ["ns-nova=workspace"]
"#,
        r#"
use cpu NovaContracts;

mod cpu Main {
  fn main() -> i64 {
    return NovaContracts.runtime_score(16, 4, 3, 2, 9, 1);
  }
}
"#,
        vec![],
    );

    let error = load_project(root.as_path()).unwrap_err();
    assert!(
        error.contains("project use `use cpu NovaContracts;` is unavailable in the current scope")
    );
    assert!(error.contains("provided by galaxy `ns-nova` (nuis.ns-nova)"));
    assert!(error.contains("`lib/nova_contracts.ns`"));
    assert!(error.contains("import policy `manual-only`"));
}

#[test]
fn explicit_galaxy_import_allows_manual_only_library_module() {
    let root = write_temp_project_fixture(
        "galaxy_ns_nova_explicit_import",
        r#"
name = "galaxy-ns-nova-explicit-import"
entry = "main.ns"
galaxy = ["ns-nova=workspace"]
galaxy_imports = ["ns-nova:lib/nova_contracts.ns"]
"#,
        r#"
use cpu CorePrelude;
use cpu NovaContracts;
use cpu StdTaskContracts;

mod cpu Main {
  fn main() -> i64 {
    return CorePrelude.sum3_i64(
      NovaContracts.runtime_score(16, 4, 3, 2, 9, 1),
      StdTaskContracts.add_bias(10, 5),
      CorePrelude.one_i64()
    );
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    assert!(project
        .modules
        .iter()
        .any(|module| module.ast.unit == "NovaContracts"));
    assert!(project
        .modules
        .iter()
        .any(|module| module.origin.source_kind() == "galaxy-explicit-import"));

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.starts_with("summary\t"));
    assert!(imports_index.contains(
        "library\tns-nova\tlib/nova_contracts.ns\timport_policy=manual-only\tauto_injectable=false\tvisible=true"
    ));
    assert!(imports_index.contains("visible\tcpu\tNovaContracts\tdoc_items="));
    assert!(imports_index.contains(
        "use\tcpu.Main\tcpu.NovaContracts\tresolution=local-visible:galaxy-explicit-import:galaxy=ns-nova\tpackage=nuis.ns-nova\tlibrary_module=lib/nova_contracts.ns\timport_policy=manual-only"
    ));

    let artifacts = crate::pipeline::compile_project(root.as_path()).unwrap();
    assert!(artifacts
        .nir
        .functions
        .iter()
        .any(|function| function.name == "NovaContracts.runtime_score"));
}

#[test]
fn rejects_duplicate_galaxy_import_entries() {
    let root = write_temp_project_fixture(
        "galaxy_duplicate_imports",
        r#"
name = "galaxy-duplicate-imports"
entry = "main.ns"
galaxy = ["ns-nova=workspace"]
galaxy_imports = [
  "ns-nova:lib/nova_contracts.ns",
  "ns-nova:lib/nova_contracts.ns",
]
"#,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 0;
  }
}
"#,
        vec![],
    );

    let error = load_project(root.as_path()).unwrap_err();
    assert!(error.contains("declares duplicate galaxy_imports entry"));
    assert!(error.contains("ns-nova:lib/nova_contracts.ns"));
}

#[test]
fn explicit_import_of_auto_injected_library_keeps_single_visible_origin() {
    let root = write_temp_project_fixture(
        "galaxy_redundant_auto_import",
        r#"
name = "galaxy-redundant-auto-import"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
galaxy_imports = ["pixelmagic:lib/image_contracts.ns"]
"#,
        r#"
use cpu PixelMagicContracts;

mod cpu Main {
  fn main() -> i64 {
    return PixelMagicContracts.blur_op_kind();
  }
}
"#,
        vec![],
    );

    let project = load_project(root.as_path()).unwrap();
    let pixelmagic_modules = project
        .modules
        .iter()
        .filter(|module| module.ast.unit == "PixelMagicContracts")
        .collect::<Vec<_>>();
    assert_eq!(pixelmagic_modules.len(), 1);
    assert_eq!(
        pixelmagic_modules[0].origin.source_kind(),
        "galaxy-auto-inject"
    );

    let imports_index = render_project_import_index(&project);
    assert!(imports_index.contains(
        "summary\tlibraries=8\tvisible_libraries=8\tvisible_modules=9\tdocumented_visible_modules=8\tdocumented_visible_items=54"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/image_contracts.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/shader_contracts.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/packet_bridge_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/render_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/texture_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains(
        "library\tpixelmagic\tlib/pipeline_surface.ns\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
    ));
    assert!(imports_index.contains("visible\tcpu\tPixelMagicContracts\tdoc_items="));
    assert!(imports_index.contains("visible\tshader\tPixelMagicSurfaceContracts\tdoc_items="));
    assert!(!imports_index.contains("source_kind=galaxy-explicit-import\tgalaxy=pixelmagic"));
}
