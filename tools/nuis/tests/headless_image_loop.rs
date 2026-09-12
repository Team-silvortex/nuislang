#![cfg(target_os = "macos")]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use yir_runtime_host::{
    run_application_script, ApplicationProviderSource, ApplicationScript, ApplicationScriptOutcome,
    ApplicationScriptTermination,
};

struct Artifacts(PathBuf);
impl Drop for Artifacts {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn bounded(command: &mut Command, directory: &Path) -> (ExitStatus, String, String) {
    let stdout = directory.join("process.stdout");
    let stderr = directory.join("process.stderr");
    let mut child = command
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_BUILD_JOBS", "1")
        .env_remove("NUIS_TEST_QUIET_SUCCESS_LOGS")
        .stdin(Stdio::null())
        .stdout(fs::File::create(&stdout).unwrap())
        .stderr(fs::File::create(&stderr).unwrap())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(180);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "headless image command exceeded deadline: {}",
                fs::read_to_string(&stderr).unwrap()
            );
        }
        thread::sleep(Duration::from_millis(20));
    };
    (
        status,
        fs::read_to_string(stdout).unwrap(),
        fs::read_to_string(stderr).unwrap(),
    )
}

fn frontdoor() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_nuis"));
    command
        .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
        .env_remove(yir_runtime_host::PROVIDER_RESULT_STREAM_ENV);
    command
}

fn script() -> ApplicationScript {
    ApplicationScript {
        id: "window".to_owned(),
        open: vec![160, 120],
        events: vec![vec![0, 0], vec![1, 32]],
        termination: ApplicationScriptTermination::Close(vec![1, 0]),
    }
}

fn expected_frame(phase: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(160 * 120 * 4);
    for y in 0..120 {
        for x in 0..160 {
            let alternate = (x / 5 / 4 + y / 5 / 4 + phase) % 2;
            bytes.extend(if y < 5 && x < 20 {
                [255, 0, 255, 255]
            } else if alternate == 0 {
                [0, 255, 255, 255]
            } else {
                [255, 255, 0, 255]
            });
        }
    }
    bytes
}

fn verify_frames(log: &str) {
    assert_eq!(
        log.matches("application_session_frame=").count(),
        2,
        "{log}"
    );
    for phase in 0..2 {
        let hash = yir_core::provider_runtime_ipc::hash_bytes(&expected_frame(phase));
        assert!(
            log.contains(&format!(
                "application_session_frame={};rgba8_fnv1a64={hash};bytes=76800",
                phase + 1
            )),
            "{log}"
        );
    }
}

