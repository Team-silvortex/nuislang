use super::*;

pub(super) fn source(slots: usize) -> String {
    let kinds = ["bool", "i32", "i64", "f32", "f64"];
    let fields = (0..slots)
        .map(|i| format!("f{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    let params = (0..slots)
        .map(|i| format!("p{i}: {}", kinds[i % 5]))
        .collect::<Vec<_>>()
        .join(", ");
    let arguments = |second: bool| {
        (0..slots)
            .map(|i| match (second, i % 5) {
                (true, 0) => format!("!p{i}"),
                (true, 2) => format!("p{i} + 10"),
                _ => format!("p{i}"),
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let values = |advanced: bool| {
        (0..slots)
            .rev()
            .map(|i| {
                if advanced && i % 5 == 2 {
                    format!("f{i}: p{i} + 1")
                } else {
                    format!("f{i}: p{i}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    let selected = (0..slots)
        .rev()
        .map(|i| {
            let snapshot = if i % 2 == 0 { "first" } else { "second" };
            format!("f{i}: {snapshot}.payload.f{i}")
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "mod cpu Main {{
        struct Payload {{ {fields} }}
        struct Packet {{ payload: Payload }}
        struct State {{ payload: Payload }}
        @noinline fn packet({params}) -> Packet {{
            if p0 {{ return Packet {{ payload: Payload {{ {} }} }}; }}
            return Packet {{ payload: Payload {{ {} }} }};
        }}
        @noinline fn relay(packet: Packet) -> Packet {{ return packet; }}
        fn start({params}) -> State {{
            let first = relay(packet({}));
            let second = relay(packet({}));
            return stop(State {{ payload: Payload {{ {selected} }} }});
        }}
        fn step(state: State) -> State {{
            let next = relay(Packet {{ payload: state.payload }});
            return State {{ payload: next.payload }};
        }}
        @noinline fn stop(state: State) -> State {{ return state; }}
        fn main() -> i64 {{ return 0; }}
    }}",
        values(false),
        values(true),
        arguments(false),
        arguments(true)
    )
}

#[test]
fn typed_nested_helpers_execute_independent_snapshots_without_owned_allocations() {
    for slots in [1, 6, 64] {
        check_values(slots, &source(slots), 6);
    }
}

pub(super) fn check_values(slots: usize, source: &str, entries: u64) {
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let bridge = yir_lower_llvm::native_session::emit_registered_with_work_limits(
        &compiled.yir,
        "counter",
        0,
        entries,
    )
    .unwrap();
    for helper in ["packet", "relay", "stop"] {
        assert!(bridge
            .llvm_ir
            .contains(&format!("define [{slots} x i64] @nuis_fn_{helper}(")));
        assert!(bridge
            .llvm_ir
            .contains(&format!("call [{slots} x i64] @nuis_fn_{helper}(")));
    }
    assert!(!bridge
        .llvm_ir
        .contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!bridge
        .llvm_ir
        .contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    let ordinary = yir_lower_llvm::emit_module(&compiled.yir).unwrap();
    assert!(ordinary.contains("define i64 @nuis_fn_packet("));
    assert!(ordinary.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));

    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str(&format!(
        "\ndefine i64 @nuis_yir_entry() {{\n  %words = alloca [{} x i64], align 8\n",
        slots + 2
    ));
    let mut expected = Vec::new();
    let mut next = 0;
    let registry = yir_verify::default_registry();
    for flag in [0, 1] {
        for (gain, scale) in [
            (1.5_f32.to_bits(), (-2.25_f64).to_bits()),
            (0x8000_0000, 0x8000_0000_0000_0000),
            (0x7fc0_1234, 0x7ff8_0000_0000_4321),
        ] {
            let input = (0..slots)
                .map(|i| match i % 5 {
                    0 => flag,
                    1 => (-17 - i as i64) as u64,
                    2 => 100 + i as u64,
                    3 => u64::from(gain),
                    _ => scale,
                })
                .collect::<Vec<_>>();
            let output = input
                .iter()
                .enumerate()
                .map(|(i, word)| {
                    let second = i % 2 != 0;
                    match i % 5 {
                        0 if second => 1 - word,
                        2 => word + if second { 10 + flag } else { 1 - flag },
                        _ => *word,
                    }
                })
                .collect::<Vec<_>>();
            let args = bridge.callbacks[0]
                .arguments
                .iter()
                .zip(&input)
                .map(|(kind, word)| kind.unpack(*word).unwrap())
                .collect();
            let (mut reference, _) =
                ApplicationSession::open_registered(&compiled.yir, &registry, "counter", args)
                    .unwrap();
            assert_eq!(state_words(reference.state()), output);
            reference.event(vec![]).unwrap();
            assert_eq!(state_words(reference.state()), output);
            reference.close(vec![]).unwrap().unwrap();
            assert_eq!(state_words(reference.state()), output);
            reference.completion_status().unwrap();

            for (slot, word) in std::iter::once(&91).chain(&input).chain([&92]).enumerate() {
                llvm.push_str(&format!("  %p{next}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  store volatile i64 {}, ptr %p{next}_{slot}, align 8\n", *word as i64));
            }
            llvm.push_str(&format!(
                "  %in{next} = getelementptr i64, ptr %words, i64 1\n"
            ));
            for (role, export) in bridge.callbacks.iter().enumerate() {
                // In-place input/output also checks publication after every helper returned.
                llvm.push_str(&format!("  %s{next}_{role} = call i32 @{}(ptr %in{next}, i64 {slots}, ptr %in{next}, i64 {slots})\n  %w{next}_{role} = zext i32 %s{next}_{role} to i64\n  call void @nuis_debug_print_i64(i64 %w{next}_{role})\n", export.symbol));
                expected.push(0);
                for (slot, word) in std::iter::once(&91).chain(&output).chain([&92]).enumerate() {
                    llvm.push_str(&format!("  %r{next}_{role}_{slot} = load volatile i64, ptr %p{next}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %r{next}_{role}_{slot})\n"));
                    expected.push(*word as i64);
                }
                for counter in ["probe_allocs", "probe_drops"] {
                    llvm.push_str(&format!("  %{counter}{next}_{role} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter}{next}_{role})\n"));
                    expected.push(0);
                }
            }
            next += 1;
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
    assert_eq!(actual, expected, "slots={slots}");
}
