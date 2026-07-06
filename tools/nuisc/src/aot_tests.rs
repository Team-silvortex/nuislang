use super::{
    build_nuis_lifecycle_contract, c_shim_source, decode_nuis_compiled_artifact_binary,
    decode_nuis_executable_envelope_binary, encode_nuis_compiled_artifact_binary,
    encode_nuis_compiled_artifact_section_table_binary, encode_nuis_executable_envelope_binary,
    inspect_nuis_compiled_artifact_container, parse_nuis_compiled_artifact,
    parse_nuis_executable_envelope, render_nuis_executable_envelope,
    resolve_cpu_build_target_from_abi, verify_build_manifest, verify_nuis_compiled_artifact,
    BuildManifestCacheInfo, BuildManifestContext, BuildManifestDomainBuildUnit,
    BuildManifestProjectInfo, CompileArtifacts, CpuBuildTarget, NuisExecutableEnvelope,
};
use nuis_artifact::{
    decode_nuis_compiled_artifact_section_table_binary,
    encode_nuis_compiled_artifact_section_table,
    protocol::COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML,
};
use nuis_semantics::model::{AstExternFunction, AstModule, AstTypeRef, AstVisibility};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn registry_root() -> PathBuf {
    let root = temp_dir("nustar_registry");
    fs::write(
            root.join("index.toml"),
            "[[package]]\npackage_id = \"official.cpu\"\nmanifest = \"cpu.toml\"\ndomain_family = \"cpu\"\n",
        )
        .unwrap();
    fs::write(
            root.join("cpu.toml"),
            "manifest_schema = \"nustar-manifest-v1\"\npackage_id = \"official.cpu\"\ndomain_family = \"cpu\"\nfrontend = \"nustar-cpu\"\nentry_crate = \"crates/yir-domain-cpu\"\nast_entry = \"cpu.ast.bootstrap.v1\"\nnir_entry = \"cpu.nir.bootstrap.v1\"\nyir_lowering_entry = \"cpu.yir.lowering.v1\"\npart_verify_entry = \"cpu.verify.partial.v1\"\nast_surface = [\"cpu.mod-ast.v1\"]\nnir_surface = [\"nir.cpu.surface.v1\"]\nyir_lowering = [\"yir.cpu.lowering.v1\"]\npart_verify = [\"verify.cpu.contract.v1\"]\nbinary_extension = \"nustar\"\npackage_layout = \"single-envelope\"\nmachine_abi_policy = \"exact-match\"\nabi_profiles = [\"cpu.arm64.apple_aapcs64\", \"cpu.x86_64.apple_sysv64\", \"cpu.x86_64.sysv64\", \"cpu.x86_64.win64\"]\nabi_capabilities = [\"cpu.arm64.apple_aapcs64:op:cpu.*\", \"cpu.x86_64.apple_sysv64:op:cpu.*\", \"cpu.x86_64.sysv64:op:cpu.*\", \"cpu.x86_64.win64:op:cpu.*\"]\nabi_targets = [\"cpu.arm64.apple_aapcs64:arch=arm64|os=darwin|object=mach-o|calling=aapcs64-darwin|clang=aarch64-apple-darwin\", \"cpu.x86_64.apple_sysv64:arch=x86_64|os=darwin|object=mach-o|calling=sysv64|clang=x86_64-apple-darwin\", \"cpu.x86_64.sysv64:arch=x86_64|os=linux|object=elf|calling=sysv64|clang=x86_64-unknown-linux-gnu\", \"cpu.x86_64.win64:arch=x86_64|os=windows|object=coff|calling=win64|clang=x86_64-pc-windows-msvc\"]\nimplementation_kinds = [\"native-stub\"]\nloader_entry = \"nustar.bootstrap.v1\"\nloader_abi = \"nustar-loader-v1\"\nhost_ffi_surface = []\nhost_ffi_abis = []\nhost_ffi_bridge = \"none\"\nsupport_surface = []\nsupport_profile_slots = []\ndefault_lanes = []\nprofiles = [\"aot\"]\nresource_families = [\"cpu\", \"cpu.arm64\", \"cpu.x86_64\"]\nunit_types = [\"Main\"]\nlowering_targets = [\"llvm\", \"x86_64\"]\nops = [\"cpu.const\"]\n",
        )
        .unwrap();
    root
}

fn write_minimal_cpu_artifact(label: &str) -> (PathBuf, PathBuf) {
    let dir = temp_dir(label);
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
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
        isa_family: "x86_64".to_owned(),
        isa_features: vec!["x86-64".to_owned(), "sse2".to_owned()],
        cross_compile: true,
    };
    let manifest = super::write_build_manifest(
        &dir,
        &written,
        &BuildManifestContext {
            input_path: "/tmp/demo.ns".to_owned(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec!["official.cpu".to_owned()],
            compile_cache: None,
            project: None,
            doc_index: None,
            cpu_target,
        },
    )
    .unwrap();
    (dir, PathBuf::from(manifest))
}

#[test]
fn verify_compiled_artifact_rejects_binary_name_with_path_traversal() {
    let (dir, _manifest) = write_minimal_cpu_artifact("artifact_binary_name_traversal");
    let artifact_path = dir.join("nuis.compiled.artifact");
    let mut artifact = parse_nuis_compiled_artifact(&artifact_path).unwrap();
    artifact.binary_name = "../evil".to_owned();
    super::write_nuis_compiled_artifact(&artifact_path, &artifact).unwrap();

    let error = match verify_nuis_compiled_artifact(&artifact_path) {
        Ok(_) => panic!("artifact with traversal binary_name should fail verification"),
        Err(error) => error,
    };
    assert!(error.contains("unsafe binary_name"));
    assert!(error.contains("single file name"));
}

#[test]
fn inspect_compiled_artifact_container_rejects_lowering_index_manifest_drift() {
    let (dir, _manifest) = write_minimal_cpu_artifact("artifact_lowering_index_drift");
    let artifact_path = dir.join("nuis.compiled.artifact");
    let artifact = parse_nuis_compiled_artifact(&artifact_path).unwrap();
    let encoded = encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap();
    let mut table = decode_nuis_compiled_artifact_section_table_binary(&encoded).unwrap();
    let lowering_section = table
        .sections
        .iter_mut()
        .find(|section| section.name == COMPILED_ARTIFACT_SECTION_LOWERING_INDEX_TOML)
        .unwrap();
    let drifted = std::str::from_utf8(&lowering_section.bytes)
        .unwrap()
        .replace(
            "selected_lowering_target = \"llvm\"",
            "selected_lowering_target = \"shader-msl\"",
        );
    lowering_section.bytes = drifted.into_bytes();
    let drifted_path = dir.join("nuis.compiled.drifted.v2.artifact");
    fs::write(
        &drifted_path,
        encode_nuis_compiled_artifact_section_table(&table).unwrap(),
    )
    .unwrap();

    let error = inspect_nuis_compiled_artifact_container(&drifted_path).unwrap_err();

    assert!(error.contains("inconsistent nuis artifact section payloads"));
    assert!(error.contains("selected_lowering_target"));
    assert!(error.contains("shader-msl"));
}

#[test]
fn verify_build_manifest_rejects_artifact_path_outside_output_dir() {
    let (dir, manifest) = write_minimal_cpu_artifact("manifest_artifact_path_traversal");
    let mut source = fs::read_to_string(&manifest).unwrap();
    source = source.replace(
        &format!(
            "artifact_path = \"{}\"",
            dir.join("nuis.compiled.artifact").display()
        ),
        &format!(
            "artifact_path = \"{}\"",
            dir.join("..")
                .join("evil")
                .join("nuis.compiled.artifact")
                .display()
        ),
    );
    fs::write(&manifest, source).unwrap();

    let error = match verify_build_manifest(&manifest) {
        Ok(_) => panic!("manifest with traversal artifact_path should fail verification"),
        Err(error) => error,
    };
    assert!(error.contains("unsafe nuis_artifact.artifact_path"));
    assert!(error.contains("parent-directory traversal"));
}

#[test]
fn verify_build_manifest_rejects_artifact_hash_path_outside_output_dir() {
    let (dir, manifest) = write_minimal_cpu_artifact("manifest_artifact_hash_traversal");
    let mut source = fs::read_to_string(&manifest).unwrap();
    source = source.replace(
        &format!("path = \"{}\"", dir.join("demo.ast.txt").display()),
        &format!(
            "path = \"{}\"",
            dir.join("..").join("evil").join("demo.ast.txt").display()
        ),
    );
    fs::write(&manifest, source).unwrap();

    let error = match verify_build_manifest(&manifest) {
        Ok(_) => panic!("manifest with traversal artifact_hash path should fail verification"),
        Err(error) => error,
    };
    assert!(error.contains("unsafe artifact_hash.path"));
    assert!(error.contains("parent-directory traversal"));
}

#[test]
fn project_metadata_summary_mismatch_error_suggests_rebuild_for_legacy_outputs() {
    let source_root = temp_dir("metadata_mismatch_source_exists");
    let message = super::project_metadata_summary_mismatch_error(
        "galaxy",
        "build/nuis.project.galaxy.txt",
        "summary\tgalaxies=1",
        "summary\tgalaxies=0\ncore\tpackage=nuis.core",
        &source_root.display().to_string(),
        "build",
    );
    assert!(message.contains("project galaxy index `build/nuis.project.galaxy.txt`"));
    assert!(message.contains("expected `summary\tgalaxies=1`"));
    assert!(message.contains("found `summary\tgalaxies=0`"));
    assert!(message.contains("older nuisc metadata format"));
    assert!(message.contains("Rebuild the project with the current nuisc"));
    assert!(message.contains(&format!(
        "nuisc compile \"{}\" \"build\"",
        source_root.display()
    )));
    assert!(message.contains(&format!(
        "nuisc inspect-project-metadata \"{}\"",
        source_root.display()
    )));
}

#[test]
fn project_metadata_summary_mismatch_error_falls_back_to_manifest_commands_when_source_missing() {
    let message = super::project_metadata_summary_mismatch_error(
        "galaxy",
        "build/nuis.project.galaxy.txt",
        "summary\tgalaxies=1",
        "summary\tgalaxies=0\ncore\tpackage=nuis.core",
        "/tmp/definitely-missing-nuis-project-input",
        "build/out",
    );
    assert!(message.contains("older nuisc metadata format"));
    assert!(
        message.contains("nuisc inspect-project-metadata \"build/out/nuis.build.manifest.toml\"")
    );
    assert!(message.contains("nuisc verify-build-manifest \"build/out/nuis.build.manifest.toml\""));
}

