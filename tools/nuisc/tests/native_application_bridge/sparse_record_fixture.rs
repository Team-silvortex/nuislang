pub fn source(width: usize) -> String {
    let source =
        include_str!("../control_flow_syntax_native/scoped_sparse_typed_record_carries.ns");
    with_width(source, width)
}

pub fn branch_source(width: usize) -> String {
    with_width(
        include_str!("../control_flow_syntax_native/scoped_branch_typed_record_carries.ns"),
        width,
    )
}

pub fn join_source(width: usize) -> String {
    with_width(
        include_str!("../control_flow_syntax_native/scoped_join_typed_record_carries.ns"),
        width,
    )
}

pub fn loop_source(width: usize) -> String {
    with_width(
        include_str!("../control_flow_syntax_native/scoped_loop_typed_record_carries.ns"),
        width,
    )
}

pub fn return_source(width: usize) -> String {
    with_width(
        include_str!("../control_flow_syntax_native/scoped_return_typed_record_carries.ns"),
        width,
    )
}

pub fn child_return_source(width: usize) -> String {
    with_width(
        include_str!("../control_flow_syntax_native/scoped_return_child_exits.ns"),
        width,
    )
}

pub fn joined_return_source(width: usize) -> String {
    return_source(width)
        .replace(
            "let limit = state.count;",
            "let bounds = state;
             if state.count % 2 == 0 { let bounds = carry; }
             else { let saved = carry; let bounds = saved; }
             let limit = bounds.count;",
        )
        .replace(
            "tag: selected.tag, enabled: !enabled",
            "tag: i32_from_i64(17), enabled: !enabled",
        )
}

pub fn parent_return_source(width: usize) -> String {
    return_source(width).replace(
        "tag: previous.tag, enabled: previous.enabled",
        "tag: state.left.tag, enabled: previous.enabled",
    )
}

fn with_width(source: &str, width: usize) -> String {
    assert!((9..=64).contains(&width));
    if width == 9 {
        return source.into();
    }
    let extra = |value: &str| {
        (9..width)
            .map(|i| format!(", extra{i}: {value}"))
            .collect::<String>()
    };
    source
        .replace("count: i64", &format!("count: i64{}", extra("i64")))
        .replacen("count: limit", &format!("count: limit{}", extra("27")), 1)
        .replace("count: limit\n", &format!("count: limit{}\n", extra("99")))
}
