use super::*;

mod callback;

#[test]
fn value_return_requires_unique_bounded_flat_i64_fields() {
    for count in [0, 1, 2, 7, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}:i64"))
            .collect::<Vec<_>>()
            .join(";");
        let layout = yir_core::parse_owned_struct_layout(&format!("Packet{{{fields}}}")).unwrap();
        assert_eq!(
            NativeValueReturn::flat_i64(&layout).is_ok(),
            (1..=64).contains(&count)
        );
    }
    for fields in [
        "x:i64;x:i64",
        "x:bool",
        "x:i32",
        "x:f32",
        "x:f64",
        "x:Bytes",
        "x:Nested{n:i64}",
    ] {
        let layout = yir_core::parse_owned_struct_layout(&format!("Packet{{{fields}}}")).unwrap();
        assert!(NativeValueReturn::flat_i64(&layout).is_err(), "{fields}");
    }
}

fn returned() -> StructLlvmValueRef {
    StructLlvmValueRef {
        type_name: "Packet".to_owned(),
        fields: vec![
            ("first".to_owned(), LlvmValueRef::I64("11".to_owned())),
            ("second".to_owned(), LlvmValueRef::I64("22".to_owned())),
        ],
    }
}

fn lower(value: StructLlvmValueRef, layout: &str, guarded: bool) -> Result<String, String> {
    let transport = NativeValueReturn::flat_i64(&yir_core::parse_owned_struct_layout(
        "Packet{second:i64;first:i64}",
    )?)
    .unwrap();
    let args = if guarded {
        vec!["condition", "value", layout]
    } else {
        vec!["value", layout]
    };
    let node = Node {
        name: "returned".to_owned(),
        resource: "cpu".to_owned(),
        op: yir_core::Operation::parse(
            if guarded {
                "cpu.guard_return"
            } else {
                "cpu.return_owned_struct"
            },
            args.into_iter().map(str::to_owned).collect(),
        )
        .unwrap(),
    };
    let registers = BTreeMap::from([
        (
            "condition".to_owned(),
            LlvmValueRef::I64("%flag".to_owned()),
        ),
        ("value".to_owned(), LlvmValueRef::Struct(value)),
    ]);
    let mut body = Vec::new();
    assert_eq!(
        transport.lower(&node, &registers, &mut body, &mut 0, &mut 0)?,
        Some(!guarded)
    );
    Ok(body.join("\n"))
}

#[test]
fn value_returns_keep_declared_field_order_and_guarded_packing() {
    for guarded in [false, true] {
        let body = lower(returned(), "Packet{second:i64;first:i64}", guarded).unwrap();
        assert!(body.contains("poison, i64 22, 0"));
        assert!(body.contains(", i64 11, 1"));
        assert!(body.contains("ret [2 x i64]"));
        assert!(!body.contains("alloca") && !body.contains("@nuis_scheduler_"));
        if guarded {
            assert!(
                body.find("guard_return_struct_then.0:").unwrap()
                    < body.find("insertvalue").unwrap()
            );
            assert!(
                body.find("ret [2 x i64]").unwrap()
                    < body.find("guard_return_struct_cont.1:").unwrap()
            );
        }
    }
}

#[test]
fn value_returns_reject_nominal_kind_field_and_local_layout_drift() {
    let mut wrong_name = returned();
    wrong_name.type_name = "Other".to_owned();
    let mut wrong_kind = returned();
    wrong_kind.fields[0].1 = LlvmValueRef::I32("11".to_owned());
    let mut missing = returned();
    missing.fields.pop();
    let mut duplicate = returned();
    duplicate.fields[0].0 = "second".to_owned();
    for guarded in [false, true] {
        for value in [&wrong_name, &wrong_kind, &missing, &duplicate] {
            assert!(lower(value.clone(), "Packet{second:i64;first:i64}", guarded).is_err());
        }
        for layout in [
            "Other{second:i64;first:i64}",
            "Packet{first:i64;second:i64}",
            "Packet{second:i64;first:bool}",
        ] {
            assert!(lower(returned(), layout, guarded).is_err());
        }
    }
}

#[test]
fn unpack_materializes_independent_named_scalar_values() {
    let transport = NativeValueReturn::flat_i64(
        &yir_core::parse_owned_struct_layout("Packet{second:i64;first:i64}").unwrap(),
    )
    .unwrap();
    let mut body = Vec::new();
    let mut register = 0;
    let first = transport.unpack("%a", &mut body, &mut register);
    let second = transport.unpack("%b", &mut body, &mut register);
    assert_eq!(first.type_name, "Packet");
    assert_eq!(
        first
            .fields
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["second", "first"]
    );
    assert!(
        matches!((&first.fields[0].1, &second.fields[0].1), (LlvmValueRef::I64(a), LlvmValueRef::I64(b)) if a != b)
    );
    assert_eq!(body.len(), 4);
    assert!(body
        .iter()
        .all(|line| line.contains("extractvalue [2 x i64]")));
}
