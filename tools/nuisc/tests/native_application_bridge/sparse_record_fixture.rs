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
