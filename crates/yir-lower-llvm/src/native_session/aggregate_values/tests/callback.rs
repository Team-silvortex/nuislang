use super::*;

const LAYOUT: &str = "State{child:Values{flag:bool;tag:i32;count:i64;gain:f32;scale:f64}}";

#[test]
fn callback_value_plan_uses_shared_scalar_bounds_without_widening_helpers() {
    let layout = yir_core::parse_owned_struct_layout(LAYOUT).unwrap();
    assert!(NativeValueReturn::flat_i64(&layout).is_err());
    assert_eq!(
        NativeValueReturn::callback(LAYOUT).unwrap().llvm_type(),
        "[5 x i64]"
    );
    for count in [0, 1, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}:f32"))
            .collect::<Vec<_>>()
            .join(";");
        assert_eq!(
            NativeValueReturn::callback(&format!("S{{nested:N{{{fields}}}}}")).is_ok(),
            (1..=64).contains(&count)
        );
    }
    for invalid in ["S{x:Bytes}", "S{x:String}", "S{x:bool;x:bool}", "S{x:N{}}"] {
        assert!(NativeValueReturn::callback(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn callback_value_plan_preserves_typed_nested_snapshots_and_guarded_bits() {
    let plan = NativeValueReturn::callback(LAYOUT).unwrap();
    let mut child = StructLlvmValueRef {
        type_name: "Values".to_owned(),
        fields: vec![
            (
                "flag".to_owned(),
                LlvmValueRef::Bool {
                    i1: "%flag".to_owned(),
                    i64: "%wide".to_owned(),
                },
            ),
            ("tag".to_owned(), LlvmValueRef::I32("%tag".to_owned())),
            ("count".to_owned(), LlvmValueRef::I64("%count".to_owned())),
            ("gain".to_owned(), LlvmValueRef::F32("%gain".to_owned())),
            ("scale".to_owned(), LlvmValueRef::F64("%scale".to_owned())),
        ],
    };
    child.fields.reverse();
    let value = LlvmValueRef::Struct(StructLlvmValueRef {
        type_name: "State".to_owned(),
        fields: vec![("child".to_owned(), LlvmValueRef::Struct(child))],
    });
    for guarded in [false, true] {
        let mut args = vec!["value".to_owned(), LAYOUT.to_owned()];
        if guarded {
            args.insert(0, "condition".to_owned());
        }
        let node = Node {
            name: "result".to_owned(),
            resource: "cpu".to_owned(),
            op: yir_core::Operation::parse(
                if guarded {
                    "cpu.guard_return"
                } else {
                    "cpu.return_owned_struct"
                },
                args,
            )
            .unwrap(),
        };
        let registers = BTreeMap::from([
            ("value".to_owned(), value.clone()),
            (
                "condition".to_owned(),
                LlvmValueRef::I64("%condition".to_owned()),
            ),
        ]);
        let mut body = Vec::new();
        let mut reg = 0;
        assert_eq!(
            plan.lower(&node, &registers, &mut body, &mut reg, &mut 0)
                .unwrap(),
            Some(!guarded)
        );
        let first = plan.unpack("%first", &mut body, &mut reg);
        let second = plan.unpack("%second", &mut body, &mut reg);
        for result in [&first, &second] {
            let LlvmValueRef::Struct(child) = &result.fields[0].1 else {
                panic!("nested");
            };
            assert_eq!(child.type_name, "Values");
            assert!(matches!(child.fields[0].1, LlvmValueRef::Bool { .. }));
            assert!(matches!(child.fields[1].1, LlvmValueRef::I32(_)));
            assert!(matches!(child.fields[2].1, LlvmValueRef::I64(_)));
            assert!(matches!(child.fields[3].1, LlvmValueRef::F32(_)));
            assert!(matches!(child.fields[4].1, LlvmValueRef::F64(_)));
        }
        let body = body.join("\n");
        for operation in [
            "zext i1 %flag",
            "sext i32 %tag",
            "bitcast float %gain",
            "bitcast double %scale",
            ", i64 %count, 2",
            "ret [5 x i64]",
            "extractvalue [5 x i64] %first, 4",
            "extractvalue [5 x i64] %second, 4",
        ] {
            assert!(body.contains(operation), "{operation}");
        }
        assert!(!body.contains("call ") && !body.contains("alloca "));
        if guarded {
            assert!(
                body.find("guard_return_struct_then.0:").unwrap()
                    < body.find("zext i1 %flag").unwrap()
            );
        }
        let mut drift = registers.clone();
        let LlvmValueRef::Struct(state) = drift.get_mut("value").unwrap() else {
            unreachable!();
        };
        let LlvmValueRef::Struct(child) = &mut state.fields[0].1 else {
            unreachable!();
        };
        child.type_name = "Other".to_owned();
        assert!(plan
            .lower(&node, &drift, &mut Vec::new(), &mut 0, &mut 0)
            .is_err());
    }
}
