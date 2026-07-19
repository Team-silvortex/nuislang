use super::*;

#[test]
fn owned_select_tree_checks_non_null_only_on_selected_call_leaf() {
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
            "non_null",
            "value",
            "scratch",
            "owner",
            "1",
        ],
    );

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
    selected
        .values
        .insert("scratch".to_owned(), Value::Pointer(Some(17)));
    assert_eq!(
        cpu.execute(&node, &resource, &mut selected).unwrap(),
        Value::OwnedBytes(vec![1, 2])
    );

    selected
        .values
        .insert("scratch".to_owned(), Value::Pointer(None));
    assert!(cpu
        .execute(&node, &resource, &mut selected)
        .unwrap_err()
        .contains("failed non-null Buffer proof"));

    selected
        .values
        .insert("choose_call".to_owned(), Value::Bool(false));
    assert_eq!(
        cpu.execute(&node, &resource, &mut selected).unwrap(),
        Value::OwnedBytes(vec![7])
    );
}

#[test]
fn owned_select_tree_checks_traversal_borrow_only_on_selected_call_leaf() {
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
            "traversal_borrow",
            "value",
            "head",
            "owner",
            "1",
        ],
    );

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
    state
        .values
        .insert("head".to_owned(), Value::Pointer(Some(17)));
    assert_eq!(
        cpu.execute(&node, &resource, &mut state).unwrap(),
        Value::OwnedBytes(vec![1, 2])
    );

    state.values.insert("head".to_owned(), Value::Pointer(None));
    assert!(cpu
        .execute(&node, &resource, &mut state)
        .unwrap_err()
        .contains("null traversal pointer borrow"));

    state
        .values
        .insert("choose_call".to_owned(), Value::Bool(false));
    assert_eq!(
        cpu.execute(&node, &resource, &mut state).unwrap(),
        Value::OwnedBytes(vec![7])
    );
}