fn sample_domain_unit(
    domain_family: &str,
    package_id: &str,
    backend_family: &str,
    vendor: &str,
    device_class: &str,
    selected_lowering_target: &str,
) -> BuildManifestDomainBuildUnit {
    BuildManifestDomainBuildUnit {
        package_id: package_id.to_owned(),
        domain_family: domain_family.to_owned(),
        abi: None,
        machine_arch: Some("arm64".to_owned()),
        machine_os: Some("darwin".to_owned()),
        backend_family: Some(backend_family.to_owned()),
        vendor: Some(vendor.to_owned()),
        device_class: Some(device_class.to_owned()),
        target_device: Some(device_class.to_owned()),
        ir_format: None,
        dispatch_abi: None,
        backend_priority: None,
        verification: Some("contract-only".to_owned()),
        selected_lowering_target: Some(selected_lowering_target.to_owned()),
        artifact_stub_path: None,
        artifact_stub_inline: None,
        artifact_payload_path: None,
        artifact_bridge_stub_path: None,
        artifact_ir_sidecar_path: None,
        artifact_bridge_stub_inline: None,
        artifact_payload_blob_path: None,
        artifact_payload_blob_bytes: None,
        artifact_payload_format: None,
        artifact_payload_blob_inline: None,
        contract_family: format!("nustar.{domain_family}"),
        packaging_role: "hetero-contract".to_owned(),
    }
}

#[test]
fn resolve_cpu_build_target_for_known_abis() {
    let registry_root = registry_root();
    let apple =
        resolve_cpu_build_target_from_abi(&registry_root, "cpu.arm64.apple_aapcs64").unwrap();
    assert_eq!(apple.machine_arch, "arm64");
    assert_eq!(apple.machine_os, "darwin");
    assert_eq!(apple.clang_target, "aarch64-apple-darwin");
    assert_eq!(apple.isa_family, "aarch64");
    assert!(apple.isa_features.contains(&"neon".to_owned()));
    assert!(apple.isa_features.contains(&"lse".to_owned()));

    let apple_amd64 =
        resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.apple_sysv64").unwrap();
    assert_eq!(apple_amd64.machine_arch, "x86_64");
    assert_eq!(apple_amd64.machine_os, "darwin");
    assert_eq!(apple_amd64.object_format, "mach-o");
    assert_eq!(apple_amd64.calling_abi, "sysv64");
    assert_eq!(apple_amd64.clang_target, "x86_64-apple-darwin");
    assert_eq!(apple_amd64.isa_family, "x86_64");
    assert!(apple_amd64.isa_features.contains(&"sse2".to_owned()));
    assert!(apple_amd64.isa_features.contains(&"avx2".to_owned()));

    let linux = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.sysv64").unwrap();
    assert_eq!(linux.machine_arch, "x86_64");
    assert_eq!(linux.machine_os, "linux");
    assert_eq!(linux.object_format, "elf");
    assert_eq!(linux.calling_abi, "sysv64");
    assert_eq!(linux.isa_family, "x86_64");
    assert!(linux.isa_features.contains(&"bmi2".to_owned()));

    let windows = resolve_cpu_build_target_from_abi(&registry_root, "cpu.x86_64.win64").unwrap();
    assert_eq!(windows.machine_os, "windows");
    assert_eq!(windows.clang_target, "x86_64-pc-windows-msvc");
    assert_eq!(windows.isa_family, "x86_64");
    assert!(windows.isa_features.contains(&"sse4.2".to_owned()));
    assert!(!windows.isa_features.contains(&"avx2".to_owned()));
}

#[test]
fn shader_lowering_and_stub_include_profile_aware_fields() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "metal",
        "apple",
        "apple-silicon-gpu",
        "metal.apple-silicon-gpu",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);
    let host_bridge_stub = super::render_domain_build_unit_host_bridge_stub(&shader_unit);

    assert!(lowering_plan.contains("lowering_profile = \"metal.apple-silicon-gpu\""));
    assert!(lowering_plan.contains("execution_route = \"unified-render-graph\""));
    assert!(lowering_plan.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(lowering_plan.contains("wake_adapter = \"metal-shared-event\""));
    assert!(lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]"));
    assert!(lowering_plan.contains("shader.profile.texture.v1"));
    assert!(lowering_plan.contains("shader.profile.sample-path.v1"));
    assert!(
        lowering_plan.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]")
    );
    assert!(lowering_plan.contains("lowering_ir = \"msl2.4\""));
    assert!(lowering_plan.contains("shader_stage_model = \"metal-render-pipeline\""));
    assert!(lowering_plan.contains("stage_binding_model = \"argument-buffer-specialized\""));
    assert!(lowering_plan.contains("dispatch_encoding_model = \"tile-and-threadgroup\""));

    assert!(backend_stub.contains("backend_profile = \"metal.apple-silicon-gpu\""));
    assert!(backend_stub.contains("execution_route = \"unified-render-graph\""));
    assert!(backend_stub.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(backend_stub.contains("wake_adapter = \"metal-shared-event\""));
    assert!(backend_stub.contains("shader_ir = \"msl2.4\""));
    assert!(backend_stub.contains("shader_entry_model = \"metal-function-constant-specialized\""));
    assert!(backend_stub.contains("queue_binding_model = \"unified-command-queue\""));
    assert!(backend_stub.contains("resource_binding_model = \"argument-buffer-table\""));

    assert!(host_bridge_stub.contains("bridge_profile = \"metal.apple-silicon-gpu\""));
    assert!(host_bridge_stub.contains("execution_route = \"unified-render-graph\""));
    assert!(host_bridge_stub.contains("submission_adapter = \"metal-command-encoder\""));
    assert!(host_bridge_stub.contains("wake_adapter = \"metal-shared-event\""));
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
    assert!(sidecar.contains("ir_container = \"text.msl\""));
    assert!(sidecar.contains("shader.profile.bind-set.v1"));
    assert!(sidecar.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]"));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"shader-nustar\""));
    assert!(sidecar.contains("native_ir = \"msl2.4\""));
    assert!(sidecar.contains("resource_lowering = \"argument-buffer-table\""));
    assert!(sidecar.contains("texture_lowering = \"texture2d-sampler-argument\""));
    assert!(sidecar.contains("shader.stage-interface"));
    assert!(sidecar.contains("entry_symbol = \"main0\""));
    assert!(sidecar.contains("stage_kind = \"fragment\""));
    assert!(sidecar.contains("resource_layout = \"argument-buffer\""));
    assert!(sidecar.contains("[pipeline_layout]"));
    assert!(sidecar.contains("color_targets = [\"rgba8unorm\"]"));
    assert!(sidecar.contains("threadgroup_topology = \"tile\""));
    assert!(sidecar.contains("[resource_bindings]"));
    assert!(sidecar.contains("binding_table = \"material.uniforms, frame.texture0\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("vertex = \"vs_main\""));
    assert!(sidecar.contains("fragment = \"main0\""));
    assert!(sidecar.contains("compute = \"cs_main\""));
    assert!(sidecar.contains("#include <metal_stdlib>"));
    assert!(sidecar.contains("vertex float4 vs_main"));
    assert!(sidecar.contains("fragment float4 main0"));
    assert!(sidecar.contains("kernel void cs_main"));
}

#[test]
fn shader_vulkan_lowering_plan_switches_to_spirv_pipeline_profile() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "vulkan",
        "cross-vendor",
        "discrete-or-integrated-gpu",
        "vulkan.discrete-or-integrated-gpu",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&shader_unit);

    assert!(lowering_plan.contains("lowering_profile = \"vulkan.discrete-or-integrated-gpu\""));
    assert!(lowering_plan.contains("execution_route = \"spirv-render-queue\""));
    assert!(lowering_plan.contains("submission_adapter = \"vulkan-command-buffer\""));
    assert!(lowering_plan.contains("wake_adapter = \"vulkan-timeline-semaphore\""));
    assert!(lowering_plan.contains("supported_stages = [\"vertex\", \"fragment\", \"compute\"]"));
    assert!(lowering_plan.contains("shader.profile.sampler.v1"));
    assert!(
        lowering_plan.contains("registered_lane_groups = [\"setup\", \"resource\", \"render\"]")
    );
    assert!(lowering_plan.contains("lowering_ir = \"spirv1.6\""));
    assert!(lowering_plan.contains("shader_stage_model = \"spirv-graphics-pipeline\""));
    assert!(lowering_plan.contains("stage_binding_model = \"descriptor-set-layout\""));
    assert!(lowering_plan.contains("dispatch_encoding_model = \"renderpass-command-buffer\""));

    assert!(backend_stub.contains("backend_profile = \"vulkan.discrete-or-integrated-gpu\""));
    assert!(backend_stub.contains("shader_ir = \"spirv1.6\""));
    assert!(backend_stub.contains("shader_entry_model = \"vulkan-pipeline\""));
    assert!(backend_stub.contains("queue_binding_model = \"explicit-device-queue\""));
    assert!(backend_stub.contains("resource_binding_model = \"descriptor-set-layout\""));
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);
    assert!(sidecar.contains("ir_container = \"text.spirv\""));
    assert!(sidecar.contains("pipeline_lowering = \"vulkan-graphics-pipeline\""));
    assert!(sidecar.contains("resource_lowering = \"descriptor-set-layout\""));
    assert!(sidecar.contains("texture_lowering = \"sampled-image-descriptor\""));
    assert!(sidecar.contains("spirv.interface-layout"));
    assert!(sidecar.contains("entry_symbol = \"main\""));
    assert!(sidecar.contains("stage_kind = \"fragment\""));
    assert!(sidecar.contains("resource_layout = \"descriptor-set\""));
    assert!(sidecar.contains("[pipeline_layout]"));
    assert!(sidecar.contains("threadgroup_topology = \"quad-fragment\""));
    assert!(sidecar.contains("[resource_bindings]"));
    assert!(sidecar.contains("binding_table = \"set0.binding0.texture, set0.binding1.sampler\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("vertex = \"vs_main\""));
    assert!(sidecar.contains("fragment = \"main\""));
    assert!(sidecar.contains("compute = \"cs_main\""));
    assert!(sidecar.contains("OpCapability Shader"));
    assert!(sidecar.contains("OpEntryPoint Vertex %vs_main"));
    assert!(sidecar.contains("OpEntryPoint Fragment %main"));
    assert!(sidecar.contains("OpEntryPoint GLCompute %cs_main"));
}

#[test]
fn shader_unknown_profile_falls_back_to_fragment_only_stage_set() {
    let shader_unit = sample_domain_unit(
        "shader",
        "official.shader",
        "experimental",
        "generic",
        "fragment-only-lab",
        "experimental.fragment-only-lab",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&shader_unit);
    let sidecar = super::render_domain_build_unit_shader_ir_sidecar(&shader_unit);

    assert!(lowering_plan.contains("supported_stages = [\"fragment\"]"));
    assert!(sidecar.contains("supported_stages = [\"fragment\"]"));
    assert!(sidecar.contains("entry_symbol = \"unimplemented\""));
    assert!(sidecar.contains("fragment = \"unimplemented\""));
    assert!(!sidecar.contains("vertex = "));
    assert!(!sidecar.contains("compute = "));
}

