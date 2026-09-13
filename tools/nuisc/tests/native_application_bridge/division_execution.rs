use super::*;
use std::path::Path;

#[derive(Clone, Copy, Debug)]
pub(super) enum Shape {
    Scalar,
    Aggregate,
    Discarded,
    Argument,
    Prefix,
}

pub(super) fn source(op: &str, shape: Shape) -> String {
    let (result, body, publish) = match shape {
        Shape::Aggregate => (
            "Carries",
            "if !enabled { return Carries { carry0: seed, carry1: seed }; }
             return Carries { carry0: leaf(a, b), carry1: seed };",
            "let next: Carries = compute(enabled, a, b, seed);
             return State { value: next.carry0, seed: next.carry1 };",
        ),
        _ => (
            "i64",
            match shape {
                Shape::Scalar => "if !enabled { return seed; } return leaf(a, b);",
                Shape::Discarded => {
                    "if !enabled { return seed; }
                    let unused: i64 = leaf(a, b); return seed;"
                }
                Shape::Argument => "if !enabled { return seed; } return ignore(leaf(a, b));",
                Shape::Prefix => {
                    "let value: i64 = leaf(a, b);
                    if !enabled { return seed; } return value;"
                }
                Shape::Aggregate => unreachable!(),
            },
            "return State { value: compute(enabled, a, b, seed), seed: seed };",
        ),
    };
    format!(
        "mod cpu Main {{
      struct State {{ value: i64, seed: i64 }}
      struct Carries {{ carry0: i64, carry1: i64 }}
      @noinline
      fn leaf(a: i64, b: i64) -> i64 {{ return a {op} b; }}
      @noinline
      fn ignore(value: i64) -> i64 {{ return 7; }}
      @noinline
      fn compute(enabled: bool, a: i64, b: i64, seed: i64) -> {result} {{ {body} }}
      fn start(enabled: bool, a: i64, b: i64, seed: i64) -> State {{ {publish} }}
      fn step(state: State, enabled: bool, a: i64, b: i64) -> State {{
        let seed: i64 = state.value; {publish}
      }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ print(999); return 0; }}
    }}"
    )
}

#[derive(Clone, Copy)]
struct Case {
    enabled: bool,
    left: i64,
    right: i64,
}

fn expected(op: &str, shape: Shape, case: Case) -> Result<(i64, bool), &'static str> {
    let entered = case.enabled || matches!(shape, Shape::Prefix);
    if !entered {
        return Ok((41, false));
    }
    if case.right == 0 {
        return Err("zero");
    }
    if (case.left, case.right) == (i64::MIN, -1) {
        return Err("overflow");
    }
    let (a, b) = (i128::from(case.left), i128::from(case.right));
    let result = if op == "/" { a / b } else { a % b } as i64;
    let result = if !case.enabled || matches!(shape, Shape::Discarded) {
        41
    } else if matches!(shape, Shape::Argument) {
        7
    } else {
        result
    };
    Ok((result, true))
}

fn execute(op: &str, shape: Shape, cases: &[Case], trap: bool) {
    execute_variant(op, shape, cases, trap, false);
}

