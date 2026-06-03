use std::collections::BTreeMap;

use nuis_semantics::model::{AstExpr, NirExpr, NirStructDef, NirTypeRef};

use super::super::{lower_expr, named_type, FunctionSignature, ModuleConstValue};

#[allow(clippy::too_many_arguments)]
pub(super) fn lower_nova_view_accessor_builtin_call(
    callee: &str,
    args: &[AstExpr],
    current_domain: &str,
    _current_function_is_async: bool,
    bindings: &BTreeMap<String, NirTypeRef>,
    _module_consts: &BTreeMap<String, ModuleConstValue>,
    signatures: &BTreeMap<String, FunctionSignature>,
    struct_table: &BTreeMap<String, NirStructDef>,
) -> Result<Option<NirExpr>, String> {
    let Some((expected_type, field_name)) = view_state_accessor_target(callee) else {
        return Ok(None);
    };
    let [state] = args else {
        return Err(format!("{callee}(...) expects 1 arg"));
    };
    let state = lower_expr(
        state,
        current_domain,
        bindings,
        signatures,
        struct_table,
        Some(&named_type(expected_type)),
    )?;
    Ok(Some(NirExpr::FieldAccess {
        base: Box::new(state),
        field: field_name.to_owned(),
    }))
}

fn view_state_accessor_target(callee: &str) -> Option<(&'static str, &'static str)> {
    Some(match callee {
        "nova_tabs_state_active" => ("NovaTabsState", "active"),
        "nova_tabs_state_compact" => ("NovaTabsState", "compact"),
        "nova_list_state_dense" => ("NovaListState", "dense"),
        "nova_list_state_selected" => ("NovaListState", "selected"),
        "nova_table_state_zebra" => ("NovaTableState", "zebra"),
        "nova_table_state_selected_row" => ("NovaTableState", "selected_row"),
        "nova_tree_state_expanded" => ("NovaTreeState", "expanded"),
        "nova_tree_state_selected" => ("NovaTreeState", "selected"),
        "nova_inspector_state_pinned" => ("NovaInspectorState", "pinned"),
        "nova_inspector_state_selected" => ("NovaInspectorState", "selected"),
        "nova_outline_state_collapsed" => ("NovaOutlineState", "collapsed"),
        "nova_outline_state_selected" => ("NovaOutlineState", "selected"),
        _ => return None,
    })
}
