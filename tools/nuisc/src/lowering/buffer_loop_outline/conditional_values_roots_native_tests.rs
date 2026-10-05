use super::source;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "nuis-root-short-circuit-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn conditional_root_values_execute_default_source_lazy_rhs_and_native_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (root, op, gate, divisor, expected) in [
        ("let", "&&", false, 0, Some("19")),
        ("inferred", "||", true, 0, Some("11")),
        ("const", "&&", false, 0, Some("19")),
        ("return", "||", true, 0, Some("11")),
        ("let", "&&", true, 2, Some("11")),
        ("const", "||", false, 2, Some("11")),
        ("return", "&&", true, 2, Some("11")),
        ("return", "||", false, -2, Some("19")),
        ("const", "&&", true, 0, None),
        ("return", "||", false, 0, None),
    ] {
        let source = source(root, op, gate, divisor, false)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        // No explicit private projection registration or foreign runtime shim.
        let compiled = crate::pipeline::compile_source(&source).unwrap();
        let llvm = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        let scratch = Scratch::new();
        let source_path = scratch.0.join("main.ns");
        fs::write(&source_path, &source).unwrap();
        let artifact = crate::aot::write_and_link_with_source(
            &source_path,
            &scratch.0.join("out"),
            &source,
            crate::aot::AotCompileProgram {
                ast: &compiled.ast,
                nir: &compiled.nir,
                yir: &compiled.yir,
                llvm_ir: Some(&llvm),
            },
            &crate::aot::host_cpu_build_target(),
        )
        .unwrap();
        let stdout = scratch.0.join("stdout");
        let stderr = scratch.0.join("stderr");
        let mut child = Command::new(&artifact.binary_path)
            .current_dir(&scratch.0)
            .stdout(fs::File::create(&stdout).unwrap())
            .stderr(fs::File::create(&stderr).unwrap())
            .spawn()
            .unwrap();
        let started = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if started.elapsed() > Duration::from_secs(10) {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("native short-circuit probe timed out: {root}/{op}/{gate}/{divisor}");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        let output = fs::read_to_string(stdout).unwrap();
        let errors = fs::read_to_string(stderr).unwrap();
        if let Some(expected) = expected {
            assert!(status.success(), "{root}/{op}: {status:?} {errors}");
            assert_eq!(output.trim(), expected);
        } else {
            assert!(!status.success());
            assert!(output.is_empty(), "{output:?}");
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                assert!(
                    matches!(status.signal(), Some(4 | 5)),
                    "{status:?} {errors}"
                );
            }
        }
    }
}
