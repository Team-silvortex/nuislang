use super::test_support::write_temp_project_fixture;
use super::*;
use std::fs;
use std::path::Path;

fn load_pixelmagic_project(name: &str) -> (std::path::PathBuf, LoadedProject) {
    let root = write_temp_project_fixture(
        name,
        r#"
name = "galaxy-lock"
entry = "main.ns"
galaxy = ["pixelmagic=workspace"]
"#,
        r#"
mod cpu Main {
  fn main() -> i64 {
    return 1;
  }
}
"#,
        vec![],
    );
    let project = load_project(&root).expect("load project");
    (root, project)
}

#[test]
fn renders_portable_hash_bound_dependency_closure() {
    let (_root, project) = load_pixelmagic_project("galaxy_resolution_lock");
    let first = render_project_galaxy_resolution_lock(&project).expect("render lock");
    let second = render_project_galaxy_resolution_lock(&project).expect("render lock again");

    assert_eq!(first, second);
    assert_eq!(first.summary.schema, PROJECT_GALAXY_RESOLUTION_LOCK_SCHEMA);
    assert_eq!(first.summary.digest_contract, "sha256");
    assert_eq!(first.summary.dependencies, 3);
    assert_eq!(first.summary.library_modules, 28);
    assert_eq!(first.summary.selected_library_modules, 28);
    assert!(first.summary.source_modules >= 19);
    assert!(first.source.contains("name = \"pixelmagic\""));
    assert!(first.source.contains("depends_on = [\"core\", \"std\"]"));
    assert!(first
        .source
        .contains("library_import_policy = \"project-auto\""));
    assert!(first.source.contains("selection=auto-injected"));
    assert!(first.source.contains("sha256:"));
    assert!(!first.source.contains("bundle = "));
    for dependency in &project.resolved_galaxies {
        assert!(!first
            .source
            .contains(&dependency.module_dir.display().to_string()));
    }

    let verified = verify_project_galaxy_resolution_lock(
        &project,
        &first.source,
        Path::new("nuis.project.galaxy.lock"),
    )
    .expect("verify canonical lock");
    assert_eq!(verified, first.summary);
}

#[test]
fn rejects_payload_and_resolved_closure_drift() {
    let (_root, mut project) = load_pixelmagic_project("galaxy_resolution_lock_drift");
    let rendered = render_project_galaxy_resolution_lock(&project).expect("render lock");
    let tampered = rendered.source.replacen(
        "package_id = \"nuis.core\"",
        "package_id = \"nuis.core.drift\"",
        1,
    );
    let integrity_error = verify_project_galaxy_resolution_lock_source(
        &tampered,
        Path::new("nuis.project.galaxy.lock"),
    )
    .expect_err("tampered payload must fail");
    assert!(integrity_error.contains("payload hash mismatch"));

    project
        .resolved_galaxies
        .iter_mut()
        .find(|dependency| dependency.name == "core")
        .expect("resolved core")
        .package_id = "nuis.core.changed".to_owned();
    let closure_error = verify_project_galaxy_resolution_lock(
        &project,
        &rendered.source,
        Path::new("nuis.project.galaxy.lock"),
    )
    .expect_err("changed closure must fail");
    assert!(closure_error.contains("does not reproduce the current project dependency closure"));
}

#[test]
fn freezes_content_identity_at_resolution_and_rejects_later_source_drift() {
    let (root, mut project) = load_pixelmagic_project("galaxy_resolution_content_identity");
    let before = render_project_galaxy_resolution_lock(&project).expect("render lock");
    let dependency = project
        .resolved_galaxies
        .iter_mut()
        .find(|dependency| dependency.name == "core")
        .expect("resolved core");
    let identity = dependency.library_content_identities[0].clone();
    let drifted_path = root.join("drifted-library.ns");
    fs::write(&drifted_path, "mod cpu Drifted {}\n").unwrap();

    let error = crate::stdlib_registry::read_verified_galaxy_text(&drifted_path, &identity)
        .expect_err("changed source must not satisfy resolved identity");
    assert!(error.contains("drifted after resolution"));

    dependency.resolved_library_paths[0] = drifted_path;
    let after = render_project_galaxy_resolution_lock(&project).expect("render frozen lock");
    assert_eq!(after, before);
}

#[test]
fn project_metadata_writes_verifiable_resolution_lock() {
    let (root, project) = load_pixelmagic_project("galaxy_resolution_lock_metadata");
    let plan = build_project_compilation_plan(&project).expect("build plan");
    let metadata = write_project_metadata(&root.join("build"), &project, &plan)
        .expect("write project metadata");
    let source = fs::read_to_string(&metadata.galaxy_lock_path).expect("read lock");
    let verified = verify_project_galaxy_resolution_lock(
        &project,
        &source,
        Path::new(&metadata.galaxy_lock_path),
    )
    .expect("verify written lock");

    assert_eq!(verified, metadata.galaxy_lock_summary);
    assert!(metadata
        .galaxy_lock_path
        .ends_with("nuis.project.galaxy.lock"));
}
