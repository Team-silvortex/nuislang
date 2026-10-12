use super::*;
use yir_lower_llvm::native_session::{emit_registered_with_literal_prints, LiteralPrintPolicy};

#[path = "return_float_pair_oracle.rs"]
mod oracle;

const SOURCE: &str = include_str!("return_float_pair_mixed_logical_effectful_scalar_selections.ns");

fn bridge(module: &YirModule) -> NativeSessionBridge {
    let sites = module
        .nodes
        .iter()
        .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
        .map(|n| n.name.clone());
    let policy = LiteralPrintPolicy::new(sites).unwrap();
    emit_registered_with_literal_prints(module, "counter", &policy, 0, 100).unwrap()
}

fn words(ty: &str) -> Vec<u64> {
    if ty == "f32" {
        vec![
            0,
            0x8000_0000,
            1,
            0x8000_0001,
            0x007f_ffff,
            0x807f_ffff,
            0x3fc0_0000,
            0xc020_0000,
            0x7f80_0000,
            0xff80_0000,
            0x7fc0_1234,
            0xff80_0001,
        ]
    } else {
        vec![
            0,
            0x8000_0000_0000_0000,
            1,
            0x8000_0000_0000_0001,
            0x000f_ffff_ffff_ffff,
            0x800f_ffff_ffff_ffff,
            0x3ff8_0000_0000_0000,
            0xc004_0000_0000_0000,
            0x7ff0_0000_0000_0000,
            0xfff0_0000_0000_0000,
            0x7ff8_1234_5678_abcd,
            0xfff0_0000_0000_0001,
        ]
    }
}

