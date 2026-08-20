use super::*;

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn native_compile_embeds_real_relocatable_host_objects() {
    let dir = temp_dir("native_host_object_handoff");
    let input = dir.join("main.ns");
    let first_output = dir.join("first");
    let restored_output = dir.join("restored");
    let repaired_output = dir.join("repaired");
    fs::write(&input, "mod cpu Main { fn main() -> i64 { return 0; } }\n").unwrap();

    crate::run(crate::cli::CommandKind::Compile {
        input: input.clone(),
        output_dir: first_output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("native-cpu-llvm".to_owned()),
    })
    .unwrap();
    assert!(first_output.join("main.host-program.o").is_file());
    assert!(first_output.join("main.host-runtime.o").is_file());

    crate::run(crate::cli::CommandKind::Compile {
        input: input.clone(),
        output_dir: restored_output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("native-cpu-llvm".to_owned()),
    })
    .unwrap();
    assert!(restored_output.join("main.host-program.o").is_file());
    assert!(restored_output.join("main.host-runtime.o").is_file());

    let artifact_path = restored_output.join("nuis.compiled.artifact");
    let artifact = parse_nuis_compiled_artifact(&artifact_path).unwrap();
    assert_eq!(artifact.host_objects.len(), 2);
    assert_eq!(
        artifact
            .host_objects
            .iter()
            .map(|object| object.role.as_str())
            .collect::<Vec<_>>(),
        vec!["program-llvm", "runtime-shim"]
    );
    for object in &artifact.host_objects {
        assert_eq!(&object.bytes[..4], &[0xcf, 0xfa, 0xed, 0xfe]);
        assert_eq!(
            u32::from_le_bytes(object.bytes[12..16].try_into().unwrap()),
            1
        );
    }

    let report = verify_nuis_compiled_artifact(&artifact_path).unwrap();
    assert_eq!(report.host_object_count, 2);
    assert_eq!(
        report.host_object_roles,
        vec!["program-llvm", "runtime-shim"]
    );
    assert_eq!(report.host_object_hashes.len(), 2);
    assert!(report.artifact_section_names.contains(
        &nuis_artifact::protocol::COMPILED_ARTIFACT_SECTION_HOST_OBJECTS_BINARY.to_owned()
    ));

    let manifest_report =
        verify_build_manifest(&restored_output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(manifest_report.compile_cache_status.as_deref(), Some("hit"));

    let cache_key = crate::cache::compute_compile_cache_key(&input, None).unwrap();
    fs::remove_file(
        cache_key
            .root
            .join(&cache_key.key)
            .join("main.host-program.o"),
    )
    .unwrap();
    crate::run(crate::cli::CommandKind::Compile {
        input: input.clone(),
        output_dir: repaired_output.clone(),
        verbose_cache: false,
        cpu_abi: None,
        target: None,
        packaging_mode: Some("native-cpu-llvm".to_owned()),
    })
    .unwrap();
    assert!(repaired_output.join("main.host-program.o").is_file());
    assert!(repaired_output.join("main.host-runtime.o").is_file());
    let repaired_manifest =
        verify_build_manifest(&repaired_output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(
        repaired_manifest.compile_cache_status.as_deref(),
        Some("miss")
    );

    let _ = fs::remove_dir_all(cache_key.root.join(cache_key.key));
    fs::remove_dir_all(dir).unwrap();
}
