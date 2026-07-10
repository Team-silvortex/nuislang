use std::collections::BTreeMap;

use super::{
    coerce_to_i64, loop_scalar_value_ref, CpuLoopScalarKind, LlvmValueRef, StructLlvmValueRef,
};

pub(crate) fn insert_i64_loop_chain_result(
    registers: &mut BTreeMap<String, LlvmValueRef>,
    node_name: &str,
    final_current: String,
    final_carries: Vec<String>,
    last_cpu_value: &mut Option<String>,
) {
    let mut fields = vec![(
        "current".to_owned(),
        LlvmValueRef::I64(final_current.clone()),
    )];
    for (index, final_carry) in final_carries.iter().enumerate() {
        fields.push((
            format!("carry{index}"),
            LlvmValueRef::I64(final_carry.clone()),
        ));
    }
    registers.insert(
        node_name.to_owned(),
        LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: "LoopChain".to_owned(),
            fields,
        }),
    );
    *last_cpu_value = final_carries.last().cloned().or(Some(final_current));
}

pub(crate) fn insert_scalar_loop_chain_result(
    body: &mut Vec<String>,
    next_reg: &mut usize,
    registers: &mut BTreeMap<String, LlvmValueRef>,
    node_name: &str,
    loop_scalar_kind: CpuLoopScalarKind,
    final_current: String,
    final_carries: Vec<String>,
    last_cpu_value: &mut Option<String>,
) {
    let mut fields = vec![(
        "current".to_owned(),
        loop_scalar_value_ref(loop_scalar_kind, final_current.clone()),
    )];
    for (index, final_carry) in final_carries.iter().enumerate() {
        fields.push((
            format!("carry{index}"),
            loop_scalar_value_ref(loop_scalar_kind, final_carry.clone()),
        ));
    }
    registers.insert(
        node_name.to_owned(),
        LlvmValueRef::Struct(StructLlvmValueRef {
            type_name: "LoopChain".to_owned(),
            fields,
        }),
    );
    *last_cpu_value = final_carries
        .last()
        .map(|carry| loop_scalar_value_ref(loop_scalar_kind, carry.clone()))
        .or_else(|| Some(loop_scalar_value_ref(loop_scalar_kind, final_current)))
        .and_then(|value| coerce_to_i64(&value, body, next_reg));
}
