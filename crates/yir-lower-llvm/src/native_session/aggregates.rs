use std::collections::BTreeMap;
use yir_core::{
    Node, OwnedStructFieldLayout, OwnedStructLayout, OwnedStructScalarLayout, YirFunction,
    YirValueOwnership,
};

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
    let layout = yir_core::parse_owned_struct_layout(encoded)?;
    if !flat_i64_values(&layout) {
        return Err(format!(
            "native aggregate call `{}` requires a flat i64 value layout",
            node.name
        ));
    }
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
    let layout = yir_core::parse_owned_struct_layout(&node.op.args[1])?;
    if layout.type_name != result.ty || !flat_i64_values(&layout) {
        return Err(fail());
    }
    Ok(layout)
}

fn flat_i64_values(layout: &OwnedStructLayout) -> bool {
    let mut names = std::collections::BTreeSet::new();
    !layout.fields.is_empty()
        && layout.fields.len() <= super::MAX_SCALAR_SLOTS
        && layout.fields.iter().all(|(name, kind)| {
            names.insert(name)
                && kind == &OwnedStructFieldLayout::Scalar(OwnedStructScalarLayout::I64)
        })
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
    fn flat_return_call_rejects_missing_layout_and_other_payload_families() {
        for args in [vec![], vec!["branch".to_owned()]] {
            assert!(parse(&call(args)).is_err());
        }
        for layout in [
            "Values{carry0:i64;carry0:i64}",
            "Values{carry0:bool}",
            "Values{carry0:i32}",
            "Values{carry0:f32}",
            "Values{carry0:f64}",
            "Values{carry0:Bytes}",
            "Values{carry0:String}",
            "Values{carry0:Nested{x:i64}}",
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
}
