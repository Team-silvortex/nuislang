use super::validate_scalar_leaf;
use yir_core::{
    parse_owned_struct_layout, Node, OwnedStructFieldLayout, OwnedStructLayout,
    OwnedStructScalarLayout, StructValue, Value, VariantUnionValue,
    OWNED_VARIANT_UNION_LAYOUT_PREFIX,
};

pub(super) fn validate_call_result(node: &Node, value: &Value) -> Result<(), String> {
    let valid = match (node.op.instruction.as_str(), value) {
        ("call_bool", Value::Bool(_))
        | ("call_i32", Value::I32(_))
        | ("call_i64", Value::Int(_))
        | ("call_f32", Value::F32(_))
        | ("call_f64", Value::F64(_))
        | ("call_owned_bytes", Value::OwnedBytes(_))
        | ("call_owned_struct", Value::Struct(_)) => true,
        // Enum helpers use the same owned aggregate call instruction as structs.
        // Admit the canonical union shape, not an arbitrary aggregate result.
        ("call_owned_struct", Value::VariantUnion(value)) => node
            .op
            .args
            .get(1)
            .and_then(|source| parse_owned_struct_layout(source).ok())
            .is_some_and(|layout| union_matches(&layout, value)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(format!(
            "node `{}` received incompatible result {value} from `{}`",
            node.name, node.op.args[0]
        ))
    }
}

fn union_matches(layout: &OwnedStructLayout, value: &VariantUnionValue) -> bool {
    if layout
        .type_name
        .strip_prefix(OWNED_VARIANT_UNION_LAYOUT_PREFIX)
        != Some(value.parent_type_name.as_str())
        || !value.variants.contains_key(&value.active_variant)
        || layout.fields.len() != value.variants.len() + 1
    {
        return false;
    }
    let mut fields = std::collections::BTreeSet::new();
    layout.fields.iter().all(|(name, field)| {
        if !fields.insert(name.as_str()) {
            return false;
        }
        if name == "tag" {
            return *field == OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::I64);
        }
        let OwnedStructFieldLayout::Struct(nested) = field else {
            return false;
        };
        nested.type_name == *name
            && value
                .variants
                .get(name)
                .is_some_and(|value| struct_matches(nested, value))
    }) && fields.contains("tag")
}

fn struct_matches(layout: &OwnedStructLayout, value: &StructValue) -> bool {
    layout.type_name == value.type_name
        && layout.fields.len() == value.fields.len()
        && layout
            .fields
            .iter()
            .zip(&value.fields)
            .all(|((name, field), (actual, value))| {
                name == actual
                    && match (field, value) {
                        (OwnedStructFieldLayout::Scalar(kind), value) => {
                            validate_scalar_leaf(*kind, value).is_ok()
                        }
                        (OwnedStructFieldLayout::Struct(nested), Value::Struct(value)) => {
                            struct_matches(nested, value)
                        }
                        (OwnedStructFieldLayout::Struct(nested), Value::VariantUnion(value)) => {
                            union_matches(nested, value)
                        }
                        _ => false,
                    }
            })
}

#[cfg(test)]
mod tests {
    use super::*;
    use yir_core::Operation;

    const LAYOUT: &str = "__nuis_variant_union__Result{tag:i64;Result.Ok:Result.Ok{value:i64};Result.Err:Result.Err{}}";

    fn node(layout: &str) -> Node {
        Node {
            name: "call".to_owned(),
            resource: "cpu0".to_owned(),
            op: Operation {
                module: "cpu".to_owned(),
                instruction: "call_owned_struct".to_owned(),
                args: vec!["helper".to_owned(), layout.to_owned()],
            },
        }
    }

    fn union() -> VariantUnionValue {
        VariantUnionValue {
            parent_type_name: "Result".to_owned(),
            active_variant: "Result.Ok".to_owned(),
            variants: [
                (
                    "Result.Ok".to_owned(),
                    StructValue {
                        type_name: "Result.Ok".to_owned(),
                        fields: vec![("value".to_owned(), Value::Int(42))],
                    },
                ),
                (
                    "Result.Err".to_owned(),
                    StructValue {
                        type_name: "Result.Err".to_owned(),
                        fields: vec![],
                    },
                ),
            ]
            .into_iter()
            .collect(),
        }
    }

    #[test]
    fn owned_calls_accept_canonical_enum_results_and_reject_drift() {
        for active in ["Result.Ok", "Result.Err"] {
            let mut value = union();
            value.active_variant = active.to_owned();
            validate_call_result(&node(LAYOUT), &Value::VariantUnion(value)).unwrap();
        }
        for layout in [
            "Result",
            "Result{}",
            "__nuis_variant_union__Other{tag:i64}",
            &LAYOUT.replace("tag:i64", "tag:bool"),
            &LAYOUT.replace("tag:i64;", ""),
            &LAYOUT.replace("value:i64", "value:f64"),
        ] {
            assert!(
                validate_call_result(&node(layout), &Value::VariantUnion(union())).is_err(),
                "{layout}"
            );
        }
        let mut wrong_parent = union();
        wrong_parent.parent_type_name = "Other".to_owned();
        let mut wrong_active = union();
        wrong_active.active_variant = "Result.Missing".to_owned();
        let mut missing_variant = union();
        missing_variant.variants.remove("Result.Err");
        let mut wrong_payload = union();
        wrong_payload.variants.get_mut("Result.Ok").unwrap().fields[0].1 = Value::Bool(true);
        for value in [wrong_parent, wrong_active, missing_variant, wrong_payload] {
            assert!(validate_call_result(&node(LAYOUT), &Value::VariantUnion(value)).is_err());
        }
        assert!(validate_call_result(&node(LAYOUT), &Value::Int(42)).is_err());
    }
}
