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
            "nuis-conditional-return-{}-{}",
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
fn conditional_returns_execute_default_source_parent_effects_exits_and_native_traps() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    for (shape, op, outer, gate, divisor, early, expected) in [
        ("then", "&&", false, true, 0, false, Some("99\n77\n19")),
        ("then", "&&", true, false, 0, false, Some("99\n19")),
        ("else", "||", true, false, 0, false, Some("99\n77\n19")),
        ("else", "||", false, true, 0, false, Some("99\n11")),
        ("both", "&&", true, true, 2, false, Some("99\n11")),
        ("both", "&&", false, false, -2, false, Some("99\n19")),
        ("both", "||", false, false, 0, true, Some("19")),
        ("then", "&&", true, true, 0, false, None),
        ("else", "||", false, false, 0, false, None),
        ("both", "&&", false, false, 0, false, None),
    ] {
        let source = source(shape, op, outer, gate, divisor, early)
            .replace("print(11); return 11;", "print(11); return 0;")
            .replace("print(19); return 19;", "print(19); return 0;");
        run(&source, expected);
    }
}

pub(in crate::lowering::buffer_loop_outline::conditional_returns) fn run(
    source: &str,
    expected: Option<&str>,
) {
    // Default source compilation, with no private projection registration.
    let compiled = crate::pipeline::compile_source(source).unwrap();
    let llvm = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    let scratch = Scratch::new();
    let source_path = scratch.0.join("main.ns");
    fs::write(&source_path, source).unwrap();
    let artifact = crate::aot::write_and_link_with_source(
        &source_path,
        &scratch.0.join("out"),
        source,
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
            panic!("native conditional return timed out: {source}");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(stdout).unwrap();
    let errors = fs::read_to_string(stderr).unwrap();
    if let Some(expected) = expected {
        assert!(status.success(), "{source}: {status:?} {errors}");
        assert_eq!(output.trim(), expected);
    } else {
        assert!(!status.success());
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
