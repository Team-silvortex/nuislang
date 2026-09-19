use super::{lower_nir_to_yir_builtin_cpu, parse_nuis_module};

const SOURCE: &str = r#"
        mod cpu Main {
          fn main() -> i64 {
            let value: i64 = 0;
            let acc: i64 = 0;
            while value < 5 {
              let value: i64 = value + 1;
              if value > 3 {
                let acc: i64 = acc + value;
              } else {
                if value > 1 {
                  let acc: i64 = acc + value;
                } else {
                  let acc: i64 = acc + 0;
                }
              }
            }
            return acc;
          }
        }
        "#;

fn check(source: &str, expected: i64) {
    let mut module = parse_nuis_module(source).unwrap();
    crate::optimize::simplify_nir_module(&mut module);
    for reversed in [false, true] {
        if reversed {
            module.functions.reverse();
        }
        let mut yir = lower_nir_to_yir_builtin_cpu(&module).unwrap();
        assert!(yir
            .functions
            .iter()
            .any(|f| f.name.starts_with("__nuis_scalar_iteration_")));
        assert_eq!(yir.functions.iter().filter(|f| f.name == "main").count(), 1);
        assert!(yir
            .functions
            .iter()
            .any(|f| f.name == "main" && f.role == yir_core::YirFunctionRole::Helper));
        let entry = yir
            .functions
            .iter()
            .find(|f| f.role == yir_core::YirFunctionRole::Entry)
            .unwrap();
        assert_ne!(entry.name, "main");
        if source.contains("fn __nuis_entry_main_1") {
            assert_eq!(entry.name, "__nuis_entry_main_2");
        }
        // Permute declarations, never instructions or their dependency edges.
        if reversed {
            yir.nodes.reverse();
            yir.functions.reverse();
        }
        yir_verify::verify_module(&yir).unwrap();
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
        assert_eq!(trace.values[&result.node], yir_core::Value::Int(expected));
        yir_lower_llvm::emit_module(&yir).unwrap();
    }
}

#[test]
fn lowers_nested_if_branching_carry_through_scoped_iteration() {
    // Preserve the earlier compact-chain test's source; scoped lowering now
    // validates execution semantics rather than one obsolete metadata shape.
    check(SOURCE, 14);
}

#[test]
fn callable_entry_wrapper_avoids_user_names() {
    let source = SOURCE
        .replace(
            "fn main()",
            "@noinline fn __nuis_entry_main_0() -> i64 { return 3; }
@noinline fn __nuis_entry_main_1() -> i64 { return 5; }
fn main()",
        )
        .replace(
            "return acc;",
            "return acc + __nuis_entry_main_0() + __nuis_entry_main_1();",
        );
    check(&source, 22);
}

#[test]
fn repeated_carry_updates_execute_in_the_ordinary_entry() {
    let source = SOURCE.replace(
        "let acc: i64 = acc + value;",
        "let acc: i64 = acc + value; let acc: i64 = acc + 1;",
    );
    check(&source, 18);
}
