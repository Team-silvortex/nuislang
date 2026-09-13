use super::*;

#[cfg(test)]
#[path = "control_values/tests.rs"]
mod tests;

pub(super) type FlatLayouts = BTreeMap<String, Vec<String>>;

// This is source-level pure-value normalization. Native slot/layout admission
// remains a separate backend contract, not a compiler-wide ABI restriction.
pub(super) fn layouts(module: &NirModule) -> FlatLayouts {
    module
        .structs
        .iter()
        .filter(|definition| {
            definition.generic_params.is_empty()
                && definition.where_bounds.is_empty()
                && !definition.fields.is_empty()
                && definition.fields.iter().all(|f| f.ty == scalar_type("i64"))
        })
        .map(|definition| {
            (
                definition.name.clone(),
                definition.fields.iter().map(|f| f.name.clone()).collect(),
            )
        })
        .collect()
}

pub(super) fn supported_type(ty: &NirTypeRef, layouts: &FlatLayouts) -> bool {
    ty == &scalar_type("i64")
        || ty == &scalar_type("bool")
        || (layouts.contains_key(&ty.name) && ty == &scalar_type(&ty.name))
}

pub(super) fn value_type(
    expr: &NirExpr,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &FlatLayouts,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::Var(name) => scope
            .get(name)
            .filter(|ty| supported_type(ty, layouts))
            .cloned(),
        NirExpr::Binary { op, lhs, rhs } => binary_type(
            *op,
            value_type(lhs, scope, catalog, layouts)?,
            value_type(rhs, scope, catalog, layouts)?,
        ),
        NirExpr::Call { callee, args } => {
            scalar_helpers::value_call_type(callee, args, scope, catalog, layouts)
        }
        NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } => {
            let layout = layouts.get(type_name)?;
            if !type_args.is_empty() || fields.len() != layout.len() {
                return None;
            }
            let mut seen = BTreeSet::new();
            for (name, value) in fields {
                if !layout.contains(name)
                    || !seen.insert(name)
                    || value_type(value, scope, catalog, layouts)? != scalar_type("i64")
                {
                    return None;
                }
            }
            Some(scalar_type(type_name))
        }
        NirExpr::FieldAccess { base, field } => {
            let base = value_type(base, scope, catalog, layouts)?;
            layouts
                .get(&base.name)?
                .contains(field)
                .then(|| scalar_type("i64"))
        }
        _ => None,
    }
}

pub(super) fn zero_value(ty: &NirTypeRef, layouts: &FlatLayouts) -> NirExpr {
    if let Some(fields) = layouts.get(&ty.name) {
        NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: fields
                .iter()
                .map(|name| (name.clone(), NirExpr::Int(0)))
                .collect(),
        }
    } else if ty == &scalar_type("bool") {
        NirExpr::Bool(false)
    } else {
        assert_eq!(ty, &scalar_type("i64"));
        NirExpr::Int(0)
    }
}

pub(super) fn binary_type(op: NirBinaryOp, lhs: NirTypeRef, rhs: NirTypeRef) -> Option<NirTypeRef> {
    if lhs != rhs {
        return None;
    }
    match op {
        NirBinaryOp::Add
        | NirBinaryOp::Sub
        | NirBinaryOp::Mul
        | NirBinaryOp::Div
        | NirBinaryOp::Rem
            if lhs == scalar_type("i64") =>
        {
            Some(lhs)
        }
        NirBinaryOp::Lt | NirBinaryOp::Le | NirBinaryOp::Gt | NirBinaryOp::Ge
            if lhs == scalar_type("i64") =>
        {
            Some(scalar_type("bool"))
        }
        NirBinaryOp::Eq | NirBinaryOp::Ne
            if lhs == scalar_type("i64") || lhs == scalar_type("bool") =>
        {
            Some(scalar_type("bool"))
        }
        NirBinaryOp::And | NirBinaryOp::Or | NirBinaryOp::Xor if lhs == scalar_type("bool") => {
            Some(lhs)
        }
        _ => None,
    }
}
