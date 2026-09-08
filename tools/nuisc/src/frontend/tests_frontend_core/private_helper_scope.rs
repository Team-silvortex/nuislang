use super::*;

fn recipes() -> nuis_semantics::model::AstModule {
    parse_nuis_ast(
        r#"
      mod cpu Recipes {
        pub fn compose(value: i64) -> i64 { return Recipes.adjust(value); }
        fn adjust(value: i64) -> i64 { return leaf(value) + 1; }
        fn leaf(value: i64) -> i64 { return value * 2; }
        fn unused(value: i64) -> i64 { return value + 900; }
      }
    "#,
    )
    .unwrap()
}

#[test]
fn imported_private_helper_composition_keeps_owner_scope_and_loop_execution() {
    let entry = parse_nuis_ast(
        r#"
      use cpu Recipes;
      use cpu Other;
      mod cpu Main {
        fn leaf(value: i64) -> i64 { return value + 500; }
        fn main() -> i64 {
          let buffer: ref Buffer = alloc_buffer(4, 0);
          let index: i64 = 0;
          while index < 4 {
            buffer[index] = Recipes.compose(index);
            let index: i64 = index + 1;
          }
          let result: i64 = buffer[0] + buffer[3] + Other.compute(1) + index;
          free(buffer);
          return result;
        }
      }
    "#,
    )
    .unwrap();
    let other = parse_nuis_ast(
        r#"
      mod cpu Other {
        pub fn compute(value: i64) -> i64 { return leaf(value); }
        fn leaf(value: i64) -> i64 { return value + 100; }
      }
    "#,
    )
    .unwrap();
    for helpers in [
        vec![recipes(), other.clone()],
        vec![other.clone(), recipes()],
    ] {
        let module = lower_project_ast_to_nir(&entry, &helpers).unwrap();
        for name in ["Recipes.adjust", "Recipes.leaf", "Other.leaf"] {
            let helper = module.functions.iter().find(|f| f.name == name).unwrap();
            assert_eq!(
                helper.visibility,
                nuis_semantics::model::NirVisibility::Private
            );
        }
        assert!(!module.functions.iter().any(|f| f.name == "Recipes.unused"));
        let manifest =
            crate::registry::load_manifest(std::path::Path::new("nustar-packages"), "official.cpu")
                .unwrap();
        let yir = crate::lowering::lower_nir_to_yir(&module, &manifest, None).unwrap();
        yir_verify::verify_module(&yir).unwrap();
        assert!(yir
            .nodes
            .iter()
            .any(|node| node.op.instruction == "call_i64"
                && node.op.args.first().map(String::as_str) == Some("Recipes.adjust")));
        let trace = yir_runtime_host::execute_module_source_with_registry(
            &crate::render::render_yir(&yir),
            &yir_verify::default_registry(),
        )
        .unwrap();
        let result = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap()
            .result
            .as_ref()
            .unwrap();
        assert_eq!(trace.values[&result.node], yir_core::Value::Int(113));
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}

#[test]
fn imported_private_helpers_are_not_exported_to_consumers() {
    for callee in ["adjust", "Recipes.adjust", "leaf", "Recipes.leaf"] {
        let entry = parse_nuis_ast(&format!(
            "use cpu Recipes; mod cpu Main {{ fn main() -> i64 {{ return {callee}(3); }} }}"
        ))
        .unwrap();
        assert!(
            lower_project_ast_to_nir(&entry, &[recipes()]).is_err(),
            "private call {callee}"
        );
    }
}

#[test]
fn private_helper_signatures_do_not_leak_between_imported_modules() {
    for callee in ["adjust", "Recipes.adjust"] {
        let entry = parse_nuis_ast(
            "use cpu Recipes; use cpu Other; mod cpu Main { fn main() -> i64 { return Other.compute(3); } }"
        ).unwrap();
        let other = parse_nuis_ast(&format!(
            "mod cpu Other {{ pub fn compute(value: i64) -> i64 {{ return {callee}(value); }} }}"
        ))
        .unwrap();
        for helpers in [
            vec![recipes(), other.clone()],
            vec![other.clone(), recipes()],
        ] {
            assert!(
                lower_project_ast_to_nir(&entry, &helpers).is_err(),
                "private sibling call {callee}"
            );
        }
    }
}
