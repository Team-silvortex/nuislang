use super::*;
use yir_core::native_resource_session::CHECKED_CONTRACT;

const SOURCE: &str = include_str!(
    "../../../../examples/projects/domains/native_checked_canvas_session_demo/main.ns"
);
const MANIFEST: &str = include_str!(
    "../../../../examples/projects/domains/native_checked_canvas_session_demo/nuis.toml"
);

fn script(output: &Path, width: i64, color: i64, close: i64) -> Vec<String> {
    [
        "--resource-session".to_owned(),
        "canvas".to_owned(),
        "--buffer-words".to_owned(),
        "4278190080,4278190080,4278190080,4278190080,4278190080,4278190080".to_owned(),
        "--open-arg".to_owned(),
        width.to_string(),
        "--event-arg".to_owned(),
        color.to_string(),
        "--close-arg".to_owned(),
        close.to_string(),
        "--resource-frame-fields".to_owned(),
        "width,height".to_owned(),
        "--resource-frame-output".to_owned(),
        output.to_str().unwrap().to_owned(),
    ]
    .to_vec()
}

fn check_failure(output: Output, role: &str, code: i64) {
    assert_eq!(
        output.status.code(),
        Some(1),
        "must return an error, not a signal: {output:?}"
    );
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains(&format!(
            "native resource application error (role={role}, code={code})"
        )),
        "{text}"
    );
    assert!(!text.contains("native_resource_completed=1"), "{text}");
    assert!(!text.contains("native_resource_frame_contract="), "{text}");
    assert!(
        !text.contains(&format!("native_resource_state={role}:")),
        "{text}"
    );
    if role == "open" {
        assert!(!text.contains("native_resource_state="), "{text}");
    }
}

fn check_pixels(output: Output, path: &Path) {
    let output = success(output);
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(
        text.contains("native_resource_scalars=close:3:[3, 2, 1, 1]"),
        "{text}"
    );
    let mut ppm = b"P6\n3 2\n255\n\xff\x00\x00".to_vec();
    ppm.extend([0; 15]);
    assert_eq!(fs::read(path).unwrap(), ppm);
    fs::remove_file(path).unwrap();
}

#[test]
fn checked_canvas_errors_and_success_survive_source_free_delivery() {
    delivery(SOURCE);
}

#[test]
fn direct_resource_error_codes_survive_cache_nsb_and_binary_only_launch() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    fs::write(project.0.join("main.ns"), with_error_codes(1..=7)).unwrap();
    fs::write(project.0.join("nuis.toml"), MANIFEST).unwrap();
    let build = project.0.join("build");
    let output = project.command("build", &project.0, &[build.to_str().unwrap()]);
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("resource return helpers exceed the source closure bound"),
        "{output:?}"
    );
    drop(project);

    // Exercise every callback role without widening the source closure budget.
    delivery(&with_error_codes([1, 4, 7]));
}

fn with_error_codes(codes: impl IntoIterator<Item = i64>) -> String {
    let mut source = SOURCE.replace("mod cpu Main {", "mod cpu Main { fn error_code(valid: bool, code: i64) -> i64 { if valid { return code; } return 99; }");
    for code in codes {
        source = source.replace(
            &format!("return Result.Err({code});"),
            &format!("let saved: Bytes = copy_bytes(pixels); let spare: Bytes = copy_bytes(pixels); return Result.Err(error_code(bytes_len(saved) == 48 || load_at(pixels, 99) > 0, {code}));"),
        );
    }
    assert!(!source.contains("return Result.Err(1)"));
    source
}

fn delivery(source: &str) {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    fs::write(project.0.join("main.ns"), source).unwrap();
    fs::write(project.0.join("nuis.toml"), MANIFEST).unwrap();
    project.build(None);
    let build = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&build.join("nuis.build.manifest.toml")).unwrap();
    let binary = build.join(&report.artifact_binary_name);
    let bundle = fs::read_to_string(build.join("bundle.txt")).unwrap();
    assert!(bundle.contains(CHECKED_CONTRACT));
    assert!(bundle.contains("nuis_checked_resource_"));
    nuisc::aot::resource_session::verify_frame_export_capability(&bundle).unwrap();
    let image = project.0.join("checked.ppm");
    let good = script(&image, 3, 4278190335, 0);
    let refs = good.iter().map(String::as_str).collect::<Vec<_>>();
    check_pixels(project.command("run-artifact", &build, &refs), &image);
    for (width, color, close, role, code) in [
        (0, 1, 0, "open", 1),
        (-1, 1, 0, "open", 1),
        (i64::MIN, 1, 0, "open", 1),
        (7, 1, 0, "open", 2),
        (i64::MAX, 1, 0, "open", 2),
        (4, 1, 0, "open", 3),
        (3, -1, 0, "event", 4),
        (3, 4294967296, 0, "event", 4),
        (3, 1, -1, "close", 7),
    ] {
        let args = script(&image, width, color, close);
        check_failure(
            Command::new(&binary).args(&args).output().unwrap(),
            role,
            code,
        );
        check_failure(
            project.command(
                "run-artifact",
                &build,
                &args.iter().map(String::as_str).collect::<Vec<_>>(),
            ),
            role,
            code,
        );
        assert!(!image.exists());
    }
    assert!(project.build(None).contains("compile_cache: hit"));
    check_pixels(project.command("run-artifact", &build, &refs), &image);
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    let bare = standalone.join("checked-app");
    fs::copy(build.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::copy(&binary, &bare).unwrap();
    fs::remove_dir_all(&build).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    check_pixels(
        Command::new(&bare)
            .args(&good)
            .env("PATH", "")
            .current_dir(&standalone)
            .output()
            .unwrap(),
        &image,
    );
    for (width, color, close, role, code) in [
        (0, 1, 0, "open", 1),
        (7, 1, 0, "open", 2),
        (4, 1, 0, "open", 3),
        (3, -1, 0, "event", 4),
        (3, 1, -1, "close", 7),
    ] {
        check_failure(
            Command::new(&bare)
                .args(script(&image, width, color, close))
                .env("PATH", "")
                .output()
                .unwrap(),
            role,
            code,
        );
        assert!(!image.exists());
    }
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    check_pixels(project.command("run-artifact", &relocated, &refs), &image);
    let bad = script(&image, 0, 1, 0);
    check_failure(
        project.command(
            "run-artifact",
            &relocated,
            &bad.iter().map(String::as_str).collect::<Vec<_>>(),
        ),
        "open",
        1,
    );
    assert!(!image.exists());
}
