use super::*;
use std::{
    path::Path,
    process::Output,
    time::{Duration, Instant},
};

const REJECTED: i64 = -42424242;
type Case = (i64, i64, i64, bool);

fn source(compare: &str, operation: &str) -> String {
    format!(
        "mod cpu Main {{
      struct State {{ value: i64 }}
      @noinline
      fn walk(initial: i64, limit: i64, stride: i64, skip: bool) -> i64 {{
        if skip {{ return initial; }}
        let mut current = initial;
        while current {compare} limit {{ current = current {operation} stride; }}
        return current;
      }}
      fn start(initial: i64, limit: i64, stride: i64, skip: bool) -> State {{
        return State {{ value: walk(initial, limit, stride, skip) }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ return 0; }}
    }}"
    )
}

// Independent checked-step oracle, not a second copy of the closed-form proof.
fn simulate((initial, limit, step, skip): Case, compare: &str, op: &str) -> Option<(i64, i64)> {
    if skip {
        return Some((initial, 0));
    }
    let mut current = initial;
    for trips in 0..=65536 {
        let active = match compare {
            "==" => current == limit,
            "!=" => current != limit,
            "<" => current < limit,
            "<=" => current <= limit,
            ">" => current > limit,
            ">=" => current >= limit,
            _ => unreachable!(),
        };
        if !active {
            return Some((current, trips));
        }
        current = if op == "+" {
            current.checked_add(step)?
        } else {
            current.checked_sub(step)?
        };
    }
    None
}

fn cases() -> Vec<Case> {
    let mut cases = Vec::new();
    for start in [-6, -1, 0, 1, 6] {
        for limit in [-6, -1, 0, 1, 6] {
            for step in [-3, -1, 0, 1, 3] {
                cases.push((start, limit, step, false));
            }
        }
    }
    cases.extend([
        (i64::MIN, i64::MAX, 1, false),
        (i64::MAX, i64::MIN, 1, false),
        (i64::MIN, -1, i64::MAX, false),
        (-1, i64::MAX, i64::MIN, false),
        (0, i64::MIN, i64::MIN, false),
        (i64::MAX, i64::MAX, 1, false),
        (i64::MIN, i64::MIN, 1, false),
        (0, i64::MAX, i64::MAX - 1, false),
        (0, 65536, 1, false),
        (0, 65537, 1, false),
        (65536, 0, 1, false),
        (65537, 0, 1, false),
        (0, 5, 0, true),
        (5, 0, 0, true),
        (i64::MAX, i64::MAX, 1, true),
    ]);
    cases
}

