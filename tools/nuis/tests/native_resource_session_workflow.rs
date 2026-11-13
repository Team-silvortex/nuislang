//! Static Buffer/Bytes packaging and launch without the original source checkout.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const SOURCE: &str = include_str!("../../nuisc/tests/native_resource_session/main.ns");
#[path = "native_resource_session_workflow/checked.rs"]
mod checked;
#[path = "native_resource_session_workflow/early_ok.rs"]
mod early_ok;
#[path = "native_resource_session_workflow/frame.rs"]
mod frame;
#[path = "native_resource_session_workflow/mixed.rs"]
mod mixed;
const SCRIPT: &[&str] = &[
    "--resource-session",
    "bytes",
    "--buffer-words",
    "0,-17,9223372036854775807,-9223372036854775808",
    "--open-arg",
    "10",
    "--event-arg",
    "3",
    "--event-arg",
    "-2",
    "--close-arg",
    "5",
];

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "nuis-resource-workflow-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.ns"), SOURCE).unwrap();
        fs::write(path.join("nuis.toml"), "name = \"resource_workflow\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\npackaging_mode = \"native-resource-session-aot-bundle:bytes\"\nresource_sessions = [\"bytes open=start event=step close=stop min=1 max=4096\", \"other open=start event=step close=stop min=1 max=4096\"]\n").unwrap();
        Self(path)
    }

    fn command(&self, verb: &str, input: &Path, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_nuis"))
            .arg(verb)
            .arg(input)
            .args(args)
            .env("CARGO_BUILD_JOBS", "1")
            .env("CARGO_INCREMENTAL", "0")
            .env_remove("NUIS_TEST_QUIET_SUCCESS_LOGS")
            .output()
            .unwrap()
    }

    fn build(&self, mode: Option<&str>) -> String {
        let output = self.0.join("build");
        let mut args = vec![output.to_str().unwrap()];
        if let Some(mode) = mode {
            args.extend(["--packaging-mode", mode]);
        }
        String::from_utf8(success(self.command("build", &self.0, &args)).stdout).unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}
fn states(output: Output) -> Vec<String> {
    let output = success(output);
    assert!(output.stdout.is_empty());
    let text = String::from_utf8(output.stderr).unwrap();
    assert!(text.contains("native_resource_completed=1"), "{text}");
    text.lines()
        .filter_map(|s| {
            s.strip_prefix("nuis: native_resource_state=")
                .map(str::to_owned)
        })
        .collect()
}
fn rejected(output: Output) {
    assert!(
        output.status.code().is_some_and(|code| code != 0),
        "must reject, not crash or fall back: {output:?}"
    );
    let text = String::from_utf8_lossy(&output.stderr);
    assert!(!text.contains("native_resource_state="), "{text}");
    assert!(!text.contains("native_resource_completed=1"), "{text}");
}

