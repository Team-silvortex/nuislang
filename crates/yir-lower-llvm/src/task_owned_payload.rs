use super::{fresh_reg, LlvmLoweringState, LlvmValueRef, StructLlvmValueRef};

const OWNED_DESCRIPTOR_SIZE: usize = 48;
const OWNED_AGGREGATE_HEADER_SIZE: usize = 24;
const OWNED_AGGREGATE_SLOT_SIZE: usize = 16;

pub(crate) fn emit_owned_struct_spawn(
    value: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    let leaf_count = scalar_leaf_count(&LlvmValueRef::Struct(value.clone()))?;
    let data = emit_owned_struct_data(value, state)?;

    let descriptor = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {descriptor} = call ptr @malloc(i64 {OWNED_DESCRIPTOR_SIZE})"
    ));
    store_descriptor_field(&descriptor, 0, "ptr", &data, state);
    store_descriptor_field(
        &descriptor,
        8,
        "i64",
        &owned_aggregate_size(leaf_count).to_string(),
        state,
    );
    store_descriptor_field(&descriptor, 16, "i64", "8", state);
    store_descriptor_field(
        &descriptor,
        24,
        "i64",
        &stable_struct_type_id(value).to_string(),
        state,
    );
    store_descriptor_field(&descriptor, 32, "ptr", "null", state);
    store_descriptor_field(
        &descriptor,
        40,
        "ptr",
        "@nuis_scheduler_owned_aggregate_drop_v1",
        state,
    );
    let handle = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {handle} = call i64 @nuis_scheduler_task_spawn_owned_v1(ptr {descriptor})"
    ));
    state
        .body
        .push(format!("  call void @free(ptr {descriptor})"));
    Some(handle)
}

pub(crate) fn emit_owned_struct_data(
    value: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    let leaf_count = scalar_leaf_count(&LlvmValueRef::Struct(value.clone()))?;
    let data = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {data} = call ptr @nuis_scheduler_owned_aggregate_alloc_v1(i64 {leaf_count})"
    ));
    let mut leaf_index = 0;
    pack_value(
        &LlvmValueRef::Struct(value.clone()),
        &data,
        &mut leaf_index,
        stable_struct_type_id(value),
        state,
    )?;
    let finalized = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {finalized} = call ptr @nuis_scheduler_owned_aggregate_finish_v1(ptr {data})"
    ));
    Some(finalized)
}

pub(crate) fn emit_owned_struct_invoker_spawn(
    callee: &str,
    context: &str,
    template: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    let leaf_count = scalar_leaf_count(&LlvmValueRef::Struct(template.clone()))?;
    let handle = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {handle} = call i64 @nuis_scheduler_task_spawn_owned_invoker_v1(ptr @nuis_task_invoker_{callee}, ptr {context}, i64 {}, i64 8, i64 {}, ptr @nuis_scheduler_owned_aggregate_drop_v1)",
        owned_aggregate_size(leaf_count),
        stable_struct_type_id(template)
    ));
    Some(handle)
}

pub(crate) fn emit_owned_struct_take(
    handle: &str,
    template: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<StructLlvmValueRef> {
    scalar_leaf_count(&LlvmValueRef::Struct(template.clone()))?;
    let descriptor = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {descriptor} = call ptr @malloc(i64 {OWNED_DESCRIPTOR_SIZE})"
    ));
    let taken = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {taken} = call i64 @nuis_scheduler_task_take_owned_v1(i64 {handle}, ptr {descriptor})"
    ));
    let data_slot = byte_offset(&descriptor, 0, state);
    let data = fresh_reg(&mut state.next_reg);
    state
        .body
        .push(format!("  {data} = load ptr, ptr {data_slot}, align 8"));
    let mut leaf_index = 0;
    let value = unpack_value(
        &LlvmValueRef::Struct(template.clone()),
        &data,
        &mut leaf_index,
        state,
    )?;
    state.body.push(format!(
        "  call void @nuis_scheduler_owned_payload_drop_v1(ptr {descriptor})"
    ));
    state
        .body
        .push(format!("  call void @free(ptr {descriptor})"));
    let LlvmValueRef::Struct(value) = value else {
        unreachable!("owned struct template must unpack as a struct")
    };
    Some(value)
}

