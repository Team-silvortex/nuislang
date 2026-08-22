use super::*;
use crate::{
    final_executable_elf_input::parse_elf64_amd64_object_linkage,
    final_executable_elf_loader_probe::ELF_AMD64_LOADER_PROBE_CONTRACT,
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
    final_executable_finalizer_registry::{
        invoke_registered_finalizer, invoke_registered_loader_probe, select_executable_finalizer,
        ExecutableFinalizerCommandContext,
    },
    final_executable_registered_loader_probe::{
        validate_registered_loader_probe_outcome, REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT,
    },
    fnv1a64_hex,
    main_final_executable_commands::try_run_registered_loader_probe,
    main_test_support::empty_link_plan,
};
use nuisc::{
    aot::{
        encode_nuis_compiled_artifact_section_table_binary, NuisCompiledArtifact,
        NuisCompiledArtifactHostObject, NuisExecutableEnvelope, NuisLifecycleContract,
    },
    linker::LinkPlanHostObject,
};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn validates_exec_and_pie_elf64_images() {
    let executable = elf_executable(ELF_TYPE_EXECUTABLE);
    let pie = elf_executable(ELF_TYPE_SHARED);

    assert!(validate_elf64_amd64_executable(&executable).is_ok());
    assert!(validate_elf64_amd64_executable(&pie).is_ok());
}

#[test]
fn rejects_entry_outside_executable_load_segment() {
    let mut image = elf_executable(ELF_TYPE_EXECUTABLE);
    write_u64(&mut image, 24, 0x500000);

    let error = validate_elf64_amd64_executable(&image).unwrap_err();

    assert!(error.contains("not inside a file-backed executable PT_LOAD"));
}

#[test]
fn rejects_wrong_machine_and_truncated_program_table() {
    let mut wrong_machine = elf_executable(ELF_TYPE_EXECUTABLE);
    write_u16(&mut wrong_machine, 18, 183);
    let mut truncated = elf_executable(ELF_TYPE_EXECUTABLE);
    let truncated_program_offset = truncated.len() as u64 - 8;
    write_u64(&mut truncated, 32, truncated_program_offset);

    assert!(validate_elf64_amd64_executable(&wrong_machine)
        .unwrap_err()
        .contains("expected x86_64"));
    assert!(validate_elf64_amd64_executable(&truncated)
        .unwrap_err()
        .contains("program-header table exceeds image bounds"));
}

#[test]
fn validates_relocatable_host_object_shape() {
    assert!(parse_elf64_amd64_object_linkage(&elf_program_object(R_X86_64_PLT32)).is_ok());
}

#[test]
fn rejects_non_sysv64_plan_before_artifact_loading() {
    let root = temp_dir("unsupported-abi");
    let (_, mut plan) = artifact_and_plan(&root, elf_executable(ELF_TYPE_EXECUTABLE));
    plan.cpu_target.abi = "cpu.x86_64.unsupported".to_owned();

    assert!(validate_plan_target(&plan)
        .unwrap_err()
        .contains("unsupported CPU ABI"));

    plan.cpu_target.abi = ELF_AMD64_CPU_ABI.to_owned();
    plan.cpu_target.calling_abi = "win64".to_owned();
    assert!(validate_plan_target(&plan)
        .unwrap_err()
        .contains("unsupported calling ABI"));
}

