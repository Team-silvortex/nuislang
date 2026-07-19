use super::*;
use yir_core::{Operation, ResourceKind, StructValue, VariantUnionValue};

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
    let moved = cpu
        .execute(
            &cpu_node("moved", "cpu.move_owned_bytes", vec!["bytes"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes move");
    assert_eq!(moved, Value::OwnedBytes(vec![7, 7, 7]));
    state.values.insert("moved".to_owned(), moved);
    let returned = cpu
        .execute(
            &cpu_node("returned", "cpu.return_owned_bytes", vec!["moved"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes return");
    assert_eq!(returned, Value::OwnedBytes(vec![7, 7, 7]));
    state.values.insert("returned".to_owned(), returned);
    let byte_len = cpu
        .execute(
            &cpu_node("byte_len", "cpu.owned_bytes_len", vec!["returned"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes length");
    assert_eq!(byte_len, Value::Int(24));
    let dropped = cpu
        .execute(
            &cpu_node("dropped", "cpu.drop_owned_bytes", vec!["returned"]),
            &resource,
            &mut state,
        )
        .expect("owned bytes drop");
    assert_eq!(dropped, Value::Unit);
}

#[test]
fn select_owned_bytes_executes_as_a_typed_resource_choice() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("cond".to_owned(), Value::Bool(false));
    state
        .values
        .insert("then_bytes".to_owned(), Value::OwnedBytes(vec![1, 2]));
    state
        .values
        .insert("else_bytes".to_owned(), Value::OwnedBytes(vec![7, 8, 9]));

    let selected = cpu
        .execute(
            &cpu_node(
                "selected",
                "cpu.select_owned_bytes",
                vec!["cond", "then_bytes", "else_bytes"],
            ),
            &resource,
            &mut state,
        )
        .expect("owned bytes select should execute");
    assert_eq!(selected, Value::OwnedBytes(vec![7, 8, 9]));

    state.values.insert("not_bytes".to_owned(), Value::Int(1));
    let error = cpu
        .execute(
            &cpu_node(
                "invalid",
                "cpu.select_owned_bytes",
                vec!["cond", "then_bytes", "not_bytes"],
            ),
            &resource,
            &mut state,
        )
        .expect_err("owned bytes select must reject scalar candidates");
    assert!(error.contains("expects owned bytes in both select branches"));
}

#[test]
fn distinct_owned_bytes_select_executes_as_a_typed_resource_choice() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("cond".to_owned(), Value::Bool(true));
    state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7, 8, 9]));

    let selected = cpu
        .execute(
            &cpu_node(
                "selected",
                "cpu.select_owned_bytes_drop_unselected",
                vec!["cond", "left", "right"],
            ),
            &resource,
            &mut state,
        )
        .expect("owned bytes cleanup select should execute");
    assert_eq!(selected, Value::OwnedBytes(vec![1, 2]));
}

#[test]
fn nested_owned_select_tree_interpreter_chooses_runtime_leaf() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("outer".to_owned(), Value::Bool(true));
    state.values.insert("inner".to_owned(), Value::Bool(false));
    state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1]));
    state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7, 8]));

    let selected = cpu
        .execute(
            &cpu_node(
                "selected",
                "cpu.select_owned_bytes_tree",
                vec![
                    "2", "left", "right", "if", "outer", "if", "inner", "owner", "0", "owner", "1",
                    "owner", "0",
                ],
            ),
            &resource,
            &mut state,
        )
        .expect("nested owned select tree should execute");
    assert_eq!(selected, Value::OwnedBytes(vec![7, 8]));
}

#[test]
fn nested_owned_select_tree_interpreter_accepts_static_call_leaf() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("outer".to_owned(), Value::Bool(false));
    state.values.insert("delta".to_owned(), Value::Int(3));
    state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));

    let selected = cpu
        .execute(
            &cpu_node(
                "selected",
                "cpu.select_owned_bytes_tree",
                vec![
                    "2", "left", "right", "if", "outer", "owner", "1", "call", "keep", "0", "1",
                    "value", "delta",
                ],
            ),
            &resource,
            &mut state,
        )
        .expect("tree call leaf should execute");
    assert_eq!(selected, Value::OwnedBytes(vec![1, 2]));
}

#[test]
fn owned_select_tree_projects_variant_field_only_on_selected_call_leaf() {
    use std::collections::BTreeMap;

    let cpu = CpuMod;
    let resource = cpu_resource();
    let node = cpu_node(
        "selected",
        "cpu.select_owned_bytes_tree",
        vec![
            "2",
            "left",
            "right",
            "if",
            "choose_call",
            "call",
            "retain",
            "0",
            "1",
            "variant_field",
            "route",
            "Route.Left",
            "value",
            "owner",
            "1",
        ],
    );

    let left_variant = StructValue {
        type_name: "Route.Left".to_owned(),
        fields: vec![("value".to_owned(), Value::Int(9))],
    };
    let mut variants = BTreeMap::new();
    variants.insert("Route.Left".to_owned(), left_variant.clone());
    let mut selected_state = ExecutionState::default();
    selected_state
        .values
        .insert("choose_call".to_owned(), Value::Bool(true));
    selected_state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    selected_state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    selected_state.values.insert(
        "route".to_owned(),
        Value::VariantUnion(VariantUnionValue {
            parent_type_name: "Route".to_owned(),
            active_variant: "Route.Left".to_owned(),
            variants,
        }),
    );
    assert_eq!(
        cpu.execute(&node, &resource, &mut selected_state).unwrap(),
        Value::OwnedBytes(vec![1, 2])
    );

    let mut skipped_state = ExecutionState::default();
    skipped_state
        .values
        .insert("choose_call".to_owned(), Value::Bool(false));
    skipped_state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    skipped_state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    skipped_state.values.insert(
        "route".to_owned(),
        Value::Struct(StructValue {
            type_name: "Route.Right".to_owned(),
            fields: vec![("value".to_owned(), Value::Int(4))],
        }),
    );
    assert_eq!(
        cpu.execute(&node, &resource, &mut skipped_state).unwrap(),
        Value::OwnedBytes(vec![7])
    );
}

