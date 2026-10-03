use super::*;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Literal {
    I64(i64),
    I32(i32),
    Bool(bool),
    F32(u32),
    F64(u64),
}

impl Literal {
    fn ty(&self) -> NirTypeRef {
        scalar_type(match self {
            Self::I64(_) => "i64",
            Self::I32(_) => "i32",
            Self::Bool(_) => "bool",
            Self::F32(_) => "f32",
            Self::F64(_) => "f64",
        })
    }
}

impl Proof<'_> {
    pub(super) fn literal(&mut self, literal: Literal) -> Option<Rc<Value>> {
        let ty = literal.ty();
        if self.shape(&ty)?.leaves.len() != 1 {
            return None;
        }
        self.budget.charge(1)?;
        Some(Rc::new(Value {
            ty,
            words: vec![Some(Origin::Literal(literal))],
        }))
    }

    pub(super) fn float_literal(&mut self, expr: &NirExpr) -> Option<Rc<Value>> {
        // Equality is typed storage identity, not floating-point comparison.
        // Charge text before parsing; invalid/nonfinite text grants no origin.
        let (literal, kind) = match expr {
            NirExpr::F32(text) => {
                self.budget.charge(text.len())?;
                (
                    text.parse::<f32>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .map(|v| Literal::F32(v.to_bits())),
                    "f32",
                )
            }
            NirExpr::F64(text) => {
                self.budget.charge(text.len())?;
                (
                    text.parse::<f64>()
                        .ok()
                        .filter(|v| v.is_finite())
                        .map(|v| Literal::F64(v.to_bits())),
                    "f64",
                )
            }
            _ => return None,
        };
        match literal {
            Some(literal) => self.literal(literal),
            None => self.unknown(scalar_type(kind)),
        }
    }

    pub(super) fn integer_cast(
        &mut self,
        expr: &NirExpr,
        env: &Env,
        depth: usize,
    ) -> Option<Rc<Value>> {
        let (operand, from, to) = match expr {
            NirExpr::CastI64ToI32(operand) => (operand, "i64", "i32"),
            NirExpr::CastI32ToI64(operand) => (operand, "i32", "i64"),
            _ => return None,
        };
        let value = self.expression(operand, env, depth)?;
        if value.ty != scalar_type(from) || value.words.len() != 1 {
            return None;
        }
        let literal = match (&value.words[0], to) {
            (Some(Origin::Literal(Literal::I64(value))), "i32") => Literal::I32(*value as i32),
            (Some(Origin::Literal(Literal::I32(value))), "i64") => Literal::I64(i64::from(*value)),
            _ => return self.unknown(scalar_type(to)),
        };
        self.literal(literal)
    }
}
