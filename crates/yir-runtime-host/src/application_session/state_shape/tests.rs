use super::*;
use yir_core::{parse_owned_struct_layout, YirValueOwnership};

fn parameters(fields: &[(&str, &str)]) -> Vec<YirFunctionParameter> {
    fields
        .iter()
        .enumerate()
        .map(|(index, (name, ty))| YirFunctionParameter {
            name: format!("state.{name}"),
            ty: (*ty).to_owned(),
            ownership: YirValueOwnership::Value,
            node: format!("p{index}"),
        })
        .collect()
}

fn record(ty: &str, fields: Vec<(&str, Value)>) -> Value {
    Value::Struct(StructValue {
        type_name: ty.to_owned(),
        fields: fields.into_iter().map(|(n, v)| (n.to_owned(), v)).collect(),
    })
}

fn shape() -> StateShape {
    let mut shape = StateShape::bind(
        "State",
        "state.",
        &parameters(&[("first", "i64"), ("inner.x", "i32"), ("inner.y", "bool")]),
    )
    .unwrap();
    shape
        .bind_layout(
            &parse_owned_struct_layout("State{first:i64;inner:Inner{x:i32;y:bool}}").unwrap(),
        )
        .unwrap();
    shape
}

fn inner() -> Value {
    record(
        "Inner",
        vec![("y", Value::Bool(true)), ("x", Value::I32(-17))],
    )
}

#[test]
fn reorders_named_fields_at_every_depth_and_is_idempotent() {
    let shape = shape();
    let actual = shape
        .normalize(record(
            "State",
            vec![("inner", inner()), ("first", Value::Int(9))],
        ))
        .unwrap();
    let expected = record(
        "State",
        vec![
            ("first", Value::Int(9)),
            (
                "inner",
                record(
                    "Inner",
                    vec![("x", Value::I32(-17)), ("y", Value::Bool(true))],
                ),
            ),
        ],
    );
    assert_eq!(actual, expected);
    assert_eq!(shape.normalize(actual).unwrap(), expected);
}

#[test]
fn preserves_exact_scalar_bits_without_coercion() {
    let params = parameters(&[
        ("b", "bool"),
        ("i", "i32"),
        ("n", "i64"),
        ("f", "f32"),
        ("d", "f64"),
    ]);
    let shape = StateShape::bind("State", "state.", &params).unwrap();
    for (f, d) in [
        (0x8000_0000, 0x8000_0000_0000_0000),
        (0x7fc0_1234, 0x7ff8_0000_0000_4321),
    ] {
        let value = record(
            "State",
            vec![
                ("d", Value::F64(f64::from_bits(d))),
                ("f", Value::F32(f32::from_bits(f))),
                ("n", Value::Int(i64::MIN)),
                ("i", Value::I32(i32::MIN)),
                ("b", Value::Bool(true)),
            ],
        );
        let Value::Struct(value) = shape.normalize(value).unwrap() else {
            panic!("not a struct")
        };
        assert_eq!(
            value
                .fields
                .iter()
                .map(|(n, _)| n.as_str())
                .collect::<Vec<_>>(),
            ["b", "i", "n", "f", "d"]
        );
        let words = params
            .iter()
            .zip(&value.fields)
            .map(|(p, (_, v))| ScalarKind::parse(&p.ty).unwrap().pack(v).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            words,
            [1, i32::MIN as i64 as u64, i64::MIN as u64, u64::from(f), d]
        );
    }
}

#[test]
fn rejects_missing_extra_duplicate_and_spoofed_fields() {
    let shape = shape();
    let malformed = [
        record("State", vec![("inner", inner())]),
        record(
            "State",
            vec![
                ("inner", inner()),
                ("first", Value::Int(1)),
                ("extra", Value::Int(2)),
            ],
        ),
        record(
            "State",
            vec![("first", Value::Int(1)), ("first", Value::Int(2))],
        ),
        record(
            "State",
            vec![("inner.x", inner()), ("first", Value::Int(1))],
        ),
        record(
            "State",
            vec![
                (
                    "inner",
                    record("Inner", vec![("x", Value::I32(1)), ("x", Value::I32(2))]),
                ),
                ("first", Value::Int(1)),
            ],
        ),
        record(
            "State",
            vec![
                (
                    "inner",
                    record(
                        "Inner",
                        vec![("x", Value::I32(1)), ("extra", Value::Bool(true))],
                    ),
                ),
                ("first", Value::Int(1)),
            ],
        ),
        record(
            "State",
            vec![("inner", record("Inner", vec![])), ("first", Value::Int(1))],
        ),
        record(
            "State",
            vec![("inner", inner()), ("first", record("Empty", vec![]))],
        ),
    ];
    for (case, value) in malformed.into_iter().enumerate() {
        assert!(
            shape.normalize(value).is_err(),
            "accepted malformed case {case}"
        );
    }
}

