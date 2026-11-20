use super::continuations::bridge;
use super::nested_returns::registration;
use super::record_helpers::compile;
use super::resource_predicates::source;
use super::*;

fn helpers(expression: &str) -> String {
    source(expression).replace(
        "mod cpu Main {",
        r#"mod cpu Main {
      fn word(flag: bool) -> i64 { if flag { return 9; } return 0; }
      fn gate(flag: bool, value: bool) -> bool { if flag { return value; } return false; }
    "#,
    )
}

#[test]
fn ready_scalar_selection_preserves_types_state_and_owners_for_120_events() {
    for expression in [
        "load_at(pixels, 3) == word(valid && bytes_len(saved) == 48)",
        "gate(valid && bytes_len(saved) == 48, mode < 3)",
        "word(gate(valid && bytes_len(saved) == 48, mode < 3)) == 9",
    ] {
        let source = helpers(expression);
        let program = compile(&source);
        assert!(program
            .yir
            .nodes
            .iter()
            .any(|n| n.op.instruction == "select"));
        assert_eq!(
            program.yir,
            yir_syntax::parse_explicit_module(&nuisc::render::render_yir(&program.yir)).unwrap()
        );
        let bridge = bridge(&program);
        assert!(!bridge.llvm_ir.contains("; deferred lowering"));
        let library = native::Library::build(&source, &program, &bridge.llvm_ir);
        let (mut session, _) = NativeResourceSession::open(
            &program.yir,
            &registration(),
            library.checked_binding(&program.yir, &bridge),
            vec![9; 6],
            3,
        )
        .unwrap();
        let mut pixels = vec![9; 6];
        let mut width = 3;
        for mode in (0..6).cycle().take(120) {
            let decision = mode % 2 == 0 && (!expression.contains("gate") || mode < 3);
            if decision {
                width += 10;
            } else {
                pixels[1] = mode;
                pixels[0] = width;
                width += 1;
            }
            session.event(mode).unwrap();
            assert_eq!(session.state(), pixels, "{expression}/{mode}");
            assert_eq!(session.scalar_state(), [width]);
            assert_eq!(library.live_blobs(), 0);
        }
        session.close(0).unwrap();
        session.completion_status().unwrap();
        assert_eq!(library.live_blobs(), 0);
    }
}

#[test]
fn scalar_selection_rechecks_both_arms_and_never_selects_resources() {
    for expression in [
        "load_at(pixels, 3) == word(valid && bytes_len(saved) == 48)",
        "gate(valid && bytes_len(saved) == 48, mode < 3)",
    ] {
        let program = compile(&helpers(expression));
        bridge(&program);
        let index = program
            .yir
            .nodes
            .iter()
            .position(|n| n.op.instruction == "select")
            .unwrap();
        let select = &program.yir.nodes[index];
        let function = program
            .yir
            .functions
            .iter()
            .find(|f| f.body_nodes.contains(&select.name))
            .unwrap();
        let buffer = function
            .parameters
            .iter()
            .find(|p| p.ty == "ref Buffer")
            .unwrap()
            .node
            .clone();
        let bytes = function
            .parameters
            .iter()
            .find(|p| p.ty == "Bytes")
            .unwrap()
            .node
            .clone();
        let integer = program
            .yir
            .nodes
            .iter()
            .find(|n| {
                function.body_nodes.contains(&n.name)
                    && matches!(n.op.instruction.as_str(), "const_i64" | "param_i64")
            })
            .unwrap()
            .name
            .clone();
        for case in 0..10 {
            let mut changed = program.yir.clone();
            let args = &mut changed.nodes[index].op.args;
            match case {
                0 => args[0] = integer.clone(),
                1 | 2 => {
                    args[case] = if expression.contains("gate") {
                        integer.clone()
                    } else {
                        args[0].clone()
                    }
                }
                3 => args[1] = buffer.clone(),
                4 => args[2] = bytes.clone(),
                5 => {
                    args[1] = bytes.clone();
                    args[2] = bytes.clone();
                }
                6 => args[2] = "unavailable_select_arm".into(),
                7 => {
                    args.pop();
                }
                8 => args[2] = select.name.clone(),
                _ => {
                    args.push(integer.clone());
                }
            }
            connect_select_inputs(&mut changed, index);
            assert!(
                yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                    .is_err(),
                "accepted {expression}/{case}"
            );
        }
        // Even a constant condition cannot excuse an invalid unselected arm.
        for (literal, arm) in [("true", 2), ("false", 1)] {
            let mut changed = program.yir.clone();
            let condition = insert_ready_node(
                &mut changed,
                select,
                &function.name,
                "const_bool",
                vec![literal.into()],
                &[],
            );
            changed.nodes[index].op.args[0] = condition;
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .unwrap();
            changed.nodes[index].op.args[arm] = bytes.clone();
            connect_select_inputs(&mut changed, index);
            let error =
                yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                    .unwrap_err();
            assert!(
                error.contains(&format!("resource callback `{}`", select.name)),
                "{error}"
            );
        }
        // Resource-free records and effect tokens are not scalar select values either.
        for instruction in ["struct", "store_at"] {
            let mut changed = program.yir.clone();
            let (args, dependencies) = if instruction == "struct" {
                (
                    vec!["Only".into(), format!("value={integer}")],
                    vec![integer.clone()],
                )
            } else {
                (
                    vec![buffer.clone(), integer.clone(), integer.clone()],
                    vec![buffer.clone(), integer.clone()],
                )
            };
            let value = insert_ready_node(
                &mut changed,
                select,
                &function.name,
                instruction,
                args,
                &dependencies,
            );
            yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                .unwrap();
            changed.nodes[index].op.args[1] = value.clone();
            changed.nodes[index].op.args[2] = value;
            let error =
                yir_lower_llvm::native_resource_session::emit_registered(&changed, &registration())
                    .unwrap_err();
            assert!(
                error.contains("select requires ready i64 or bool arms"),
                "{error}"
            );
        }
    }
}