fn scalar_leaf_count(value: &LlvmValueRef) -> Option<usize> {
    match value {
        value if is_scalar(value) => Some(1),
        LlvmValueRef::Struct(value) if !value.fields.is_empty() => {
            value.fields.iter().try_fold(0usize, |count, (_, field)| {
                count.checked_add(scalar_leaf_count(field)?)
            })
        }
        _ => None,
    }
}

fn pack_value(
    value: &LlvmValueRef,
    data: &str,
    leaf_index: &mut usize,
    glm_token_base: u64,
    state: &mut LlvmLoweringState,
) -> Option<()> {
    if is_scalar(value) {
        let packed = pack_scalar(value, state)?;
        if matches!(value, LlvmValueRef::TextHandle { .. }) {
            let blob = fresh_reg(&mut state.next_reg);
            let glm_token = glm_token_base.wrapping_add(*leaf_index as u64).max(1);
            state.body.push(format!(
                "  {blob} = call ptr @nuis_scheduler_owned_blob_copy_text_v1(i64 {packed}, i64 {glm_token})"
            ));
            let stored = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {stored} = call i64 @nuis_scheduler_owned_aggregate_set_blob_v1(ptr {data}, i64 {leaf_index}, ptr {blob})"
            ));
        } else {
            let stored = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {stored} = call i64 @nuis_scheduler_owned_aggregate_set_scalar_v1(ptr {data}, i64 {leaf_index}, i64 {packed})"
            ));
        }
        *leaf_index += 1;
        return Some(());
    }
    let LlvmValueRef::Struct(value) = value else {
        return None;
    };
    for (_, field) in &value.fields {
        pack_value(field, data, leaf_index, glm_token_base, state)?;
    }
    Some(())
}

fn unpack_value(
    template: &LlvmValueRef,
    data: &str,
    leaf_index: &mut usize,
    state: &mut LlvmLoweringState,
) -> Option<LlvmValueRef> {
    if is_scalar(template) {
        let packed = fresh_reg(&mut state.next_reg);
        state.body.push(format!(
            "  {packed} = call i64 @nuis_scheduler_owned_aggregate_get_v1(ptr {data}, i64 {leaf_index})"
        ));
        *leaf_index += 1;
        return unpack_scalar(&packed, template, state);
    }
    let LlvmValueRef::Struct(template) = template else {
        return None;
    };
    let fields = template
        .fields
        .iter()
        .map(|(name, field)| Some((name.clone(), unpack_value(field, data, leaf_index, state)?)))
        .collect::<Option<Vec<_>>>()?;
    Some(LlvmValueRef::Struct(StructLlvmValueRef {
        type_name: template.type_name.clone(),
        fields,
    }))
}

fn is_scalar(value: &LlvmValueRef) -> bool {
    matches!(
        value,
        LlvmValueRef::Bool { .. }
            | LlvmValueRef::I32(_)
            | LlvmValueRef::I64(_)
            | LlvmValueRef::F32(_)
            | LlvmValueRef::F64(_)
            | LlvmValueRef::TextHandle { .. }
    )
}

fn pack_scalar(value: &LlvmValueRef, state: &mut LlvmLoweringState) -> Option<String> {
    let (instruction, source) = match value {
        LlvmValueRef::Bool { i1, .. } => ("zext i1", i1.as_str()),
        LlvmValueRef::I32(value) => ("sext i32", value.as_str()),
        LlvmValueRef::I64(value) => return Some(value.clone()),
        LlvmValueRef::F32(value) => {
            let bits = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {bits} = bitcast float {value} to i32"));
            let packed = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {packed} = zext i32 {bits} to i64"));
            return Some(packed);
        }
        LlvmValueRef::F64(value) => {
            let packed = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {packed} = bitcast double {value} to i64"));
            return Some(packed);
        }
        LlvmValueRef::TextHandle { handle, .. } => return Some(handle.clone()),
        _ => return None,
    };
    let packed = fresh_reg(&mut state.next_reg);
    state
        .body
        .push(format!("  {packed} = {instruction} {source} to i64"));
    Some(packed)
}

