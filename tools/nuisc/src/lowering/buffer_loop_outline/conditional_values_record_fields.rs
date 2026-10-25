use super::*;

impl<L: control_values::ValueLayouts> Builder<'_, L> {
    pub(super) fn record_field_value(&mut self, value: &NirExpr, scope: &Scope) -> NirExpr {
        let NirExpr::StructLiteral {
            type_name,
            type_args,
            fields,
        } = value
        else {
            return value.clone();
        };
        let mut lowered = Vec::with_capacity(fields.len());
        for (name, value) in fields {
            let boolean = self
                .layouts
                .fields(type_name)
                .and_then(|mut fields| fields.find(|(field, _)| *field == name))
                .is_some_and(|(_, ty)| ty == scalar_type("bool"));
            let value = if boolean
                && matches!(
                    value,
                    NirExpr::Binary {
                        op: NirBinaryOp::And | NirBinaryOp::Or,
                        ..
                    }
                ) {
                self.field_logical_root(value, scope)
            } else {
                self.record_field_value(value, scope)
            };
            // Keep each helper call at its original field, after preceding work.
            lowered.push((name.clone(), value));
        }
        NirExpr::StructLiteral {
            type_name: type_name.clone(),
            type_args: type_args.clone(),
            fields: lowered,
        }
    }
}
