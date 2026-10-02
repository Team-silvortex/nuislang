use super::*;
use crate::frontend::parse_nuis_module;

#[path = "../../../../tests/control_flow_syntax_native/scoped_index_recovery_cases.rs"]
mod fixture;

fn recovery_bindings(body: &[NirStmt], names: &mut BTreeSet<String>) {
    for stmt in body {
        match stmt {
            NirStmt::Let { name, .. } if name.starts_with("__nuis_advanced_index_") => {
                names.insert(name.clone());
            }
            NirStmt::If {
                then_body,
                else_body,
                ..
            } => {
                recovery_bindings(then_body, names);
                recovery_bindings(else_body, names);
            }
            NirStmt::While { body, .. } => recovery_bindings(body, names),
            _ => {}
        }
    }
}

#[test]
fn index_recovery_only_elides_unobserved_exits_and_keeps_helper_outputs() {
    for case in fixture::cases() {
        eprintln!("index recovery: {}", case.name);
        let mut module = parse_nuis_module(&fixture::source(&case.body, 3)).unwrap();
        let catalog =
            scalar_helpers::collect_with_layouts(&module, &control_values::layouts(&module));
        assert!(catalog.contains_key("work"), "{}", case.name);
        let outlined = outline_buffer_loops(&mut module).unwrap();
        assert!(!outlined.break_controls.is_empty(), "{}", case.name);
        let mut names = BTreeSet::new();
        for function in &module.functions {
            recovery_bindings(&function.body, &mut names);
        }
        assert_eq!(names.len(), case.recoveries, "{}", case.name);
        // The original leading step still executes, even when its exit value
        // is dead. Eliminating recovery is not eliminating source computation.
        assert!(module.functions.iter().any(|f| {
            f.name.starts_with("__nuis_scalar_iteration") && f.body.iter().any(|s| {
                matches!(s, NirStmt::Let { name, value: NirExpr::Binary { op: NirBinaryOp::Add, .. }, .. } if name == "index")
            })
        }), "{}", case.name);
    }
}

#[test]
fn index_recovery_matches_independent_results_across_zero_trips_and_backedges() {
    for case in fixture::cases() {
        for (limit, expected) in case.expected.into_iter().enumerate() {
            eprintln!("index recovery: {} limit={limit}", case.name);
            let mut yir = crate::pipeline::compile_source(&fixture::source(&case.body, limit))
                .unwrap()
                .yir;
            yir.nodes.reverse();
            yir.functions.reverse();
            for function in &mut yir.functions {
                function.body_nodes.reverse();
            }
            yir_verify::verify_module(&yir).unwrap();
            let trace = yir_runtime_host::execute_module_source_with_registry(
                &crate::render::render_yir(&yir),
                &yir_verify::default_registry(),
            )
            .unwrap();
            let entry = yir
                .functions
                .iter()
                .find(|f| f.role == yir_core::YirFunctionRole::Entry)
                .unwrap();
            assert_eq!(
                trace.values[&entry.result.as_ref().unwrap().node],
                yir_core::Value::Int(expected),
                "{} limit={limit}",
                case.name
            );
        }
    }
}
