use super::*;
use crate::main_test_support::empty_link_plan;

#[test]
fn registry_is_deterministic_and_conformant() {
    let validation = executable_finalizer_registry_validation();

    assert!(validation.valid, "{:?}", validation.issues);
    assert_eq!(validation.contract, EXECUTABLE_FINALIZER_CONTRACT);
    assert_eq!(validation.registration_count, 6);
    assert!(validation.registry_hash.starts_with("0x"));
}

#[test]
fn registry_selects_ready_mach_o_arm64_provider_after_alias_normalization() {
    let mut plan = empty_link_plan();
    plan.cpu_target.machine_arch = "arm64".to_owned();
    plan.cpu_target.machine_os = "darwin".to_owned();
    plan.cpu_target.object_format = "macho".to_owned();

    let selection = select_executable_finalizer(&plan).unwrap();

    assert_eq!(selection.target_key, "aarch64-macos-mach-o");
    assert_eq!(
        selection.provider_id(),
        "nsld.finalizer.mach-o.arm64.host-command-shell-v1"
    );
    assert!(selection.ready());
    assert!(selection.requires_host_driver());
}

#[test]
fn registry_prefers_internal_artifact_image_for_native_cpu_llvm() {
    let mut plan = empty_link_plan();
    plan.packaging_mode = "native-cpu-llvm".to_owned();

    let selection = select_executable_finalizer(&plan).unwrap();

    assert_eq!(selection.target_key, "aarch64-macos-mach-o");
    assert_eq!(
        selection.provider_id(),
        "nsld.finalizer.mach-o.arm64.artifact-image-v1"
    );
    assert_eq!(selection.input_kind(), "compiled-artifact-native-handoff");
    assert!(selection.ready());
    assert!(!selection.requires_host_driver());
    assert!(selection.supports_private_image_publication());
    assert!(selection.private_image_publication_ready());
    assert!(!selection.supports_loader_probe());
    assert!(!selection.loader_probe_ready());
    assert_eq!(
        selection.private_image_publication_capability(),
        Some(MACHO_ARM64_PRIVATE_IMAGE_PUBLICATION_CAPABILITY)
    );
}

#[test]
fn registry_selects_ready_internal_elf_amd64_artifact_provider() {
    let mut plan = empty_link_plan();
    plan.packaging_mode = "native-cpu-llvm".to_owned();
    plan.cpu_target.machine_arch = "amd64".to_owned();
    plan.cpu_target.machine_os = "linux-gnu".to_owned();
    plan.cpu_target.object_format = "elf".to_owned();
    plan.cpu_target.calling_abi = "sysv64".to_owned();

    let selection = select_executable_finalizer(&plan).unwrap();

    assert_eq!(selection.target_key, "x86_64-linux-elf");
    assert_eq!(
        selection.provider_id(),
        "nsld.finalizer.elf.amd64.artifact-image-v1"
    );
    assert_eq!(selection.input_kind(), "compiled-artifact-native-handoff");
    assert_eq!(
        selection.execution_kind(),
        "registered-nsld-artifact-image-writer"
    );
    assert!(selection.ready());
    assert!(!selection.requires_host_driver());
    assert!(!selection.supports_private_image_publication());
    assert!(selection.supports_loader_probe());
    assert!(selection.loader_probe_ready());
    assert_eq!(
        selection.loader_probe_capability(),
        Some(ELF_AMD64_REGISTERED_LOADER_PROBE_CAPABILITY)
    );
    assert_eq!(
        selected_loader_probe_capability(&plan).unwrap(),
        Some(ELF_AMD64_REGISTERED_LOADER_PROBE_CAPABILITY)
    );
}

#[test]
fn registry_keeps_elf_and_pe_coff_as_explicit_open_targets() {
    let mut elf = empty_link_plan();
    elf.cpu_target.machine_arch = "riscv64".to_owned();
    elf.cpu_target.machine_os = "linux-gnu".to_owned();
    elf.cpu_target.object_format = "elf".to_owned();
    let mut pe = empty_link_plan();
    pe.cpu_target.machine_arch = "amd64".to_owned();
    pe.cpu_target.machine_os = "win64".to_owned();
    pe.cpu_target.object_format = "pe/coff".to_owned();

    let elf = select_executable_finalizer(&elf).unwrap();
    let pe = select_executable_finalizer(&pe).unwrap();

    assert_eq!(elf.provider_id(), "nsld.finalizer.elf.registered-v1");
    assert_eq!(pe.provider_id(), "nsld.finalizer.pe-coff.registered-v1");
    assert_eq!(elf.provider_status(), "registered-not-implemented");
    assert_eq!(pe.provider_status(), "registered-not-implemented");
    assert!(!elf.ready());
    assert!(!pe.ready());
    assert!(!elf.supports_private_image_publication());
    assert!(!pe.supports_private_image_publication());
    assert!(!elf.private_image_publication_ready());
    assert!(!pe.private_image_publication_ready());
    assert!(!elf.supports_loader_probe());
    assert!(!pe.supports_loader_probe());
    assert!(!elf.loader_probe_ready());
    assert!(!pe.loader_probe_ready());
}

#[test]
fn provider_without_registered_loader_probe_fails_closed() {
    let plan = empty_link_plan();
    let error = invoke_registered_loader_probe(&plan, Path::new("."), false).unwrap_err();

    assert!(error.contains("has no ready loader-probe capability"));
    assert!(error.contains("nsld.finalizer.mach-o.arm64.host-command-shell-v1"));
}

#[test]
fn command_planning_is_format_independent_after_provider_selection() {
    for (machine_os, object_format, expected_provider) in [
        (
            "macos",
            "mach-o",
            "nsld.finalizer.mach-o.arm64.host-command-shell-v1",
        ),
        ("linux", "elf", "nsld.finalizer.elf.registered-v1"),
        ("windows", "pe/coff", "nsld.finalizer.pe-coff.registered-v1"),
    ] {
        let mut plan = empty_link_plan();
        plan.cpu_target.machine_os = machine_os.to_owned();
        plan.cpu_target.object_format = object_format.to_owned();
        let selection = select_executable_finalizer(&plan).unwrap();
        let args = selection.command_args(&ExecutableFinalizerCommandContext {
            driver: "registered-driver",
            target_triple: "registered-target",
            native_object_path: "input.native-object",
            compiled_artifact_path: "input.compiled-artifact",
            output_path: "output.executable",
        });

        assert_eq!(selection.provider_id(), expected_provider);
        assert_eq!(args[0], "registered-driver");
        assert_eq!(args[1..3], ["-target", "registered-target"]);
        assert_eq!(args[3], "input.native-object");
        assert_eq!(args[4..], ["-o", "output.executable"]);
    }
}

#[test]
fn registry_rejects_unregistered_object_format() {
    let mut plan = empty_link_plan();
    plan.cpu_target.object_format = "wasm".to_owned();

    let error = select_executable_finalizer(&plan)
        .err()
        .expect("unregistered object format must fail closed");

    assert!(error.contains("no executable finalizer provider registered"));
    assert!(error.contains("aarch64-macos-wasm"));
}

#[test]
fn ready_host_provider_requires_the_resolved_driver_authority() {
    let plan = empty_link_plan();
    let error = invoke_registered_finalizer(
        &plan,
        &["clang".to_owned(), "-o".to_owned(), "output".to_owned()],
        None,
        Path::new("output"),
    )
    .unwrap_err();

    assert!(error.contains("requires a verified host driver path"));
    assert!(error.contains("nsld.finalizer.mach-o.arm64.host-command-shell-v1"));
}
