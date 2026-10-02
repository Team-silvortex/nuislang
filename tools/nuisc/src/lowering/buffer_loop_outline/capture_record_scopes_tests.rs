use super::*;

#[test]
fn sparse_word_scopes_compile_branch_local_typed_snapshots_with_current_backedges() {
    let source = include_str!(
        "../../../tests/control_flow_syntax_native/scoped_branch_typed_record_carries.ns"
    );
    let mut yir = crate::pipeline::compile_source(source).unwrap().yir;
    let call = yir
        .nodes
        .iter()
        .find_map(|node| {
            yir_core::loop_carry_contract::parse_scoped_i64_carries(&node.op.args)
                .ok()
                .flatten()
        })
        .unwrap();
    assert_eq!(call.seeds.len(), 9);
    assert_eq!(call.operands.len(), 6);
    assert_eq!(scoped_inputs::carry_tests::reference(&mut yir).unwrap(), 19);
}

fn binding(name: &str, ty: &str, value: NirExpr) -> NirStmt {
    NirStmt::Let {
        name: name.into(),
        ty: Some(scalar_type(ty)),
        value,
    }
}

fn path(root: &str, fields: &[&str]) -> NirExpr {
    source_value(
        root,
        &fields.iter().map(|s| (*s).into()).collect::<Vec<_>>(),
    )
}

fn branch(then_body: Vec<NirStmt>, else_body: Vec<NirStmt>) -> NirStmt {
    NirStmt::If {
        condition: NirExpr::Bool(true),
        then_body,
        else_body,
    }
}

#[test]
fn sparse_word_scopes_project_branch_local_aliases_without_sibling_leaks() {
    for reverse in [false, true] {
        let (mut module, _) = fixture();
        module.functions[0].body.insert(
            1,
            branch(
                vec![
                    binding("local", "Packet", path("old", &[])),
                    binding("leaf", "Leaf", path("local", &["right"])),
                    NirStmt::Expr(path("leaf", &["value"])),
                ],
                vec![
                    binding("local", "Leaf", path("old", &["left"])),
                    NirStmt::Expr(path("local", &["value"])),
                ],
            ),
        );
        // This name was child-local, so it must not constrain the later parent binding.
        module.functions[0]
            .body
            .insert(2, binding("local", "i64", NirExpr::Int(7)));
        if reverse {
            module.functions.reverse();
        }
        assert!(project_fixture(&mut module));
        crate::nir_verify::verify_nir_module(&module).unwrap();
        let helper = module
            .functions
            .iter()
            .find(|f| f.name == "helper")
            .unwrap();
        assert_eq!(helper.params.len(), 7);
        assert!(helper.params.iter().all(|p| p.ty == scalar_type("i64")));
        assert!(!project_fixture(&mut module));
    }
}

#[test]
fn sparse_word_scopes_keep_identity_versions_across_joins_and_loop_backedges() {
    let (mut module, _) = fixture();
    let aliases = vec![
        binding("local", "Packet", path("old", &[])),
        binding("old", "Packet", path("local", &[])),
        binding("leaf", "Leaf", path("local", &["left"])),
        NirStmt::Expr(path("leaf", &["tag"])),
    ];
    module.functions[0]
        .body
        .insert(1, branch(aliases.clone(), vec![]));
    module.functions[0].body.insert(
        2,
        NirStmt::While {
            condition: NirExpr::Bool(false),
            body: vec![branch(aliases, vec![])],
        },
    );
    assert!(project_fixture(&mut module));
    crate::nir_verify::verify_nir_module(&module).unwrap();
    assert_eq!(module.functions[0].params.len(), 6);
    assert!(!project_fixture(&mut module));
}

