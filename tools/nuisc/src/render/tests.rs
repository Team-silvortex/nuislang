use super::*;

#[test]
fn renders_mutable_and_reassigned_ast_locals() {
    let module = AstModule {
        attributes: vec![],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![],
        traits: vec![],
        impls: vec![],
        functions: vec![AstFunction {
            visibility: AstVisibility::Private,
            name: "main".to_owned(),
            attributes: vec![],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params: vec![],
            return_type: Some(AstTypeRef {
                name: "i64".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            }),
            body: vec![
                AstStmt::Let {
                    mutable: true,
                    name: "value".to_owned(),
                    ty: Some(AstTypeRef {
                        name: "i64".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: false,
                    }),
                    value: AstExpr::Int(1),
                },
                AstStmt::AssignLocal {
                    name: "value".to_owned(),
                    value: AstExpr::Binary {
                        op: AstBinaryOp::Add,
                        lhs: Box::new(AstExpr::Var("value".to_owned())),
                        rhs: Box::new(AstExpr::Int(2)),
                    },
                },
                AstStmt::Return(Some(AstExpr::Var("value".to_owned()))),
            ],
        }],
    };

    let rendered = render_ast(&module);
    assert!(rendered.contains("let mut value: i64 = 1"), "{rendered}");
    assert!(rendered.contains("value = (value + 2)"), "{rendered}");
    assert!(rendered.contains("return value"), "{rendered}");
}

#[test]
fn renders_enum_declarations_in_ast() {
    let module = AstModule {
        attributes: vec![],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Main".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![nuis_semantics::model::AstEnumDef {
            visibility: AstVisibility::Public,
            attributes: vec![],
            name: "Option".to_owned(),
            generic_params: vec![AstGenericParam {
                name: "T".to_owned(),
                bounds: vec![],
            }],
            where_bounds: vec![],
            variants: vec![
                nuis_semantics::model::AstEnumVariant {
                    attributes: vec![],
                    name: "None".to_owned(),
                    kind: nuis_semantics::model::AstEnumVariantKind::Unit,
                },
                nuis_semantics::model::AstEnumVariant {
                    attributes: vec![],
                    name: "Some".to_owned(),
                    kind: nuis_semantics::model::AstEnumVariantKind::Tuple(vec![AstTypeRef {
                        name: "T".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: false,
                    }]),
                },
            ],
        }],
        traits: vec![],
        impls: vec![],
        functions: vec![],
    };

    let rendered = render_ast(&module);
    assert!(rendered.contains("pub enum Option<T>"), "{rendered}");
    assert!(rendered.contains("variant None"), "{rendered}");
    assert!(rendered.contains("variant Some(T)"), "{rendered}");
}

