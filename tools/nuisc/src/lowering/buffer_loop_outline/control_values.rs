use super::*;

#[cfg(test)]
#[path = "control_values/tests.rs"]
mod tests;
#[cfg(test)]
#[path = "control_values/typed_tests.rs"]
mod typed_tests;

#[path = "control_values/layouts.rs"]
mod value_layouts;
pub(super) use value_layouts::{TypedLayouts, ValueLayouts};

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

pub(super) fn supported_type(ty: &NirTypeRef, layouts: &impl ValueLayouts) -> bool {
    ty == &scalar_type(&ty.name) && (layouts.scalar(&ty.name) || layouts.fields(&ty.name).is_some())
}

pub(super) fn value_type(
    expr: &NirExpr,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl ValueLayouts,
) -> Option<NirTypeRef> {
    match expr {
        NirExpr::Int(_) => Some(scalar_type("i64")),
        NirExpr::Bool(_) => Some(scalar_type("bool")),
        NirExpr::F32(_) if layouts.scalar("f32") => Some(scalar_type("f32")),
        NirExpr::F64(_) if layouts.scalar("f64") => Some(scalar_type("f64")),
        NirExpr::CastI64ToI32(value) if layouts.scalar("i32") => {
            (value_type(value, scope, catalog, layouts)? == scalar_type("i64"))
                .then(|| scalar_type("i32"))
        }
        NirExpr::CastI32ToI64(value) if layouts.scalar("i32") => {
            (value_type(value, scope, catalog, layouts)? == scalar_type("i32"))
                .then(|| scalar_type("i64"))
        }
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
            let layout = layouts.fields(type_name)?;
            if !type_args.is_empty() || fields.len() != layout.len() {
                return None;
            }
            let mut seen = BTreeSet::new();
            for (name, value) in fields {
                let expected = layouts
                    .fields(type_name)?
                    .find(|(field, _)| *field == name)?
                    .1;
                if !seen.insert(name) || value_type(value, scope, catalog, layouts)? != expected {
                    return None;
                }
            }
            Some(scalar_type(type_name))
        }
        NirExpr::FieldAccess { base, field } => {
            let base = value_type(base, scope, catalog, layouts)?;
            let ty = layouts
                .fields(&base.name)?
                .find(|(name, _)| *name == field)
                .map(|(_, ty)| ty);
            ty
        }
        _ => None,
    }
}

pub(super) fn zero_value(ty: &NirTypeRef, layouts: &impl ValueLayouts) -> NirExpr {
    if let Some(fields) = layouts.fields(&ty.name) {
        NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: fields
                .map(|(name, ty)| (name.to_owned(), zero_value(&ty, layouts)))
                .collect(),
        }
    } else {
        assert!(supported_type(ty, layouts));
        match ty.name.as_str() {
            "bool" => NirExpr::Bool(false),
            "i32" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0))),
            "i64" => NirExpr::Int(0),
            "f32" => NirExpr::F32("0.0".into()),
            "f64" => NirExpr::F64("0.0".into()),
            _ => unreachable!("admitted scalar kind"),
        }
    }
}

pub(super) fn collect_inputs(expr: &NirExpr, inputs: &mut BTreeSet<String>) {
    match expr {
        NirExpr::Var(name) => {
            inputs.insert(name.clone());
        }
        NirExpr::Binary { lhs, rhs, .. } => {
            collect_inputs(lhs, inputs);
            collect_inputs(rhs, inputs);
        }
        NirExpr::Call { args, .. } => {
            for arg in args {
                collect_inputs(arg, inputs);
            }
        }
        NirExpr::StructLiteral { fields, .. } => {
            for (_, value) in fields {
                collect_inputs(value, inputs);
            }
        }
        // Enclosing branch capture runs after loop normalization, which inserts
        // these private conversions. Source admission still uses value_type.
        NirExpr::FieldAccess { base, .. }
        | NirExpr::CastI64ToI32(base)
        | NirExpr::CastI32ToI64(base)
        | NirExpr::CastBoolToI64(base)
        | NirExpr::CastI64ToBool(base) => collect_inputs(base, inputs),
        NirExpr::Int(_) | NirExpr::Bool(_) | NirExpr::F32(_) | NirExpr::F64(_) => {}
        _ => unreachable!("admitted pure value expression"),
    }
}

pub(super) fn has_aggregate_expressions(body: &[NirStmt]) -> bool {
    fn expression(expr: &NirExpr) -> bool {
        match expr {
            NirExpr::StructLiteral { .. } | NirExpr::FieldAccess { .. } => true,
            NirExpr::Binary { lhs, rhs, .. } => expression(lhs) || expression(rhs),
            NirExpr::Call { args, .. } => args.iter().any(expression),
            _ => false,
        }
    }
    body.iter().any(|stmt| match stmt {
        NirStmt::Let { value, .. } => expression(value),
        NirStmt::If {
            condition,
            then_body,
            else_body,
        } => {
            expression(condition)
                || has_aggregate_expressions(then_body)
                || has_aggregate_expressions(else_body)
        }
        _ => false,
    })
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
