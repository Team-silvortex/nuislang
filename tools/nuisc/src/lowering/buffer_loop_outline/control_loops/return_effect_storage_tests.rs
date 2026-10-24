use super::*;
use crate::frontend::parse_nuis_module;

#[allow(dead_code)]
#[path = "../../../../tests/native_application_bridge/state_loop_return_shared_cases.rs"]
pub(super) mod cases;

pub(super) fn plans(
    source: &str,
    function: &str,
) -> (NirModule, control_values::CarryLayouts, ScalarHelpers) {
    let mut module = parse_nuis_module(source).unwrap();
    // Match the pipeline's source-declaration phase before strict call admission.
    let values = control_values::TypedLayouts::collect(&module);
    let pure = scalar_helpers::collect_capture_values(&module, &values);
    let mut names = module.functions.iter().map(|f| f.name.clone()).collect();
    conditional_values::outline_declarations(&mut module, &pure, &values, &mut names);
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    assert!(
        catalog.contains_key(function),
        "helper {function} was not admitted; admitted helpers: {:?}",
        catalog.keys().collect::<Vec<_>>()
    );
    (module, layouts, catalog)
}

#[test]
fn effect_return_storage_reuses_real_record_carries_without_pure_or_name_authority() {
    assert_eq!(cases::variants().len(), 8);
    for variant in cases::variants() {
        for name in ["carry", "working_copy", "__nuis_return_value_0"] {
            let source = cases::source(variant).replace("carry", name);
            let (module, layouts, catalog) = plans(&source, "work");
            let before = module.clone();
            let function = module.functions.iter().find(|f| f.name == "work").unwrap();
            let independent =
                rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
            let shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
            assert_eq!(shared.body.len() + 1, independent.body.len());
            assert_eq!(shared.signal.name(), independent.signal.name());
            assert!(verify_independent(function, &shared, &layouts).is_none());
            assert!(verify(function, &independent, &shared, name, &layouts, &catalog).is_some());
            assert_eq!(
                prepare_effects(function, &layouts, &catalog)
                    .unwrap()
                    .unwrap()
                    .body,
                shared.body
            );
            assert!(!scalar_helpers::collect_with_layouts(&module, &layouts).contains_key("work"));
            assert_eq!(module, before);
        }
    }
}

#[test]
fn effect_return_storage_revalidates_new_inner_writes_and_keeps_independent_fallback() {
    let source = include_str!(
        "../../../../tests/control_flow_syntax_native/scoped_return_typed_record_carries.ns"
    )
    .replace(
        "let j = j + 1;",
        "let j = j + 1; print(7); let snapshot = carry.right;",
    );
    let (module, layouts, catalog) = plans(&source, "step");
    let function = module.functions.iter().find(|f| f.name == "step").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let shared = rewrite(
        function,
        &layouts,
        Some("carry".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(verify_independent(function, &independent, &layouts).is_some());
    assert!(
        scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
            .is_none()
    );
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    assert!(coalesced_effects(function, &layouts, &catalog, &independent).is_none());
    assert_eq!(
        prepare_effects(function, &layouts, &catalog)
            .unwrap()
            .unwrap()
            .body,
        independent.body
    );
}

#[test]
fn effect_return_storage_never_uses_parameters_constants_or_noncarried_locals() {
    for body in [
        "let index = 0; while index < limit { index = index + 1; print(7); if index == 1 { return seed; } } return seed;",
        "const carry: State = seed; let index = 0; while index < limit { index = index + 1; print(7); if index == 1 { return carry; } } return carry;",
        "let carry = seed; let index = 0; while index < limit { index = index + 1; print(7); if index == 1 { return seed; } } return carry;",
    ] {
        let body = body.replace("let index = 0;", "let mut index = 0;");
        let source = format!("mod cpu Main {{ struct State {{ total: i64 }} fn work(seed: State, limit: i64) -> State {{ {body} }} fn main() -> i64 {{ return 0; }} }}");
        let (module, layouts, catalog) = plans(&source, "work");
        let function = module.functions.iter().find(|f| f.name == "work").unwrap();
        let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
        for storage in ["seed", "carry", "missing"] {
            let shared = rewrite(function, &layouts, Some(storage.into()), validation::LiteralPrints::Allow).unwrap();
            assert!(verify(function, &independent, &shared, storage, &layouts, &catalog).is_none());
        }
        assert_eq!(prepare_effects(function, &layouts, &catalog).unwrap().unwrap().body, independent.body);
    }
}

#[test]
fn effect_return_storage_rejects_pending_hoisting_and_unvalidated_source_reads() {
    let source = cases::source(cases::variants()[0]);
    let (module, layouts, catalog) = plans(&source, "work");
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let mut shared = coalesced_effects(function, &layouts, &catalog, &independent).unwrap();
    shared.body.insert(
        1,
        binding(shared.signal.name(), &scalar_type("i64"), NirExpr::Int(1)),
    );
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    let mut malformed = function.clone();
    malformed.body.push(NirStmt::Print(NirExpr::Int(99)));
    assert!(verify(
        &malformed,
        &independent,
        &shared,
        "carry",
        &layouts,
        &catalog
    )
    .is_none());
}

#[test]
fn effect_return_storage_does_not_grant_a_return_an_assignments_self_read() {
    let source = "mod cpu Main { struct State { total: i64 }
        fn work(seed: State, limit: i64) -> State {
            let mut carry = seed; let mut index = 0;
            while index < limit { index = index + 1; print(7);
                if index == 1 { return State { total: carry.total }; }
                carry = State { total: carry.total + 1 };
            } return carry;
        } fn main() -> i64 { return 0; } }";
    let module = parse_nuis_module(source).unwrap();
    let layouts = control_values::layouts(&module);
    let catalog = scalar_helpers::collect_iteration_calls(&module, &layouts);
    let function = module.functions.iter().find(|f| f.name == "work").unwrap();
    let independent = rewrite(function, &layouts, None, validation::LiteralPrints::Allow).unwrap();
    let shared = rewrite(
        function,
        &layouts,
        Some("carry".into()),
        validation::LiteralPrints::Allow,
    )
    .unwrap();
    assert!(scalar_helpers::validate_iteration_body(
        function,
        &independent.body,
        &catalog,
        &layouts
    )
    .is_none());
    assert!(
        scalar_helpers::validate_iteration_body(function, &shared.body, &catalog, &layouts)
            .is_some()
    );
    assert!(!catalog.contains_key("work"));
    assert!(verify(function, &independent, &shared, "carry", &layouts, &catalog).is_none());
    assert!(prepare_effects(function, &layouts, &catalog).is_none());
}
