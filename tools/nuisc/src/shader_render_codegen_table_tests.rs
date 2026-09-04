use super::*;

#[test]
fn projects_ns_nova_inline_render_into_content_addressed_msl() {
    let workspace_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project_root = workspace_root.join("examples/projects/domains/ns_nova_showcase");
    let project = crate::pipeline::compile_project(&project_root).unwrap();
    let source = crate::render::render_yir(&project.yir);
    let table = table_from_compiled_project_yir(&source, "metal.apple-silicon-gpu")
        .unwrap()
        .expect("NS Nova render table");

    assert_eq!(table.assets.len(), 1);
    assert_eq!(table.passes.len(), 1);
    assert_eq!(table.assets[0].entries, ["vs_main", "fs_main"]);
    assert_eq!(table.assets[0].format, "metal-source");
    assert!(table.assets[0]
        .source
        .contains("vertex NuisRasterOut vs_main"));
    assert!(table.assets[0].source.contains("fragment float4 fs_main"));
    assert_eq!(table.passes[0].width, 160);
    assert_eq!(table.passes[0].height, 120);
    assert_eq!(table.passes[0].asset_id, table.assets[0].asset_id);
    assert!(table.assets[0]
        .file_name
        .ends_with(&format!("{}.metal", &table.assets[0].content_hash[2..])));

    let rendered = render_codegen_table(&table).unwrap();
    assert!(rendered.contains("schema = \"nuis-shader-render-codegen-table-v1\""));
    assert!(rendered.contains("entries = [\"vs_main\", \"fs_main\"]"));
    assert!(rendered.contains("width = 160"));
    assert!(!include_str!("shader_render_codegen_table.rs").contains("PixelMagic"));
    assert!(!include_str!("shader_render_codegen_table.rs").contains("ns_nova"));
}

#[test]
fn rejects_unregistered_render_target() {
    let source = "version 0.1\n";
    assert!(
        table_from_compiled_project_yir(source, "vulkan.discrete-or-integrated-gpu")
            .unwrap_err()
            .contains("no registered producer")
    );
}
