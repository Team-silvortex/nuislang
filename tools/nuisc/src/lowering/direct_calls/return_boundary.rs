use super::*;

pub(super) fn emit(
    function: &NirFunction,
    returned: &str,
    effect: Option<String>,
    state: &mut LoweringState<'_>,
) -> Result<(String, Option<DirectCallScalarKind>), String> {
    let owned_layout = function_owned_struct_layout(function, state);
    if let Some(node) = state.yir.nodes.iter().find(|n| n.name == returned) {
        if node.op.instruction == "tail_call_owned_struct_return" {
            if owned_layout.as_ref() != node.op.args.get(1) {
                return Err("owned continuation changes its caller result layout".into());
            }
            return Ok((returned.to_owned(), None));
        }
    }
    let name = format!("__fn_{}_return", function.name);
    let kind = if owned_layout.is_some() {
        None
    } else {
        Some(direct_call_return_kind(function).ok_or_else(|| {
            format!(
                "ordinary direct-call lowering does not support the return type in `{}`",
                function.name
            )
        })?)
    };
    let (instruction, mut args) = if let Some(layout) = owned_layout {
        ("return_owned_struct", vec![returned.to_owned(), layout])
    } else {
        let instruction = match kind.expect("non-struct return kind") {
            DirectCallScalarKind::Bool => "return_bool",
            DirectCallScalarKind::I32 => "return_i32",
            DirectCallScalarKind::I64 => "return_i64",
            DirectCallScalarKind::F32 => "return_f32",
            DirectCallScalarKind::F64 => "return_f64",
            DirectCallScalarKind::BorrowedBuffer => unreachable!("borrowed refs cannot return"),
            DirectCallScalarKind::TraversalPointer => unreachable!("traversal refs cannot return"),
            DirectCallScalarKind::OwnedBytes => "return_owned_bytes",
            DirectCallScalarKind::OwnedExternalBuffer => "return_owned_external_buffer",
        };
        (instruction, vec![returned.to_owned()])
    };
    if kind == Some(DirectCallScalarKind::OwnedExternalBuffer) {
        args.extend(owned_external_buffer_metadata_for_node(
            state.yir, returned,
        )?);
    }
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".into(),
        op: Operation {
            module: "cpu".into(),
            instruction: instruction.into(),
            args,
        },
    });
    push_dep_edges(state, returned, &name);
    if let Some(effect) = effect {
        crate::lowering::edge_helpers::push_effect_edge(state, &effect, &name);
    }
    if kind == Some(DirectCallScalarKind::OwnedExternalBuffer) {
        push_lifetime_edge(state, returned, &name);
    }
    crate::lowering::edge_helpers::push_effect_edge(state, returned, &name);
    Ok((name, kind))
}
