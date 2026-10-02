use super::*;

#[path = "return_signal.rs"]
mod return_signal;

pub(super) fn return_owned(effects: &[NirStmt], signal: &returns::Signal) -> bool {
    return_signal::can_share(effects, signal.name())
}

#[derive(Default)]
pub(in crate::lowering::buffer_loop_outline) struct Boundary {
    pub before: Vec<NirStmt>,
    pub after: Vec<NirStmt>,
}

pub(super) struct Plan {
    pub scope: Scope,
    pub effects: Vec<NirStmt>,
    pub carries: Vec<String>,
    pub breaking: Option<String>,
    pub boundary: Boundary,
}

pub(super) fn prepare(
    iteration: &induction::Iteration<'_>,
    scope: &Scope,
    names: &mut BTreeSet<String>,
    recover_index: bool,
    return_signal: Option<&returns::Signal>,
) -> Plan {
    let shared = return_signal.filter(|signal| {
        scope.get(signal.name()) == Some(&scalar_type("i64"))
            && return_signal::can_share(iteration.effects, signal.name())
    });
    let normalized = if let Some(signal) = shared {
        control_flow::normalize_return_exit(
            iteration.effects,
            scope,
            (!iteration.leading).then_some(iteration.step),
            signal.name(),
        )
    } else {
        iteration.normalize(scope)
    }
    .expect("admitted counted value exits");
    let (mut effects, breaking) = match normalized {
        Some(flow) => (flow.effects, flow.breaking),
        None => (iteration.effects.to_vec(), None),
    };
    let mut carries = sequences::carry_names(&effects)
        .into_iter()
        .filter(|name| scope.contains_key(name) && Some(name) != breaking.as_ref())
        .collect::<Vec<_>>();
    let mut scope = scope.clone();
    let mut boundary = Boundary::default();
    if let Some(flag) = &breaking {
        if iteration.leading && recover_index {
            let induction = &iteration.prepared.binding_name;
            names.extend(scope.keys().cloned());
            branches::collect_bindings(&effects, names);
            let advanced = branches::fresh_name("__nuis_advanced_index", names);
            // Only a leading source step needs recovery when the shared driver
            // suppresses its tail on break, and only if a continuation can
            // observe that index. Zero trips retain the entry seed.
            boundary.before.push(copy(&advanced, induction));
            effects.insert(0, copy(&advanced, induction));
            boundary.after.push(copy(induction, &advanced));
            scope.insert(advanced.clone(), scalar_type("i64"));
            carries.push(advanced);
        }
        scope.insert(flag.clone(), scalar_type("i64"));
        // Existing scoped-break admission requires the canonical signal last.
        carries.push(flag.clone());
    }
    Plan {
        scope,
        effects,
        carries,
        breaking,
        boundary,
    }
}

fn copy(name: &str, source: &str) -> NirStmt {
    NirStmt::Let {
        name: name.to_owned(),
        ty: Some(scalar_type("i64")),
        value: NirExpr::Var(source.to_owned()),
    }
}

pub(super) fn guard(flag: String) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Binary {
            op: NirBinaryOp::Eq,
            lhs: Box::new(NirExpr::Var(flag)),
            rhs: Box::new(NirExpr::Int(1)),
        },
        then_body: vec![NirStmt::Break],
        else_body: vec![],
    }
}
