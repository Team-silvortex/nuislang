use super::test_support::write_temp_project_fixture;
use super::*;

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
        "summary\tlibraries=21\tvisible_libraries=21\tvisible_modules=22\tdocumented_visible_modules=21\tdocumented_visible_items="
    ));
    for library in [
        "image_contracts.ns",
        "shader_contracts.ns",
        "packet_bridge_surface.ns",
        "render_surface.ns",
        "texture_surface.ns",
        "pipeline_surface.ns",
    ] {
        assert!(imports_index.contains(&format!(
            "library\tpixelmagic\tlib/{library}\timport_policy=project-auto\tauto_injectable=true\tvisible=true"
        )));
    }
    assert!(imports_index.contains("visible\tcpu\tPixelMagicContracts\tdoc_items="));
    assert!(imports_index.contains("visible\tshader\tPixelMagicSurfaceContracts\tdoc_items="));
    assert!(!imports_index.contains("source_kind=galaxy-explicit-import\tgalaxy=pixelmagic"));
}
