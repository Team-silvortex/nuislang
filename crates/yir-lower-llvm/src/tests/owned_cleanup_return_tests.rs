use super::support::*;
use crate::{guard_return_lowering::lower_cpu_guard_return_node, LlvmValueRef, StructLlvmValueRef};

fn state() -> LlvmValueRef {
    LlvmValueRef::Struct(StructLlvmValueRef {
        type_name: "State".to_owned(),
        fields: vec![
            ("second".to_owned(), LlvmValueRef::I64("22".to_owned())),
            ("first".to_owned(), LlvmValueRef::I64("11".to_owned())),
        ],
    })
}

fn lower(
    terminal: bool,
    layout: Option<&str>,
    values: &[LlvmValueRef],
) -> (Result<(), String>, String) {
    let instruction = if terminal {
        "cpu.branch_drop_owned_bytes_return"
    } else {
        "cpu.guard_drop_owned_bytes_return"
    };
    let args = if terminal {
        vec!["cond", "bytes", "then", "bytes", "else"]
    } else {
        vec!["cond", "bytes", "then"]
    };
    let node = Node {
        name: "cleanup".to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(instruction, args.into_iter().map(str::to_owned).collect()).unwrap(),
    };
    let mut registers = BTreeMap::from([
        ("cond".to_owned(), LlvmValueRef::I64("%cond".to_owned())),
        (
            "bytes".to_owned(),
            LlvmValueRef::OwnedBytes {
                blob: "%blob".to_owned(),
            },
        ),
    ]);
    registers.extend(
        ["then", "else"]
            .into_iter()
            .zip(values)
            .map(|(name, value)| (name.to_owned(), value.clone())),
    );
    let mut body = Vec::new();
    let layout = layout.map(|layout| yir_core::parse_owned_struct_layout(layout).unwrap());
    let result = lower_cpu_guard_return_node(
        &node,
        &mut body,
        &registers,
        &mut 0,
        &mut 0,
        CpuCallScalarKind::I64,
        layout.as_ref(),
    )
    .map(|_| ());
    (result, body.join("\n"))
}

#[test]
fn cleanup_returns_pack_declared_order_and_drop_only_inside_selected_branches() {
    for terminal in [false, true] {
        let (result, llvm) = lower(
            terminal,
            Some("State{first:i64;second:i64}"),
            &[state(), state()],
        );
        result.unwrap();
        let branches = if terminal { 2 } else { 1 };
        assert_eq!(
            llvm.matches("call void @nuis_scheduler_owned_blob_drop_v1")
                .count(),
            branches
        );
        assert_eq!(
            llvm.matches("call ptr @nuis_scheduler_owned_aggregate_alloc_v1")
                .count(),
            branches
        );
        assert_eq!(llvm.matches("ret i64").count(), branches);
        assert_eq!(llvm.matches("i64 0, i64 11)").count(), branches, "{llvm}");
        assert_eq!(llvm.matches("i64 1, i64 22)").count(), branches, "{llvm}");
        assert!(
            llvm.find("br i1").unwrap()
                < llvm
                    .find("call void @nuis_scheduler_owned_blob_drop_v1")
                    .unwrap()
        );
        if !terminal {
            let continuation = llvm.lines().last().unwrap();
            assert!(
                continuation.starts_with("guard_drop_bytes_return_cont."),
                "{llvm}"
            );
        }
    }
}

#[test]
fn cleanup_return_layout_mismatch_and_missing_inputs_fail_before_effects() {
    let LlvmValueRef::Struct(mut wrong_type) = state() else {
        unreachable!()
    };
    wrong_type.type_name = "Other".to_owned();
    let LlvmValueRef::Struct(mut duplicate) = state() else {
        unreachable!()
    };
    duplicate.fields[0].0 = "first".to_owned();
    let LlvmValueRef::Struct(mut wrong_kind) = state() else {
        unreachable!()
    };
    wrong_kind.fields[0].1 = LlvmValueRef::I32("22".to_owned());
    for invalid in [
        LlvmValueRef::I64("42".to_owned()),
        LlvmValueRef::Struct(wrong_type),
        LlvmValueRef::Struct(duplicate),
        LlvmValueRef::Struct(wrong_kind),
    ] {
        for (terminal, values) in [
            (false, vec![invalid.clone()]),
            (true, vec![state(), invalid]),
        ] {
            let (result, llvm) = lower(terminal, Some("State{first:i64;second:i64}"), &values);
            assert!(result.unwrap_err().contains("function return layout"));
            assert!(!llvm.contains("br i1"), "{llvm}");
            assert!(!llvm.contains("call void"), "{llvm}");
        }
    }
    for (layout, values) in [
        (None, vec![state()]),
        (Some("State{first:i64;second:i64}"), vec![]),
    ] {
        let (result, llvm) = lower(false, layout, &values);
        assert!(result.is_err());
        assert!(!llvm.contains("br i1"));
    }
}

#[test]
fn cleanup_return_rejects_directly_aliased_owned_bytes_inside_nested_results() {
    for alias in [true, false] {
        let result = LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: "Result".to_owned(),
            fields: vec![(
                "nested".to_owned(),
                LlvmValueRef::Struct(StructLlvmValueRef {
                    type_name: "Payload".to_owned(),
                    fields: vec![(
                        "bytes".to_owned(),
                        LlvmValueRef::OwnedBytes {
                            blob: if alias { "%blob" } else { "%other_blob" }.to_owned(),
                        },
                    )],
                }),
            )],
        });
        let (result, llvm) = lower(
            false,
            Some("Result{nested:Payload{bytes:Bytes}}"),
            &[result],
        );
        if alias {
            assert!(result.unwrap_err().contains("Bytes being dropped"));
            assert!(!llvm.contains("call void"));
        } else {
            result.unwrap();
            assert!(llvm.contains("set_blob_v1"));
            assert!(llvm.contains("ptr %other_blob"));
        }
    }
}
