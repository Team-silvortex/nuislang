use std::collections::{BTreeMap, BTreeSet};

use crate::{variant_select::variant_tag_value, LlvmValueRef};
use yir_core::{
    Node, OwnedStructFieldLayout as Field, OwnedStructLayout, OwnedStructScalarLayout as Scalar,
    YirFunction, OWNED_VARIANT_UNION_LAYOUT_PREFIX as PREFIX,
};

/// Only the discriminant crosses a private call. Empty case records describe
/// membership; they are not fields to pad or public state slots.
#[derive(Clone, PartialEq, Eq)]
pub(super) struct UnitEnum {
    parent: String,
    cases: BTreeSet<String>,
}

impl UnitEnum {
    pub(super) fn parse(encoded: &str) -> Result<Option<(OwnedStructLayout, Self)>, String> {
        if !encoded.starts_with(PREFIX) {
            return Ok(None);
        }
        let layout = yir_core::parse_owned_struct_layout(encoded)?;
        let parent = layout.type_name[PREFIX.len()..].to_owned();
        let invalid = || format!("native unit enum has unsupported layout for `{parent}`");
        let Some(((tag, Field::Scalar(Scalar::I64)), cases)) = layout.fields.split_first() else {
            return Err(invalid());
        };
        if tag != "tag" || parent.is_empty() || !(1..=64).contains(&cases.len()) {
            return Err(invalid());
        }
        let mut names = BTreeSet::new();
        let mut tags = BTreeSet::new();
        for (name, field) in cases {
            if !matches!(field, Field::Struct(s) if s.type_name == *name && s.fields.is_empty())
                || !name
                    .rsplit_once('.')
                    .is_some_and(|(p, c)| p == parent && !c.is_empty())
                || !names.insert(name.clone())
                || !tags.insert(variant_tag_value(name))
            {
                return Err(invalid());
            }
        }
        Ok(Some((
            layout,
            Self {
                parent,
                cases: names,
            },
        )))
    }

    pub(super) fn tag(&self, value: &LlvmValueRef) -> Result<String, String> {
        match value {
            LlvmValueRef::Struct(case)
                if case.fields.is_empty() && self.cases.contains(&case.type_name) =>
            {
                Ok(variant_tag_value(&case.type_name).to_string())
            }
            LlvmValueRef::VariantUnion(union)
                if union.parent_type_name == self.parent
                    && !union.variants.is_empty()
                    && union.variants.iter().all(|(n, c)| {
                        c.type_name == *n && c.fields.is_empty() && self.cases.contains(n)
                    }) =>
            {
                Ok(union.tag_i64.clone())
            }
            _ => Err(format!(
                "native unit enum return does not match `{}`",
                self.parent
            )),
        }
    }
}

pub(super) fn check_predicate(
    node: &Node,
    nodes: &BTreeMap<&str, &Node>,
    function: &YirFunction,
    schemas: &mut BTreeMap<String, UnitEnum>,
) -> Result<(), String> {
    let fail = || {
        format!(
            "native unit enum predicate `{}` requires a matching unit variant",
            node.name
        )
    };
    let [input, case] = node.op.args.as_slice() else {
        return Err(fail());
    };
    let (parent, leaf) = case.rsplit_once('.').ok_or_else(fail)?;
    if parent.is_empty() || leaf.is_empty() {
        return Err(fail());
    }
    // A qualified name alone does not prove enum membership. The compiler's
    // private return layout retains the complete nominal set, even if a local
    // select can construct only a subset of its cases.
    if !schemas.contains_key(parent) {
        let mut declared = None;
        for name in &function.body_nodes {
            let n = nodes.get(name.as_str()).ok_or_else(fail)?;
            if !matches!(
                n.op.instruction.as_str(),
                "return_owned_struct" | "call_owned_struct"
            ) {
                continue;
            }
            let encoded = n.op.args.get(1).ok_or_else(fail)?;
            if encoded
                .strip_prefix(PREFIX)
                .and_then(|s| s.split_once('{'))
                .map(|(p, _)| p)
                != Some(parent)
            {
                continue;
            }
            let (_, schema) = UnitEnum::parse(encoded)?.ok_or_else(fail)?;
            if declared.as_ref().is_some_and(|d| d != &schema) {
                return Err(fail());
            }
            declared = Some(schema);
        }
        schemas.insert(parent.to_owned(), declared.ok_or_else(fail)?);
    }
    let declared = &schemas[parent];
    if !declared.cases.contains(case) {
        return Err(fail());
    }
    let mut pending = vec![input];
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name) {
            continue;
        }
        if seen.len() > 4096 {
            return Err(fail());
        }
        let src = nodes.get(name.as_str()).ok_or_else(fail)?;
        match (src.op.full_name().as_str(), src.op.args.as_slice()) {
            ("cpu.struct", [n]) if declared.cases.contains(n) => {}
            ("cpu.select", [_, a, b]) => pending.extend([a, b]),
            ("cpu.call_owned_struct", [_, encoded, ..]) => {
                let (_, schema) = UnitEnum::parse(encoded)?.ok_or_else(fail)?;
                if &schema != declared {
                    return Err(fail());
                }
            }
            _ => return Err(fail()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn helper_only_codec_requires_distinct_nominal_unit_cases_and_i64_tag() {
        let prefix = format!("{PREFIX}Reason");
        let layout = format!("{prefix}{{tag:i64;Reason.Ok:Reason.Ok{{}};Reason.No:Reason.No{{}}}}");
        assert!(UnitEnum::parse(&layout).unwrap().is_some());
        assert!(super::super::aggregate_values::NativeValueReturn::callback(&layout).is_err());
        for fields in [
            "tag:i64",
            "tag:bool;Reason.Ok:Reason.Ok{}",
            "tag:i64;Other.Ok:Other.Ok{}",
            "tag:i64;Reason.Ok:Reason.Other{}",
            "tag:i64;Reason.Ok:Reason.Ok{x:i64}",
            "tag:i64;Reason.Ok:Reason.Ok{};Reason.Ok:Reason.Ok{}",
        ] {
            assert!(
                UnitEnum::parse(&format!("{prefix}{{{fields}}}")).is_err(),
                "{fields}"
            );
        }
    }
}