#[path = "tests_owned_select_non_null.rs"]
mod tests_owned_select_non_null;

#[test]
fn owned_select_tree_recursively_projects_struct_field_on_selected_leaf() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let node = cpu_node(
        "selected",
        "cpu.select_owned_bytes_tree",
        vec![
            "2",
            "left",
            "right",
            "if",
            "choose_call",
            "call",
            "retain",
            "0",
            "1",
            "struct_field",
            "score",
            "variant_field",
            "route",
            "Route.Left",
            "value",
            "owner",
            "1",
        ],
    );
    let payload = Value::Struct(StructValue {
        type_name: "Payload".to_owned(),
        fields: vec![("score".to_owned(), Value::Int(9))],
    });
    let route = Value::Struct(StructValue {
        type_name: "Route.Left".to_owned(),
        fields: vec![("value".to_owned(), payload)],
    });
    let mut state = ExecutionState::default();
    state
        .values
        .insert("choose_call".to_owned(), Value::Bool(true));
    state
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    state
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    state.values.insert("route".to_owned(), route);

    assert_eq!(
        cpu.execute(&node, &resource, &mut state).unwrap(),
        Value::OwnedBytes(vec![1, 2])
    );

    let mut skipped = ExecutionState::default();
    skipped
        .values
        .insert("choose_call".to_owned(), Value::Bool(false));
    skipped
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    skipped
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    skipped.values.insert(
        "route".to_owned(),
        Value::Struct(StructValue {
            type_name: "Route.Right".to_owned(),
            fields: Vec::new(),
        }),
    );
    assert_eq!(
        cpu.execute(&node, &resource, &mut skipped).unwrap(),
        Value::OwnedBytes(vec![7])
    );
}

#[test]
fn owned_select_tree_casts_recursive_projection_only_on_selected_leaf() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let node = cpu_node(
        "selected",
        "cpu.select_owned_bytes_tree",
        vec![
            "2",
            "left",
            "right",
            "if",
            "choose_call",
            "call",
            "retain",
            "0",
            "1",
            "cast",
            "i32_to_i64",
            "struct_field",
            "score",
            "variant_field",
            "route",
            "Route.Left",
            "value",
            "owner",
            "1",
        ],
    );
    let payload = Value::Struct(StructValue {
        type_name: "Payload".to_owned(),
        fields: vec![("score".to_owned(), Value::I32(9))],
    });
    let mut selected = ExecutionState::default();
    selected
        .values
        .insert("choose_call".to_owned(), Value::Bool(true));
    selected
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    selected
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    selected.values.insert(
        "route".to_owned(),
        Value::Struct(StructValue {
            type_name: "Route.Left".to_owned(),
            fields: vec![("value".to_owned(), payload)],
        }),
    );
    assert_eq!(
        cpu.execute(&node, &resource, &mut selected).unwrap(),
        Value::OwnedBytes(vec![1, 2])
    );

    let mut skipped = ExecutionState::default();
    skipped
        .values
        .insert("choose_call".to_owned(), Value::Bool(false));
    skipped
        .values
        .insert("left".to_owned(), Value::OwnedBytes(vec![1, 2]));
    skipped
        .values
        .insert("right".to_owned(), Value::OwnedBytes(vec![7]));
    skipped.values.insert("route".to_owned(), Value::Int(99));
    assert_eq!(
        cpu.execute(&node, &resource, &mut skipped).unwrap(),
        Value::OwnedBytes(vec![7])
    );
}

#[test]
fn branch_owned_call_interpreter_selects_one_static_helper() {
    let cpu = CpuMod;
    let resource = cpu_resource();
    let mut state = ExecutionState::default();
    state.values.insert("cond".to_owned(), Value::Bool(true));
    state
        .values
        .insert("bytes".to_owned(), Value::OwnedBytes(vec![4, 5, 6]));

    let selected = cpu
        .execute(
            &cpu_node(
                "selected",
                "cpu.branch_call_owned_bytes",
                vec!["cond", "left", "right", "bytes", "0", "0"],
            ),
            &resource,
            &mut state,
        )
        .expect("branch owned call should execute");
    assert_eq!(selected, Value::OwnedBytes(vec![4, 5, 6]));
    assert!(state
        .events
        .iter()
        .any(|event| event.contains("cpu.branch_call_owned_bytes") && event.contains("left")));
    assert!(!state.events.iter().any(|event| event.contains("right")));
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
