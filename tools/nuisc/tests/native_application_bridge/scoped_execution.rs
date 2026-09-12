use super::*;
use std::path::Path;

fn source(compare: &str) -> String {
    format!(
        "mod cpu Main {{
      struct State {{ value: i64 }}
      @noinline
      fn advance(total: i64, index: i64, tag: i32, flag: bool, gain: f32, scale: f64) -> i64 {{
        if flag {{ return total + index; }}
        return total;
      }}
      fn start(initial: i64, limit: i64, stride: i64, seed: i64,
               tag: i32, flag: bool, gain: f32, scale: f64) -> State {{
        let index: i64 = initial;
        let total: i64 = seed;
        while index {compare} limit {{
          let total: i64 = advance(total, index, tag, flag, gain, scale);
          let index: i64 = index + stride;
        }}
        return State {{ value: total }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ return 0; }}
    }}"
    )
}

// Slots: start, bound, step, carry, i32, bool, f32 bits, f64 bits.
type Case = [u64; 8];

fn execute(compare: &str, cases: &[Case]) -> std::process::Output {
    let source = source(compare);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let node = compiled
        .yir
        .nodes
        .iter()
        .find(|n| n.op.instruction == "loop_while_i64_effect")
        .unwrap();
    assert_eq!(node.op.args[6], "scoped_call_i64_carry");
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    let start = llvm
        .find(&format!("define i64 @nuis_fn_{}(", node.op.args[8]))
        .unwrap();
    let insert = start + llvm[start..].find("{\n").unwrap() + 2;
    // Observe the actual call operands, not merely captures copied into final state.
    llvm.insert_str(
        insert,
        concat!(
            "  call void @nuis_debug_print_i64(i64 %arg0)\n",
            "  call void @nuis_debug_print_i64(i64 %arg1)\n",
            "  %probe_tag = sext i32 %arg2 to i64\n",
            "  call void @nuis_debug_print_i64(i64 %probe_tag)\n",
            "  %probe_flag = zext i1 %arg3 to i64\n",
            "  call void @nuis_debug_print_i64(i64 %probe_flag)\n",
            "  %probe_gain = bitcast float %arg4 to i32\n",
            "  %probe_gain_bits = zext i32 %probe_gain to i64\n",
            "  call void @nuis_debug_print_i64(i64 %probe_gain_bits)\n",
            "  %probe_scale = bitcast double %arg5 to i64\n",
            "  call void @nuis_debug_print_i64(i64 %probe_scale)\n",
        ),
    );
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [8 x i64], align 8\n  %out = alloca i64, align 8\n");
    for (index, case) in cases.iter().enumerate() {
        for (slot, word) in case.iter().enumerate() {
            llvm.push_str(&format!("  %p{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {}, ptr %p{index}_{slot}, align 8\n", *word as i64));
        }
        llvm.push_str(&format!("  %status{index} = call i32 @{}(ptr %args, i64 8, ptr %out, i64 1)\n  %wide{index} = zext i32 %status{index} to i64\n  call void @nuis_debug_print_i64(i64 %wide{index})\n  %result{index} = load i64, ptr %out, align 8\n  call void @nuis_debug_print_i64(i64 %result{index})\n", bridge.callbacks[0].symbol));
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
    dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0)
}

#[test]
fn scoped_native_calls_observe_pre_step_carry_and_all_five_exact_scalar_kinds() {
    for (compare, ranges) in [
        ("<", [(0_i64, 4_i64, 1_i64), (5, 2, 0), (-5, 1, 2)]),
        (">", [(4, 0, -1), (2, 5, 0), (5, -1, -2)]),
    ] {
        let mut cases = Vec::new();
        let mut expected = Vec::new();
        for (gain, scale) in [
            (1.5_f32.to_bits(), (-2.25_f64).to_bits()),
            (0x8000_0000, 0x8000_0000_0000_0000),
            (0x7fc0_1234, 0x7ff8_0000_0000_4321),
        ] {
            for enabled in [false, true] {
                for (initial, limit, step) in ranges {
                    let seed = i64::MAX;
                    cases.push([
                        initial as u64,
                        limit as u64,
                        step as u64,
                        seed as u64,
                        -17_i64 as u64,
                        u64::from(enabled),
                        u64::from(gain),
                        scale,
                    ]);
                    let mut current = initial;
                    let mut carry = seed;
                    while if compare == "<" {
                        current < limit
                    } else {
                        current > limit
                    } {
                        expected.extend([
                            carry as u64,
                            current as u64,
                            -17_i64 as u64,
                            u64::from(enabled),
                            u64::from(gain),
                            scale,
                        ]);
                        if enabled {
                            carry = carry.wrapping_add(current);
                        }
                        current += step;
                    }
                    expected.extend([0, carry as u64]);
                }
            }
        }
        let run = execute(compare, &cases);
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let output = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|line| line.parse::<i64>().unwrap() as u64)
            .collect::<Vec<_>>();
        assert_eq!(output, expected, "{compare}");
    }
}

#[test]
fn scoped_native_preflight_traps_before_the_first_helper_invocation() {
    for (initial, limit, stride) in [
        (0_i64, 5_i64, 0_i64),
        (0, 65537, 1),
        (i64::MAX - 1, i64::MAX, 2),
    ] {
        let run = execute(
            "<",
            &[[
                initial as u64,
                limit as u64,
                stride as u64,
                10,
                -17_i64 as u64,
                1,
                0,
                0,
            ]],
        );
        assert!(!run.status.success());
        assert!(
            run.stdout.is_empty(),
            "must not enter helper or publish output"
        );
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(run.status.signal().is_some());
        }
    }
}

#[test]
fn scoped_reference_fuel_failure_preserves_the_last_accepted_session_state() {
    let source = include_str!("scoped_loops.ns").replace("delta + 4", "delta + 1000");
    let project = Project::with_source(&source);
    let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
    emit_registered(&module, "counter").unwrap();
    let registry = yir_verify::default_registry();
    let (mut session, _) = ApplicationSession::open_registered(
        &module,
        &registry,
        "counter",
        vec![
            Value::Int(10),
            Value::I32(-17),
            Value::Bool(true),
            Value::F32(1.5),
            Value::F64(-2.25),
        ],
    )
    .unwrap();
    let accepted = session.state().clone();
    let error = session
        .event_budgeted(vec![Value::Int(3), Value::Bool(false)], 100)
        .unwrap_err();
    assert!(error.contains("budget"), "{error}");
    assert_eq!(session.state(), &accepted);
    session.close(vec![Value::Int(0)]).unwrap();
    assert!(session.completion_status().is_err());
}
