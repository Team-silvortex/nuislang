use super::*;
use nuis_semantics::model::NirStructField;

#[cfg(test)]
#[path = "conditional_returns_signal_tests.rs"]
mod tests;

pub(super) struct Arm {
    pub inputs: BTreeSet<String>,
    pub has_return: bool,
}

pub(super) fn prepare(
    body: &[NirStmt],
    result: &NirTypeRef,
    scope: &Scope,
    catalog: &ScalarHelpers,
    layouts: &impl control_values::ValueLayouts,
) -> Option<Arm> {
    let inputs = terminal::partial_inputs(body, result, scope, catalog, layouts)?;
    let mut pending = vec![body];
    let mut has_return = false;
    let mut continues = false;
    while let Some(body) = pending.pop() {
        match body.last() {
            Some(NirStmt::Return(Some(_))) => has_return = true,
            Some(NirStmt::If {
                then_body,
                else_body,
                ..
            }) => {
                pending.extend([then_body.as_slice(), else_body.as_slice()]);
            }
            _ => continues = true,
        }
    }
    continues.then_some(Arm { inputs, has_return })
}

pub(super) fn definition(result: &NirTypeRef, names: &mut BTreeSet<String>) -> NirStructDef {
    NirStructDef {
        visibility: NirVisibility::Private,
        annotations: vec![],
        name: branches::fresh_name("__nuis_return_signal", names),
        generic_params: vec![],
        where_bounds: vec![],
        fields: [("exited", scalar_type("bool")), ("value", result.clone())]
            .into_iter()
            .map(|(name, ty)| NirStructField {
                visibility: NirVisibility::Private,
                annotations: vec![],
                name: name.into(),
                ty,
            })
            .collect(),
    }
}

// Only validated bounded terminal trees reach this rewrite. Keep every prefix
// and condition in place; wrap real returns and seed only continuation leaves.
pub(super) fn wrap(
    mut body: Vec<NirStmt>,
    signal: &NirTypeRef,
    result: &NirTypeRef,
    bindings: &mut BTreeSet<String>,
) -> Vec<NirStmt> {
    match body.pop() {
        Some(NirStmt::Return(Some(value))) => {
            // Keep logical returns at an expression root so the established
            // short-circuit outliner can guard their RHS before construction.
            let name = branches::fresh_name("__nuis_return_leaf", bindings);
            body.push(NirStmt::Let {
                name: name.clone(),
                ty: Some(result.clone()),
                value,
            });
            body.push(returned(signal, true, NirExpr::Var(name)));
        }
        Some(NirStmt::If {
            condition,
            then_body,
            else_body,
        }) => {
            body.push(NirStmt::If {
                condition,
                then_body: wrap(then_body, signal, result, bindings),
                else_body: wrap(else_body, signal, result, bindings),
            });
        }
        last => {
            body.extend(last);
            let seed = if result == &scalar_type("bool") {
                NirExpr::Bool(false)
            } else {
                NirExpr::Int(0)
            };
            body.push(returned(signal, false, seed));
        }
    }
    body
}

fn returned(signal: &NirTypeRef, exited: bool, value: NirExpr) -> NirStmt {
    NirStmt::Return(Some(NirExpr::StructLiteral {
        type_name: signal.name.clone(),
        type_args: vec![],
        fields: vec![
            ("exited".into(), NirExpr::Bool(exited)),
            ("value".into(), value),
        ],
    }))
}

pub(super) fn read(
    call: NirExpr,
    signal: &NirTypeRef,
    result: &NirTypeRef,
    bindings: &mut BTreeSet<String>,
) -> Vec<NirStmt> {
    let snapshot = branches::fresh_name("__nuis_return_snapshot", bindings);
    let value = branches::fresh_name("__nuis_return_value", bindings);
    let ready = branches::fresh_name("__nuis_return_exit", bindings);
    let field = |name: &str| NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var(snapshot.clone())),
        field: name.into(),
    };
    vec![
        NirStmt::Let {
            name: snapshot.clone(),
            ty: Some(signal.clone()),
            value: call,
        },
        NirStmt::Let {
            name: value.clone(),
            ty: Some(result.clone()),
            value: field("value"),
        },
        NirStmt::Let {
            name: ready.clone(),
            ty: Some(scalar_type("bool")),
            value: field("exited"),
        },
        NirStmt::If {
            condition: NirExpr::Var(ready),
            then_body: vec![NirStmt::Return(Some(NirExpr::Var(value)))],
            else_body: vec![],
        },
    ]
}
