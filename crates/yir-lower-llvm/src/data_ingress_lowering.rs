use super::{coerce_to_i64, LlvmLoweringState, LlvmValueRef};
use yir_core::Node;

pub(crate) fn lower_data_provider_request_ingress(
    node: &Node,
    state: &mut LlvmLoweringState,
) -> bool {
    if node.op.module != "data" || node.op.instruction != "provider_request_ingress" {
        return false;
    }

    let Some(request_value) = state.registers.get(&node.op.args[0]).cloned() else {
        state.body.push(format!(
            "  ; deferred lowering for data.provider_request_ingress `{}` because its request handle is outside the current LLVM slice",
            node.name
        ));
        return true;
    };
    let Some(request_handle) = coerce_to_i64(&request_value, &mut state.body, &mut state.next_reg)
    else {
        state.body.push(format!(
            "  ; deferred lowering for data.provider_request_ingress `{}` because its request handle is not scalar",
            node.name
        ));
        return true;
    };

    state.body.push(
        "  ; Nuis-owned provider request ingress preserves descriptor/provider metadata in YIR"
            .to_owned(),
    );
    state
        .registers
        .insert(node.name.clone(), LlvmValueRef::I64(request_handle.clone()));
    state.last_cpu_value = Some(request_handle);
    true
}
