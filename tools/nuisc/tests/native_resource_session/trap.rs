use super::*;
use std::os::unix::process::ExitStatusExt;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

struct TemporaryRoot(PathBuf);
impl Drop for TemporaryRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn native_resource_trap_is_a_process_failure_not_recoverable_completion() {
    const ENV: &str = "NUIS_RESOURCE_TRAP_ROOT";
    if std::env::var_os(ENV).is_some() {
        let source = SOURCE.replace("store_at(buffer, 0, value);", "let pending: Bytes = copy_bytes(buffer); store_at(buffer, 99, value); drop_bytes(pending);");
        let program = compile(&source);
        let bridge =
            yir_lower_llvm::native_resource_session::emit_registered(&program.yir, &registration())
                .unwrap();
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.binding(&program.yir, &bridge),
            vec![0, -7],
            10,
        )
        .unwrap();
        assert_eq!(session.state(), &[10, -7]);
        assert_eq!(library.live_blobs(), 0);
        eprintln!("resource-before-trap:10,-7");
        let _ = session.event(4);
        eprintln!("resource-after-trap");
        panic!("native trap must not return to the host");
    }
    let root = TemporaryRoot(
        std::env::temp_dir().join(format!("nuis-resource-trap-parent-{}", std::process::id())),
    );
    fs::create_dir(&root.0).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "native_trap::native_resource_trap_is_a_process_failure_not_recoverable_completion",
            "--nocapture",
        ])
        .env(ENV, &root.0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("resource trap probe exceeded its deadline");
        }
        thread::sleep(Duration::from_millis(20));
    }
    let output = child.wait_with_output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        matches!(output.status.signal(), Some(4 | 5)),
        "{}: {stderr}",
        output.status
    );
    assert!(stderr.contains("resource-before-trap:10,-7"), "{stderr}");
    assert!(!stderr.contains("resource-after-trap"));
}
