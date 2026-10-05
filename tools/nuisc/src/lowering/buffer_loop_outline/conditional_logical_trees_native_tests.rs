use super::super::tests::native::run;
use super::*;

#[test]
fn conditional_logical_trees_execute_default_aot_nested_edges_traps_signals_and_zero_exits() {
    if !cfg!(all(
        any(target_os = "macos", target_os = "linux"),
        target_pointer_width = "64"
    )) {
        return;
    }
    let right = Tree::edge(
        false,
        Tree::Gate,
        Tree::edge(true, Tree::Leaf(0), Tree::Leaf(1)),
    );
    let left = Tree::edge(
        true,
        Tree::edge(false, Tree::Gate, Tree::Leaf(0)),
        Tree::Leaf(1),
    );
    let total = Tree::edge(
        false,
        Tree::edge(true, Tree::Gate, Tree::Leaf(0)),
        Tree::Gate,
    );
    for (mode, tree, gate, values, tail, early, expected) in [
        ("return", &right, false, [0, 0, 0], 0, false, Some("99\n19")),
        ("let", &right, true, [2, 0, 0], 0, false, Some("99\n11")),
        (
            "inferred",
            &right,
            true,
            [-2, 2, 0],
            0,
            false,
            Some("99\n11"),
        ),
        ("const", &right, true, [-2, 0, 0], 0, false, None),
        ("return", &right, true, [0, 2, 0], 0, false, None),
        (
            "complete",
            &left,
            false,
            [0, 2, 0],
            0,
            false,
            Some("99\n11"),
        ),
        ("complete", &left, true, [2, 0, 0], 0, false, Some("99\n11")),
        ("return", &left, true, [-2, 0, 0], 0, false, None),
        ("return", &total, true, [0, 0, 0], 0, false, Some("99\n11")),
        ("return", &total, false, [0, 0, 0], 0, false, None),
        (
            "partial",
            &right,
            false,
            [0, 0, 0],
            2,
            false,
            Some("99\n77\n19"),
        ),
        ("partial", &right, true, [2, 0, 0], 0, false, Some("99\n19")),
        ("suffix", &right, true, [0, 0, 0], 0, false, Some("99\n19")),
        (
            "continuation",
            &left,
            false,
            [0, 2, 0],
            2,
            false,
            Some("99\n77\n19"),
        ),
        ("return", &right, true, [0, 0, 0], 0, true, Some("19")),
    ] {
        let input = Input {
            outer: true,
            gate,
            values,
            tail,
            early,
        };
        let text = source(mode, tree, input)
            .replace("return 11;", "return 0;")
            .replace("return 19;", "return 0;");
        run(&text, expected);
    }
}
