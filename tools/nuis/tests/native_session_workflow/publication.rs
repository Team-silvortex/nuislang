#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]

use super::*;

#[test]
fn native_cache_and_repeated_materialization_publish_fresh_file_identities() {
    use std::os::unix::fs::MetadataExt;
    let project = Project::new(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let binary_name = report.artifact_binary_name;
    let binary = output.join(&binary_name);
    let bytes = fs::read(&binary).unwrap();
    let expected = states(&success(project.command("run-artifact", &output, SCRIPT)));
    assert_eq!(expected.len(), 5);
    let previous = project.0.join("previous-build-image");
    fs::hard_link(&binary, &previous).unwrap();
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_ne!(
        fs::metadata(&binary).unwrap().ino(),
        fs::metadata(&previous).unwrap().ino()
    );
    assert_eq!(fs::read(&previous).unwrap(), bytes);
    assert_eq!(
        states(&success(project.command("run-artifact", &output, SCRIPT))),
        expected
    );
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    let restored = project.0.join("restored");
    for cycle in 0..4 {
        let previous = project.0.join(format!("previous-restored-image-{cycle}"));
        if cycle != 0 {
            fs::hard_link(restored.join(&binary_name), &previous).unwrap();
        }
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        let binary = restored.join(&binary_name);
        assert_eq!(fs::read(&binary).unwrap(), bytes);
        if cycle != 0 {
            assert_ne!(
                fs::metadata(&binary).unwrap().ino(),
                fs::metadata(&previous).unwrap().ino()
            );
            assert_eq!(fs::read(previous).unwrap(), bytes);
        }
        let run = success(project.command("run-artifact", &restored, SCRIPT));
        assert_eq!(
            states(&run),
            expected,
            "source-free publication cycle {cycle}"
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join("nuis.toml").exists());
    for entry in fs::read_dir(&restored).unwrap() {
        assert!(!entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".nuis-publish-"));
    }
    let binary = restored.join(binary_name);
    let referent = project.0.join("must-not-overwrite");
    fs::write(&referent, b"protected data").unwrap();
    fs::remove_file(&binary).unwrap();
    std::os::unix::fs::symlink(&referent, &binary).unwrap();
    let rejected = project.command(
        "materialize-artifact",
        &artifact,
        &[restored.to_str().unwrap()],
    );
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("non-regular destination"));
    assert_eq!(fs::read(referent).unwrap(), b"protected data");
}
