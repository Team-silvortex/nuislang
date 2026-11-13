use super::*;

fn source(body: &str) -> AstModule {
    crate::frontend::parse_nuis_ast(&format!(r#"
      mod cpu Main {{
        const __nuis_resource_local_0: i64 = 88;
        fn step(pixels: ref Buffer, width: i64) -> i64 {{ {body} }}
        fn other(width: i64) -> i64 {{ if width > 0 {{ let width: i64 = width + 1; }} return width; }}
      }}
    "#)).unwrap()
}

#[test]
fn resource_roots_only_preserve_source_ast_and_skip_empty_registration() {
    let ast = source("if width > 0 { let width: i64 = width + 1; } return width;");
    assert!(matches!(
        normalize(&ast, &BTreeSet::new()).ast,
        Cow::Borrowed(_)
    ));
    let normalized = normalize(&ast, &BTreeSet::from(["step".into()])).ast;
    assert_ne!(ast.functions[0], normalized.functions[0]);
    assert_eq!(ast.functions[1], normalized.functions[1]);
    assert_eq!(ast.functions[0].params, normalized.functions[0].params);
}

#[test]
fn initializers_and_symbols_keep_identity_while_later_names_are_reserved() {
    let ast = source(
        r#"
      if width > 0 {
        let width: i64 = width + 1;
        width(width);
        Result.Ok(Canvas { width: width });
      }
      let __nuis_resource_local_1: i64 = 7;
      return width;
    "#,
    );
    let normalized = normalize(&ast, &BTreeSet::from(["step".into()])).ast;
    let AstStmt::If { then_body, .. } = &normalized.functions[0].body[0] else {
        panic!()
    };
    let AstStmt::Let { name, value, .. } = &then_body[0] else {
        panic!()
    };
    assert_eq!(name, "__nuis_resource_local_2");
    let AstExpr::Binary { lhs, .. } = value else {
        panic!()
    };
    assert_eq!(lhs.as_ref(), &AstExpr::Var("width".into()));
    let AstStmt::Expr(AstExpr::Call { callee, args, .. }) = &then_body[1] else {
        panic!()
    };
    assert_eq!(callee, "width");
    assert_eq!(args, &[AstExpr::Var(name.clone())]);
    let AstStmt::Expr(AstExpr::Call { callee, args, .. }) = &then_body[2] else {
        panic!()
    };
    assert_eq!(callee, "Result.Ok");
    let AstExpr::StructLiteral {
        type_name, fields, ..
    } = &args[0]
    else {
        panic!()
    };
    assert_eq!(type_name, "Canvas");
    assert_eq!(fields, &[("width".into(), AstExpr::Var(name.clone()))]);
    assert_eq!(
        normalized.functions[0].body.last(),
        Some(&AstStmt::Return(Some(AstExpr::Var("width".into()))))
    );
}

#[test]
fn unbound_private_looking_uses_cannot_be_captured_by_generated_names() {
    let ast = source(
        r#"
      if width > 0 {
        let width: i64 = width + 1;
        store_at(pixels, 0, __nuis_resource_local_1);
      }
      return width;
    "#,
    );
    let normalized = normalize(&ast, &BTreeSet::from(["step".into()])).ast;
    let AstStmt::If { then_body, .. } = &normalized.functions[0].body[0] else {
        panic!()
    };
    let AstStmt::Let { name, .. } = &then_body[0] else {
        panic!()
    };
    assert_eq!(name, "__nuis_resource_local_2");
    assert_eq!(
        then_body[1],
        match &ast.functions[0].body[0] {
            AstStmt::If { then_body, .. } => then_body[1].clone(),
            _ => panic!(),
        }
    );
}

#[test]
fn unsupported_lexical_regions_are_never_partially_renamed() {
    for suffix in [
        "while width > 0 { break; } return width;",
        "return if width > 0 { width } else { 0 };",
    ] {
        let ast = source(&format!(
            "if width > 0 {{ let width: i64 = width + 1; }} {suffix}"
        ));
        assert_eq!(
            normalize(&ast, &BTreeSet::from(["step".into()]))
                .ast
                .as_ref(),
            &ast
        );
    }
}

#[test]
fn shared_semantics_resolves_imported_generic_aliases_and_inferred_locals() {
    let ast = crate::frontend::parse_nuis_ast(
        r#"
      use cpu Types;
      mod cpu Main {
        fn step(pixels: ref Buffer, width: Size) -> i64 {
          let count = width;
          if width > 0 {
            let count = load_at(pixels, 0) + count;
            const width: Identity<Size> = count;
            store_at(pixels, 1, width);
          }
          return count;
        }
      }
    "#,
    )
    .unwrap();
    let helper = crate::frontend::parse_nuis_ast(
        r#"
      mod cpu Types {
        pub type Identity<T> = T;
        pub type Size = Identity<i64>;
      }
    "#,
    )
    .unwrap();
    let normalized = normalize(&ast, &BTreeSet::from(["step".into()]));
    assert_eq!(normalized.shadows["step"].len(), 2);
    let nir = normalized.lower(&[helper]).unwrap();
    let step = nir.functions.iter().find(|f| f.name == "step").unwrap();
    assert_eq!(step.params[1].ty.render(), "i64");
    let nuis_semantics::model::NirStmt::If { then_body, .. } = &step.body[1] else {
        panic!()
    };
    let nuis_semantics::model::NirStmt::Let { ty: Some(ty), .. } = &then_body[0] else {
        panic!()
    };
    assert_eq!(ty.render(), "i64");
}

#[test]
fn type_admission_uses_the_binding_at_the_declaration_not_the_last_same_name() {
    let ast = source(
        r#"
      let count = width;
      if width > 0 {
        let count = count + 1;
        store_at(pixels, 0, count);
      }
      let count = copy_bytes(pixels);
      drop_bytes(count);
      return width;
    "#,
    );
    normalize(&ast, &BTreeSet::from(["step".into()]))
        .lower(&[])
        .unwrap();
}

#[test]
fn canonical_type_and_declaration_drift_cannot_skip_shadow_admission() {
    use nuis_semantics::model::NirStmt;
    let ast =
        source("if width > 0 { let width = width + 1; store_at(pixels, 0, width); } return width;");
    let normalized = normalize(&ast, &BTreeSet::from(["step".into()]));
    let nir = normalized.lower(&[]).unwrap();
    for case in 0..6 {
        let mut changed = nir.clone();
        let step = changed
            .functions
            .iter_mut()
            .find(|f| f.name == "step")
            .unwrap();
        let NirStmt::If { then_body, .. } = &mut step.body[0] else {
            panic!()
        };
        if case == 0 {
            then_body.remove(0);
        } else if case == 1 {
            step.params[1].ty.name = "Bytes".into();
        } else {
            let NirStmt::Let { ty, .. } = &mut then_body[0] else {
                panic!()
            };
            match case {
                2 => *ty = None,
                3 => ty.as_mut().unwrap().is_ref = true,
                4 => ty.as_mut().unwrap().is_optional = true,
                _ => ty.as_mut().unwrap().name = "f64".into(),
            }
        }
        assert!(
            types::validate(&changed, &normalized.shadows).is_err(),
            "case={case}"
        );
    }
    let mut changed = nir;
    changed.functions.retain(|f| f.name != "step");
    assert!(types::validate(&changed, &normalized.shadows).is_err());
}

#[test]
fn alias_cycles_and_invalid_inference_remain_semantic_errors() {
    for declarations in [
        "type Size = Missing;",
        "type Size = Other; type Other = Size;",
    ] {
        let ast = crate::frontend::parse_nuis_ast(&format!(
            r#"
          mod cpu Main {{
            {declarations}
            fn step(width: i64) -> i64 {{
              if width > 0 {{ let width: Size = width + 1; }}
              return width;
            }}
          }}
        "#
        ))
        .unwrap();
        assert!(normalize(&ast, &BTreeSet::from(["step".into()]))
            .lower(&[])
            .is_err());
    }
    let ast = source("if width > 0 { let width = missing_value; } return width;");
    assert!(normalize(&ast, &BTreeSet::from(["step".into()]))
        .lower(&[])
        .is_err());
}
