#![cfg(any(target_os = "macos", target_os = "linux"))]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
    thread,
    time::{Duration, Instant},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
const CPU: &str = "yir 0.1\n\
resource host cpu.main\n\
function main cpu entry\n\
function-result main i64 value done\n\
function-node main left\n\
function-node main right\n\
function-node main sum\n\
function-node main output\n\
function-node main done\n\
cpu.const_i64 left host 7\n\
cpu.const_i64 right host 5\n\
cpu.add sum host left right\n\
cpu.print output host sum\n\
cpu.const_i64 done host 0\n\
edge dep left sum\n\
edge dep right sum\n\
edge dep sum output\n\
edge effect sum output\n\
edge dep output done\n";

struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "ns-pack-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn pack(&self, source: &str, options: &[&str]) -> (ExitStatus, String) {
        let input = self.0.join("input.yir");
        fs::write(&input, source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_yir-pack-aot"));
        command.arg(&input).arg(self.0.join("bundle")).args(options);
        run(&mut command, &self.0.join("pack.log"))
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(command: &mut Command, log: &Path) -> (ExitStatus, String) {
    let stdout = fs::File::create(log).unwrap();
    let stderr = stdout.try_clone().unwrap();
    let mut child = command
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(60);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            return (status, fs::read_to_string(log).unwrap());
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "packager process exceeded deadline: {}",
                fs::read_to_string(log).unwrap()
            );
        }
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn cpu_bundle_keeps_manifest_layout_and_executes_real_native_output() {
    let project = Project::new();
    let (status, log) = project.pack(CPU, &[]);
    assert!(status.success(), "{log}");
    assert!(log.contains("packed AOT bundle into"));
    let bundle = project.0.join("bundle");
    let manifest = fs::read_to_string(bundle.join("bundle.txt")).unwrap();
    for field in [
        "binary_mode=llvm_clang",
        "shader_backend_eligible=false",
        "kernel_backend_eligible=false",
        "render_mode=none",
        "host_ffi_used_symbols=0",
    ] {
        assert!(manifest.lines().any(|line| line == field), "{manifest}");
    }
    for file in ["input.ll", "input_shim.c", "input"] {
        assert!(bundle.join(file).is_file(), "{file}");
    }
    assert!(!bundle.join("input_host.m").exists());
    let (status, stdout) = run(
        &mut Command::new(bundle.join("input")),
        &project.0.join("run.log"),
    );
    assert!(status.success(), "{status}: {stdout}");
    assert_eq!(stdout, "12\n");
}

#[test]
fn invalid_source_and_profiles_fail_before_bundle_publication() {
    for (source, options, diagnostic) in [
        ("not a YIR module", vec![], None),
        (
            CPU,
            vec!["--headless"],
            Some("requires a registered application session"),
        ),
        (CPU, vec!["--native-session"], Some("requires")),
    ] {
        let project = Project::new();
        let (status, log) = project.pack(source, &options);
        assert!(!status.success());
        if let Some(diagnostic) = diagnostic {
            assert!(log.contains(diagnostic), "{log}");
        }
        assert!(!project.0.join("bundle").exists(), "{log}");
    }
}
