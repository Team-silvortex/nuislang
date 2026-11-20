use super::Kind;
use std::collections::{BTreeMap, BTreeSet};
use yir_core::{
    Node, OwnedStructFieldLayout, OwnedStructScalarLayout, YirFunction, YirModule,
    YirValueOwnership,
};

pub(super) fn is_call(node: &Node) -> bool {
    matches!(
        node.op.instruction.as_str(),
        "call_owned_struct" | "call_bool" | "call_i64"
    )
}

pub(super) fn target<'a>(module: &'a YirModule, node: &Node) -> Result<&'a YirFunction, String> {
    if !is_call(node) {
        return Err("expected a resource scalar/record value helper call".into());
    }
    if node.op.instruction == "call_owned_struct" {
        let call = crate::native_session::aggregates::parse(node)?.unwrap();
        record_kind(&call.layout)?;
    }
    let helper = crate::native_session::value_helpers::target(module, node)?;
    for parameter in &helper.parameters {
        parameter_kind(parameter)?;
    }
    Ok(helper)
}

pub(super) fn validate(
    module: &YirModule,
    function: &YirFunction,
    metadata: &BTreeSet<String>,
) -> Result<(), String> {
    crate::native_session::value_helpers::validate(module, function, metadata)?;
    for node in module
        .nodes
        .iter()
        .filter(|n| function.body_nodes.contains(&n.name))
    {
        // Resource callbacks have no scalar loop-work counter. Do not inherit
        // the broader scalar-session loop or call grants through this adapter.
        if node.op.instruction.starts_with("loop_")
            || (node.op.instruction.starts_with("call_") && !is_call(node))
        {
            return Err("resource value helper requires a bounded scalar/record-call body".into());
        }
        if is_call(node) {
            target(module, node)?;
        }
    }
    Ok(())
}

pub(super) fn check(
    module: &YirModule,
    node: &Node,
    values: &BTreeMap<String, Kind>,
) -> Result<Kind, String> {
    let helper = target(module, node)?;
    let call = crate::native_session::aggregates::parse(node)?;
    let operands = call
        .as_ref()
        .map_or(&node.op.args[1..], |call| call.operands);
    // target() checks arity; match each ready operand to its declared kind,
    // rather than widening bools into the private i64 return-slot transport.
    for (operand, parameter) in operands.iter().zip(&helper.parameters) {
        if values.get(operand) != Some(&parameter_kind(parameter)?) {
            return Err(
                "resource value helper operands must match ready declared i64/bool values".into(),
            );
        }
    }
    call.map_or_else(
        || {
            Ok(if node.op.instruction == "call_bool" {
                Kind::Bool
            } else {
                Kind::I64
            })
        },
        |call| record_kind(&call.layout),
    )
}

fn parameter_kind(parameter: &yir_core::YirFunctionParameter) -> Result<Kind, String> {
    match (parameter.ty.as_str(), &parameter.ownership) {
        ("i64", YirValueOwnership::Value) => Ok(Kind::I64),
        ("bool", YirValueOwnership::Value) => Ok(Kind::Bool),
        _ => Err("resource value helper requires i64/bool value parameters".into()),
    }
}

fn record_kind(layout: &yir_core::OwnedStructLayout) -> Result<Kind, String> {
    let fields = layout
        .fields
        .iter()
        .map(|(name, kind)| {
            let kind = match kind {
                OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::I64) => Kind::I64,
                OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::Bool) => Kind::Bool,
                OwnedStructFieldLayout::Struct(nested) => record_kind(nested)?,
                _ => return Err("resource value helper requires i64/bool record leaves".into()),
            };
            Ok((name.clone(), kind))
        })
        .collect::<Result<_, String>>()?;
    Ok(Kind::Record(layout.type_name.clone(), fields))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_value_parameters_preserve_exact_kinds_and_value_ownership() {
        for ty in [
            "bool",
            "i64",
            "i32",
            "f32",
            "f64",
            "Bytes",
            "ref Buffer",
            "Flags",
        ] {
            for ownership in [
                YirValueOwnership::Value,
                YirValueOwnership::Owned,
                YirValueOwnership::Borrowed,
            ] {
                let parameter = yir_core::YirFunctionParameter {
                    name: "value".into(),
                    node: "param".into(),
                    ty: ty.into(),
                    ownership,
                };
                let kind = parameter_kind(&parameter);
                let allowed =
                    matches!(ty, "bool" | "i64") && parameter.ownership == YirValueOwnership::Value;
                assert_eq!(kind.is_ok(), allowed, "{parameter:?}");
                if allowed {
                    assert!(kind.unwrap() == if ty == "bool" { Kind::Bool } else { Kind::I64 });
                }
            }
        }
    }

    #[test]
    fn nested_resource_value_layout_keeps_the_shared_depth_bound() {
        for depth in [1, 12, 64, 65] {
            let encoded = (0..depth).fold("i64".to_owned(), |inner, i| {
                format!("L{i}{{value:{inner}}}")
            });
            let node = Node {
                name: "result".into(),
                resource: "cpu".into(),
                op: yir_core::Operation::parse(
                    "cpu.call_owned_struct",
                    vec!["measure".into(), encoded],
                )
                .unwrap(),
            };
            let parsed = crate::native_session::aggregates::parse(&node);
            if depth <= 64 {
                assert!(record_kind(&parsed.unwrap().unwrap().layout)
                    .unwrap()
                    .is_record_value());
            } else {
                assert!(parsed.err().unwrap().contains("maximum depth"));
            }
        }
    }

    #[test]
    fn boolean_result_admission_keeps_other_scalar_and_resource_leaves_closed() {
        for (kind, allowed) in [
            ("bool", true),
            ("i64", true),
            ("i32", false),
            ("f32", false),
            ("f64", false),
            ("Bytes", false),
            ("String", false),
        ] {
            let layout = yir_core::parse_owned_struct_layout(&format!(
                "Result{{flags:Flags{{value:{kind}}};count:i64}}"
            ))
            .unwrap();
            assert_eq!(record_kind(&layout).is_ok(), allowed, "{kind}");
        }
    }
}
