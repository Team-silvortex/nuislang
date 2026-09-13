use super::*;
use std::path::Path;

pub(super) fn source(slots: usize, compare: &str) -> String {
    let fields = (0..slots)
        .map(|i| format!("carry{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let params = (0..slots)
        .rev()
        .map(|i| format!("c{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let kept = (0..slots)
        .map(|i| format!("carry{i}: c{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let updates = (0..slots)
        .map(|i| {
            let delta = if i == 0 {
                "index".to_owned()
            } else {
                format!("v{}", i - 1)
            };
            format!("let v{i}: i64 = sum(c{i}, {delta});")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let returned = (0..slots)
        .map(|i| format!("carry{i}: v{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let seeds = (0..slots)
        .map(|i| format!("seed{i}: i64"))
        .collect::<Vec<_>>()
        .join(", ");
    let bindings = (0..slots)
        .map(|i| format!("let s{i}: i64 = seed{i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let operands = (0..slots)
        .rev()
        .map(|i| format!("s{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let projections = (0..slots)
        .map(|i| format!("let s{i}: i64 = returned.carry{i};"))
        .collect::<Vec<_>>()
        .join("\n");
    let state = (0..slots)
        .map(|i| format!("carry{i}: s{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    format!("mod cpu Main {{
      struct Carries {{ {fields} }}
      struct State {{ {fields} }}
      @noinline
      fn sum(left: i64, right: i64) -> i64 {{ return left + right; }}
      fn advance({params}, index: i64, tag: i32, flag: bool, gain: f32, scale: f64) -> Carries {{
        if !flag {{ return Carries {{ {kept} }}; }}
        {updates}
        return Carries {{ {returned} }};
      }}
      fn start(initial: i64, limit: i64, stride: i64, {seeds}, tag: i32, flag: bool, gain: f32, scale: f64) -> State {{
        let index: i64 = initial;
        {bindings}
        while index {compare} limit {{
          let returned: Carries = advance({operands}, index, tag, flag, gain, scale);
          {projections}
          let index: i64 = index + stride;
        }}
        return State {{ {state} }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ print(999); return 0; }}
    }}")
}

// The wrappers delegate to the real allocator/drop functions. Counting actual
// calls tests lifecycle balance without substituting a fake aggregate runtime.
const ALLOCATION_PROBE: &str = "
declare i32 @fflush(ptr)
@probe_allocs = internal global i64 0
@probe_drops = internal global i64 0
define ptr @probe_alloc(i64 %count) {
  %old = load i64, ptr @probe_allocs
  %next = add i64 %old, 1
  store i64 %next, ptr @probe_allocs
  %result = call ptr @nuis_scheduler_owned_aggregate_alloc_v1(i64 %count)
  ret ptr %result
}
define void @probe_drop(ptr %aggregate) {
  call void @nuis_scheduler_owned_aggregate_drop_v1(ptr %aggregate)
  %old = load i64, ptr @probe_drops
  %next = add i64 %old, 1
  store i64 %next, ptr @probe_drops
  ret void
}
";

fn execute(slots: usize, compare: &str, cases: &[Vec<u64>]) -> std::process::Output {
    let source = source(slots, compare);
    execute_with(&source, slots, cases, |_| {})
}

pub(super) fn execute_with(
    source: &str,
    slots: usize,
    cases: &[Vec<u64>],
    prepare: impl FnOnce(&mut YirModule),
) -> std::process::Output {
    execute_observed(source, slots, cases, prepare, None)
}

pub(super) fn execute_observed(
    source: &str,
    slots: usize,
    cases: &[Vec<u64>],
    prepare: impl FnOnce(&mut YirModule),
    observed: Option<&str>,
) -> std::process::Output {
    let project = Project::with_source(source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    let mut module = compiled.yir.clone();
    prepare(&mut module);
    let node = module
        .nodes
        .iter()
        .find(|n| {
            matches!(
                n.op.args.get(6).map(String::as_str),
                Some("scoped_call_i64_carries" | "scoped_call_i64_carries_break")
            )
        })
        .unwrap();
    let bridge = emit_registered(&module, "counter").unwrap();
    assert_eq!(bridge.callbacks[0].arguments.len(), slots + 7);
    let mut llvm = bridge
        .llvm_ir
        .replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        )
        .replace(
            "call ptr @nuis_scheduler_owned_aggregate_alloc_v1(",
            "call ptr @probe_alloc(",
        )
        .replace(
            "call void @nuis_scheduler_owned_aggregate_drop_v1(",
            "call void @probe_drop(",
        );
    let callee = observed.unwrap_or(&node.op.args[8]);
    let start = llvm
        .find(&format!("define i64 @nuis_fn_{callee}("))
        .unwrap();
    let insert = start + llvm[start..].find("{\n").unwrap() + 2;
    let mut probe = String::from("  %probe_a = load i64, ptr @probe_allocs\n  %probe_d = load i64, ptr @probe_drops\n  %probe_live = sub i64 %probe_a, %probe_d\n  call void @nuis_debug_print_i64(i64 %probe_live)\n");
    for index in 0..=slots {
        probe.push_str(&format!(
            "  call void @nuis_debug_print_i64(i64 %arg{index})\n"
        ));
    }
    probe.push_str(&format!("  %probe_tag = sext i32 %arg{} to i64\n  call void @nuis_debug_print_i64(i64 %probe_tag)\n  %probe_flag = zext i1 %arg{} to i64\n  call void @nuis_debug_print_i64(i64 %probe_flag)\n  %probe_gain = bitcast float %arg{} to i32\n  %probe_gain_bits = zext i32 %probe_gain to i64\n  call void @nuis_debug_print_i64(i64 %probe_gain_bits)\n  %probe_scale = bitcast double %arg{} to i64\n  call void @nuis_debug_print_i64(i64 %probe_scale)\n", slots + 1, slots + 2, slots + 3, slots + 4));
    // A real llvm.trap must not discard buffered evidence of helper entry.
    probe.push_str("  %probe_flushed = call i32 @fflush(ptr null)\n");
    llvm.insert_str(insert, &probe);
    llvm.push_str(ALLOCATION_PROBE);
    llvm.push_str(&format!("\ndefine i64 @nuis_yir_entry() {{\n  %args = alloca [{} x i64], align 8\n  %out = alloca [{slots} x i64], align 8\n", slots + 7));
    for (index, case) in cases.iter().enumerate() {
        assert_eq!(case.len(), slots + 7);
        for (slot, word) in case.iter().enumerate() {
            llvm.push_str(&format!("  %p{index}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {}, ptr %p{index}_{slot}, align 8\n", *word as i64));
        }
        llvm.push_str(&format!("  %status{index} = call i32 @{}(ptr %args, i64 {}, ptr %out, i64 {slots})\n  %wide{index} = zext i32 %status{index} to i64\n  call void @nuis_debug_print_i64(i64 %wide{index})\n", bridge.callbacks[0].symbol, slots + 7));
        for slot in 0..slots {
            llvm.push_str(&format!("  %o{index}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %r{index}_{slot} = load i64, ptr %o{index}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %r{index}_{slot})\n"));
        }
        for counter in ["allocs", "drops"] {
            llvm.push_str(&format!("  %{counter}{index} = load i64, ptr @probe_{counter}\n  call void @nuis_debug_print_i64(i64 %{counter}{index})\n"));
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    let artifact = nuisc::aot::write_and_link_with_source(
        &project.0.join("main.ns"),
        &project.0.join("out"),
        source,
        nuisc::aot::AotCompileProgram {
            ast: &compiled.ast,
            nir: &compiled.nir,
            yir: &module,
            llvm_ir: Some(&llvm),
        },
        &nuisc::aot::host_cpu_build_target(),
    )
    .unwrap();
    dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0)
}

#[test]
fn multi_carry_native_calls_preserve_sequential_values_exact_captures_and_drop_balance() {
    for slots in [2, 3, 7] {
        for (compare, ranges) in [
            (
                "<",
                [(0_i64, 4_i64, 1_i64), (5, 2, 0), (-5, 1, 2), (3, 4, 1)],
            ),
            (">", [(4, 0, -1), (2, 5, 0), (5, -1, -2), (4, 3, -1)]),
        ] {
            let mut cases = Vec::new();
            let mut expected = Vec::new();
            let mut allocations = 0_u64;
            for (gain, scale) in [
                (1.5_f32.to_bits(), (-2.25_f64).to_bits()),
                (0x8000_0000, 0x8000_0000_0000_0000),
                (0x7fc0_1234, 0x7ff8_0000_0000_4321),
            ] {
                for flag in [false, true] {
                    for (initial, limit, stride) in ranges {
                        let mut carry = (0..slots)
                            .map(|i| if i == 0 { i64::MAX } else { -3 * i as i64 })
                            .collect::<Vec<_>>();
                        let captures = [-17_i64 as u64, u64::from(flag), u64::from(gain), scale];
                        let mut case = vec![initial as u64, limit as u64, stride as u64];
                        case.extend(carry.iter().map(|v| *v as u64));
                        case.extend(captures);
                        cases.push(case);
                        let mut index = initial;
                        while if compare == "<" {
                            index < limit
                        } else {
                            index > limit
                        } {
                            expected.push(0); // Previous returned aggregate must already be dropped.
                            expected.extend(carry.iter().rev().map(|v| *v as u64));
                            expected.push(index as u64);
                            expected.extend(captures);
                            if flag {
                                let mut delta = index;
                                for value in &mut carry {
                                    *value = value.wrapping_add(delta);
                                    delta = *value;
                                }
                            }
                            index += stride;
                            allocations += 1;
                        }
                        allocations += 1; // The callback's final State is also unpacked and dropped.
                        expected.push(0);
                        expected.extend(carry.iter().map(|v| *v as u64));
                        expected.extend([allocations, allocations]);
                    }
                }
            }
            let run = execute(slots, compare, &cases);
            assert!(
                run.status.success(),
                "{}",
                String::from_utf8_lossy(&run.stderr)
            );
            let actual = String::from_utf8(run.stdout)
                .unwrap()
                .lines()
                .map(|line| line.parse::<i64>().unwrap() as u64)
                .collect::<Vec<_>>();
            assert_eq!(actual, expected, "{slots} carries, {compare}");
        }
    }
}

#[test]
fn multi_carry_native_preflight_traps_before_any_call_or_output() {
    for (initial, limit, stride) in [
        (0_i64, 5_i64, 0_i64),
        (0, 65537, 1),
        (i64::MAX - 1, i64::MAX, 2),
    ] {
        let run = execute(
            2,
            "<",
            &[vec![
                initial as u64,
                limit as u64,
                stride as u64,
                10,
                20,
                -17_i64 as u64,
                1,
                0,
                0,
            ]],
        );
        assert!(!run.status.success());
        assert!(
            run.stdout.is_empty(),
            "must not call helper or publish output"
        );
        #[cfg(unix)]
        {
            use std::os::unix::process::ExitStatusExt;
            assert!(run.status.signal().is_some());
        }
    }
}
