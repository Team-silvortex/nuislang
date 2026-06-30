use super::*;

pub(crate) fn infer_data_window_type(
    input: &NirExpr,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    mode: NirWindowMode,
) -> Option<NirTypeRef> {
    let inner = infer_nir_expr_type(input, bindings, signatures, struct_table)?;
    let payload = if inner.is_ref && inner.name == "Buffer" {
        i64_type()
    } else {
        inner
    };
    Some(match mode {
        NirWindowMode::Mutable => generic_named_type("WindowMut", vec![payload]),
        NirWindowMode::Immutable => generic_named_type("Window", vec![payload]),
    })
}

pub(crate) fn resolve_declared_or_inferred(
    name: &str,
    declared: Option<NirTypeRef>,
    inferred: Option<NirTypeRef>,
) -> Result<NirTypeRef, String> {
    match (declared, inferred) {
        (Some(declared), Some(inferred)) => {
            if compatible_types(&declared, &inferred) {
                Ok(declared)
            } else {
                Err(format!(
                    "binding `{name}` expected type `{}`, found `{}`",
                    render_type_name(&declared),
                    render_type_name(&inferred)
                ))
            }
        }
        (Some(declared), None) => Ok(declared),
        (None, Some(inferred)) => Ok(inferred),
        (None, None) => Err(format!(
            "binding `{name}` requires an explicit type annotation in the current minimal frontend"
        )),
    }
}
