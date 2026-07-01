use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

static CLI_SMOKE_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("nuis_{label}_{nonce}"));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_nuis(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_nuis"))
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("failed to run nuis {:?}: {error}", args))
}

fn assert_success(output: &std::process::Output, context: &str) {
    assert!(
        output.status.success(),
        "{context} failed\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

struct FfiSmokeCase {
    name: &'static str,
    source: &'static str,
    artifact: &'static str,
    yir_contains: &'static [&'static str],
    llvm_contains: &'static [&'static str],
    stdout_contains: &'static [&'static str],
    expect_manifest: bool,
}

const FFI_SMOKE_CASES: &[FfiSmokeCase] = &[
    FfiSmokeCase {
        name: "libc_usleep_demo",
        source: "../../examples/ns/ffi/libc_usleep_demo.ns",
        artifact: "libc_usleep_demo",
        yir_contains: &["libc getpid", "libc usleep"],
        llvm_contains: &["declare i32 @getpid()", "declare i32 @usleep(i32)"],
        stdout_contains: &["exit_status: 0"],
        expect_manifest: true,
    },
    FfiSmokeCase {
        name: "libc_puts_demo",
        source: "../../examples/ns/ffi/libc_puts_demo.ns",
        artifact: "libc_puts_demo",
        yir_contains: &["libc puts"],
        llvm_contains: &["declare i32 @puts(ptr)", "call i32 @puts(ptr"],
        stdout_contains: &["nuis libc puts bridge", "exit_status: 0"],
        expect_manifest: false,
    },
    FfiSmokeCase {
        name: "libc_strlen_demo",
        source: "../../examples/ns/ffi/libc_strlen_demo.ns",
        artifact: "libc_strlen_demo",
        yir_contains: &["libc strlen"],
        llvm_contains: &["declare i64 @strlen(ptr)", "call i64 @strlen(ptr"],
        stdout_contains: &["exit_status: 0"],
        expect_manifest: false,
    },
    FfiSmokeCase {
        name: "libc_write_demo",
        source: "../../examples/ns/ffi/libc_write_demo.ns",
        artifact: "libc_write_demo",
        yir_contains: &["libc strlen", "libc write"],
        llvm_contains: &["declare i64 @write(i32, ptr, i64)", "call i64 @write(i32"],
        stdout_contains: &["nuis libc write bridge", "exit_status: 0"],
        expect_manifest: false,
    },
    FfiSmokeCase {
        name: "libc_close_demo",
        source: "../../examples/ns/ffi/libc_close_demo.ns",
        artifact: "libc_close_demo",
        yir_contains: &["libc close"],
        llvm_contains: &["declare i32 @close(i32)", "call i32 @close(i32"],
        stdout_contains: &["exit_status: 0"],
        expect_manifest: false,
    },
    FfiSmokeCase {
        name: "libc_read_buffer_demo",
        source: "../../examples/ns/ffi/libc_read_buffer_demo.ns",
        artifact: "libc_read_buffer_demo",
        yir_contains: &["libc read"],
        llvm_contains: &["declare i64 @read(i32, ptr, i64)", "call i64 @read(i32"],
        stdout_contains: &["exit_status: 0"],
        expect_manifest: false,
    },
];

#[test]
fn ffi_libc_demos_build_and_run_as_native_artifacts() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();

    for case in FFI_SMOKE_CASES {
        let output_dir = temp_dir(case.name);
        let output_dir_text = output_dir.display().to_string();

        let build = run_nuis(&[
            "build",
            "--cpu-abi",
            "cpu.arm64.apple_aapcs64",
            case.source,
            &output_dir_text,
        ]);
        assert_success(&build, &format!("nuis build {}", case.name));

        let build_stdout = String::from_utf8_lossy(&build.stdout);
        assert!(build_stdout.contains("ready_to_run: true"));
        assert!(output_dir.join(case.artifact).exists());
        if case.expect_manifest {
            assert!(output_dir.join("nuis.build.manifest.toml").exists());
        }

        let yir = fs::read_to_string(output_dir.join(format!("{}.yir", case.artifact))).unwrap();
        for expected in case.yir_contains {
            assert!(
                yir.contains(expected),
                "{} missing yir marker {expected}",
                case.name
            );
        }

        let llvm_ir = fs::read_to_string(output_dir.join(format!("{}.ll", case.artifact))).unwrap();
        for expected in case.llvm_contains {
            assert!(
                llvm_ir.contains(expected),
                "{} missing llvm marker {expected}",
                case.name
            );
        }

        let run = run_nuis(&["run-artifact", &output_dir_text]);
        assert_success(&run, &format!("nuis run-artifact {}", case.name));
        let run_stdout = String::from_utf8_lossy(&run.stdout);
        for expected in case.stdout_contains {
            assert!(
                run_stdout.contains(expected),
                "{} missing stdout marker {expected}",
                case.name
            );
        }
    }
}

#[test]
fn clock_test_facade_builds_and_runs_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("clock_test_facades");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "../../examples/ns/ffi/hello_clock_test_facades.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build hello_clock_test_facades");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("hello_clock_test_facades").exists());
    let llvm_ir = fs::read_to_string(output_dir.join("hello_clock_test_facades.ll")).unwrap();
    assert!(llvm_ir.contains("static AOT lowering freezes cpu.tick_i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.tick_i64"));
    assert!(!llvm_ir.contains("deferred lowering for cpu.target_config"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact hello_clock_test_facades");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("exit_status: 0"));
}
