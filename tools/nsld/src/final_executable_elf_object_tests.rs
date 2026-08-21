use super::*;
use crate::{
    final_executable_elf_test_fixture::{elf_program_object, elf_runtime_object, R_X86_64_PLT32},
    main_test_support::empty_link_plan,
};
use nuisc::{
    aot::{
        NuisCompiledArtifact, NuisCompiledArtifactHostObject, NuisExecutableEnvelope,
        NuisLifecycleContract,
    },
    linker::LinkPlanHostObject,
};

#[test]
fn summarizes_cross_object_internal_symbol_closure() {
    let (artifact, plan) =
        artifact_and_plan(elf_program_object(R_X86_64_PLT32), elf_runtime_object());

    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();

    assert_eq!(
        product.summary.contract,
        ELF_AMD64_HOST_OBJECT_LINKAGE_CONTRACT
    );
    assert_eq!(product.summary.status, "verified-internal-closure");
    assert_eq!(product.summary.object_count, 2);
    assert_eq!(product.summary.section_count, 9);
    assert_eq!(product.summary.symbol_count, 5);
    assert_eq!(product.summary.relocation_count, 1);
    assert_eq!(product.summary.defined_symbol_count, 2);
    assert_eq!(product.summary.undefined_symbol_count, 1);
    assert_eq!(
        product.summary.internally_resolved_symbols,
        ["nuis_runtime_entry"]
    );
    assert!(product.summary.unresolved_external_symbols.is_empty());
    assert_eq!(product.objects[0].role, "program-llvm");
    assert_eq!(product.objects[1].role, "runtime-shim");
}

#[test]
fn rejects_duplicate_strong_definitions_across_objects() {
    let (artifact, plan) = artifact_and_plan(
        elf_program_object(R_X86_64_PLT32),
        elf_program_object(R_X86_64_PLT32),
    );

    let error = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap_err();

    assert!(error.contains("strong symbol `__nuis_entry`"));
    assert!(error.contains("host.program-llvm"));
    assert!(error.contains("host.runtime-shim"));
}

fn artifact_and_plan(
    program: Vec<u8>,
    runtime: Vec<u8>,
) -> (NuisCompiledArtifact, nuisc::linker::LinkPlan) {
    let objects = vec![
        NuisCompiledArtifactHostObject {
            object_id: "host.program-llvm".to_owned(),
            role: "program-llvm".to_owned(),
            object_format: "elf".to_owned(),
            bytes: program,
        },
        NuisCompiledArtifactHostObject {
            object_id: "host.runtime-shim".to_owned(),
            role: "runtime-shim".to_owned(),
            object_format: "elf".to_owned(),
            bytes: runtime,
        },
    ];
    let artifact = NuisCompiledArtifact {
        schema: "nuis-compiled-artifact-v1".to_owned(),
        packaging_mode: "native-cpu-llvm".to_owned(),
        cpu_target_abi: "cpu.x86_64.sysv64".to_owned(),
        cpu_target_machine_arch: "x86_64".to_owned(),
        cpu_target_machine_os: "linux".to_owned(),
        cpu_target_object_format: "elf".to_owned(),
        cpu_target_calling_abi: "sysv64".to_owned(),
        binary_name: "demo".to_owned(),
        binary_bytes: 0,
        build_manifest_bytes: 0,
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
            hook_surface: Vec::new(),
            export_surface: Vec::new(),
            runtime_capability_flags: Vec::new(),
        },
        build_manifest_source: String::new(),
        binary_blob: Vec::new(),
        host_objects: objects.clone(),
    };
    let mut plan = empty_link_plan();
    plan.packaging_mode = "native-cpu-llvm".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    plan.compiled_artifact.host_objects = objects
        .iter()
        .map(|object| LinkPlanHostObject {
            object_id: object.object_id.clone(),
            role: object.role.clone(),
            object_format: object.object_format.clone(),
            bytes: object.bytes.len(),
            content_hash: crate::fnv1a64_hex(&object.bytes),
        })
        .collect();
    (artifact, plan)
}
