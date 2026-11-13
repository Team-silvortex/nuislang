use super::*;

const SOURCE: &str = include_str!(
    "../../../../examples/projects/domains/native_guarded_canvas_session_demo/main.ns"
);
const MANIFEST: &str = include_str!(
    "../../../../examples/projects/domains/native_guarded_canvas_session_demo/nuis.toml"
);

fn check(output: Output, image: &Path) {
    let output = success(output);
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("native_resource_scalars=close:7:[3, 2, 5, 2]"),
        "{text}"
    );
    assert!(text.contains("native_resource_completed=1"), "{text}");
    let mut expected = b"P6\n3 2\n255\n\xff\x00\x00\x00\xff\x00".to_vec();
    expected.extend([0; 12]);
    assert_eq!(fs::read(image).unwrap(), expected);
    fs::remove_file(image).unwrap();
}

#[test]
fn shared_resource_continuation_survives_cache_nsb_and_binary_only_launch() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    fs::write(project.0.join("main.ns"), SOURCE).unwrap();
    fs::write(project.0.join("nuis.toml"), MANIFEST).unwrap();
    project.build(None);
    let build = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&build.join("nuis.build.manifest.toml")).unwrap();
    let binary = build.join(&report.artifact_binary_name);
    let image = project.0.join("guarded.ppm");
    let args = [
        "--resource-session",
        "canvas",
        "--buffer-words",
        "4278190080,4278190080,4278190080,4278190080,4278190080,4278190080",
        "--open-arg",
        "3",
        "--event-arg",
        "0",
        "--event-arg",
        "4278190335",
        "--event-arg",
        "0",
        "--event-arg",
        "4278255360",
        "--event-arg",
        "0",
        "--close-arg",
        "0",
        "--resource-frame-fields",
        "width,height",
        "--resource-frame-output",
        image.to_str().unwrap(),
    ];
    check(project.command("run-artifact", &build, &args), &image);
    assert!(project.build(None).contains("compile_cache: hit"));
    check(project.command("run-artifact", &build, &args), &image);
    let artifact = project.0.join("nuis.compiled.artifact");
    let bare = project.0.join("guarded-app");
    fs::copy(build.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::copy(binary, &bare).unwrap();
    fs::remove_dir_all(&build).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    check(
        Command::new(&bare)
            .args(args)
            .env("PATH", "")
            .current_dir(&project.0)
            .output()
            .unwrap(),
        &image,
    );
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    check(project.command("run-artifact", &relocated, &args), &image);
}
