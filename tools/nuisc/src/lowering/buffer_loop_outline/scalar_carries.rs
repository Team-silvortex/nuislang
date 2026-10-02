use super::*;
use crate::lowering::scalar_record_shape::{self, Shape};
use nuis_semantics::model::NirStructField;

pub(super) struct Plan {
    bindings: Vec<(String, NirTypeRef, Option<Shape>)>,
}

impl Plan {
    pub(super) fn new(
        carries: &[String],
        scope: &Scope,
        layouts: Option<&control_values::CarryLayouts>,
    ) -> Self {
        let bindings = carries
            .iter()
            .map(|name| {
                let ty = scope[name].clone();
                let fields = if ty == scalar_type("i64")
                    || ty == scalar_type("bool")
                    || ty == scalar_type("i32")
                    || ty == scalar_type("f32")
                    || ty == scalar_type("f64")
                {
                    None
                } else {
                    assert_eq!(ty, scalar_type(&ty.name));
                    Some(
                        Shape::from_layouts(&ty, layouts.expect("admitted record carry"))
                            .expect("validated pure record shape"),
                    )
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
                if let Some(shape) = fields {
                    shape
                        .leaves()
                        .iter()
                        .map(|(path, ty)| encode(ty, scalar_record_shape::source_value(name, path)))
                        .collect()
                } else {
                    vec![encode(ty, base)]
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
    body.extend(projections(&temporary, plan));
    body
}

fn projections(temporary: &str, plan: &Plan) -> Vec<NirStmt> {
    let mut body = Vec::new();
    let mut index = 0;
    let mut word = || {
        let value = NirExpr::FieldAccess {
            base: Box::new(NirExpr::Var(temporary.to_owned())),
            field: format!("carry{index}"),
        };
        index += 1;
        value
    };
    for (name, ty, fields) in &plan.bindings {
        let value = if let Some(shape) = fields {
            shape.reconstruct(&mut |ty| decode(ty, word()))
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

pub(super) fn record_input(
    param: &mut NirParam,
    scope: &Scope,
    layouts: &control_values::CarryLayouts,
    names: &mut BTreeSet<String>,
    structs: &mut Vec<NirStructDef>,
    bindings: &mut BTreeSet<String>,
    seeds: &mut Vec<NirStmt>,
) -> Option<NirExpr> {
    if !layouts
        .get(&param.ty.name)?
        .iter()
        .any(|(_, ty)| ty != &scalar_type("i64"))
    {
        return None;
    }
    let plan = Plan::new(std::slice::from_ref(&param.name), scope, Some(layouts));
    let ty = state_type(&plan, names, structs);
    let word = branches::fresh_name("__nuis_record_seed", bindings);
    seeds.extend(projections(&word, &plan));
    param.name = word;
    param.ty = ty.clone();
    Some(value(&plan, Some(&ty)))
}

pub(super) fn encode(ty: &NirTypeRef, value: NirExpr) -> NirExpr {
    if ty == &scalar_type("bool") {
        NirExpr::CastBoolToI64(Box::new(value))
    } else if ty == &scalar_type("i32") {
        NirExpr::CastI32ToI64(Box::new(value))
    } else if ty == &scalar_type("f32") {
        NirExpr::PackF32Word(Box::new(value))
    } else if ty == &scalar_type("f64") {
        NirExpr::PackF64Word(Box::new(value))
    } else {
        assert_eq!(ty, &scalar_type("i64"));
        value
    }
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

pub(super) fn decode(ty: &NirTypeRef, word: NirExpr) -> NirExpr {
    if ty == &scalar_type("bool") {
        NirExpr::CastI64ToBool(Box::new(word))
    } else if ty == &scalar_type("i32") {
        NirExpr::CastI64ToI32(Box::new(word))
    } else if ty == &scalar_type("f32") {
        NirExpr::UnpackF32Word(Box::new(word))
    } else if ty == &scalar_type("f64") {
        NirExpr::UnpackF64Word(Box::new(word))
    } else {
        assert_eq!(ty, &scalar_type("i64"));
        word
    }
}
