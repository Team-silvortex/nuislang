use super::*;
#[path = "../../../nuisc/tests/native_application_bridge/state_return_eager_pair_model.rs"]
mod model;

const SOURCE: &str = include_str!(
    "../../../nuisc/tests/native_application_bridge/state_return_direct_eager_pair.ns"
);

fn run(project: &Project, output: &Path, rows: &[[i64; 8]]) -> Output {
    let mut args = vec!["--native-session".to_owned(), "counter".into()];
    for (index, input) in rows.iter().enumerate() {
        args.push(
            if index == 0 {
                "--open-args"
            } else {
                "--event-args"
            }
            .into(),
        );
        args.push(
            input
                .iter()
                .enumerate()
                .map(|(i, word)| {
                    if i < 5 {
                        (*word != 0).to_string()
                    } else {
                        word.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    args.extend(["--close-args".into(), String::new()]);
    project.command(
        "run-artifact",
        output,
        &args.iter().map(String::as_str).collect::<Vec<_>>(),
    )
}

fn image(project: &Project, output: &Path, binary: &[u8], mode: &str, kind: &str, op: &str) {
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, mode);
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    let container = fs::read(output.join("nuis.compiled.artifact")).unwrap();
    let mut rows = vec![[0; 8]];
    rows.extend((0..16).map(|bits| {
        [
            1,
            bits & 1,
            (bits >> 1) & 1,
            (bits >> 2) & 1,
            (bits >> 3) & 1,
            1,
            1,
            1,
        ]
    }));
    let mut control = 0;
    let mut expected = Vec::new();
    let mut trace = String::new();
    for (i, input) in rows.iter().enumerate() {
        let (next, printed) = model::execute(input, control, op).unwrap();
        control = next;
        expected.push(format!(
            "{}:State{{total: 1.5{kind}, control: {control}}}",
            if i == 0 { "open" } else { "event" }
        ));
        trace.extend(printed.iter().map(|word| format!("{word}\n")));
    }
    let mut closed = expected.clone();
    closed.push(format!(
        "close:State{{total: 1.5{kind}, control: {control}}}"
    ));
    let healthy = || {
        let result = success(run(project, output, &rows));
        assert_eq!(states(&result), closed);
        assert_eq!(String::from_utf8(result.stdout).unwrap(), trace);
        assert!(String::from_utf8_lossy(&result.stderr).contains("native_session_completed=1"));
    };
    healthy();
    for bad in [
        [1, 1, 0, 0, 1, 0, 0, 0],
        [1, 0, 0, 0, 1, 0, 0, 0],
        [1, 1, 0, 0, 1, 1, 1, 0],
    ] {
        let mut failed_rows = rows.clone();
        failed_rows.push(bad);
        let result = run(project, output, &failed_rows);
        assert!(!result.status.success());
        assert_eq!(states(&result), expected);
        let mut prefix = trace.clone();
        prefix.extend(
            model::execute(&bad, control, op)
                .unwrap_err()
                .iter()
                .map(|word| format!("{word}\n")),
        );
        // Trapping CLI children can buffer stdout; native probes check exact traces.
        assert!(prefix.starts_with(&String::from_utf8(result.stdout).unwrap()));
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
        assert!(!stderr.contains("native_session_completed=1"));
    }
    healthy();
    assert_eq!(
        fs::read(output.join(&report.artifact_binary_name)).unwrap(),
        binary
    );
    assert_eq!(
        fs::read(output.join("nuis.compiled.artifact")).unwrap(),
        container
    );
}

#[test]
fn state_return_direct_eager_pair_fresh_cache_and_source_free_delivery() {
    for kind in ["f32", "f64"] {
        for (op, binding) in [("==", "let"), ("!=", "const"), ("^", "let")] {
            let source = SOURCE
                .replace("f64", kind)
                .replace("0.0", "1.5")
                .replace(" == ", &format!(" {op} "))
                .replace("let local: bool", &format!("{binding} local: bool"));
            let (project, mode) = project_for_source_with_i64_prints(&source, 0, 192);
            eprintln!("direct eager pair {kind} {op}: fresh");
            project.build(None);
            let output = project.0.join("build");
            let report =
                nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml"))
                    .unwrap();
            let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
            let artifact_path = output.join("nuis.compiled.artifact");
            let fresh = nuisc::aot::parse_nuis_compiled_artifact(&artifact_path).unwrap();
            image(&project, &output, &binary, &mode, kind, op);
            assert!(project.build(None).contains("compile_cache: hit"));
            let cached = nuisc::aot::parse_nuis_compiled_artifact(&artifact_path).unwrap();
            assert_eq!(fresh.binary_blob, cached.binary_blob);
            assert!(fresh.host_objects == cached.host_objects);
            assert_eq!(fresh.envelope, cached.envelope);
            assert_eq!(fresh.lifecycle, cached.lifecycle);
            assert_eq!(fresh.packaging_mode, cached.packaging_mode);
            image(&project, &output, &binary, &mode, kind, op);
            let artifact = project.0.join("nuis.compiled.artifact");
            fs::copy(&artifact_path, &artifact).unwrap();
            let bytes = fs::read(&artifact).unwrap();
            fs::remove_dir_all(&output).unwrap();
            for name in ["main.ns", "nuis.toml"] {
                fs::remove_file(project.0.join(name)).unwrap();
            }
            fs::remove_dir_all(project.0.join(".nuis")).unwrap();
            success(project.command("verify-artifact", &artifact, &[]));
            for i in 0..2 {
                eprintln!("direct eager pair {kind} {op}: restore {i}");
                let restored = project.0.join(format!("restored-{i}"));
                success(project.command(
                    "materialize-artifact",
                    &artifact,
                    &[restored.to_str().unwrap()],
                ));
                image(&project, &restored, &binary, &mode, kind, op);
                assert_eq!(fs::read(&artifact).unwrap(), bytes);
            }
            for name in ["main.ns", "nuis.toml", ".nuis"] {
                assert!(!project.0.join(name).exists());
            }
        }
    }
}
