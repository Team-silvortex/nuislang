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
            "nuis-caller-spill-{}-{}",
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
fn caller_spills_execute_projected_ir_with_full_results_and_guarded_native_failures() {
    run_probes(false);
}

#[test]
fn caller_spills_return_roots_execute_full_operands_and_preserve_early_exit_traps() {
    run_probes(true);
}

#[test]
fn caller_spills_condition_roots_execute_selected_branches_and_retain_native_traps() {
    if !native_host() {
        return;
    }
    for (unused, guard, flag, yes, no, expected) in [
        (2, 1, true, 2, 0, Some("11")),
        (2, 1, false, 0, 2, Some("19")),
        (0, 0, true, 0, 0, Some("0")),
        (0, 1, true, 2, 0, None),
        (2, 1, false, 2, 0, None),
    ] {
        let mut source =
            super::conditions::source("produce(state, unused)", super::conditions::BODY);
        assert_eq!(source.pop(), Some('}'));
        source.push_str(&format!("fn main() -> i64 {{
            print(entry(State {{ a: Pair {{ x: 11, y: 13 }}, b: Pair {{ x: 17, y: 19 }}, unused: 23 }},
                2, {unused}, {guard}, {flag}, {yes}, {no})); return 0;
        }} }}"));
        run_source_probe(&source, true, expected);
    }
}

#[test]
fn caller_spills_gated_predicates_execute_lazy_rhs_and_keep_native_traps() {
    if !native_host() {
        return;
    }
    for (op, divisor, unused, guard, flag, yes, no, gate, expected) in [
        ("&&", 0, 0, 1, true, 2, 2, false, Some("19")),
        ("||", 0, 0, 1, false, 2, 2, true, Some("11")),
        ("&&", 2, 2, 1, true, 2, 0, true, Some("11")),
        ("||", 2, 2, 1, false, 0, 2, false, Some("19")),
        ("&&", 2, 0, 1, true, 2, 2, true, None),
        ("||", 2, 2, 1, true, 0, 2, false, None),
        ("&&", 0, 0, 0, true, 0, 0, true, Some("0")),
    ] {
        let mut source = super::gated::source("gate", op, "produce(state, unused)");
        assert_eq!(source.pop(), Some('}'));
        source.push_str(&format!("fn main() -> i64 {{
            print(entry(State {{ a: Pair {{ x: 11, y: 13 }}, b: Pair {{ x: 17, y: 19 }}, unused: 23 }},
                {divisor}, {unused}, {guard}, {flag}, {yes}, {no}, {gate})); return 0;
        }} }}"));
        run_projected_probe(&source, true, true, expected);
    }
}

fn native_host() -> bool {
    cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    ))
}

fn run_probes(return_root: bool) {
    if !native_host() {
        return;
    }
    let call = "helper(checked(divisor), produce(state, unused), 20 / divisor, true)";
    let body = if return_root {
        format!("return {call};")
    } else {
        format!("let result = {call}; return result;")
    };
    for (unused, guard, expected) in [(2, 1, Some("26")), (0, 0, Some("0")), (0, 1, None)] {
        let source = format!(
            "mod cpu Main {{ {RECORDS}
            @noinline fn checked(value: i64) -> i64 {{ return 10 / value; }}
            @noinline fn produce(state: State, divisor: i64) -> State {{ return {CONSTRUCTOR}; }}
            @noinline fn helper(before: i64, state: State, after: i64, flag: bool) -> i64 {{
                if flag {{ return before + state.a.x + after; }} return before + state.b.y + after;
            }} @noinline fn entry(state: State, divisor: i64, unused: i64, guard: i64) -> i64 {{
                if guard == 0 {{ return 0; }}
                {body}
            }} fn main() -> i64 {{
                print(entry(State {{ a: Pair {{ x: 11, y: 13 }}, b: Pair {{ x: 17, y: 19 }}, unused: 23 }}, 2, {unused}, {guard}));
                return 0;
            }} }}"
        );
        run_source_probe(&source, false, expected);
    }
}

