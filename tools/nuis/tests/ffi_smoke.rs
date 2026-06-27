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

#[test]
fn libc_usleep_demo_builds_and_runs_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_usleep_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_usleep_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_usleep_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_usleep_demo").exists());
    assert!(output_dir.join("nuis.build.manifest.toml").exists());
    let yir = fs::read_to_string(output_dir.join("libc_usleep_demo.yir")).unwrap();
    assert!(yir.contains("libc getpid"));
    assert!(yir.contains("libc usleep"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_usleep_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i32 @getpid()"));
    assert!(llvm_ir.contains("declare i32 @usleep(i32)"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_usleep_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("exit_status: 0"));
}

#[test]
fn libc_puts_demo_builds_and_prints_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_puts_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_puts_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_puts_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_puts_demo").exists());
    let yir = fs::read_to_string(output_dir.join("libc_puts_demo.yir")).unwrap();
    assert!(yir.contains("libc puts"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_puts_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i32 @puts(ptr)"));
    assert!(llvm_ir.contains("call i32 @puts(ptr"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_puts_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("nuis libc puts bridge"));
    assert!(run_stdout.contains("exit_status: 0"));
}

#[test]
fn libc_strlen_demo_builds_and_runs_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_strlen_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_strlen_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_strlen_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_strlen_demo").exists());
    let yir = fs::read_to_string(output_dir.join("libc_strlen_demo.yir")).unwrap();
    assert!(yir.contains("libc strlen"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_strlen_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i64 @strlen(ptr)"));
    assert!(llvm_ir.contains("call i64 @strlen(ptr"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_strlen_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("exit_status: 0"));
}

#[test]
fn libc_write_demo_builds_and_writes_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_write_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_write_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_write_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_write_demo").exists());
    let yir = fs::read_to_string(output_dir.join("libc_write_demo.yir")).unwrap();
    assert!(yir.contains("libc strlen"));
    assert!(yir.contains("libc write"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_write_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i64 @write(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @write(i32"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_write_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("nuis libc write bridge"));
    assert!(run_stdout.contains("exit_status: 0"));
}

#[test]
fn libc_close_demo_builds_and_runs_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_close_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_close_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_close_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_close_demo").exists());
    let yir = fs::read_to_string(output_dir.join("libc_close_demo.yir")).unwrap();
    assert!(yir.contains("libc close"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_close_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i32 @close(i32)"));
    assert!(llvm_ir.contains("call i32 @close(i32"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_close_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("exit_status: 0"));
}

#[test]
fn libc_read_buffer_demo_builds_and_runs_as_native_artifact() {
    let _guard = CLI_SMOKE_LOCK.lock().unwrap();
    let output_dir = temp_dir("libc_read_buffer_demo");
    let output_dir_text = output_dir.display().to_string();

    let build = run_nuis(&[
        "build",
        "--cpu-abi",
        "cpu.arm64.apple_aapcs64",
        "/Users/Shared/chroot/dev/nuislang/examples/ns/ffi/libc_read_buffer_demo.ns",
        &output_dir_text,
    ]);
    assert_success(&build, "nuis build libc_read_buffer_demo");

    let build_stdout = String::from_utf8_lossy(&build.stdout);
    assert!(build_stdout.contains("ready_to_run: true"));
    assert!(output_dir.join("libc_read_buffer_demo").exists());
    let yir = fs::read_to_string(output_dir.join("libc_read_buffer_demo.yir")).unwrap();
    assert!(yir.contains("libc read"));
    let llvm_ir = fs::read_to_string(output_dir.join("libc_read_buffer_demo.ll")).unwrap();
    assert!(llvm_ir.contains("declare i64 @read(i32, ptr, i64)"));
    assert!(llvm_ir.contains("call i64 @read(i32"));

    let run = run_nuis(&["run-artifact", &output_dir_text]);
    assert_success(&run, "nuis run-artifact libc_read_buffer_demo");
    let run_stdout = String::from_utf8_lossy(&run.stdout);
    assert!(run_stdout.contains("exit_status: 0"));
}
