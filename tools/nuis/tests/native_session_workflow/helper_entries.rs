use super::*;

fn fanout_source() -> String {
    let base = include_str!("../../../nuisc/tests/native_application_bridge/helper_entries.ns")
        .replace(
            "if skip { return value; }",
            "if skip { return fork18(value); }",
        )
        .replace("return pair(value);", "return fork19(value);");
    let mut helpers = "@noinline fn fork0(value: i64) -> i64 { return value; }\n".to_owned();
    for level in 1..=19 {
        let child = level - 1;
        helpers.push_str(&format!(
            "@noinline fn fork{level}(value: i64) -> i64 {{
                let left = fork{child}(value);
                let right = fork{child}(value + 1);
                return left + right;
            }}\n"
        ));
    }
    base.replace("fn main()", &format!("{helpers} fn main()"))
}

const SCRIPT: &[&str] = &[
    "--native-session",
    "counter",
    "--open-args",
    "10,true",
    "--event-args",
    "10,true",
    "--close-args",
    "10,true",
];

fn verify_runs(project: &Project, output: &Path) {
    let run = success(project.command("run-artifact", output, SCRIPT));
    assert!(run.stdout.is_empty());
    let actual = states(&run);
    assert_eq!(actual.len(), 3);
    for (state, value) in actual.iter().zip([4980736, 9961472, 14942208]) {
        assert!(state.contains(&format!("value: {value}")), "{state}");
    }
    assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    // fork19 expands to 2^20 - 1 entries, exceeding the shared default once the
    // selected wrapper and lifecycle root are included, despite containing no loop.
    let mut over = SCRIPT.to_vec();
    over[3] = "10,false";
    rejected_before_open(project.command("run-artifact", output, &over));
    over[3] = "10,true";
    over[5] = "10,false";
    let rejected = project.command("run-artifact", output, &over);
    assert!(!rejected.status.success());
    assert_eq!(states(&rejected), actual[..1]);
    assert!(!String::from_utf8_lossy(&rejected.stderr).contains("native_session_completed=1"));
}

#[test]
fn loop_free_helper_fanout_is_bounded_through_build_cache_and_standalone() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new(&fanout_source());
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm =
        fs::read_to_string(output.join(format!("{}.ll", report.artifact_binary_name))).unwrap();
    assert_eq!(
        llvm.matches("store i64 1048576, ptr %nuis_helper_entries")
            .count(),
        3
    );
    assert!(llvm.contains("native_helper_entry_rejected"));
    assert!(!llvm.contains("native_loop_work_rejected"));
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
