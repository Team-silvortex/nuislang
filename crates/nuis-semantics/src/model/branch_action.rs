use super::NirExpr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NirBranchEffectAction<'a> {
    pub module: &'static str,
    pub instruction: &'static str,
    pub operands: Vec<&'a NirExpr>,
}

impl NirExpr {
    pub fn branch_effect_action(&self) -> Option<NirBranchEffectAction<'_>> {
        let (module, instruction, operands) = match self {
            Self::LoadValue(pointer) => ("cpu", "load_value", vec![pointer.as_ref()]),
            Self::Free(pointer) => ("cpu", "free", vec![pointer.as_ref()]),
            _ => return None,
        };
        Some(NirBranchEffectAction {
            module,
            instruction,
            operands,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_registered_keys_without_lowering_metadata() {
        let expr = NirExpr::LoadValue(Box::new(NirExpr::Var("head".to_owned())));
        let action = expr.branch_effect_action().expect("load branch action");
        assert_eq!((action.module, action.instruction), ("cpu", "load_value"));
        assert!(matches!(action.operands.as_slice(), [NirExpr::Var(name)] if name == "head"));
        assert!(NirExpr::Int(1).branch_effect_action().is_none());
    }
}
