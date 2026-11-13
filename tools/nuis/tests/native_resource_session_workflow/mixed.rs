use super::*;
use yir_core::{
    native_resource_session::{MIXED_CONTRACT, MIXED_FRAME_EXPORT_CONTRACT},
    Value,
};

const SOURCE: &str =
    include_str!("../../../../examples/projects/domains/native_mixed_canvas_session_demo/main.ns");
const MANIFEST: &str = include_str!(
    "../../../../examples/projects/domains/native_mixed_canvas_session_demo/nuis.toml"
);
const BLACK: i64 = 0xff000000;
const COLORS: [i64; 7] = [
    0xff0000ff, 0xff00ff00, 0xffff0000, 0xff00ffff, 0xffff00ff, 0xffffff00, 0xffffffff,
];

fn script(path: &Path) -> Vec<String> {
    let mut args = vec![
        "--resource-session".to_owned(),
        "canvas".to_owned(),
        "--buffer-words".to_owned(),
        vec![BLACK.to_string(); 6].join(","),
        "--open-arg".to_owned(),
        "3".to_owned(),
        "--close-arg".to_owned(),
        "0".to_owned(),
        "--resource-frame-fields".to_owned(),
        "width,height".to_owned(),
        "--resource-frame-output".to_owned(),
        path.to_str().unwrap().to_owned(),
    ];
    for color in COLORS {
        args.extend(["--event-arg".to_owned(), color.to_string()]);
    }
    args
}

fn reference(name: &str, pixels: &[i64], scalars: &[i64], arg: i64) -> (Vec<i64>, Vec<i64>) {
    let stores = pixels
        .iter()
        .enumerate()
        .map(|(i, v)| format!("store_at(buffer, {i}, {v});"))
        .collect::<String>();
    let mut args = vec!["buffer".to_owned()];
    args.extend(scalars.iter().map(ToString::to_string));
    args.push(arg.to_string());
    let main = format!("fn main() -> i64 {{ let buffer: ref Buffer = alloc_buffer({}, 0); {stores} let state: Canvas = {name}({}); let result: i64 = state.events; drop_bytes(state.pixels); free(buffer); return result; }}", pixels.len(), args.join(","));
    let compiled =
        nuisc::pipeline::compile_source(&SOURCE.replace("fn main() -> i64 { return 0; }", &main))
            .unwrap();
    let trace = yir_runtime_host::execute_module_source_with_registry(
        &nuisc::render::render_yir(&compiled.yir),
        &yir_verify::default_registry(),
    )
    .unwrap();
    let records = compiled
        .yir
        .nodes
        .iter()
        .filter(|n| {
            (n.op.instruction == "struct" && n.op.args[0] == "Canvas")
                || (n.op.instruction == "call_owned_struct" && n.op.args[0] == name)
        })
        .filter_map(|n| trace.values.get(&n.name))
        .collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let Value::Struct(record) = records[0] else {
        panic!("missing Canvas reference State");
    };
    let field = |name: &str| &record.fields.iter().find(|(key, _)| key == name).unwrap().1;
    let scalar = |name: &str| match field(name) {
        Value::Int(value) => *value,
        other => panic!("not i64: {other:?}"),
    };
    let Value::OwnedBytes(pixels) = field("pixels") else {
        panic!("missing owned pixels");
    };
    (
        pixels.clone(),
        ["width", "height", "events", "cursor"].map(scalar).to_vec(),
    )
}

fn observe(output: Output, pixels: &[String], scalars: &[String]) {
    let output = success(output);
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(text.contains(MIXED_FRAME_EXPORT_CONTRACT), "{text}");
    let controls = text
        .lines()
        .filter_map(|line| line.strip_prefix("nuis: native_resource_scalars="))
        .collect::<Vec<_>>();
    assert_eq!(controls, scalars);
    assert_eq!(states(output), pixels);
}

