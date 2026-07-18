use super::*;
use yir_core::{Operation, ResourceKind, StructValue};

fn cpu_resource() -> Resource {
    Resource {
        name: "cpu0".to_owned(),
        kind: ResourceKind::parse("cpu.main"),
    }
}

fn cpu_node(name: &str, instruction: &str, args: Vec<&str>) -> Node {
    Node {
        name: name.to_owned(),
        resource: "cpu0".to_owned(),
        op: Operation::parse(
            instruction,
            args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
        )
        .unwrap(),
    }
}

#[test]
fn copy_buffer_owned_is_independent_from_source_mutation() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("len".to_owned(), Value::Int(3));
    state.values.insert("fill".to_owned(), Value::Int(7));

    let buffer = cpu
        .execute(
            &cpu_node("buffer", "cpu.alloc_buffer", vec!["len", "fill"]),
            &resource,
            &mut state,
        )
        .expect("buffer allocation");
    state.values.insert("buffer".to_owned(), buffer);
    let bytes = cpu
        .execute(
            &cpu_node("bytes", "cpu.copy_buffer_owned", vec!["buffer"]),
            &resource,
            &mut state,
        )
        .expect("owned Buffer copy");

    let pointer = state.expect_pointer("buffer").expect("buffer pointer");
    state
        .write_heap_buffer_at(pointer, 1, 99)
        .expect("mutate source buffer");
    assert_eq!(bytes, Value::OwnedBytes(vec![7, 7, 7]));

    state.values.insert("bytes".to_owned(), bytes);
    let byte_len = cpu
        .execute(
            &cpu_node("byte_len", "cpu.owned_bytes_len", vec!["bytes"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes length");
    assert_eq!(byte_len, Value::Int(24));
    let dropped = cpu
        .execute(
            &cpu_node("dropped", "cpu.drop_owned_bytes", vec!["bytes"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes drop");
    assert_eq!(dropped, Value::Unit);
}

#[test]
fn execute_variant_is_and_variant_field_on_enum_structs() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert(
        "result".to_owned(),
        Value::Struct(StructValue {
            type_name: "Result.Ok".to_owned(),
            fields: vec![("value".to_owned(), Value::Int(42))],
        }),
    );

    let is_ok = cpu
        .execute(
            &cpu_node("is_ok", "cpu.variant_is", vec!["result", "Result.Ok"]),
            &resource,
            &mut state,
        )
        .expect("variant_is should execute");
    assert_eq!(is_ok, Value::Bool(true));

    let payload = cpu
        .execute(
            &cpu_node(
                "payload",
                "cpu.variant_field",
                vec!["result", "Result.Ok", "value"],
            ),
            &resource,
            &mut state,
        )
        .expect("variant_field should execute");
    assert_eq!(payload, Value::Int(42));

    let wrong_variant = cpu
        .execute(
            &cpu_node(
                "wrong_payload",
                "cpu.variant_field",
                vec!["result", "Result.Err", "value"],
            ),
            &resource,
            &mut state,
        )
        .expect_err("wrong variant access should fail");
    assert!(wrong_variant.contains("expects variant `Result.Err`"));
}

#[test]
fn execute_select_between_enum_variants_preserves_union_payloads() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("cond".to_owned(), Value::Bool(true));
    state.values.insert(
        "ok".to_owned(),
        Value::Struct(StructValue {
            type_name: "Result.Ok".to_owned(),
            fields: vec![("value".to_owned(), Value::Int(7))],
        }),
    );
    state.values.insert(
        "err".to_owned(),
        Value::Struct(StructValue {
            type_name: "Result.Err".to_owned(),
            fields: vec![("value".to_owned(), Value::Int(99))],
        }),
    );

    let selected = cpu
        .execute(
            &cpu_node("selected", "cpu.select", vec!["cond", "ok", "err"]),
            &resource,
            &mut state,
        )
        .expect("select should execute");
    state.values.insert("selected".to_owned(), selected);

    let is_ok = cpu
        .execute(
            &cpu_node("is_ok", "cpu.variant_is", vec!["selected", "Result.Ok"]),
            &resource,
            &mut state,
        )
        .expect("variant_is should execute");
    assert_eq!(is_ok, Value::Bool(true));

    let ok_payload = cpu
        .execute(
            &cpu_node(
                "ok_payload",
                "cpu.variant_field",
                vec!["selected", "Result.Ok", "value"],
            ),
            &resource,
            &mut state,
        )
        .expect("selected Ok payload should stay available");
    assert_eq!(ok_payload, Value::Int(7));

    let err_payload = cpu
        .execute(
            &cpu_node(
                "err_payload",
                "cpu.variant_field",
                vec!["selected", "Result.Err", "value"],
            ),
            &resource,
            &mut state,
        )
        .expect("non-active Err payload should stay available for guarded select lowering");
    assert_eq!(err_payload, Value::Int(99));
}

#[test]
fn parse_carry_branch_source_accepts_keep_prev_carry_kind() {
    let args = vec!["keep_prev_carry".to_owned()];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(source.kind, "keep_prev_carry");
    assert!(source.payload.is_empty());
    assert_eq!(next, 1);
}

#[test]
fn parse_conditional_carries_accepts_keep_prev_carry_branch_kind() {
    let args = vec![
        "acc0".to_owned(),
        "prev_current_gt".to_owned(),
        "rhs0".to_owned(),
        "add_prev_current".to_owned(),
        "keep_prev_carry".to_owned(),
    ];
    let carries = parse_conditional_carries(&args, 0, "loop_node", true).expect("expected carries");
    assert_eq!(carries.len(), 1);
    assert_eq!(carries[0].else_source.kind, "keep_prev_carry");
    assert!(carries[0].else_source.payload.is_empty());
}

#[test]
fn parse_carry_branch_source_accepts_add_current_plus_invariant_kind() {
    let args = vec!["add_current_plus_invariant".to_owned(), "rhs0".to_owned()];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(source.kind, "add_current_plus_invariant");
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_conditional_carries_accepts_add_invariant_branch_kind() {
    let args = vec![
        "acc0".to_owned(),
        "current_gt".to_owned(),
        "rhs0".to_owned(),
        "add_current_plus_invariant".to_owned(),
        "rhs1".to_owned(),
        "add_invariant".to_owned(),
        "rhs2".to_owned(),
    ];
    let carries = parse_conditional_carries(&args, 0, "loop_node", true).expect("expected carries");
    assert_eq!(carries.len(), 1);
    assert_eq!(carries[0].then_source.kind, "add_current_plus_invariant");
    assert_eq!(carries[0].then_source.payload, vec!["rhs1".to_owned()]);
    assert_eq!(carries[0].else_source.kind, "add_invariant");
    assert_eq!(carries[0].else_source.payload, vec!["rhs2".to_owned()]);
}

#[test]
fn parse_carry_branch_source_accepts_add_current_plus_current_plus_invariant_kind() {
    let args = vec![
        "add_current_plus_current_plus_invariant".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(source.kind, "add_current_plus_current_plus_invariant");
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_current_plus_current_plus_current_plus_invariant_kind() {
    let args = vec![
        "add_current_plus_current_plus_current_plus_invariant".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_current_plus_current_plus_current_plus_invariant"
    );
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_current_plus_current_plus_invariant_kind() {
    let args = vec![
        "add_scaled_current_plus_current_plus_invariant".to_owned(),
        "factor0".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_current_plus_current_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec!["factor0".to_owned(), "rhs0".to_owned()]
    );
    assert_eq!(next, 3);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_current_kind() {
    let args = vec!["add_scaled_by_current_current_plus_current".to_owned()];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(source.kind, "add_scaled_by_current_current_plus_current");
    assert!(source.payload.is_empty());
    assert_eq!(next, 1);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_current_plus_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_current_plus_current_plus_invariant".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_current_plus_current_plus_invariant"
    );
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_current_plus_factor_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_factor_invariant_current_plus_current".to_owned(),
        "factor_rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_factor_invariant_current_plus_current"
    );
    assert_eq!(source.payload, vec!["factor_rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_current_plus_factor_invariant_plus_invariant_kind(
) {
    let args = vec![
        "add_scaled_by_current_plus_factor_invariant_current_plus_current_plus_invariant"
            .to_owned(),
        "factor_rhs0".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_factor_invariant_current_plus_current_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec!["factor_rhs0".to_owned(), "rhs0".to_owned()]
    );
    assert_eq!(next, 3);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_multi_state_factor_kind() {
    let args = vec!["add_scaled_by_current_plus_current_times_current_plus_current".to_owned()];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_times_current_plus_current"
    );
    assert!(source.payload.is_empty());
    assert_eq!(next, 1);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_multi_state_factor_plus_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_times_current_plus_current_plus_invariant".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_times_current_plus_current_plus_invariant"
    );
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_multi_state_factor_and_factor_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_current_plus_current"
            .to_owned(),
        "factor_rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_current_plus_current"
    );
    assert_eq!(source.payload, vec!["factor_rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_factor_group_product_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_terms_current_plus_current"
            .to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_terms_current_plus_current"
    );
    assert_eq!(source.payload, vec!["rhs0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_factor_group_product_plus_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_factor_group_current_plus_factor_invariant_times_terms_current_plus_current_plus_invariant"
            .to_owned(),
        "lhs0".to_owned(),
        "rhs0".to_owned(),
        "base0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_factor_group_current_plus_factor_invariant_times_terms_current_plus_current_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec!["lhs0".to_owned(), "rhs0".to_owned(), "base0".to_owned()]
    );
    assert_eq!(next, 4);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_factor_group_product_times_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_factor_invariant_times_terms_current_plus_current"
            .to_owned(),
        "factor_scale0".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_times_factor_group_current_plus_factor_invariant_times_factor_invariant_times_terms_current_plus_current"
    );
    assert_eq!(
        source.payload,
        vec!["factor_scale0".to_owned(), "rhs0".to_owned()]
    );
    assert_eq!(next, 3);
}

#[test]
fn parse_carry_branch_source_accepts_mul_scaled_additive_kind() {
    let args = vec![
        "mul_scaled_current_plus_current_plus_invariant".to_owned(),
        "factor0".to_owned(),
        "offset0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "mul_scaled_current_plus_current_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec!["factor0".to_owned(), "offset0".to_owned()]
    );
    assert_eq!(next, 3);
}

#[test]
fn parse_carry_branch_source_accepts_mul_scaled_by_state_additive_kind() {
    let args = vec![
        "mul_scaled_by_current_current_plus_carry0_plus_invariant".to_owned(),
        "offset0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "mul_scaled_by_current_current_plus_carry0_plus_invariant"
    );
    assert_eq!(source.payload, vec!["offset0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_mul_scaled_by_state_plus_invariant_additive_kind() {
    let args = vec![
        "mul_scaled_by_current_plus_factor_invariant_current_plus_carry0_plus_invariant".to_owned(),
        "factor0".to_owned(),
        "offset0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "mul_scaled_by_current_plus_factor_invariant_current_plus_carry0_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec!["factor0".to_owned(), "offset0".to_owned()]
    );
    assert_eq!(next, 3);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_multi_state_factor_times_invariant_kind() {
    let args = vec![
        "add_scaled_by_current_plus_current_times_factor_invariant_times_current_plus_current"
            .to_owned(),
        "factor_scale0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_times_factor_invariant_times_current_plus_current"
    );
    assert_eq!(source.payload, vec!["factor_scale0".to_owned()]);
    assert_eq!(next, 2);
}

#[test]
fn parse_carry_branch_source_accepts_add_scaled_by_multi_state_factor_and_offset_times_invariant_kind(
) {
    let args = vec![
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_factor_invariant_times_current_plus_current_plus_invariant"
            .to_owned(),
        "factor_scale0".to_owned(),
        "factor_rhs0".to_owned(),
        "rhs0".to_owned(),
    ];
    let (source, next) =
        parse_carry_branch_source(&args, 0, "loop_node").expect("expected branch source");
    assert_eq!(
        source.kind,
        "add_scaled_by_current_plus_current_plus_factor_invariant_times_factor_invariant_times_current_plus_current_plus_invariant"
    );
    assert_eq!(
        source.payload,
        vec![
            "factor_scale0".to_owned(),
            "factor_rhs0".to_owned(),
            "rhs0".to_owned()
        ]
    );
    assert_eq!(next, 4);
}
