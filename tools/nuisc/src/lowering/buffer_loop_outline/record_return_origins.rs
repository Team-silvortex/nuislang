use super::*;
use scalar_joins::nested_prefix::{Arm, Join, Local};
use std::borrow::Cow;

pub(super) fn value(value: &NirExpr, certified: &BTreeSet<String>) -> bool {
    control_values::has_logical_record_fields(value)
        || matches!(value, NirExpr::Var(origin) if certified.contains(origin))
}

// Consume only a complete typed source join after whole-function preflight.
pub(super) fn join(join: &Join<'_>, result: &NirTypeRef, certified: &BTreeSet<String>) -> bool {
    join.yes.ty == result
        && join.no.ty == result
        && arm(&join.yes, result, certified)
        && arm(&join.no, result, certified)
}

fn arm(arm: &Arm<'_>, result: &NirTypeRef, certified: &BTreeSet<String>) -> bool {
    let mut certified = certified.clone();
    for local in &arm.locals {
        match local {
            Local::Binding { stmt, ty } => {
                let (NirStmt::Let {
                    name,
                    value: source,
                    ..
                }
                | NirStmt::Const {
                    name,
                    value: source,
                    ..
                }) = stmt
                else {
                    unreachable!()
                };
                if ty == result && value(source, &certified) {
                    certified.insert(name.clone());
                }
            }
            Local::Join {
                join: child,
                export,
            } => {
                if *export && join(child, result, &certified) {
                    certified.insert(child.yes.name.to_owned());
                }
            }
            Local::Condition(_) => {}
        }
    }
    // Synthetic defaults retain discarded work, never source provenance.
    let Cow::Borrowed(source) = &arm.value else {
        return false;
    };
    value(source, &certified)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_join_rejects_synthetic_or_unrelated_arms_without_a_union_shortcut() {
        let ty = scalar_type("R");
        let condition = NirExpr::Bool(true);
        let literal = NirExpr::StructLiteral {
            type_name: "R".into(),
            type_args: vec![],
            fields: vec![(
                "tag".into(),
                NirExpr::Binary {
                    op: NirBinaryOp::And,
                    lhs: Box::new(NirExpr::Bool(true)),
                    rhs: Box::new(NirExpr::Bool(false)),
                },
            )],
        };
        let unrelated = NirExpr::Var("parameter".into());
        let borrowed = || Arm {
            name: "result",
            ty: &ty,
            value: Cow::Borrowed(&literal),
            locals: vec![],
            inputs: BTreeSet::new(),
        };
        let mut plan = Join {
            condition: &condition,
            yes: borrowed(),
            no: borrowed(),
        };
        assert!(join(&plan, &ty, &BTreeSet::new()));
        for reversed in [false, true] {
            plan.no.value = Cow::Owned(literal.clone());
            if reversed {
                std::mem::swap(&mut plan.yes, &mut plan.no);
            }
            assert!(!join(&plan, &ty, &BTreeSet::new()));
            plan.yes = borrowed();
            plan.no = borrowed();
        }
        plan.no.value = Cow::Borrowed(&unrelated);
        assert!(!join(&plan, &ty, &BTreeSet::new()));
    }
}
