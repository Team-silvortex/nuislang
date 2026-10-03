#![cfg(all(
    any(target_os = "macos", target_os = "linux"),
    target_pointer_width = "64"
))]

use super::*;

#[allow(dead_code)]
#[path = "../../../nuisc/tests/native_application_bridge/sparse_record_fixture.rs"]
mod fixture;

fn verify(project: &Project, output: &Path) -> Vec<Vec<String>> {
    [0, 1, 2, 4].into_iter().map(|limit| {
        let run = success(project.command("run-artifact", output, &[
            "--native-session", "counter", "--open-args", &format!("1.5,2.5,-17,{limit}"),
            "--event-args", "", "--event-args", "", "--close-args", "",
        ]));
        assert!(run.stdout.is_empty());
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        let (mut value, mut gain, mut tag, mut flag, mut count) = (1.5, 2.5, -17, false, limit);
        let mut extras = vec![27; 55];
        let mut right = "value: 1.5f64, gain: 2.5f32, tag: 7i32, enabled: true";
        let expected = ["open", "event", "event", "close"].into_iter().map(|phase| {
            if phase == "event" {
                extras[0] = 99;
                if count > 0 {
                    let delta = if count == 1 { 0.5 } else { 2.0 };
                    value += delta; gain += delta / 2.0; tag = 17; flag = !flag;
                    count += 1; extras.fill(99);
                    right = "value: 3.5f64, gain: 4.5f32, tag: 9i32, enabled: false";
                }
            }
            let extra = extras.iter().enumerate().map(|(i, word)| format!(", extra{}: {word}", i + 9)).collect::<String>();
            format!("{phase}:State{{left: Leaf{{value: {value}f64, gain: {gain}f32, tag: {tag}i32, enabled: {flag}}}, right: Leaf{{{right}}}, count: {count}{extra}}}")
        }).collect::<Vec<_>>();
        assert_eq!(states(&run), expected);
        expected
    }).collect()
}

#[test]
fn native_literal_snapshots_build_cache_and_restore_all_words_without_sources() {
    let project = Project::new(&fixture::literal_return_source(64));
    project.build(None);
    let output = project.0.join("build");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    let llvm_name = format!("{}.ll", report.artifact_binary_name);
    let llvm = fs::read_to_string(output.join(&llvm_name)).unwrap();
    assert!(llvm.contains("[57 x i64] %arg"));
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let expected = verify(&project, &output);
    assert!(project.build(None).contains("compile_cache: hit"));
    assert_eq!(verify(&project, &output), expected);
    fs::write(output.join(&llvm_name), format!("{llvm}\n")).unwrap();
    rejected_before_open(project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            "1.5,2.5,-17,3",
        ],
    ));
    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    fs::remove_dir_all(output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    success(project.command("verify-artifact", &artifact, &[]));
    let restored = project.0.join("restored");
    for cycle in 0..2 {
        success(project.command(
            "materialize-artifact",
            &artifact,
            &[restored.to_str().unwrap()],
        ));
        assert_eq!(fs::read_to_string(restored.join(&llvm_name)).unwrap(), llvm);
        assert_eq!(
            verify(&project, &restored),
            expected,
            "literal snapshot restore {cycle}"
        );
    }
}

#[test]
fn native_literal_snapshots_keep_selected_checked_work_before_publication() {
    let source = fixture::literal_return_source(64).replace(
        "let enabled = selected.enabled;",
        "let checked = 1 / (limit - 3); let enabled = selected.enabled;",
    );
    let project = Project::new(&source);
    project.build(None);
    let output = project.0.join("build");
    let zero = success(project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            "1.5,2.5,-17,0",
            "--event-args",
            "",
            "--close-args",
            "",
        ],
    ));
    assert_eq!(states(&zero).len(), 3);
    let failed = project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "counter",
            "--open-args",
            "1.5,2.5,-17,3",
            "--event-args",
            "",
            "--close-args",
            "",
        ],
    );
    let stderr = String::from_utf8_lossy(&failed.stderr);
    assert!(!failed.status.success(), "{stderr}");
    assert!(
        stderr.contains("native session process failed:"),
        "{stderr}"
    );
    assert!(stderr.contains("no fallback was attempted"), "{stderr}");
    assert!(!stderr.contains("SIGKILL"), "{stderr}");
    assert!(!stderr.contains("native_session_completed=1"), "{stderr}");
    let observed = states(&failed);
    assert_eq!(observed.len(), 1, "{stderr}");
    assert!(observed[0].starts_with("open:"));
    assert!(failed.stdout.is_empty());
}
