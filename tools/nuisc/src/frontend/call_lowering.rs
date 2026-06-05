use std::collections::BTreeMap;

use nuis_semantics::model::{AstBinaryOp, AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::metadata::hidden_private_field_count;
use super::{
    infer_nir_expr_type, lower_binary_expr_with_async, lower_direct_call_builtin_or_named_call,
    lower_expr_with_async, lower_routed_call_or_core_builtin, resolve_declared_or_inferred,
    FunctionSignature, ModuleConstValue,
};

#[allow(dead_code)]
pub(super) fn lower_binary_expr(
    op: &AstBinaryOp,
    lhs: &AstExpr,
    rhs: &AstExpr,
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<NirExpr, String> {
    lower_binary_expr_with_async(
        op,
        lhs,
        rhs,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
    )
}

#[allow(dead_code)]
pub(super) fn lower_call_expr(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    bindings: &BTreeMap<String, NirTypeRef>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
) -> Result<NirExpr, String> {
    lower_call_expr_with_async(
        callee,
        args,
        current_domain,
        false,
        bindings,
        &BTreeMap::new(),
        signatures,
        struct_table,
        expected,
        false,
    )
}

pub(super) fn lower_call_expr_with_async(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    expected: Option<&NirTypeRef>,
    allow_async_calls: bool,
) -> Result<NirExpr, String> {
    if let Some(payload_struct_constructor) = lower_payload_struct_constructor_sugar(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        allow_async_calls,
    )? {
        return Ok(payload_struct_constructor);
    }
    if let Some(routed_or_core) = lower_routed_call_or_core_builtin(
        callee,
        args,
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        expected,
    )? {
        return Ok(routed_or_core);
    }
    match callee {
        _ => lower_direct_call_builtin_or_named_call(
            callee,
            args,
            current_domain,
            current_function_is_async,
            bindings,
            module_consts,
            signatures,
            struct_table,
            allow_async_calls,
        )?
        .ok_or_else(|| format!("unknown function `{callee}`")),
    }
}

#[allow(clippy::too_many_arguments)]
fn lower_payload_struct_constructor_sugar(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
    allow_async_calls: bool,
) -> Result<Option<NirExpr>, String> {
    if signatures.contains_key(callee) {
        return Ok(None);
    }
    let Some(definition) = struct_table.get(callee) else {
        return Ok(None);
    };
    if definition.fields.len() != 1 {
        return Err(format!(
            "payload-style struct constructor `{callee}(...)` requires struct `{callee}` to have exactly one field"
        ));
    }
    let hidden_private_fields = hidden_private_field_count(definition);
    if hidden_private_fields > 0 {
        return Err(format!(
            "struct literal `{}` cannot be constructed outside its defining module because it hides {} private field(s)",
            callee, hidden_private_fields
        ));
    }
    if args.len() != 1 {
        return Err(format!(
            "payload-style struct constructor `{callee}(...)` expects exactly 1 arg"
        ));
    }
    let field = &definition.fields[0];
    let lowered = lower_expr_with_async(
        &args[0],
        current_domain,
        current_function_is_async,
        bindings,
        module_consts,
        signatures,
        struct_table,
        Some(&field.ty),
        allow_async_calls,
    )?;
    let inferred = infer_nir_expr_type(&lowered, bindings, signatures, struct_table);
    let _ = resolve_declared_or_inferred(&field.name, Some(field.ty.clone()), inferred)?;
    Ok(Some(NirExpr::StructLiteral {
        type_name: callee.to_owned(),
        fields: vec![(field.name.clone(), lowered)],
    }))
}
