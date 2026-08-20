use crate::frontend::is_host_execution_domain;
use nuis_semantics::model::{AstExpr, NirExpr};

use crate::frontend::{i64_type, lower_expr, ref_type};

use super::DirectCallLoweringContext;

pub(super) fn lower_object_call(
    callee: &str,
    args: &[AstExpr],
    context: DirectCallLoweringContext<'_>,
) -> Result<Option<NirExpr>, String> {
    if !matches!(callee, "owned_object_size" | "owned_object_read_i64") {
        return Ok(None);
    }
    if !is_host_execution_domain(context.current_domain) {
        return Err(format!(
            "{callee}(...) requires a host execution module (`mod cpu` or `mod cffi`)"
        ));
    }
    match (callee, args) {
        ("owned_object_size", [object]) => {
            let object = lower_object(object, context)?;
            Ok(Some(NirExpr::OwnedObjectSize(Box::new(object))))
        }
        ("owned_object_read_i64", [object, index]) => {
            let object = lower_object(object, context)?;
            let index = lower_expr(
                index,
                context.current_domain,
                context.bindings,
                context.signatures,
                context.struct_table,
                Some(&i64_type()),
            )?;
            Ok(Some(NirExpr::OwnedObjectReadI64 {
                object: Box::new(object),
                index: Box::new(index),
            }))
        }
        ("owned_object_size", _) => Err("owned_object_size(...) expects 1 arg".to_owned()),
        ("owned_object_read_i64", _) => Err("owned_object_read_i64(...) expects 2 args".to_owned()),
        _ => unreachable!(),
    }
}

fn lower_object(
    object: &AstExpr,
    context: DirectCallLoweringContext<'_>,
) -> Result<NirExpr, String> {
    lower_expr(
        object,
        context.current_domain,
        context.bindings,
        context.signatures,
        context.struct_table,
        Some(&ref_type("FfiObject")),
    )
}
