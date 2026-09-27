use super::*;

pub(super) fn fields(
    ty: &NirTypeRef,
    definitions: &BTreeMap<&str, &NirStructDef>,
) -> Option<Vec<(String, NirTypeRef)>> {
    let definition = definitions.get(ty.name.as_str())?;
    if ty.is_ref
        || ty.is_optional
        || !ty.generic_args.is_empty()
        || !definition.generic_params.is_empty()
        || !definition.where_bounds.is_empty()
        || definition.fields.is_empty()
        || definition.fields.iter().any(|field| {
            !is_scalar_i64(&field.ty)
                && !is_bool(&field.ty)
                && !is_i32(&field.ty)
                && !is_f32(&field.ty)
                && !is_f64(&field.ty)
        })
    {
        return None;
    }
    Some(
        definition
            .fields
            .iter()
            .map(|field| (field.name.clone(), field.ty.clone()))
            .collect(),
    )
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

pub(super) fn source_word(binding: &str, field: &str, ty: &NirTypeRef) -> NirExpr {
    let value = NirExpr::FieldAccess {
        base: Box::new(NirExpr::Var(binding.to_owned())),
        field: field.to_owned(),
    };
    encode(value, ty)
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
            .all(|(index, (((name, value), word), (field, ty)))| {
                name == word
                    && word == &format!("carry{index}")
                    && value == &source_word(binding.name, field, ty)
            })
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