#[test]
fn kernel_coreml_profile_reports_dispatch_kinds() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "coreml",
        "apple",
        "apple-ane",
        "coreml.apple-ane",
    );
    let lowering_plan = super::render_domain_build_unit_lowering_plan(&kernel_unit);
    let backend_stub = super::render_domain_build_unit_backend_stub(&kernel_unit);

    assert!(lowering_plan.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
    assert!(lowering_plan.contains("kernel.profile.tensor-reduce.v1"));
    assert!(lowering_plan.contains("kernel.profile.result-buffer.v1"));
    assert!(lowering_plan.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(backend_stub.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
}

#[test]
fn kernel_coreml_sidecar_emits_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "coreml",
        "apple",
        "apple-ane",
        "coreml.apple-ane",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]"));
    assert!(sidecar.contains("kernel.profile.tensor-selection.v1"));
    assert!(sidecar.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"kernel-nustar\""));
    assert!(sidecar.contains("native_ir = \"coreml-program\""));
    assert!(sidecar.contains("tensor_lowering = \"ranked-tensor-graph\""));
    assert!(sidecar.contains("dispatch_lowering = \"ane-graph-submit\""));
    assert!(sidecar.contains("kernel.shape-contract"));
    assert!(sidecar.contains("[dispatch_shapes]"));
    assert!(sidecar.contains("primary = \"graph\""));
    assert!(sidecar.contains("[entry_points]"));
    assert!(sidecar.contains("graph = \"infer_main\""));
    assert!(sidecar.contains("batch = \"infer_batch\""));
    assert!(sidecar.contains("graph_body = \"program infer_main"));
}

#[test]
fn kernel_vulkan_sidecar_emits_grid_and_indirect_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "vulkan",
        "cross-vendor",
        "discrete-or-integrated-gpu",
        "vulkan.discrete-or-integrated-gpu",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"grid\", \"indirect\", \"batch\"]"));
    assert!(sidecar.contains("native_ir = \"spirv1.6\""));
    assert!(sidecar.contains("tensor_lowering = \"storage-buffer-tensor-view\""));
    assert!(sidecar.contains("dispatch_lowering = \"compute-grid-or-indirect\""));
    assert!(sidecar.contains("spirv.compute-layout"));
    assert!(sidecar.contains("primary = \"grid\""));
    assert!(sidecar.contains("fallback = \"indirect\""));
    assert!(sidecar.contains("binding_table = \"set0.buffer0, set0.buffer1\""));
    assert!(sidecar.contains("grid = \"main\""));
    assert!(sidecar.contains("indirect = \"main_indirect\""));
    assert!(sidecar.contains("OpEntryPoint GLCompute %main"));
}

#[test]
fn kernel_cpu_fallback_sidecar_emits_range_and_tile_dispatch_templates() {
    let kernel_unit = sample_domain_unit(
        "kernel",
        "official.kernel",
        "cpu-fallback",
        "generic",
        "cpu-host",
        "cpu-fallback.cpu-host",
    );
    let sidecar = super::render_domain_build_unit_kernel_ir_sidecar(&kernel_unit);

    assert!(sidecar.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(sidecar.contains("supported_dispatch_kinds = [\"range\", \"tile\", \"batch\"]"));
    assert!(sidecar.contains("native_ir = \"host-simd\""));
    assert!(sidecar.contains("tensor_lowering = \"slice-backed-tensor-view\""));
    assert!(sidecar.contains("dispatch_lowering = \"threadpool-range-or-tile\""));
    assert!(sidecar.contains("host.slice-bounds"));
    assert!(sidecar.contains("primary = \"range\""));
    assert!(sidecar.contains("fallback = \"tile\""));
    assert!(sidecar.contains("binding_table = \"slice.input, slice.output\""));
    assert!(sidecar.contains("range = \"run_range\""));
    assert!(sidecar.contains("tile = \"run_tile\""));
    assert!(sidecar.contains("range_body = \"fn run_range"));
}

#[test]
fn network_urlsession_sidecar_emits_foundation_session_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "urlsession",
        "apple",
        "socket-io",
        "urlsession.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"foundation-url-request\""));
    assert!(sidecar.contains("transport_binding_model = \"session-task-packet\""));
    assert!(sidecar.contains("[lowering_capabilities]"));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("frontend_ir = \"nuis-yir.network\""));
    assert!(sidecar.contains("native_ir = \"foundation-url-request\""));
    assert!(sidecar.contains("transport_lowering = \"session-task-packet\""));
    assert!(sidecar.contains("dispatch_lowering = \"urlsession-task-submit\""));
    assert!(sidecar.contains("network.session-shape"));
    assert!(sidecar.contains("[session_shapes]"));
    assert!(sidecar.contains("request = \"http-client-session\""));
    assert!(sidecar.contains("response = \"completion-callback\""));
    assert!(sidecar.contains("streaming = \"delegate-push-stream\""));
    assert!(sidecar.contains("binding_table = \"session.handle, request.packet, response.slot\""));
    assert!(sidecar.contains("connect = \"open_session\""));
    assert!(sidecar.contains("send = \"submit_request\""));
    assert!(sidecar.contains("recv = \"on_response\""));
    assert!(sidecar.contains("finalize = \"finish_exchange\""));
}

#[test]
fn network_socket_abi_sidecar_emits_poll_reactor_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "socket-abi",
        "cross-vendor",
        "socket-io",
        "socket-abi.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"posix-socket\""));
    assert!(sidecar.contains("transport_binding_model = \"packet-poll-reactor\""));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("native_ir = \"posix-socket\""));
    assert!(sidecar.contains("transport_lowering = \"packet-poll-reactor\""));
    assert!(sidecar.contains("dispatch_lowering = \"poll-send-recv-submit\""));
    assert!(sidecar.contains("network.packet-shape"));
    assert!(sidecar.contains("request = \"socket-reactor-session\""));
    assert!(sidecar.contains("response = \"poll-ready-response\""));
    assert!(sidecar.contains("streaming = \"fd-edge-stream\""));
    assert!(sidecar.contains("binding_table = \"fd.handle, packet.buffer, ready.token\""));
    assert!(sidecar.contains("connect = \"open_fd_session\""));
    assert!(sidecar.contains("recv = \"poll_ready_response\""));
    assert!(sidecar.contains("finalize = \"finish_poll_exchange\""));
}

