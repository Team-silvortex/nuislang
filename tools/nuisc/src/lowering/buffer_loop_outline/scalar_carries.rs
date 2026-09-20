use super::*;
use nuis_semantics::model::NirStructField;

pub(super) struct Plan {
    bindings: Vec<(String, NirTypeRef, Option<Vec<String>>)>,
}

impl Plan {
    pub(super) fn new(
        carries: &[String],
        scope: &Scope,
        layouts: Option<&control_values::FlatLayouts>,
    ) -> Self {
        let bindings = carries
            .iter()
            .map(|name| {
                let ty = scope[name].clone();
                let fields = if ty == scalar_type("i64") || ty == scalar_type("bool") {
                    None
                } else {
                    assert_eq!(ty, scalar_type(&ty.name));
                    Some(layouts.expect("admitted flat carry")[&ty.name].clone())
                };
                (name.clone(), ty, fields)
            })
            .collect();
        Self { bindings }
    }

    pub(super) fn needs_struct(&self) -> bool {
        self.bindings.len() > 1 || self.bindings.iter().any(|(_, _, fields)| fields.is_some())
    }

    fn words(&self) -> Vec<NirExpr> {
        self.bindings
            .iter()
            .flat_map(|(name, ty, fields)| {
                let base = NirExpr::Var(name.clone());
                if let Some(fields) = fields {
                    fields
                        .iter()
                        .map(|field| NirExpr::FieldAccess {
                            base: Box::new(base.clone()),
                            field: field.clone(),
                        })
                        .collect()
                } else if ty == &scalar_type("bool") {
                    vec![NirExpr::CastBoolToI64(Box::new(base))]
                } else {
                    vec![base]
                }
            })
            .collect()
    }
}

pub(super) fn value(plan: &Plan, ty: Option<&NirTypeRef>) -> NirExpr {
    // Private branch transport stays flat-i64. Rebuild nominal records at the
    // join instead of mutating their storage or changing old snapshot bindings.
    let words = plan.words();
    if let Some(ty) = ty {
        NirExpr::StructLiteral {
            type_name: ty.name.clone(),
            type_args: vec![],
            fields: words
                .into_iter()
                .enumerate()
                .map(|(index, word)| (format!("carry{index}"), word))
                .collect(),
        }
    } else {
        assert!(words.len() <= 1);
        words.into_iter().next().unwrap_or(NirExpr::Int(0))
    }
}

pub(super) fn state_type(
    plan: &Plan,
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
        fields: (0..plan.words().len())
            .map(|index| NirStructField {
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
    plan: &Plan,
    call: NirExpr,
) -> Vec<NirStmt> {
    let mut body = vec![NirStmt::Let {
        name: temporary.clone(),
        ty: Some(ty.clone()),
        value: call,
    }];
    let mut index = 0;
    let mut word = || {
        let value = NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var(temporary.clone())),
            field: format!("carry{index}"),
        };
        index += 1;
        value
    };
    for (name, ty, fields) in &plan.bindings {
        let value = if let Some(fields) = fields {
            NirExpr::StructLiteral {
                type_name: ty.name.clone(),
                type_args: vec![],
                fields: fields.iter().map(|field| (field.clone(), word())).collect(),
            }
        } else {
            decode(ty, word())
        };
        body.push(NirStmt::Let {
            name: name.clone(),
            ty: Some(ty.clone()),
            value,
        });
    }
    body
}

pub(super) fn binding(name: &str, scope: &Scope, word: NirExpr) -> NirStmt {
    let ty = scope[name].clone();
    let value = decode(&ty, word);
    NirStmt::Let {
        name: name.to_owned(),
        ty: Some(ty),
        value,
    }
}

fn decode(ty: &NirTypeRef, word: NirExpr) -> NirExpr {
    if ty == &scalar_type("bool") {
        NirExpr::CastI64ToBool(Box::new(word))
    } else {
        assert_eq!(ty, &scalar_type("i64"));
        word
    }
}