fn run_source_probe(source: &str, predicate: bool, expected: Option<&str>) {
    run_projected_probe(source, predicate, false, expected);
}

fn run_projected_probe(source: &str, predicate: bool, gated: bool, expected: Option<&str>) {
    // Explicit private-capture registration, not source-name eligibility.
    let compiled = crate::pipeline::compile_source(source).unwrap();
    let mut nir = compiled.nir.clone();
    assert!(run(&mut nir));
    crate::nir_verify::verify_nir_module(&nir).unwrap();
    assert_eq!(
        function(&nir, "produce"),
        function(&compiled.nir, "produce")
    );
    let yir = crate::lowering::lower_nir_to_yir_builtin_cpu(&nir).unwrap();
    let llvm = yir_lower_llvm::emit_module(&yir).unwrap();
    let result_kind = if predicate { "i1" } else { "i64" };
    let prefix = format!("define {result_kind} @nuis_fn_helper(");
    let signature = llvm.lines().find(|line| line.starts_with(&prefix)).unwrap();
    assert_eq!(signature.matches("i64 %").count(), 4, "{signature}");
    assert_eq!(signature.matches("i1 %").count(), 1, "{signature}");
    let entry = if gated {
        let sites = llvm
            .split("define ")
            .filter_map(|block| block.split("\n}").next())
            .filter(|block| {
                [
                    "@nuis_fn_checked(",
                    "@nuis_fn_produce(",
                    "@nuis_fn_helper(",
                    "sdiv i64",
                ]
                .iter()
                .all(|needle| block.contains(needle))
            })
            .collect::<Vec<_>>();
        assert_eq!(
            sites.len(),
            1,
            "ordered operand producer must belong to one guarded body"
        );
        sites[0]
    } else {
        llvm.split("define i64 @nuis_fn_entry(")
            .nth(1)
            .unwrap()
            .split("\n}")
            .next()
            .unwrap()
    };
    let producer_calls = entry
        .lines()
        .filter(|line| line.contains("call ") && line.contains("@nuis_fn_produce("))
        .collect::<Vec<_>>();
    assert_eq!(producer_calls.len(), 1, "{producer_calls:?}");
    let before = entry.find("@nuis_fn_checked(").unwrap();
    let produced = entry.find("@nuis_fn_produce(").unwrap();
    let after = if gated {
        // Guarded transports may decode bool words with earlier divisions by 2.
        // Identify the original 20/divisor operand, not those total decoders.
        let literals = entry
            .lines()
            .filter(|line| line.contains("= add i64 0, 20"))
            .collect::<Vec<_>>();
        assert_eq!(literals.len(), 1);
        let numerator = literals[0].split('=').next().unwrap().trim();
        entry.find(&format!("sdiv i64 {numerator},")).unwrap()
    } else {
        entry.find("sdiv i64").unwrap()
    };
    let invoked = entry.find("@nuis_fn_helper(").unwrap();
    assert!(
        before < produced && produced < after && after < invoked,
        "operand order {before}/{produced}/{after}/{invoked}: {entry}"
    );
    let scratch = Scratch::new();
    let source_path = scratch.0.join("main.ns");
    fs::write(&source_path, source).unwrap();
    let artifact = crate::aot::write_and_link_with_source(
        &source_path,
        &scratch.0.join("out"),
        source,
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
            let _ = child.kill();
            let _ = child.wait();
            panic!("caller spill native probe exceeded its deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let output = fs::read_to_string(&stdout).unwrap();
    let errors = fs::read_to_string(&stderr).unwrap();
    if let Some(expected) = expected {
        assert!(status.success(), "{errors}");
        assert_eq!(output.trim(), expected);
    } else {
        assert!(!status.success());
        assert!(output.is_empty(), "{output}");
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(status.signal(), Some(4 | 5)),
                "{status:?}: {errors}"
            );
        }
    }
}
