use super::*;
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
            "nuis-caller-record-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
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
fn caller_record_constructors_execute_projected_ir_as_a_native_binary() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let source = format!(
        "mod cpu Main {{ {RECORDS}
        @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
        @noinline fn helper(before: i64, state: State, after: i64) -> i64 {{ return before + state.a.x + after; }}
        @noinline fn entry(state: State, divisor: i64) -> i64 {{
            return helper(checked(divisor), {ARGUMENT}, checked(divisor));
        }} fn main() -> i64 {{
            print(entry(State {{ a: Pair {{ x: 11, y: 13 }}, b: Pair {{ x: 17, y: 19 }}, unused: 23 }}, 2));
            return 0;
        }} }}"
    );
    // Exercise the private-capture IR contract explicitly. The source helper's
    // public signature is not made projectable merely by its source spelling.
    let compiled = crate::pipeline::compile_source(&source).unwrap();
    let mut nir = compiled.nir.clone();
    assert!(run(&mut nir));
    crate::nir_verify::verify_nir_module(&nir).unwrap();
    let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
    let llvm = yir_lower_llvm::emit_module(&yir).unwrap();
    assert!(llvm.contains("@nuis_fn_helper("));
    let signature = llvm
        .lines()
        .find(|line| line.starts_with("define i64 @nuis_fn_helper("))
        .unwrap();
    assert_eq!(signature.matches("i64 %").count(), 3, "{signature}");
    let scratch = Scratch::new();
    let source_path = scratch.0.join("main.ns");
    fs::write(&source_path, &source).unwrap();
    let artifact = crate::aot::write_and_link_with_source(
        &source_path,
        &scratch.0.join("out"),
        &source,
        crate::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &nir,
            yir: &yir,
            llvm_ir: Some(&llvm),
        },
        &crate::aot::host_cpu_build_target(),
    )
    .unwrap();
    let stdout = scratch.0.join("stdout");
    let stderr = scratch.0.join("stderr");
    let mut child = Command::new(&artifact.binary_path)
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
            let _ = child.kill();
            let _ = child.wait();
            panic!("caller record native probe exceeded its deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(status.success(), "{}", fs::read_to_string(&stderr).unwrap());
    assert_eq!(fs::read_to_string(stdout).unwrap().trim(), "27");
}
