use super::*;
use yir_lower_llvm::native_session::{emit_registered_with_i64_prints, I64PrintPolicy};

#[test]
fn state_control_exit_preflight_rejects_shapes_and_noncanonical_words_without_writing_state() {
    for kind in cases::KINDS {
        let source = cases::SOURCE.replace("f64", kind.ty());
        let project = Project::with_source(&source);
        let compiled = nuisc::pipeline::compile_project(&project.0).unwrap();
        for reversed in [false, true] {
            let module = typed_effect_returns::reordered(&compiled.yir, reversed);
            let sites = module
                .nodes
                .iter()
                .filter(|n| matches!(n.op.instruction.as_str(), "print" | "guard_print"))
                .map(|n| n.name.clone())
                .collect::<Vec<_>>();
            let bridge = emit_registered_with_i64_prints(
                &module,
                "counter",
                &I64PrintPolicy::new(sites).unwrap(),
                cases::LOOP_LIMIT,
                cases::ENTRY_LIMIT,
            )
            .unwrap();
            assert_eq!(
                bridge.state_fields,
                vec![
                    (
                        "total".to_owned(),
                        if kind == cases::Kind::F32 {
                            ScalarKind::F32
                        } else {
                            ScalarKind::F64
                        }
                    ),
                    ("control".to_owned(), ScalarKind::I64),
                ]
            );
            assert_eq!(
                bridge
                    .callbacks
                    .iter()
                    .map(|c| c.arguments.len())
                    .collect::<Vec<_>>(),
                [37, 39, 2]
            );
            let mut llvm = bridge.llvm_ir.replacen(
                "define i64 @nuis_yir_entry()",
                "define i64 @unused_native_entry()",
                1,
            );
            for callback in &bridge.callbacks {
                let start = llvm
                    .find(&format!("define i32 @{}(", callback.symbol))
                    .unwrap();
                let body = &llvm[start..start + llvm[start..].find("\n}\n").unwrap()];
                let call = body.find("%returned = call [2 x i64]").unwrap();
                let last_extract = body.rfind(" = extractvalue [2 x i64]").unwrap();
                assert!(body.rfind(" = load i64, ptr %input_ptr").unwrap() < call);
                assert!(call < last_extract);
                assert!(last_extract < body.find("store i64 %result0").unwrap());
            }
            llvm.push_str("\ndefine i64 @nuis_yir_entry() {\n  %input = alloca [39 x i64], align 8\n  %output = alloca [4 x i64], align 8\n  %out = getelementptr i64, ptr %output, i64 1\n");
            let mut expected = Vec::new();
            let mut id = 0;
            for (role, callback) in bridge.callbacks.iter().enumerate() {
                let mut words = if role == 0 {
                    cases::seed(kind).words()
                } else {
                    let mut words = vec![kind.nan(), i64::MAX];
                    if role == 1 {
                        words.extend(cases::seed(kind).words());
                    }
                    words
                };
                let original = words.clone();
                for rejection in 0..7 {
                    if role == 2 && matches!(rejection, 4 | 6) {
                        continue;
                    }
                    if rejection == 5 && kind != cases::Kind::F32 {
                        continue;
                    }
                    words.clone_from(&original);
                    let shift = if role == 1 { 2 } else { 0 };
                    let mut argc = words.len() as i64;
                    let mut outc = 2;
                    let mut input = "%input";
                    let mut output = "%out";
                    let mut status = 1;
                    match rejection {
                        0 => argc -= 1,
                        1 => outc = 1,
                        2 => input = "null",
                        3 => output = "null",
                        4 => {
                            words[shift + 1] = 2;
                            status = 2;
                        }
                        5 => {
                            words[0] = 1_i64 << 32;
                            status = 2;
                        }
                        6 => {
                            words[shift + 3] = u32::MAX as i64;
                            status = 2;
                        }
                        _ => unreachable!(),
                    }
                    for (slot, word) in words.iter().enumerate() {
                        llvm.push_str(&format!("  %a{id}_{slot} = getelementptr i64, ptr %input, i64 {slot}\n  store i64 {word}, ptr %a{id}_{slot}\n"));
                    }
                    for (slot, word) in [91, 11, 22, 92].into_iter().enumerate() {
                        llvm.push_str(&format!("  %s{id}_{slot} = getelementptr i64, ptr %output, i64 {slot}\n  store i64 {word}, ptr %s{id}_{slot}\n"));
                    }
                    llvm.push_str(&format!("  %r{id} = call i32 @{}(ptr {input}, i64 {argc}, ptr {output}, i64 {outc})\n  %w{id} = zext i32 %r{id} to i64\n  call void @nuis_debug_print_i64(i64 %w{id})\n",callback.symbol));
                    expected.push(status);
                    for slot in 0..4 {
                        llvm.push_str(&format!("  %v{id}_{slot} = load i64, ptr %s{id}_{slot}\n  call void @nuis_debug_print_i64(i64 %v{id}_{slot})\n"));
                    }
                    expected.extend([91, 11, 22, 92]);
                    id += 1;
                }
            }
            llvm.push_str("  ret i64 0\n}\n");
            let artifact = nuisc::aot::write_and_link_with_source(
                &project.0.join("main.ns"),
                &project.0.join(format!("preflight-{reversed}")),
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
            assert_eq!(actual, expected, "reversed={reversed} {kind:?}");
        }
    }
}