fn insert_ready_node(
    module: &mut yir_core::YirModule,
    select: &yir_core::Node,
    function: &str,
    instruction: &str,
    args: Vec<String>,
    dependencies: &[String],
) -> String {
    let name = "scalar_select_test_ready".to_owned();
    assert!(!module.nodes.iter().any(|n| n.name == name));
    let mut node = select.clone();
    node.name = name.clone();
    node.op.instruction = instruction.into();
    node.op.args = args;
    module.nodes.push(node);
    module
        .node_lanes
        .insert(name.clone(), format!("fn:{function}"));
    let body = &mut module
        .functions
        .iter_mut()
        .find(|f| f.name == function)
        .unwrap()
        .body_nodes;
    body.insert(
        body.iter().position(|n| n == &select.name).unwrap(),
        name.clone(),
    );
    for dependency in dependencies {
        module.edges.push(yir_core::Edge {
            from: dependency.clone(),
            to: name.clone(),
            kind: yir_core::EdgeKind::Dep,
        });
    }
    if instruction == "store_at" {
        module.edges.push(yir_core::Edge {
            from: dependencies[0].clone(),
            to: name.clone(),
            kind: yir_core::EdgeKind::Lifetime,
        });
    }
    module.edges.push(yir_core::Edge {
        from: name.clone(),
        to: select.name.clone(),
        kind: yir_core::EdgeKind::Dep,
    });
    name
}

fn connect_select_inputs(module: &mut yir_core::YirModule, index: usize) {
    let node = &module.nodes[index];
    for arg in &node.op.args {
        if module.nodes.iter().any(|n| n.name == *arg)
            && !module
                .edges
                .iter()
                .any(|e| e.from == *arg && e.to == node.name && e.kind == yir_core::EdgeKind::Dep)
        {
            module.edges.push(yir_core::Edge {
                from: arg.clone(),
                to: node.name.clone(),
                kind: yir_core::EdgeKind::Dep,
            });
        }
    }
}

#[test]
fn resource_comparison_wrappers_use_guarded_lowering() {
    for expression in [
        "(valid && load_at(pixels, 3) == 9) == (mode < 3)",
        "(valid || bytes_len(saved) == 0) != (mode < 3)",
    ] {
        let program = compile(&helpers(expression));
        assert!(!bridge(&program).llvm_ir.contains("; deferred lowering"));
    }
}