fn driver(bridge: &NativeSessionBridge, ty: &str) -> (String, Vec<i64>) {
    let mut llvm = bridge.llvm_ir.replacen(
        "define i64 @nuis_yir_entry()",
        "define i64 @unused_native_entry()",
        1,
    );
    assert!(!llvm.contains("call ptr @nuis_scheduler_owned_aggregate_alloc_v1("));
    assert!(!llvm.contains("call void @nuis_scheduler_owned_aggregate_drop_v1("));
    // Observe the actual carried argument inside the compiled source callee.
    // No computation, join, helper result or source return is replaced.
    let start = aggregate_values::definition(&llvm, "after_work");
    let insertion = start + llvm[start..].find("{\n").unwrap() + 2;
    let probe = if ty == "f32" {
        "  %probe_bits = bitcast float %arg0 to i32\n  %probe_word = zext i32 %probe_bits to i64\n"
    } else {
        "  %probe_word = bitcast double %arg0 to i64\n"
    };
    llvm.insert_str(
        insertion,
        &format!("{probe}  call void @nuis_debug_print_i64(i64 %probe_word)\n"),
    );
    llvm.push_str(multi_execution::ALLOCATION_PROBE);
    llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %inputs = alloca [24 x i64], align 8\n  %output = alloca [15 x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n");
    let mut expected = Vec::new();
    let mut case = 0;
    for word in words(ty) {
        for first in [false, true] {
            for second in [false, true] {
                for role in 0..3 {
                    let (a, b) = if role == 0 {
                        (first, second)
                    } else {
                        (!first, !second)
                    };
                    let outcome = oracle::oracle(ty, word, a, b);
                    let state = oracle::state(ty, word, a, b);
                    let input = match role {
                        0 => std::iter::once(word)
                            .chain(oracle::arguments(ty, word, a, b))
                            .collect::<Vec<_>>(),
                        1 => vec![0; 13]
                            .into_iter()
                            .chain(oracle::arguments(ty, word, a, b))
                            .collect(),
                        _ => vec![0; 13],
                    };
                    for (slot, value) in input.iter().enumerate() {
                        llvm.push_str(&format!(
                            "  %i{case}_{slot} = getelementptr i64, ptr %inputs, i64 {slot}\n"
                        ));
                        if role != 0 && slot < 13 {
                            llvm.push_str(&format!("  %prev{case}_{slot} = getelementptr i64, ptr %out, i64 {slot}\n  %saved{case}_{slot} = load volatile i64, ptr %prev{case}_{slot}\n  store volatile i64 %saved{case}_{slot}, ptr %i{case}_{slot}\n"));
                        } else {
                            llvm.push_str(&format!(
                                "  store volatile i64 {}, ptr %i{case}_{slot}\n",
                                *value as i64
                            ));
                        }
                    }
                    for (slot, word) in [(0, 91), (14, 92)] {
                        llvm.push_str(&format!("  %guard{case}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  store volatile i64 {word}, ptr %guard{case}_{slot}\n"));
                    }
                    llvm.push_str(&format!(
                        "  %s{case} = call i32 @{}(ptr %inputs, i64 {}, ptr %out, i64 13)\n",
                        bridge.callbacks[role].symbol,
                        input.len()
                    ));
                    if role != 2 {
                        for stage in outcome.trace {
                            if stage == 96 {
                                expected.push(outcome.child.unwrap() as i64);
                            }
                            expected.push(i64::from(stage));
                        }
                    }
                    llvm.push_str(&format!("  %status{case} = zext i32 %s{case} to i64\n  call void @nuis_debug_print_i64(i64 %status{case})\n"));
                    expected.push(0);
                    for (slot, word) in std::iter::once(91).chain(state).chain([92]).enumerate() {
                        llvm.push_str(&format!("  %p{case}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  %w{case}_{slot} = load volatile i64, ptr %p{case}_{slot}\n  call void @nuis_debug_print_i64(i64 %w{case}_{slot})\n"));
                        expected.push(word as i64);
                    }
                    for counter in ["probe_allocs", "probe_drops"] {
                        llvm.push_str(&format!("  %{counter}{case} = load i64, ptr @{counter}\n  call void @nuis_debug_print_i64(i64 %{counter}{case})\n"));
                        expected.push(0);
                    }
                    case += 1;
                }
            }
        }
    }
    llvm.push_str("  ret i64 0\n}\n");
    assert_eq!(case, 144);
    (llvm, expected)
}

fn check(ty: &str) {
    let source = SOURCE.replace("f64", ty);
    let project = Project::with_source(&source);
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for reversed in [false, true] {
        let module = typed_effect_returns::reordered(&compiled.yir, reversed);
        let native = bridge(&module);
        assert!(module
            .nodes
            .iter()
            .any(|n| n.op.instruction == "call_owned_struct"
                && n.op
                    .args
                    .get(1)
                    .is_some_and(|v| v.ends_with(&format!("{{owner:bool;child:{ty}}}")))));
        let (llvm, expected) = driver(&native, ty);
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
            .map(|v| v.parse::<i64>().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{ty} reversed={reversed}");
    }
}

#[test]
fn return_float_pair_mixed_logical_effectful_scalar_selections_f32_native_exact_bits_and_order() {
    check("f32");
}
#[test]
fn return_float_pair_mixed_logical_effectful_scalar_selections_f64_native_exact_bits_and_order() {
    check("f64");
}

fn reject_casts(ty: &str) {
    let project = Project::with_source(&SOURCE.replace("f64", ty));
    let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
    for (from, to) in [
        (format!("pack_{ty}_word"), format!("cast_{ty}_to_i64")),
        (format!("unpack_{ty}_word"), format!("cast_i64_to_{ty}")),
    ] {
        let mut module = compiled.yir.clone();
        let mut changed = 0;
        for n in &mut module.nodes {
            if n.op.instruction == from {
                n.op.instruction = to.clone();
                changed += 1;
            }
        }
        assert!(changed > 0);
        let policy = LiteralPrintPolicy::new(
            module
                .nodes
                .iter()
                .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
                .map(|n| n.name.clone()),
        )
        .unwrap();
        assert!(emit_registered_with_literal_prints(&module, "counter", &policy, 0, 100).is_err());
    }
}
#[test]
fn return_float_pair_mixed_logical_effectful_scalar_selections_f32_reject_numeric_word_casts() {
    reject_casts("f32");
}
#[test]
fn return_float_pair_mixed_logical_effectful_scalar_selections_f64_reject_numeric_word_casts() {
    reject_casts("f64");
}