#[test]
fn materializes_validated_artifact_without_host_driver() {
    let root = temp_dir("materialize");
    fs::create_dir_all(&root).unwrap();
    let image = elf_executable(ELF_TYPE_EXECUTABLE);
    let artifact_path = root.join("nuis.compiled.artifact");
    let output_path = root.join("demo");
    let (artifact, mut plan) = artifact_and_plan(&root, image.clone());
    fs::write(
        &artifact_path,
        encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap(),
    )
    .unwrap();

    let selection = select_executable_finalizer(&plan).unwrap();
    assert_eq!(
        selection.provider_id(),
        "nsld.finalizer.elf.amd64.artifact-image-v1"
    );
    let command_args = selection.command_args(&ExecutableFinalizerCommandContext {
        driver: &plan.final_stage.driver,
        target_triple: &plan.cpu_target.clang_target,
        native_object_path: "",
        compiled_artifact_path: &plan.compiled_artifact.path,
        output_path: output_path.to_str().unwrap(),
    });
    invoke_registered_finalizer(&plan, &command_args, None, &output_path).unwrap();
    let actual = fs::read(&output_path).unwrap();
    assert_eq!(actual, image);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_ne!(
            fs::metadata(&output_path).unwrap().permissions().mode() & 0o111,
            0
        );
    }

    plan.compiled_artifact.host_objects[0].content_hash = "0x0000000000000000".to_owned();
    let issues = elf_amd64_artifact_image_validation_issues(&plan);
    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("hash mismatch"));

    plan.compiled_artifact.host_objects[0].content_hash =
        fnv1a64_hex(&artifact.host_objects[0].bytes);
    let mut drifted_artifact = artifact.clone();
    drifted_artifact.cpu_target_abi = "cpu.x86_64.unsupported".to_owned();
    fs::write(
        &artifact_path,
        encode_nuis_compiled_artifact_section_table_binary(&drifted_artifact).unwrap(),
    )
    .unwrap();
    let issues = elf_amd64_artifact_image_validation_issues(&plan);
    fs::remove_dir_all(root).unwrap();

    assert_eq!(issues.len(), 1);
    assert!(issues[0].contains("CPU ABI mismatch"));
}

