use super::*;

fn source(ty: &str, arithmetic: bool) -> String {
    let operand = if arithmetic { "(value * 1.0)" } else { "value" };
    format!(
        "mod cpu Main {{
        struct State {{ original: {ty}, negative: {ty}, double: {ty},
            called: {ty}, computed: {ty} }}
        @noinline fn relay(value: {ty}) -> {ty} {{ return value; }}
        @noinline fn snapshot(value: {ty}) -> State {{
            return State {{ original: value, negative: -value,
                double: -(-value), called: -relay(value), computed: -{operand} }};
        }}
        fn start(value: {ty}) -> State {{ return snapshot(value); }}
        fn step(state: State) -> State {{ return snapshot(state.original); }}
        fn stop(state: State) -> State {{ return snapshot(state.original); }}
        fn main() -> i64 {{ return 0; }}
    }}"
    )
}

fn inputs(ty: &str, arithmetic: bool) -> Vec<u64> {
    let words = if ty == "f32" {
        vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x3fa0_0000,
            0xbfa0_0000,
            0x7f7f_ffff,
            0xff7f_ffff,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_1234,
            0xffc0_5678,
            0x7f80_0001,
            0xff80_0001,
        ]
    } else {
        vec![
            0,
            0x8000_0000_0000_0000,
            1,
            0x8000_0000_0000_0001,
            0x3ff4_0000_0000_0000,
            0xbff4_0000_0000_0000,
            0x7fef_ffff_ffff_ffff,
            0xffef_ffff_ffff_ffff,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_0000_0000_1234,
            0xfff8_0000_0000_5678,
            0x7ff0_0000_0000_0001,
            0xfff0_0000_0000_0001,
        ]
    };
    words
        .into_iter()
        .take(if arithmetic { 8 } else { 14 })
        .collect()
}

fn trace_once(events: &[String]) {
    assert_eq!(
        events.iter().filter(|e| e.contains("] relay(")).count(),
        1,
        "relay: {events:?}"
    );
}

