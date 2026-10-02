use super::*;

pub(super) fn fields(
    ty: &NirTypeRef,
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> Option<Vec<(Vec<String>, NirTypeRef)>> {
    let shape = Shape::from_definitions(ty, definitions)?;
    if shape.fields.is_empty() {
        return None;
    }
    Some(shape.leaves())
}

pub(super) fn projected(value: &NirExpr, ty: &NirTypeRef, result: &str, slot: usize) -> bool {
    if is_bool(ty) {
        matches!(value, NirExpr::CastI64ToBool(word) if projected_word(word, result, slot))
    } else if is_i32(ty) {
        matches!(value, NirExpr::CastI64ToI32(word) if projected_word(word, result, slot))
    } else if is_f32(ty) {
        matches!(value, NirExpr::UnpackF32Word(word) if projected_word(word, result, slot))
    } else if is_f64(ty) {
        matches!(value, NirExpr::UnpackF64Word(word) if projected_word(word, result, slot))
    } else {
        projected_word(value, result, slot)
    }
}

#[cfg(test)]
pub(super) fn source_word(binding: &str, field: &str, ty: &NirTypeRef) -> NirExpr {
    source_path_word(binding, &[field.to_owned()], ty)
}

pub(super) fn source_path_word(binding: &str, path: &[String], ty: &NirTypeRef) -> NirExpr {
    encode(
        crate::lowering::scalar_record_shape::source_value(binding, path),
        ty,
    )
}

pub(super) fn encode(value: NirExpr, ty: &NirTypeRef) -> NirExpr {
    if is_bool(ty) {
        NirExpr::CastBoolToI64(Box::new(value))
    } else if is_i32(ty) {
        NirExpr::CastI32ToI64(Box::new(value))
    } else if is_f32(ty) {
        NirExpr::PackF32Word(Box::new(value))
    } else if is_f64(ty) {
        NirExpr::PackF64Word(Box::new(value))
    } else {
        value
    }
}

pub(super) fn seed(
    binding: &Projection<'_>,
    param: &NirParam,
    arg: &NirExpr,
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> bool {
    let NirExpr::StructLiteral {
        type_name,
        type_args,
        fields,
    } = arg
    else {
        return false;
    };
    let Some(words) = flat_fields(&param.ty, definitions) else {
        return false;
    };
    // Match the complete typed source-to-word map, not a generated name prefix.
    type_name == &param.ty.name
        && type_args.is_empty()
        && words.len() == binding.width()
        && fields.len() == words.len()
        && fields
            .iter()
            .zip(&words)
            .zip(&binding.fields)
            .enumerate()
            .all(|(index, (((name, value), word), (path, ty)))| {
                name == word
                    && word == &format!("carry{index}")
                    && value == &source_path_word(binding.name, path, ty)
            })
}

pub(super) fn rebuild(
    shape: &Shape,
    words: &mut impl Iterator<Item = String>,
    state: &mut LoweringState<'_>,
) -> String {
    if shape.fields.is_empty() {
        return words.next().expect("validated leaf count");
    }
    let fields = shape
        .fields
        .iter()
        .map(|(field, child)| (field, rebuild(child, words, state)))
        .collect::<Vec<_>>();
    let name = next_name(state, "loop_value_result");
    let mut args = vec![shape.ty.name.clone()];
    args.extend(
        fields
            .iter()
            .map(|(field, value)| format!("{field}={value}")),
    );
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: "struct".to_owned(),
            args,
        },
    });
    for (_, value) in fields {
        push_dep_edges(state, &value, &name);
    }
    name
}

pub(super) fn decode(word: &str, ty: &NirTypeRef, state: &mut LoweringState<'_>) -> String {
    let (prefix, instruction) = if is_bool(ty) {
        ("loop_bool_result", "cast_i64_to_bool")
    } else if is_i32(ty) {
        ("loop_i32_result", "cast_i64_to_i32")
    } else if is_f32(ty) {
        ("loop_f32_result", "unpack_f32_word")
    } else {
        assert!(is_f64(ty));
        ("loop_f64_result", "unpack_f64_word")
    };
    let name = next_name(state, prefix);
    state.yir.nodes.push(Node {
        name: name.clone(),
        resource: "cpu0".to_owned(),
        op: Operation {
            module: "cpu".to_owned(),
            instruction: instruction.to_owned(),
            args: vec![word.to_owned()],
        },
    });
    push_dep_edges(state, word, &name);
    name
}
