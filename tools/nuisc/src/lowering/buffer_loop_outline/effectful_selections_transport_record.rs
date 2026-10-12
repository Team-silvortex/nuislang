use super::*;
use nuis_semantics::model::NirStructField;

// Representation follows independently proved field types, not source/backend combinations.
#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    mut plan: Plan,
    fields: Vec<(String, String, NirTypeRef)>,
    type_prefix: &str,
    value_prefix: &str,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let type_name = branches::fresh_name(type_prefix, names);
    let neutral = |ty: &NirTypeRef| match ty.name.as_str() {
        "bool" => NirExpr::Bool(false),
        "i32" => NirExpr::CastI64ToI32(Box::new(NirExpr::Int(0))),
        "i64" => NirExpr::Int(0),
        "f32" => NirExpr::F32("0.0".into()),
        "f64" => NirExpr::F64("0.0".into()),
        _ => unreachable!("owned private fields have exact proved scalar kinds"),
    };
    structs.push(NirStructDef {
        visibility: NirVisibility::Private,
        annotations: vec![],
        name: type_name.clone(),
        generic_params: vec![],
        where_bounds: vec![],
        fields: fields
            .iter()
            .map(|(label, _, ty)| NirStructField {
                visibility: NirVisibility::Private,
                annotations: vec![],
                name: label.clone(),
                ty: ty.clone(),
            })
            .collect(),
    });
    let record = |zero: bool| NirExpr::StructLiteral {
        type_name: type_name.clone(),
        type_args: vec![],
        fields: fields
            .iter()
            .map(|(label, binding, ty)| {
                (
                    label.clone(),
                    if zero {
                        neutral(ty)
                    } else {
                        NirExpr::Var(binding.clone())
                    },
                )
            })
            .collect(),
    };
    for arm in [&mut plan.yes, &mut plan.no] {
        let SelectedValue::Region(_, value) = arm else {
            unreachable!("private field arms are proved staged regions")
        };
        *value = record(false);
    }
    plan.ty = scalar_type(&type_name);
    let carrier = branches::fresh_name(value_prefix, bindings);
    plan.destination = Some((carrier.clone(), false));
    let mut output =
        super::super::install_with_neutral(plan, record(true), names, bindings, helpers, structs).0;
    for (field, name, ty) in fields {
        output.push(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var(carrier.clone())),
                field,
            },
        });
    }
    output
}
