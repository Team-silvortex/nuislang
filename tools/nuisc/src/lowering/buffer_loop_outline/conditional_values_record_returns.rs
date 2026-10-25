use super::*;

#[path = "record_return_plan.rs"]
mod plan;

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    pub(super) fn direct_record_returns(
        &mut self,
        body: &mut Vec<NirStmt>,
        scope: &Scope,
        result: Option<&NirTypeRef>,
    ) -> bool {
        let Some(result) = result else {
            return false;
        };
        let Some(plan) = plan::prove(body, scope, result, self.catalog, self.layouts) else {
            return false;
        };
        let rewritten = self.record_return_body(&plan, scope);
        *body = rewritten;
        true
    }

    fn record_return_body(&mut self, plan: &[plan::Step<'_>], scope: &Scope) -> Vec<NirStmt> {
        let mut inner = scope.clone();
        let mut body = Vec::with_capacity(plan.len());
        for step in plan {
            body.push(match step {
                plan::Step::Binding { stmt, ty } => {
                    let mut stmt = (*stmt).clone();
                    let (name, value) = match &mut stmt {
                        NirStmt::Let {
                            name,
                            ty: declared,
                            value,
                        } => {
                            *declared = Some(ty.clone());
                            (name, value)
                        }
                        NirStmt::Const { name, value, .. } => (name, value),
                        _ => unreachable!(),
                    };
                    *value = if ty == &scalar_type("bool") {
                        self.gated_condition(value, &inner)
                            .unwrap_or_else(|| value.clone())
                    } else {
                        self.record_field_value(value, &inner)
                    };
                    inner.insert(name.clone(), ty.clone());
                    stmt
                }
                plan::Step::Print(stmt) => (*stmt).clone(),
                plan::Step::Join(join) => self.joined_declaration(join, &mut inner, true),
                plan::Step::Branch { condition, yes, no } => NirStmt::If {
                    condition: self
                        .gated_condition(condition, &inner)
                        .unwrap_or_else(|| (*condition).clone()),
                    then_body: self.record_return_body(yes, &inner),
                    else_body: self.record_return_body(no, &inner),
                },
                plan::Step::Return(value) => {
                    NirStmt::Return(Some(self.record_field_value(value, &inner)))
                }
            });
        }
        body
    }
}
