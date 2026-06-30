use super::*;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn infer_nir_expr_address_class(
    expr: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    address_classes: &BTreeMap<String, NirAddressClass>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Option<NirAddressClass> {
    let ty = infer_nir_expr_type(expr, bindings, signatures, struct_table)?;
    if !ty.is_address_type() {
        return None;
    }

    match expr {
        NirExpr::Var(name) => address_classes.get(name).copied(),
        NirExpr::Borrow(_) => Some(NirAddressClass::Borrowed),
        NirExpr::Move(inner) => {
            infer_nir_expr_address_class(inner, bindings, address_classes, signatures, struct_table)
        }
        NirExpr::AllocNode { .. } | NirExpr::AllocBuffer { .. } => Some(NirAddressClass::Owned),
        NirExpr::LoadNext(inner) => {
            infer_nir_expr_address_class(inner, bindings, address_classes, signatures, struct_table)
        }
        _ => None,
    }
}
