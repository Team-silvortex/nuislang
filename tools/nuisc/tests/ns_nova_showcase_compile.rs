use std::{fs, path::PathBuf};
#[cfg(target_os = "macos")]
use std::{
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(target_os = "macos")]
fn temp_output_dir() -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should follow the Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("nuis_ns_nova_showcase_{nonce}"))
}

#[cfg(target_os = "macos")]
#[test]
fn builds_ns_nova_showcase_window_aot_bundle() {
    let output_dir = temp_output_dir();
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let compile = Command::new(env!("CARGO_BIN_EXE_nuisc"))
        .args([
            "compile",
            "examples/projects/domains/ns_nova_showcase",
            output_dir
                .to_str()
                .expect("temporary output path should be UTF-8"),
        ])
        .current_dir(workspace_root)
        .output()
        .expect("nuisc should launch");
    assert!(
        compile.status.success(),
        "NS Nova AOT compile failed:\n{}",
        String::from_utf8_lossy(&compile.stderr)
    );

    let manifest = fs::read_to_string(output_dir.join("nuis.build.manifest.toml"))
        .expect("NS Nova build manifest should exist");
    assert!(manifest.contains("packaging_mode = \"window-aot-bundle\""));
    assert!(manifest.contains("official.cpu"));
    assert!(manifest.contains("official.data"));
    assert!(manifest.contains("official.shader"));
    assert!(output_dir.join("ns_nova_showcase").is_file());

    let render_table =
        fs::read_to_string(output_dir.join("nuis.domain.shader.render-codegen-table.toml"))
            .expect("NS Nova render codegen table should exist");
    assert!(render_table.contains("schema = \"nuis-shader-render-codegen-table-v1\""));
    assert!(render_table.contains("entries = [\"vs_main\", \"fs_main\"]"));
    assert!(render_table.contains("width = 160"));
    assert!(render_table.contains("height = 120"));
    let generated_asset = fs::read_dir(&output_dir)
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with("nuis.shader.project.") && name.ends_with(".metal")
                })
        })
        .expect("NS Nova generated Metal render asset should exist");
    let generated_source = fs::read_to_string(&generated_asset).unwrap();
    assert!(generated_source.contains("vertex NuisRasterOut vs_main"));
    assert!(generated_source.contains("fragment float4 fs_main"));
    let contributions =
        fs::read_to_string(output_dir.join("nuis.domain.code-asset-contributions.toml"))
            .expect("NS Nova code asset contribution table should exist");
    assert!(contributions.contains("asset_id = \"shader.metal.project."));
    assert!(contributions.contains("entry_count = 2"));
    assert!(contributions.contains(
        generated_asset
            .file_name()
            .and_then(|name| name.to_str())
            .expect("generated render asset name should be UTF-8")
    ));

    let yir = fs::read_to_string(output_dir.join("ns_nova_showcase.yir"))
        .expect("NS Nova YIR should exist");
    let ppm = yir_runtime_host::render_module_to_ppm_bytes(&yir, 1)
        .expect("registered Shader renderer should export the NS Nova frame");
    let header = b"P6\n160 120\n255\n";
    assert!(ppm.starts_with(header));
    assert_eq!(ppm.len(), header.len() + 160 * 120 * 3);

    fs::remove_dir_all(output_dir).expect("temporary NS Nova output should be removable");
}

#[cfg(not(target_os = "macos"))]
#[test]
fn keeps_ns_nova_showcase_manifest_host_adaptive() {
    let manifest = fs::read_to_string(PathBuf::from(
        "../../examples/projects/domains/ns_nova_showcase/nuis.toml",
    ))
    .expect("NS Nova project manifest should exist");
    assert!(!manifest.contains("abi ="));
    assert!(manifest.contains("ns-nova=workspace"));
    assert!(manifest.contains("pixelmagic=workspace"));
}
