use std::collections::BTreeMap;
use yir_core::{Node, OwnedStructLayout, YirFunction, YirValueOwnership};

pub(crate) struct AggregateCall<'a> {
    pub callee: &'a str,
    pub layout: OwnedStructLayout,
    pub operands: &'a [String],
}

/// A restricted view of the existing call opcode, not a new aggregate ABI.
pub(crate) fn parse(node: &Node) -> Result<Option<AggregateCall<'_>>, String> {
    if node.op.instruction != "call_owned_struct" {
        return Ok(None);
    }
    let [callee, encoded, operands @ ..] = node.op.args.as_slice() else {
        return Err(format!(
            "native aggregate call `{}` is missing target/layout",
            node.name
        ));
    };
    let (layout, _) = scalar_value_layout(encoded)
        .map_err(|error| format!("native aggregate call `{}`: {error}", node.name))?;
    Ok(Some(AggregateCall {
        callee,
        layout,
        operands,
    }))
}

// Ordinary returns keep declared field names. Scoped iterations separately check
// their carry schema before sharing this bounded, resource-free value contract.
pub(super) fn result_layout(
    function: &YirFunction,
    nodes: &BTreeMap<&str, &Node>,
) -> Result<OwnedStructLayout, String> {
    let fail = || {
        format!(
            "native aggregate helper `{}` return layout drift",
            function.name
        )
    };
    let result = function.result.as_ref().ok_or_else(fail)?;
    let node = nodes.get(result.node.as_str()).ok_or_else(fail)?;
    if result.ownership != YirValueOwnership::Owned
        || node.op.instruction != "return_owned_struct"
        || node.op.args.len() != 2
    {
        return Err(fail());
    }
    let (layout, _) =
        scalar_value_layout(&node.op.args[1]).map_err(|error| format!("{}: {error}", fail()))?;
    if layout.type_name != result.ty {
        return Err(fail());
    }
    Ok(layout)
}

pub(super) fn scalar_value_layout(encoded: &str) -> Result<(OwnedStructLayout, usize), String> {
    // Share only the bounded value schema. Helper graph/call admission and scoped
    // iteration carry validation remain independent of callback registration.
    let value = super::ScalarStateLayout::parse(encoded)?;
    Ok((
        yir_core::parse_owned_struct_layout(encoded)?,
        value.fields().len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(args: Vec<String>) -> Node {
        Node {
            name: "branch_result".to_owned(),
            resource: "cpu".to_owned(),
            op: yir_core::Operation::parse("cpu.call_owned_struct", args).unwrap(),
        }
    }

    #[test]
    fn flat_return_layout_has_a_slot_bound_not_precombined_arities() {
        for count in [0, 1, 2, 7, 64, 65] {
            let fields = (0..count)
                .map(|i| format!("carry{i}:i64"))
                .collect::<Vec<_>>()
                .join(";");
            let node = call(vec!["branch".to_owned(), format!("Values{{{fields}}}")]);
            assert_eq!(parse(&node).is_ok(), (1..=64).contains(&count));
        }
    }

    #[test]
    fn scalar_return_call_rejects_missing_layout_and_resource_payloads() {
        for args in [vec![], vec!["branch".to_owned()]] {
            assert!(parse(&call(args)).is_err());
        }
        for layout in [
            "Values{carry0:i64;carry0:i64}",
            "Values{carry0:Bytes}",
            "Values{carry0:String}",
            "Values{carry0:Nested{x:Bytes}}",
            "Values{carry0:Nested{}}",
            "Values{carry0:A{x:i64};carry0:B{y:i64}}",
        ] {
            assert!(
                parse(&call(vec!["branch".to_owned(), layout.to_owned()])).is_err(),
                "{layout}"
            );
        }
    }

    #[test]
    fn ordinary_flat_returns_preserve_declared_names_without_a_loop_carry_schema() {
        for layout in ["Values{value:i64}", "Values{second:i64;first:i64}"] {
            let node = call(vec!["branch".to_owned(), layout.to_owned()]);
            assert_eq!(
                parse(&node).unwrap().unwrap().layout,
                yir_core::parse_owned_struct_layout(layout).unwrap()
            );
        }
    }

    #[test]
    fn ordinary_nested_returns_admit_scalar_kinds_without_a_loop_carry_schema() {
        let encoded = "Values{child:Nested{flag:bool;tag:i32;count:i64;gain:f32;scale:f64}}";
        let node = call(vec!["branch".to_owned(), encoded.to_owned()]);
        let layout = parse(&node).unwrap().unwrap().layout;
        assert_eq!(
            layout,
            yir_core::parse_owned_struct_layout(encoded).unwrap()
        );
        assert_eq!(scalar_value_layout(encoded).unwrap().1, 5);
    }
}
