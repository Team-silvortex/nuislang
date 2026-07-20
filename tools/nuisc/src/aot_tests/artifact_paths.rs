use super::*;

#[test]
fn verify_compiled_artifact_preserves_heterogeneous_domain_unit_paths() {
    let dir = temp_dir("verify_compiled_artifact_heterogeneous_units");
    let ast = dir.join("demo.ast.txt");
    let nir = dir.join("demo.nir.txt");
    let yir = dir.join("demo.yir");
    let ll = dir.join("demo.ll");
    let bin = dir.join("demo.bin");
    fs::write(&ast, "ast").unwrap();
    fs::write(&nir, "nir").unwrap();
    fs::write(&yir, "yir").unwrap();
    fs::write(&ll, "llvm").unwrap();
    fs::write(&bin, "bin").unwrap();

    let written = CompileArtifacts {
        ast_path: ast.display().to_string(),
        nir_path: nir.display().to_string(),
        yir_path: yir.display().to_string(),
        llvm_ir_path: ll.display().to_string(),
        binary_path: bin.display().to_string(),
        packaging_mode: "window-aot-bundle".to_owned(),
    };
    let cpu_target = CpuBuildTarget {
        abi: "cpu.arm64.apple_aapcs64".to_owned(),
        machine_arch: "arm64".to_owned(),
        machine_os: "darwin".to_owned(),
        object_format: "macho".to_owned(),
        calling_abi: "apple_aapcs64".to_owned(),
        clang_target: "arm64-apple-darwin".to_owned(),
        isa_family: "aarch64".to_owned(),
        isa_features: vec!["a64".to_owned(), "neon".to_owned()],
        cross_compile: false,
    };
    super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: dir.join("hetero_artifact.ns").to_string_lossy().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec![
                "official.cpu".to_owned(),
                "official.kernel".to_owned(),
                "official.network".to_owned(),
            ],
            compile_cache: None,
            project: Some(BuildManifestProjectInfo {
                name: "hetero_artifact".to_owned(),
                abi_mode: "explicit".to_owned(),
                abi_graph_summary: None,
                abi_entries: vec![
                    ("cpu".to_owned(), cpu_target.abi.clone()),
                    ("kernel".to_owned(), "kernel.apple_ane.coreml.v1".to_owned()),
                    (
                        "network".to_owned(),
                        "network.socket.macos.arm64.v1".to_owned(),
                    ),
                ],
                plan_summary: None,
                effective_input: None,
                text_handle_rewrite_helper_hits: 0,
                text_handle_rewrite_local_hits: 0,
                manifest_copy_path: None,
                plan_index_path: None,
                organization_index_path: None,
                exchange_index_path: None,
                modules_index_path: None,
                docs_index_path: None,
                docs_module_count: 0,
                docs_documented_module_count: 0,
                docs_documented_item_count: 0,
                imports_index_path: None,
                imports_library_count: 0,
                imports_visible_library_count: 0,
                imports_visible_module_count: 0,
                imports_documented_visible_module_count: 0,
                imports_documented_visible_item_count: 0,
                galaxy_index_path: None,
                galaxy_count: 0,
                galaxy_documented_count: 0,
                galaxy_documented_library_module_count: 0,
                galaxy_documented_item_count: 0,
                links_index_path: None,
                packet_index_path: None,
                host_ffi_index_path: None,
                abi_index_path: None,
            }),
            doc_index: None,
            cpu_target,
        },
    )
    .unwrap();

    fs::remove_file(dir.join("nuis.bridge.registry.toml")).unwrap();
    fs::remove_file(dir.join("nuis.host-bridge.plan-index.toml")).unwrap();
    fs::remove_file(dir.join("nuis.domain.kernel.payload.toml")).unwrap();
    fs::remove_file(dir.join("nuis.domain.kernel.payload.bin")).unwrap();
    fs::remove_file(dir.join("nuis.domain.kernel.bridge.stub.txt")).unwrap();
    fs::remove_file(dir.join("nuis.domain.network.payload.toml")).unwrap();
    fs::remove_file(dir.join("nuis.domain.network.payload.bin")).unwrap();
    fs::remove_file(dir.join("nuis.domain.network.bridge.stub.txt")).unwrap();

    let artifact_report =
        verify_nuis_compiled_artifact(PathBuf::from(&dir).join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert!(artifact_report.lifecycle_contract_consistent);
    assert!(artifact_report.lifecycle_runtime_capability_flags_consistent);
    assert!(artifact_report.artifact_roundtrip_verified);
}
