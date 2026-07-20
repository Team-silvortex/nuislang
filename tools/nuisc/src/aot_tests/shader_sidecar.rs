use super::*;

#[test]
fn build_manifest_emits_shader_ir_sidecar() {
    let dir = temp_dir("build_manifest_shader_sidecar");
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
        packaging_mode: "native-cpu-llvm".to_owned(),
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
    let manifest = super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: dir.join("shader.ns").to_string_lossy().to_string(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned(), "official.shader".to_owned()],
            compile_cache: None,
            project: Some(BuildManifestProjectInfo {
                name: "shader".to_owned(),
                abi_mode: "explicit".to_owned(),
                abi_graph_summary: None,
                abi_entries: vec![
                    ("cpu".to_owned(), cpu_target.abi.clone()),
                    ("shader".to_owned(), "shader.metal.msl2_4".to_owned()),
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

    let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
    let shader_unit = report
        .domain_build_units
        .iter()
        .find(|unit| unit.domain_family == "shader")
        .unwrap();
    let shader_sidecar_path = dir.join("nuis.domain.shader.lowering.ir.txt");
    let shader_sidecar_path_text = shader_sidecar_path.display().to_string();
    let shader_payload_blob = dir.join("nuis.domain.shader.payload.bin");
    assert!(shader_sidecar_path.exists());
    assert_eq!(
        shader_unit.artifact_ir_sidecar_path.as_deref(),
        Some(shader_sidecar_path_text.as_str())
    );
    let shader_sidecar_text = fs::read_to_string(&shader_sidecar_path).unwrap();
    assert!(shader_sidecar_text.contains("schema = \"nuis-shader-ir-sidecar-v1\""));
    assert!(shader_sidecar_text.contains("lowering_profile = \"metal.apple-silicon-gpu\""));
    assert!(shader_sidecar_text.contains("lowering_ir = \"msl2.4\""));
    assert!(
        shader_sidecar_text.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]")
    );
    assert!(shader_sidecar_text.contains("ir_container = \"text.msl\""));
    assert!(shader_sidecar_text.contains("[schedule_contract]"));
    assert!(shader_sidecar_text.contains("execution_route = \"unified-render-graph\""));
    assert!(shader_sidecar_text.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(shader_sidecar_text.contains("wake_adapter = \"metal-shared-event\""));
    assert!(shader_sidecar_text.contains("clock_contract = \"global-time-partial-order\""));
    assert!(shader_sidecar_text.contains("[lowering_capabilities]"));
    assert!(shader_sidecar_text.contains("capability_owner = \"shader-nustar\""));
    assert!(shader_sidecar_text.contains("pipeline_lowering = \"metal-render-pipeline-state\""));
    assert!(shader_sidecar_text.contains("entry_symbol = \"main0\""));
    assert!(shader_sidecar_text.contains("[pipeline_layout]"));
    assert!(shader_sidecar_text.contains("[resource_bindings]"));
    assert!(shader_sidecar_text.contains("[entry_points]"));
    assert!(shader_sidecar_text.contains("vertex = \"vs_main\""));
    assert!(shader_sidecar_text.contains("compute = \"cs_main\""));
    assert!(shader_sidecar_text.contains("fragment float4 main0"));

    let shader_blob =
        super::decode_domain_build_unit_payload_blob(&fs::read(&shader_payload_blob).unwrap())
            .unwrap();
    assert_eq!(shader_blob.sections.len(), 5);
    assert_eq!(shader_blob.sections[4].name, "shader_ir_sidecar");
    assert_eq!(
        shader_blob.sections[4].bytes,
        shader_sidecar_text.as_bytes()
    );
}

#[test]
fn resolve_cpu_build_target_from_target_triple() {
    let registry_root = registry_root();
    let target = super::resolve_cpu_build_target_from_target(
        registry_root.as_path(),
        "x86_64-unknown-linux-gnu",
    )
    .unwrap();
    assert_eq!(target.machine_arch, "x86_64");
    assert_eq!(target.machine_os, "linux");
    assert_eq!(target.object_format, "elf");
    assert_eq!(target.calling_abi, "sysv64");
}

#[test]
fn resolve_cpu_build_target_from_darwin_amd64_alias_triple() {
    let registry_root = registry_root();
    let target =
        super::resolve_cpu_build_target_from_target(registry_root.as_path(), "amd64-apple-darwin")
            .unwrap();
    assert_eq!(target.abi, "cpu.x86_64.apple_sysv64");
    assert_eq!(target.machine_arch, "x86_64");
    assert_eq!(target.machine_os, "darwin");
    assert_eq!(target.object_format, "mach-o");
    assert_eq!(target.calling_abi, "sysv64");
    assert_eq!(target.clang_target, "x86_64-apple-darwin");
}

#[test]
fn reject_conflicting_cpu_abi_and_target_override() {
    let registry_root = registry_root();
    let error = super::resolve_cpu_build_target(
        registry_root.as_path(),
        None,
        Some("cpu.arm64.apple_aapcs64"),
        Some("x86_64-unknown-linux-gnu"),
    )
    .unwrap_err();
    assert!(error.contains("--cpu-abi"));
    assert!(error.contains("--target"));
}
