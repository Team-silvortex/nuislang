use super::*;
use nuis_semantics::model::NirStructField;

pub(super) fn value(carries: &[String], ty: Option<&NirTypeRef>) -> NirExpr {
    if let Some(ty) = ty {
        NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: carries
                .iter()
                .enumerate()
                .map(|(index, name)| (format!("carry{index}"), NirExpr::Var(name.clone())))
                .collect(),
        }
    } else if let Some(name) = carries.first() {
        NirExpr::Var(name.clone())
    } else {
        NirExpr::Int(0)
    }
}

pub(super) fn state_type(
    carries: &[String],
    names: &mut BTreeSet<String>,
    structs: &mut Vec<NirStructDef>,
) -> NirTypeRef {
    let name = branches::fresh_name("__nuis_scalar_carries", names);
    structs.push(NirStructDef {
        visibility: NirVisibility::Private,
        annotations: vec![],
        name: name.clone(),
        generic_params: vec![],
        where_bounds: vec![],
        fields: carries
            .iter()
            .enumerate()
            .map(|(index, _)| NirStructField {
                visibility: NirVisibility::Private,
                annotations: vec![],
                name: format!("carry{index}"),
                ty: scalar_type("i64"),
            })
            .collect(),
    });
    scalar_type(&name)
}

pub(super) fn projected_call(
    temporary: String,
    ty: &NirTypeRef,
    carries: &[String],
    call: NirExpr,
) -> Vec<NirStmt> {
    let mut body = vec![NirStmt::Let {
        name: temporary.clone(),
        ty: Some(ty.clone()),
        value: call,
    }];
    body.extend(
        carries
            .iter()
            .enumerate()
            .map(|(index, name)| NirStmt::Let {
                name: name.clone(),
                ty: Some(scalar_type("i64")),
                value: NirExpr::FieldAccess {
                    base: Box::new(NirExpr::Var(temporary.clone())),
                    field: format!("carry{index}"),
                },
            }),
    );
    body
}
