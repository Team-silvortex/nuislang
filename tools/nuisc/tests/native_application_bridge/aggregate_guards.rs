use super::*;
use std::path::Path;

const SOURCE: &str = "mod cpu Main {
    struct Carries { carry0: i64, carry1: i64 }
    struct State { left: i64, right: i64 }
    @noinline
    fn choose(enabled: bool, limit: i64, a: i64, b: i64) -> Carries {
        if !enabled { return Carries { carry0: a, carry1: b }; }
        let index: i64 = 0;
        while index < limit { let index: i64 = index + 1; }
        return Carries { carry0: a + index, carry1: b + index };
    }
    fn start(enabled: bool, limit: i64, a: i64, b: i64) -> State {
        let values: Carries = choose(enabled, limit, a, b);
        return State { left: values.carry0, right: values.carry1 };
    }
    fn step(state: State) -> State { return state; }
    fn stop(state: State) -> State { return state; }
    fn main() -> i64 { print(999); return 0; }
}";

#[test]
fn aggregate_early_return_skips_unselected_loop_preflight_but_selected_path_traps() {
    for trap in [false, true] {
        let project = Project::with_source(SOURCE);
        let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        compiled.yir.nodes.reverse();
        compiled.yir.functions.reverse();
        for function in &mut compiled.yir.functions {
            function.body_nodes.reverse();
        }
        assert!(compiled
            .yir
            .nodes
            .iter()
            .any(|n| { n.op.instruction == "call_owned_struct" && n.op.args[0] == "choose" }));
        let bridge = emit_registered(&compiled.yir, "counter").unwrap();
        assert!(bridge.llvm_ir.contains("native_loop_preflight"));
        let mut llvm = bridge.llvm_ir.replacen(
            "define i64 @nuis_yir_entry()",
            "define i64 @unused_native_entry()",
            1,
        );
        llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [4 x i64], align 8\n  %out = alloca [2 x i64], align 8\n");
        let cases = if trap {
            vec![(true, 1_000_000_i64)]
        } else {
            vec![(false, i64::MAX), (false, 1_000_000), (true, 4), (true, 0)]
        };
        let registry = yir_verify::default_registry();
        let mut expected = Vec::new();
        for (case, (enabled, limit)) in cases.into_iter().enumerate() {
            if !trap {
                let (session, _) = ApplicationSession::open_registered(
                    &compiled.yir,
                    &registry,
                    "counter",
                    vec![
                        Value::Bool(enabled),
                        Value::Int(limit),
                        Value::Int(10),
                        Value::Int(20),
                    ],
                )
                .unwrap();
                let delta = if enabled { limit } else { 0 };
                let result = [(10 + delta) as u64, (20 + delta) as u64];
                assert_eq!(state_words(session.state()), result);
                expected.push(0);
                expected.extend(result);
            }
            for (slot, word) in [i64::from(enabled), limit, 10, 20].into_iter().enumerate() {
                llvm.push_str(&format!("  %p{case}_{slot} = getelementptr i64, ptr %args, i64 {slot}\n  store volatile i64 {word}, ptr %p{case}_{slot}, align 8\n"));
            }
            llvm.push_str(&format!("  %status{case} = call i32 @{}(ptr %args, i64 4, ptr %out, i64 2)\n  %wide{case} = zext i32 %status{case} to i64\n  call void @nuis_debug_print_i64(i64 %wide{case})\n", bridge.callbacks[0].symbol));
            for slot in 0..2 {
                llvm.push_str(&format!("  %o{case}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %v{case}_{slot} = load i64, ptr %o{case}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{case}_{slot})\n"));
            }
        }
        llvm.push_str("  ret i64 0\n}\n");
        let artifact = nuisc::aot::write_and_link_with_source(
            &project.0.join("main.ns"),
            &project.0.join("out"),
            SOURCE,
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
        if trap {
            assert!(!run.status.success(), "selected excessive loop must trap");
            #[cfg(unix)]
            {
                use std::os::unix::process::ExitStatusExt;
                assert!(
                    matches!(run.status.signal(), Some(4 | 5)),
                    "{:?}",
                    run.status
                );
            }
            assert!(run.stdout.is_empty(), "trap must not return callback state");
        } else {
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
            assert_eq!(actual, expected);
        }
    }
}