#[test]
fn headless_buffer_loop_image_build_run_artifact_matches_direct_session_and_rejects_drift() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = Artifacts(
        std::env::temp_dir().join(format!("nuis-image-loop-{}-{nonce}", std::process::id())),
    );
    fs::create_dir(&directory.0).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let project = root.join("examples/projects/domains/ns_nova_image_showcase");
    let output = directory.0.join("out");
    let (status, stdout, stderr) = bounded(
        frontdoor()
            .current_dir(&root)
            .arg("build")
            .arg(project)
            .arg(&output)
            .args(["--packaging-mode", "headless-aot-bundle"]),
        &directory.0,
    );
    assert!(status.success(), "{stdout}\n{stderr}");
    let report =
        nuisc::aot::verify_build_manifest(&output.join("nuis.build.manifest.toml")).unwrap();
    assert_eq!(report.packaging_mode, "headless-aot-bundle");
    assert!(!output
        .join(format!("{}.ll", report.artifact_binary_name))
        .exists());
    let binary = output.join(&report.artifact_binary_name);
    let yir_path = output.join(format!("{}.yir", report.artifact_binary_name));
    let source = fs::read_to_string(&yir_path).unwrap();
    let module = yir_syntax::parse_module(&source).unwrap();
    let recolor = module
        .functions
        .iter()
        .find(|function| function.name.ends_with("recolor_run"))
        .expect("packaged YIR must retain the source run recoloring helper");
    let break_loop = module
        .nodes
        .iter()
        .find(|node| {
            recolor.body_nodes.contains(&node.name)
                && node.op.instruction == "loop_while_i64_effect"
                && node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries_break")
        })
        .expect("packaged recoloring must retain driver-level break, not unroll or no-op the tail");
    let break_helper = &break_loop.op.args[8];
    assert!(break_loop.op.args[9].ends_with("{carry0:i64;carry1:i64;carry2:i64}"));
    for (name, instruction) in [
        ("checkerboard_is_red", "call_bool"),
        ("checkerboard_parity", "call_i64"),
        ("checkerboard_red_total", "call_i64"),
        ("checkerboard_row_start", "call_i64"),
        ("checkerboard_row_end", "call_i64"),
    ] {
        let helper = module
            .functions
            .iter()
            .find(|function| function.name.contains(name))
            .expect("packaged YIR must retain the source scalar helper");
        assert!(
            module
                .nodes
                .iter()
                .any(|node| node.op.instruction == instruction
                    && node.op.args.first() == Some(&helper.name)),
            "missing packaged call to {name}"
        );
    }
    assert!(
        module.nodes.iter().any(|node| {
            node.op.instruction == "loop_while_i64_effect"
                && node.op.args.get(6).map(String::as_str) == Some("scoped_call_i64_carries")
                && node
                    .op
                    .args
                    .iter()
                    .any(|arg| arg.contains("__nuis_buffer_iteration_"))
        }),
        "packaged YIR must contain the real pixel-writing loop"
    );
    assert!(
        module.functions.iter().any(|function| {
            function.name.contains("__nuis_buffer_iteration_")
                && module.nodes.iter().any(|node| {
                    function.body_nodes.contains(&node.name)
                        && node.op.instruction == "loop_while_i64_effect"
                        && node.op.args.get(6).map(String::as_str)
                            == Some("scoped_call_i64_carries")
                })
        }),
        "packaged row helpers must retain nested pixel loops"
    );
    assert!(
        module.functions.iter().any(|function| {
            function.name.contains("__nuis_buffer_branch_")
                && function.result.as_ref().is_some_and(|result| {
                    result.ty.contains("__nuis_scalar_carries_")
                        && module.nodes.iter().any(|node| {
                            node.name == result.node
                                && node.op.instruction == "return_owned_struct"
                                && node.op.args.get(1).is_some_and(|layout| {
                                    layout.ends_with("{carry0:i64;carry1:i64;carry2:i64}")
                                })
                        })
                })
                && function.body_nodes.iter().any(|name| {
                    module.nodes.iter().any(|node| {
                        &node.name == name
                            && node.op.instruction == "call_i64"
                            && node
                                .op
                                .args
                                .first()
                                .is_some_and(|callee| callee.contains("checkerboard_red_total"))
                    })
                })
                && function.body_nodes.iter().any(|name| {
                    module
                        .nodes
                        .iter()
                        .any(|node| &node.name == name && node.op.instruction == "guard_return")
                })
        }),
        "packaged pixel branches must retain a real control boundary and branch-local count"
    );
    assert!(
        module.functions.iter().any(|function| {
            function.name.contains("__nuis_scalar_branch_")
                && function
                    .result
                    .as_ref()
                    .is_some_and(|result| result.ty == "bool")
                && function.body_nodes.iter().any(|name| {
                    module
                        .nodes
                        .iter()
                        .any(|node| &node.name == name && node.op.instruction == "guard_return")
                })
        }),
        "packaged source color helper must retain its typed branch guard"
    );
    let arguments = script().to_arguments().unwrap();
    let (status, stdout, stderr) = bounded(
        frontdoor()
            .arg("run-artifact")
            .arg(&output)
            .args(&arguments),
        &directory.0,
    );
    assert!(status.success(), "{stdout}\n{stderr}");
    assert!(stdout.contains("exit_status: 0"), "{stdout}");
    assert!(
        stdout.contains("runtime_provider_result_invocations: 2"),
        "{stdout}"
    );
    assert!(
        stderr.contains("application_session_outcome=[1, 1, 0]"),
        "{stderr}"
    );
    verify_frames(&stderr);

    let stream = output.join("nuis.runtime.provider-result-stream.toml");
    let saved_stream = fs::read(&stream).unwrap();
    let mut frames = Vec::new();
    let mut writes = 0;
    let mut run_exits = 0;
    let outcome = run_application_script(
        source.clone(),
        ApplicationProviderSource::Replay(&stream),
        script(),
        Duration::from_secs(90),
        |reply| {
            let trace = reply.trace.as_ref().unwrap();
            run_exits += trace
                .events
                .iter()
                .filter(|event| {
                    event.contains("iterations=5 final=4 action cpu.scoped_call_i64_carries_break")
                        && event.ends_with(break_helper)
                })
                .count();
            writes += trace
                .lane_steps
                .values()
                .flatten()
                .filter(|step| step.starts_with("cpu.store_at "))
                .count();
            frames.extend(
                trace
                    .presented_frames
                    .iter()
                    .map(|frame| frame.rgba8.clone().unwrap()),
            );
        },
    )
    .unwrap();
    assert!(matches!(outcome, ApplicationScriptOutcome::Finished(_)));
    assert_eq!(frames, vec![expected_frame(0), expected_frame(1)]);
    assert_eq!(
        run_exits, 2,
        "each callback exits at the first nonmatching pixel"
    );
    assert_eq!(
        writes,
        2 * (768 + 4 + 1),
        "pixel fill, four recolor writes and post-snapshot mutation per frame"
    );

    let mut invalid = script();
    invalid.events.push(vec![]);
    let (status, _, stderr) = bounded(
        frontdoor()
            .arg("run-artifact")
            .arg(&output)
            .args(invalid.to_arguments().unwrap()),
        &directory.0,
    );
    assert!(!status.success());
    assert!(
        !stderr.contains("application_session_reply="),
        "admit all callback arguments before effects: {stderr}"
    );
    assert_eq!(fs::read(&stream).unwrap(), saved_stream);

    // Restore each altered file before asserting so failures retain a usable bundle.
    for path in [&binary, &yir_path] {
        let original = fs::read(path).unwrap();
        let mut changed = original.clone();
        *changed.last_mut().unwrap() ^= 1;
        fs::write(path, changed).unwrap();
        let (status, _, stderr) = bounded(
            frontdoor()
                .arg("run-artifact")
                .arg(&output)
                .args(&arguments),
            &directory.0,
        );
        fs::write(path, original).unwrap();
        assert!(!status.success(), "artifact drift must be rejected");
        assert!(!stderr.contains("application_session_reply="), "{stderr}");
        assert_eq!(fs::read(&stream).unwrap(), saved_stream);
    }

    let mut exhausted = script();
    exhausted.events.push(vec![0, 0]);
    let (status, _, stderr) = bounded(
        Command::new(binary)
            .args(exhausted.to_arguments().unwrap())
            .env_remove(yir_runtime_host::PROVIDER_DISPATCH_SOCKET_ENV)
            .env(yir_runtime_host::PROVIDER_RESULT_STREAM_ENV, &stream),
        &directory.0,
    );
    assert_eq!(status.code(), Some(1), "{stderr}");
    verify_frames(&stderr);
    assert!(
        !stderr.contains("application_session_reply=Close"),
        "{stderr}"
    );
    assert!(!stderr.contains("application_session_outcome="), "{stderr}");
    assert_eq!(fs::read(stream).unwrap(), saved_stream);
}
