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

pub fn literal_return_source(width: usize) -> String {
    assert!((10..=64).contains(&width));
    let extra = (9..width)
        .map(|i| {
            format!(
                ", extra{i}: {}",
                if i == 9 {
                    "99".into()
                } else {
                    format!("state.extra{i}")
                }
            )
        })
        .collect::<String>();
    let initialize = format!(
        "let carry = State {{ left: state.left, right: state.right, count: state.count{extra} }};"
    );
    let source = return_source(width);
    let (prefix, step) = source.split_once("fn step(").unwrap();
    let step = step
        .replace("tag: selected.tag, enabled: !enabled", "tag: i32_from_i64(17), enabled: !enabled")
        .replace("count: limit", "count: limit + 1")
        .replace("let carry = state; let i = 0;", &format!(
            "{initialize} if state.count % 2 == 0 {{ {initialize} }} else {{ {initialize} }} let i = 0;"
        ));
    format!("{prefix}fn step({step}")
}

pub fn post_loop_return_source(width: usize, exit: &str) -> String {
    let extra = (9..width)
        .map(|i| format!(", extra{i}: carry.extra{i}"))
        .collect::<String>();
    let exit = if exit.is_empty() {
        String::new()
    } else {
        format!("if warm == 2 {{ {exit} }}")
    };
    parent_return_source(width).replace(
        "let carry = state; let i = 0;",
        &format!(
            "let carry = state; let warm = 0; let warm_limit = state.count;
         while warm < warm_limit {{
             let warm = warm + 1;
             let carry = carry; let previous = carry.right;
             let carry = State {{
                 left: carry.left,
                 right: Leaf {{ value: previous.value + 0.5, gain: previous.gain,
                     tag: previous.tag, enabled: previous.enabled }},
                 count: carry.count{extra}
             }};
             {exit}
         }}
         let i = 0;"
        ),
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
