use super::tests::{execute, outline_test};
use super::*;

#[path = "conditional_logical_trees_budget_tests.rs"]
mod budgets;
#[path = "conditional_logical_trees_native_tests.rs"]
mod native;

#[derive(Clone)]
enum Tree {
    Gate,
    Leaf(usize),
    Edge(bool, Box<Tree>, Box<Tree>),
}

impl Tree {
    fn edge(or: bool, lhs: Tree, rhs: Tree) -> Self {
        Self::Edge(or, Box::new(lhs), Box::new(rhs))
    }

    fn source(&self) -> String {
        match self {
            Self::Gate => "gate".into(),
            Self::Leaf(index) => {
                format!("helper(produce({}))", ["left", "middle", "right"][*index])
            }
            Self::Edge(or, lhs, rhs) => format!(
                "({}) {} ({})",
                lhs.source(),
                if *or { "||" } else { "&&" },
                rhs.source(),
            ),
        }
    }

    fn evaluate(&self, input: Input, calls: &mut usize) -> Result<bool, ()> {
        match self {
            Self::Gate => Ok(input.gate),
            Self::Leaf(index) => {
                let divisor = input.values[*index];
                if divisor == 0 {
                    return Err(());
                }
                *calls += 1;
                Ok(divisor > 0)
            }
            Self::Edge(or, lhs, rhs) => {
                let left = lhs.evaluate(input, calls)?;
                if left == *or {
                    Ok(left)
                } else {
                    rhs.evaluate(input, calls)
                }
            }
        }
    }

    fn has_calls(&self) -> bool {
        match self {
            Self::Gate => false,
            Self::Leaf(_) => true,
            Self::Edge(_, lhs, rhs) => lhs.has_calls() || rhs.has_calls(),
        }
    }

    fn helpers(&self) -> usize {
        match self {
            Self::Edge(_, lhs, rhs) => usize::from(rhs.has_calls()) + lhs.helpers() + rhs.helpers(),
            _ => 0,
        }
    }
}

#[derive(Clone, Copy)]
struct Input {
    outer: bool,
    gate: bool,
    values: [i64; 3],
    tail: i64,
    early: bool,
}

const INPUTS: [Input; 8] = [
    Input {
        outer: true,
        gate: false,
        values: [-2, 0, 0],
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [2, 0, 0],
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: false,
        values: [2, 2, -2],
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [-2, -2, 2],
        tail: 2,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [0, 2, 2],
        tail: 2,
        early: false,
    },
    Input {
        outer: false,
        gate: false,
        values: [0, 0, 0],
        tail: 0,
        early: false,
    },
    Input {
        outer: true,
        gate: true,
        values: [0, 0, 0],
        tail: 0,
        early: true,
    },
    Input {
        outer: true,
        gate: false,
        values: [2, -2, 2],
        tail: 2,
        early: false,
    },
];

fn trees() -> Vec<Tree> {
    let mut trees = Vec::new();
    for outer in [false, true] {
        for inner in [false, true] {
            trees.push(Tree::edge(
                outer,
                Tree::edge(inner, Tree::Gate, Tree::Leaf(0)),
                Tree::Leaf(1),
            ));
            trees.push(Tree::edge(
                outer,
                Tree::Leaf(0),
                Tree::edge(inner, Tree::Gate, Tree::Leaf(1)),
            ));
        }
        trees.push(Tree::edge(
            outer,
            Tree::edge(!outer, Tree::Gate, Tree::Leaf(0)),
            Tree::Gate,
        ));
    }
    trees
}

fn source(mode: &str, tree: &Tree, input: Input) -> String {
    let value = tree.source();
    let body = match mode {
        "let" => format!("let result: bool = {value}; return result;"),
        "inferred" => format!("let result = {value}; return result;"),
        "const" => format!("const result: bool = {value}; return result;"),
        "return" => format!("return {value};"),
        "complete" => format!("if {value} {{ return true; }} else {{ return false; }}"),
        "partial" => format!("if {value} {{ return false; }} else {{ let checked = 20 / tail; }}"),
        "continuation" => format!("if gate {{ return false; }} else {{ let ignored = {value}; let checked = 20 / tail; }}"),
        "suffix" => format!("if gate {{ return false; }} let ignored = {value}; let checked = 20 / tail;"),
        "entry" => unreachable!(),
        _ => panic!("{mode}"),
    };
    let branch = format!("if outer {{ {body} }}");
    let [left, middle, right] = input.values;
    format!(
        "mod cpu Main {{
        struct Packet {{ unused: i64, value: i64 }}
        @noinline fn produce(input: i64) -> Packet {{ return Packet {{ unused: 10 / input, value: input }}; }}
        @noinline fn helper(packet: Packet) -> bool {{ return packet.value > 0; }}
        @noinline fn event(outer: bool, gate: bool, left: i64, middle: i64, right: i64, tail: i64, early: bool) -> bool {{
            if early {{ return false; }} print(99); {branch} print(77); return false;
        }}
        fn main() -> i64 {{
            let result = event({}, {}, {left}, {middle}, {right}, {}, {});
            if result {{ print(11); return 11; }} print(19); return 19;
        }}
    }}", input.outer, input.gate, input.tail, input.early)
}

