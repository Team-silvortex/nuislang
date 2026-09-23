use super::*;

#[test]
fn typed_capture_words_execute_high_bits_word_boundaries_and_independent_flags() {
    let fields = (0..64)
        .map(|i| format!("f{i}: bool"))
        .collect::<Vec<_>>()
        .join(", ");
    let params = (0..64)
        .map(|i| format!("p{i}: bool"))
        .collect::<Vec<_>>()
        .join(", ");
    let values = (0..64)
        .rev()
        .map(|i| format!("f{i}: p{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let flipped = (0..64)
        .rev()
        .map(|i| format!("f{i}: !value.f{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "mod cpu Main {{
        struct Flags {{ {fields} }}
        struct State {{ flags: Flags }}
        @noinline fn relay(value: Flags) -> Flags {{ return value; }}
        @noinline fn packet(value: Flags) -> Flags {{
            let next = value;
            if value.f0 {{ let next = relay(value); }}
            else {{ let next = relay(Flags {{ {flipped} }}); }}
            return next;
        }}
        fn start({params}) -> State {{ return State {{ flags: packet(Flags {{ {values} }}) }}; }}
        fn step(state: State) -> State {{ return state; }}
        fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}"
    );
    let project = Project::with_source(&source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let selection = compiled
        .yir
        .functions
        .iter()
        .find(|f| f.name.starts_with("__nuis_conditional_value"))
        .unwrap();
    assert_eq!(selection.parameters.len(), 2);
    assert!(selection.parameters.iter().all(|p| p.ty == "i64"));
    let packet = compiled
        .yir
        .functions
        .iter()
        .find(|f| f.name == "packet")
        .unwrap();
    assert_eq!(packet.parameters.len(), 64);
    let bridge = emit_registered(&compiled.yir, "counter").unwrap();
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!bridge
        .llvm_ir
        .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %words = alloca [66 x i64], align 8\n  %input = getelementptr i64, ptr %words, i64 1\n");
    let mut expected = Vec::new();
    let registry = yir_verify::default_registry();
    // Includes i64::MAX in the first transport word and both bits of the next.
    for case in 0..8 {
        let input = (0..64)
            .map(|i| match case {
                0 => 0,
                1 => 1,
                2 => i % 2,
                3 => (i + 1) % 2,
                4 => usize::from(i == 61),
                5 => usize::from(i == 62),
                6 => usize::from(i == 63),
                _ => usize::from(i == 0 || i == 63),
            } as u64)
            .collect::<Vec<_>>();
        let output = input
            .iter()
            .map(|bit| if input[0] == 0 { 1 - bit } else { *bit })
            .collect::<Vec<_>>();
        let (mut reference, _) = ApplicationSession::open_registered(
            &compiled.yir,
            &registry,
            "counter",
            input.iter().map(|bit| Value::Bool(*bit != 0)).collect(),
        )
        .unwrap();
        assert_eq!(state_words(reference.state()), output);
        reference.event(vec![]).unwrap();
        reference.close(vec![]).unwrap();
        assert_eq!(state_words(reference.state()), output);
        reference.completion_status().unwrap();
        for (slot, value) in std::iter::once(&91).chain(&input).chain([&92]).enumerate() {
            llvm.push_str(&format!("  %p{case}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  store volatile i64 {value}, ptr %p{case}_{slot}\n"));
        }
        for (role, export) in bridge.callbacks.iter().enumerate() {
            llvm.push_str(&format!("  %status{case}_{role} = call i32 @{}(ptr %input, i64 64, ptr %input, i64 64)\n  %wide{case}_{role} = zext i32 %status{case}_{role} to i64\n  call void @nuis_debug_print_i64(i64 %wide{case}_{role})\n", export.symbol));
            expected.push(0);
            for (slot, value) in std::iter::once(&91).chain(&output).chain([&92]).enumerate() {
                llvm.push_str(&format!("  %v{case}_{role}_{slot} = load volatile i64, ptr %p{case}_{slot}\n  call void @nuis_debug_print_i64(i64 %v{case}_{role}_{slot})\n"));
                expected.push(*value as i64);
            }
            for counter in ["probe_allocs", "probe_drops"] {
                llvm.push_str(&format!("  %{counter}{case}_{role} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter}{case}_{role})\n"));
                expected.push(0);
            }
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
        dynamic_loop_guard::run_bounded(std::path::Path::new(&artifact.binary_path), &project.0);
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
    assert_eq!(actual, expected);
}
