#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]
use super::*;
use nuisc::aot::native_session::{build_policy, literal_print_packaging_mode};

#[allow(dead_code)]
#[path = "../../../nuisc/src/lowering/buffer_loop_outline/conditional_returns_effect_typed_return_fixtures.rs"]
mod fixtures;
const SCRIPT: &[&str] = &[
    "--native-session",
    "counter",
    "--open-args",
    "true,true,false,false,10,-17,2,2",
    "--event-args",
    "",
    "--event-args",
    "",
    "--close-args",
    "",
];

fn project() -> (Project, Vec<String>, String) {
    let project = Project::new(&fixtures::source("i32", "complete", false));
    let manifest = project.0.join("nuis.toml");
    let text = fs::read_to_string(&manifest).unwrap().replace(
        "packaging_mode = \"native-session-aot-bundle:counter\"\n",
        "",
    );
    fs::write(&manifest, &text).unwrap();
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    // Test-only explicit grants. The build frontdoor never discovers or grants sites.
    let sites = module
        .nodes
        .iter()
        .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
        .map(|n| n.name.clone())
        .collect::<Vec<_>>();
    assert!(!sites.is_empty());
    let mode = literal_print_packaging_mode("counter", sites.clone(), 0, 100).unwrap();
    fs::write(manifest, format!("{text}packaging_mode = \"{mode}\"\n")).unwrap();
    (project, sites, mode)
}

fn check_run(project: &Project, output: &Path) {
    for (outer, gate, early, prints, result) in [
        (true, true, false, "99\n70\n80\n", 10),
        (true, false, false, "99\n71\n80\n", -17),
        (true, false, true, "99\n72\n", -17),
        (false, true, false, "99\n77\n", -17),
    ] {
        let input = format!("{outer},{gate},false,{early},10,-17,2,2");
        let mut args = SCRIPT.to_vec();
        args[3] = &input;
        let run = success(project.command("run-artifact", output, &args));
        assert_eq!(
            String::from_utf8(run.stdout.clone()).unwrap(),
            prints.repeat(4)
        );
        let expected = ["open", "event", "event", "close"].map(|phase| format!(
            "{phase}:State{{outer: {outer}, gate: {gate}, nested: false, early: {early}, value: 10i32, fallback: -17i32, left: 2, right: 2, result: {result}i32}}"));
        assert_eq!(states(&run), expected);
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
    }
}

#[test]
fn native_literal_print_build_cache_and_source_free_materialization() {
    let (project, _, mode) = project();
    project.build(None);
    let output = project.0.join("build");
    let manifest = output.join("nuis.build.manifest.toml");
    let report = nuisc::aot::verify_build_manifest(&manifest).unwrap();
    assert_eq!(report.packaging_mode, mode);
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    assert_eq!(llvm.matches("store i64 0, ptr %nuis_loop_work").count(), 3);
    assert_eq!(
        llvm.matches("store i64 100, ptr %nuis_helper_entries")
            .count(),
        3
    );
    let bundle = fs::read_to_string(output.join("bundle.txt")).unwrap();
    for (key, value) in build_policy(&mode).unwrap().unwrap().bundle_claims() {
        assert!(bundle.lines().any(|line| line == format!("{key}={value}")));
    }
    check_run(&project, &output);
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    check_run(&project, &output);
    for (name, replacement) in [
        (
            "bundle.txt".to_owned(),
            bundle.replace(
                "native_session_helper_entry_limit=100",
                "native_session_helper_entry_limit=101",
            ),
        ),
        (
            llvm_name.clone(),
            llvm.replace(
                "store i64 100, ptr %nuis_helper_entries",
                "store i64 101, ptr %nuis_helper_entries",
            ),
        ),
    ] {
        let path = output.join(&name);
        let original = fs::read(&path).unwrap();
        fs::write(&path, replacement).unwrap();
        let rejected = project.command("run-artifact", &output, SCRIPT);
        assert!(rejected.stdout.is_empty(), "must reject before effects");
        rejected_before_open(rejected);
        fs::write(path, original).unwrap();
    }
    let invalid = project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            "true,true,false,invalid,10,-17,2,2",
        ],
    );
    assert!(invalid.stdout.is_empty());
    rejected_before_open(invalid);
    let wrong_session = project.command(
        "run-artifact",
        &output,
        &["--native-session", "other", "--open-args", SCRIPT[3]],
    );
    assert!(wrong_session.stdout.is_empty());
    rejected_before_open(wrong_session);

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let binary = fs::read(output.join(&report.artifact_binary_name)).unwrap();
    fs::remove_dir_all(output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    fs::remove_dir_all(project.0.join(".nuis")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for cycle in 0..3 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        let report =
            nuisc::aot::verify_build_manifest(&restored.join("nuis.build.manifest.toml")).unwrap();
        assert_eq!(report.packaging_mode, mode, "cycle {cycle}");
        assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm);
        assert_eq!(
            fs::read_to_string(restored.join("bundle.txt")).unwrap(),
            bundle
        );
        assert_eq!(
            fs::read(restored.join(&report.artifact_binary_name)).unwrap(),
            binary
        );
        check_run(&project, &restored);
    }
    assert!(!project.0.join("main.ns").exists());
    assert!(!project.0.join("nuis.toml").exists());
    assert!(!project.0.join(".nuis").exists());
}

#[test]
fn native_literal_print_build_profiles_never_inherit_cached_authority() {
    let (project, sites, mode) = project();
    project.build(Some(&mode));
    check_run(&project, &project.0.join("build"));
    let changed = literal_print_packaging_mode("counter", sites.clone(), 1, 100).unwrap();
    let built = project.build(Some(&changed));
    assert!(built.contains("compile_cache: miss"), "{built}");
    assert_eq!(
        nuisc::aot::verify_build_manifest(&project.0.join("build/nuis.build.manifest.toml"))
            .unwrap()
            .packaging_mode,
        changed
    );
    check_run(&project, &project.0.join("build"));
    let exhausted = literal_print_packaging_mode("counter", sites.clone(), 0, 1).unwrap();
    let built = project.build(Some(&exhausted));
    assert!(built.contains("compile_cache: miss"), "{built}");
    let rejected = project.command("run-artifact", &project.0.join("build"), SCRIPT);
    assert!(rejected.stdout.is_empty());
    #[cfg(unix)]
    {
        let stderr = String::from_utf8_lossy(&rejected.stderr);
        assert!(
            stderr.contains("SIGILL") || stderr.contains("SIGTRAP"),
            "{stderr}"
        );
    }
    rejected_before_open(rejected);
    for invalid in [
        "native-session-aot-bundle:counter".to_owned(),
        literal_print_packaging_mode("counter", Vec::<String>::new(), 0, 100).unwrap(),
        literal_print_packaging_mode("counter", sites[1..].to_vec(), 0, 100).unwrap(),
        literal_print_packaging_mode(
            "counter",
            [sites.clone(), vec!["missing".to_owned()]].concat(),
            0,
            100,
        )
        .unwrap(),
    ] {
        let output = project.0.join("build");
        let rejected = project.command(
            "build",
            &project.0,
            &[output.to_str().unwrap(), "--packaging-mode", &invalid],
        );
        assert!(!rejected.status.success());
        assert!(!String::from_utf8_lossy(&rejected.stdout).contains("compile_cache: hit"));
    }
    let cached = project.build(Some(&mode));
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    check_run(&project, &project.0.join("build"));
}