fn driver(bridge: &NativeSessionBridge, helper: &str, cases: &[Case], probe: bool) -> String {
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let start = llvm
        .find(&format!("define i64 @nuis_fn_{helper}("))
        .unwrap();
    let end = start + llvm[start..].find("\n}").unwrap() + 2;
    let original = &llvm[start..end];
    assert!(original.contains("native_loop_preflight"));
    assert!(
        original.find("\nnative_loop_preflight").unwrap()
            < original.find("\nloop_while_i64_cond").unwrap()
    );
    let mut function = original.to_owned();
    if probe {
        // Observe the exact rejection branch without launching hundreds of
        // crashing processes. A separate test retains and executes real traps.
        assert!(function.contains("  call void @llvm.trap()\n  unreachable"));
        function = function.replace(
            "  call void @llvm.trap()\n  unreachable",
            &format!("  ret i64 {REJECTED}"),
        );
    }
    let label = function.find("\nloop_while_i64_body").unwrap();
    let insertion = label + function[label..].find(":\n").unwrap() + 2;
    function.insert_str(insertion,
        "  %probe_count = load volatile i64, ptr @probe_iterations, align 8\n  %probe_next = add i64 %probe_count, 1\n  store volatile i64 %probe_next, ptr @probe_iterations, align 8\n");
    llvm.replace_range(start..end, &function);
    llvm.push_str("\n@probe_iterations = internal global i64 0, align 8\n\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [4 x i64], align 8\n  %out = alloca i64, align 8\n");
    for (index, &(initial, limit, step, skip)) in cases.iter().enumerate() {
        for (slot, value) in [initial, limit, step, i64::from(skip)]
            .into_iter()
            .enumerate()
        {
            // Volatile transport prevents optimizer-only constant evaluation
            // from substituting for runtime induction admission in this proof.
            llvm.push_str(&format!("  %p{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {value}, ptr %p{index}_{slot}, align 8\n  %v{index}_{slot} = load volatile i64, ptr %p{index}_{slot}, align 8\n"));
        }
        if probe {
            llvm.push_str(&format!("  store volatile i64 0, ptr @probe_iterations, align 8\n  %skip{index} = icmp ne i64 %v{index}_3, 0\n  %result{index} = call i64 @nuis_fn_{helper}(i64 %v{index}_0, i64 %v{index}_1, i64 %v{index}_2, i1 %skip{index})\n  call void @nuis_debug_print_i64(i64 %result{index})\n  %trips{index} = load volatile i64, ptr @probe_iterations, align 8\n  call void @nuis_debug_print_i64(i64 %trips{index})\n"));
        } else {
            let symbol = &bridge.callbacks[0].symbol;
            llvm.push_str(&format!("  %status{index} = call i32 @{symbol}(ptr %args, i64 4, ptr %out, i64 1)\n  %status_wide{index} = zext i32 %status{index} to i64\n  call void @nuis_debug_print_i64(i64 %status_wide{index})\n"));
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    llvm
}

pub(super) fn run_bounded(binary: &Path, directory: &Path) -> Output {
    let stdout = directory.join("probe.stdout");
    let stderr = directory.join("probe.stderr");
    let mut child = Command::new(binary)
        .current_dir(directory)
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
            panic!("dynamic loop exceeded its test deadline");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Output {
        status,
        stdout: fs::read(stdout).unwrap(),
        stderr: fs::read(stderr).unwrap(),
    }
}

fn execute(compare: &str, operation: &str, cases: &[Case], probe: bool) -> Output {
    let source = source(compare, operation);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    let helper = &compiled
        .yir
        .nodes
        .iter()
        .find(|node| node.op.instruction == "call_i64")
        .unwrap()
        .op
        .args[0];
    let llvm = driver(&bridge, helper, cases, probe);
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out"),
        &source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &compiled.yir,
            llvm_ir: Some(&llvm),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    run_bounded(Path::new(&artifact.binary_path), &project.0)
}

#[test]
fn dynamic_loop_guard_matches_checked_simulation_before_any_iteration() {
    let cases = cases();
    for compare in ["==", "!=", "<", "<=", ">", ">="] {
        for operation in ["+", "-"] {
            let run = execute(compare, operation, &cases, true);
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(actual.len(), cases.len() * 2);
            for (&case, actual) in cases.iter().zip(actual.chunks_exact(2)) {
                let expected = simulate(case, compare, operation).unwrap_or((REJECTED, 0));
                assert_eq!(
                    actual,
                    [expected.0, expected.1],
                    "{case:?} {compare} {operation}"
                );
            }
        }
    }
}

#[test]
fn dynamic_induction_rejects_implicit_boolean_bounds() {
    let project = Project::with_source(&source("<", "+"));
    let mut module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    let loop_node = module
        .nodes
        .iter()
        .find(|n| n.op.instruction == "loop_while_i64")
        .unwrap()
        .name
        .clone();
    let function = module
        .functions
        .iter()
        .find(|f| f.body_nodes.contains(&loop_node))
        .unwrap();
    let condition = function
        .parameters
        .iter()
        .find(|p| p.ty == "bool")
        .unwrap()
        .node
        .clone();
    module
        .nodes
        .iter_mut()
        .find(|n| n.name == loop_node)
        .unwrap()
        .op
        .args[1] = condition.clone();
    for kind in [yir_core::EdgeKind::Dep, yir_core::EdgeKind::Effect] {
        module.edges.push(yir_core::Edge {
            kind,
            from: condition.clone(),
            to: loop_node.clone(),
        });
    }
    let error = emit_registered(&module, "counter").unwrap_err();
    assert!(error.contains("declared scalar kind"), "{error}");
}

#[test]
fn invalid_dynamic_induction_traps_instead_of_returning_a_callback_error() {
    for (compare, operation, case) in [
        ("<", "+", (0, 5, 0, false)),
        ("<", "-", (0, 5, 1, false)),
        ("!=", "+", (0, 3, 2, false)),
        ("<=", "+", (i64::MAX, i64::MAX, 1, false)),
        (">=", "-", (i64::MIN, i64::MIN, 1, false)),
        ("<", "+", (0, 65537, 1, false)),
    ] {
        let run = execute(compare, operation, &[case], false);
        assert!(!run.status.success(), "{case:?} must trap");
        assert!(
            run.stdout.is_empty(),
            "must not publish a result after rejection"
        );
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                run.status.signal().is_some(),
                "must be a process trap, not a status code"
            );
        }
    }
}