#[test]
fn rejects_wrong_nominal_types_and_resource_or_scalar_kind_substitution() {
    let shape = shape();
    for value in [
        record("Other", vec![("inner", inner()), ("first", Value::Int(1))]),
        record(
            "State",
            vec![
                (
                    "inner",
                    record(
                        "OtherInner",
                        vec![("x", Value::I32(1)), ("y", Value::Bool(true))],
                    ),
                ),
                ("first", Value::Int(1)),
            ],
        ),
        record(
            "State",
            vec![("inner", Value::Int(1)), ("first", Value::Int(1))],
        ),
        record("State", vec![("inner", inner()), ("first", Value::I32(1))]),
        record(
            "State",
            vec![("inner", inner()), ("first", Value::Pointer(None))],
        ),
        record(
            "State",
            vec![("inner", inner()), ("first", Value::OwnedBytes(vec![1]))],
        ),
        Value::Int(1),
    ] {
        assert!(shape.normalize(value).is_err());
    }
}

#[test]
fn rejects_ambiguous_or_non_scalar_flattened_signatures() {
    for fields in [
        vec![("a", "i64"), ("a", "i64")],
        vec![("a", "i64"), ("a.x", "i64")],
        vec![("a.x", "i64"), ("a", "i64")],
        vec![("a.x", "i64"), ("b", "i64"), ("a.y", "i64")],
        vec![("", "i64")],
        vec![("a..x", "i64")],
        vec![("a.", "i64")],
        vec![("a", "String")],
    ] {
        assert!(
            StateShape::bind("State", "state.", &parameters(&fields)).is_err(),
            "{fields:?}"
        );
    }
    let mut params = parameters(&[("a", "i64")]);
    params[0].name = "other.a".to_owned();
    assert!(StateShape::bind("State", "state.", &params).is_err());
}

#[test]
fn metadata_must_agree_across_callbacks_including_nested_nominal_identity() {
    for layout in [
        "Other{first:i64;inner:Inner{x:i32;y:bool}}",
        "State{inner:Inner{x:i32;y:bool};first:i64}",
        "State{first:i64}",
        "State{first:i32;inner:Inner{x:i32;y:bool}}",
        "State{first:i64;inner:Other{x:i32;y:bool}}",
        "State{first:i64;inner:Inner{y:bool;x:i32}}",
        "State{first:i64;inner:Inner{x:i32;x:bool}}",
        "State{first:i64;inner:Inner{x:i32;y:Bytes}}",
        "State{first:i64;inner:i64}",
    ] {
        assert!(
            shape()
                .bind_layout(&parse_owned_struct_layout(layout).unwrap())
                .is_err(),
            "{layout}"
        );
    }
}

#[test]
fn metadata_free_yir_keeps_its_signature_only_nested_boundary() {
    let shape = StateShape::bind("State", "state.", &parameters(&[("inner.x", "i32")])).unwrap();
    let value = record(
        "State",
        vec![("inner", record("HandWritten", vec![("x", Value::I32(1))]))],
    );
    assert_eq!(shape.normalize(value.clone()).unwrap(), value);
}

#[test]
fn reference_states_are_not_limited_to_native_slot_capacity() {
    let fields = (0..128)
        .map(|i| (format!("f{i}"), "i64"))
        .collect::<Vec<_>>();
    let params = parameters(
        &fields
            .iter()
            .map(|(name, ty)| (name.as_str(), *ty))
            .collect::<Vec<_>>(),
    );
    let shape = StateShape::bind("State", "state.", &params).unwrap();
    let value = record(
        "State",
        fields
            .iter()
            .enumerate()
            .rev()
            .map(|(i, (name, _))| (name.as_str(), Value::Int(i as i64)))
            .collect(),
    );
    let Value::Struct(actual) = shape.normalize(value).unwrap() else {
        panic!("not a struct")
    };
    assert_eq!(actual.fields.len(), 128);
    for (index, (name, value)) in actual.fields.iter().enumerate() {
        assert_eq!(name, &format!("f{index}"));
        assert_eq!(value, &Value::Int(index as i64));
    }
}

#[test]
fn bounds_path_nesting_without_rejecting_the_owned_layout_depth_limit() {
    for depth in [64, 65] {
        let path = vec!["nested"; depth].join(".");
        let shape = StateShape::bind("State", "state.", &parameters(&[(&path, "i64")]));
        if depth == 65 {
            assert!(shape.is_err());
            continue;
        }
        let mut value = Value::Int(17);
        for index in (0..depth).rev() {
            value = record(
                if index == 0 { "State" } else { "Nested" },
                vec![("nested", value)],
            );
        }
        assert_eq!(shape.unwrap().normalize(value.clone()).unwrap(), value);
    }
}
