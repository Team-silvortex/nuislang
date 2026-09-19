use super::*;

const SOURCE: &str = include_str!("../../../nuisc/tests/native_application_bridge/loop_work.ns");
const SCRIPT: &[&str] = &[
    "--native-session",
    "counter",
    "--open-args",
    "16,65535,1,false,false",
    "--event-args",
    "16,65535,1,false,false",
    "--event-args",
    "16,65535,1,false,false",
    "--close-args",
    "16,65535,1,false,false",
];

fn verify_runs(project: &Project, output: &Path) {
    let run = success(project.command("run-artifact", output, SCRIPT));
    assert!(run.stdout.is_empty());
    let actual = states(&run);
    assert_eq!(actual.len(), 4);
    // Each callback reserves 16 + 16 * 65535 = 1_048_576 iterations exactly.
    // The outer counter is work, but only the inner work contributes to value.
    for (state, value) in actual.iter().zip([1048560, 2097120, 3145680, 4194240]) {
        assert!(state.contains(&format!("value: {value}")), "{state}");
    }
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    let changed = SCRIPT.join("\n").replacen("16,65535", "17,65535", 1);
    rejected_before_open(project.command(
        "run-artifact",
        output,
        &changed.split('\n').collect::<Vec<_>>(),
    ));
    // A rejected event must not publish a partial state or complete the session.
    let mut over_event = SCRIPT.to_vec();
    over_event[5] = "17,65535,1,false,false";
    let run = project.command("run-artifact", output, &over_event);
    assert!(!run.status.success());
    assert_eq!(states(&run), actual[..1]);
    assert!(!String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
}

#[test]
fn whole_callback_loop_work_survives_build_cache_and_standalone_restoration() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(SOURCE);
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm =
        fs::read_to_string(output.join(format!("{}.ll", report.artifact_binary_name))).unwrap();
    assert_eq!(
        llvm.matches("store i64 1048576, ptr %nuis_loop_work")
            .count(),
        3
    );
    assert!(llvm.contains("native_loop_work_rejected"));
    verify_runs(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    verify_runs(&project, &output);

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[restored.to_str().unwrap()],
    ));
    verify_runs(&project, &restored);
}
