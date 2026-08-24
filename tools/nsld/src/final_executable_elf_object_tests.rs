use super::*;
use crate::{
    final_executable_elf_test_fixture::{
        elf_program_object, elf_program_object_with_external_symbol, elf_runtime_object,
        elf_unrelated_runtime_object, R_X86_64_PLT32,
    },
    main_test_support::empty_link_plan,
};
use nuisc::{
    aot::{
        NuisCompiledArtifact, NuisCompiledArtifactHostObject, NuisExecutableEnvelope,
        NuisLifecycleContract,
    },
    linker::{
        LinkPlanHostFfiAbiEntry, LinkPlanHostFfiAbiGroup, LinkPlanHostFfiEntry,
        LinkPlanHostFfiValidationSummary, LinkPlanHostObject,
    },
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
    assert_eq!(
        product.placement_binding.contract,
        crate::final_executable_elf_layout::ELF_AMD64_PLACEMENT_BINDING_CONTRACT
    );
    assert_eq!(
        product.placement_binding.status,
        "placement-and-internal-binding-ready"
    );
    assert_eq!(product.placement_binding.section_placements.len(), 2);
    assert_eq!(product.placement_binding.internally_bound_symbol_count, 1);
    assert_eq!(
        product.relocation_application.contract,
        crate::final_executable_elf_relocation::ELF_AMD64_RELOCATION_APPLICATION_CONTRACT
    );
    assert_eq!(
        product.relocation_application.status,
        "ready-for-byte-preview"
    );
    assert_eq!(product.relocation_application.direct_preview_count, 1);
    assert_eq!(
        product.relocation_application.applications[0].encoded_bytes,
        [0x0b, 0, 0, 0]
    );
    assert_eq!(
        product.materialization_preview.contract,
        crate::final_executable_elf_materialization::ELF_AMD64_MATERIALIZATION_PREVIEW_CONTRACT
    );
    assert_eq!(product.materialization_preview.status, "preview-ready");
    assert_eq!(product.materialization_preview.copied_bytes, 7);
    assert_eq!(product.materialization_preview.previewed_patch_count, 1);
    assert_eq!(
        product.materialization_preview.patches[0].encoded_bytes,
        [0x0b, 0, 0, 0]
    );
    assert_eq!(
        product.patch_application.contract,
        crate::final_executable_elf_materialization::application::ELF_AMD64_PATCH_APPLICATION_CONTRACT
    );
    assert_eq!(product.patch_application.status, "direct-patches-applied");
    assert_eq!(product.patch_application.applied_patch_count, 1);
    assert_eq!(
        product.patch_application.patches[0].status,
        "applied-write-once"
    );
    assert_eq!(
        product.platform_structure_plan.contract,
        crate::final_executable_elf_materialization::application::platform::ELF_AMD64_PLATFORM_STRUCTURE_PLAN_CONTRACT
    );
    assert_eq!(product.platform_structure_plan.status, "not-required");
    assert_eq!(
        product.platform_structure_plan.planned_memory_span_bytes,
        product.patch_application.memory_span_bytes
    );
    assert_eq!(
        product.platform_patch_application.contract,
        crate::final_executable_elf_materialization::application::platform::application::ELF_AMD64_PLATFORM_PATCH_APPLICATION_CONTRACT
    );
    assert_eq!(
        product.platform_patch_application.status,
        "not-required-image-preserved"
    );
    assert_eq!(
        product
            .platform_patch_application
            .base_applied_memory_image_hash,
        product.platform_patch_application.applied_memory_image_hash
    );
    assert_eq!(
        product.platform_patch_application.application_ledger_hash,
        crate::fnv1a64_hex(
            product
                .platform_patch_application
                .canonical_ledger()
                .as_bytes()
        )
    );
    assert_eq!(
        product.shell_layout_plan.contract,
        crate::final_executable_elf_shell::ELF_AMD64_SHELL_LAYOUT_PLAN_CONTRACT
    );
    assert_eq!(
        product.shell_layout_plan.status,
        "static-closure-layout-planned"
    );
    assert_eq!(product.shell_layout_plan.entry_symbol, "__nuis_entry");
    assert_eq!(product.shell_layout_plan.dynamic_table_entry_count, 0);
    assert_eq!(
        product.shell_layout_plan.platform_application_ledger_hash,
        product.platform_patch_application.application_ledger_hash
    );
    assert_eq!(
        product.shell_image_serialization.contract,
        "nuis-nsld-elf-amd64-shell-image-serialization-v1"
    );
    assert_eq!(
        product.shell_image_serialization.status,
        "serialized-static-private-image"
    );
    assert_eq!(
        product.private_shell_image.get(..4),
        Some(b"\x7fELF".as_slice())
    );
    assert_eq!(
        product.shell_image_serialization.shell_image_hash,
        crate::fnv1a64_hex(&product.private_shell_image)
    );
    assert_eq!(
        product.shell_image_serialization.serialization_ledger_hash,
        crate::fnv1a64_hex(
            product
                .shell_image_serialization
                .canonical_ledger()
                .as_bytes()
        )
    );
    assert_eq!(
        product.shell_image_validation.contract,
        "nuis-nsld-elf-amd64-shell-image-validation-v1"
    );
    assert_eq!(
        product.shell_image_validation.status,
        "independently-validated-private-image"
    );
    assert_eq!(
        product.shell_image_validation.shell_image_hash,
        product.shell_image_serialization.shell_image_hash
    );
    assert_eq!(
        product.shell_image_validation.serialization_ledger_hash,
        product.shell_image_serialization.serialization_ledger_hash
    );
    assert_eq!(
        product.shell_image_validation.validation_ledger_hash,
        crate::fnv1a64_hex(product.shell_image_validation.canonical_ledger().as_bytes())
    );
    assert!(!product.shell_image_validation.publication_eligible);
    assert_eq!(
        product.shell_image_validation.publication_blockers,
        ["os-loader-probe-pending"]
    );
    assert_eq!(
        product.dynamic_resolution_provenance.status,
        "not-required-static-closure"
    );
    assert!(product.dynamic_resolution_provenance.provenance_ready);
    assert_eq!(
        product.dynamic_resolution_provenance.provenance_ledger_hash,
        crate::fnv1a64_hex(
            product
                .dynamic_resolution_provenance
                .canonical_ledger()
                .as_bytes()
        )
    );
    assert!(product
        .dynamic_resolution_provenance
        .dependencies
        .is_empty());
    assert!(product.dynamic_resolution_provenance.bindings.is_empty());
}

