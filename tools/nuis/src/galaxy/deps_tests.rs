use super::{
    doctor_project, install_project_deps, lock_project_deps, sync_project_deps, verify_project_lock,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempProject(PathBuf);

impl TempProject {
    fn new(name: &str, galaxy: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("nuis_galaxy_{name}_{nonce}"));
        fs::create_dir_all(&root).unwrap();
        write_manifest(&root, galaxy);
        fs::write(
            root.join("main.ns"),
            "mod cpu Main {\n  fn main() -> i64 {\n    return 0;\n  }\n}\n",
        )
        .unwrap();
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempProject {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn write_manifest(root: &Path, galaxy: &str) {
    fs::write(
        root.join("nuis.toml"),
        format!(
            "name = \"galaxy-lock-test\"\nentry = \"main.ns\"\ngalaxy = [\"{galaxy}=workspace\"]\n"
        ),
    )
    .unwrap();
}

#[test]
fn canonical_root_lock_drives_verification_and_transactional_sync() {
    let project = TempProject::new("canonical_lock_sync", "std");
    let wrote = lock_project_deps(project.path()).unwrap();
    assert_eq!(wrote.summary.dependencies, 2);
    assert_eq!(
        wrote
            .entries
            .iter()
            .filter(|entry| entry.direct)
            .map(|entry| entry.name.as_str())
            .collect::<Vec<_>>(),
        ["std"]
    );
    assert!(wrote
        .entries
        .iter()
        .any(|entry| { entry.name == "core" && !entry.direct && entry.package_id == "nuis.core" }));

    let lock_source = fs::read_to_string(&wrote.path).unwrap();
    assert!(lock_source.contains("lock_schema = \"nuis-galaxy-resolution-lock-v1\""));
    assert!(lock_source.contains("resolution_sha256 = \"sha256:"));
    assert!(!lock_source.contains("bundle = "));
    assert!(!lock_source.contains(&project.path().display().to_string()));

    let verified = verify_project_lock(project.path()).unwrap();
    assert_eq!(verified.summary, wrote.summary);
    assert_eq!(verified.entries, wrote.entries);

    let synced = sync_project_deps(project.path()).unwrap();
    assert_eq!(synced.summary, wrote.summary);
    assert!(synced.root.join("std/workspace/module.toml").is_file());
    assert!(synced
        .root
        .join("std/workspace/lib/language_core.ns")
        .is_file());
    assert!(synced.root.join("core/workspace/module.toml").is_file());
    assert_eq!(
        fs::read_to_string(synced.root.join("nuis.galaxy.lock")).unwrap(),
        lock_source
    );

    let stale = synced.root.join("stale-unlocked-file.txt");
    fs::write(&stale, "must disappear").unwrap();
    sync_project_deps(project.path()).unwrap();
    assert!(!stale.exists());

    let installed = install_project_deps(project.path()).unwrap();
    assert_eq!(installed.lock.summary, wrote.summary);
    assert_eq!(installed.installed.len(), 2);
    assert!(installed.installed.iter().all(|entry| entry
        .project
        .file_name()
        .and_then(|name| name.to_str())
        == Some("module.toml")));
    let doctor = doctor_project(project.path()).unwrap();
    assert_eq!(doctor.lock_status, "ok");
    assert!(doctor
        .dependencies
        .iter()
        .all(|dependency| dependency.source_available
            && dependency.locked
            && dependency.installed));
}

#[test]
fn closure_drift_is_rejected_without_destroying_the_previous_sync() -> Result<(), String> {
    let project = TempProject::new("closure_drift", "std");
    lock_project_deps(project.path()).unwrap();
    let synced = sync_project_deps(project.path()).unwrap();
    let preserved = synced.root.join("std/workspace/module.toml");
    let preserved_source = fs::read_to_string(&preserved).unwrap();

    write_manifest(project.path(), "pixelmagic");
    let error = verify_project_lock(project.path()).unwrap_err();
    assert!(error.contains("does not reproduce the current project dependency closure"));
    let status =
        crate::surface_render::render_project_status_text_summary(project.path())?.join("\n");
    assert!(status.contains("galaxy_lock: invalid"));
    assert!(status.contains("does not reproduce the current project dependency closure"));
    let status_json = crate::surface_render::render_project_status_json(project.path())?;
    assert!(status_json.contains("\"galaxy_lock_status\":\"invalid\""));
    assert!(status_json.contains("does not reproduce the current project dependency closure"));
    let sync_error = sync_project_deps(project.path()).unwrap_err();
    assert!(sync_error.contains("does not reproduce the current project dependency closure"));
    assert_eq!(fs::read_to_string(preserved).unwrap(), preserved_source);
    Ok::<(), String>(())
}