#[test]
fn resource_build_cache_nsb_relocation_and_binary_only_launch() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    project.build(None);
    let output = project.0.join("build");
    let manifest_path = output.join("nuis.build.manifest.toml");
    let report = nuisc::aot::verify_build_manifest(&manifest_path).unwrap();
    assert_eq!(
        report.packaging_mode,
        "native-resource-session-aot-bundle:bytes"
    );
    let stem = report.artifact_binary_name;
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    for claim in [
        "native-resource-llvm-v1",
        "native_resource_session_llvm_hex",
        "nuis-native-resource-session-build-inputs-v1",
    ] {
        assert!(manifest.contains(claim), "{claim}");
    }
    let llvm = fs::read_to_string(output.join(format!("{stem}.ll"))).unwrap();
    let yir = fs::read_to_string(output.join(format!("{stem}.yir"))).unwrap();
    let module = yir_syntax::parse_explicit_module(&yir).unwrap();
    assert_eq!(module.resource_sessions.len(), 2);
    assert_eq!(nuisc::render::render_yir(&module), yir);
    let registration = &module.resource_sessions[0];
    for symbol in registration.symbols() {
        assert!(llvm.contains(&symbol));
    }
    let expected = [
        "open:1:[10, -17, 9223372036854775807, -9223372036854775808]",
        "event:2:[13, -17, 9223372036854775807, -9223372036854775808]",
        "event:3:[11, -17, 9223372036854775807, -9223372036854775808]",
        "close:4:[16, -17, 9223372036854775807, -9223372036854775808]",
    ];
    assert_eq!(
        states(project.command("run-artifact", &output, SCRIPT)),
        expected
    );
    rejected(project.command("run-artifact", &output, &[]));
    rejected(project.command(
        "run-artifact",
        &output,
        &[
            "--native-session",
            "bytes",
            "--open-args",
            "0",
            "--close-args",
            "0",
        ],
    ));
    rejected(project.command(
        "run-artifact",
        &output,
        &[
            "--application-session",
            "bytes",
            "--open-args",
            "0",
            "--close-args",
            "0",
        ],
    ));
    let doctor = success(project.command("artifact-doctor", &output, &["--json"]));
    assert!(String::from_utf8_lossy(&doctor.stdout).contains("run_native_resource_session_script"));

    let bundle = fs::read_to_string(output.join("bundle.txt")).unwrap();
    nuisc::aot::resource_session::verify_checkpoint(
        &module,
        &report.packaging_mode,
        &llvm,
        &bundle,
    )
    .unwrap();
    // Semantic checkpoint rejection, independent of the outer file hashes.
    for changed in [
        bundle.replace(&registration.symbols()[0], "wrong_export"),
        bundle.replace(
            &registration.canonical(),
            &registration.canonical().replace(" 1 4096", " 2 4096"),
        ),
        format!("{bundle}native_resource_contract=unknown\n"),
        format!("{bundle}native_resource_unknown=true\n"),
        format!("{bundle}native_session_layout=i64\n"),
        bundle.replace("single_binary=true", "single_binary=false"),
    ] {
        assert!(nuisc::aot::resource_session::verify_checkpoint(
            &module,
            &report.packaging_mode,
            &llvm,
            &changed
        )
        .is_err());
    }
    assert!(nuisc::aot::resource_session::verify_checkpoint(
        &module,
        &report.packaging_mode,
        &(llvm.clone() + "\n"),
        &bundle
    )
    .is_err());
    for changed in [
        SCRIPT.join("\n").replace("bytes", "other"),
        SCRIPT.join("\n").replace("\n3\n", "\n+3\n"),
        SCRIPT.join("\n").replace("\n5", "\nnot-an-integer"),
        format!("{}\n--close-arg\n5", SCRIPT.join("\n")),
        format!("{}\n--native-session\nbytes", SCRIPT.join("\n")),
        format!("{}\n--drain-provider", SCRIPT.join("\n")),
    ] {
        let args = changed.split('\n').collect::<Vec<_>>();
        rejected(project.command("run-artifact", &output, &args));
        rejected(
            Command::new(output.join(&stem))
                .args(&args)
                .output()
                .unwrap(),
        );
    }
    for name in [
        "bundle.txt".to_owned(),
        format!("{stem}.yir"),
        format!("{stem}.ll"),
        format!("{stem}.source.ns"),
        stem.clone(),
        "nuis.doc-index.json".to_owned(),
        "nuis.project.galaxy.lock".to_owned(),
    ] {
        let path = output.join(name);
        let original = fs::read(&path).unwrap();
        let mut corrupt = original.clone();
        corrupt.push(b' ');
        fs::write(&path, corrupt).unwrap();
        rejected(project.command("run-artifact", &output, SCRIPT));
        fs::write(&path, original).unwrap();
    }
    project.build(Some("native-resource-session-aot-bundle:other"));
    rejected(project.command("run-artifact", &output, SCRIPT));
    let mut other = SCRIPT.to_vec();
    other[1] = "other";
    assert_eq!(
        states(project.command("run-artifact", &output, &other)),
        expected
    );
    let cached = project.build(None);
    assert!(cached.contains("compile_cache: hit"), "{cached}");
    assert_eq!(
        states(project.command("run-artifact", &output, SCRIPT)),
        expected
    );

    // Bounds also participate in cache identity even when the ID is unchanged.
    let project_manifest = project.0.join("nuis.toml");
    let original = fs::read_to_string(&project_manifest).unwrap();
    fs::write(&project_manifest, original.replace("max=4096", "max=4095")).unwrap();
    let rebuilt = project.build(None);
    assert!(!rebuilt.contains("compile_cache: hit"), "{rebuilt}");
    let changed = yir_syntax::parse_explicit_module(
        &fs::read_to_string(output.join(format!("{stem}.yir"))).unwrap(),
    )
    .unwrap();
    assert_eq!(changed.resource_sessions[0].max_words, 4095);
    assert_ne!(
        fs::read_to_string(output.join(format!("{stem}.ll"))).unwrap(),
        llvm
    );
    fs::write(&project_manifest, original).unwrap();
    assert!(project.build(None).contains("compile_cache: hit"));

    let standalone = project.0.join("standalone");
    fs::create_dir(&standalone).unwrap();
    let artifact = standalone.join("nuis.compiled.artifact");
    fs::copy(output.join("nuis.compiled.artifact"), &artifact).unwrap();
    let bare_dir = project.0.join("binary-only");
    fs::create_dir(&bare_dir).unwrap();
    let bare_binary = bare_dir.join("resource-app");
    fs::copy(output.join(&stem), &bare_binary).unwrap();
    fs::remove_dir_all(&output).unwrap();
    fs::remove_file(project.0.join("main.ns")).unwrap();
    fs::remove_file(project.0.join("nuis.toml")).unwrap();
    // No source files, compiler, linker, sidecar libraries or PATH lookup at launch.
    assert_eq!(
        states(
            Command::new(&bare_binary)
                .args(SCRIPT)
                .current_dir(&bare_dir)
                .env("PATH", "")
                .output()
                .unwrap()
        ),
        expected
    );
    success(project.command("verify-artifact", &artifact, &[]));
    let relocated = project.0.join("relocated");
    success(project.command(
        "materialize-artifact",
        &artifact,
        &[relocated.to_str().unwrap()],
    ));
    assert_eq!(
        fs::read_to_string(relocated.join(format!("{stem}.ll"))).unwrap(),
        llvm
    );
    assert_eq!(
        fs::read_to_string(relocated.join(format!("{stem}.yir"))).unwrap(),
        yir
    );
    assert_eq!(
        states(project.command("run-artifact", &relocated, SCRIPT)),
        expected
    );
}
