use std::collections::{BTreeMap, BTreeSet};

use nuis_semantics::model::{nir_expr_effect_class, NirExpr, NirExprEffectClass};

use crate::optimize::{simplify_expr, InlineTemplate};

pub(super) fn simplify_expr_vec(
    values: Vec<NirExpr>,
    env: &BTreeMap<String, NirExpr>,
    inline_templates: &BTreeMap<String, InlineTemplate>,
    active_inline: &mut BTreeSet<String>,
) -> (Vec<NirExpr>, bool) {
    let mut changed = false;
    let values = values
        .into_iter()
        .map(|value| {
            let (value, value_changed) = simplify_expr(value, env, inline_templates, active_inline);
            changed |= value_changed;
            value
        })
        .collect();
    (values, changed)
}

pub(super) fn is_inline_safe_arg(expr: &NirExpr) -> bool {
    matches!(
        nir_expr_effect_class(expr),
        NirExprEffectClass::Pure
            | NirExprEffectClass::LocalReadOnly
            | NirExprEffectClass::HostReadOnly
            | NirExprEffectClass::DomainReadOnly
    )
}