#[test]
fn sparse_word_scopes_materialize_outer_writes_and_reject_unproven_versions() {
    for change in [
        "origin",
        "loop-origin",
        "materialized",
        "prefix",
        "whole",
        "type",
    ] {
        let (mut module, _) = fixture();
        let zero = control_values::zero_value(
            &scalar_type("Packet"),
            &control_values::TypedLayouts::collect(&module),
        );
        let body = &mut module.functions[0].body;
        let NirStmt::Let {
            value: computed, ..
        } = body[2].clone()
        else {
            panic!()
        };
        let mut statements = vec![binding("local", "Packet", path("old", &[]))];
        match change {
            "origin" | "loop-origin" => statements.push(binding("old", "Packet", computed)),
            "materialized" => {
                body.insert(2, binding("outer", "Packet", computed));
                statements.push(binding("outer", "Packet", path("local", &[])));
            }
            "prefix" => {
                body.insert(0, binding("outer", "Packet", zero));
                statements.push(binding("outer", "Packet", path("local", &[])));
            }
            "whole" => statements.push(NirStmt::Expr(NirExpr::Call {
                callee: "consume".into(),
                args: vec![path("local", &[])],
            })),
            "type" => statements.push(binding("leaf", "Packet", path("local", &["left"]))),
            _ => unreachable!(),
        }
        let stmt = if change == "loop-origin" {
            NirStmt::While {
                condition: NirExpr::Bool(false),
                body: statements,
            }
        } else {
            branch(statements, vec![])
        };
        let position = if change == "materialized" { 3 } else { 2 };
        module.functions[0].body.insert(position, stmt);
        let before = module.clone();
        if matches!(change, "origin" | "loop-origin" | "materialized" | "prefix") {
            assert!(project_fixture(&mut module), "{change}");
            let position = usize::from(change == "prefix");
            assert_eq!(
                module.functions[0].body[..position],
                before.functions[0].body[..position]
            );
            assert_eq!(
                module.functions[0].body[position + 1..],
                before.functions[0].body[position + 1..]
            );
            assert_eq!(module.functions[0].params.len(), 6);
            crate::nir_verify::verify_nir_module(&module).unwrap();
        } else {
            assert!(!project_fixture(&mut module), "{change}");
            assert_eq!(module, before, "{change}");
        }
    }
}

#[test]
fn sparse_word_scopes_preserve_nonidentity_joins_and_loop_versions() {
    for looped in [false, true] {
        let (mut module, _) = fixture();
        let body = &mut module.functions[0].body;
        body.insert(1, binding("selected", "Leaf", path("old", &["left"])));
        let update = binding("selected", "Leaf", path("old", &["right"]));
        let stmt = if looped {
            NirStmt::While {
                condition: NirExpr::Bool(false),
                body: vec![update],
            }
        } else {
            branch(vec![update.clone()], vec![update])
        };
        body.insert(2, stmt);
        body.insert(3, NirStmt::Expr(path("selected", &["value"])));
        let before = module.clone();
        assert!(project_fixture(&mut module));
        assert_eq!(module.functions[0].params.len(), 7);
        assert_eq!(module.functions[0].body[1..], before.functions[0].body[1..]);
        crate::nir_verify::verify_nir_module(&module).unwrap();
    }
}

#[test]
fn sparse_word_scopes_keep_computed_child_initializers_and_outer_parameter_writes() {
    let (mut module, _) = fixture();
    let NirStmt::Let {
        value: computed, ..
    } = module.functions[0].body[2].clone()
    else {
        panic!()
    };
    module.functions[0].body.insert(
        2,
        branch(
            vec![
                binding("local", "Packet", path("old", &[])),
                binding("checked", "Packet", computed),
            ],
            vec![],
        ),
    );
    assert!(project_fixture(&mut module));
    let mut preserved = false;
    walk::visit(&module.functions[0].body, |expr| {
        preserved |=
            matches!(expr, NirExpr::StructLiteral { type_name, .. } if type_name == "Packet");
        true
    });
    let NirStmt::If { then_body, .. } = &module.functions[0].body[0] else {
        panic!()
    };
    assert!(matches!(then_body.as_slice(), [NirStmt::Let { name, .. }] if name == "checked"));
    assert!(preserved);
    crate::nir_verify::verify_nir_module(&module).unwrap();

    let (mut module, _) = fixture();
    module.functions[0].params.push(NirParam {
        name: "outer".into(),
        ty: scalar_type("Packet"),
    });
    module.functions[0].body.insert(
        1,
        branch(vec![binding("outer", "Packet", path("old", &[]))], vec![]),
    );
    walk::rewrite(&mut module.functions[1].body, |expr| {
        if let NirExpr::Call { callee, args } = expr {
            if callee == "helper" {
                args.push(path("carry", &[]));
            }
        }
    });
    let before = module.clone();
    assert!(project_fixture(&mut module));
    assert!(module.functions[0]
        .params
        .iter()
        .any(|p| p.name == "outer" && p.ty == scalar_type("Packet")));
    assert_eq!(module.functions[0].body[1..], before.functions[0].body[1..]);
    crate::nir_verify::verify_nir_module(&module).unwrap();
}