#[test]
fn renders_doc_comments_as_triple_slash_lines() {
    let module = AstModule {
        attributes: vec![AstAttribute {
            name: "doc".to_owned(),
            args: vec![AstAttributeArg {
                name: None,
                value: AstAttributeValue::String("module docs".to_owned()),
            }],
        }],
        uses: vec![],
        domain: "cpu".to_owned(),
        unit: "Docs".to_owned(),
        externs: vec![],
        extern_interfaces: vec![],
        consts: vec![nuis_semantics::model::AstConstItem {
            visibility: AstVisibility::Private,
            attributes: vec![AstAttribute {
                name: "doc".to_owned(),
                args: vec![AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String("const docs".to_owned()),
                }],
            }],
            name: "ANSWER".to_owned(),
            ty: Some(AstTypeRef {
                name: "i32".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            }),
            value: AstExpr::Int(42),
        }],
        type_aliases: vec![],
        structs: vec![],
        enums: vec![nuis_semantics::model::AstEnumDef {
            visibility: AstVisibility::Private,
            attributes: vec![],
            name: "Maybe".to_owned(),
            generic_params: vec![],
            where_bounds: vec![],
            variants: vec![nuis_semantics::model::AstEnumVariant {
                attributes: vec![AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String("empty docs".to_owned()),
                    }],
                }],
                name: "None".to_owned(),
                kind: nuis_semantics::model::AstEnumVariantKind::Unit,
            }],
        }],
        traits: vec![AstTraitDef {
            visibility: AstVisibility::Private,
            attributes: vec![AstAttribute {
                name: "doc".to_owned(),
                args: vec![AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String("trait docs".to_owned()),
                }],
            }],
            name: "Displayable".to_owned(),
            methods: vec![AstTraitMethodSig {
                attributes: vec![AstAttribute {
                    name: "doc".to_owned(),
                    args: vec![AstAttributeArg {
                        name: None,
                        value: AstAttributeValue::String("render docs".to_owned()),
                    }],
                }],
                name: "render".to_owned(),
                params: vec![nuis_semantics::model::AstParam {
                    name: "self".to_owned(),
                    ty: AstTypeRef {
                        name: "Self".to_owned(),
                        generic_args: vec![],
                        is_optional: false,
                        is_ref: false,
                    },
                }],
                return_type: Some(AstTypeRef {
                    name: "Text".to_owned(),
                    generic_args: vec![],
                    is_optional: false,
                    is_ref: false,
                }),
                default_body: None,
            }],
        }],
        impls: vec![],
        functions: vec![AstFunction {
            visibility: AstVisibility::Private,
            name: "answer".to_owned(),
            attributes: vec![AstAttribute {
                name: "doc".to_owned(),
                args: vec![AstAttributeArg {
                    name: None,
                    value: AstAttributeValue::String("function docs".to_owned()),
                }],
            }],
            test_name: None,
            test_ignored: false,
            test_should_fail: false,
            test_reason: None,
            test_timeout_ms: None,
            test_clock_domain: None,
            test_clock_policy: None,
            benchmark_name: None,
            benchmark_warmup_iters: None,
            benchmark_measure_iters: None,
            benchmark_timeout_ms: None,
            benchmark_clock_domain: None,
            benchmark_clock_policy: None,
            is_async: false,
            generic_params: vec![],
            where_bounds: vec![],
            params: vec![],
            return_type: Some(AstTypeRef {
                name: "i32".to_owned(),
                generic_args: vec![],
                is_optional: false,
                is_ref: false,
            }),
            body: vec![AstStmt::Return(Some(AstExpr::Int(42)))],
        }],
    };

    let rendered = render_ast(&module);
    assert!(rendered.starts_with("/// module docs\n"), "{rendered}");
    assert!(
        rendered.contains("/// const docs\n  const ANSWER: i32 = 42"),
        "{rendered}"
    );
    assert!(
        rendered.contains("/// empty docs\n    variant None"),
        "{rendered}"
    );
    assert!(
        rendered.contains("/// trait docs\n  trait Displayable"),
        "{rendered}"
    );
    assert!(
        rendered.contains("/// render docs\n    fn render(self: Self) -> Text;"),
        "{rendered}"
    );
    assert!(
        rendered.contains("/// function docs\n  fn answer() -> i32"),
        "{rendered}"
    );
    assert!(!rendered.contains("@doc("), "{rendered}");
}

#[test]
fn renders_multiline_shader_inline_wgsl_as_wgsl_block() {
    let rendered = render_nir_expr(&NirExpr::ShaderInlineWgsl {
        entry: "demo_shader".to_owned(),
        source: r#"
struct VsOut {
  @builtin(position) pos: vec4<f32>,
};

@vertex
fn vs_main() -> VsOut {
  var out: VsOut;
  return out;
}
"#
        .trim()
        .to_owned(),
    });

    assert!(
        rendered.contains("shader_inline_wgsl(\"demo_shader\", wgsl {"),
        "{rendered}"
    );
    assert!(rendered.contains("@vertex"), "{rendered}");
    assert!(rendered.contains("\n})"), "{rendered}");
    assert!(!rendered.contains("\\n"), "{rendered}");
}

#[test]
fn keeps_single_line_shader_inline_wgsl_as_string_literal() {
    let rendered = render_nir_expr(&NirExpr::ShaderInlineWgsl {
        entry: "demo_shader".to_owned(),
        source: "stub".to_owned(),
    });

    assert_eq!(rendered, "shader_inline_wgsl(\"demo_shader\", \"stub\")");
}