fn check(ty: &str) {
    let sign = if ty == "f32" { 1 << 31 } else { 1 << 63 };
    for arithmetic in [false, true] {
        let source = source(ty, arithmetic);
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        let registry = yir_verify::default_registry();
        for reversed in [false, true] {
            let mut module = compiled.yir.clone();
            if reversed {
                module.nodes.reverse();
                module.edges.reverse();
                module.functions.reverse();
                for function in &mut module.functions {
                    function.body_nodes.reverse();
                }
            }
            let cases = inputs(ty, arithmetic);
            for &word in &cases {
                // No float arithmetic in the sign oracle, including signaling NaNs.
                let expected = vec![word, word ^ sign, word, word ^ sign, word ^ sign];
                let value = if ty == "f32" {
                    Value::F32(f32::from_bits(word as u32))
                } else {
                    Value::F64(f64::from_bits(word))
                };
                let (mut session, trace) =
                    ApplicationSession::open_registered(&module, &registry, "counter", vec![value])
                        .unwrap();
                trace_once(&trace.events);
                assert_eq!(state_words(session.state()), expected, "{ty} {word:x}");
                let trace = session.event(vec![]).unwrap();
                trace_once(&trace.events);
                assert_eq!(state_words(session.state()), expected);
                let trace = session.close(vec![]).unwrap().unwrap();
                trace_once(&trace.events);
                assert_eq!(state_words(session.state()), expected);
                session.completion_status().unwrap();
            }
            let bridge = emit_registered(&module, "counter").unwrap();
            assert_eq!(bridge.state_fields.len(), 5);
            assert!(module.nodes.iter().any(|n| n.op.instruction == "xor"));
            assert!(bridge.llvm_ir.contains(" = xor i64 "));
            let mut llvm = bridge.llvm_ir.replacen(
                "define i64 @nuis_yir_entry()",
                "define i64 @unused_native_entry()",
                1,
            );
            // Count the compiled source helper itself; the driver never supplies its computation.
            let start = aggregate_values::definition(&llvm, "relay");
            let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
            llvm.insert_str(insertion, "  %calls = load volatile i64, ptr @negation_probe_calls\n  %next_calls = add i64 %calls, 1\n  store volatile i64 %next_calls, ptr @negation_probe_calls\n");
            llvm.push_str("\n@negation_probe_calls = internal global i64 0, align 8\n\ndefine i64 @nuis_yir_entry() {\n  %words = alloca [5 x i64], align 8\n");
            let mut expected = Vec::new();
            for (case, word) in cases.into_iter().enumerate() {
                llvm.push_str(&format!("  store i64 {}, ptr %words, align 8\n  store volatile i64 0, ptr @negation_probe_calls\n", word as i64));
                for (role, export) in bridge.callbacks.iter().enumerate() {
                    let count = export.arguments.len();
                    llvm.push_str(&format!("  %s{case}_{role} = call i32 @{}(ptr %words, i64 {count}, ptr %words, i64 5)\n  %w{case}_{role} = zext i32 %s{case}_{role} to i64\n  call void @nuis_debug_print_i64(i64 %w{case}_{role})\n", export.symbol));
                    expected.push(0);
                    for (slot, value) in [word, word ^ sign, word, word ^ sign, word ^ sign]
                        .into_iter()
                        .enumerate()
                    {
                        llvm.push_str(&format!("  %p{case}_{role}_{slot} = getelementptr i64, ptr %words, i64 {slot}\n  %v{case}_{role}_{slot} = load volatile i64, ptr %p{case}_{role}_{slot}, align 8\n  call void @nuis_debug_print_i64(i64 %v{case}_{role}_{slot})\n"));
                        expected.push(value as i64);
                    }
                    llvm.push_str(&format!("  %calls{case}_{role} = load volatile i64, ptr @negation_probe_calls\n  call void @nuis_debug_print_i64(i64 %calls{case}_{role})\n"));
                    expected.push(role as i64 + 1);
                }
            }
            llvm.push_str("  ret i64 0\n}\n");
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("out-{reversed}")),
                &source,
                nuisc::aot::AotCompileProgram {
                    ast: &compiled.ast,
                    nir: &compiled.nir,
                    yir: &module,
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
                .map(|line| line.parse::<i64>().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(
                actual, expected,
                "{ty} arithmetic={arithmetic} reversed={reversed}"
            );
        }
    }
}

#[test]
fn unary_float_negation_f32_preserves_zero_nan_payloads_and_once_only_calls() {
    check("f32");
}

#[test]
fn unary_float_negation_f64_preserves_zero_nan_payloads_and_once_only_calls() {
    check("f64");
}

#[test]
fn unary_float_negation_native_bridge_rejects_wrong_word_and_mask_kinds() {
    for ty in ["f32", "f64"] {
        let project = Project::with_source(&source(ty, false));
        let module = nuisc::pipeline::compile_project(&project.0).unwrap().yir;
        for mutation in ["mask", "pack", "unpack"] {
            let mut changed = module.clone();
            let node = changed
                .nodes
                .iter_mut()
                .find(|n| {
                    n.op.instruction
                        == match mutation {
                            "mask" => "xor",
                            "pack" if ty == "f32" => "pack_f32_word",
                            "pack" => "pack_f64_word",
                            "unpack" if ty == "f32" => "unpack_f32_word",
                            _ => "unpack_f64_word",
                        }
                })
                .unwrap();
            if mutation == "mask" {
                let word = node.op.args[1].clone();
                changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == word)
                    .unwrap()
                    .op
                    .instruction = "const_f64".into();
                changed
                    .nodes
                    .iter_mut()
                    .find(|n| n.name == word)
                    .unwrap()
                    .op
                    .args = vec!["0.0".into()];
            } else {
                node.op.instruction = match (mutation, ty) {
                    ("pack", "f32") => "pack_f64_word",
                    ("pack", _) => "pack_f32_word",
                    ("unpack", "f32") => "unpack_f64_word",
                    _ => "unpack_f32_word",
                }
                .into();
            }
            assert!(
                emit_registered(&changed, "counter").is_err(),
                "{ty} {mutation}"
            );
        }
    }
}
