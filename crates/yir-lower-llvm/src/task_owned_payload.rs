use super::{fresh_reg, LlvmLoweringState, LlvmValueRef, StructLlvmValueRef};

const OWNED_DESCRIPTOR_SIZE: usize = 48;
const SCALAR_SLOT_SIZE: usize = 8;

pub(crate) fn emit_flat_struct_spawn(
    value: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    if value.fields.is_empty() || !value.fields.iter().all(|(_, field)| is_scalar(field)) {
        return None;
    }
    let data = emit_flat_struct_data(value, state)?;

    let descriptor = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {descriptor} = call ptr @malloc(i64 {OWNED_DESCRIPTOR_SIZE})"
    ));
    store_descriptor_field(&descriptor, 0, "ptr", &data, state);
    store_descriptor_field(
        &descriptor,
        8,
        "i64",
        &(value.fields.len() * SCALAR_SLOT_SIZE).to_string(),
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
        "@nuis_scheduler_payload_free_v1",
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

pub(crate) fn emit_flat_struct_data(
    value: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    if value.fields.is_empty() || !value.fields.iter().all(|(_, field)| is_scalar(field)) {
        return None;
    }
    let data = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {data} = call ptr @malloc(i64 {})",
        value.fields.len() * SCALAR_SLOT_SIZE
    ));
    for (index, (_, field)) in value.fields.iter().enumerate() {
        let slot = byte_offset(&data, index * SCALAR_SLOT_SIZE, state);
        let packed = pack_scalar(field, state)?;
        state
            .body
            .push(format!("  store i64 {packed}, ptr {slot}, align 8"));
    }
    Some(data)
}

pub(crate) fn emit_flat_struct_invoker_spawn(
    callee: &str,
    context: &str,
    template: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<String> {
    if template.fields.is_empty() || !template.fields.iter().all(|(_, field)| is_scalar(field)) {
        return None;
    }
    let handle = fresh_reg(&mut state.next_reg);
    state.body.push(format!(
        "  {handle} = call i64 @nuis_scheduler_task_spawn_owned_invoker_v1(ptr @nuis_task_invoker_{callee}, ptr {context}, i64 {}, i64 8, i64 {}, ptr @nuis_scheduler_payload_free_v1)",
        template.fields.len() * SCALAR_SLOT_SIZE,
        stable_struct_type_id(template)
    ));
    Some(handle)
}

pub(crate) fn emit_flat_struct_take(
    handle: &str,
    template: &StructLlvmValueRef,
    state: &mut LlvmLoweringState,
) -> Option<StructLlvmValueRef> {
    if template.fields.is_empty() || !template.fields.iter().all(|(_, field)| is_scalar(field)) {
        return None;
    }
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
    let mut fields = Vec::with_capacity(template.fields.len());
    for (index, (name, field)) in template.fields.iter().enumerate() {
        let slot = byte_offset(&data, index * SCALAR_SLOT_SIZE, state);
        let packed = fresh_reg(&mut state.next_reg);
        state
            .body
            .push(format!("  {packed} = load i64, ptr {slot}, align 8"));
        fields.push((name.clone(), unpack_scalar(&packed, field, state)?));
    }
    state.body.push(format!(
        "  call void @nuis_scheduler_owned_payload_drop_v1(ptr {descriptor})"
    ));
    state
        .body
        .push(format!("  call void @free(ptr {descriptor})"));
    Some(StructLlvmValueRef {
        type_name: template.type_name.clone(),
        fields,
    })
}

fn is_scalar(value: &LlvmValueRef) -> bool {
    matches!(
        value,
        LlvmValueRef::Bool { .. }
            | LlvmValueRef::I32(_)
            | LlvmValueRef::I64(_)
            | LlvmValueRef::F32(_)
            | LlvmValueRef::F64(_)
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
    for byte in value.type_name.bytes().chain(
        value
            .fields
            .iter()
            .flat_map(|(name, field)| name.bytes().chain([scalar_tag(field)])),
    ) {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    (hash & i64::MAX as u64).max(1)
}

fn scalar_tag(value: &LlvmValueRef) -> u8 {
    match value {
        LlvmValueRef::Bool { .. } => 1,
        LlvmValueRef::I32(_) => 2,
        LlvmValueRef::I64(_) => 3,
        LlvmValueRef::F32(_) => 4,
        LlvmValueRef::F64(_) => 5,
        _ => 0,
    }
}
