use super::*;

#[path = "effectful_selections_transport_i32.rs"]
mod i32_pair;

#[path = "effectful_selections_transport_owned.rs"]
mod owned_pair;

pub(super) fn numeric(ty: &NirTypeRef) -> bool {
    ["i32", "i64", "f32", "f64"]
        .into_iter()
        .any(|name| ty == &scalar_type(name))
}

// Admission is narrower than the independently typed owned-field mechanism.
pub(super) fn numeric_pair(owner: &NirTypeRef, child: &NirTypeRef) -> bool {
    let i64_owner = owner == &scalar_type("i64")
        && ["i64", "f32", "f64"]
            .into_iter()
            .any(|name| child == &scalar_type(name));
    let float_owner = ["f32", "f64"]
        .into_iter()
        .any(|name| owner == &scalar_type(name))
        && child == &scalar_type("i64");
    i64_owner || float_owner
}

// A closed compiler-private carrier, not a source tuple or an exported result.
pub(super) struct Selection {
    plan: Plan,
    owner: String,
    child: String,
    i32_child: bool,
    i32_owner: bool,
    owned_fields: Option<(NirTypeRef, NirTypeRef)>,
}

fn binary(op: NirBinaryOp, lhs: NirExpr, rhs: NirExpr) -> NirExpr {
    NirExpr::Binary {
        op,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    }
}

fn pack(owner: &str, child: &str) -> NirExpr {
    binary(
        NirBinaryOp::Add,
        NirExpr::CastBoolToI64(Box::new(NirExpr::Var(owner.into()))),
        binary(
            NirBinaryOp::Mul,
            NirExpr::CastBoolToI64(Box::new(NirExpr::Var(child.into()))),
            NirExpr::Int(2),
        ),
    )
}

pub(super) fn prepare(
    condition: predicates::Predicate,
    yes: regions::Staging,
    no: regions::Staging,
    scope: &Scope,
    owner: &str,
    child: &str,
) -> Option<(Selection, BTreeSet<String>, BTreeSet<String>)> {
    let child_type = scope.get(child)?;
    let owner_type = scope.get(owner)?;
    let i32_child = child_type == &scalar_type("i32");
    let i32_owner =
        scope.get(owner) == Some(&scalar_type("i32")) && child_type == &scalar_type("bool");
    let bool_owner = owner_type == &scalar_type("bool");
    let wide_owner = ["i64", "f32", "f64"]
        .into_iter()
        .any(|name| owner_type == &scalar_type(name))
        && child_type == &scalar_type("bool");
    let wide_child = bool_owner
        && ["i64", "f32", "f64"]
            .into_iter()
            .any(|name| child_type == &scalar_type(name));
    let numeric_pair = numeric_pair(owner_type, child_type);
    let owned_fields = (wide_owner || wide_child || numeric_pair)
        .then(|| (owner_type.clone(), child_type.clone()));
    if owner == child
        || !(bool_owner || i32_owner || wide_owner || numeric_pair)
        || (!i32_child && owned_fields.is_none() && child_type != &scalar_type("bool"))
    {
        return None;
    }
    let mut captures = BTreeSet::new();
    let mut calls = BTreeSet::new();
    for arm in [&yes, &no] {
        captures.extend(arm.inputs.iter().cloned());
        captures.extend(
            [owner, child]
                .into_iter()
                .filter(|n| !arm.defined.contains(*n))
                .map(str::to_owned),
        );
        calls.extend(arm.calls.iter().cloned());
    }
    if captures.len() > 31 {
        return None;
    }
    let params = captures
        .iter()
        .map(|name| {
            let ty = scope.get(name)?;
            scalar(ty).then(|| NirParam {
                name: name.clone(),
                ty: ty.clone(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let value = |arm: regions::Staging| {
        SelectedValue::Region(
            arm.statements,
            if owned_fields.is_some() {
                // Installed as a typed two-field result, never scalar packing.
                NirExpr::Int(0)
            } else if i32_owner {
                // Reuse the closed codec with semantic roles reversed.
                i32_pair::pack(child, owner)
            } else if i32_child {
                i32_pair::pack(owner, child)
            } else {
                pack(owner, child)
            },
        )
    };
    Some((
        Selection {
            plan: Plan {
                condition,
                yes: value(yes),
                no: value(no),
                destination: None,
                ty: scalar_type("i64"),
                params,
            },
            owner: owner.into(),
            child: child.into(),
            i32_child,
            i32_owner,
            owned_fields,
        },
        captures,
        calls,
    ))
}

pub(super) fn install(
    selection: Selection,
    names: &mut BTreeSet<String>,
    bindings: &mut BTreeSet<String>,
    helpers: &mut Vec<NirFunction>,
    structs: &mut Vec<NirStructDef>,
) -> Vec<NirStmt> {
    let Selection {
        mut plan,
        owner,
        child,
        i32_child,
        i32_owner,
        owned_fields,
    } = selection;
    if let Some(fields) = owned_fields {
        return owned_pair::install(
            plan, owner, child, fields, names, bindings, helpers, structs,
        );
    }
    let carrier = branches::fresh_name("__nuis_effect_call_pair", bindings);
    plan.destination = Some((carrier.clone(), false));
    let mut output = super::install(plan, names, bindings, helpers, structs).0;
    if i32_owner {
        output.extend(i32_pair::restore(carrier, child, owner));
        return output;
    }
    if i32_child {
        output.extend(i32_pair::restore(carrier, owner, child));
        return output;
    }
    // Bool casts are canonical 0/1, so the carrier is exactly in 0..=3.
    // Decode with comparisons only: no generated division or trapping work.
    let equal = |n| {
        binary(
            NirBinaryOp::Eq,
            NirExpr::Var(carrier.clone()),
            NirExpr::Int(n),
        )
    };
    output.push(NirStmt::Let {
        name: owner,
        ty: Some(scalar_type("bool")),
        value: binary(NirBinaryOp::Xor, equal(1), equal(3)),
    });
    output.push(NirStmt::Let {
        name: child,
        ty: Some(scalar_type("bool")),
        value: binary(NirBinaryOp::Ge, NirExpr::Var(carrier), NirExpr::Int(2)),
    });
    output
}
