//! Production packer -> static callback object -> shared runtime session host.
//! Fault injection edits generated objects only, never substitutes a test host.
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use yir_core::{Value, YirModule};
use yir_runtime_host::ApplicationSession;

const SOURCE: &str = include_str!("native_application_bridge/main.ns");
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nuis-native-application-host-{}",
            std::process::id()
        ));
        fs::create_dir(&path).unwrap();
        fs::write(path.join("main.ns"), SOURCE).unwrap();
        fs::write(path.join("nuis.toml"), "name = \"native_session\"\nentry = \"main.ns\"\nmodules = [\"main.ns\"]\napplication_sessions = [\"counter open=start event=step close=stop state=state\"]\n").unwrap();
        Self(path)
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

fn package(input: &Path, output: &Path) {
    success(
        Command::new("cargo")
            .args(["run", "--quiet", "-p", "yir-pack-aot", "--"])
            .arg(input)
            .arg(output)
            .args(["--native-session", "counter"])
            .env("CARGO_BUILD_JOBS", "1")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .unwrap(),
    );
}

fn relink(output: &Path, runtime: &Path) {
    let mut command = Command::new("clang");
    for file in [
        "session_callbacks.ll",
        "session_native_host.ll",
        "session_scalar_runtime.c",
    ] {
        command.arg(output.join(file));
    }
    command.arg(runtime).arg("-O2");
    if cfg!(target_os = "linux") {
        command.args(["-ldl", "-lpthread", "-lm", "-lrt", "-lutil"]);
    } else {
        command.arg("-liconv");
    }
    success(
        command
            .arg("-o")
            .arg(output.join("session"))
            .output()
            .unwrap(),
    );
}

fn command(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    command.args([
        "--application-session",
        "counter",
        "--open-args",
        "10,-17,true,1.5,-2.25",
    ]);
    command
}

fn states(output: &Output) -> Vec<String> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|line| {
            line.strip_prefix("nuis: native_session_state=")
                .map(str::to_owned)
        })
        .collect()
}

fn reference(module: &YirModule, events: &[(i64, bool)]) -> Vec<String> {
    let registry = yir_verify::default_registry();
    let (mut session, _) = ApplicationSession::open_registered(
        module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(1.5),
            Value::F64(-2.25),
        ],
    )
    .unwrap();
    let mut states = vec![format!("open:{}", session.state())];
    for &(delta, paused) in events {
        session
            .event(vec![Value::Int(delta), Value::Bool(paused)])
            .unwrap();
        states.push(format!("event:{}", session.state()));
    }
    session.close(vec![Value::Int(5)]).unwrap();
    states.push(format!("close:{}", session.state()));
    session.completion_status().unwrap();
    states
}

fn encoded(source: &str) -> String {
    source.bytes().map(|b| format!("\\{b:02X}")).collect()
}

#[test]
fn packaged_native_session_uses_shared_host_without_reference_fallback() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let project = Project::new();
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let expected = reference(&compiled.yir, &[(3, false), (100, true), (-2, false)]);
    for reversed in [false, true] {
        let mut module = compiled.yir.clone();
        if reversed {
            module.nodes.reverse();
            module.functions.reverse();
            for function in &mut module.functions {
                function.body_nodes.reverse();
            }
        }
        let yir = nuisc::render::render_yir(&module);
        let input = project.0.join("session.yir");
        fs::write(&input, &yir).unwrap();
        let output = project.0.join(format!("out-{reversed}"));
        package(&input, &output);
        let binary = output.join("session");
        let run = success(
            command(&binary)
                .args([
                    "--event-args",
                    "3,false",
                    "--event-args",
                    "100,true",
                    "--event-args",
                    "-2,false",
                    "--close-args",
                    "5",
                ])
                .output()
                .unwrap(),
        );
        assert_eq!(states(&run), expected);
        assert!(run.stdout.is_empty(), "unrelated main must not run");
        assert!(String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));
        let manifest = fs::read_to_string(output.join("bundle.txt")).unwrap();
        assert!(manifest.contains("cpu_host_binary_mode=native_scalar_session\n"));
        assert!(manifest.contains("native_session_identity=exact-yir-graph\n"));
        assert!(!manifest.contains("embedded_yir_headless"));
        for invalid in [
            vec!["--event-args", "3,1", "--close-args", "5"],
            vec!["--event-args", "3,false", "--close-args", "true"],
            vec!["--close-args", "5", "--application-session", "other"],
            vec!["--cancel-after-events", "true", "--close-args", "5"],
        ] {
            let run = command(&binary).args(invalid).output().unwrap();
            assert!(!run.status.success());
            assert!(
                states(&run).is_empty(),
                "all script arguments must be admitted before open"
            );
        }
        if reversed {
            continue;
        }
        let runtime = PathBuf::from(
            manifest
                .lines()
                .find_map(|l| l.strip_prefix("runtime_host_staticlib="))
                .unwrap(),
        );
        let callback_path = output.join("session_callbacks.ll");
        let callbacks = fs::read_to_string(&callback_path).unwrap();
        let bridge = yir_lower_llvm::native_session::emit_registered(&module, "counter").unwrap();
        let signature = format!(
            "define i32 @{}(ptr %args, i64 %argc, ptr %out, i64 %outc) {{\nentry:\n",
            bridge.callbacks[1].symbol
        );
        assert_eq!(callbacks.matches(&signature).count(), 1);
        fs::write(
            &callback_path,
            callbacks.replace(
                &signature,
                &format!("{signature}  ret i32 77\nunreachable_callback_body:\n"),
            ),
        )
        .unwrap();
        relink(&output, &runtime);
        let run = command(&binary)
            .args(["--event-args", "3,false", "--close-args", "5"])
            .output()
            .unwrap();
        assert!(!run.status.success());
        let result = states(&run);
        assert_eq!(
            result.len(),
            2,
            "failed native event must not fall back to reference execution"
        );
        assert_eq!(
            result,
            reference(&module, &[]),
            "cleanup must use last accepted state"
        );
        assert!(String::from_utf8_lossy(&run.stderr).contains("status 77"));
        assert!(!String::from_utf8_lossy(&run.stderr).contains("native_session_completed=1"));

        fs::write(&callback_path, callbacks).unwrap();
        let host_path = output.join("session_native_host.ll");
        let host = fs::read_to_string(&host_path).unwrap();
        let stale = yir.replace("999", "998");
        assert_ne!(stale, yir);
        fs::write(&host_path, host.replace(&encoded(&yir), &encoded(&stale))).unwrap();
        relink(&output, &runtime);
        let run = command(&binary)
            .args(["--close-args", "5"])
            .output()
            .unwrap();
        assert!(!run.status.success());
        assert!(states(&run).is_empty());
        assert!(String::from_utf8_lossy(&run.stderr).contains("artifact identity mismatch"));
    }
}