#[test]
fn object_chain_applies_external_platform_records_and_deferred_call() {
    let (artifact, plan) = artifact_and_plan(
        elf_program_object(R_X86_64_PLT32),
        elf_unrelated_runtime_object(),
    );

    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();

    assert_eq!(
        product.summary.status,
        "verified-with-external-compatibility-boundary"
    );
    assert_eq!(product.platform_structure_plan.target_count, 1);
    assert_eq!(
        product.platform_patch_application.status,
        "platform-structures-and-deferred-patches-applied-with-unresolved-dynamic-binds"
    );
    assert_eq!(
        product
            .platform_patch_application
            .applied_structure_write_count,
        7
    );
    assert_eq!(
        product
            .platform_patch_application
            .applied_deferred_patch_count,
        1
    );
    assert_eq!(
        product
            .platform_patch_application
            .unresolved_dynamic_bind_count,
        1
    );
    assert_eq!(
        product.platform_patch_application.dynamic_bind_records[0].target_symbol,
        "nuis_runtime_entry"
    );
    assert_ne!(
        product
            .platform_patch_application
            .base_applied_memory_image_hash,
        product.platform_patch_application.applied_memory_image_hash
    );
    assert_eq!(
        product.shell_layout_plan.status,
        "layout-planned-with-external-resolution-boundary"
    );
    assert_eq!(product.shell_layout_plan.dynamic_table_entry_count, 12);
    assert!(product
        .shell_layout_plan
        .dynamic_table_file_offset
        .is_some());
    assert_eq!(
        product.shell_image_serialization.status,
        "serialized-private-image-with-external-resolution-boundary"
    );
    assert_eq!(
        product.shell_image_serialization.applied_shell_write_count,
        5
    );
    assert_eq!(
        product.shell_image_serialization.dynamic_table_bytes,
        product.shell_layout_plan.dynamic_table_bytes
    );
    assert_eq!(
        product.shell_image_validation.status,
        "independently-validated-private-image"
    );
    assert_eq!(product.shell_image_validation.dynamic_segment_count, 1);
    assert_eq!(product.shell_image_validation.dynamic_entry_count, 12);
    assert!(!product.shell_image_validation.publication_eligible);
    assert_eq!(
        product.shell_image_validation.publication_blockers,
        [
            "os-loader-probe-pending",
            "registered-external-resolution-provenance-pending"
        ]
    );
    assert_eq!(
        product.dynamic_resolution_provenance.status,
        "blocked-dynamic-resolution-provenance"
    );
    assert!(!product.dynamic_resolution_provenance.provenance_ready);
    assert!(product
        .dynamic_resolution_provenance
        .issues
        .iter()
        .any(|issue| issue == "missing-host-ffi-whitelist:nuis_runtime_entry"));
    assert!(product.dynamic_resolution_provenance.bindings.is_empty());
}