fn oracle(mode: &str, tree: &Tree, input: Input) -> (Option<i64>, Vec<i64>, usize) {
    let mut calls = 0;
    let continuation = matches!(mode, "continuation" | "suffix");
    let entered = input.outer && !input.early;
    let evaluated = entered && (!continuation || !input.gate);
    let value = if evaluated {
        tree.evaluate(input, &mut calls)
    } else {
        Ok(false)
    };
    let complete = !matches!(mode, "partial" | "continuation" | "suffix");
    let returned = entered
        && if continuation {
            input.gate
        } else {
            complete || value == Ok(true)
        };
    let failed = value.is_err() || entered && !returned && input.tail == 0;
    let result = if entered && complete && value == Ok(true) {
        11
    } else {
        19
    };
    let prints = if input.early {
        vec![19]
    } else if returned {
        vec![99, result]
    } else {
        vec![99, 77, result]
    };
    ((!failed).then_some(result), prints, calls)
}

#[test]
fn conditional_logical_trees_preserve_nested_lhs_rhs_binding_roots_and_exit_readiness() {
    let mut cases = 0;
    for mode in [
        "let",
        "inferred",
        "const",
        "return",
        "complete",
        "partial",
        "continuation",
        "suffix",
    ] {
        for tree in trees() {
            for input in INPUTS {
                let (expected, prints, calls) = oracle(mode, &tree, input);
                execute(&source(mode, &tree, input), expected, &prints, calls);
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 640);
}

#[test]
fn conditional_logical_trees_compose_outer_entries_balanced_edges_repeated_calls_and_zero_exits() {
    let mut cases = 0;
    for tree in [
        Tree::edge(
            false,
            Tree::edge(true, Tree::Gate, Tree::Leaf(0)),
            Tree::edge(false, Tree::Leaf(1), Tree::Leaf(2)),
        ),
        Tree::edge(
            true,
            Tree::edge(false, Tree::Gate, Tree::Leaf(0)),
            Tree::edge(true, Tree::Leaf(1), Tree::Leaf(0)),
        ),
    ] {
        for input in INPUTS {
            let text = source("return", &tree, input);
            let (expected, prints, calls) = oracle("return", &tree, input);
            execute(
                &text.replace("print(11); return 11;", "print(0); return 0;"),
                expected.map(|n| if n == 11 { 0 } else { n }),
                &prints
                    .iter()
                    .map(|n| if *n == 11 { 0 } else { *n })
                    .collect::<Vec<_>>(),
                calls,
            );
            let entry = text.replace(
                &format!("if outer {{ return {}; }}", tree.source()),
                &format!("if outer && ({}) {{ return true; }}", tree.source()),
            );
            let entry_prints = if expected == Some(19) && !input.early {
                vec![99, 77, 19]
            } else {
                prints
            };
            execute(&entry, expected, &entry_prints, calls);
            cases += 2;
        }
    }
    assert_eq!(cases, 32);
}

fn outline_values(module: &mut NirModule) -> BTreeSet<String> {
    let layouts = control_values::TypedLayouts::collect(module);
    let carries = control_values::layouts(module);
    let control = scalar_helpers::collect_with_layouts(module, &carries);
    let catalog = scalar_helpers::collect_typed_values(module, &layouts, &control);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    conditional_values::outline(
        module,
        &catalog,
        &BTreeSet::from(["event".into()]),
        &layouts,
        &mut names,
    )
}

fn event(module: &mut NirModule) -> &mut NirFunction {
    module
        .functions
        .iter_mut()
        .find(|f| f.name == "event")
        .unwrap()
}

#[test]
fn conditional_logical_trees_keep_linear_helpers_original_capture_scope_and_idempotence() {
    for tree in trees() {
        let mut module =
            crate::frontend::parse_nuis_module(&source("return", &tree, INPUTS[2])).unwrap();
        let generated = outline_values(&mut module);
        assert_eq!(generated.len(), tree.helpers());
        crate::nir_verify::verify_nir_module(&module).unwrap();
        for helper in module
            .functions
            .iter()
            .filter(|f| generated.contains(&f.name))
        {
            assert_eq!(helper.return_type, Some(scalar_type("bool")));
            assert_eq!(helper.params[0].ty, scalar_type("bool"));
            assert!(helper
                .params
                .iter()
                .skip(1)
                .all(|p| ["gate", "left", "middle", "right"].contains(&p.name.as_str())));
            let [NirStmt::If {
                condition: NirExpr::Var(predicate),
                then_body,
                else_body,
            }] = helper.body.as_slice()
            else {
                panic!("{:?}", helper.body)
            };
            assert_eq!(predicate, &helper.params[0].name);
            assert!(matches!(then_body.as_slice(), [NirStmt::Return(Some(_))]));
            assert!(matches!(else_body.as_slice(), [NirStmt::Return(Some(_))]));
        }
        let once = module.clone();
        assert!(outline_values(&mut module).is_empty());
        assert_eq!(module, once);
    }
}

#[test]
fn conditional_logical_trees_retain_leaf_embedding_type_effect_scope_and_loop_vetoes() {
    let tree = Tree::edge(
        false,
        Tree::Gate,
        Tree::edge(true, Tree::Gate, Tree::Leaf(0)),
    );
    let base = source("return", &tree, INPUTS[2]);
    for mutation in [
        "borrow", "optional", "generic", "effect", "unknown", "kind", "embedded", "loop",
    ] {
        let mut module = crate::frontend::parse_nuis_module(&base).unwrap();
        match mutation {
            "effect" => module
                .functions
                .iter_mut()
                .find(|f| f.name == "produce")
                .unwrap()
                .body
                .insert(0, NirStmt::Print(NirExpr::Int(88))),
            "borrow" => event(&mut module).params[1].ty.is_ref = true,
            "optional" => event(&mut module).params[1].ty.is_optional = true,
            "generic" => event(&mut module).params[1]
                .ty
                .generic_args
                .push(scalar_type("bool")),
            "loop" => {
                let NirStmt::If { then_body, .. } = &mut event(&mut module).body[2] else {
                    panic!()
                };
                let body = std::mem::take(then_body);
                *then_body = vec![NirStmt::While {
                    condition: NirExpr::Bool(false),
                    body,
                }];
            }
            _ => {
                let NirStmt::If { then_body, .. } = &mut event(&mut module).body[2] else {
                    panic!()
                };
                let NirStmt::Return(Some(value)) = &mut then_body[0] else {
                    panic!()
                };
                let NirExpr::Binary { rhs, .. } = value else {
                    panic!()
                };
                **rhs = match mutation {
                    "unknown" => NirExpr::Call {
                        callee: "missing".into(),
                        args: vec![],
                    },
                    "kind" => NirExpr::Int(1),
                    "embedded" => NirExpr::Binary {
                        op: NirBinaryOp::Eq,
                        lhs: rhs.clone(),
                        rhs: Box::new(NirExpr::Bool(true)),
                    },
                    _ => unreachable!(),
                };
            }
        }
        let before = module.clone();
        assert!(outline_values(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
        assert!(outline_test(&mut module).is_empty(), "{mutation}");
        assert_eq!(module, before, "{mutation}");
    }
    // The legacy atom-loop gate is still accepted, but nested RHS admission
    // must not leak through the atom check in a source loop.
    for nested in [false, true] {
        let text = base.replace(
            &format!("return {};", tree.source()),
            if nested {
                "if gate && (gate || helper(produce(left))) { print(88); }"
            } else {
                "if gate && helper(produce(left)) { print(88); }"
            },
        );
        let mut module = crate::frontend::parse_nuis_module(&text).unwrap();
        let NirStmt::If { then_body, .. } = &mut event(&mut module).body[2] else {
            panic!()
        };
        let body = std::mem::take(then_body);
        *then_body = vec![NirStmt::While {
            condition: NirExpr::Bool(false),
            body,
        }];
        let before = module.clone();
        assert_eq!(outline_values(&mut module).is_empty(), nested);
        if nested {
            assert_eq!(module, before);
        }
    }
}
