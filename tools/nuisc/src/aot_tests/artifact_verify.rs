use super::*;

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
        "definitely-missing-nuis-project-input",
        "build/out",
    );
    assert!(message.contains("older nuisc metadata format"));
    assert!(
        message.contains("nuisc inspect-project-metadata \"build/out/nuis.build.manifest.toml\"")
    );
    assert!(message.contains("nuisc verify-build-manifest \"build/out/nuis.build.manifest.toml\""));
}