#[test]
fn binds_whitelisted_libc_symbol_to_registered_linux_gnu_provider() {
    let (artifact, mut plan) = artifact_and_plan(
        elf_program_object_with_external_symbol(R_X86_64_PLT32, "puts"),
        elf_unrelated_runtime_object(),
    );
    install_host_ffi_entries(
        &mut plan,
        vec![host_ffi_entry("libc", "puts", "i32(String)")],
    );

    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();
    let provenance = &product.dynamic_resolution_provenance;

    assert_eq!(
        product.dynamic_dependency_plan.status,
        "registered-dynamic-dependency-plan-ready"
    );
    assert!(product.dynamic_dependency_plan.plan_ready);
    assert_eq!(
        product
            .shell_layout_plan
            .dynamic_dependency_plan_hash
            .as_deref(),
        Some(product.dynamic_dependency_plan.plan_hash.as_str())
    );
    assert_eq!(
        product.shell_layout_plan.interpreter_path.as_deref(),
        Some("/lib64/ld-linux-x86-64.so.2")
    );
    assert_eq!(product.shell_layout_plan.needed_libraries.len(), 1);
    assert_eq!(
        product.shell_layout_plan.needed_libraries[0].needed_name,
        "libc.so.6"
    );
    assert_eq!(product.shell_layout_plan.dynamic_table_entry_count, 17);
    assert_eq!(
        product.shell_image_serialization.applied_shell_write_count,
        9
    );
    assert_eq!(product.shell_image_validation.interpreter_segment_count, 1);
    assert_eq!(
        product.shell_image_validation.interpreter_path.as_deref(),
        Some("/lib64/ld-linux-x86-64.so.2")
    );
    assert_eq!(
        product.shell_image_validation.needed_libraries,
        ["libc.so.6"]
    );
    assert_eq!(
        product.shell_image_validation.version_symbol_indexes,
        [0, 2]
    );
    assert_eq!(
        product.shell_image_validation.version_requirements,
        ["libc.so.6@GLIBC_2.2.5#2"]
    );
    for tag in ["DT_VERSYM", "DT_VERNEED", "DT_VERNEEDNUM"] {
        assert!(product
            .shell_layout_plan
            .dynamic_entries
            .iter()
            .any(|entry| entry.tag_name == tag));
    }

    let mut interpreter_drift = product.private_shell_image.clone();
    interpreter_drift[product.shell_layout_plan.interpreter_file_offset.unwrap()] ^= 0x01;
    let error = crate::final_executable_elf_shell::validate_elf_amd64_shell_bytes_against_plan(
        &interpreter_drift,
        &product.shell_layout_plan,
    )
    .unwrap_err();
    assert!(error.contains("PT_INTERP payload differs"));

    let dynstr = product
        .shell_layout_plan
        .sections
        .iter()
        .find(|section| section.section_name == ".dynstr")
        .unwrap();
    let mut needed_drift = product.private_shell_image.clone();
    needed_drift[dynstr.file_offset
        + product.shell_layout_plan.needed_libraries[0].dynamic_string_offset] ^= 0x01;
    let error = crate::final_executable_elf_shell::validate_elf_amd64_shell_bytes_against_plan(
        &needed_drift,
        &product.shell_layout_plan,
    )
    .unwrap_err();
    assert!(error.contains("DT_NEEDED name differs"));

    let mut version_symbol_drift = product.private_shell_image.clone();
    version_symbol_drift[product
        .shell_layout_plan
        .version_symbol_table_file_offset
        .unwrap()
        + 2] ^= 0x01;
    let error = crate::final_executable_elf_shell::validate_elf_amd64_shell_bytes_against_plan(
        &version_symbol_drift,
        &product.shell_layout_plan,
    )
    .unwrap_err();
    assert!(error.contains("version-symbol value differs"));

    let mut version_need_drift = product.private_shell_image.clone();
    version_need_drift[product
        .shell_layout_plan
        .version_need_table_file_offset
        .unwrap()
        + 16] ^= 0x01;
    let error = crate::final_executable_elf_shell::validate_elf_amd64_shell_bytes_against_plan(
        &version_need_drift,
        &product.shell_layout_plan,
    )
    .unwrap_err();
    assert!(error.contains("version auxiliary differs"));

    assert_eq!(
        provenance.status,
        "verified-registered-dynamic-resolution-provenance"
    );
    assert!(provenance.provenance_ready);
    assert!(provenance.issues.is_empty());
    assert_eq!(provenance.unresolved_symbol_count, 1);
    assert_eq!(provenance.dynamic_bind_count, 1);
    assert_eq!(provenance.resolved_binding_count, 1);
    assert_eq!(provenance.dependencies.len(), 1);
    assert_eq!(
        provenance.dependencies[0].provider_id,
        "nsld.elf.amd64.linux-gnu.libc-v1"
    );
    assert_eq!(
        provenance.dependencies[0].interpreter_path,
        "/lib64/ld-linux-x86-64.so.2"
    );
    assert_eq!(provenance.dependencies[0].needed_name, "libc.so.6");
    assert_eq!(
        provenance.dependencies[0].symbol_version_policy,
        "elf-registered-symbol-version-whitelist-v1"
    );
    assert_eq!(
        provenance.dependencies[0].resolver_identity,
        "elf.sysv.amd64.bind-now-plt-v1"
    );
    assert_eq!(provenance.bindings[0].target_symbol, "puts");
    assert_eq!(provenance.bindings[0].dynamic_symbol_index, 1);
    assert_eq!(
        provenance.bindings[0].symbol_version_identity,
        "linux.gnu.glibc.2.2.5-v1"
    );
    assert_eq!(provenance.bindings[0].symbol_version_name, "GLIBC_2.2.5");
    assert_eq!(provenance.bindings[0].symbol_version_index, 2);
    assert_eq!(provenance.bindings[0].symbol_version_hash, 0x0969_1a75);
    assert_eq!(provenance.bindings[0].host_ffi_abi, "libc");
    assert_eq!(
        provenance.bindings[0].platform_bind_audit_hash,
        product.platform_patch_application.dynamic_bind_records[0].audit_hash
    );
    crate::final_executable_elf_dynamic_provenance::validate_elf_amd64_dynamic_resolution_provenance_report(provenance)
        .unwrap();

    let mut dependency_drift = provenance.clone();
    dependency_drift.dependencies[0].needed_name = "libm.so.6".to_owned();
    let error = crate::final_executable_elf_dynamic_provenance::validate_elf_amd64_dynamic_resolution_provenance_report(&dependency_drift)
        .unwrap_err();
    assert!(error.contains("dependency plan record"));

    let mut relocated_plan = plan.clone();
    relocated_plan.host_ffi.index_path = Some("relocated/host_ffi.index".to_owned());
    let relocated = build_elf_amd64_host_object_linkage(&artifact, &relocated_plan).unwrap();
    assert_eq!(
        relocated
            .dynamic_resolution_provenance
            .host_ffi_footprint_hash,
        provenance.host_ffi_footprint_hash
    );

    let mut target_drift_plan = plan;
    target_drift_plan.cpu_target.clang_target = "x86_64-unknown-linux-musl".to_owned();
    let target_drift = build_elf_amd64_host_object_linkage(&artifact, &target_drift_plan).unwrap();
    assert!(!target_drift.dynamic_resolution_provenance.provenance_ready);
    assert_eq!(
        target_drift.dynamic_resolution_provenance.issues,
        ["registered-dynamic-provider-missing:libc:puts"]
    );
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[test]
fn executes_registered_libc_symbol_through_the_system_loader() {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (artifact, mut plan) = artifact_and_plan(
        elf_program_object_with_external_symbol(R_X86_64_PLT32, "sched_yield"),
        crate::final_executable_elf_test_fixture::elf_linux_exit_runtime_object(),
    );
    install_host_ffi_entries(
        &mut plan,
        vec![host_ffi_entry("libc", "sched_yield", "i32()")],
    );
    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();
    assert!(product.dynamic_dependency_plan.plan_ready);
    assert_eq!(
        product.shell_image_validation.needed_libraries,
        ["libc.so.6"]
    );

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "nuis-nsld-dynamic-loader-probe-{}-{nonce}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    let report = crate::final_executable_elf_loader_probe::probe_elf_amd64_private_shell_image(
        crate::final_executable_elf_loader_probe::ElfAmd64LoaderProbeInput {
            bytes: &product.private_shell_image,
            validation: &product.shell_image_validation,
            unresolved_external_symbol_count: product.summary.unresolved_external_symbols.len(),
            dynamic_provenance: Some(&product.dynamic_resolution_provenance),
        },
        &root,
        true,
    )
    .unwrap();

    assert!(report.input_eligible, "{report:#?}");
    assert!(report.dynamic_provenance_ready, "{report:#?}");
    assert_eq!(report.exit_code, Some(0), "{report:#?}");
    assert!(report.publication_eligible, "{report:#?}");
    assert!(std::fs::read_dir(&root).unwrap().next().is_none());
    std::fs::remove_dir(root).unwrap();
}

