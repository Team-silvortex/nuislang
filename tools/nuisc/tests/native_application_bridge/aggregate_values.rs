use super::*;

pub(super) fn definition(llvm: &str, callee: &str) -> usize {
    let symbol = format!(" @nuis_fn_{callee}(");
    llvm.match_indices("define ")
        .find(|(start, _)| llvm[*start..].lines().next().unwrap().contains(&symbol))
        .map(|(start, _)| start)
        .expect("native helper definition")
}

#[test]
fn native_returns_are_values_while_ordinary_abi_stays_owned() {
    for slots in [1, 2, 7, 64] {
        let fields = (0..slots)
            .map(|i| format!("f{i}: i64"))
            .collect::<Vec<_>>()
            .join(", ");
        let values = (0..slots)
            .rev()
            .map(|i| format!("f{i}: seed + {i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let advanced = values.replace("seed +", "(seed + 10) +");
        let source = format!(
            "mod cpu Main {{
            struct Packet {{ {fields} }} struct State {{ value: i64 }}
            @noinline fn packet(seed: i64, skip: bool) -> Packet {{
                if skip {{ return Packet {{ {values} }}; }}
                return Packet {{ {advanced} }};
            }}
            fn start(seed: i64, skip: bool) -> State {{
                let first = packet(seed, skip);
                let second = packet(seed + 100, !skip);
                return stop(State {{ value: first.f0 + first.f{} + second.f0 }});
            }}
            fn step(state: State) -> State {{ return state; }}
            @noinline fn stop(state: State) -> State {{ return state; }}
            fn main() -> i64 {{ return 0; }}
        }}",
            slots - 1
        );
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let bridge = emit_registered(&compiled.yir, "counter").unwrap();
        let native = &bridge.llvm_ir;
        assert!(native.contains(&format!("define [{slots} x i64] @nuis_fn_packet(")));
        assert!(native.contains(&format!("call [{slots} x i64] @nuis_fn_packet(")));
        let start = definition(native, "packet");
        let body = &native[start..start + native[start..].find("\n}\n").unwrap()];
        assert!(!body.contains("@nuis_scheduler_owned_aggregate_"));
        assert!(!body.contains("alloca "));
        assert!(!body.contains("ptrtoint "));
        assert!(native.contains("define [1 x i64] @nuis_fn_start("));
        assert!(native.contains("call [1 x i64] @nuis_fn_stop("));
        assert!(!native.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
        let ordinary = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
        assert!(ordinary.contains("define i64 @nuis_fn_packet("));
        assert!(ordinary.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));

        let mut llvm = native
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
        llvm.push_str(multi_execution::ALLOCATION_PROBE);
        llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %args = alloca [2 x i64], align 8\n  %out = alloca i64, align 8\n  %flag = getelementptr i64, ptr %args, i64 1\n  store i64 3, ptr %args, align 8\n");
        let mut expected = Vec::new();
        let registry = yir_verify::default_registry();
        for skip in 0..=1 {
            let value = if skip == 0 { 129 } else { 119 } + slots as i64 - 1;
            llvm.push_str(&format!("  store i64 {skip}, ptr %flag, align 8\n  %s{skip} = call i32 @{}(ptr %args, i64 2, ptr %out, i64 1)\n  %status{skip} = zext i32 %s{skip} to i64\n  call void @nuis_debug_print_i64(i64 %status{skip})\n  %v{skip} = load i64, ptr %out\n  call void @nuis_debug_print_i64(i64 %v{skip})\n  %a{skip} = load i64, ptr @probe_allocs\n  %d{skip} = load i64, ptr @probe_drops\n  call void @nuis_debug_print_i64(i64 %a{skip})\n  call void @nuis_debug_print_i64(i64 %d{skip})\n", bridge.callbacks[0].symbol));
            expected.extend([0, value, 0, 0]);
            let (reference, _) = ApplicationSession::open_registered(
                &compiled.yir,
                &registry,
                "counter",
                vec![Value::Int(3), Value::Bool(skip != 0)],
            )
            .unwrap();
            assert_eq!(state_words(reference.state()), vec![value as u64]);
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
        let run = dynamic_loop_guard::run_bounded(
            std::path::Path::new(&artifact.binary_path),
            &project.0,
        );
        assert!(
            run.status.success(),
            "{}",
            String::from_utf8_lossy(&run.stderr)
        );
        let actual = String::from_utf8(run.stdout)
            .unwrap()
            .lines()
            .map(|v| v.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "slots={slots}");
    }
}