#[test]
fn mixed_canvas_nuis_owned_extent_control_and_source_free_artifact() {
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
    let output = project.0.join("mixed.ppm");
    let args = script(&output);
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    let mut pixels = vec![BLACK; 6];
    let mut scalars = vec![];
    let mut expected_pixels = Vec::new();
    let mut expected_scalars = Vec::new();
    let mut calls = vec![("open", "canvas_open", 3)];
    calls.extend(COLORS.map(|color| ("event", "canvas_event", color)));
    calls.push(("close", "canvas_close", 0));
    for (i, (role, function, arg)) in calls.into_iter().enumerate() {
        (pixels, scalars) = reference(function, &pixels, &scalars, arg);
        expected_pixels.push(format!("{role}:{}:{pixels:?}", i + 1));
        expected_scalars.push(format!("{role}:{}:{scalars:?}", i + 1));
    }
    assert_eq!(scalars, [3, 2, 7, 1]);
    assert_eq!(
        pixels,
        [COLORS[6], COLORS[1], COLORS[2], COLORS[3], COLORS[4], COLORS[5]]
    );
    let ppm =
        b"P6\n3 2\n255\n\xff\xff\xff\x00\xff\x00\x00\x00\xff\xff\xff\x00\xff\x00\xff\x00\xff\xff";
    observe(
        project.command("run-artifact", &build, &refs),
        &expected_pixels,
        &expected_scalars,
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    rejected(project.command("run-artifact", &build, &refs));
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    for (key, value) in [
        ("--resource-frame-fields", "height,unknown"),
        ("--resource-frame-fields", "width,pixels"),
        ("--resource-frame-fields", "width,width"),
        ("--close-arg", "+0"),
    ] {
        let mut bad = args.clone();
        let at = bad.iter().position(|s| s == key).unwrap();
        bad[at + 1] = value.to_owned();
        rejected(Command::new(&binary).args(&bad).output().unwrap());
        rejected(project.command(
            "run-artifact",
            &build,
            &bad.iter().map(String::as_str).collect::<Vec<_>>(),
        ));
        assert!(!output.exists());
    }
    let mut override_dimensions = args.clone();
    let at = override_dimensions
        .iter()
        .position(|s| s == "--resource-frame-fields")
        .unwrap();
    override_dimensions[at] = "--resource-frame".to_owned();
    override_dimensions[at + 1] = "3x2".to_owned();
    rejected(
        Command::new(&binary)
            .args(&override_dimensions)
            .output()
            .unwrap(),
    );
    assert!(!output.exists());

    // The same binary must derive a different image shape from Nuis State, not
    // from the export flags or host-special-cased width/height values.
    let mut reshaped = args.clone();
    let width = reshaped.iter().position(|s| s == "--open-arg").unwrap() + 1;
    reshaped[width] = "2".to_owned();
    let result = success(Command::new(&binary).args(&reshaped).output().unwrap());
    assert!(String::from_utf8_lossy(&result.stderr)
        .contains("native_resource_scalars=close:9:[2, 3, 7, 1]"));
    assert_eq!(&fs::read(&output).unwrap()[..11], b"P6\n2 3\n255\n");
    assert_eq!(&fs::read(&output).unwrap()[11..], &ppm[11..]);
    fs::remove_file(&output).unwrap();
    for bad_width in ["4", "0"] {
        reshaped[width] = bad_width.to_owned();
        let result = Command::new(&binary).args(&reshaped).output().unwrap();
        assert!(!result.status.success());
        assert!(!String::from_utf8_lossy(&result.stderr).contains("native_resource_completed=1"));
        assert!(!output.exists());
    }
    let mut invalid_color = args.clone();
    invalid_color.extend(["--event-arg".to_owned(), "-1".to_owned()]);
    assert!(!Command::new(&binary)
        .args(invalid_color)
        .output()
        .unwrap()
        .status
        .success());
    assert!(!output.exists());
    let bundle = fs::read_to_string(build.join("bundle.txt")).unwrap();
    assert!(bundle.contains(MIXED_CONTRACT));
    nuisc::aot::resource_session::verify_frame_export_capability(&bundle).unwrap();
    assert!(
        nuisc::aot::resource_session::verify_frame_export_capability(&bundle.replace(
            MIXED_FRAME_EXPORT_CONTRACT,
            yir_core::native_resource_session::FRAME_EXPORT_CONTRACT
        ))
        .is_err()
    );
    assert!(project.build(None).contains("compile_cache: hit"));
    observe(
        project.command("run-artifact", &build, &refs),
        &expected_pixels,
        &expected_scalars,
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    let bare = standalone.join("mixed-app");
    fs::copy(build.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::copy(&binary, &bare).unwrap();
    fs::remove_dir_all(&build).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    observe(
        Command::new(&bare)
            .args(&args)
            .env("PATH", "")
            .current_dir(&standalone)
            .output()
            .unwrap(),
        &expected_pixels,
        &expected_scalars,
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
    fs::remove_file(&output).unwrap();
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    observe(
        project.command("run-artifact", &relocated, &refs),
        &expected_pixels,
        &expected_scalars,
    );
    assert_eq!(fs::read(&output).unwrap(), ppm);
}