#[test]
fn blocks_ambiguous_host_ffi_signatures_before_provider_binding() {
    let (artifact, mut plan) = artifact_and_plan(
        elf_program_object_with_external_symbol(R_X86_64_PLT32, "puts"),
        elf_unrelated_runtime_object(),
    );
    install_host_ffi_entries(
        &mut plan,
        vec![
            host_ffi_entry("libc", "puts", "i32(String)"),
            host_ffi_entry("libc", "puts", "i32(i64)"),
        ],
    );

    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();
    let provenance = &product.dynamic_resolution_provenance;

    assert_eq!(provenance.status, "blocked-dynamic-resolution-provenance");
    assert!(!provenance.provenance_ready);
    assert_eq!(provenance.issues, ["ambiguous-host-ffi-signature:puts:2"]);
    assert!(provenance.dependencies.is_empty());
    assert!(provenance.bindings.is_empty());
}

#[test]
fn blocks_unregistered_libc_symbol_version_before_shell_emission() {
    let (artifact, mut plan) = artifact_and_plan(
        elf_program_object_with_external_symbol(R_X86_64_PLT32, "getpid"),
        elf_unrelated_runtime_object(),
    );
    install_host_ffi_entries(&mut plan, vec![host_ffi_entry("libc", "getpid", "i32()")]);

    let product = build_elf_amd64_host_object_linkage(&artifact, &plan).unwrap();

    assert!(!product.dynamic_dependency_plan.plan_ready);
    assert_eq!(
        product.dynamic_dependency_plan.issues,
        ["registered-symbol-version-missing:libc:getpid"]
    );
    assert!(product.dynamic_dependency_plan.dependencies.is_empty());
    assert!(product.dynamic_dependency_plan.bindings.is_empty());
    assert!(product.shell_layout_plan.interpreter_path.is_none());
    assert!(product.shell_layout_plan.version_symbols.is_empty());
    assert!(product.shell_layout_plan.version_needs.is_empty());
    assert!(product
        .shell_layout_plan
        .sections
        .iter()
        .all(|section| !section.section_name.starts_with(".gnu.version")));
    assert!(!product.dynamic_resolution_provenance.provenance_ready);
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
    plan.cpu_target.abi = "cpu.x86_64.sysv64".to_owned();
    plan.cpu_target.machine_arch = "x86_64".to_owned();
    plan.cpu_target.machine_os = "linux".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    plan.cpu_target.calling_abi = "sysv64".to_owned();
    plan.cpu_target.clang_target = "x86_64-unknown-linux-gnu".to_owned();
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

fn host_ffi_entry(abi: &str, symbol: &str, signature: &str) -> LinkPlanHostFfiEntry {
    LinkPlanHostFfiEntry {
        abi: abi.to_owned(),
        symbol: symbol.to_owned(),
        signature_pattern: signature.to_owned(),
        signature_hash: yir_core::ffi::ffi_symbol_signature_hash(abi, symbol, signature),
        policy: "signature-whitelist-required".to_owned(),
        memory_capabilities: Vec::new(),
    }
}

fn install_host_ffi_entries(
    plan: &mut nuisc::linker::LinkPlan,
    entries: Vec<LinkPlanHostFfiEntry>,
) {
    let abi = entries[0].abi.clone();
    assert!(entries.iter().all(|entry| entry.abi == abi));
    let notes = (entries.len() > 1).then(|| {
        format!(
            "host_ffi ABI `{abi}` symbol `{}` has {} whitelisted signatures",
            entries[0].symbol,
            entries.len()
        )
    });
    let validation = LinkPlanHostFfiValidationSummary {
        checked: entries.len(),
        valid: true,
        link_allowed: true,
        issues: Vec::new(),
        notes: notes.iter().cloned().collect(),
    };
    let abi_entries = entries
        .iter()
        .map(|entry| LinkPlanHostFfiAbiEntry {
            symbol: entry.symbol.clone(),
            signature_pattern: entry.signature_pattern.clone(),
            signature_hash: entry.signature_hash.clone(),
            policy: entry.policy.clone(),
            memory_capabilities: entry.memory_capabilities.clone(),
        })
        .collect::<Vec<_>>();
    plan.host_ffi.index_path = Some("nuis.project.host_ffi.index".to_owned());
    plan.host_ffi.symbol_count = entries.len();
    plan.host_ffi.policy_count = entries.len();
    plan.host_ffi.memory_capability_count = 0;
    plan.host_ffi.policy = "signature-whitelist-required".to_owned();
    plan.host_ffi.abi_groups = vec![LinkPlanHostFfiAbiGroup {
        abi,
        symbol_count: entries.len(),
        policy_count: entries.len(),
        memory_capability_count: 0,
        symbols: entries
            .iter()
            .map(|entry| format!("{}:{}", entry.symbol, entry.signature_pattern))
            .collect(),
        entries: abi_entries,
        validation: validation.clone(),
    }];
    plan.host_ffi.entries = entries;
    plan.host_ffi.validation = validation;
}
