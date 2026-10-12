use super::*;
use nuis_semantics::model::NirStructField;

// A bounded owned value uses the existing aggregate call/return contract.
// No bit is borrowed from either field and no source work is replayed.
#[allow(clippy::too_many_arguments)]
pub(super) fn install(
    mut plan: Plan,
    owner: String,
    child: String,
    field_types: (NirTypeRef, NirTypeRef),
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let type_name = branches::fresh_name("__nuis_effect_pair_value", names);
    let neutral = |ty: &NirTypeRef| match ty.name.as_str() {
        "bool" => NirExpr::Bool(false),
        "i64" => NirExpr::Int(0),
        "f32" => NirExpr::F32("0.0".into()),
        "f64" => NirExpr::F64("0.0".into()),
        _ => unreachable!("owned private fields have exact proved scalar kinds"),
    };
    let fields = [("owner", field_types.0), ("child", field_types.1)];
    let zero = (neutral(&fields[0].1), neutral(&fields[1].1));
    structs.push(NirStructDef {
        visibility: NirVisibility::Private,
        annotations: vec![],
        name: type_name.clone(),
        generic_params: vec![],
        where_bounds: vec![],
        fields: fields
            .iter()
            .map(|(name, ty)| NirStructField {
                visibility: NirVisibility::Private,
                annotations: vec![],
                name: (*name).into(),
                ty: ty.clone(),
            })
            .collect(),
    });
    let record = |a, b| NirExpr::StructLiteral {
        type_name: type_name.clone(),
        type_args: vec![],
        fields: vec![("owner".into(), a), ("child".into(), b)],
    };
    for arm in [&mut plan.yes, &mut plan.no] {
        let SelectedValue::Region(_, value) = arm else {
            unreachable!("private pair arms are proved staged regions")
        };
        *value = record(NirExpr::Var(owner.clone()), NirExpr::Var(child.clone()));
    }
    plan.ty = scalar_type(&type_name);
    let carrier = branches::fresh_name("__nuis_effect_call_pair", bindings);
    plan.destination = Some((carrier.clone(), false));
    let mut output = super::super::install_with_neutral(
        plan,
        record(zero.0, zero.1),
        names,
        bindings,
        helpers,
        structs,
    )
    .0;
    for ((field, ty), name) in fields.into_iter().zip([owner, child]) {
        output.push(NirStmt::Let {
            name,
            ty: Some(ty),
            value: NirExpr::FieldAccess {
                base: Box::new(NirExpr::Var(carrier.clone())),
                field: field.into(),
            },
        });
    }
    output
}