fn unpack_scalar(
    packed: &str,
    template: &LlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<LlvmValueRef> {
    match template {
        LlvmValueRef::Bool { .. } => {
            let i1 = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {i1} = trunc i64 {packed} to i1"));
            Some(LlvmValueRef::Bool {
                i1,
                i64: packed.to_owned(),
            })
        }
        LlvmValueRef::I32(_) => {
            let value = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {value} = trunc i64 {packed} to i32"));
            Some(LlvmValueRef::I32(value))
        }
        LlvmValueRef::I64(_) => Some(LlvmValueRef::I64(packed.to_owned())),
        LlvmValueRef::F32(_) => {
            let bits = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {bits} = trunc i64 {packed} to i32"));
            let value = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {value} = bitcast i32 {bits} to float"));
            Some(LlvmValueRef::F32(value))
        }
        LlvmValueRef::F64(_) => {
            let value = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {value} = bitcast i64 {packed} to double"));
            Some(LlvmValueRef::F64(value))
        }
        LlvmValueRef::TextHandle { .. } => {
            let blob = fresh_reg(&mut state.next_reg);
            state
                .body
                .push(format!("  {blob} = inttoptr i64 {packed} to ptr"));
            let handle = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {handle} = call i64 @nuis_scheduler_owned_blob_text_lift_v1(ptr {blob})"
            ));
            let ptr = fresh_reg(&mut state.next_reg);
            state.body.push(format!(
                "  {ptr} = call ptr @nuis_host_text_ptr(i64 {handle})"
            ));
            Some(LlvmValueRef::TextHandle { ptr, handle })
        }
        _ => None,
    }
}

fn store_descriptor_field(
    descriptor: &str,
    offset: usize,
    ty: &str,
    value: &str,
    state: &mut LlvmLoweringState,
) {
    let field = byte_offset(descriptor, offset, state);
    state
        .body
        .push(format!("  store {ty} {value}, ptr {field}, align 8"));
}

fn byte_offset(base: &str, offset: usize, state: &mut LlvmLoweringState) -> String {
    if offset == 0 {
        return base.to_owned();
    }
    let pointer = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {pointer} = getelementptr i8, ptr {base}, i64 {offset}"
    ));
    pointer
}

fn stable_struct_type_id(value: &StructLlvmValueRef) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    let mut shape = Vec::new();
    append_shape(&LlvmValueRef::Struct(value.clone()), &mut shape);
    for byte in shape {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash & i64::MAX as u64).max(1)
}

fn append_shape(value: &LlvmValueRef, out: &mut Vec<u8>) {
    match value {
        LlvmValueRef::Struct(value) => {
            out.push(7);
            out.extend_from_slice(value.type_name.as_bytes());
            out.push(0);
            for (name, field) in &value.fields {
                out.extend_from_slice(name.as_bytes());
                out.push(0);
                append_shape(field, out);
            }
            out.push(8);
        }
        scalar => out.push(scalar_tag(scalar)),
    }
}

fn scalar_tag(value: &LlvmValueRef) -> u8 {
    match value {
        LlvmValueRef::Bool { .. } => 1,
        LlvmValueRef::I32(_) => 2,
        LlvmValueRef::I64(_) => 3,
        LlvmValueRef::F32(_) => 4,
        LlvmValueRef::F64(_) => 5,
        LlvmValueRef::TextHandle { .. } => 6,
        _ => 0,
    }
}

fn owned_aggregate_size(leaf_count: usize) -> usize {
    OWNED_AGGREGATE_HEADER_SIZE + leaf_count * OWNED_AGGREGATE_SLOT_SIZE
}
