use super::*;

#[path = "record_input_fixture.rs"]
pub(super) mod fixture;

#[test]
fn typed_record_inputs_replace_65_leaf_captures_without_widening_callback_abi() {
    for nested in [false, true] {
        let cases = [-3_i64, 0, 5]
            .into_iter()
            .map(|seed| {
                let initial = (0..64).map(|i| (seed + i) as u64).collect::<Vec<_>>();
                let mut event = initial.clone();
                if seed > 0 {
                    event[0] += 1;
                }
                (vec![seed as u64], vec![initial, event.clone(), event])
            })
            .collect::<Vec<_>>();
        check(&fixture::source(nested), cases);
    }
}

#[test]
fn typed_record_inputs_preserve_mixed_nested_snapshot_bits() {
    let mut cases = Vec::new();
    for flag in [0, 1] {
        for (gain, scale) in [
            (0x8000_0000, 0x8000_0000_0000_0000),
            (0x7fc0_1234, 0x7ff8_0000_0000_4321),
        ] {
            let input = (0..64)
                .map(|i| match i % 5 {
                    0 => flag,
                    1 => (-17 - i as i64) as u64,
                    2 => 100 + i,
                    3 => gain,
                    _ => scale,
                })
                .collect::<Vec<_>>();
            let event = input
                .iter()
                .enumerate()
                .map(|(i, word)| if i % 5 == 2 { word + 2 + flag } else { *word })
                .collect::<Vec<_>>();
            cases.push((input.clone(), vec![input, event.clone(), event]));
        }
    }
    check(&fixture::mixed_source(), cases);
}

pub(super) fn check(source: &str, cases: Vec<(Vec<u64>, Vec<Vec<u64>>)>) {
    check_transport(source, cases, true);
}

pub(super) fn check_flattened(source: &str, cases: Vec<(Vec<u64>, Vec<Vec<u64>>)>) {
    check_transport(source, cases, false);
}

fn check_transport(source: &str, cases: Vec<(Vec<u64>, Vec<Vec<u64>>)>, whole: bool) {
    let width = cases[0].1[0].len();
    let project = Project::with_source(source);
    let mut compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    compiled.yir.nodes.reverse();
    compiled.yir.functions.reverse();
    for function in &mut compiled.yir.functions {
        function.body_nodes.reverse();
    }
    let record_nodes = compiled
        .yir
        .nodes
        .iter()
        .filter(|node| node.op.instruction == "param_value_struct")
        .collect::<Vec<_>>();
    assert_eq!(
        !record_nodes.is_empty(),
        whole,
        "fixture must exercise the expected capture transport"
    );
    for node in &record_nodes {
        let function = compiled
            .yir
            .functions
            .iter()
            .find(|function| function.parameters.iter().any(|p| p.node == node.name))
            .unwrap();
        assert!(function.name.starts_with("__nuis_"));
        assert!(function.parameters.len() < 64);
    }
    let relay = compiled
        .yir
        .functions
        .iter()
        .find(|function| function.name == "relay")
        .unwrap();
    assert_eq!(
        relay.parameters.len(),
        width,
        "source helper ABI remains flattened"
    );
    let bridge = emit_registered(&compiled.yir, "counter").unwrap_or_else(|error| {
        panic!(
            "{error}; functions={:?}",
            compiled
                .yir
                .functions
                .iter()
                .map(|f| (&f.name, f.parameters.len(), f.body_nodes.len()))
                .collect::<Vec<_>>()
        )
    });
    assert_eq!(
        bridge
            .llvm_ir
            .lines()
            .any(|line| line.starts_with("define ")
                && line.contains(" @nuis_fn___nuis_")
                && line
                    .split_once('(')
                    .unwrap()
                    .1
                    .contains(&format!("[{width} x i64] %arg"))),
        whole
    );
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
    for (case, (input, outputs)) in cases.into_iter().enumerate() {
        let args = bridge.callbacks[0]
            .arguments
            .iter()
            .zip(&input)
            .map(|(kind, word)| kind.unpack(*word).unwrap())
            .collect();
        let (mut reference, _) =
            ApplicationSession::open_registered(&compiled.yir, &registry, "counter", args).unwrap();
        assert_eq!(state_words(reference.state()), outputs[0]);
        reference.event(vec![]).unwrap();
        assert_eq!(state_words(reference.state()), outputs[1]);
        reference.close(vec![]).unwrap().unwrap();
        assert_eq!(state_words(reference.state()), outputs[2]);
        reference.completion_status().unwrap();
        for slot in 0..66 {
            let word = if slot == 0 {
                91
            } else if slot == width + 1 {
                92
            } else {
                input.get(slot - 1).copied().unwrap_or(93)
            };
            llvm.push_str(&format!("  %p{case}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  store volatile i64 {}, ptr %p{case}_{slot}, align 8\n", word as i64));
        }
        for (role, (export, output)) in bridge.callbacks.iter().zip(outputs).enumerate() {
            let inputs = export.arguments.len();
            llvm.push_str(&format!("  %s{case}_{role} = call i32 @{}(ptr %input, i64 {inputs}, ptr %input, i64 {width})\n  %w{case}_{role} = zext i32 %s{case}_{role} to i64\n  call void @nuis_debug_print_i64(i64 %w{case}_{role})\n", export.symbol));
            expected.push(0);
            for (slot, word) in std::iter::once(&91).chain(&output).chain([&92]).enumerate() {
                llvm.push_str(&format!("  %r{case}_{role}_{slot} = load volatile i64, ptr %p{case}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %r{case}_{role}_{slot})\n"));
                expected.push(*word as i64);
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
    assert_eq!(actual, expected);
}