fn execute_variant(op: &str, shape: Shape, cases: &[Case], trap: bool, literal: bool) {
    let mut source = source(op, shape);
    if literal {
        let (left, right) = if cases[0].right == 0 {
            ("7", "0")
        } else {
            ("(0 - 9223372036854775807 - 1)", "(0 - 1)")
        };
        source = source.replace(
            &format!("return a {op} b;"),
            &format!("return {left} {op} {right};"),
        );
    }
    let project = Project::with_source(&source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(bridge.llvm_ir.contains("integer_divisor_invalid"));
    assert!(bridge
        .llvm_ir
        .contains(if op == "/" { "sdiv i64" } else { "srem i64" }));
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let start = llvm.find("define i64 @nuis_fn_leaf(").unwrap();
    let insert = start + llvm[start..].find("{\n").unwrap() + 2;
    // Flush real helper-entry evidence before a possible process trap.
    llvm.insert_str(insert, "  call void @nuis_debug_print_i64(i64 93)\n  call void @nuis_debug_print_i64(i64 %arg0)\n  call void @nuis_debug_print_i64(i64 %arg1)\n  %flushed = call i32 @fflush(ptr null)\n");
    llvm.push_str("\ndeclare i32 @fflush(ptr)\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [4 x i64], align 8\n  %out = alloca [2 x i64], align 8\n");
    let registry = yir_verify::default_registry();
    let mut oracle = Vec::new();
    for (index, case) in cases.iter().copied().enumerate() {
        let reference = ApplicationSession::open_registered(
            &compiled.yir,
            &registry,
            "counter",
            vec![
                Value::Bool(case.enabled),
                Value::Int(case.left),
                Value::Int(case.right),
                Value::Int(41),
            ],
        );
        match expected(op, shape, case) {
            Ok((value, entered)) => {
                assert!(!trap);
                let (session, _) = reference.unwrap_or_else(|error| {
                    panic!(
                        "op={op} shape={shape:?} enabled={} left={} right={}: {error}",
                        case.enabled, case.left, case.right,
                    )
                });
                assert_eq!(state_words(session.state()), [value as u64, 41]);
                if entered {
                    oracle.extend([93, case.left, case.right]);
                }
                oracle.extend([0, value, 41]);
            }
            Err(wanted) => {
                assert!(trap);
                let error = reference
                    .err()
                    .expect("reference must reject invalid arithmetic");
                assert!(error.contains(wanted), "{error}");
                oracle.extend([93, case.left, case.right]);
            }
        }
        for (slot, word) in [i64::from(case.enabled), case.left, case.right, 41]
            .into_iter()
            .enumerate()
        {
            llvm.push_str(&format!("  %p{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %p{index}_{slot}, align 8\n"));
        }
        llvm.push_str(&format!("  %status{index} = call i32 @{}(ptr %args, i64 4, ptr %out, i64 2)\n  %wide{index} = zext i32 %status{index} to i64\n  call void @nuis_debug_print_i64(i64 %wide{index})\n", bridge.callbacks[0].symbol));
        for slot in 0..2 {
            llvm.push_str(&format!("  %o{index}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %v{index}_{slot} = load i64, ptr %o{index}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{index}_{slot})\n"));
        }
        llvm.push_str(&format!("  %flush{index} = call i32 @fflush(ptr null)\n"));
    }
    llvm.push_str("  ret i64 0\n}\n");
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
    let run = dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0);
    assert_eq!(
        run.status.success(),
        !trap,
        "op={op} shape={shape:?}: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    if trap {
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(
                matches!(run.status.signal(), Some(4 | 5)),
                "{:?}",
                run.status
            );
        }
    }
    let actual = String::from_utf8(run.stdout)
        .unwrap()
        .lines()
        .map(|line| line.parse::<i64>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(actual, oracle, "op={op} shape={shape:?}");
}

#[test]
fn checked_division_remainder_match_wide_oracle_and_skip_unselected_invalid_values() {
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -17,
        -3,
        -1,
        0,
        1,
        3,
        17,
        i64::MAX - 1,
        i64::MAX,
    ];
    for op in ["/", "%"] {
        for shape in [
            Shape::Scalar,
            Shape::Aggregate,
            Shape::Discarded,
            Shape::Argument,
        ] {
            let mut cases = Vec::new();
            for enabled in [false, true] {
                for left in values {
                    for right in values {
                        let case = Case {
                            enabled,
                            left,
                            right,
                        };
                        if expected(op, shape, case).is_ok() {
                            cases.push(case);
                        }
                    }
                }
            }
            assert_eq!(cases.len(), 230);
            execute(op, shape, &cases, false);
        }
    }
}

#[test]
fn selected_invalid_arithmetic_traps_even_for_unused_results_and_arguments() {
    for op in ["/", "%"] {
        for shape in [
            Shape::Scalar,
            Shape::Aggregate,
            Shape::Discarded,
            Shape::Argument,
            Shape::Prefix,
        ] {
            for (left, right) in [(7, 0), (i64::MIN, -1)] {
                execute(
                    op,
                    shape,
                    &[Case {
                        enabled: !matches!(shape, Shape::Prefix),
                        left,
                        right,
                    }],
                    true,
                );
            }
        }
    }
}

#[test]
fn invalid_constant_arithmetic_is_not_folded_away_or_hoisted_across_a_guard() {
    for op in ["/", "%"] {
        for (left, right) in [(7, 0), (i64::MIN, -1)] {
            for enabled in [false, true] {
                execute_variant(
                    op,
                    Shape::Scalar,
                    &[Case {
                        enabled,
                        left,
                        right,
                    }],
                    enabled,
                    true,
                );
            }
        }
    }
}

#[test]
fn reference_arithmetic_failure_preserves_the_last_accepted_session_state() {
    for op in ["/", "%"] {
        let project = Project::with_source(&source(op, Shape::Aggregate));
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        emit_registered(&module, "counter").unwrap();
        let registry = yir_verify::default_registry();
        for (left, right, diagnostic) in [(7, 0, "zero"), (i64::MIN, -1, "overflow")] {
            let (mut session, _) = ApplicationSession::open_registered(
                &module,
                &registry,
                "counter",
                vec![
                    Value::Bool(false),
                    Value::Int(0),
                    Value::Int(0),
                    Value::Int(41),
                ],
            )
            .unwrap();
            session
                .event(vec![Value::Bool(true), Value::Int(17), Value::Int(3)])
                .unwrap();
            let accepted = session.state().clone();
            let error = session
                .event(vec![Value::Bool(true), Value::Int(left), Value::Int(right)])
                .unwrap_err();
            assert!(error.contains(diagnostic), "{error}");
            assert_eq!(session.state(), &accepted);
            session.close(vec![]).unwrap();
            assert!(session.completion_status().is_err());
        }
    }
}
