use super::*;

pub(in crate::lowering::buffer_loop_outline) fn preserve(module: &NirModule) -> bool {
    let Some(entry) = module
        .functions
        .iter()
        .find(|function| function.name == "main")
    else {
        return false;
    };
    // The ordinary entry cannot be a registered native-session callback. Preserve
    // its established flow path, including inlineable helper arms, only when the
    // complete entry body can keep that path without scoped branch normalization.
    if !entry
        .body
        .iter()
        .any(|stmt| matches!(stmt, NirStmt::While { .. }))
        || entry.body.iter().any(|stmt| {
            !matches!(
                stmt,
                NirStmt::Let { .. }
                    | NirStmt::Const { .. }
                    | NirStmt::While { .. }
                    | NirStmt::Return(_)
            )
        })
    {
        return false;
    }
    let helpers = collect_pure_helper_functions(module);
    let inline = collect_inlineable_pure_helper_exprs(module);
    let blocks = collect_pure_helper_blocks(module);
    entry.body.iter().all(|stmt| match stmt {
        NirStmt::While { condition, body } => {
            prepare_counted_while(condition, body, &helpers, &inline, &blocks).is_some()
                || (control_flow::contains_exit(body, false)
                    && (prepare_flow_while(condition, body, &helpers, &inline, &blocks).is_some()
                        || prepare_post_flow_while(condition, body, &helpers, &inline, &blocks)
                            .is_some()))
        }
        _ => true,
    })
}
