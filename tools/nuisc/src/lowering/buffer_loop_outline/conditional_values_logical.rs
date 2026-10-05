use super::*;

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    pub(super) fn gated_condition(
        &mut self,
        condition: &NirExpr,
        scope: &Scope,
    ) -> Option<NirExpr> {
        if !matches!(
            condition,
            NirExpr::Binary {
                op: NirBinaryOp::And | NirBinaryOp::Or,
                ..
            }
        ) || !prefix::computed_logical_root(condition)
            || control_values::value_type(condition, scope, self.catalog, self.layouts)?
                != scalar_type("bool")
        {
            return None;
        }
        // Prove the complete original tree before allocating names or helpers.
        // Generated symbols are deliberately absent from the original catalog.
        let (value, changed) = self.logical_tree(condition, scope);
        changed.then_some(value)
    }

    fn logical_tree(&mut self, expression: &NirExpr, scope: &Scope) -> (NirExpr, bool) {
        let NirExpr::Binary {
            op: op @ (NirBinaryOp::And | NirBinaryOp::Or),
            lhs,
            rhs,
        } = expression
        else {
            return (expression.clone(), false);
        };
        let original_rhs = [NirStmt::Return(Some((**rhs).clone()))];
        let guarded = speculation::block_has_checked_arithmetic(&original_rhs, self.checked)
            || scalar_helpers::contains_calls(&original_rhs);
        let (left, left_changed) = self.logical_tree(lhs, scope);
        let (right, right_changed) = self.logical_tree(rhs, scope);
        if !guarded {
            // A total outer RHS does not excuse speculation in its nested LHS.
            return (
                NirExpr::Binary {
                    op: *op,
                    lhs: Box::new(left),
                    rhs: Box::new(right),
                },
                left_changed || right_changed,
            );
        }
        let name = branches::fresh_name("__nuis_short_circuit", &mut self.bindings);
        let mut inputs = BTreeSet::new();
        control_values::collect_inputs(rhs, &mut inputs);
        let selected = Selection {
            name: name.clone(),
            ty: scalar_type("bool"),
            constant: false,
            body: vec![NirStmt::Return(Some(right))],
            inputs,
        };
        let disjunction = *op == NirBinaryOp::Or;
        let skipped = Selection {
            name,
            ty: scalar_type("bool"),
            constant: false,
            body: vec![NirStmt::Return(Some(NirExpr::Bool(disjunction)))],
            inputs: BTreeSet::new(),
        };
        let (yes, no) = if disjunction {
            (skipped, selected)
        } else {
            (selected, skipped)
        };
        // Each original guarded edge contributes one helper. Its lowered LHS
        // is evaluated once as argument zero; its complete RHS stays selected.
        let NirStmt::Let { value, .. } = self.extract(left, yes, no, scope, false) else {
            unreachable!()
        };
        (value, true)
    }
}
