use super::*;
use std::path::Path;

fn source(descending: bool, nested: bool, multi_state: bool) -> String {
    let comparison = if descending { ">" } else { "<" };
    let step = if descending { "-" } else { "+" };
    let exits = if multi_state {
        format!(
            "if index == stop {{
          let total: i64 = total + 10;
          let checksum: i64 = checksum + total;
          break;
        }}
        if index == initial {{
          let total: i64 = total + 2;
          let checksum: i64 = checksum + total;
          let index: i64 = index {step} 1;
          continue;
        }}"
        )
    } else {
        "if index == stop { break; }".to_owned()
    };
    let prefix = if nested {
        "let inner: i64 = 0;
         while inner < 3 {
           let total: i64 = sum(total, index);
           if inner == 1 { break; }
           let inner: i64 = inner + 1;
         }
         let checksum: i64 = sum(checksum, inner);"
    } else {
        "let total: i64 = sum(total, index);
         let checksum: i64 = sum(checksum, total);"
    };
    format!(
        "mod cpu Main {{
      struct State {{ index: i64, total: i64, checksum: i64 }}
      fn sum(a: i64, b: i64) -> i64 {{ return a + b; }}
      fn start(initial: i64, limit: i64, stop: i64, seed: i64) -> State {{
        let index: i64 = initial;
        let total: i64 = seed;
        let checksum: i64 = 0;
        while index {comparison} limit {{
          {prefix}
          {exits}
          let total: i64 = total + 1;
          let index: i64 = index {step} 1;
        }}
        return State {{ index: index, total: total, checksum: checksum }};
      }}
      fn step(state: State) -> State {{ return state; }}
      fn stop(state: State) -> State {{ return state; }}
      fn main() -> i64 {{ print(999); return 0; }}
    }}"
    )
}

#[test]
fn scalar_break_source_keeps_current_skips_suffix_and_isolates_nested_exits() {
    for descending in [false, true] {
        for nested in [false, true] {
            for multi_state in [false, true] {
                let source = source(descending, nested, multi_state);
                let project = Project::with_source(&source);
                let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
                let bridge = emit_registered(&compiled.yir, "counter").unwrap();
                let mut llvm = bridge.llvm_ir.replacen(
                    "define i64 @nuis_yir_entry()",
                    "define i64 @unused_native_entry()",
                    1,
                );
                llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [4 x i64], align 8\n  %out = alloca [3 x i64], align 8\n");
                let registry = yir_verify::default_registry();
                let mut expected = Vec::new();
                let cases = if descending {
                    [
                        (4_i64, 0_i64, 4_i64),
                        (4, 0, 2),
                        (4, 0, 1),
                        (4, 0, 9),
                        (0, 4, 0),
                    ]
                } else {
                    [(0, 4, 0), (0, 4, 2), (0, 4, 3), (0, 4, 9), (4, 0, 4)]
                };
                for (case, (initial, limit, stop)) in cases.into_iter().enumerate() {
                    let (mut index, mut total, mut checksum) = (initial, 10_i64, 0_i64);
                    while if descending {
                        index > limit
                    } else {
                        index < limit
                    } {
                        if nested {
                            total += index * 2;
                            checksum += 1;
                        } else {
                            total += index;
                            checksum += total;
                        }
                        if index == stop {
                            if multi_state {
                                total += 10;
                                checksum += total;
                            }
                            break;
                        }
                        if multi_state && index == initial {
                            total += 2;
                            checksum += total;
                            index += if descending { -1 } else { 1 };
                            continue;
                        }
                        total += 1;
                        index += if descending { -1 } else { 1 };
                    }
                    let result = [index as u64, total as u64, checksum as u64];
                    let args = [initial, limit, stop, 10];
                    let (session, _) = ApplicationSession::open_registered(
                        &compiled.yir,
                        &registry,
                        "counter",
                        args.into_iter().map(Value::Int).collect(),
                    )
                    .unwrap();
                    assert_eq!(state_words(session.state()), result);
                    expected.push(0);
                    expected.extend(result);
                    for (slot, word) in args.into_iter().enumerate() {
                        llvm.push_str(&format!("  %p{case}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %p{case}_{slot}, align 8\n"));
                    }
                    llvm.push_str(&format!("  %status{case} = call i32 @{}(ptr %args, i64 4, ptr %out, i64 3)\n  %wide{case} = zext i32 %status{case} to i64\n  call void @nuis_debug_print_i64(i64 %wide{case})\n", bridge.callbacks[0].symbol));
                    for slot in 0..3 {
                        llvm.push_str(&format!("  %o{case}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %v{case}_{slot} = load i64, ptr %o{case}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{case}_{slot})\n"));
                    }
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
                let run =
                    dynamic_loop_guard::run_bounded(Path::new(&artifact.binary_path), &project.0);
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
                assert_eq!(
                    actual, expected,
                    "descending={descending} nested={nested} multi_state={multi_state}"
                );
            }
        }
    }
}
