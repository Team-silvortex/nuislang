use super::bundle::decode_bundle;
use super::{
    escape, parse_manifest, parse_ns_nova_manifest, render_manifest, render_ns_nova_manifest,
    GalaxyManifest, NsNovaManifest, GALAXY_BUNDLE_VERSION, GALAXY_MAGIC,
};
use std::path::Path;

#[test]
fn renders_and_parses_optional_framework_metadata() {
    let manifest = GalaxyManifest {
        manifest_schema: "galaxy-manifest-v1".to_owned(),
        name: "nova-demo".to_owned(),
        version: "0.1.0".to_owned(),
        package_kind: "nuis-framework".to_owned(),
        framework: Some("ns-nova".to_owned()),
        project: "nuis.toml".to_owned(),
        summary: "Galaxy package for ns-nova framework project `nova-demo`".to_owned(),
        license: "UNLICENSED".to_owned(),
        repository: String::new(),
        authors: vec!["OpenAI".to_owned()],
        include: vec!["nuis.toml".to_owned(), "main.ns".to_owned()],
    };
    let rendered = render_manifest(&manifest);
    assert!(rendered.contains("framework = \"ns-nova\""));
    let parsed = parse_manifest(&rendered, Path::new("galaxy.toml")).unwrap();
    assert_eq!(parsed.framework.as_deref(), Some("ns-nova"));
    assert_eq!(parsed.package_kind, "nuis-framework");
}

#[test]
fn renders_without_framework_line_when_absent() {
    let manifest = GalaxyManifest {
        manifest_schema: "galaxy-manifest-v1".to_owned(),
        name: "plain-demo".to_owned(),
        version: "0.1.0".to_owned(),
        package_kind: "nuis-project".to_owned(),
        framework: None,
        project: "nuis.toml".to_owned(),
        summary: "Galaxy package for nuis project `plain-demo`".to_owned(),
        license: "UNLICENSED".to_owned(),
        repository: String::new(),
        authors: Vec::new(),
        include: vec!["nuis.toml".to_owned()],
    };
    let rendered = render_manifest(&manifest);
    assert!(!rendered.contains("\nframework = "));
    let parsed = parse_manifest(&rendered, Path::new("galaxy.toml")).unwrap();
    assert_eq!(parsed.framework, None);
}

#[test]
fn escape_still_handles_quotes_and_slashes() {
    assert_eq!(escape("a\\b\"c"), "a\\\\b\\\"c");
}

#[test]
fn parse_manifest_rejects_path_like_package_name() {
    let source = "manifest_schema = \"galaxy-manifest-v1\"\nname = \"../evil\"\nversion = \"0.1.0\"\npackage_kind = \"nuis-project\"\nproject = \"nuis.toml\"\ninclude = [\"nuis.toml\"]\n";
    let error = match parse_manifest(source, Path::new("galaxy.toml")) {
        Ok(_) => panic!("path-like galaxy name should fail"),
        Err(error) => error,
    };
    assert!(error.contains("unsafe galaxy name"));
}

#[test]
fn decode_bundle_rejects_entry_path_traversal() {
    let manifest = "manifest_schema = \"galaxy-manifest-v1\"\nname = \"safe-demo\"\nversion = \"0.1.0\"\npackage_kind = \"nuis-project\"\nproject = \"nuis.toml\"\ninclude = [\"nuis.toml\"]\n";
    let entry_path = "../evil.txt";
    let content = b"owned";
    let mut bytes = Vec::new();
    bytes.extend_from_slice(GALAXY_MAGIC);
    bytes.extend_from_slice(&GALAXY_BUNDLE_VERSION.to_le_bytes());
    bytes.extend_from_slice(&(manifest.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(manifest.as_bytes());
    bytes.extend_from_slice(&(entry_path.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(content.len() as u64).to_le_bytes());
    bytes.extend_from_slice(entry_path.as_bytes());
    bytes.extend_from_slice(content);

    let error = match decode_bundle(&bytes, Path::new("unsafe.galaxy")) {
        Ok(_) => panic!("bundle entry path traversal should fail"),
        Err(error) => error,
    };
    assert!(error.contains("unsafe galaxy entry path"));
}

#[test]
fn renders_and_parses_ns_nova_profile() {
    let manifest = NsNovaManifest {
        framework_schema: "ns-nova-manifest-v1".to_owned(),
        framework: "ns-nova".to_owned(),
        project: "nuis.toml".to_owned(),
        stdlib_schema: Some("ns-nova-stdlib-v1".to_owned()),
        stdlib_manifest: Some("stdlib/ns-nova/module.toml".to_owned()),
        stdlib_sources: vec![
            "core/theme_surface.ns".to_owned(),
            "ui/panel_selection.ns".to_owned(),
        ],
        family_schema: Some("ns-nova-family-v1".to_owned()),
        family_layers: vec!["core".to_owned(), "ui".to_owned()],
        entry_cpu_unit: Some("cpu.Main".to_owned()),
        primary_data_unit: Some("data.FabricPlane".to_owned()),
        primary_shader_unit: Some("shader.SurfaceShader".to_owned()),
        primary_kernel_unit: None,
        render_schema: Some("ns-nova-render-v1".to_owned()),
        render_owner_unit: Some("cpu.Main".to_owned()),
        render_bridge_unit: Some("data.FabricPlane".to_owned()),
        render_surface_unit: Some("shader.SurfaceShader".to_owned()),
        selection_schema: Some("ns-nova-selection-v1".to_owned()),
        selection_owner_unit: Some("cpu.Main".to_owned()),
        selection_bridge_unit: Some("data.FabricPlane".to_owned()),
        selection_render_unit: Some("shader.SurfaceShader".to_owned()),
        selection_controls: vec![
            "list".to_owned(),
            "table".to_owned(),
            "tree".to_owned(),
            "inspector".to_owned(),
            "outline".to_owned(),
        ],
        render_links: vec![
            "cpu.Main -> shader.SurfaceShader via data.FabricPlane".to_owned(),
            "shader.SurfaceShader -> cpu.Main via data.FabricPlane".to_owned(),
        ],
        cpu_units: vec!["Main".to_owned()],
        data_units: vec!["FabricPlane".to_owned()],
        shader_units: vec!["SurfaceShader".to_owned()],
        kernel_units: Vec::new(),
    };
    let rendered = render_ns_nova_manifest(&manifest);
    let parsed = parse_ns_nova_manifest(&rendered, Path::new("ns-nova.toml")).unwrap();
    assert_eq!(parsed, manifest);
}
