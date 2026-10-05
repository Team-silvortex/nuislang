use super::*;

#[derive(Clone)]
enum Value {
    Bool(NirExpr),
    Int(NirExpr),
    Compared(NirExpr),
}

impl Value {
    fn atom(&self) -> Option<(&NirExpr, &str)> {
        match self {
            Self::Bool(value) => Some((value, "bool")),
            Self::Int(value) => Some((value, "i64")),
            Self::Compared(_) => None,
        }
    }
}

#[derive(Clone)]
pub(super) struct Values(BTreeMap<String, Value>);

impl Values {
    pub(super) fn from_parent(scope: &Scope) -> Self {
        let mut values = BTreeMap::new();
        for (name, ty) in scope {
            let value = NirExpr::Var(name.clone());
            if ty == &scalar_type("bool") {
                values.insert(name.clone(), Value::Bool(value));
            } else if ty == &scalar_type("i64") {
                values.insert(name.clone(), Value::Int(value));
            }
        }
        Self(values)
    }

    pub(super) fn bind(&mut self, name: &str, value: &NirExpr) {
        if let Some(value) = self.value(value) {
            self.0.insert(name.to_owned(), value);
        }
    }

    pub(super) fn boolean(&self, value: &NirExpr) -> Option<NirExpr> {
        match self.value(value)? {
            Value::Bool(value) | Value::Compared(value) => Some(value),
            Value::Int(_) => None,
        }
    }

    fn value(&self, expr: &NirExpr) -> Option<Value> {
        match expr {
            NirExpr::Bool(_) => Some(Value::Bool(expr.clone())),
            NirExpr::Int(_) => Some(Value::Int(expr.clone())),
            NirExpr::Var(name) => self.0.get(name).cloned(),
            NirExpr::Binary { op, lhs, rhs }
                if matches!(
                    op,
                    NirBinaryOp::Eq
                        | NirBinaryOp::Ne
                        | NirBinaryOp::Lt
                        | NirBinaryOp::Le
                        | NirBinaryOp::Gt
                        | NirBinaryOp::Ge
                ) =>
            {
                let left = self.value(lhs)?;
                let right = self.value(rhs)?;
                // Operands are single atoms, never another comparison. Every
                // normalized value stays at most three nodes across alias chains.
                let (lhs, left_kind) = left.atom()?;
                let (rhs, right_kind) = right.atom()?;
                if control_values::binary_type(*op, scalar_type(left_kind), scalar_type(right_kind))
                    != Some(scalar_type("bool"))
                {
                    return None;
                }
                Some(Value::Compared(NirExpr::Binary {
                    op: *op,
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(rhs.clone()),
                }))
            }
            _ => None,
        }
    }
}
