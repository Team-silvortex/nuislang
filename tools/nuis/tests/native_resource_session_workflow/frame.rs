use super::*;
use yir_core::native_resource_session::FRAME_EXPORT_CONTRACT;

const CANVAS: &str =
    include_str!("../../../../examples/projects/domains/native_canvas_session_demo/main.ns");
const MANIFEST: &str =
    include_str!("../../../../examples/projects/domains/native_canvas_session_demo/nuis.toml");
const INITIAL: &str = "4278190080,4278190080,4278190080,4278190080";
const RED: i64 = 0xff0000ff;
const GREEN: i64 = 0xff00ff00;
const BLUE: i64 = 0xffff0000;
const YELLOW: i64 = 0xff00ffff;

fn script(output: &Path) -> Vec<String> {
    [
        "--resource-session",
        "canvas",
        "--buffer-words",
        INITIAL,
        "--open-arg",
        &RED.to_string(),
        "--event-arg",
        &((1_i64 << 32) + GREEN).to_string(),
        "--event-arg",
        &((2_i64 << 32) + BLUE).to_string(),
        "--event-arg",
        &((1_i64 << 32) + YELLOW).to_string(),
        "--close-arg",
        "0",
        "--resource-frame",
        "2x2",
        "--resource-frame-output",
        output.to_str().unwrap(),
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn set(arguments: &mut [String], key: &str, value: &str) {
    let index = arguments.iter().position(|arg| arg == key).unwrap();
    arguments[index + 1] = value.to_owned();
}

fn reference(name: &str, state: &[i64], argument: i64) -> Vec<i64> {
    let stores = state
        .iter()
        .enumerate()
        .map(|(i, value)| format!("store_at(pixels, {i}, {value});"))
        .collect::<String>();
    let main = format!("fn main() -> i64 {{ let pixels: ref Buffer = alloc_buffer({}, 0); {stores} let result: Bytes = {name}(pixels, {argument}); let size: i64 = bytes_len(result); drop_bytes(result); free(pixels); return size; }}", state.len());
    let compiled =
        nuisc::pipeline::compile_source(&CANVAS.replace("fn main() -> i64 { return 0; }", &main))
            .unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let copies = compiled
        .yir
        .nodes
        .iter()
        .filter(|n| n.op.instruction == "copy_buffer_owned")
        .filter_map(|n| trace.values.get(&n.name))
        .collect::<Vec<_>>();
    assert_eq!(copies.len(), 1);
    match copies[0] {
        yir_core::Value::OwnedBytes(words) => words.clone(),
        value => panic!("expected reference pixels, got {value:?}"),
    }
}

#[test]
fn native_canvas_exact_pixels_preflight_faults_and_source_free_delivery() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    fs::write(project.0.join("main.ns"), CANVAS).unwrap();
    fs::write(project.0.join("nuis.toml"), MANIFEST).unwrap();
    project.build(None);
    let build = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&build.join("nuis.build.manifest.toml")).unwrap();
    let binary = build.join(&report.artifact_binary_name);
    let output = project.0.join("canvas-\u{753b}\u{5e03}.ppm");
    let args = script(&output);
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let result = project.command("run-artifact", &build, &refs);
    let text = String::from_utf8_lossy(&result.stderr);
    assert!(text.contains(FRAME_EXPORT_CONTRACT), "{text}");
    assert!(
        text.contains("native_resource_frame_execution=native-cpu-resource-session"),
        "{text}"
    );

    let mut expected_state = vec![0xff000000; 4];
    let expected = [
        ("open", "canvas_open", RED),
        ("event", "canvas_event", (1_i64 << 32) + GREEN),
        ("event", "canvas_event", (2_i64 << 32) + BLUE),
        ("event", "canvas_event", (1_i64 << 32) + YELLOW),
        ("close", "canvas_close", 0),
    ]
    .into_iter()
    .enumerate()
    .map(|(i, (role, name, arg))| {
        expected_state = reference(name, &expected_state, arg);
        format!("{role}:{}:{expected_state:?}", i + 1)
    })
    .collect::<Vec<_>>();
    assert_eq!(states(result), expected);
    assert_eq!(expected_state, [RED, YELLOW, BLUE, 0xff000000]);
    let ppm = b"P6\n2 2\n255\n\xff\x00\x00\xff\xff\x00\x00\x00\xff\x00\x00\x00";
    assert_eq!(fs::read(&output).unwrap(), ppm);

    // The presentation capability is optional for old resource-only artifacts,
    // but must be unique and exact when claimed; it does not change the ABI.
    let bundle = fs::read_to_string(build.join("bundle.txt")).unwrap();
    let llvm =
        fs::read_to_string(build.join(format!("{}.ll", report.artifact_binary_name))).unwrap();
    let module = yir_syntax::parse_explicit_module(
        &fs::read_to_string(build.join(format!("{}.yir", report.artifact_binary_name))).unwrap(),
    )
    .unwrap();
    let claim = format!("native_resource_frame_contract={FRAME_EXPORT_CONTRACT}\n");
    assert!(bundle.contains(&claim));
    nuisc::aot::resource_session::verify_frame_export_capability(&bundle).unwrap();
    let old = bundle.replace(&claim, "");
    nuisc::aot::resource_session::verify_checkpoint(&module, &report.packaging_mode, &llvm, &old)
        .unwrap();
    assert!(nuisc::aot::resource_session::verify_frame_export_capability(&old).is_err());
    for corrupt in [
        format!("{bundle}{claim}"),
        bundle.replace(FRAME_EXPORT_CONTRACT, "unknown"),
    ] {
        assert!(nuisc::aot::resource_session::verify_frame_export_capability(&corrupt).is_err());
        assert!(nuisc::aot::resource_session::verify_checkpoint(
            &module,
            &report.packaging_mode,
            &llvm,
            &corrupt
        )
        .is_err());
    }

    // Reject a pre-existing destination before open, without clobbering it.
    rejected(project.command("run-artifact", &build, &refs));
    rejected(Command::new(&binary).args(&args).output().unwrap());
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    let mut bad_scripts = Vec::new();
    for (key, value) in [
        ("--resource-frame", "1x4x1"),
        ("--resource-frame", "1x3"),
        ("--resource-frame", "0x4"),
        ("--resource-frame", "01x4"),
        ("--resource-frame", "4097x1"),
        ("--buffer-words", "-1,0,0,0"),
        ("--buffer-words", "4294967296,0,0,0"),
        ("--close-arg", "no"),
        ("--resource-frame-output", ""),
    ] {
        let mut bad = args.clone();
        set(&mut bad, key, value);
        bad_scripts.push(bad);
    }
    for key in ["--resource-frame", "--resource-frame-output"] {
        let mut missing = args.clone();
        let index = missing.iter().position(|arg| arg == key).unwrap();
        missing.drain(index..index + 2);
        bad_scripts.push(missing);
        let mut duplicate = args.clone();
        duplicate.extend(args[index..index + 2].iter().cloned());
        bad_scripts.push(duplicate);
    }
    for pair in [["--export-frame", "old.ppm"], ["--window-session", "other"]] {
        let mut mixed = args.clone();
        mixed.extend(pair.map(str::to_owned));
        bad_scripts.push(mixed);
    }
    for bad in bad_scripts {
        let refs = bad.iter().map(String::as_str).collect::<Vec<_>>();
        rejected(project.command("run-artifact", &build, &refs));
        rejected(Command::new(&binary).args(&bad).output().unwrap());
        assert!(!output.exists());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(project.0.join("missing-target"), &output).unwrap();
        rejected(Command::new(&binary).args(&args).output().unwrap());
        assert!(fs::symlink_metadata(&output)
            .unwrap()
            .file_type()
            .is_symlink());
        fs::remove_file(&output).unwrap();
    }
    for color in [-1_i64, 4294967296] {
        let mut bad = args.clone();
        set(&mut bad, "--open-arg", &color.to_string());
        let result = Command::new(&binary).args(&bad).output().unwrap();
        assert!(result.status.code().is_some_and(|code| code != 0));
        let text = String::from_utf8_lossy(&result.stderr);
        assert!(text.contains("native_resource_state=close:5:"), "{text}");
        assert!(text.contains("unsigned packed RGBA8"), "{text}");
        assert!(!text.contains("native_resource_completed=1"));
        assert!(!output.exists());
    }
    let mut trap = args.clone();
    set(&mut trap, "--event-arg", &(4_i64 << 32).to_string());
    let result = Command::new(&binary).args(&trap).output().unwrap();
    assert!(!result.status.success());
    let text = String::from_utf8_lossy(&result.stderr);
    assert!(text.contains("native_resource_state=open:1:"), "{text}");
    assert!(!text.contains("native_resource_state=close:"));
    assert!(!text.contains("native_resource_completed=1"));
    assert!(!output.exists());

    // The new options must not reduce the existing 256-event script bound.
    let mut maximum = args.clone();
    for _ in 3..256 {
        maximum.extend([
            "--event-arg".to_owned(),
            ((1_i64 << 32) + YELLOW).to_string(),
        ]);
    }
    assert_eq!(
        states(Command::new(&binary).args(&maximum).output().unwrap()).len(),
        258
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    maximum.extend(["--event-arg".to_owned(), "0".to_owned()]);
    rejected(Command::new(&binary).args(&maximum).output().unwrap());
    assert!(!output.exists());

    assert!(project.build(None).contains("compile_cache: hit"));
    assert_eq!(
        states(project.command("run-artifact", &build, &refs)),
        expected
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    let bare = standalone.join("canvas-app");
    fs::copy(build.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::copy(&binary, &bare).unwrap();
    fs::remove_dir_all(&build).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    assert_eq!(
        states(
            Command::new(&bare)
                .args(&args)
                .env("PATH", "")
                .current_dir(&standalone)
                .output()
                .unwrap()
        ),
        expected
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    assert_eq!(
        states(project.command("run-artifact", &relocated, &refs)),
        expected
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
}