#[test]
fn registered_loader_probe_projects_protocol_neutral_plan_only_outcome() {
    let root = temp_dir("registered-loader-probe");
    fs::create_dir_all(&root).unwrap();
    let artifact_path = root.join("nuis.compiled.artifact");
    let probe_root = root.join("probe-root");
    let (artifact, plan) = artifact_and_plan(&root, elf_executable(ELF_TYPE_EXECUTABLE));
    fs::write(
        &artifact_path,
        encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap(),
    )
    .unwrap();

    let outcome = invoke_registered_loader_probe(&plan, &probe_root, false).unwrap();

    assert_eq!(outcome.contract, REGISTERED_LOADER_PROBE_OUTCOME_CONTRACT);
    assert_eq!(outcome.status, "execution-not-attempted");
    assert_eq!(
        outcome.provider_id,
        "nsld.finalizer.elf.amd64.artifact-image-v1"
    );
    assert_eq!(outcome.target_key, "x86_64-linux-elf");
    assert_eq!(
        outcome.capability_id,
        ELF_AMD64_REGISTERED_LOADER_PROBE_CAPABILITY
    );
    assert_eq!(
        outcome.provider_probe_contract,
        ELF_AMD64_LOADER_PROBE_CONTRACT
    );
    assert_eq!(outcome.probe_mode, "plan-only");
    assert!(outcome.input_eligible);
    assert!(!outcome.attempted);
    assert!(!outcome.materialized);
    assert!(!outcome.execution_admitted);
    assert_eq!(outcome.blockers.len(), 1);
    assert!(!probe_root.exists());
    validate_registered_loader_probe_outcome(&outcome).unwrap();

    let mut drifted = outcome;
    drifted.provider_probe_status.push_str("-drift");
    assert!(validate_registered_loader_probe_outcome(&drifted)
        .unwrap_err()
        .contains("ledger drift"));

    assert!(try_run_registered_loader_probe(&plan, true, false).unwrap());
    let apply_error = try_run_registered_loader_probe(&plan, true, true).unwrap_err();
    assert!(apply_error.contains("apply remains disabled until host execution evidence"));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn rejects_unregistered_object_relocation_before_output_mutation() {
    let root = temp_dir("unsupported-relocation");
    fs::create_dir_all(&root).unwrap();
    let artifact_path = root.join("nuis.compiled.artifact");
    let output_path = root.join("demo");
    let (mut artifact, mut plan) = artifact_and_plan(&root, elf_executable(ELF_TYPE_EXECUTABLE));
    artifact.host_objects[0].bytes = elf_program_object(0x7fff);
    plan.compiled_artifact.host_objects[0].bytes = artifact.host_objects[0].bytes.len();
    plan.compiled_artifact.host_objects[0].content_hash =
        fnv1a64_hex(&artifact.host_objects[0].bytes);
    fs::write(
        &artifact_path,
        encode_nuis_compiled_artifact_section_table_binary(&artifact).unwrap(),
    )
    .unwrap();
    fs::write(&output_path, b"preserve-me").unwrap();

    let error = materialize_elf_amd64_artifact_image(&plan, &output_path).unwrap_err();
    let output = fs::read(&output_path).unwrap();
    fs::remove_dir_all(root).unwrap();

    assert!(error.contains("unsupported R_X86_64 type 32767"));
    assert_eq!(output, b"preserve-me");
}

fn artifact_and_plan(
    root: &std::path::Path,
    image: Vec<u8>,
) -> (NuisCompiledArtifact, nuisc::linker::LinkPlan) {
    let objects = vec![
        NuisCompiledArtifactHostObject {
            object_id: "host.program-llvm".to_owned(),
            role: "program-llvm".to_owned(),
            object_format: "elf".to_owned(),
            bytes: elf_program_object(R_X86_64_PLT32),
        },
        NuisCompiledArtifactHostObject {
            object_id: "host.runtime-shim".to_owned(),
            role: "runtime-shim".to_owned(),
            object_format: "elf".to_owned(),
            bytes: elf_runtime_object(),
        },
    ];
    let manifest = build_manifest(root, image.len());
    let artifact = NuisCompiledArtifact {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
        cpu_target_machine_arch: "x86_64".to_owned(),
        cpu_target_machine_os: "linux".to_owned(),
        cpu_target_object_format: "elf".to_owned(),
        cpu_target_calling_abi: "sysv64".to_owned(),
        binary_name: "demo".to_owned(),
        binary_bytes: image.len(),
        build_manifest_bytes: manifest.len(),
        envelope: NuisExecutableEnvelope {
            schema: "nuis-executable-envelope-v1".to_owned(),
            executable_kind: "native-cpu-llvm".to_owned(),
            package_count: 0,
            domain_families: Vec::new(),
            contract_families: Vec::new(),
            function_kind: "function-node".to_owned(),
            graph_kind: "function-graph".to_owned(),
            default_time_mode: "logical".to_owned(),
        },
        lifecycle: NuisLifecycleContract {
            schema: "nuis-lifecycle-contract-v1".to_owned(),
            bootstrap_entry: "nuis.bootstrap.lifecycle.v1".to_owned(),
            tick_policy: "cooperative".to_owned(),
            shutdown_policy: "graceful".to_owned(),
            yalivia_rpc: "disabled".to_owned(),
            hook_surface: vec!["on_bootstrap".to_owned()],
            export_surface: vec!["main".to_owned()],
            runtime_capability_flags: Vec::new(),
        },
        build_manifest_source: manifest,
        binary_blob: image,
        host_objects: objects.clone(),
    };
    let mut plan = empty_link_plan();
    plan.output_dir = root.display().to_string();
    plan.packaging_mode = "native-cpu-llvm".to_owned();
    plan.cpu_target.abi = "cpu.x86_64.sysv64".to_owned();
    plan.cpu_target.machine_arch = "x86_64".to_owned();
    plan.cpu_target.machine_os = "linux".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    plan.cpu_target.calling_abi = "sysv64".to_owned();
    plan.cpu_target.clang_target = "x86_64-unknown-linux-gnu".to_owned();
    plan.compiled_artifact.path = root.join("nuis.compiled.artifact").display().to_string();
    plan.compiled_artifact.binary_name = "demo".to_owned();
    plan.compiled_artifact.binary_bytes = artifact.binary_bytes;
    plan.compiled_artifact.host_objects = objects
        .iter()
        .map(|object| LinkPlanHostObject {
            object_id: object.object_id.clone(),
            role: object.role.clone(),
            object_format: object.object_format.clone(),
            bytes: object.bytes.len(),
            content_hash: fnv1a64_hex(&object.bytes),
        })
        .collect();
    plan.final_stage.output_path = root.join("demo").display().to_string();
    (artifact, plan)
}

fn build_manifest(root: &std::path::Path, binary_bytes: usize) -> String {
    format!(
        "manifest_schema = \"nuis-build-manifest-v1\"\ninput = \"demo.ns\"\noutput_dir = \"{}\"\npackaging_mode = \"native-cpu-llvm\"\npath = \"nuis.executable.envelope.toml\"\nschema = \"nuis-executable-envelope-v1\"\npackage_count = 0\nartifact_path = \"nuis.compiled.artifact\"\nartifact_schema = \"nuis-compiled-artifact-v1\"\nartifact_binary_name = \"demo\"\nartifact_binary_bytes = {binary_bytes}\nlifecycle_schema = \"nuis-lifecycle-contract-v1\"\nlifecycle_bootstrap_entry = \"nuis.bootstrap.lifecycle.v1\"\nlifecycle_tick_policy = \"cooperative\"\nlifecycle_shutdown_policy = \"graceful\"\nlifecycle_yalivia_rpc = \"disabled\"\nlifecycle_hook_surface = [\"on_bootstrap\"]\nlifecycle_export_surface = [\"main\"]\nlifecycle_runtime_capability_flags = []\nfunction_kind = \"function-node\"\ngraph_kind = \"function-graph\"\ndefault_time_mode = \"logical\"\ncpu_target_abi = \"cpu.x86_64.sysv64\"\ncpu_target_machine_arch = \"x86_64\"\ncpu_target_machine_os = \"linux\"\ncpu_target_object_format = \"elf\"\ncpu_target_calling_abi = \"sysv64\"\ncpu_target_clang = \"x86_64-unknown-linux-gnu\"\ncpu_target_cross = false\n",
        root.display()
    )
}

fn elf_executable(file_type: u16) -> Vec<u8> {
    let mut bytes = vec![0u8; 256];
    let image_size = bytes.len() as u64;
    write_ident(&mut bytes);
    write_u16(&mut bytes, 16, file_type);
    write_u16(&mut bytes, 18, ELF_MACHINE_X86_64);
    write_u32(&mut bytes, 20, 1);
    write_u64(&mut bytes, 24, 0x400080);
    write_u64(&mut bytes, 32, ELF64_HEADER_SIZE as u64);
    write_u16(&mut bytes, 52, ELF64_HEADER_SIZE as u16);
    write_u16(&mut bytes, 54, ELF64_PROGRAM_HEADER_SIZE as u16);
    write_u16(&mut bytes, 56, 1);
    write_u32(&mut bytes, ELF64_HEADER_SIZE, ELF_PROGRAM_TYPE_LOAD);
    write_u32(&mut bytes, ELF64_HEADER_SIZE + 4, 5);
    write_u64(&mut bytes, ELF64_HEADER_SIZE + 8, 0);
    write_u64(&mut bytes, ELF64_HEADER_SIZE + 16, 0x400000);
    write_u64(&mut bytes, ELF64_HEADER_SIZE + 32, image_size);
    write_u64(&mut bytes, ELF64_HEADER_SIZE + 40, image_size);
    write_u64(&mut bytes, ELF64_HEADER_SIZE + 48, 0x1000);
    bytes[0x80] = 0xc3;
    bytes
}

fn write_ident(bytes: &mut [u8]) {
    bytes[..7].copy_from_slice(&[0x7f, b'E', b'L', b'F', 2, 1, 1]);
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "nsld-elf-finalizer-{label}-{}-{nonce}",
        std::process::id()
    ))
}