#[test]
fn network_winsock_sidecar_emits_iocp_templates() {
    let network_unit = sample_domain_unit(
        "network",
        "official.network",
        "winsock",
        "microsoft",
        "socket-io",
        "winsock.socket-io",
    );
    let sidecar = super::render_domain_build_unit_network_ir_sidecar(&network_unit);

    assert!(sidecar.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(sidecar.contains("transport_ir = \"winsock-overlapped\""));
    assert!(sidecar.contains("transport_binding_model = \"overlapped-packet-reactor\""));
    assert!(sidecar.contains("capability_owner = \"network-nustar\""));
    assert!(sidecar.contains("native_ir = \"winsock-overlapped\""));
    assert!(sidecar.contains("transport_lowering = \"overlapped-packet-reactor\""));
    assert!(sidecar.contains("dispatch_lowering = \"winsock-overlapped-submit\""));
    assert!(sidecar.contains("network.overlapped-shape"));
    assert!(sidecar.contains("request = \"overlapped-client-session\""));
    assert!(sidecar.contains("response = \"iocp-completion\""));
    assert!(sidecar.contains("streaming = \"completion-port-stream\""));
    assert!(
        sidecar.contains("binding_table = \"socket.handle, overlapped.packet, completion.port\"")
    );
    assert!(sidecar.contains("connect = \"connect_overlapped\""));
    assert!(sidecar.contains("recv = \"await_iocp_completion\""));
    assert!(sidecar.contains("finalize = \"finish_iocp_exchange\""));
}

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
            input_path: "/tmp/shader.ns".to_owned(),
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

#[test]
fn build_manifest_round_trips_cpu_target_metadata() {
    let dir = temp_dir("build_manifest_cpu_target");
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
        abi: "cpu.x86_64.sysv64".to_owned(),
        machine_arch: "x86_64".to_owned(),
        machine_os: "linux".to_owned(),
        object_format: "elf".to_owned(),
        calling_abi: "sysv64".to_owned(),
        clang_target: "x86_64-unknown-linux-gnu".to_owned(),
        isa_family: "x86_64".to_owned(),
        isa_features: vec!["x86-64".to_owned(), "sse2".to_owned()],
        cross_compile: true,
    };
    let manifest = super::write_build_manifest(
            &dir,
            &written,
            &BuildManifestContext {
                input_path: "/tmp/demo.ns".to_owned(),
                output_dir: dir.display().to_string(),
                loaded_nustar: vec!["official.cpu".to_owned()],
                compile_cache: Some(BuildManifestCacheInfo {
                    status: "miss".to_owned(),
                    key: "abc".to_owned(),
                    root: "/tmp/cache".to_owned(),
                }),
                project: Some(BuildManifestProjectInfo {
                    name: "demo".to_owned(),
                    abi_mode: "explicit".to_owned(),
                    abi_graph_summary: Some(
                        "graph\tmode=explicit\tdomains=cpu\tcpu_summary=present\tdata_summary=absent\tkernel_target=absent\tshader_target=absent\tnetwork_target=absent"
                            .to_owned(),
                    ),
                    abi_entries: vec![("cpu".to_owned(), cpu_target.abi.clone())],
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
                cpu_target: cpu_target.clone(),
            },
        )
        .unwrap();
    let manifest_text = std::fs::read_to_string(&manifest).unwrap();
    assert!(manifest_text.contains("[nuis_envelope]"));
    assert!(manifest_text.contains("path = "));
    assert!(manifest_text.contains("schema = \"nuis-executable-envelope-v1\""));
    assert!(manifest_text.contains("[nuis_artifact]"));
    assert!(manifest_text.contains("artifact_schema = \"nuis-compiled-artifact-v1\""));
    assert!(manifest_text.contains("domain_families = [\"cpu\"]"));
    assert!(manifest_text.contains("abi_graph = "));
    assert!(manifest_text.contains("graph\tmode=explicit"));
    assert!(manifest_text.contains("[[execution_contract]]"));
    assert!(manifest_text.contains("[[domain_build_unit]]"));
    assert!(manifest_text.contains("package_id = \"official.cpu\""));
    assert!(manifest_text.contains("contract_family = \"nustar.cpu\""));
    assert!(manifest_text.contains("packaging_role = \"host-binary\""));
    let envelope = parse_nuis_executable_envelope(PathBuf::from(&manifest).as_path()).unwrap();
    assert_eq!(envelope.schema, "nuis-executable-envelope-v1");
    assert_eq!(envelope.executable_kind, "native-cpu-llvm");
    assert_eq!(envelope.package_count, 1);
    assert_eq!(envelope.domain_families, vec!["cpu".to_owned()]);
    assert_eq!(envelope.contract_families, vec!["nustar.cpu".to_owned()]);
    let rendered_envelope = render_nuis_executable_envelope(&envelope);
    assert!(rendered_envelope.contains("envelope_schema = \"nuis-executable-envelope-v1\""));
    assert!(rendered_envelope.contains("executable_kind = \"native-cpu-llvm\""));
    let encoded_envelope = encode_nuis_executable_envelope_binary(&envelope).unwrap();
    let decoded_envelope = decode_nuis_executable_envelope_binary(&encoded_envelope).unwrap();
    assert_eq!(decoded_envelope, envelope);
    let compiled_artifact =
        parse_nuis_compiled_artifact(PathBuf::from(&dir).join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert_eq!(compiled_artifact.schema, "nuis-compiled-artifact-v1");
    assert_eq!(compiled_artifact.packaging_mode, "native-cpu-llvm");
    assert_eq!(compiled_artifact.binary_name, "demo.bin");
    assert_eq!(compiled_artifact.binary_blob, b"bin".to_vec());
    assert_eq!(compiled_artifact.build_manifest_source, manifest_text);
    assert_eq!(compiled_artifact.build_manifest_bytes, manifest_text.len());
    assert_eq!(compiled_artifact.envelope, envelope);
    assert_eq!(
        compiled_artifact.lifecycle.schema,
        "nuis-lifecycle-contract-v1"
    );
    assert_eq!(
        compiled_artifact.lifecycle.bootstrap_entry,
        "nuis.bootstrap.lifecycle.v1"
    );
    assert_eq!(compiled_artifact.lifecycle.export_surface.len(), 4);
    assert_eq!(
        compiled_artifact.lifecycle.runtime_capability_flags.len(),
        4
    );
    assert!(compiled_artifact
        .lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_tick_export_v1".to_owned()));
    assert!(compiled_artifact
        .lifecycle
        .runtime_capability_flags
        .contains(&"runtime.tick".to_owned()));
    assert!(manifest_text.contains("[nuis_lifecycle]"));
    assert!(manifest_text.contains("lifecycle_schema = \"nuis-lifecycle-contract-v1\""));
    assert!(manifest_text.contains("lifecycle_export_surface = ["));
    let unpacked_dir = dir.join("unpacked");
    fs::create_dir_all(&unpacked_dir).unwrap();
    let unpacked_envelope = unpacked_dir.join("nuis.executable.envelope.toml");
    let unpacked_artifact = unpacked_dir.join("nuis.compiled.artifact");
    let unpacked_binary = unpacked_dir.join("demo.bin");
    fs::write(&unpacked_binary, &compiled_artifact.binary_blob).unwrap();
    super::write_nuis_executable_envelope(&unpacked_envelope, &compiled_artifact.envelope).unwrap();
    let relocated_manifest = super::render_relocated_unpacked_build_manifest(
        &compiled_artifact,
        &unpacked_dir,
        &unpacked_envelope,
        &unpacked_artifact,
        &unpacked_binary,
    )
    .unwrap();
    assert!(relocated_manifest.contains(&format!("output_dir = \"{}\"", unpacked_dir.display())));
    assert!(relocated_manifest.contains(&format!(
        "artifact_path = \"{}\"",
        unpacked_artifact.display()
    )));
    assert!(!relocated_manifest.contains("plan_index = "));
    let encoded_artifact = encode_nuis_compiled_artifact_binary(&compiled_artifact).unwrap();
    let decoded_artifact = decode_nuis_compiled_artifact_binary(&encoded_artifact).unwrap();
    assert_eq!(decoded_artifact, compiled_artifact);
    let artifact_verify_report =
        verify_nuis_compiled_artifact(PathBuf::from(&dir).join("nuis.compiled.artifact").as_path())
            .unwrap();
    assert!(artifact_verify_report.lifecycle_contract_consistent);
    assert!(artifact_verify_report.lifecycle_runtime_capability_flags_consistent);
    let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
    assert!(std::path::Path::new(&report.envelope_path).exists());
    assert!(std::path::Path::new(&report.artifact_path).exists());
    assert_eq!(report.envelope_schema, "nuis-executable-envelope-v1");
    assert_eq!(report.envelope_package_count, 1);
    assert_eq!(report.artifact_schema, "nuis-compiled-artifact-v1");
    assert_eq!(report.artifact_binary_name, "demo.bin");
    assert_eq!(report.artifact_binary_bytes, 3);
    assert_eq!(report.lifecycle_schema, "nuis-lifecycle-contract-v1");
    assert_eq!(
        report.lifecycle_bootstrap_entry,
        "nuis.bootstrap.lifecycle.v1"
    );
    assert!(report.lifecycle_hook_count >= 7);
    assert!(report
        .lifecycle_hook_surface
        .contains(&"on_scheduler_tick".to_owned()));
    assert_eq!(report.lifecycle_export_count, 4);
    assert!(report
        .lifecycle_export_surface
        .contains(&"nuis_lifecycle_shutdown_export_v1".to_owned()));
    assert!(report
        .lifecycle_runtime_capability_flags
        .contains(&"runtime.shutdown".to_owned()));
    assert_eq!(report.execution_contracts_checked, 1);
    assert_eq!(report.domain_build_unit_count, 1);
    assert_eq!(report.heterogeneous_domain_count, 0);
    assert_eq!(report.domain_payload_blobs_checked, 0);
    assert_eq!(report.bridge_registry_path, None);
    assert_eq!(report.bridge_registry_units, 0);
    assert_eq!(report.bridge_registry_checked, 0);
    assert_eq!(report.host_bridge_plan_index_path, None);
    assert_eq!(report.host_bridge_plan_units, 0);
    assert_eq!(report.host_bridge_plan_checked, 0);
    assert_eq!(report.lowering_plan_index_path, None);
    assert_eq!(report.lowering_plan_units, 0);
    assert_eq!(report.lowering_plan_index_checked, 0);
    assert_eq!(report.doc_index_path, None);
    assert_eq!(report.doc_index_module_count, 0);
    assert_eq!(report.doc_index_documented_item_count, 0);
    assert_eq!(report.doc_index_checked, 0);
    assert_eq!(report.domain_build_units.len(), 1);
    assert_eq!(report.domain_build_units[0].domain_family, "cpu");
    assert_eq!(report.domain_build_units[0].artifact_stub_path, None);
    assert_eq!(report.domain_build_units[0].artifact_payload_path, None);
    assert_eq!(report.domain_build_units[0].artifact_bridge_stub_path, None);
    assert_eq!(
        report.domain_build_units[0].artifact_payload_blob_path,
        None
    );
    assert_eq!(
        report.domain_build_units[0].artifact_payload_blob_bytes,
        None
    );
    assert_eq!(report.domain_build_units[0].artifact_payload_format, None);
    assert_eq!(
        report.domain_build_units[0]
            .selected_lowering_target
            .as_deref(),
        Some("llvm")
    );
    assert_eq!(report.cpu_target_abi, cpu_target.abi);
    assert_eq!(report.cpu_target_machine_arch, cpu_target.machine_arch);
    assert_eq!(report.cpu_target_machine_os, cpu_target.machine_os);
    assert_eq!(report.cpu_target_object_format, cpu_target.object_format);
    assert_eq!(report.cpu_target_calling_abi, cpu_target.calling_abi);
    assert_eq!(report.cpu_target_clang, cpu_target.clang_target);
    assert!(report.cpu_target_cross);
    assert_eq!(report.project_metadata_checked, 0);
}

#[test]
fn build_manifest_tracks_heterogeneous_domain_build_units() {
    let dir = temp_dir("build_manifest_heterogeneous_units");
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
            input_path: "/tmp/hetero.ns".to_owned(),
            output_dir: dir.display().to_string(),
            loaded_nustar: vec![
                "official.cpu".to_owned(),
                "official.kernel".to_owned(),
                "official.network".to_owned(),
            ],
            compile_cache: None,
            project: Some(BuildManifestProjectInfo {
                name: "hetero".to_owned(),
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

    let report = verify_build_manifest(PathBuf::from(&manifest).as_path()).unwrap();
    assert_eq!(report.envelope_package_count, 3);
    assert_eq!(report.execution_contracts_checked, 3);
    assert_eq!(report.domain_build_unit_count, 3);
    assert_eq!(report.heterogeneous_domain_count, 2);
    assert_eq!(report.domain_payload_blobs_checked, 2);
    assert_eq!(report.domain_payload_blob_sections_checked, 10);
    assert_eq!(report.domain_payload_contract_sections_checked, 2);
    assert_eq!(report.domain_payload_lowering_plans_checked, 2);
    assert_eq!(report.domain_payload_backend_stubs_checked, 2);
    assert_eq!(report.domain_payload_bridge_plans_checked, 2);
    assert_eq!(report.domain_bridge_stubs_checked, 2);
    assert_eq!(report.bridge_registry_units, 2);
    assert_eq!(report.bridge_registry_checked, 1);
    assert_eq!(report.bridge_registry_entries_checked, 2);
    assert_eq!(report.host_bridge_plan_units, 2);
    assert_eq!(report.host_bridge_plan_checked, 1);
    assert_eq!(report.host_bridge_plan_entries_checked, 2);
    assert_eq!(report.lowering_plan_units, 2);
    assert_eq!(report.lowering_plan_index_checked, 1);
    assert_eq!(report.lowering_plan_entries_checked, 2);
    assert_eq!(report.hetero_calculate_plan_units, 2);
    assert_eq!(report.hetero_calculate_plan_checked, 1);
    assert_eq!(report.hetero_calculate_plan_entries_checked, 2);
    let kernel_payload = dir.join("nuis.domain.kernel.payload.toml");
    let kernel_bridge_stub = dir.join("nuis.domain.kernel.bridge.stub.txt");
    let kernel_payload_blob = dir.join("nuis.domain.kernel.payload.bin");
    let network_payload = dir.join("nuis.domain.network.payload.toml");
    let network_bridge_stub = dir.join("nuis.domain.network.bridge.stub.txt");
    let network_payload_blob = dir.join("nuis.domain.network.payload.bin");
    let bridge_registry = dir.join("nuis.bridge.registry.toml");
    let host_bridge_plan_index = dir.join("nuis.host-bridge.plan-index.toml");
    let lowering_plan_index = dir.join("nuis.lowering.plan-index.toml");
    let hetero_calculate_plan = dir.join("nuis.hetero-calculate.plan.toml");
    assert!(kernel_payload.exists());
    assert!(kernel_bridge_stub.exists());
    assert!(kernel_payload_blob.exists());
    assert!(network_payload.exists());
    assert!(network_bridge_stub.exists());
    assert!(network_payload_blob.exists());
    assert!(bridge_registry.exists());
    assert!(host_bridge_plan_index.exists());
    assert!(lowering_plan_index.exists());
    assert!(hetero_calculate_plan.exists());
    let kernel_payload_text = fs::read_to_string(&kernel_payload).unwrap();
    let kernel_bridge_stub_text = fs::read_to_string(&kernel_bridge_stub).unwrap();
    let network_payload_text = fs::read_to_string(&network_payload).unwrap();
    let network_bridge_stub_text = fs::read_to_string(&network_bridge_stub).unwrap();
    let bridge_registry_text = fs::read_to_string(&bridge_registry).unwrap();
    let host_bridge_plan_index_text = fs::read_to_string(&host_bridge_plan_index).unwrap();
    let lowering_plan_index_text = fs::read_to_string(&lowering_plan_index).unwrap();
    let hetero_calculate_plan_text = fs::read_to_string(&hetero_calculate_plan).unwrap();
    let bridge_registry_path_text = bridge_registry.display().to_string();
    let host_bridge_plan_index_path_text = host_bridge_plan_index.display().to_string();
    let lowering_plan_index_path_text = lowering_plan_index.display().to_string();
    let hetero_calculate_plan_path_text = hetero_calculate_plan.display().to_string();
    assert_eq!(
        report.bridge_registry_path.as_deref(),
        Some(bridge_registry_path_text.as_str())
    );
    assert_eq!(
        report.host_bridge_plan_index_path.as_deref(),
        Some(host_bridge_plan_index_path_text.as_str())
    );
    assert_eq!(
        report.lowering_plan_index_path.as_deref(),
        Some(lowering_plan_index_path_text.as_str())
    );
    assert_eq!(
        report.hetero_calculate_plan_path.as_deref(),
        Some(hetero_calculate_plan_path_text.as_str())
    );
    assert!(bridge_registry_text.contains("schema = \"nuis-bridge-registry-v1\""));
    assert!(bridge_registry_text.contains("bridge_count = 2"));
    assert!(bridge_registry_text.contains("[[bridge]]"));
    assert!(bridge_registry_text.contains("domain_family = \"kernel\""));
    assert!(bridge_registry_text.contains("domain_family = \"network\""));
    assert!(bridge_registry_text.contains("backend_family = \"coreml\""));
    assert!(bridge_registry_text.contains("vendor = \"apple\""));
    assert!(bridge_registry_text.contains("device_class = \"apple-ane\""));
    assert!(bridge_registry_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(bridge_registry_text.contains("backend_family = \"urlsession\""));
    assert!(bridge_registry_text.contains("device_class = \"socket-io\""));
    assert!(bridge_registry_text.contains("selected_lowering_target = \"urlsession.socket-io\""));
    assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(bridge_registry_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\""));
    assert!(bridge_registry_text.contains("host_ffi_policy = \"signature-whitelist-required\""));
    assert!(bridge_registry_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(bridge_registry_text
        .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
    assert!(bridge_registry_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(bridge_registry_text.contains("bridge_stub_path = "));
    assert!(host_bridge_plan_index_text.contains("schema = \"nuis-host-bridge-plan-index-v1\""));
    assert!(host_bridge_plan_index_text.contains("plan_count = 2"));
    assert!(host_bridge_plan_index_text.contains("[[plan]]"));
    assert!(host_bridge_plan_index_text.contains("domain_family = \"kernel\""));
    assert!(host_bridge_plan_index_text.contains("domain_family = \"network\""));
    assert!(host_bridge_plan_index_text.contains("backend_family = \"coreml\""));
    assert!(host_bridge_plan_index_text.contains("vendor = \"apple\""));
    assert!(host_bridge_plan_index_text.contains("device_class = \"apple-ane\""));
    assert!(host_bridge_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(host_bridge_plan_index_text.contains("backend_family = \"urlsession\""));
    assert!(host_bridge_plan_index_text.contains("device_class = \"socket-io\""));
    assert!(
        host_bridge_plan_index_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(host_bridge_plan_index_text.contains("host_ffi_bridge = \"cffi.network.dispatch.v1\""));
    assert!(
        host_bridge_plan_index_text.contains("host_ffi_policy = \"signature-whitelist-required\"")
    );
    assert!(host_bridge_plan_index_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(host_bridge_plan_index_text
        .contains("host_ffi_symbol = \"nuis_network_urlsession_socket_io_dispatch_v1\""));
    assert!(host_bridge_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(host_bridge_plan_index_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(lowering_plan_index_text.contains("schema = \"nuis-domain-lowering-plan-index-v1\""));
    assert!(lowering_plan_index_text.contains("plan_count = 2"));
    assert!(lowering_plan_index_text.contains("[[lowering_plan]]"));
    assert!(lowering_plan_index_text.contains("domain_family = \"kernel\""));
    assert!(lowering_plan_index_text.contains("domain_family = \"network\""));
    assert!(lowering_plan_index_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(
        lowering_plan_index_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(lowering_plan_index_text.contains("execution_route = \"ane-graph-execution\""));
    assert!(lowering_plan_index_text.contains("execution_route = \"foundation-session-reactor\""));
    assert!(lowering_plan_index_text.contains("kernel.profile.tensor-reduce.v1"));
    assert!(lowering_plan_index_text.contains("kernel.profile.result-buffer.v1"));
    assert!(lowering_plan_index_text.contains(
        "registered_lane_groups = [\"setup\", \"memory\", \"compute\", \"shape\", \"reduce\", \"select\", \"debug\"]"
    ));
    assert!(lowering_plan_index_text
        .contains("symbol_namespace = \"nuis::domain::kernel::coreml_apple_ane\""));
    assert!(
        lowering_plan_index_text.contains("debug_anchor = \"nuis.debug.kernel.coreml_apple_ane\"")
    );
    assert!(
        lowering_plan_index_text.contains("linkage_anchor = \"nuis.link.kernel.coreml_apple_ane\"")
    );
    assert!(lowering_plan_index_text.contains(
        "source_map_scope = \"domain:kernel/package:official.kernel/target:coreml.apple-ane\""
    ));
    assert!(lowering_plan_index_text.contains("host_ffi_bridge = \"cffi.kernel.dispatch.v1\""));
    assert!(lowering_plan_index_text.contains("host_ffi_policy = \"signature-whitelist-required\""));
    assert!(lowering_plan_index_text
        .contains("host_ffi_symbol = \"nuis_kernel_coreml_apple_ane_dispatch_v1\""));
    assert!(lowering_plan_index_text.contains(
        "host_ffi_signature = \"fn(payload: ptr, payload_len: usize, bridge_state: ptr) -> i64\""
    ));
    assert!(lowering_plan_index_text.contains("host_ffi_signature_hash = \"0x"));
    assert!(lowering_plan_index_text
        .contains("symbol_namespace = \"nuis::domain::network::urlsession_socket_io\""));
    assert!(lowering_plan_index_text
        .contains("debug_anchor = \"nuis.debug.network.urlsession_socket_io\""));
    assert!(lowering_plan_index_text.contains("ir_sidecar_path = "));
    assert!(lowering_plan_index_text.contains("payload_blob_path = "));
    assert!(lowering_plan_index_text.contains("bridge_stub_path = "));
    let manifest_text = fs::read_to_string(&manifest).unwrap();
    assert!(manifest_text.contains("[hetero_calculate_plan]"));
    assert!(manifest_text.contains("hetero_calculate_plan_path = "));
    assert!(manifest_text
        .contains("hetero_calculate_plan_schema = \"nuis-hetero-calculate-link-plan-v1\""));
    assert!(manifest_text.contains("hetero_calculate_plan_units = 2"));
    assert!(manifest_text.contains("hetero_calculate_plan_inline = "));
    assert!(hetero_calculate_plan_text.contains("schema = \"nuis-hetero-calculate-link-plan-v1\""));
    assert!(hetero_calculate_plan_text.contains("mode = \"heterogeneous-static-lifecycle\""));
    assert!(hetero_calculate_plan_text.contains("static_link = true"));
    assert!(hetero_calculate_plan_text.contains("lifecycle_driven = true"));
    assert!(hetero_calculate_plan_text.contains("[validation]"));
    assert!(hetero_calculate_plan_text.contains("valid = true"));
    assert!(hetero_calculate_plan_text.contains("[[node]]"));
    assert!(hetero_calculate_plan_text.contains("timestamp = \"t0001.kernel\""));
    assert!(hetero_calculate_plan_text.contains("timestamp = \"t0002.network\""));
    assert!(hetero_calculate_plan_text.contains("wait_on = [\"t0001.kernel\"]"));
    assert!(hetero_calculate_plan_text.contains("[[data_segment]]"));
    assert!(hetero_calculate_plan_text.contains("order_key = \"data:0001:kernel\""));
    assert!(hetero_calculate_plan_text.contains("order_key = \"data:0002:network\""));
    let kernel_blob =
        super::decode_domain_build_unit_payload_blob(&fs::read(&kernel_payload_blob).unwrap())
            .unwrap();
    let network_blob =
        super::decode_domain_build_unit_payload_blob(&fs::read(&network_payload_blob).unwrap())
            .unwrap();
    let kernel_lowering_plan = super::render_domain_build_unit_lowering_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_backend_stub = super::render_domain_build_unit_backend_stub(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_ir_sidecar = super::render_domain_build_unit_kernel_ir_sidecar(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let kernel_bridge_plan = super::render_domain_build_unit_bridge_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "kernel")
            .unwrap(),
    );
    let network_lowering_plan = super::render_domain_build_unit_lowering_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_backend_stub = super::render_domain_build_unit_backend_stub(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_ir_sidecar = super::render_domain_build_unit_network_ir_sidecar(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    let network_bridge_plan = super::render_domain_build_unit_bridge_plan(
        report
            .domain_build_units
            .iter()
            .find(|unit| unit.domain_family == "network")
            .unwrap(),
    );
    assert!(kernel_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
    assert!(kernel_payload_text.contains("support_surface = ["));
    assert!(kernel_payload_text.contains("default_lanes = ["));
    assert!(kernel_payload_text.contains("resource_families = ["));
    assert!(kernel_payload_text.contains("lowering_targets = ["));
    assert!(kernel_payload_text.contains("ops = ["));
    assert!(network_payload_text.contains("schema = \"nuis-domain-build-payload-v1\""));
    assert!(network_payload_text.contains("host_ffi_surface = ["));
    assert!(network_payload_text.contains("clock_bridge_default = "));
    assert_eq!(kernel_blob.domain_family, "kernel");
    assert_eq!(kernel_blob.package_id, "official.kernel");
    assert_eq!(kernel_blob.backend_family.as_deref(), Some("coreml"));
    assert_eq!(
        kernel_blob.selected_lowering_target.as_deref(),
        Some("coreml.apple-ane")
    );
    assert_eq!(kernel_blob.contract_family, "nustar.kernel");
    assert_eq!(kernel_blob.packaging_role, "hetero-contract");
    assert_eq!(kernel_blob.payload_kind, "contract-sidecar");
    assert_eq!(kernel_blob.payload_format, "toml");
    assert_eq!(kernel_blob.sections.len(), 5);
    assert_eq!(kernel_blob.sections[0].name, "contract_toml");
    assert_eq!(
        kernel_blob.sections[0].bytes,
        kernel_payload_text.as_bytes()
    );
    assert_eq!(kernel_blob.sections[1].name, "lowering_plan");
    assert_eq!(
        kernel_blob.sections[1].bytes,
        kernel_lowering_plan.as_bytes()
    );
    assert_eq!(kernel_blob.sections[2].name, "backend_stub");
    assert_eq!(
        kernel_blob.sections[2].bytes,
        kernel_backend_stub.as_bytes()
    );
    assert_eq!(kernel_blob.sections[3].name, "bridge_plan");
    assert_eq!(kernel_blob.sections[3].bytes, kernel_bridge_plan.as_bytes());
    assert_eq!(kernel_blob.sections[4].name, "kernel_ir_sidecar");
    assert_eq!(kernel_blob.sections[4].bytes, kernel_ir_sidecar.as_bytes());
    let kernel_backend_text = std::str::from_utf8(&kernel_blob.sections[2].bytes).unwrap();
    let kernel_bridge_text = std::str::from_utf8(&kernel_blob.sections[3].bytes).unwrap();
    let kernel_sidecar_text = std::str::from_utf8(&kernel_blob.sections[4].bytes).unwrap();
    assert!(kernel_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
    assert!(kernel_bridge_stub_text.contains("vendor = \"apple\""));
    assert!(kernel_bridge_stub_text.contains("device_class = \"apple-ane\""));
    assert!(kernel_bridge_stub_text.contains("selected_lowering_target = \"coreml.apple-ane\""));
    assert!(kernel_bridge_stub_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(kernel_bridge_stub_text.contains("host_ffi_surface = \"buffer,queue,fence\""));
    assert!(kernel_bridge_stub_text.contains("handle_family = \"kernel.buffer,kernel.dispatch\""));
    assert!(kernel_bridge_stub_text.contains(
        "phase_submit_inputs = [\"dispatch.handle\", \"bound.buffer.table\", \"queue.slot\"]"
    ));
    assert!(kernel_bridge_stub_text.contains("phase_wait_wake = \"completion-fence\""));
    assert!(kernel_bridge_stub_text.contains("bridge_plan_begin = true"));
    assert!(kernel_bridge_stub_text.contains("bridge_plan_end = true"));
    assert!(kernel_bridge_stub_text.contains("phase_submit = \"queue-dispatch-submit\""));
    assert!(kernel_backend_text.contains("stub_kind = \"kernel-dispatch\""));
    assert!(kernel_backend_text.contains("dispatch_shape = \"grid-launch\""));
    assert!(kernel_backend_text.contains("memory_binding = \"buffer-table\""));
    assert!(kernel_backend_text.contains("completion_model = \"device-fence\""));
    assert!(kernel_backend_text.contains("scheduler_binding = \"hetero-submit-bridge\""));
    assert!(kernel_backend_text.contains("backend_profile = \"coreml.apple-ane\""));
    assert!(kernel_backend_text.contains("execution_route = \"ane-graph-execution\""));
    assert!(kernel_backend_text.contains("submission_adapter = \"coreml-graph-submit\""));
    assert!(kernel_backend_text.contains("wake_adapter = \"coreml-completion-callback\""));
    assert!(
        kernel_backend_text.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
    );
    assert!(kernel_backend_text.contains("kernel_ir = \"coreml-program\""));
    assert!(kernel_backend_text.contains("kernel_entry_model = \"mlmodelc-function\""));
    assert!(kernel_backend_text.contains("queue_binding_model = \"ane-submission-service\""));
    assert!(kernel_backend_text.contains("resource_binding_model = \"tensor-argument-table\""));
    assert!(kernel_sidecar_text.contains("schema = \"nuis-kernel-ir-sidecar-v1\""));
    assert!(
        kernel_sidecar_text.contains("supported_dispatch_kinds = [\"graph\", \"batch\", \"tile\"]")
    );
    assert!(kernel_sidecar_text.contains("graph = \"infer_main\""));
    assert!(kernel_backend_text.contains("bind_phase = \"buffer-and-argument-bind\""));
    assert!(kernel_backend_text.contains("launch_phase = \"queue-dispatch-submit\""));
    assert!(kernel_backend_text.contains("wait_phase = \"fence-await-or-poll\""));
    assert!(kernel_backend_text.contains("finalize_phase = \"result-commit-and-release\""));
    assert!(kernel_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
    assert!(kernel_bridge_text.contains("phase_bind = \"buffer-and-argument-bind\""));
    assert!(kernel_bridge_text.contains("phase_submit = \"queue-dispatch-submit\""));
    assert!(kernel_bridge_text.contains("phase_wait = \"fence-await-or-poll\""));
    assert!(kernel_bridge_text.contains("phase_finalize = \"result-commit-and-release\""));
    assert_eq!(network_blob.domain_family, "network");
    assert_eq!(network_blob.package_id, "official.network");
    assert_eq!(network_blob.backend_family.as_deref(), Some("urlsession"));
    assert_eq!(
        network_blob.selected_lowering_target.as_deref(),
        Some("urlsession.socket-io")
    );
    assert_eq!(network_blob.contract_family, "nustar.network");
    assert_eq!(network_blob.packaging_role, "hetero-contract");
    assert_eq!(network_blob.payload_kind, "contract-sidecar");
    assert_eq!(network_blob.payload_format, "toml");
    assert_eq!(network_blob.sections.len(), 5);
    assert_eq!(network_blob.sections[0].name, "contract_toml");
    assert_eq!(
        network_blob.sections[0].bytes,
        network_payload_text.as_bytes()
    );
    assert_eq!(network_blob.sections[1].name, "lowering_plan");
    assert_eq!(
        network_blob.sections[1].bytes,
        network_lowering_plan.as_bytes()
    );
    assert_eq!(network_blob.sections[2].name, "backend_stub");
    assert_eq!(
        network_blob.sections[2].bytes,
        network_backend_stub.as_bytes()
    );
    assert_eq!(network_blob.sections[3].name, "bridge_plan");
    assert_eq!(
        network_blob.sections[3].bytes,
        network_bridge_plan.as_bytes()
    );
    assert_eq!(network_blob.sections[4].name, "network_ir_sidecar");
    assert_eq!(
        network_blob.sections[4].bytes,
        network_ir_sidecar.as_bytes()
    );
    let network_backend_text = std::str::from_utf8(&network_blob.sections[2].bytes).unwrap();
    let network_bridge_text = std::str::from_utf8(&network_blob.sections[3].bytes).unwrap();
    let network_sidecar_text = std::str::from_utf8(&network_blob.sections[4].bytes).unwrap();
    assert!(network_bridge_stub_text.contains("schema = \"nuis-host-bridge-spec-v1\""));
    assert!(network_bridge_stub_text.contains("vendor = \"apple\""));
    assert!(network_bridge_stub_text.contains("device_class = \"socket-io\""));
    assert!(
        network_bridge_stub_text.contains("selected_lowering_target = \"urlsession.socket-io\"")
    );
    assert!(network_bridge_stub_text
        .contains("phase_order = [\"bind\", \"submit\", \"wait\", \"finalize\"]"));
    assert!(network_bridge_stub_text.contains("host_ffi_surface = \"socket,urlsession\""));
    assert!(
        network_bridge_stub_text.contains("handle_family = \"network.request,network.response\"")
    );
    assert!(network_bridge_stub_text.contains(
        "phase_submit_inputs = [\"session.handle\", \"request.handle\", \"request.packet\"]"
    ));
    assert!(network_bridge_stub_text.contains("phase_wait_wake = \"io-ready\""));
    assert!(network_bridge_stub_text.contains("bridge_plan_begin = true"));
    assert!(network_bridge_stub_text.contains("bridge_plan_end = true"));
    assert!(network_bridge_stub_text.contains("phase_submit = \"packet-write-dispatch\""));
    assert!(network_backend_text.contains("stub_kind = \"network-host-bridge\""));
    assert!(network_backend_text.contains("transport_model = \"client-session\""));
    assert!(network_backend_text.contains("request_shape = \"packetized-exchange\""));
    assert!(network_backend_text.contains("response_shape = \"completion-callback\""));
    assert!(network_backend_text.contains("scheduler_binding = \"network-poll-bridge\""));
    assert!(network_backend_text.contains("backend_profile = \"urlsession.socket-io\""));
    assert!(network_backend_text.contains("execution_route = \"foundation-session-reactor\""));
    assert!(network_backend_text.contains("submission_adapter = \"urlsession-task-submit\""));
    assert!(network_backend_text.contains("wake_adapter = \"urlsession-callback\""));
    assert!(network_backend_text.contains("transport_ir = \"foundation-url-request\""));
    assert!(network_backend_text.contains("transport_entry_model = \"urlsession-task\""));
    assert!(network_backend_text.contains("socket_binding_model = \"session-owned-socket\""));
    assert!(network_backend_text.contains("completion_binding_model = \"delegate-callback\""));
    assert!(network_backend_text.contains("connect_phase = \"socket-bind-or-session-open\""));
    assert!(network_backend_text.contains("send_phase = \"packet-write-dispatch\""));
    assert!(network_backend_text.contains("recv_phase = \"callback-or-read-ready\""));
    assert!(network_backend_text.contains("finalize_phase = \"response-commit-and-wake\""));
    assert!(network_bridge_text.contains("bridge_kind = \"managed-lifecycle-bridge\""));
    assert!(network_bridge_text.contains("phase_bind = \"socket-bind-or-session-open\""));
    assert!(network_bridge_text.contains("phase_submit = \"packet-write-dispatch\""));
    assert!(network_bridge_text.contains("phase_wait = \"callback-or-read-ready\""));
    assert!(network_bridge_text.contains("phase_finalize = \"response-commit-and-wake\""));
    assert!(network_sidecar_text.contains("schema = \"nuis-network-ir-sidecar-v1\""));
    assert!(network_sidecar_text.contains("request = \"http-client-session\""));
    assert!(network_sidecar_text.contains("response = \"completion-callback\""));
    assert!(network_sidecar_text.contains("streaming = \"delegate-push-stream\""));
    assert!(network_sidecar_text.contains("connect = \"open_session\""));
    assert!(network_sidecar_text.contains("finalize = \"finish_exchange\""));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "cpu"
            && unit.packaging_role == "host-binary"
            && unit.artifact_stub_path.is_none()
            && unit.selected_lowering_target.as_deref() == Some("llvm")));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "kernel"
            && unit
                .artifact_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.artifact.toml"))
            && unit
                .artifact_payload_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.toml"))
            && unit
                .artifact_bridge_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.bridge.stub.txt"))
            && unit
                .artifact_ir_sidecar_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.lowering.ir.txt"))
            && unit
                .artifact_payload_blob_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.kernel.payload.bin"))
            && unit
                .artifact_payload_blob_bytes
                .is_some_and(|bytes| bytes > 0)
            && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
            && unit.backend_family.as_deref() == Some("coreml")
            && unit.selected_lowering_target.as_deref() == Some("coreml.apple-ane")));
    assert!(report
        .domain_build_units
        .iter()
        .any(|unit| unit.domain_family == "network"
            && unit
                .artifact_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.artifact.toml"))
            && unit
                .artifact_payload_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.payload.toml"))
            && unit
                .artifact_bridge_stub_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.bridge.stub.txt"))
            && unit
                .artifact_ir_sidecar_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.lowering.ir.txt"))
            && unit
                .artifact_payload_blob_path
                .as_deref()
                .is_some_and(|path| path.ends_with("nuis.domain.network.payload.bin"))
            && unit
                .artifact_payload_blob_bytes
                .is_some_and(|bytes| bytes > 0)
            && unit.artifact_payload_format.as_deref() == Some("ndpb-v2")
            && unit.backend_family.as_deref() == Some("urlsession")
            && unit.selected_lowering_target.as_deref() == Some("urlsession.socket-io")));
}

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
            input_path: "/tmp/hetero_artifact.ns".to_owned(),
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

#[test]
fn c_shim_source_includes_native_cli_runtime_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_argv_count".to_owned(),
                params: Vec::new(),
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_ends_with".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "suffix_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_cwd_handle".to_owned(),
                params: Vec::new(),
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_monotonic_time_ns".to_owned(),
                params: Vec::new(),
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_bool_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_parse_header_line".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "expected_name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_find_header_value".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "expected_name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_find_status_line_reason".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_parse_http_response_summary".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_parse_http_request_summary".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_parse_http_roundtrip_summary".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "request_buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "request_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "request_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "response_buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "response_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "response_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_bool_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_bool_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_bool_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_from".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_equals".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "expected_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_starts_with".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "prefix_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_equals".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "expected_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_starts_with".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "prefix_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_equals".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "expected_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_starts_with".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "prefix_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_deserialize_text_contains".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "needle_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_find_byte".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "needle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_fill_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "value".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_copy_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_compare_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "lhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_find_text".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "needle_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_find_text".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "needle_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_find_text".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "needle_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_find_line_end".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_buffer_trim_line_end".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_fill_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "value".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_copy_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_compare_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "lhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_fill_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "value".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_copy_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_compare_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "lhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_fill_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "value".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_copy_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_compare_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "lhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_fill_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "value".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_copy_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "src_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_compare_bytes".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "lhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "lhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_offset".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "rhs_len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("int main(int argc, char** argv)"));
    assert!(shim.contains("nuis_argc = argc;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_surface_slots = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_bootstrap_entry_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_tick_once_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_shutdown_v1(int64_t status)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_yalivia_rpc_hook_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_bridge_bind_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_scheduler_tick_v1(int64_t tick)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_task_poll_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_result_commit_v1(int64_t status)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_summary_flush_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_sample_network_bridge_progress_v1(void)"));
    assert!(
        shim.contains("static int64_t nuis_lifecycle_sample_hetero_submission_progress_v1(void)")
    );
    assert!(shim.contains("static int64_t nuis_lifecycle_on_network_bridge_progress_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_hetero_submission_progress_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_managed_rpc_v1(void)"));
    assert!(shim.contains("static int64_t nuis_lifecycle_on_shutdown_prepare_v1(int64_t status)"));
    assert!(shim.contains("int64_t nuis_lifecycle_bootstrap_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_bootstrap_entry_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_tick_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_tick_once_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_shutdown_export_v1(int64_t status) {"));
    assert!(shim.contains("return nuis_lifecycle_shutdown_v1(status);"));
    assert!(shim.contains("int64_t nuis_lifecycle_yalivia_rpc_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_yalivia_rpc_hook_v1();"));
    assert!(shim.contains("int64_t nuis_lifecycle_network_bridge_progress_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_state.network_bridge_progress_count;"));
    assert!(shim.contains("int64_t nuis_lifecycle_hetero_submission_progress_export_v1(void) {"));
    assert!(shim.contains("return nuis_lifecycle_state.hetero_submission_progress_count;"));
    assert!(shim.contains("if (nuis_lifecycle_bootstrap_entry_v1() != 0) {"));
    assert!(shim.contains("(void)nuis_lifecycle_on_bridge_bind_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_managed_rpc_v1();"));
    assert!(shim.contains("int64_t entry_status = nuis_yir_entry();"));
    assert!(shim.contains("(void)nuis_lifecycle_tick_once_v1();"));
    assert!(shim
        .contains("(void)nuis_lifecycle_on_scheduler_tick_v1(nuis_lifecycle_state.tick_count);"));
    assert!(shim.contains("(void)nuis_lifecycle_on_task_poll_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_network_bridge_progress_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_hetero_submission_progress_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_result_commit_v1(status);"));
    assert!(shim.contains("(void)nuis_lifecycle_on_summary_flush_v1();"));
    assert!(shim.contains("(void)nuis_lifecycle_on_shutdown_prepare_v1(status);"));
    assert!(shim.contains("return (int)nuis_lifecycle_shutdown_v1(entry_status);"));
    assert!(shim.contains("return nuis_host_argv_count();"));
    assert!(shim.contains("return nuis_host_cwd_handle();"));
    assert!(shim.contains("return nuis_host_monotonic_time_ns();"));
}

#[test]
fn lifecycle_contract_expands_export_surface_for_network_and_hetero_domains() {
    let envelope = NuisExecutableEnvelope {
        schema: "nuis-executable-envelope-v1".to_owned(),
        executable_kind: "native-cpu-llvm".to_owned(),
        package_count: 3,
        domain_families: vec!["cpu".to_owned(), "network".to_owned(), "kernel".to_owned()],
        contract_families: vec![
            "nustar.cpu".to_owned(),
            "nustar.network".to_owned(),
            "nustar.kernel".to_owned(),
        ],
        function_kind: "function-node".to_owned(),
        graph_kind: "function-graph".to_owned(),
        default_time_mode: "host-monotonic".to_owned(),
    };

    let lifecycle = build_nuis_lifecycle_contract(&envelope, "native-cpu-llvm");
    assert!(lifecycle
        .hook_surface
        .contains(&"on_network_bridge_progress".to_owned()));
    assert!(lifecycle
        .hook_surface
        .contains(&"on_hetero_submission_progress".to_owned()));
    assert!(lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_network_bridge_progress_export_v1".to_owned()));
    assert!(lifecycle
        .export_surface
        .contains(&"nuis_lifecycle_hetero_submission_progress_export_v1".to_owned()));
    assert_eq!(lifecycle.export_surface.len(), 6);
    assert!(lifecycle
        .runtime_capability_flags
        .contains(&"runtime.progress.network".to_owned()));
    assert!(lifecycle
        .runtime_capability_flags
        .contains(&"runtime.progress.hetero".to_owned()));
}

#[test]
fn c_shim_source_enables_hetero_lifecycle_surface_for_shader_modules() {
    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "shader".to_owned(),
        unit: "SurfaceShader".to_owned(),
        externs: Vec::new(),
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };

    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 0;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_enabled = 1;"));
    assert!(shim.contains("static int64_t nuis_lifecycle_hetero_surface_slots = 1;"));
    assert!(shim.contains("return nuis_lifecycle_hetero_surface_slots;"));
}

#[test]
fn c_shim_source_includes_native_env_path_and_stat_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_env_has".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "key_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_basename".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_filename".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_basename_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_filename_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_parent_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_stem_matches".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "name_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_parent".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_has_parent".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_basename_only".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_depth".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_empty".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_dot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_dotdot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_relative".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_root".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_stem".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_extension".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_has_extension".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_matches_extension".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "ext_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_extension_is".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "ext_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_starts_with_dot".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_ends_with_slash".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_is_hidden".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_stat_mode".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("return nuis_host_env_has(key_handle);"));
    assert!(shim.contains("return nuis_host_path_is_empty(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_dot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_dotdot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_relative(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_root(path_handle);"));
    assert!(shim.contains("return nuis_host_path_basename(path_handle);"));
    assert!(shim.contains("return nuis_host_path_filename(path_handle);"));
    assert!(shim.contains("return nuis_host_path_basename_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_filename_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_parent_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_stem_matches(path_handle, name_handle);"));
    assert!(shim.contains("return nuis_host_path_parent(path_handle);"));
    assert!(shim.contains("return nuis_host_path_has_parent(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_basename_only(path_handle);"));
    assert!(shim.contains("return nuis_host_path_depth(path_handle);"));
    assert!(shim.contains("return nuis_host_path_stem(path_handle);"));
    assert!(shim.contains("return nuis_host_path_extension(path_handle);"));
    assert!(shim.contains("return nuis_host_path_has_extension(path_handle);"));
    assert!(shim.contains("return nuis_host_path_matches_extension(path_handle, ext_handle);"));
    assert!(shim.contains("return nuis_host_path_extension_is(path_handle, ext_handle);"));
    assert!(shim.contains("return nuis_host_path_starts_with_dot(path_handle);"));
    assert!(shim.contains("return nuis_host_path_ends_with_slash(path_handle);"));
    assert!(shim.contains("return nuis_host_path_is_hidden(path_handle);"));
    assert!(shim.contains("return nuis_host_stat_mode(path_handle);"));
}

#[test]
fn c_shim_source_includes_native_file_stdin_and_tty_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_file_open".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "path_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "flags".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_file_write".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "file_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "text_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_stdin_read".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "buffer_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "len".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_tty_width".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "fd".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("return nuis_host_file_open(path_handle, flags);"));
    assert!(shim.contains("return nuis_host_file_write(file_handle, text_handle);"));
    assert!(shim.contains("return nuis_host_stdin_read(buffer_handle, len);"));
    assert!(shim.contains("return nuis_host_tty_width(fd);"));
}

#[test]
fn c_shim_source_includes_network_control_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_connect_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "remote_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "connect_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_accept_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_open_tcp_listener".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_bind_udp_datagram".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_accept_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "listener_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "read_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "write_timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_close".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_send_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "send_window".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_recv_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_recv_http_status_owned".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_send_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "send_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "remote_port".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_network_recv_probe".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "stream_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "recv_window".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "local_port".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };

    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_lifecycle_network_enabled = 1;"));
    assert!(shim.contains("return nuis_host_network_fd_len;"));
    assert!(shim.contains(
        "return nuis_host_network_connect_probe(local_port, remote_port, connect_timeout_ms);"
    ));
    assert!(shim.contains(
        "return nuis_host_network_accept_probe(local_port, read_timeout_ms, write_timeout_ms);"
    ));
    assert!(shim.contains(
            "return nuis_host_network_open_tcp_listener(local_port, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains(
            "return nuis_host_network_bind_udp_datagram(local_port, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains(
            "return nuis_host_network_accept_owned(listener_handle, read_timeout_ms, write_timeout_ms);"
        ));
    assert!(shim.contains("return nuis_host_network_close(handle);"));
    assert!(
        shim.contains("return nuis_host_network_send_owned(handle, stream_window, send_window);")
    );
    assert!(
        shim.contains("return nuis_host_network_recv_owned(handle, stream_window, recv_window);")
    );
    assert!(shim.contains(
        "return nuis_host_network_recv_http_status_owned(handle, stream_window, recv_window);"
    ));
    assert!(shim
        .contains("return nuis_host_network_send_probe(stream_window, send_window, remote_port);"));
    assert!(shim
        .contains("return nuis_host_network_recv_probe(stream_window, recv_window, local_port);"));
}

#[test]
fn c_shim_source_includes_native_directory_temp_and_process_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_dir_open".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_dir_create".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_dir_remove".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_rename".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_copy".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "src_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "dst_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_path_remove".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "path_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_temp_file_handle".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "prefix_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_command_spawn".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_command_spawn_in".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "cwd_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("return nuis_host_dir_open(path_handle);"));
    assert!(shim.contains("return nuis_host_dir_create(path_handle);"));
    assert!(shim.contains("return nuis_host_dir_remove(path_handle);"));
    assert!(shim.contains("return nuis_host_path_rename(src_handle, dst_handle);"));
    assert!(shim.contains("return nuis_host_path_copy(src_handle, dst_handle);"));
    assert!(shim.contains("return nuis_host_path_remove(path_handle);"));
    assert!(shim.contains("return nuis_host_temp_file_handle(prefix_handle);"));
    assert!(shim.contains("return nuis_host_command_spawn(program_handle, argv_handle);"));
    assert!(shim.contains(
        "return nuis_host_command_spawn_in(program_handle, argv_handle, cwd_handle, timeout_ms);"
    ));
    assert!(shim.contains("static char* nuis_host_build_shell_command("));
    assert!(shim.contains("env %s %s %s"));
    assert!(shim.contains("static int64_t nuis_host_command_spawn_in("));
}

#[test]
fn c_shim_source_includes_native_command_and_subprocess_exit_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_subprocess_spawn_in".to_owned(),
                params: vec![
                    nuis_semantics::model::AstParam {
                        name: "program_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "argv_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "env_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "cwd_handle".to_owned(),
                        ty: i64_ty(),
                    },
                    nuis_semantics::model::AstParam {
                        name: "timeout_ms".to_owned(),
                        ty: i64_ty(),
                    },
                ],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_command_wait_exit".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "command_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
            AstExternFunction {
                visibility: AstVisibility::Private,
                abi: "c".to_owned(),
                interface: None,
                name: "host_subprocess_join_exit".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "process_handle".to_owned(),
                    ty: i64_ty(),
                }],
                return_type: i64_ty(),
                host_symbol: None,
            },
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_host_command_wait_exit("));
    assert!(shim.contains("static int64_t nuis_host_subprocess_join_exit("));
    assert!(shim.contains("static int64_t nuis_host_subprocess_spawn_in("));
    assert!(shim.contains(
            "return nuis_host_subprocess_spawn_in(program_handle, argv_handle, env_handle, cwd_handle, timeout_ms);"
        ));
    assert!(shim.contains("return nuis_host_command_wait_exit(command_handle);"));
    assert!(shim.contains("return nuis_host_subprocess_join_exit(process_handle);"));
}

#[test]
fn c_shim_source_includes_native_text_concat_hook() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "host_text_concat".to_owned(),
            params: vec![
                nuis_semantics::model::AstParam {
                    name: "lhs_handle".to_owned(),
                    ty: i64_ty(),
                },
                nuis_semantics::model::AstParam {
                    name: "rhs_handle".to_owned(),
                    ty: i64_ty(),
                },
            ],
            return_type: i64_ty(),
            host_symbol: None,
        }],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_host_text_concat("));
    assert!(shim.contains("return nuis_host_text_concat(lhs_handle, rhs_handle);"));
}

#[test]
fn c_shim_source_includes_native_serialization_hooks() {
    fn i64_ty() -> AstTypeRef {
        AstTypeRef {
            name: "i64".to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    fn host_extern(name: &str, params: &[&str]) -> AstExternFunction {
        AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: name.to_owned(),
            params: params
                .iter()
                .map(|param| nuis_semantics::model::AstParam {
                    name: (*param).to_owned(),
                    ty: i64_ty(),
                })
                .collect(),
            return_type: i64_ty(),
            host_symbol: None,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![
            host_extern(
                "host_serialize_text_into",
                &["text_handle", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_i64_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_bool_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_serialize_byte_into",
                &["value", "buffer_handle", "offset"],
            ),
            host_extern(
                "host_deserialize_i64_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern("host_deserialize_byte_from", &["buffer_handle", "offset"]),
            host_extern(
                "host_deserialize_bool_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern(
                "host_deserialize_text_from",
                &["buffer_handle", "offset", "len"],
            ),
            host_extern(
                "host_fill_bytes",
                &["buffer_handle", "offset", "len", "value"],
            ),
            host_extern(
                "host_copy_bytes",
                &[
                    "dst_handle",
                    "dst_offset",
                    "dst_len",
                    "src_handle",
                    "src_offset",
                    "src_len",
                ],
            ),
            host_extern(
                "host_compare_bytes",
                &[
                    "lhs_handle",
                    "lhs_offset",
                    "lhs_len",
                    "rhs_handle",
                    "rhs_offset",
                    "rhs_len",
                ],
            ),
        ],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(shim.contains("static int64_t nuis_host_serialize_text_into("));
    assert!(shim.contains("static int64_t nuis_host_text_line_count("));
    assert!(shim.contains("static int64_t nuis_host_text_word_count("));
    assert!(shim.contains("static int64_t nuis_host_serialize_i64_into("));
    assert!(shim.contains("static int64_t nuis_host_serialize_bool_into("));
    assert!(shim.contains("static int64_t nuis_host_serialize_byte_into("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_i64_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_byte_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_bool_from("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_from("));
    assert!(shim.contains("static int64_t nuis_host_parse_header_line("));
    assert!(shim.contains("static int64_t nuis_host_find_header_value("));
    assert!(shim.contains("static int64_t nuis_host_find_status_line_reason("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_response_summary("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_request_summary("));
    assert!(shim.contains("static int64_t nuis_host_parse_http_roundtrip_summary("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_equals("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_starts_with("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_contains("));
    assert!(shim.contains("static int64_t nuis_host_deserialize_text_ends_with("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_byte("));
    assert!(shim.contains("static int64_t nuis_host_fill_bytes("));
    assert!(shim.contains("static int64_t nuis_host_copy_bytes("));
    assert!(shim.contains("static int64_t nuis_host_compare_bytes("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_text("));
    assert!(shim.contains("static int64_t nuis_host_buffer_find_line_end("));
    assert!(shim.contains("static int64_t nuis_host_buffer_trim_line_end("));
    assert!(
        shim.contains("return nuis_host_serialize_text_into(text_handle, buffer_handle, offset);")
    );
    assert!(shim.contains("return nuis_host_serialize_i64_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_serialize_bool_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_serialize_byte_into(value, buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_deserialize_i64_from(buffer_handle, offset, len);"));
    assert!(shim.contains("return nuis_host_deserialize_byte_from(buffer_handle, offset);"));
    assert!(shim.contains("return nuis_host_deserialize_bool_from("));
    assert!(shim.contains("return nuis_host_deserialize_text_from("));
    assert!(shim.contains("return nuis_host_fill_bytes("));
    assert!(shim.contains("return nuis_host_copy_bytes("));
    assert!(shim.contains("return nuis_host_compare_bytes("));
}

#[test]
fn c_shim_source_leaves_plain_system_externs_unstubbed() {
    fn ty(name: &str) -> AstTypeRef {
        AstTypeRef {
            name: name.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![AstExternFunction {
            visibility: AstVisibility::Private,
            abi: "c".to_owned(),
            interface: None,
            name: "usleep".to_owned(),
            params: vec![nuis_semantics::model::AstParam {
                name: "usec".to_owned(),
                ty: ty("i64"),
            }],
            return_type: ty("i32"),
            host_symbol: None,
        }],
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: Vec::new(),
    };
    let shim = c_shim_source(&ast);
    assert!(!shim.contains("int32_t usleep("));
}

#[test]
fn c_shim_source_includes_exported_main_wrapper() {
    fn ty(name: &str) -> AstTypeRef {
        AstTypeRef {
            name: name.to_owned(),
            generic_args: Vec::new(),
            is_optional: false,
            is_ref: false,
        }
    }

    let ast = AstModule {
        attributes: Vec::new(),
        uses: Vec::new(),
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: Vec::new(),
        extern_interfaces: Vec::new(),
        consts: Vec::new(),
        type_aliases: Vec::new(),
        structs: Vec::new(),
        enums: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        functions: vec![nuis_semantics::model::AstFunction {
            name: "main".to_owned(),
            visibility: nuis_semantics::model::AstVisibility::Private,
            attributes: vec![nuis_semantics::model::AstAttribute {
                name: "export".to_owned(),
                args: vec![nuis_semantics::model::AstAttributeArg {
                    name: Some("name".to_owned()),
                    value: nuis_semantics::model::AstAttributeValue::String(
                        "entry_main".to_owned(),
                    ),
                }],
            }],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: Vec::new(),
            where_bounds: Vec::new(),
            params: Vec::new(),
            return_type: Some(ty("i64")),
            body: Vec::new(),
        }],
    };

    let shim = c_shim_source(&ast);
    assert!(shim.contains("int64_t entry_main(void) {"));
    assert!(shim.contains("return nuis_yir_entry();"));
}
