use super::*;

mod native;

fn record(name: &str, fields: Vec<(&str, LlvmValueRef)>) -> LlvmValueRef {
    LlvmValueRef::Struct(StructLlvmValueRef {
        type_name: name.to_owned(),
        fields: fields
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value))
            .collect(),
    })
}

fn value() -> LlvmValueRef {
    record(
        "Packet",
        vec![
            ("last", LlvmValueRef::I64("-17".to_owned())),
            (
                "child",
                record(
                    "Inner",
                    vec![
                        ("scale", LlvmValueRef::F64("%scale".to_owned())),
                        ("gain", LlvmValueRef::F32("%gain".to_owned())),
                        ("tag", LlvmValueRef::I32("-3".to_owned())),
                        (
                            "flag",
                            LlvmValueRef::Bool {
                                i1: "true".to_owned(),
                                i64: "1".to_owned(),
                            },
                        ),
                    ],
                ),
            ),
        ],
    )
}

const LAYOUT: &str = "Packet{child:Inner{flag:bool;tag:i32;gain:f32;scale:f64};last:i64}";

#[test]
fn transport_counts_all_nested_scalar_leaves_and_rejects_resources() {
    for count in [0, 1, 2, 7, 63, 64, 65] {
        let fields = (0..count)
            .map(|i| format!("f{i}:i64"))
            .collect::<Vec<_>>()
            .join(";");
        for layout in [
            format!("Packet{{{fields}}}"),
            format!("Packet{{child:Inner{{{fields}}}}}"),
        ] {
            let parsed = NativeValueLayout::parse(&layout);
            assert_eq!(parsed.is_ok(), (1..=64).contains(&count), "{layout}");
            if let Ok(parsed) = parsed {
                assert_eq!(parsed.llvm_type(), format!("[{count} x i64]"));
            }
        }
    }
    let fields = (0..32)
        .map(|i| format!("f{i}:f32"))
        .collect::<Vec<_>>()
        .join(";");
    assert!(NativeValueLayout::parse(&format!(
        "Packet{{a:Inner{{{fields}}};b:Inner{{{fields}}}}}"
    ))
    .is_ok());
    assert!(NativeValueLayout::parse(&format!(
        "Packet{{a:Inner{{{fields}}};b:Inner{{{fields}}};extra:bool}}"
    ))
    .is_err());
    for invalid in [
        "Packet{x:i64;x:i64}",
        "Packet{child:Inner{x:bool;x:bool}}",
        "Packet{x:Bytes}",
        "Packet{x:String}",
        "Packet{x:buffer_ref}",
        "Packet{x:node_ref}",
        "Packet{x:Inner{data:Bytes}}",
        "Packet{x:Inner{}}",
        "Packet{x:i64",
    ] {
        assert!(NativeValueLayout::parse(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn prepared_values_order_nested_fields_and_own_their_snapshot() {
    let layout = NativeValueLayout::parse(LAYOUT).unwrap();
    let mut original = value();
    let first = layout.prepare(&original).unwrap();
    let LlvmValueRef::Struct(root) = &mut original else {
        unreachable!();
    };
    root.fields[0].1 = LlvmValueRef::I64("123".to_owned());
    let second = layout.prepare(&original).unwrap();
    let mut body = Vec::new();
    let mut next = 0;
    let first_register = first.emit(&mut body, &mut next);
    let split = body.len();
    let second_register = second.emit(&mut body, &mut next);
    assert_ne!(first_register, second_register);
    let first_body = body[..split].join("\n");
    let second_body = body[split..].join("\n");
    for operation in [
        "zext i1 true to i64",
        "sext i32 -3 to i64",
        "bitcast float %gain to i32",
        "bitcast double %scale to i64",
    ] {
        assert!(first_body.contains(operation), "{operation}");
        assert!(second_body.contains(operation), "{operation}");
    }
    assert!(first_body.contains(", i64 -17, 4"));
    assert!(second_body.contains(", i64 123, 4"));
    assert_eq!(first_body.matches("insertvalue [5 x i64]").count(), 5);
    assert_eq!(second_body.matches("insertvalue [5 x i64]").count(), 5);
    assert!(!body.iter().any(|line| line.contains("ret ")));
}

#[test]
fn prepare_rejects_nominal_field_and_scalar_kind_drift_before_emission() {
    let layout = NativeValueLayout::parse(LAYOUT).unwrap();
    let LlvmValueRef::Struct(original) = value() else {
        unreachable!();
    };
    let mut malformed = Vec::new();
    let mut wrong_root = original.clone();
    wrong_root.type_name = "Other".to_owned();
    malformed.push(wrong_root);
    let mut missing = original.clone();
    missing.fields.pop();
    malformed.push(missing);
    let mut duplicate = original.clone();
    duplicate.fields[0].0 = "child".to_owned();
    malformed.push(duplicate);
    let mut wrong_child = original.clone();
    let LlvmValueRef::Struct(child) = &mut wrong_child.fields[1].1 else {
        unreachable!();
    };
    child.type_name = "Other".to_owned();
    malformed.push(wrong_child);
    for replacement in [
        LlvmValueRef::I32("1".to_owned()),
        LlvmValueRef::I64("1".to_owned()),
        LlvmValueRef::F32("1.0".to_owned()),
        LlvmValueRef::Ptr("%pointer".to_owned()),
        LlvmValueRef::OwnedBytes {
            blob: "%bytes".to_owned(),
        },
    ] {
        let mut wrong_kind = original.clone();
        let LlvmValueRef::Struct(child) = &mut wrong_kind.fields[1].1 else {
            unreachable!();
        };
        child.fields[0].1 = replacement;
        malformed.push(wrong_kind);
    }
    for malformed in malformed {
        assert!(layout.prepare(&LlvmValueRef::Struct(malformed)).is_err());
    }
    assert!(layout.prepare(&LlvmValueRef::I64("0".to_owned())).is_err());
}

#[test]
fn input_and_output_decoding_keep_typed_independent_registers() {
    let layout = NativeValueLayout::parse(LAYOUT).unwrap();
    let mut body = Vec::new();
    let mut next = 0;
    let first = layout.unpack("%input", &mut body, &mut next);
    let split = body.len();
    let second = layout.unpack("%output", &mut body, &mut next);
    for value in [&first, &second] {
        assert_eq!(value.type_name, "Packet");
        let LlvmValueRef::Struct(child) = &value.fields[0].1 else {
            panic!("nested typed record");
        };
        assert_eq!(child.type_name, "Inner");
        assert!(matches!(child.fields[0].1, LlvmValueRef::Bool { .. }));
        assert!(matches!(child.fields[1].1, LlvmValueRef::I32(_)));
        assert!(matches!(child.fields[2].1, LlvmValueRef::F32(_)));
        assert!(matches!(child.fields[3].1, LlvmValueRef::F64(_)));
        assert!(matches!(value.fields[1].1, LlvmValueRef::I64(_)));
    }
    let first_names = body[..split]
        .iter()
        .filter_map(|line| line.split_once(" = ").map(|(name, _)| name))
        .collect::<std::collections::BTreeSet<_>>();
    assert!(body[split..]
        .iter()
        .all(|line| { !first_names.contains(line.split_once(" = ").unwrap().0) }));
    let body = body.join("\n");
    assert_eq!(body.matches("extractvalue [5 x i64] %input").count(), 5);
    assert_eq!(body.matches("extractvalue [5 x i64] %output").count(), 5);
    for prohibited in ["call ", "alloca ", "load ", "store ", "ptr ", "ret "] {
        assert!(!body.contains(prohibited), "{prohibited}");
    }
}

#[test]
fn pure_record_transport_does_not_inherit_owned_variant_name_semantics() {
    let name = format!("{}Choice", yir_core::OWNED_VARIANT_UNION_LAYOUT_PREFIX);
    let inner_layout = format!("{name}{{tag:i64;payload:i64}}");
    let exact = record(
        &name,
        vec![
            ("tag", LlvmValueRef::I64("17".to_owned())),
            ("payload", LlvmValueRef::I64("29".to_owned())),
        ],
    );
    let variant = record("Choice.One", vec![]);
    for nested in [false, true] {
        let (encoded, exact, wrong) = if nested {
            (
                format!("Packet{{child:{inner_layout}}}"),
                record("Packet", vec![("child", exact.clone())]),
                record("Packet", vec![("child", variant.clone())]),
            )
        } else {
            (inner_layout.clone(), exact.clone(), variant.clone())
        };
        let plan = NativeValueLayout::parse(&encoded).unwrap();
        assert!(
            plan.prepare(&wrong).is_err(),
            "a variant must not synthesize zero record fields"
        );
        let prepared = plan
            .prepare(&exact)
            .expect("an exact nominal record stays a record");
        let mut body = Vec::new();
        prepared.emit(&mut body, &mut 0);
        let body = body.join("\n");
        assert!(body.contains("poison, i64 17, 0"));
        assert!(body.contains(", i64 29, 1"));
    }
}
